use anyhow::{Context, Result};
use database::Database;
use oci_client::{
    client::ClientConfig, secrets::RegistryAuth, Client, Reference, RegistryOperation,
};
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
        let auth = self.resolve_auth(version_id).await?;

        let client = Client::new(ClientConfig::default());
        let reference: Reference = oci_reference.parse().context("Invalid OCI reference")?;

        // 0. Authenticate the client for this registry operation
        client
            .auth(&reference, &auth, RegistryOperation::Pull)
            .await
            .context("Failed to authenticate with GHCR")?;

        tracing::info!("Pulling manifest for {} from GHCR...", oci_reference);

        // 1. Pull the manifest
        let (manifest, _digest) = client
            .pull_manifest(&reference, &auth)
            .await
            .context("Failed to pull manifest from GHCR")?;

        let layers = match manifest {
            oci_client::manifest::OciManifest::Image(m) => m.layers,
            _ => anyhow::bail!("Unsupported manifest type (expected Image)"),
        };

        // 2. Find the WASM layer descriptor
        let wasm_layer_desc = layers
            .iter()
            .find(|l| {
                l.media_type == "application/vnd.w3c.wasm.component.v1+wasm"
                    || l.media_type == "application/wasm"
            })
            .context("No WASM layer found in OCI manifest")?;

        tracing::info!(
            "Found WASM layer: {} ({} bytes)",
            wasm_layer_desc.digest,
            wasm_layer_desc.size
        );

        // 3. Pull the specific WASM blob
        let mut blob = Vec::new();
        client
            .pull_blob(&reference, wasm_layer_desc, &mut blob)
            .await
            .context("Failed to pull WASM blob from GHCR")?;

        Ok(blob)
    }
}
