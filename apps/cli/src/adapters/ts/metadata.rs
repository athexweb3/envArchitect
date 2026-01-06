use crate::adapters::PluginMetadata;
use anyhow::{Context, Result};
use std::path::Path;

pub fn extract_metadata(dir: &Path) -> Result<PluginMetadata> {
    let package_json = dir.join("package.json");
    let content = std::fs::read_to_string(&package_json).context("Missing package.json")?;
    let val: serde_json::Value = serde_json::from_str(&content)?;

    let name = val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let version = val
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Authors in package.json can be string or array or object
    let authors = match val.get("author") {
        Some(v) if v.is_string() => Some(vec![v.as_str().unwrap().to_string()]),
        Some(v) if v.is_array() => Some(
            v.as_array()
                .unwrap()
                .iter()
                .filter_map(|s| s.as_str().map(|x| x.to_string()))
                .collect(),
        ),
        // Object format (name, email) - simplify to name for now
        Some(v) if v.is_object() => v
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| vec![s.to_string()]),
        _ => None,
    };

    let license = val
        .get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Repository
    let repository = match val.get("repository") {
        Some(v) if v.is_string() => Some(v.as_str().unwrap().to_string()),
        Some(v) if v.is_object() => v.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()),
        _ => None,
    };

    Ok(PluginMetadata {
        name,
        version,
        description,
        authors,
        license,
        repository,
        capabilities: None, // package.json doesn't have this
    })
}
