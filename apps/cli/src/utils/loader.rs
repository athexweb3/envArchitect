use anyhow::{Context, Result};
use env_architect::domain::entities::manifest::EnhancedManifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Finds and loads an environment manifest following the ecosystem precedence rules.
pub fn find_and_load_manifest(start_dir: &Path) -> Result<(PathBuf, EnhancedManifest)> {
    // Discovery Order:
    // 1. env.toml (Native/Rust)
    // 2. env.json (JS/Web)
    // 3. env.yaml (DevOps)
    let candidates = vec!["env.toml", "env.json", "env.yaml", "env.yml"];

    for filename in candidates {
        let path = start_dir.join(filename);
        if path.exists() {
            return load_manifest(&path).map(|m| (path, m));
        }
    }

    anyhow::bail!(
        "No environment manifest (env.toml, env.json, or env.yaml) found in {:?}",
        start_dir
    )
}

/// Loads a manifest from a specific path, detecting format by extension.
pub fn load_manifest(path: &Path) -> Result<EnhancedManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest file: {:?}", path))?;

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "toml" => toml::from_str(&content).with_context(|| "Failed to parse TOML manifest"),
        "json" => serde_json::from_str(&content).with_context(|| "Failed to parse JSON manifest"),
        "yaml" | "yml" => {
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML manifest")
        }
        _ => anyhow::bail!("Unsupported manifest format: {}", ext),
    }
}
