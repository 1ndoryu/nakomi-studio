/* [044A-38 Fase 9] NotificationHub: gestión de conexiones WS de notificaciones por usuario.
 * Patrón similar a ChatHub pero indexado por user_id en vez de session_id.
 * Cada usuario autenticado puede conectarse vía WS y recibir notificaciones push.
 * El hub persiste en BD y broadcastea en tiempo real a todos los tabs del usuario. */

use std::sync::Arc;

use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateNotification, Notification, WsNotification};
use crate::repositories::NotificationRepository;

/* Capacidad del canal broadcast por usuario (búfer de mensajes no consumidos) */
const USER_CHANNEL_CAPACITY: usize = 32;

pub type UserNotifSender = broadcast::Sender<WsNotification>;

/// Hub central de notificaciones: broadcast channels indexados por `user_id`
#[derive(Clone)]
pub struct NotificationHub {
    pool: PgPool,
    /* user_id → broadcast::Sender para ese usuario */
    channels: Arc<DashMap<Uuid, UserNotifSender>>,
}

impl NotificationHub {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            channels: Arc::new(DashMap::new()),
        }
    }

    /// Obtiene o crea el canal broadcast para un usuario
    #[must_use]
    pub fn get_or_create_channel(&self, user_id: Uuid) -> UserNotifSender {
        self.channels
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(USER_CHANNEL_CAPACITY).0)
            .clone()
    }

    /// Suscribirse al canal de un usuario (para recibir notificaciones en WS)
    #[must_use]
    pub fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<WsNotification> {
        self.get_or_create_channel(user_id).subscribe()
    }

    /// Crea una notificación en BD y la emite por WS al usuario si está conectado
    pub async fn notify(&self, params: CreateNotification) -> Result<Notification, AppError> {
        let user_id = params.user_id;

        /* Persistir en BD */
        let notification = NotificationRepository::create(&self.pool, &params).await?;

        /* Broadcast al usuario si tiene canal abierto */
        let ws_msg: WsNotification = notification.clone().into();
        if let Some(sender) = self.channels.get(&user_id) {
            /* Ignorar error si nadie escucha (no hay receivers activos) */
            let _ = sender.send(ws_msg);
        }

        /* También enviar contador actualizado */
        self.send_unread_count(user_id).await;

        Ok(notification)
    }

    /// Envía el conteo actual de no leídas por WS
    pub async fn send_unread_count(&self, user_id: Uuid) {
        if let Some(sender) = self.channels.get(&user_id) {
            if let Ok(count) = NotificationRepository::count_unread(&self.pool, user_id).await {
                let _ = sender.send(WsNotification::UnreadCount { count });
            }
        }
    }

    /// Emite notificación a múltiples usuarios (ej: admins).
    /// Recibe un `CreateNotification` base y lo clona para cada usuario.
    pub async fn notify_many(
        &self,
        user_ids: &[Uuid],
        base: &CreateNotification,
    ) -> Result<Vec<Notification>, AppError> {
        let mut results = Vec::with_capacity(user_ids.len());

        for &uid in user_ids {
            let params = CreateNotification {
                user_id: uid,
                notification_type: base.notification_type.clone(),
                title: base.title.clone(),
                body: base.body.clone(),
                link: base.link.clone(),
                reference_type: base.reference_type.clone(),
                reference_id: base.reference_id,
            };
            match self.notify(params).await {
                Ok(n) => results.push(n),
                Err(e) => {
                    tracing::warn!("Error notificando a {uid}: {e}");
                }
            }
        }

        Ok(results)
    }

    /// Limpia canales de usuarios sin receivers activos (housekeeping)
    pub fn cleanup_empty_channels(&self) {
        self.channels
            .retain(|_user_id, sender| sender.receiver_count() > 0);
    }
}
