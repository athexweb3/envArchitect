use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tough::{DefaultTransport, RepositoryLoader, TargetName};
use url::Url;

/// Verifies the integrity of the plugin registry using The Update Framework (TUF).
/// Ensures that metadata is fresh, signed, and targets (plugins) match their hashes.
pub struct RepositoryVerifier {
    metadata_base_url: Url,
    targets_base_url: Url,
    root_json_path: PathBuf,
    cache_dir: PathBuf,
}

impl RepositoryVerifier {
    /// Initialize a TUF repository verifier.
    pub fn new(
        root_json_path: &Path,
        metadata_base_url: Url,
        targets_base_url: Url,
        cache_dir: &Path,
    ) -> Self {
        Self {
            root_json_path: root_json_path.to_path_buf(),
            metadata_base_url,
            targets_base_url,
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    /// Load the repository and verify all metadata.
    /// This performs a 'refresh' to ensure we have the latest trusted state.
    pub async fn verify_and_download(&self, target_name_str: &str) -> Result<PathBuf> {
        let root_data = std::fs::read(&self.root_json_path).context("Failed to read root.json")?;

        // In tough 0.21, load() is async.
        let repo = RepositoryLoader::new(
            &root_data,
            self.metadata_base_url.clone(),
            self.targets_base_url.clone(),
        )
        .transport(DefaultTransport::default())
        .load()
        .await
        .context("TUF repository load/refresh failed")?;

        // Download and verify the target
        let target_path = self.cache_dir.join("targets").join(target_name_str);
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Convert string to TargetName (using the public type)
        let target_name: TargetName = target_name_str
            .try_into()
            .map_err(|e| anyhow::anyhow!("Invalid target name: {}", e))?;

        // read_target returns a stream of verified chunks.
        let mut stream = repo
            .read_target(&target_name)
            .await
            .context(format!(
                "Failed to find target '{}' in TUF manifest",
                target_name_str
            ))?
            .context("Target not found")?;

        let mut file = tokio::fs::File::create(&target_path)
            .await
            .context("Failed to create local target file")?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.context("Failure while downloading/verifying target chunk")?;
            file.write_all(&chunk)
                .await
                .context("Failed to write target chunk to disk")?;
        }

        file.flush().await?;

        Ok(target_path)
    }
}
