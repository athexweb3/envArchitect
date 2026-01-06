use anyhow::{Context, Result};
use domain::system::StoreManager;
use std::path::PathBuf;

/// The entry point for the shim proxy
pub async fn execute_shim(tool_name: String, args: Vec<String>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir).context(
        "Could not find an Architect project (env.json/toml) in the current directory or parents",
    )?;

    let manifest_path = project_root.join(crate::constants::MANIFEST_JSON); // Or .architect/lock.json
    let content = std::fs::read_to_string(&manifest_path)
        .context("Failed to read project manifest. Have you run 'architect resolve'?")?;

    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let _version_req = manifest
        .get("dependencies")
        .and_then(|d| d.get(&tool_name))
        .and_then(|v| v.as_str())
        .or_else(|| None)
        .context(format!(
            "Tool '{}' is not defined in this project's environment",
            tool_name
        ))?;

    let store = StoreManager::default()?;

    let tools = store.list_tools()?;
    let match_in_store = tools
        .into_iter()
        .find(|t| t == &tool_name)
        .context(format!(
            "Tool '{}' is defined in manifest but not found in Architect Store",
            tool_name
        ))?;

    let exec_path = store
        .get_executable_path(&match_in_store, "20.11.0", "abc123456789", &tool_name)
        .context(format!(
            "Failed to find executable for '{}' in store",
            tool_name
        ))?;

    let mut child = std::process::Command::new(exec_path)
        .args(args)
        .spawn()
        .context(format!("Failed to execute shimmed tool: {}", tool_name))?;

    let status = child.wait()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn find_project_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        use crate::constants::MANIFEST_JSON;
        if current.join(MANIFEST_JSON).exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}
