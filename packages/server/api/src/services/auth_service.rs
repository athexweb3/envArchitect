use crate::services::encryption;
use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use database::models::User;
use database::repositories::token_repo::DeviceCode;
use database::repositories::{TokenRepository, UserRepository};
use database::Database;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use shared::dto::AuthDeviceResponse;
use shared::dto::TokenResponse;
use std::sync::Arc;

pub struct AuthService {
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
        Self {
            db: db.clone(),
            user_repo: UserRepository::new(db.pool.clone()),
            token_repo: TokenRepository::new(db.pool.clone()),
        }
    }

    pub async fn poll_device_flow(
        &self,
        device_code: &str,
    ) -> Result<Option<TokenResponse>, Error> {
        let code_res: Result<Option<DeviceCode>, database::sqlx::Error> =
            self.token_repo.find_device_code(device_code).await;
        let code = code_res.map_err(|e| Error::new(e))?;

        match code {
            Some(c) if c.expires_at < Utc::now() => Err(anyhow::anyhow!("Device code expired")),
            Some(c) if c.user_id.is_some() => {
                let user_id = c.user_id.unwrap();

                // 1. Generate Access Token (JWT)
                let expiration = Utc::now() + Duration::hours(1);
                let claims = Claims {
                    sub: user_id.to_string(),
                    exp: expiration.timestamp() as usize,
                };

                let access_token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret("placeholder_secret".as_ref()),
                )?;

                // 2. Generate Refresh Token
                let refresh_token_raw: String = rand::rng()
                    .sample_iter(&Alphanumeric)
                    .take(64)
                    .map(char::from)
                    .collect();

                let refresh_expires = Utc::now() + Duration::days(30);
                let hashed = shared::crypto::hash_token(&refresh_token_raw);

                let _ = self
                    .token_repo
                    .create_refresh_token(user_id, &hashed, refresh_expires)
                    .await
                    .map_err(|e| Error::new(e))?;

                Ok(Some(TokenResponse {
                    access_token,
                    refresh_token: Some(refresh_token_raw),
                    expires_in: 3600,
                    token_type: "Bearer".to_string(),
                }))
            }
            _ => Ok(None),
        }
    }

    pub async fn initiate_device_flow(&self, host: &str) -> Result<AuthDeviceResponse, Error> {
        let device_code: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let user_code: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .map(|c: char| c.to_ascii_uppercase())
            .collect();

        let expires_in = 600;
        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);

        let _ = self
            .token_repo
            .create_device_code(&device_code, &user_code, expires_at)
            .await
            .map_err(|e| Error::new(e))?;

        Ok(AuthDeviceResponse {
            device_code,
            user_code,
            verification_uri: format!("{}/auth/portal", host),
            expires_in: expires_in as u64,
            interval: 5,
        })
    }

    pub async fn authorize_device(
        &self,
        user_code: &str,
        user_id: uuid::Uuid,
    ) -> Result<(), Error> {
        let _ = self
            .token_repo
            .authorize_device_code(user_code, user_id)
            .await
            .map_err(|e| Error::new(e))?;
        Ok(())
    }

    pub async fn authenticate_github(
        &self,
        github_id: i64,
        username: &str,
        email: Option<&str>,
        github_token: &str,
    ) -> Result<User, Error> {
        let encrypted_token = encryption::encrypt(github_token)?;

        let user_opt_res: Result<Option<User>, database::sqlx::Error> =
            self.user_repo.find_by_github_id(github_id).await;
        let user_opt = user_opt_res.map_err(|e| Error::new(e))?;

        if let Some(user) = user_opt {
            // Update token
            self.user_repo
                .update_github_token(user.id, &encrypted_token)
                .await
                .map_err(|e| Error::new(e))?;
            Ok(user)
        } else {
            let u = self
                .user_repo
                .create(github_id, username, email, &encrypted_token)
                .await
                .map_err(|e| Error::new(e))?;
            Ok(u)
        }
    }

    pub async fn get_ghcr_token(&self, user_id: uuid::Uuid) -> Result<Option<String>, Error> {
        // Fallback: Check for GHCR PAT first, then GitHub Access Token
        if let Some(pat) = self.user_repo.get_ghcr_pat(user_id).await? {
            return Ok(Some(encryption::decrypt(&pat)?));
        }

        if let Some(token) = self.user_repo.get_github_token(user_id).await? {
            // Handle plain text migration safely
            match encryption::decrypt(&token) {
                Ok(decrypted) => return Ok(Some(decrypted)),
                Err(e) => {
                    tracing::warn!("Failed to decrypt token for user {}: {}", user_id, e);
                    // If it's 40 chars and hex-decodable, it might be a plain PAT (not recommended but for dev transition)
                    // Better to just return None and force re-auth
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub async fn store_ghcr_pat(&self, user_id: uuid::Uuid, pat: &str) -> Result<(), Error> {
        let encrypted = encryption::encrypt(pat)?;
        self.user_repo.update_ghcr_pat(user_id, &encrypted).await?;
        Ok(())
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, Error> {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret("placeholder_secret".as_ref()),
            &Validation::default(),
        )
        .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

        Ok(token_data.claims)
    }

    pub async fn find_user_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, Error> {
        let user = self
            .user_repo
            .find_by_id(id)
            .await
            .map_err(|e| Error::new(e))?;
        Ok(user)
    }
    pub async fn verify_api_key(&self, token: &str) -> Result<User, Error> {
        // 1. Compute Lookup Hash (SHA256)
        let lookup_hash = shared::crypto::hash_token(token);

        // 2. Find Key
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
        .await
        .map_err(|e| Error::new(e))?;

        if let Some(rec) = record {
            // 3. Verify Argon2 Hash (Prevent Timing/Pre-image attacks)
            if shared::keys::verify_key_hash(token, &rec.hash) {
                // 4. Update usage stats (Async fire-and-forget ideally)
                let _ = sqlx::query!(
                    "UPDATE api_keys SET last_used_at = NOW() WHERE lookup_hash = $1",
                    lookup_hash
                )
                .execute(&self.db.pool)
                .await;

                // 5. Fetch User
                if let Some(user) = self.find_user_by_id(rec.user_id).await? {
                    return Ok(user);
                }
            }
        }

        Err(anyhow::anyhow!("Invalid API Key"))
    }
}
