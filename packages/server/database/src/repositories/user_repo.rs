use crate::models::User;
use sqlx::{PgPool, Result};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_github_id(&self, github_id: i64) -> Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE github_id = $1")
            .bind(github_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        github_id: i64,
        username: &str,
        email: Option<&str>,
        github_token: &str,
    ) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (github_id, username, email, github_access_token)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(github_id)
        .bind(username)
        .bind(email)
        .bind(github_token)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_github_token(&self, id: uuid::Uuid, token: &str) -> Result<()> {
        sqlx::query("UPDATE users SET github_access_token = $1 WHERE id = $2")
            .bind(token)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_github_token(&self, id: uuid::Uuid) -> Result<Option<String>> {
        sqlx::query_scalar("SELECT github_access_token FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_ghcr_pat(&self, id: uuid::Uuid, token: &str) -> Result<()> {
        sqlx::query("UPDATE users SET ghcr_pat = $1 WHERE id = $2")
            .bind(token)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_ghcr_pat(&self, id: uuid::Uuid) -> Result<Option<String>> {
        sqlx::query_scalar("SELECT ghcr_pat FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}
