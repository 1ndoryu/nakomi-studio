/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: chat usa runtime query_as
 * para soportar campos con #[sqlx(default)] (visitor_ip, visitor_user_agent). */
/* [044A-38 Fase 5] Repositorio de chat: CRUD sesiones y mensajes.
 * [064A-72] ChatSession queries migradas a runtime query_as para soportar
 * campos con #[sqlx(default)] (visitor_ip, visitor_user_agent).
 * Queries con prepared statements. Soporta sesiones anónimas y autenticadas. */

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{ChatAttachment, ChatMessage, ChatSession, ChatSessionNote, VisitorProfile};

pub struct ChatRepository;

impl ChatRepository {
    /* ============================================================
       SESIONES
       ============================================================ */

    /// Crear sesión de chat (pre-venta o vinculada a orden)
    pub async fn create_session(
        pool: &PgPool,
        visitor_id: Option<&str>,
        visitor_name: Option<&str>,
        user_id: Option<Uuid>,
        order_id: Option<Uuid>,
    ) -> Result<ChatSession, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "INSERT INTO chat_sessions (visitor_id, visitor_name, user_id, order_id) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at",
        )
        .bind(visitor_id)
        .bind(visitor_name)
        .bind(user_id)
        .bind(order_id)
        .fetch_one(pool)
        .await
    }

    pub async fn find_session_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<ChatSession>, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "SELECT id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at, \
               visitor_ip, visitor_user_agent, last_viewed_at, visitor_last_connected_at \
             FROM chat_sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Sesiones activas para un usuario autenticado
    pub async fn list_sessions_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ChatSession>, sqlx::Error> {
        /* [074A-30] Filtrar sesiones sin mensajes */
        sqlx::query_as::<_, ChatSession>(
            "SELECT id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at \
             FROM chat_sessions \
             WHERE (user_id = $1 OR assigned_staff_id = $1) \
             AND status != 'closed' \
             AND EXISTS (SELECT 1 FROM chat_messages WHERE session_id = chat_sessions.id) \
             ORDER BY updated_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Sesión activa por orden (para evitar duplicados)
    pub async fn find_session_by_order(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<ChatSession>, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "SELECT id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at \
             FROM chat_sessions \
             WHERE order_id = $1 AND status != 'closed' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await
    }

    /// Sesión activa por `visitor_id` (anónimo)
    pub async fn find_session_by_visitor(
        pool: &PgPool,
        visitor_id: &str,
    ) -> Result<Option<ChatSession>, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "SELECT id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at, \
               visitor_ip, visitor_user_agent \
             FROM chat_sessions \
             WHERE visitor_id = $1 AND status != 'closed' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(visitor_id)
        .fetch_optional(pool)
        .await
    }

    /// Todas las sesiones activas (panel staff)
    pub async fn list_active_sessions(
        pool: &PgPool,
    ) -> Result<Vec<ChatSession>, sqlx::Error> {
        /* [074A-30] Filtrar sesiones sin mensajes — no tiene sentido mostrarlas */
        sqlx::query_as::<_, ChatSession>(
            "SELECT id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at, \
               visitor_ip, visitor_user_agent, last_viewed_at, visitor_last_connected_at \
             FROM chat_sessions \
             WHERE status != 'closed' \
             AND EXISTS (SELECT 1 FROM chat_messages WHERE session_id = chat_sessions.id) \
             ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// Staff toma una sesión
    pub async fn assign_staff(
        pool: &PgPool,
        session_id: Uuid,
        staff_id: Uuid,
    ) -> Result<ChatSession, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "UPDATE chat_sessions SET assigned_staff_id = $2, \
             status = 'staff_handling', ai_enabled = false, \
             updated_at = NOW() WHERE id = $1 \
             RETURNING id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at",
        )
        .bind(session_id)
        .bind(staff_id)
        .fetch_one(pool)
        .await
    }

    /// Toggle IA en una sesión
    pub async fn toggle_ai(
        pool: &PgPool,
        session_id: Uuid,
        enabled: bool,
    ) -> Result<ChatSession, sqlx::Error> {
        let new_status = if enabled { "ai_handling" } else { "staff_handling" };
        sqlx::query_as::<_, ChatSession>(
            "UPDATE chat_sessions SET ai_enabled = $2, status = $3, \
             updated_at = NOW() WHERE id = $1 \
             RETURNING id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at",
        )
        .bind(session_id)
        .bind(enabled)
        .bind(new_status)
        .fetch_one(pool)
        .await
    }

    /// Cerrar sesión
    pub async fn close_session(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<ChatSession, sqlx::Error> {
        sqlx::query_as::<_, ChatSession>(
            "UPDATE chat_sessions SET status = 'closed', \
             updated_at = NOW() WHERE id = $1 \
             RETURNING id, visitor_id, visitor_name, user_id, order_id, status, \
               assigned_staff_id, ai_enabled, created_at, updated_at",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
    }

    /* [104A-39] Marcar sesión como vista por staff — actualiza last_viewed_at = NOW().
     * Permite que el badge de ChatBell solo cuente sesiones con mensajes no leídos. */
    pub async fn mark_session_viewed(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_sessions SET last_viewed_at = NOW() WHERE id = $1",
        )
        .bind(session_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [104A-40] Actualizar timestamp de última conexión WS del visitante.
     * Llamado en ws_visitor.rs al conectar. Devuelve el timestamp actualizado
     * para poder brodcastearlo inmediatamente al canal de staff. */
    pub async fn update_visitor_last_connected(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<DateTime<Utc>, sqlx::Error> {
        let row: (DateTime<Utc>,) = sqlx::query_as(
            "UPDATE chat_sessions SET visitor_last_connected_at = NOW() \
             WHERE id = $1 RETURNING visitor_last_connected_at",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /* [084A-40] Borrar todos los mensajes de una sesión (usado por /reset) */
    pub async fn delete_session_messages(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM chat_messages WHERE session_id = $1",
        )
        .bind(session_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /* [084A-40] Borrar perfil del visitante (usado por /reset para limpiar
     * context_summary, preferences, sesiones acumuladas, etc.) */
    pub async fn delete_visitor_profile(
        pool: &PgPool,
        visitor_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM visitor_profiles WHERE visitor_id = $1",
        )
        .bind(visitor_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /* ============================================================
       MENSAJES
       ============================================================ */

    /// Guardar mensaje en BD
    pub async fn save_message(
        pool: &PgPool,
        session_id: Uuid,
        sender_type: &str,
        sender_id: Option<&str>,
        content: &str,
    ) -> Result<ChatMessage, sqlx::Error> {
        /* Actualizar timestamp de sesión al recibir mensaje */
        let _ = sqlx::query!(
            r#"UPDATE chat_sessions SET updated_at = NOW() WHERE id = $1"#,
            session_id,
        )
        .execute(pool)
        .await;

        sqlx::query_as!(
            ChatMessage,
            r#"INSERT INTO chat_messages (session_id, sender_type, sender_id, content)
             VALUES ($1, $2, $3, $4)
             RETURNING id, session_id, sender_type, sender_id, content, created_at,
                       message_type, metadata"#,
            session_id,
            sender_type,
            sender_id,
            content,
        )
        .fetch_one(pool)
        .await
    }

    /* [T-2] Guardar mensaje rico con tipo y metadatos estructurados.
     * Usado para service_cards, invoices, order_cards, etc. generados por tool use. */
    pub async fn save_rich_message(
        pool: &PgPool,
        session_id: Uuid,
        sender_type: &str,
        sender_id: Option<&str>,
        content: &str,
        message_type: &str,
        metadata: &serde_json::Value,
    ) -> Result<ChatMessage, sqlx::Error> {
        let _ = sqlx::query!(
            r#"UPDATE chat_sessions SET updated_at = NOW() WHERE id = $1"#,
            session_id,
        )
        .execute(pool)
        .await;

        sqlx::query_as!(
            ChatMessage,
            r#"INSERT INTO chat_messages (session_id, sender_type, sender_id, content, message_type, metadata)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, session_id, sender_type, sender_id, content, created_at,
                       message_type, metadata"#,
            session_id,
            sender_type,
            sender_id,
            content,
            message_type,
            metadata,
        )
        .fetch_one(pool)
        .await
    }

    /// Historial de mensajes de una sesión (paginado)
    pub async fn list_messages(
        pool: &PgPool,
        session_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ChatMessage>, sqlx::Error> {
        sqlx::query_as!(
            ChatMessage,
            r#"SELECT id, session_id, sender_type, sender_id, content, created_at,
                      message_type, metadata
             FROM chat_messages WHERE session_id = $1
             ORDER BY created_at ASC LIMIT $2 OFFSET $3"#,
            session_id,
            limit,
            offset,
        )
        .fetch_all(pool)
        .await
    }

    /// Último mensaje de múltiples sesiones (para preview en lista)
    pub async fn last_messages_for_sessions(
        pool: &PgPool,
        session_ids: &[Uuid],
    ) -> Result<Vec<ChatMessage>, sqlx::Error> {
        sqlx::query_as!(
            ChatMessage,
            r#"SELECT DISTINCT ON (session_id)
               id, session_id, sender_type, sender_id, content, created_at,
               message_type, metadata
             FROM chat_messages
             WHERE session_id = ANY($1)
             ORDER BY session_id, created_at DESC"#,
            session_ids,
        )
        .fetch_all(pool)
        .await
    }

    /* ============================================================
       TYPING
       ============================================================ */

    /// Actualizar typing preview
    pub async fn update_typing(
        pool: &PgPool,
        session_id: Uuid,
        content: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO chat_typing (session_id, content, updated_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (session_id) DO UPDATE SET content = $2, updated_at = NOW()"#,
            session_id,
            content,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /* ============================================================
       NOTAS DE SESIÓN (064A-72)
       ============================================================ */

    /// Listar notas de una sesión
    pub async fn list_session_notes(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<Vec<ChatSessionNote>, sqlx::Error> {
        sqlx::query_as::<_, ChatSessionNote>(
            "SELECT id, session_id, author_id, content, created_at \
             FROM chat_session_notes WHERE session_id = $1 \
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
    }

    /// Crear nota en una sesión
    pub async fn create_session_note(
        pool: &PgPool,
        session_id: Uuid,
        author_id: Uuid,
        content: &str,
    ) -> Result<ChatSessionNote, sqlx::Error> {
        sqlx::query_as::<_, ChatSessionNote>(
            "INSERT INTO chat_session_notes (session_id, author_id, content) \
             VALUES ($1, $2, $3) \
             RETURNING id, session_id, author_id, content, created_at",
        )
        .bind(session_id)
        .bind(author_id)
        .bind(content)
        .fetch_one(pool)
        .await
    }

    /// Renombrar visitante de una sesión
    pub async fn update_visitor_name(
        pool: &PgPool,
        session_id: Uuid,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_sessions SET visitor_name = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(session_id)
        .bind(name)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* ============================================================
       VISITOR PROFILES (T-3 — Memoria usuario + contexto)
       ============================================================ */

    /* [T-3] Buscar perfil por visitor_id (localStorage UUID del visitante) */
    pub async fn find_visitor_profile(
        pool: &PgPool,
        visitor_id: &str,
    ) -> Result<Option<VisitorProfile>, sqlx::Error> {
        sqlx::query_as::<_, VisitorProfile>(
            "SELECT id, visitor_id, email, user_id, display_name, context_summary, \
               preferences, first_seen_at, last_seen_at, total_sessions, \
               ip_addresses, device_fingerprints \
             FROM visitor_profiles WHERE visitor_id = $1",
        )
        .bind(visitor_id)
        .fetch_optional(pool)
        .await
    }

    /* [T-3] Upsert: crea o actualiza perfil al conectar WS.
     * Usa ON CONFLICT para atomicidad (evita race conditions).
     * Agrega IP y fingerprint solo si no existen ya en el array. */
    pub async fn upsert_visitor_profile(
        pool: &PgPool,
        visitor_id: &str,
        ip: Option<&str>,
        device_fingerprint: Option<&str>,
    ) -> Result<VisitorProfile, sqlx::Error> {
        sqlx::query_as::<_, VisitorProfile>(
            "INSERT INTO visitor_profiles (visitor_id, ip_addresses, device_fingerprints) \
             VALUES ($1, \
               CASE WHEN $2::text IS NOT NULL THEN ARRAY[$2::text] ELSE '{}'::text[] END, \
               CASE WHEN $3::text IS NOT NULL THEN ARRAY[$3::text] ELSE '{}'::text[] END) \
             ON CONFLICT (visitor_id) DO UPDATE SET \
               last_seen_at = NOW(), \
               total_sessions = visitor_profiles.total_sessions + 1, \
               ip_addresses = CASE WHEN $2::text IS NOT NULL AND NOT ($2::text = ANY(visitor_profiles.ip_addresses)) \
                 THEN array_append(visitor_profiles.ip_addresses, $2::text) \
                 ELSE visitor_profiles.ip_addresses END, \
               device_fingerprints = CASE WHEN $3::text IS NOT NULL AND NOT ($3::text = ANY(visitor_profiles.device_fingerprints)) \
                 THEN array_append(visitor_profiles.device_fingerprints, $3::text) \
                 ELSE visitor_profiles.device_fingerprints END \
             RETURNING id, visitor_id, email, user_id, display_name, context_summary, \
               preferences, first_seen_at, last_seen_at, total_sessions, \
               ip_addresses, device_fingerprints",
        )
        .bind(visitor_id)
        .bind(ip)
        .bind(device_fingerprint)
        .fetch_one(pool)
        .await
    }

    /* [T-3] Capturar email del visitante (tool call capture_email).
     * También actualiza display_name si se proporciona. */
    pub async fn update_visitor_email(
        pool: &PgPool,
        visitor_id: &str,
        email: &str,
        display_name: Option<&str>,
    ) -> Result<VisitorProfile, sqlx::Error> {
        sqlx::query_as::<_, VisitorProfile>(
            "UPDATE visitor_profiles SET \
               email = $2, \
               display_name = COALESCE($3, display_name), \
               last_seen_at = NOW() \
             WHERE visitor_id = $1 \
             RETURNING id, visitor_id, email, user_id, display_name, context_summary, \
               preferences, first_seen_at, last_seen_at, total_sessions, \
               ip_addresses, device_fingerprints",
        )
        .bind(visitor_id)
        .bind(email)
        .bind(display_name)
        .fetch_one(pool)
        .await
    }

    /* [T-3] Actualizar resumen de contexto (generado por IA al cerrar sesión) */
    pub async fn update_context_summary(
        pool: &PgPool,
        visitor_id: &str,
        summary: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE visitor_profiles SET context_summary = $2, last_seen_at = NOW() \
             WHERE visitor_id = $1",
        )
        .bind(visitor_id)
        .bind(summary)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [T-3] Actualizar preferencias extraídas de la conversación (JSON merge) */
    pub async fn update_visitor_preferences(
        pool: &PgPool,
        visitor_id: &str,
        preferences: &serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE visitor_profiles SET \
               preferences = COALESCE(preferences, '{}'::jsonb) || $2::jsonb, \
               last_seen_at = NOW() \
             WHERE visitor_id = $1",
        )
        .bind(visitor_id)
        .bind(preferences)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [T-3] Vincular visitante con usuario registrado */
    pub async fn link_visitor_to_user(
        pool: &PgPool,
        visitor_id: &str,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE visitor_profiles SET user_id = $2, last_seen_at = NOW() \
             WHERE visitor_id = $1",
        )
        .bind(visitor_id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* ============================================================
       ATTACHMENTS (T-5 — Archivos en chat)
       ============================================================ */

    /* [T-5] Guardar adjunto vinculado a un mensaje de chat.
     * ai_description se actualiza después con el resultado de Vision/Whisper/PDF. */
    pub async fn save_attachment(
        pool: &PgPool,
        message_id: Uuid,
        file_name: &str,
        file_path: &str,
        mime_type: &str,
        file_size_bytes: i64,
    ) -> Result<ChatAttachment, sqlx::Error> {
        sqlx::query_as::<_, ChatAttachment>(
            "INSERT INTO chat_attachments (message_id, file_name, file_path, mime_type, file_size_bytes) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, message_id, file_name, file_path, mime_type, file_size_bytes, ai_description, created_at",
        )
        .bind(message_id)
        .bind(file_name)
        .bind(file_path)
        .bind(mime_type)
        .bind(file_size_bytes)
        .fetch_one(pool)
        .await
    }

    /* [T-5] Actualizar descripción generada por IA (Vision, Whisper, PDF extract) */
    pub async fn update_attachment_description(
        pool: &PgPool,
        attachment_id: Uuid,
        description: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE chat_attachments SET ai_description = $2 WHERE id = $1",
        )
        .bind(attachment_id)
        .bind(description)
        .execute(pool)
        .await?;
        Ok(())
    }

    /* [T-5] Listar adjuntos de un mensaje */
    pub async fn list_attachments_for_message(
        pool: &PgPool,
        message_id: Uuid,
    ) -> Result<Vec<ChatAttachment>, sqlx::Error> {
        sqlx::query_as::<_, ChatAttachment>(
            "SELECT id, message_id, file_name, file_path, mime_type, file_size_bytes, ai_description, created_at \
             FROM chat_attachments WHERE message_id = $1 ORDER BY created_at ASC",
        )
        .bind(message_id)
        .fetch_all(pool)
        .await
    }
}
