use crate::handlers::registry::ServiceError;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    extract::{Multipart, State},
    http::HeaderMap,
    Extension, Json,
};
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use shared::dto::{DependencyPayload as DependencyPayloadShared, PublishPayload};

// Alias for convenience if needed, or just use the shared one
type DependencyPayload = DependencyPayloadShared;

#[derive(Serialize)]
pub struct PublishResponse {
    pub id: Uuid,
    pub message: String,
}
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::jobs::Job;

pub async fn publish_plugin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<PublishResponse>, ServiceError> {
    // 1. Extract Parts (Metadata + File)
    let mut payload: Option<PublishPayload> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut sbom_bytes: Option<Vec<u8>> = None;

    while let Some(field) =
        multipart
            .next_field()
            .await
            .map_err(|e: axum::extract::multipart::MultipartError| {
                ServiceError::BadRequest(e.to_string())
            })?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "metadata" {
            let data =
                field
                    .bytes()
                    .await
                    .map_err(|e: axum::extract::multipart::MultipartError| {
                        ServiceError::BadRequest(e.to_string())
                    })?;
            payload =
                Some(serde_json::from_slice(&data).map_err(|e| {
                    ServiceError::BadRequest(format!("Invalid metadata JSON: {}", e))
                })?);
        } else if name == "file" || name == "artifact" {
            let data =
                field
                    .bytes()
                    .await
                    .map_err(|e: axum::extract::multipart::MultipartError| {
                        ServiceError::BadRequest(e.to_string())
                    })?;
            file_bytes = Some(data.to_vec());
        } else if name == "sbom" {
            let data =
                field
                    .bytes()
                    .await
                    .map_err(|e: axum::extract::multipart::MultipartError| {
                        ServiceError::BadRequest(e.to_string())
                    })?;
            sbom_bytes = Some(data.to_vec());
        }
    }

    let payload = payload.ok_or(ServiceError::BadRequest("Missing 'metadata' field".into()))?;
    let wasm_bytes = file_bytes.ok_or(ServiceError::BadRequest("Missing 'file' field".into()))?;

    tracing::info!("Publishing component: {}", payload.purl);

    // 2. Security: Verify Signature
    let signature_header = headers
        .get("X-Signature")
        .ok_or(ServiceError::BadRequest(
            "Missing X-Signature header".into(),
        ))?
        .to_str()
        .map_err(|_| ServiceError::BadRequest("Invalid X-Signature encoding".into()))?;

    let user_pub_key_str = auth_user.0.signing_key.ok_or(ServiceError::Forbidden(
        "User has no registered signing key".into(),
    ))?;

    // Decode Keys & Sig
    let public_key_bytes = general_purpose::STANDARD
        .decode(&user_pub_key_str)
        .map_err(|_| ServiceError::Forbidden("Invalid user public key format".into()))?;
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_header)
        .map_err(|_| ServiceError::BadRequest("Invalid signature format (base64)".into()))?;

    let verifying_key =
        VerifyingKey::from_bytes(public_key_bytes.as_slice().try_into().unwrap())
            .map_err(|_| ServiceError::Forbidden("Invalid user public key length".into()))?;
    let signature = Signature::from_bytes(signature_bytes.as_slice().try_into().unwrap());

    verifying_key.verify(&wasm_bytes, &signature).map_err(|e| {
        tracing::error!("Signature verification failed: {}", e);
        ServiceError::Forbidden("Invalid Artifact Signature".into())
    })?;

    // 3. Cognitive Ingestion (Search Index)
    let embedding_text = format!(
        "{} {} {}",
        payload.name,
        payload.ecosystem,
        payload.description.clone().unwrap_or_default()
    );
    let embedding_vec = state
        .search_engine
        .generate_embedding(&embedding_text)
        .unwrap_or_default(); // Fail safe

    // Upsert Package
    let package_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO packages (name, owner_id, description, embedding, score_quality, score_popularity, score_maintenance, score_authority, score_trending, created_at, updated_at)
        VALUES ($1, $2, $3, $4::real[]::vector, 0.5, 0.0, 0.9, 0.0, 0.0, NOW(), NOW())
        ON CONFLICT (name) DO UPDATE SET 
            description = EXCLUDED.description,
            embedding = EXCLUDED.embedding,
            updated_at = NOW()
        RETURNING id
        "#)
        .bind(&payload.name)
        .bind(auth_user.0.id)
        .bind(&payload.description)
        .bind(&embedding_vec)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

    // 4. Insert Component (Node)
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&wasm_bytes);
    let sha256_hash = format!("{:x}", hasher.finalize());
    let size_bytes = wasm_bytes.len() as i64;

    let source_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO components (purl, ecosystem, name, version, sha256, size_bytes)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (purl) DO UPDATE SET 
            created_at = NOW(),
            sha256 = EXCLUDED.sha256,
            size_bytes = EXCLUDED.size_bytes
        RETURNING id
        "#,
    )
    .bind(&payload.purl)
    .bind(&payload.ecosystem)
    .bind(&payload.name)
    .bind(&payload.version)
    .bind(&sha256_hash)
    .bind(size_bytes)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    // 4.5 Insert into package_versions (required for Notary Scan foreign key)
    let parsed_version = semver::Version::parse(&payload.version)
        .map_err(|_| ServiceError::BadRequest("Invalid semantic version".into()))?;

    let version_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO package_versions (package_id, version_major, version_minor, version_patch, version_prerelease, version_raw, oci_reference, integrity_hash, approval_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'PENDING')
        ON CONFLICT (package_id, version_major, version_minor, version_patch, version_prerelease) 
        DO UPDATE SET integrity_hash = EXCLUDED.integrity_hash
        RETURNING id
        "#
    )
    .bind(package_id)
    .bind(parsed_version.major as i32)
    .bind(parsed_version.minor as i32)
    .bind(parsed_version.patch as i32)
    .bind(if parsed_version.pre.is_empty() { None } else { Some(parsed_version.pre.as_str()) })
    .bind(&payload.version)
    .bind(format!("local://{}", source_id)) // Dummy OCI ref for now
    .bind(&sha256_hash)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

    // 5. Save Artifact (Storage)
    let storage_dir = std::path::Path::new("storage");
    if !storage_dir.exists() {
        std::fs::create_dir_all(storage_dir)
            .map_err(|e| ServiceError::InternalError(e.to_string()))?;
    }

    // If oci_reference is provided, we can optionally skip local storage or just keep it as cache.
    // For now, if we have oci_reference, we still save locally for Notary Scan ease,
    // but in production, we'd pull from GHCR.
    let filename = format!("{}.wasm", source_id);
    let filepath = storage_dir.join(&filename);
    std::fs::write(&filepath, &wasm_bytes)
        .map_err(|_| ServiceError::InternalError("Failed to save artifact".into()))?;

    // Save SBOM if provided
    if let Some(bytes) = sbom_bytes {
        let sbom_filename = format!("{}.sbom.json", source_id);
        let sbom_path = storage_dir.join(&sbom_filename);
        std::fs::write(&sbom_path, &bytes)
            .map_err(|_| ServiceError::InternalError("Failed to save SBOM".into()))?;
    }

    // Update package_version with OCI ref if provided
    if let Some(oci_ref) = &payload.oci_reference {
        sqlx::query!(
            "UPDATE package_versions SET oci_reference = $1 WHERE id = $2",
            oci_ref,
            version_id
        )
        .execute(&state.db.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    }

    // 6. Trigger Notary Scan

    // Yes, `state.worker_tx`.
    // We need to make sure `Job` is available.
    // Wait, `ScanPlugin` job needs `version_id`. `components` table is basically `versions`.
    // `source_id` IS `version_id`.
    let scan_job = Job::ScanPlugin {
        version_id, // Use the ID from package_versions
        name: payload.name.clone(),
        version: payload.version.clone(),
        bucket_key: filepath.to_string_lossy().to_string(), // Local path for now
    };

    if let Err(e) = state.worker_tx.send(scan_job).await {
        tracing::error!("Failed to queue Notary scan: {}", e);
        // Don't fail the upload, but log heavily.
    }

    // 7. Process Dependencies
    for dep in payload.dependencies {
        let (scan_ecosystem, scan_name) = parse_purl_meta(&dep.purl);
        let target_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO components (purl, ecosystem, name, version)
            VALUES ($1, $2, $3, 'external')
            ON CONFLICT (purl) DO UPDATE SET created_at = NOW()
            RETURNING id
            "#,
        )
        .bind(&dep.purl)
        .bind(&scan_ecosystem)
        .bind(&scan_name)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO dependencies (source_id, target_id, version_req, kind)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (source_id, target_id) DO NOTHING
            "#,
        )
        .bind(source_id)
        .bind(target_id)
        .bind(&dep.req)
        .bind(&dep.kind)
        .execute(&state.db.pool)
        .await
        .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;
    }

    Ok(Json(PublishResponse {
        id: source_id,
        message: "Published successfully. Artifact queued for Notary Scan.".to_string(),
    }))
}

fn parse_purl_meta(purl: &str) -> (String, String) {
    // pkg:ecosystem/name... or pkg:type/namespace/name
    if let Some(rest) = purl.strip_prefix("pkg:") {
        if let Some((eco, rest)) = rest.split_once('/') {
            let name_part = rest.split('@').next().unwrap_or(rest);
            return (eco.to_string(), name_part.to_string());
        }
    }
    ("unknown".to_string(), "unknown".to_string())
}
