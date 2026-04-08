use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{User, UserRole};

/* [054A-1] Row intermedia para list_all con conteo total (paginación offset) */
pub struct UserWithTotal {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub active_role: Option<UserRole>,
    pub email_verified: bool,
    pub status: String,
    pub avatar_url: Option<String>,
    pub display_name: Option<String>,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub total_count: i64,
}

pub struct UserRepository;

impl UserRepository {
    /// Crea un usuario y retorna el registro completo
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let username = email.split('@').next().unwrap_or("user").to_lowercase();
        sqlx::query_as!(
            User,
            r#"INSERT INTO users (id, email, password_hash, username)
             VALUES ($1, $2, $3, $4)
             RETURNING id, email, password_hash,
                       role as "role: UserRole", active_role as "active_role: UserRole",
                       email_verified, status, avatar_url, display_name, username, created_at"#,
            id,
            email,
            password_hash,
            username,
        )
        .fetch_one(pool)
        .await
    }

    /// Busca un usuario por email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, password_hash,
                      role as "role: UserRole", active_role as "active_role: UserRole",
                      email_verified, status, avatar_url, display_name, username, created_at
             FROM users WHERE email = $1"#,
            email,
        )
        .fetch_optional(pool)
        .await
    }

    /// Busca un usuario por ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, password_hash,
                      role as "role: UserRole", active_role as "active_role: UserRole",
                      email_verified, status, avatar_url, display_name, username, created_at
             FROM users WHERE id = $1"#,
            id,
        )
        .fetch_optional(pool)
        .await
    }

    /* [044A-38] Actualiza el active_role de un admin para cambiar genuinamente de vista */
    pub async fn update_active_role(
        pool: &PgPool,
        user_id: Uuid,
        active_role: Option<UserRole>,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"UPDATE users SET active_role = $2
             WHERE id = $1
             RETURNING id, email, password_hash,
                       role as "role: UserRole", active_role as "active_role: UserRole",
                       email_verified, status, avatar_url, display_name, username, created_at"#,
            user_id,
            active_role as Option<UserRole>,
        )
        .fetch_one(pool)
        .await
    }

    /* [044A-43] Actualiza la URL del avatar del usuario */
    pub async fn update_avatar(
        pool: &PgPool,
        user_id: Uuid,
        avatar_url: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE users SET avatar_url = $2 WHERE id = $1", user_id, avatar_url)
            .execute(pool)
            .await?;
        Ok(())
    }

    /* [074A-23] Actualiza display_name en users + campos extendidos en user_profiles.
     * Upsert en user_profiles porque puede no existir row para el usuario. */
    pub async fn update_profile(
        pool: &PgPool,
        user_id: Uuid,
        display_name: Option<&str>,
        bio: Option<&str>,
        linkedin: Option<&str>,
        twitter: Option<&str>,
        website: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET display_name = COALESCE($2, display_name) WHERE id = $1",
            user_id,
            display_name,
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            r#"INSERT INTO user_profiles (user_id, bio, linkedin, twitter, website)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (user_id) DO UPDATE SET
               bio = COALESCE($2, user_profiles.bio),
               linkedin = COALESCE($3, user_profiles.linkedin),
               twitter = COALESCE($4, user_profiles.twitter),
               website = COALESCE($5, user_profiles.website),
               updated_at = NOW()"#,
            user_id,
            bio,
            linkedin,
            twitter,
            website,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /* [054A-1] Lista paginada de usuarios con búsqueda y filtros para panel admin.
     * Usa COUNT(*) OVER() para obtener el total sin query extra.
     * Filtros: role, status, búsqueda por email/display_name (ILIKE). */
    pub async fn list_all(
        pool: &PgPool,
        search: Option<&str>,
        role_filter: Option<UserRole>,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserWithTotal>, sqlx::Error> {
        let search_pattern = search.map(|s| format!("%{s}%"));
        let status_owned = status_filter.map(str::to_string);

        sqlx::query_as!(
            UserWithTotal,
            r#"SELECT id, email, password_hash,
                      role as "role: UserRole",
                      active_role as "active_role: UserRole",
                      email_verified, status, avatar_url, display_name, username, created_at,
                      COUNT(*) OVER() as "total_count!: i64"
             FROM users
             WHERE ($1::text IS NULL OR email ILIKE $1 OR display_name ILIKE $1)
               AND ($2::user_role IS NULL OR role = $2)
               AND ($3::text IS NULL OR status = $3)
             ORDER BY created_at DESC
             LIMIT $4 OFFSET $5"#,
            search_pattern as Option<String>,
            role_filter as Option<UserRole>,
            status_owned,
            limit,
            offset,
        )
        .fetch_all(pool)
        .await
    }

    /* [054A-1] Cambia el role real de un usuario (admin action).
     * No permite que un admin se cambie el rol a sí mismo por seguridad. */
    pub async fn update_role(
        pool: &PgPool,
        user_id: Uuid,
        new_role: UserRole,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"UPDATE users SET role = $2, active_role = NULL
             WHERE id = $1
             RETURNING id, email, password_hash,
                       role as "role: UserRole", active_role as "active_role: UserRole",
                       email_verified, status, avatar_url, display_name, username, created_at"#,
            user_id,
            new_role as UserRole,
        )
        .fetch_one(pool)
        .await
    }

    /* [054A-1] Cambia el status de un usuario (ban/reactivar).
     * status = 'active' | 'banned' | 'suspended' */
    pub async fn update_status(
        pool: &PgPool,
        user_id: Uuid,
        status: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"UPDATE users SET status = $2
             WHERE id = $1
             RETURNING id, email, password_hash,
                       role as "role: UserRole", active_role as "active_role: UserRole",
                       email_verified, status, avatar_url, display_name, username, created_at"#,
            user_id,
            status,
        )
        .fetch_one(pool)
        .await
    }

    /* [084A-1] Busca el primer usuario activo con un rol dado.
     * Usado por switch_role para encontrar un usuario real a impersonar.
     * [074A-59] Prioriza usuarios de fixtures (@test.com) para que switch-role
     * impersone al usuario que tiene datos de prueba, no un registro vacío. */
    pub async fn find_first_by_role(
        pool: &PgPool,
        role: UserRole,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, password_hash,
                      role as "role: UserRole", active_role as "active_role: UserRole",
                      email_verified, status, avatar_url, display_name, username, created_at
             FROM users
             WHERE role = $1 AND status = 'active'
             ORDER BY
                 CASE WHEN email LIKE '%@test.com' THEN 0 ELSE 1 END,
                 created_at DESC
             LIMIT 1"#,
            role as UserRole,
        )
        .fetch_optional(pool)
        .await
    }

    /* [T-6] IDs de todos los admins activos para notificaciones de escalación */
    pub async fn admin_ids(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
        let rows = sqlx::query_scalar!(
            r#"SELECT id FROM users WHERE role = 'admin' AND status = 'active'"#
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
