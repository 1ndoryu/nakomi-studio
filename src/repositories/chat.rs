/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: chat usa runtime query_as
 * para soportar campos con #[sqlx(default)] (visitor_ip, visitor_user_agent). */
/* [044A-38 Fase 5] Repositorio de chat: CRUD sesiones y mensajes.
 * [064A-72] ChatSession queries migradas a runtime query_as para soportar
 * campos con #[sqlx(default)] (visitor_ip, visitor_user_agent).
 * Queries con prepared statements. Soporta sesiones anónimas y autenticadas. */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ChatMessage, ChatSession, ChatSessionNote};

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
               visitor_ip, visitor_user_agent \
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
               visitor_ip, visitor_user_agent \
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
}
