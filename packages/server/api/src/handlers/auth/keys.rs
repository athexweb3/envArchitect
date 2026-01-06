use crate::handlers::registry::ServiceError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use shared::keys;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateKeyPayload {
    pub name: String,
    pub scopes: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub key: String, // Returned ONCE
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ApiKeyDto {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_key(
    State(state): State<AppState>,
    // TODO: Require Session Auth (User) to create keys
    Json(payload): Json<CreateKeyPayload>,
) -> Result<Json<CreateKeyResponse>, ServiceError> {
    // 1. Generate Key
    let (raw_key, key_hash) = keys::generate_api_key(true);
    let prefix = keys::KEY_PREFIX_LIVE.to_string();

    // 2. Mock User Lookup
    let user_rec = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;
    let user_id = user_rec.0;

    // 3. Store
    let lookup_hash = shared::crypto::hash_token(&raw_key);

    let rec = sqlx::query_as::<_, (Uuid, chrono::DateTime<chrono::Utc>)>(
        r#"
        INSERT INTO api_keys (user_id, name, prefix, hash, scopes, lookup_hash)
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING id, created_at
        "#,
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&prefix)
    .bind(&key_hash)
    .bind(&payload.scopes)
    .bind(&lookup_hash)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    Ok(Json(CreateKeyResponse {
        id: rec.0,
        name: payload.name,
        prefix,
        key: raw_key,
    }))
}

pub async fn list_keys(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKeyDto>>, ServiceError> {
    let user_rec = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;
    let user_id = user_rec.0;

    let keys = sqlx::query_as::<_, ApiKeyDto>(
        r#"
        SELECT id, name, prefix, scopes, last_used_at, created_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    Ok(Json(keys))
}

pub async fn revoke_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(&state.db.pool)
        .await
        .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "revoked" })))
}

pub async fn register_signing_key(
    State(state): State<AppState>,
    axum::Extension(auth_user): axum::Extension<crate::middleware::auth::AuthUser>,
    Json(payload): Json<shared::dto::RegisterKeyRequest>,
) -> Result<Json<shared::dto::RegisterKeyResponse>, ServiceError> {
    use base64::{engine::general_purpose, Engine as _};

    let public_key_bytes = general_purpose::STANDARD
        .decode(&payload.public_key)
        .map_err(|_| ServiceError::BadRequest("Invalid base64 encoding".to_string()))?;

    if public_key_bytes.len() != 32 {
        return Err(ServiceError::BadRequest(
            "Invalid Ed25519 public key length (expected 32 bytes)".to_string(),
        ));
    }

    use ed25519_dalek::VerifyingKey;
    let key_bytes: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| ServiceError::BadRequest("Invalid key format".to_string()))?;

    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| ServiceError::BadRequest("Invalid Ed25519 public key".to_string()))?;

    sqlx::query(
        r#"
        UPDATE users
        SET signing_key = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&payload.public_key)
    .bind(auth_user.0.id)
    .execute(&state.db.pool)
    .await
    .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    Ok(Json(shared::dto::RegisterKeyResponse {
        success: true,
        message: "Signing key registered successfully".to_string(),
    }))
}
