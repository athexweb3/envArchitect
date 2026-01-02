use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct InitCommand {
    /// Name of the plugin to use (e.g. node, python, rust)
    #[arg(long, short)]
    pub plugin: Option<String>,

    /// Environment name
    #[arg(long, default_value = "my-env")]
    pub name: String,

    /// Force overwrite existing env.toml
    #[arg(long, short)]
    pub force: bool,
}

impl InitCommand {
    pub async fn execute(self) -> Result<()> {
        let _terminal = cliclack::intro("EnvArchitect Initializer")?;

        // 1. Interactive Prompt if no plugin specified
        let plugin = if let Some(p) = self.plugin {
            p
        } else {
            cliclack::select("Select a plugin to initialize:")
                .item("node", "Node.js (Standard)", "")
                .item("python", "Python (Standard)", "")
                .item("rust", "Rust (Standard)", "")
                .interact()?
                .to_string()
        };

        // 2. Determine Capabilities & Resolution
        // For this MVP, we hardcode defaults for known plugins and support local paths for dev
        let (resolution, caps) = match plugin.as_str() {
            "node" => (
                // In dev mode (this repo), we point to target. In real usage, this would be registry:node
                if std::path::Path::new(
                    "../../target/wasm32-wasip1/debug/env_plugin_node.component.wasm",
                )
                .exists()
                {
                    "path:../../target/wasm32-wasip1/debug/env_plugin_node.component.wasm"
                } else {
                    "registry:node"
                },
                vec!["sys-exec", "fs-read", "fs-write"],
            ),
            "python" => (
                if std::path::Path::new(
                    "../../target/wasm32-wasip1/debug/env_plugin_python.component.wasm",
                )
                .exists()
                {
                    "path:../../target/wasm32-wasip1/debug/env_plugin_python.component.wasm"
                } else {
                    "registry:python"
                },
                vec!["sys-exec", "fs-read", "fs-write", "env-read:PYTHON_VERSION"],
            ),
            "rust" => ("registry:rust", vec!["sys-exec", "fs-write"]),
            _ => ("registry:unknown", vec![]),
        };

        // 3. Generate Content
        let caps_toml = caps
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");

        let content = format!(
            r#"[environment]
name = "{}"
resolution = "{}"

[capabilities]
{} = [{}]
"#,
            self.name, resolution, plugin, caps_toml
        );

        // 4. Write File
        let path = PathBuf::from("env.toml");
        if path.exists() && !self.force {
            cliclack::confirm("env.toml already exists. Overwrite?").interact()?;
        }

        std::fs::write(&path, content).context("Failed to write env.toml")?;

        cliclack::outro(format!("Initialized environment for '{}'!", plugin))?;

        Ok(())
    }
}
