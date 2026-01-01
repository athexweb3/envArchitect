use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Constraints on where this environment can run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
pub struct PlatformConstraints {
    /// Allowed operating systems.
    #[serde(default = "default_all_platforms")]
    pub platforms: Vec<OperatingSystem>,

    /// Allowed CPU architectures.
    #[serde(default = "default_all_architectures")]
    pub architectures: Vec<CpuArchitecture>,

    /// Minimum version requirements for the OS (e.g., `macos: ">=12.0"`).
    #[serde(default)]
    pub requirements: HashMap<OperatingSystem, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    Linux,
    #[serde(alias = "darwin")]
    Macos,
    Windows,
    Freebsd,
    Openbsd,
    Netbsd,
    Dragonfly,
    Ios,
    Android,
    #[serde(rename = "*")]
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CpuArchitecture {
    X86_64,
    #[serde(alias = "amd64")]
    Amd64, // Alias to X86_64 conceptually, but commonly used
    Aarch64,
    #[serde(alias = "arm64")]
    Arm64,
    Arm,
    Wasm32,
    Riscv64,
    #[serde(rename = "*")]
    Any,
}

fn default_all_platforms() -> Vec<OperatingSystem> {
    vec![OperatingSystem::Any]
}

fn default_all_architectures() -> Vec<CpuArchitecture> {
    vec![CpuArchitecture::Any]
}
