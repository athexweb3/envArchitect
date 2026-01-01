pub mod install_service;
pub mod install_usecase;

pub use install_service::InstallService;

use anyhow::Result;
use domain::entities::tool::Tool;
use domain::ports::package_manager::PackageManager;

/// Legacy install function (for backward compatibility)
pub async fn install_tool(tool: Tool, pm: &impl PackageManager) -> Result<()> {
    if pm.is_installed(&tool)? {
        println!("✅ {} is already installed.", tool.name);
        return Ok(());
    }

    println!("📦 Installing {}...", tool.name);
    pm.install(&tool)?;

    println!("✨ Successfully installed {}.", tool.name);
    Ok(())
}
