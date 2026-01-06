use anyhow::Result;
use env_architect::domain::entities::manifest::{
    DependencySpec, EnhancedManifest, ProjectMetadata,
};
use std::collections::HashMap;

/// Creates an ephemeral (in-memory) manifest for system-level installation.
/// This is used when the user runs `env-architect install <pkg>` without an existing env.toml.
pub struct VirtualManifestBuilder;

impl VirtualManifestBuilder {
    pub fn build(package_name: &str, resolution: &str) -> Result<EnhancedManifest> {
        let mut manifest = EnhancedManifest::default();

        manifest.project = ProjectMetadata {
            name: format!("system-install-{}", package_name),
            version: "0.0.0-ephemeral"
                .parse()
                .expect("Failed to parse hardcoded ephemeral version"),
            description: format!("Ephemeral system context for {}", package_name),
            authors: vec!["env-architect-cli".to_string()],
            license: "".to_string(),
            repository: None,
            homepage: None,
        };

        let mut deps = HashMap::new();

        // Parse resolution string (simple heuristic for this MVP)
        use env_architect::domain::entities::manifest::DependencyDetails;
        use semver::VersionReq;

        let spec = if let Some(path) = resolution.strip_prefix("path:") {
            DependencySpec::Detailed(DependencyDetails {
                version: VersionReq::parse("*").expect("Failed to parse wildcard version"),
                manager: None,
                source: Some(path.to_string()),
                optional: false,
            })
        } else {
            DependencySpec::Simple(
                VersionReq::parse("*").expect("Failed to parse wildcard version"),
            )
        };

        deps.insert(package_name.to_string(), spec);
        manifest.dependencies = deps;

        use env_architect::domain::entities::manifest::Capability;

        let caps = match package_name {
            "node" => vec![
                Capability::SysExec(vec!["node".into(), "npm".into(), "nvm".into(), "sh".into()]),
                Capability::FsRead(vec![".".into()]),
                Capability::FsWrite(vec![".".into()]),
                Capability::EnvRead(vec!["PATH".into(), "HOME".into(), "NVM_DIR".into()]),
            ],
            "python" => vec![
                Capability::SysExec(vec!["python".into(), "pip".into()]),
                Capability::FsRead(vec![".".into()]),
                Capability::FsWrite(vec![".".into()]),
                Capability::EnvRead(vec!["PYTHON_VERSION".into(), "PATH".into()]),
            ],
            _ => vec![],
        };

        if !caps.is_empty() {
            manifest.capabilities = Some(caps);
        }

        Ok(manifest)
    }
}
