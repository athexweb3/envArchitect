use serde::{Deserialize, Serialize};

/// The "World Class" Score Card
/// Every plugin gets one of these calculated in the background.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSignals {
    /// Code Health, Docs Presence, etc. (0.0 - 1.0)
    pub quality: f32,
    /// Logarithmic usage + Stars (0.0 - 1.0)
    pub popularity: f32,
    /// Commit velocity, Recent Releases (0.0 - 1.0)
    pub maintenance: f32,
    /// PageRank / Dependency Centrality (0.0 - 1.0)
    pub authority: f32,
    /// 7-day Install Velocity (0.0 - 1.0)
    pub trending: f32,
}

impl Default for SearchSignals {
    fn default() -> Self {
        Self {
            quality: 0.0,
            popularity: 0.0,
            maintenance: 0.0,
            authority: 0.0,
            trending: 0.0,
        }
    }
}
