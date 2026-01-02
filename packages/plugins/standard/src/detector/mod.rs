use env_architect_sdk::prelude::*; // For InstallPlan, Result

pub mod node;
pub mod python;
pub mod rust;

use self::node::NodeDetector;
use self::python::PythonDetector;
use self::rust::RustDetector;

/// The common interface for language detectors
pub trait LanguageDetector {
    /// Returns true if this language is detected in the repository
    fn detect(&self) -> Result<bool>;

    /// Returns the installation plan for this language
    fn plan(&self) -> Result<InstallPlan>;
}

/// Main entry point for detection
pub fn detect_all() -> Result<InstallPlan> {
    let mut final_plan = InstallPlan::default();

    // 1. Register Detectors
    let detectors: Vec<Box<dyn LanguageDetector>> = vec![
        Box::new(NodeDetector),
        Box::new(PythonDetector),
        Box::new(RustDetector),
    ];

    // 2. Run them
    for detector in detectors {
        if detector.detect()? {
            let plan = detector.plan()?;
            // Merge logic
            final_plan.manifest.env.extend(plan.manifest.env);
            final_plan
                .manifest
                .dependencies
                .extend(plan.manifest.dependencies);
            final_plan.instructions.extend(plan.instructions);
        }
    }

    Ok(final_plan)
}
