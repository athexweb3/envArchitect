use super::manifest::EnhancedManifest;
use anyhow::{Context, Result};
use std::path::Path;

/// Multi-format manifest parser (JSON, YAML, TOML)
pub struct ManifestParser;

/// Supported manifest formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestFormat {
    Json,
    Yaml,
    Toml,
}

impl ManifestParser {
    /// Auto-detect format from file extension and parse
    pub fn parse_file(path: &Path) -> Result<EnhancedManifest> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;

        let format = Self::detect_format(path)?;
        Self::parse(&content, format)
    }

    /// Parse manifest from string with explicit format
    pub fn parse(content: &str, format: ManifestFormat) -> Result<EnhancedManifest> {
        match format {
            ManifestFormat::Json => Self::parse_json(content),
            ManifestFormat::Yaml => Self::parse_yaml(content),
            ManifestFormat::Toml => Self::parse_toml(content),
        }
    }

    /// Detect format from file extension
    pub fn detect_format(path: &Path) -> Result<ManifestFormat> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", path.display()))?;

        match extension.to_lowercase().as_str() {
            "json" => Ok(ManifestFormat::Json),
            "yaml" | "yml" => Ok(ManifestFormat::Yaml),
            "toml" => Ok(ManifestFormat::Toml),
            _ => anyhow::bail!("Unsupported manifest format: .{}", extension),
        }
    }

    /// Find manifest file in directory with priority:
    /// 1. env.toml
    /// 2. env.json
    /// 3. env.yaml / env.yml
    /// 4. envarchitect.json
    /// 5. .envarchitect (any format)
    pub fn find_manifest(dir: &Path) -> Result<(std::path::PathBuf, ManifestFormat)> {
        let candidates = vec![
            ("env.toml", ManifestFormat::Toml),
            ("env.json", ManifestFormat::Json),
            ("env.yaml", ManifestFormat::Yaml),
            ("env.yml", ManifestFormat::Yaml),
            ("envarchitect.json", ManifestFormat::Json),
            (".envarchitect.toml", ManifestFormat::Toml),
            (".envarchitect.json", ManifestFormat::Json),
            (".envarchitect.yaml", ManifestFormat::Yaml),
        ];

        for (filename, format) in candidates {
            let path = dir.join(filename);
            if path.exists() {
                return Ok((path, format));
            }
        }

        anyhow::bail!("No manifest file found in directory: {}", dir.display())
    }

    /// Parse JSON manifest
    fn parse_json(content: &str) -> Result<EnhancedManifest> {
        serde_json::from_str(content).context("Failed to parse JSON manifest")
    }

    /// Parse YAML manifest
    fn parse_yaml(content: &str) -> Result<EnhancedManifest> {
        serde_yaml::from_str(content).context("Failed to parse YAML manifest")
    }

    /// Parse TOML manifest
    fn parse_toml(content: &str) -> Result<EnhancedManifest> {
        toml::from_str(content).context("Failed to parse TOML manifest")
    }

    /// Serialize manifest to string
    pub fn serialize(manifest: &EnhancedManifest, format: ManifestFormat) -> Result<String> {
        match format {
            ManifestFormat::Json => {
                serde_json::to_string_pretty(manifest).context("Failed to serialize to JSON")
            }
            ManifestFormat::Yaml => {
                serde_yaml::to_string(manifest).context("Failed to serialize to YAML")
            }
            ManifestFormat::Toml => {
                toml::to_string_pretty(manifest).context("Failed to serialize to TOML")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_minimal() {
        let json = r#"
        {
            "dependencies": {
                "nodejs": "^20.0.0"
            }
        }
        "#;

        let manifest = ManifestParser::parse(json, ManifestFormat::Json).unwrap();
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_parse_yaml_minimal() {
        let yaml = r#"
dependencies:
  nodejs: ^20.0.0
  python: ~3.11
        "#;

        let manifest = ManifestParser::parse(yaml, ManifestFormat::Yaml).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
    }

    #[test]
    fn test_parse_toml_minimal() {
        let toml_str = r#"
[dependencies]
nodejs = "^20.0.0"
postgresql = ">=15.0"
        "#;

        let manifest = ManifestParser::parse(toml_str, ManifestFormat::Toml).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
    }

    #[test]
    fn test_parse_toml_complete() {
        let toml_str = r#"
[project]
name = "my-app"
version = "1.0.0"
description = "Test application"
authors = ["Architect Team"]

[platform]
platforms = ["macos", "linux"]
architectures = ["x86_64", "aarch64"]

[dependencies]
nodejs = ">=20.0.0"
python = "~3.11"

[dev-dependencies]
rust = ">=1.75"

[profiles.dev]
description = "Development environment"
dependencies = ["dev-dependencies"]

[profiles.dev.env]
DEBUG = "true"
        "#;

        let manifest = ManifestParser::parse(toml_str, ManifestFormat::Toml).unwrap();

        assert_eq!(manifest.project.name, "my-app".to_string());
        assert_eq!(manifest.project.version.to_string(), "1.0.0".to_string());
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dev_dependencies.len(), 1);
        assert_eq!(manifest.profiles.len(), 1);
    }

    #[test]
    fn test_roundtrip_json() {
        let mut manifest = EnhancedManifest::default();
        manifest.project.name = "test".to_string();
        manifest.dependencies.insert(
            "nodejs".to_string(),
            env_manifest::DependencySpec::Simple(semver::VersionReq::parse("^20.0.0").unwrap()),
        );

        let json = ManifestParser::serialize(&manifest, ManifestFormat::Json).unwrap();
        let parsed = ManifestParser::parse(&json, ManifestFormat::Json).unwrap();

        assert_eq!(manifest.project.name, parsed.project.name);
        assert_eq!(manifest.dependencies.len(), parsed.dependencies.len());
    }
}
