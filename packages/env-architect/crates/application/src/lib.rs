pub mod install_usecase;
use anyhow::Result;
use domain::entities::tool::Tool;
use domain::ports::package_manager::PackageManager;

pub async fn install_tool(tool: Tool, pm: &impl PackageManager) -> Result<()> {
    // 1. Check if installed
    if pm.is_installed(&tool)? {
        println!("✅ {} is already installed.", tool.name);
        return Ok(());
    }

    // 2. Install
    println!("📦 Installing {}...", tool.name);
    pm.install(&tool)?;

    println!("✨ Successfully installed {}.", tool.name);
    Ok(())
}
