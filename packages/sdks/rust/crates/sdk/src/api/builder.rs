use crate::contract::reexports::{
    Asset, Capability, CpuArchitecture, DependencySpec, EnhancedManifest, OperatingSystem,
    PlatformConstraints, ServiceDef,
};
use semver::{Version, VersionReq};
/// A fluent builder for constructing `EnhancedManifest`s.
///
/// This builder provides a convenient, method-chaining API to create complex
/// environment manifests without manually instantiating nested structures.
///
/// # Example
///
/// ```rust
/// use env_architect_sdk::EnvBuilder;
///
/// // Prefer loading from context:
/// // let manifest = EnvBuilder::from_context(&context)?;
///
/// let manifest = EnvBuilder::new()
///     .add_dependency("node", "18.x")
///     .build();
/// ```
pub struct EnvBuilder {
    manifest: EnhancedManifest,
}

impl EnvBuilder {
    /// Creates a new, empty `EnvBuilder`.
    pub fn new() -> Self {
        Self {
            manifest: EnhancedManifest::default(),
        }
    }

    /// Automatically loads configuration from `env.json` in the project root.
    ///
    /// Automatically loads configuration from the context or `env.json`.
    ///
    /// This prefers the `configuration` object passed by the host (which supports `env.toml`, `env.json`, etc.).
    /// If missing, it attempts to read `env.json` relative to the `project_root`.
    pub fn from_context(context: &crate::api::context::ResolutionContext) -> anyhow::Result<Self> {
        use crate::api::host;
        use std::path::PathBuf;

        let mut builder = Self::new();

        // 1. Try to use configuration passed by Host (Preferred)
        if let Some(config) = &context.configuration {
            host::debug("Loading configuration from host context...");
            Self::populate_from_json(&mut builder, config);
            return Ok(builder);
        }

        // 2. Fallback: Read env.json manually (Legacy/Test mode)
        let root = PathBuf::from(&context.project_root);
        let env_path = root.join("env.json");
        host::debug(format!(
            "Attempting to auto-load config from: {}",
            env_path.to_string_lossy()
        ));

        match host::read_file(env_path.to_string_lossy().into_owned()) {
            Ok(content) => {
                let json: serde_json::Value = serde_json::from_str(&content)?;
                Self::populate_from_json(&mut builder, &json);
                Ok(builder)
            }
            Err(e) => {
                host::warn(format!("Could not load env.json: {}", e));
                Ok(builder)
            }
        }
    }

    fn populate_from_json(builder: &mut Self, json: &serde_json::Value) {
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
    }

    /// Sets the project metadata (name and version).
    ///
    /// # Deprecated
    /// This method violates the "Manifest Source of Truth" principle.
    /// Project metadata should be defined in `env.json` or `env.toml` and loaded via `from_context`.
    #[deprecated(
        note = "Define project metadata in env.json and load with EnvBuilder::from_context()"
    )]
    pub fn project(mut self, name: &str, version: &str) -> Self {
        self.manifest.project.name = name.to_string();
        self.manifest.project.version = Version::parse(version).expect("Invalid SemVer version");
        self
    }

    /// Adds a production dependency.
    ///
    /// # Arguments
    /// * `name` - The package name of the dependency.
    /// * `version_req` - A SemVer version requirement string (e.g., "^1.0").
    pub fn add_dependency(mut self, name: &str, version_req: &str) -> Self {
        self.manifest.dependencies.insert(
            name.to_string(),
            DependencySpec::Simple(VersionReq::parse(version_req).expect("Invalid VersionReq")),
        );
        self
    }

    /// Adds a development-only dependency.
    pub fn add_dev_dependency(mut self, name: &str, version_req: &str) -> Self {
        self.manifest.dev_dependencies.insert(
            name.to_string(),
            DependencySpec::Simple(VersionReq::parse(version_req).expect("Invalid VersionReq")),
        );
        self
    }

    /// Declares a conflict with another package.
    pub fn conflict(mut self, package: &str, reason: &str) -> Self {
        self.manifest
            .conflicts
            .insert(package.to_string(), reason.to_string());
        self
    }

    /// Adds a background service definition.
    pub fn service(mut self, name: &str, service: ServiceDef) -> Self {
        self.manifest.services.insert(name.to_string(), service);
        self
    }

    /// Declares a required security capability.
    pub fn capability(mut self, cap: Capability) -> Self {
        if let Some(caps) = &mut self.manifest.capabilities {
            caps.push(cap);
        } else {
            self.manifest.capabilities = Some(vec![cap]);
        }
        self
    }

    /// Adds an asset bundle for air-gapped environments.
    pub fn asset(mut self, asset: Asset) -> Self {
        self.manifest.assets.push(asset);
        self
    }

    /// Adds platform constraints (OS and Architecture support).
    pub fn support_platform(mut self, os: &str, arch: &str) -> Self {
        if self.manifest.platform.is_none() {
            self.manifest.platform = Some(PlatformConstraints {
                platforms: vec![],
                architectures: vec![],
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

    /// Adds a proposed resolution action for a conflict.
    pub fn resolution_action(mut self, action: env_manifest::ResolutionAction) -> Self {
        if self.manifest.intelligence.is_none() {
            self.manifest.intelligence = Some(env_manifest::IntelligenceData::default());
        }

        if let Some(intel) = &mut self.manifest.intelligence {
            intel.proposed_actions.push(action);
        }
        self
    }

    /// Consumes the builder and returns the construct `EnhancedManifest`.
    pub fn build(self) -> EnhancedManifest {
        self.manifest
    }
}

impl Default for EnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}
