use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Lockfile generation settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct LockfileConfig {
    /// Auto-generate `env.lock` on install.
    #[serde(default = "default_true")]
    pub generate: bool,

    /// Remind user to commit lock file if git is detected.
    #[serde(default = "default_true")]
    pub commit_recommendation: bool,
}

/// Cache settings for the environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct CacheConfig {
    /// Enable caching.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Time-to-live for cached entries (e.g., "24h", "30m").
    #[serde(default = "default_cache_ttl", with = "humantime_serde")]
    #[schemars(with = "String")]
    pub ttl: Duration,
}

/// Configuration profile (like overrides).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Profile {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub exclude_groups: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_cache_ttl() -> Duration {
    Duration::from_secs(24 * 60 * 60) // 24 hours
}
