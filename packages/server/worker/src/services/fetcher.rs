use anyhow::{Context, Result};
use database::Database;
use oci_client::{client::ClientConfig, secrets::RegistryAuth, Client, Reference};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait ArtifactFetcher: Send + Sync {
    async fn fetch(&self, oci_reference: &str, version_id: Uuid) -> Result<Vec<u8>>;
}

pub struct GhcrFetcher {
    db: Arc<Database>,
}

impl GhcrFetcher {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Resolves the owner of the package version and checks for a PAT.
    async fn resolve_auth(&self, version_id: Uuid) -> Result<RegistryAuth> {
        let rec = sqlx::query_as::<_, (Option<String>,)>(
            r#"
            SELECT u.ghcr_pat
            FROM users u
            JOIN packages p ON p.owner_id = u.id
            JOIN package_versions pv ON pv.package_id = p.id
            WHERE pv.id = $1
            "#,
        )
        .bind(version_id)
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(row) = rec {
            if let Some(_pat) = row.0 {
                // We assume the PAT belongs to the user who owns the package.
                // For GHCR, username is often required. We might need to select username too.
                // But oci-distribution often accepts just the token as bearer for some registries,
                // or we use "oauth2" / "basic".
                // GitHub usually treats the username as the "user" and PAT as "password" for Basic auth.

                // Let's refetch with username.
                let user_rec = sqlx::query_as::<_, (String, Option<String>)>(
                    r#"
                    SELECT u.username, u.ghcr_pat
                    FROM users u
                    JOIN packages p ON p.owner_id = u.id
                    JOIN package_versions pv ON pv.package_id = p.id
                    WHERE pv.id = $1
                    "#,
                )
                .bind(version_id)
                .fetch_one(&self.db.pool)
                .await?;

                let username = user_rec.0;
                let pat = user_rec.1;

                if let Some(token) = pat {
                    tracing::info!("Found PAT for user {}, using for GHCR auth", username);
                    return Ok(RegistryAuth::Basic(username, token));
                }
            }
        }

        tracing::info!(
            "No PAT found for version_id {}. Attempting anonymous pull.",
            version_id
        );
        Ok(RegistryAuth::Anonymous)
    }
}

#[async_trait::async_trait]
impl ArtifactFetcher for GhcrFetcher {
    async fn fetch(&self, oci_reference: &str, version_id: Uuid) -> Result<Vec<u8>> {
        if let Some(path_suffix) = oci_reference.strip_prefix("local://") {
            tracing::info!("Reading {} from local storage...", oci_reference);
            let filename = format!("{}.wasm", path_suffix);
            let filepath = std::path::Path::new("../../../storage").join(filename);
            let data = std::fs::read(&filepath)
                .with_context(|| format!("Failed to read local artifact at {:?}", filepath))?;
            return Ok(data);
        }

        let auth = self.resolve_auth(version_id).await?;

        let client = Client::new(ClientConfig::default());
        let reference: Reference = oci_reference.parse().context("Invalid OCI reference")?;

        tracing::info!("Pulling {} from GHCR...", oci_reference);

        // 1. Pull the image (manifest + layers)
        let image = client
            .pull(
                &reference,
                &auth,
                vec![
                    "application/vnd.w3c.wasm.component.v1+wasm",
                    "application/wasm",
                ],
            )
            .await
            .context("Failed to pull image from GHCR")?;

        // Find the layer with WASM media type
        let wasm_layer = image
            .layers
            .iter()
            .find(|l| {
                l.media_type == "application/vnd.w3c.wasm.component.v1+wasm"
                    || l.media_type == "application/wasm"
            })
            .context("No WASM layer found in OCI image")?;

        tracing::info!("Extracted WASM layer: {} bytes", wasm_layer.data.len());

        Ok(wasm_layer.data.clone())
    }
}
