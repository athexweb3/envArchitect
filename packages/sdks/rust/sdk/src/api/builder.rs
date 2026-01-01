use crate::contract::reexports::{
    Asset, Capability, CpuArchitecture, DependencySpec, EnhancedManifest, OperatingSystem,
    PlatformConstraints, ServiceDef,
};
use semver::{Version, VersionReq};
// // use crate::contract::reexports::{DependencyDetails, ProjectMetadata};

/// A Builder for constructing `EnhancedManifest`s easily in Rust.
/// This prevents users from having to manually construct complex nested structs.
pub struct EnvBuilder {
    manifest: EnhancedManifest,
}

impl EnvBuilder {
    pub fn new() -> Self {
        Self {
            manifest: EnhancedManifest::default(),
        }
    }

    /// Automatically load configuration from `env.json` in the project root.
    pub fn from_context(context: &crate::api::context::ResolutionContext) -> anyhow::Result<Self> {
        use crate::api::host;
        use std::path::PathBuf;

        let mut builder = Self::new();
        let root = PathBuf::from(&context.project_root);

        let env_path = root.join("env.json");
        host::debug(format!(
            "Attempting to auto-load config from: {}",
            env_path.to_string_lossy()
        ));

        match host::read_file(env_path.to_string_lossy().into_owned()) {
            Ok(content) => {
                let json: serde_json::Value = serde_json::from_str(&content)?;

                if let Some(proj) = json.get("project") {
                    if let Some(n) = proj.get("name").and_then(|v| v.as_str()) {
                        builder.manifest.project.name = n.to_string();
                    }
                    if let Some(v) = proj.get("version").and_then(|v| v.as_str()) {
                        if let Ok(ver) = Version::parse(v) {
                            builder.manifest.project.version = ver;
                        }
                    }
                    if let Some(d) = proj.get("description").and_then(|v| v.as_str()) {
                        builder.manifest.project.description = d.to_string();
                    }
                    if let Some(authors) = proj.get("authors").and_then(|v| v.as_array()) {
                        builder.manifest.project.authors = authors
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }

                Ok(builder)
            }
            Err(e) => {
                host::warn(format!("Could not load env.json: {}", e));
                Ok(builder)
            }
        }
    }

    /// Set project metadata
    pub fn project(mut self, name: &str, version: &str) -> Self {
        self.manifest.project.name = name.to_string();
        self.manifest.project.version = Version::parse(version).expect("Invalid SemVer version");
        self
    }

    /// Add a production dependency
    pub fn add_dependency(mut self, name: &str, version_req: &str) -> Self {
        self.manifest.dependencies.insert(
            name.to_string(),
            DependencySpec::Simple(VersionReq::parse(version_req).expect("Invalid VersionReq")),
        );
        self
    }

    /// Add a dev dependency
    pub fn add_dev_dependency(mut self, name: &str, version_req: &str) -> Self {
        self.manifest.dev_dependencies.insert(
            name.to_string(),
            DependencySpec::Simple(VersionReq::parse(version_req).expect("Invalid VersionReq")),
        );
        self
    }

    /// Add a conflict
    pub fn conflict(mut self, package: &str, reason: &str) -> Self {
        self.manifest
            .conflicts
            .insert(package.to_string(), reason.to_string());
        self
    }

    /// Add a background service
    pub fn service(mut self, name: &str, service: ServiceDef) -> Self {
        self.manifest.services.insert(name.to_string(), service);
        self
    }

    /// Add a required security capability
    pub fn capability(mut self, cap: Capability) -> Self {
        if let Some(caps) = &mut self.manifest.capabilities {
            caps.push(cap);
        } else {
            self.manifest.capabilities = Some(vec![cap]);
        }
        self
    }

    /// Add an asset for air-gap mode
    pub fn asset(mut self, asset: Asset) -> Self {
        self.manifest.assets.push(asset);
        self
    }

    /// Add a supported platform
    pub fn support_platform(mut self, os: &str, arch: &str) -> Self {
        if self.manifest.platform.is_none() {
            self.manifest.platform = Some(PlatformConstraints {
                platforms: vec![],     // Clear default "Any"
                architectures: vec![], // Clear default "Any"
                requirements: Default::default(),
            });
        }

        let os_enum: OperatingSystem =
            serde_json::from_value(serde_json::Value::String(os.to_string()))
                .expect("Invalid Operating System name");
        let arch_enum: CpuArchitecture =
            serde_json::from_value(serde_json::Value::String(arch.to_string()))
                .expect("Invalid CPU Architecture name");

        if let Some(p) = &mut self.manifest.platform {
            if !p.platforms.contains(&os_enum) {
                p.platforms.push(os_enum);
            }
            if !p.architectures.contains(&arch_enum) {
                p.architectures.push(arch_enum);
            }
        }
        self
    }

    /// Add a proposed resolution action for a conflict
    pub fn resolution_action(mut self, action: env_manifest::ResolutionAction) -> Self {
        if self.manifest.intelligence.is_none() {
            self.manifest.intelligence = Some(env_manifest::IntelligenceData::default());
        }

        if let Some(intel) = &mut self.manifest.intelligence {
            intel.proposed_actions.push(action);
        }
        self
    }

    /// Build the final manifest
    pub fn build(self) -> EnhancedManifest {
        self.manifest
    }
}

impl Default for EnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}
