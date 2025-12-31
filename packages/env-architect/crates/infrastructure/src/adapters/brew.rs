use anyhow::Result;
use domain::entities::tool::Tool;
use domain::ports::package_manager::{PackageError, PackageManager};
use std::process::Command;

pub struct BrewAdapter;

impl BrewAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for BrewAdapter {
    fn install(&self, tool: &Tool) -> Result<(), PackageError> {
        let package = tool.package_name.as_deref().unwrap_or(&tool.name);

        println!("🍺 Brew installing: {}", package);

        let status = Command::new("brew")
            .arg("install")
            .arg(package)
            .status()
            .map_err(|e| PackageError::InstallFailed(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(PackageError::InstallFailed("Non-zero exit code".into()))
        }
    }

    fn is_installed(&self, tool: &Tool) -> Result<bool, PackageError> {
        let package = tool.package_name.as_deref().unwrap_or(&tool.name);
        // brew list --versions <name> returns 0 if installed, 1 if not
        let status = Command::new("brew")
            .arg("list")
            .arg("--versions")
            .arg(package)
            .output()
            .map_err(|e| PackageError::InstallFailed(e.to_string()))?
            .status;

        Ok(status.success())
    }
}
