use anyhow::{Context, Result};
use env_architect::domain::entities::manifest::EnhancedManifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Finds and loads an environment manifest following the ecosystem precedence rules.
pub fn find_and_load_manifest(start_dir: &Path) -> Result<(PathBuf, EnhancedManifest)> {
    // Discovery Order:

    use crate::constants::MANIFEST_JSON;
    let candidates = vec![MANIFEST_JSON, "env.toml", "env.yaml"];

    for filename in candidates {
        let path = start_dir.join(filename);
        if path.exists() {
            return load_manifest(&path).map(|m| (path, m));
        }
    }

    anyhow::bail!(
        "No environment manifest (env.json) found in {:?}",
        start_dir
    )
}

/// Loads a manifest from a specific path, detecting format by extension.
pub fn load_manifest(path: &Path) -> Result<EnhancedManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest file: {:?}", path))?;

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "json" => serde_json::from_str(&content).with_context(|| "Failed to parse JSON manifest"),
        "toml" => toml::from_str(&content).with_context(|| "Failed to parse TOML manifest"),
        "yaml" | "yml" => {
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML manifest")
        }
        _ => anyhow::bail!("Unsupported manifest format: {}", ext),
    }
}
