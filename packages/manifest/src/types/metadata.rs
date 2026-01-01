use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

/// Project identity and metadata.
///
/// This section defines who the project belongs to, what it is called, and how it is licensed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ProjectMetadata {
    /// The name of the project.
    /// Should be kebab-case (e.g., `my-cool-project`).
    #[serde(default)]
    pub name: String,

    /// The semantic version of the project.
    /// Strictly checked against SemVer 2.0.0.
    #[serde(default = "default_version")]
    #[schemars(schema_with = "version_schema")]
    pub version: Version,

    /// A short, human-readable description of what the project does.
    #[serde(default)]
    pub description: String,

    /// List of authors or maintainers.
    #[serde(default)]
    pub authors: Vec<String>,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: String,

    /// URL to the source code repository.
    #[serde(default)]
    pub repository: Option<Url>,

    /// URL to the project homepage.
    #[serde(default)]
    pub homepage: Option<Url>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_version(),
            description: String::new(),
            authors: Vec::new(),
            license: String::new(),
            repository: None,
            homepage: None,
        }
    }
}

fn default_version() -> Version {
    Version::parse("0.0.0").unwrap_or_else(|_| Version::new(0, 0, 0))
}

fn version_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let mut schema = gen.subschema_for::<String>().into_object();
    schema.metadata().description = Some("SemVer version string (e.g. 1.0.0)".to_string());
    schemars::schema::Schema::Object(schema)
}
