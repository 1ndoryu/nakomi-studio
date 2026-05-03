/* [074A-13] Repositorio de miembros del equipo.
 * CRUD completo con COALESCE para updates parciales. */
/* sentinel-disable-file sqlx-query-sin-macro sqlx-query-as-sin-macro: queries largas con COALESCE */

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::TeamMember;

pub struct TeamMemberRepository;

pub struct CreateTeamMemberParams<'a> {
    pub name: &'a str,
    pub slug: &'a str,
    pub role: &'a str,
    pub bio: &'a str,
    pub avatar: Option<&'a str>,
    pub linkedin: Option<&'a str>,
    pub twitter: Option<&'a str>,
    pub github: Option<&'a str>,
    pub status: &'a str,
    pub sort_order: i32,
}

pub struct UpdateTeamMemberParams<'a> {
    pub name: Option<&'a str>,
    pub slug: Option<&'a str>,
    pub role: Option<&'a str>,
    pub bio: Option<&'a str>,
    pub avatar: Option<&'a str>,
    pub linkedin: Option<&'a str>,
    pub twitter: Option<&'a str>,
    pub github: Option<&'a str>,
    pub status: Option<&'a str>,
    pub sort_order: Option<i32>,
}

impl TeamMemberRepository {
    pub async fn list_published(pool: &PgPool) -> Result<Vec<TeamMember>, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "SELECT id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at
             FROM team_members WHERE status = 'published' ORDER BY sort_order, name"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<TeamMember>, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "SELECT id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at
             FROM team_members ORDER BY sort_order, name"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TeamMember>, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "SELECT id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at
             FROM team_members WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_published_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<TeamMember>, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "SELECT id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at
             FROM team_members WHERE slug = $1 AND status = 'published'"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        params: CreateTeamMemberParams<'_>,
    ) -> Result<TeamMember, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "INSERT INTO team_members (name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at"
        )
        .bind(params.name)
        .bind(params.slug)
        .bind(params.role)
        .bind(params.bio)
        .bind(params.avatar)
        .bind(params.linkedin)
        .bind(params.twitter)
        .bind(params.github)
        .bind(params.status)
        .bind(params.sort_order)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        params: UpdateTeamMemberParams<'_>,
    ) -> Result<Option<TeamMember>, sqlx::Error> {
        sqlx::query_as::<_, TeamMember>(
            "UPDATE team_members SET
             name = COALESCE($2, name),
             slug = COALESCE($3, slug),
             role = COALESCE($4, role),
             bio = COALESCE($5, bio),
             avatar = COALESCE($6, avatar),
             linkedin = COALESCE($7, linkedin),
             twitter = COALESCE($8, twitter),
             github = COALESCE($9, github),
             status = COALESCE($10, status),
             sort_order = COALESCE($11, sort_order),
             updated_at = NOW()
             WHERE id = $1
             RETURNING id, name, slug, role, bio, avatar, linkedin, twitter, github, status, sort_order, created_at, updated_at"
        )
        .bind(id)
        .bind(params.name)
        .bind(params.slug)
        .bind(params.role)
        .bind(params.bio)
        .bind(params.avatar)
        .bind(params.linkedin)
        .bind(params.twitter)
        .bind(params.github)
        .bind(params.status)
        .bind(params.sort_order)
        .fetch_optional(pool)
        .await
    }

    pub async fn archive(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE team_members SET status = 'archived', updated_at = NOW() WHERE id = $1 AND status != 'archived'"
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /* [084A-10] Hard delete: elimina permanentemente el miembro de equipo */
    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM team_members WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
