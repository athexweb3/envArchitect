pub mod conflicts;
pub mod metrics;
pub mod strategies;

pub use conflicts::ConflictResolver;
pub use metrics::MetricsDetector;
pub use strategies::{
    Conflict, ConflictSource, InstallStrategy, Recommendation, Resolution, ResolutionStrategy, Risk,
};
