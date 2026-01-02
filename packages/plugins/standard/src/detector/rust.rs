use crate::detector::LanguageDetector;
use env_architect_sdk::host;
use env_architect_sdk::prelude::*;

pub struct RustDetector;

impl LanguageDetector for RustDetector {
    fn detect(&self) -> Result<bool> {
        Ok(host::read_file("Cargo.toml").is_ok())
    }

    fn plan(&self) -> Result<InstallPlan> {
        let mut plan = InstallPlan::default();
        plan.manifest
            .env
            .insert("rust".to_string(), "stable".to_string());
        plan.instructions.push("cargo build".to_string());
        Ok(plan)
    }
}
