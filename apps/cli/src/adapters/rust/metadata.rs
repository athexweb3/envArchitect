use super::super::PluginMetadata;
use anyhow::{Context, Result};
use std::path::Path;

pub fn extract_metadata(dir: &Path) -> Result<PluginMetadata> {
    // Use cargo metadata to avoid parsing TOML manually

    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(dir)
        .output()
        .context("Failed to run cargo metadata")?;

    if !output.status.success() {
        return Ok(PluginMetadata {
            name: "unknown".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            authors: None,
            license: None,
            repository: None,
            capabilities: None,
        });
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    // Find the package that matches the directory

    let packages = json.get("packages").and_then(|v| v.as_array());

    let pkg = if let Some(pkgs) = packages {
        // Filter for the package that corresponds to the directory we are in (by manifest_path)
        let target_manifest = dir.join("Cargo.toml");
        pkgs.iter()
            .find(|p| {
                p.get("manifest_path")
                    .and_then(|s| s.as_str())
                    .map(Path::new)
                    .map(|p| p == target_manifest)
                    .unwrap_or(false)
            })
            .or(pkgs.first()) // Fallback to first if not found (e.g. workspace root)
            .unwrap_or(
                return Ok(PluginMetadata {
                    name: "unknown".to_string(),
                    version: "0.0.0".to_string(),
                    description: None,
                    authors: None,
                    license: None,
                    repository: None,
                    capabilities: None,
                }),
            )
    } else {
        return Ok(PluginMetadata {
            name: "unknown".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            authors: None,
            license: None,
            repository: None,
            capabilities: None,
        });
    };

    let name = pkg
        .get("name")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    let version = pkg
        .get("version")
        .and_then(|s| s.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    let description = pkg
        .get("description")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    let authors = pkg.get("authors").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    let license = pkg
        .get("license")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());
    let repository = pkg
        .get("repository")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    Ok(PluginMetadata {
        name,
        version,
        description,
        authors,
        license,
        repository,
        capabilities: None,
    })
}
