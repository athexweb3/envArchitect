// use crate::services::encryption;
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use database::models::User;
use database::repositories::{TokenRepository, UserRepository};
use database::Database;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use shared::dto::AuthDeviceResponse;
use shared::dto::TokenResponse;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    #[allow(dead_code)]
    db: Arc<Database>,
    user_repo: UserRepository,
    token_repo: TokenRepository,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl AuthService {
    pub fn new(db: Arc<Database>) -> Self {
        let user_repo = UserRepository::new(db.pool.clone());
        let token_repo = TokenRepository::new(db.pool.clone());
        Self {
            db,
            user_repo,
            token_repo,
        }
    }

    pub async fn poll_device_flow(&self, device_code: &str) -> Result<Option<TokenResponse>> {
        let code_opt = self.token_repo.find_device_code(device_code).await?;

        match code_opt {
            Some(c) if c.expires_at < Utc::now() => Err(anyhow!("Device code expired")),
            Some(c) if c.user_id.is_some() => {
                let user_id = c.user_id.unwrap();

                let access_token = self.generate_jwt(
                    user_id,
                    Duration::minutes(30),
                    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string()),
                )?;

                // Generate Refresh Token
                let rng = rand::thread_rng();
                let refresh_token_raw: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect();

                let hashed = shared::crypto::hash_token(&refresh_token_raw);
                let refresh_expires = Utc::now() + Duration::days(30);

                self.token_repo
                    .create_refresh_token(user_id, &hashed, refresh_expires)
                    .await?;

                self.token_repo.delete_device_code(device_code).await?;

                Ok(Some(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    refresh_token: Some(refresh_token_raw),
                    expires_in: 1800,
                }))
            }
            Some(_) => Ok(None),
            None => Err(anyhow!("Invalid device code")),
        }
    }

    pub async fn initiate_device_flow(&self, host: &str) -> Result<AuthDeviceResponse> {
        let device_code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let user_code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(|c| (c as char).to_ascii_uppercase())
            .collect();

        let expires_at = Utc::now() + Duration::minutes(15);

        self.token_repo
            .create_device_code(&device_code, &user_code, expires_at)
            .await?;

        Ok(AuthDeviceResponse {
            device_code,
            user_code: user_code.clone(),
            verification_uri: format!("{}/auth/verify", host),
            interval: 5,
            expires_in: 900,
        })
    }

    pub async fn authorize_device(&self, user_code: &str, user_id: Uuid) -> Result<()> {
        self.token_repo
            .authorize_device_code(user_code, user_id)
            .await
            .map_err(|e| anyhow!(e))
    }

    pub async fn authenticate_github(
        &self,
        github_id: i64,
        github_login: &str,
        email: Option<&str>,
        access_token: &str,
    ) -> Result<User> {
        let user_opt = self.user_repo.find_by_github_id(github_id).await?;

        let user = match user_opt {
            Some(u) => {
                self.user_repo
                    .update_github_token(u.id, access_token)
                    .await?;
                u
            }
            None => {
                self.user_repo
                    .create(github_id, github_login, email, access_token)
                    .await?
            }
        };

        Ok(user)
    }

    pub fn generate_jwt(
        &self,
        user_id: Uuid,
        duration: Duration,
        secret: String,
    ) -> Result<String> {
        let exp = (Utc::now() + duration).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;
        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string());
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    #[allow(dead_code)]
    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let hash = shared::crypto::hash_token(refresh_token);

        let user_id_opt = self.token_repo.find_refresh_token(&hash).await?;

        match user_id_opt {
            Some(uid) => {
                let access_token = self.generate_jwt(
                    uid,
                    Duration::minutes(30),
                    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret".to_string()),
                )?;

                Ok(TokenResponse {
                    access_token,
                    token_type: "Bearer".to_string(),
                    refresh_token: Some(refresh_token.to_string()),
                    expires_in: 1800,
                })
            }
            None => Err(anyhow!("Invalid or expired refresh token")),
        }
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        self.user_repo.find_by_id(id).await.map_err(|e| anyhow!(e))
    }

    pub async fn get_ghcr_token(&self, user_id: Uuid) -> Result<Option<String>> {
        self.user_repo
            .get_ghcr_pat(user_id)
            .await
            .map_err(|e| anyhow!(e))
    }

    pub async fn store_ghcr_pat(&self, user_id: Uuid, pat: &str) -> Result<()> {
        self.user_repo
            .update_ghcr_pat(user_id, pat)
            .await
            .map_err(|e| anyhow!(e))
    }

    #[allow(dead_code)]
    pub async fn verify_api_key(&self, token: &str) -> Result<User> {
        let lookup_hash = shared::crypto::hash_token(token);

        let record = sqlx::query!(
            r#"
            SELECT user_id, hash 
            FROM api_keys 
            WHERE lookup_hash = $1 
            LIMIT 1
            "#,
            lookup_hash
        )
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(rec) = record {
            if shared::keys::verify_key_hash(token, &rec.hash) {
                let _ = sqlx::query!(
                    "UPDATE api_keys SET last_used_at = NOW() WHERE lookup_hash = $1",
                    lookup_hash
                )
                .execute(&self.db.pool)
                .await;

                let user = self.user_repo.find_by_id(rec.user_id).await?;
                return user.ok_or_else(|| anyhow!("User not found"));
            }
        }

        Err(anyhow!("Invalid API key"))
    }
}
