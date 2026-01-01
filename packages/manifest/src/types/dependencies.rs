use schemars::JsonSchema;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// A dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string (e.g., `^1.0.0`).
    Simple(
        #[serde(deserialize_with = "deserialize_version_req")]
        #[schemars(with = "String")]
        VersionReq,
    ),

    /// Detailed configuration object.
    Detailed(DependencyDetails),
}

/// Detailed configuration for a dependency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct DependencyDetails {
    /// Version requirement (SemVer range).
    #[serde(
        default = "default_version_req",
        deserialize_with = "deserialize_version_req"
    )]
    #[schemars(with = "String")]
    pub version: VersionReq,

    /// Explicit package manager to use.
    pub manager: Option<PackageManager>,

    /// Custom source URL (git repo, tarball, etc).
    /// Kept as String because it might be a file path or non-standard URI.
    pub source: Option<String>,

    /// Whether this dependency is optional.
    #[serde(default)]
    pub optional: bool,
}

/// Supported package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pip,
    Pip3,
    Cargo,
    Gem,
    Go,
    Maven,
    Gradle,
    Composer,
    Nuget,
    Chocolatey,
    Brew,
    Apt,
    Yum,
    Pacman,
    Docker,
}

/// Target-specific dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TargetDependencies {
    pub dependencies: HashMap<String, DependencySpec>,
}

/// A logical group of dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct DependencyGroup {
    /// Whether this group is installed by default.
    #[serde(default)]
    pub optional: bool,

    /// Dependencies belonging to this group.
    pub dependencies: HashMap<String, DependencySpec>,
}

/// An external asset bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Asset {
    pub name: String,
    pub url: Url,
    pub checksum: String,
}

use serde::Deserializer;

pub fn deserialize_dependency_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, DependencySpec>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DepMapOrSeq {
        Map(HashMap<String, DependencySpec>),
        Seq(Vec<String>),
    }

    match DepMapOrSeq::deserialize(deserializer)? {
        DepMapOrSeq::Map(m) => Ok(m),
        DepMapOrSeq::Seq(s) => {
            let mut map = HashMap::new();
            for name in s {
                map.insert(name.clone(), DependencySpec::Simple(default_version_req()));
            }
            Ok(map)
        }
    }
}

pub fn default_version_req() -> VersionReq {
    VersionReq::parse("*").unwrap()
}

pub fn deserialize_version_req<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(VersionReq::parse("*").unwrap())
    } else {
        VersionReq::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_spec_simple() {
        let json = r#""^1.2.3""#;
        let spec: DependencySpec = serde_json::from_str(json).unwrap();
        match spec {
            DependencySpec::Simple(v) => assert_eq!(v, VersionReq::parse("^1.2.3").unwrap()),
            _ => panic!("Expected Simple"),
        }
    }

    #[test]
    fn test_dependency_spec_empty_string() {
        let json = r#""""#;
        let spec: DependencySpec = serde_json::from_str(json).unwrap();
        match spec {
            DependencySpec::Simple(v) => assert_eq!(v, VersionReq::parse("*").unwrap()),
            _ => panic!("Expected Simple"),
        }
    }

    #[test]
    fn test_dependency_spec_empty() {
        let json = r#""""#;
        let spec: DependencySpec = serde_json::from_str(json).unwrap();
        match spec {
            DependencySpec::Simple(v) => assert_eq!(v, VersionReq::parse("*").unwrap()),
            _ => panic!("Expected Simple"),
        }
    }
}
