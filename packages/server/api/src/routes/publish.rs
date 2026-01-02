use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use database::Database;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<Arc<Database>> {
    Router::new().route("/publish", post(publish_handler))
}

#[derive(Deserialize)]
struct PublishRequest {
    name: String,
    version: String,
    oci_ref: String,
    signature: String,
    description: Option<String>,
}

async fn publish_handler(
    State(db): State<Arc<Database>>,
    Json(payload): Json<PublishRequest>,
) -> Result<Json<Value>, StatusCode> {
    // 1. MOCK AUTH: Get or Create a Demo User
    // In production, this comes from the JWT Token claims
    let user_id = get_or_create_demo_user(&db).await.map_err(|e| {
        tracing::error!("Auth failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 2. Parsed Version (in real app, use semver crate)
    // For MVP, we naively parse "1.0.0"
    let parts: Vec<&str> = payload.version.split('.').collect();
    if parts.len() < 3 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let major: i32 = parts[0].parse().unwrap_or(0);
    let minor: i32 = parts[1].parse().unwrap_or(0);
    let patch: i32 = parts[2].parse().unwrap_or(0);

    // 3. Upsert Package (Idempotent)
    // If the package doesn't exist, create it under this user.
    // If it exists but owned by someone else, return FORBIDDEN.
    let row = sqlx::query(
        r#"
        INSERT INTO packages (name, owner_id, description)
        VALUES ($1, $2, $3)
        ON CONFLICT (name) DO UPDATE SET updated_at = NOW()
        RETURNING id, owner_id
        "#,
    )
    .bind(&payload.name)
    .bind(user_id)
    .bind(&payload.description)
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to upsert package: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let package_id: Uuid = row
        .try_get("id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let owner_id: Uuid = row
        .try_get("owner_id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if owner_id != user_id {
        return Err(StatusCode::FORBIDDEN);
    }

    // 4. Insert Version
    // If version exists, conflict error (Immutable versions)
    // 4. Insert Version
    // If version exists, conflict error (Immutable versions)
    let version_id: Uuid = sqlx::query(
        r#"
        INSERT INTO package_versions 
        (package_id, version_major, version_minor, version_patch, version_raw, oci_reference, integrity_hash, approval_status)
        VALUES ($1, $2, $3, $4, $5, $6, 'sha256:placeholder', 'PENDING')
        RETURNING id
        "#,
    )
    .bind(package_id)
    .bind(major)
    .bind(minor)
    .bind(patch)
    .bind(&payload.version)
    .bind(&payload.oci_ref)
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
         tracing::error!("Failed to insert version: {:?}", e);
         StatusCode::CONFLICT // Version likely exists
    })?
    .get("id");

    // 5. Insert Signature (Developer's Lock)
    // 5. Insert Signature (Developer's Lock)
    sqlx::query(
        r#"
        INSERT INTO signatures (version_id, signer_type, signer_id, signature_content)
        VALUES ($1, 'DEVELOPER', $2, $3)
        "#,
    )
    .bind(version_id)
    .bind(user_id)
    .bind(&payload.signature)
    .execute(&db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Published {} v{}", payload.name, payload.version);

    Ok(Json(json!({
        "status": "success",
        "plugin_id": package_id,
        "version_id": version_id,
        "message": "Registered successfully. Waiting for Notary approval."
    })))
}

// Helper for MVP
async fn get_or_create_demo_user(db: &Database) -> anyhow::Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (github_id, username, role)
        VALUES (101, 'demo_dev', 'admin')
        ON CONFLICT (github_id) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
    )
    .fetch_one(&db.pool)
    .await?;

    let id: Uuid = row.try_get("id")?;
    Ok(id)
}
