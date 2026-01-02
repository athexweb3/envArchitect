use crate::detector::LanguageDetector;
use env_architect_sdk::host;
use env_architect_sdk::prelude::*;

pub struct NodeDetector;

impl LanguageDetector for NodeDetector {
    fn detect(&self) -> Result<bool> {
        Ok(host::get_env("package.json").is_some() || host::read_file("package.json").is_ok())
    }

    fn plan(&self) -> Result<InstallPlan> {
        let mut plan = InstallPlan::default();

        // Add Node.js requirement to manifest.env
        plan.manifest
            .env
            .insert("node".to_string(), "18.x".to_string());

        // Add install command
        if host::read_file("yarn.lock").is_ok() {
            plan.instructions.push("yarn install".to_string());
        } else {
            plan.instructions.push("npm install".to_string());
        }

        Ok(plan)
    }
}
