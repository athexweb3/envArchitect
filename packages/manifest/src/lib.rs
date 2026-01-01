pub mod types;
pub use types::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced manifest with multi-format support (JSON, YAML, TOML)
/// Follows the specification from enhanced_manifest_proposal.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnhancedManifest {
    /// Project metadata (name, version, etc.)
    #[serde(default, alias = "plugin")]
    pub project: ProjectMetadata,

    /// Platform constraints (OS, architecture, versions)
    #[serde(default)]
    pub platform: Option<PlatformConstraints>,

    /// Main production dependencies (REQUIRED: at least one entry)
    #[serde(default, deserialize_with = "types::deserialize_dependency_map")]
    #[schemars(schema_with = "dependency_map_schema")]
    pub dependencies: HashMap<String, DependencySpec>,

    /// Development-only dependencies
    #[serde(
        default,
        rename = "dev-dependencies",
        deserialize_with = "types::deserialize_dependency_map"
    )]
    #[schemars(schema_with = "dependency_map_schema")]
    pub dev_dependencies: HashMap<String, DependencySpec>,

    /// Test-only dependencies
    #[serde(
        default,
        rename = "test-dependencies",
        deserialize_with = "types::deserialize_dependency_map"
    )]
    #[schemars(schema_with = "dependency_map_schema")]
    pub test_dependencies: HashMap<String, DependencySpec>,

    /// Build-time dependencies
    #[serde(
        default,
        rename = "build-dependencies",
        deserialize_with = "types::deserialize_dependency_map"
    )]
    #[schemars(schema_with = "dependency_map_schema")]
    pub build_dependencies: HashMap<String, DependencySpec>,

    /// Platform-specific dependencies (à la Cargo)
    #[serde(default)]
    pub target: HashMap<String, TargetDependencies>,

    /// Dependency groups (à la Poetry)
    #[serde(default)]
    pub group: HashMap<String, DependencyGroup>,

    /// Environment profiles (à la docker-compose)
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,

    /// Lifecycle hooks (à la npm)
    #[serde(default)]
    pub hooks: Option<LifecycleHooks>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Named scripts
    #[serde(default)]
    pub scripts: HashMap<String, ScriptCommand>,

    /// Optional feature sets (extras)
    #[serde(default)]
    pub extras: HashMap<String, Vec<String>>,

    /// Lockfile and cache settings
    #[serde(default)]
    pub lockfile: Option<LockfileConfig>,

    #[serde(default)]
    pub cache: Option<CacheConfig>,

    /// Background services (daemons)
    #[serde(default)]
    pub services: HashMap<String, ServiceDef>,

    /// Conflict declarations (incompatible packages)
    #[serde(default)]
    pub conflicts: HashMap<String, String>,

    /// Security capability requests
    #[serde(default, deserialize_with = "types::deserialize_capability_list")]
    pub capabilities: Option<Vec<Capability>>,

    /// Static assets for air-gap bundling (Gov/Enterprise)
    #[serde(default)]
    pub assets: Vec<Asset>,

    /// Intelligent environment resolution and conflict data
    #[serde(default)]
    pub intelligence: Option<IntelligenceData>,
}

/// Container for intelligence-related manifest data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, Default)]
pub struct IntelligenceData {
    /// Proposed actions to resolve environment conflicts
    #[serde(default)]
    pub proposed_actions: Vec<ResolutionAction>,
}

fn dependency_map_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    use schemars::schema::{Schema, SchemaObject};

    let object_schema = gen.subschema_for::<HashMap<String, DependencySpec>>();
    let array_schema = gen.subschema_for::<Vec<String>>();

    Schema::Object(SchemaObject {
        subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
            any_of: Some(vec![object_schema, array_schema]),
            ..Default::default()
        })),
        ..Default::default()
    })
}

impl Default for EnhancedManifest {
    fn default() -> Self {
        Self {
            project: ProjectMetadata::default(),
            platform: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            test_dependencies: HashMap::new(),
            build_dependencies: HashMap::new(),
            target: HashMap::new(),
            group: HashMap::new(),
            profiles: HashMap::new(),
            hooks: None,
            env: HashMap::new(),
            scripts: HashMap::new(),
            extras: HashMap::new(),
            lockfile: None,
            cache: Some(CacheConfig {
                enabled: true,
                ttl: std::time::Duration::from_secs(24 * 60 * 60),
            }),
            services: HashMap::new(),
            conflicts: HashMap::new(),
            capabilities: None,
            assets: Vec::new(),
            intelligence: None,
        }
    }
}
