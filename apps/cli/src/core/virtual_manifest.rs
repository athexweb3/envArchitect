use anyhow::{Context, Result};
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

        // 1. Setup minimal Project Metadata
        manifest.project = ProjectMetadata {
            name: format!("system-install-{}", package_name),
            version: "0.0.0-ephemeral"
                .parse()
                .unwrap_or_else(|_| "0.0.0".parse().unwrap()),
            description: format!("Ephemeral system context for {}", package_name),
            authors: vec!["env-architect-cli".to_string()],
            license: "".to_string(),
            repository: None,
            homepage: None,
        };

        // 2. Add the requested dependency
        let mut deps = HashMap::new();

        // Parse resolution string (simple heuristic for this MVP)
        use env_architect::domain::entities::manifest::DependencyDetails;
        use semver::VersionReq;

        let spec = if resolution.starts_with("path:") {
            let path = resolution.trim_start_matches("path:");
            DependencySpec::Detailed(DependencyDetails {
                version: VersionReq::parse("*").unwrap(),
                manager: None,
                source: Some(path.to_string()),
                optional: false,
            })
        } else {
            // Assume "registry:..." or just a version
            // For "registry:node", we treat it as ANY version for now
            DependencySpec::Simple(VersionReq::parse("*").unwrap())
        };

        deps.insert(package_name.to_string(), spec);
        manifest.dependencies = deps;

        // 3. Inject standard capabilities based on the known plugin requirements.
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
