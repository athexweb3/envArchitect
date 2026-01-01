use anyhow::{Context, Result};
use domain::system::StoreManager;
use std::path::PathBuf;

/// The entry point for the shim proxy
pub async fn execute_shim(tool_name: String, args: Vec<String>) -> Result<()> {
    // 1. Discover the project root by searching upwards for env.json/toml
    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir).context(
        "Could not find an Architect project (env.json/toml) in the current directory or parents",
    )?;

    // 2. Load the project manifest (Resolution result)
    // For Phase 1, we assume a simplified lookup. In production, we'd check the Lockfile.
    let manifest_path = project_root.join("env.json"); // Or .architect/lock.json
    let content = std::fs::read_to_string(&manifest_path)
        .context("Failed to read project manifest. Have you run 'architect resolve'?")?;

    // 3. Find the version required for this tool
    // We'll use a hack for the prototype: search the manifest for the tool name
    let manifest: serde_json::Value = serde_json::from_str(&content)?;
    let version_req = manifest
        .get("dependencies")
        .and_then(|d| d.get(&tool_name))
        .and_then(|v| v.as_str())
        .or_else(|| {
            // Check services or other fields if needed
            None
        })
        .context(format!(
            "Tool '{}' is not defined in this project's environment",
            tool_name
        ))?;

    // 4. Resolve the version to a store path
    let store = StoreManager::default()?;

    // For Phase 1 prototype, we'll assume a hardcoded hash or look up existing entries
    let tools = store.list_tools()?;
    let match_in_store = tools
        .into_iter()
        .find(|t| t == &tool_name)
        .context(format!(
            "Tool '{}' is defined in manifest but not found in Architect Store",
            tool_name
        ))?;

    // 5. Execute the real binary
    // In a real implementation, we'd use the exact version/hash from the lockfile
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
        if current.join("env.json").exists() || current.join("env.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}
