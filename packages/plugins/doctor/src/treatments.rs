use crate::diagnosis::{RiskLevel, Treatment};
use anyhow::Result;
use env_architect_sdk::host::exec;

#[derive(Debug)]
pub struct BrewInstallTreatment {
    pub package: String,
}

impl Treatment for BrewInstallTreatment {
    fn description(&self) -> String {
        format!("Install '{}' via Homebrew", self.package)
    }

    fn risk(&self) -> RiskLevel {
        RiskLevel::Medium // System modification
    }

    fn apply(&self) -> Result<()> {
        // In a real plugin, this would verify brew exists first
        println!("Applying treatment: brew install {}", self.package);
        // logic to run exec("brew", &["install", &self.package])
        Ok(())
    }
}

#[derive(Debug)]
pub struct NvmInstallTreatment {
    pub version: String,
}

impl Treatment for NvmInstallTreatment {
    fn description(&self) -> String {
        format!("Install Node.js {} via nvm", self.version)
    }

    fn risk(&self) -> RiskLevel {
        RiskLevel::Low // User-space modification
    }

    fn apply(&self) -> Result<()> {
        println!("Applying treatment: nvm install {}", self.version);
        // logic to run exec("nvm", &["install", &self.version])
        Ok(())
    }
}

#[derive(Debug)]
pub struct CargoInstallTreatment {
    pub package: String,
}

impl Treatment for CargoInstallTreatment {
    fn description(&self) -> String {
        format!("Install '{}' via Cargo", self.package)
    }

    fn risk(&self) -> RiskLevel {
        RiskLevel::Medium
    }

    fn apply(&self) -> Result<()> {
        exec("cargo", &["install", &self.package])
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}
