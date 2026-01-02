use crate::detector::LanguageDetector;
use env_architect_sdk::host;
use env_architect_sdk::prelude::*;

pub struct PythonDetector;

impl LanguageDetector for PythonDetector {
    fn detect(&self) -> Result<bool> {
        Ok(host::read_file("requirements.txt").is_ok()
            || host::read_file("Pipfile").is_ok()
            || host::read_file("pyproject.toml").is_ok())
    }

    fn plan(&self) -> Result<InstallPlan> {
        let mut plan = InstallPlan::default();
        plan.manifest
            .env
            .insert("python".to_string(), "3.10".to_string());

        if host::read_file("Pipfile").is_ok() {
            plan.instructions.push("pipenv install".to_string());
            // Skipping complex dependency insertion for now to avoid dependency headaches
        } else if host::read_file("requirements.txt").is_ok() {
            plan.instructions
                .push("pip install -r requirements.txt".to_string());
        }

        Ok(plan)
    }
}
