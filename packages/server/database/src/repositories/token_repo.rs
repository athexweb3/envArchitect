use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Result};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeviceCode {
    pub id: Uuid,
    pub device_code: String,
    pub user_code: String,
    pub user_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct TokenRepository {
    pool: PgPool,
}

impl TokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_device_code(
        &self,
        device_code: &str,
        user_code: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<DeviceCode> {
        sqlx::query_as::<_, DeviceCode>(
            r#"
            INSERT INTO device_codes (device_code, user_code, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(device_code)
        .bind(user_code)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_device_code(&self, device_code: &str) -> Result<Option<DeviceCode>> {
        sqlx::query_as::<_, DeviceCode>("SELECT * FROM device_codes WHERE device_code = $1")
            .bind(device_code)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn authorize_device_code(&self, user_code: &str, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE device_codes SET user_id = $1 WHERE user_code = $2")
            .bind(user_id)
            .bind(user_code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
