use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
pub struct ShellCommand {
    /// Path to the project root
    #[arg(long, short)]
    pub project_root: Option<PathBuf>,
}

impl ShellCommand {
    pub async fn execute(self) -> Result<()> {
        use crate::ui::{Icon, Theme};

        let root = self
            .project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from(".")));

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root);

        println!(
            "{} {} {}",
            Icon::Architect,
            Theme::primary("Activating Environment:"),
            Theme::bold(
                absolute_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Project")
            )
        );

        // 1. Discover required shims from env.json/toml
        // For Phase 1, we simply create a temporary shims directory and add it to PATH
        let shims_dir = absolute_root.join(".architect").join("shims");
        if !shims_dir.exists() {
            std::fs::create_dir_all(&shims_dir)?;
        }

        // 2. Setup the shell environment
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());

        // Update PATH: Prepend shims directory
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", shims_dir.to_string_lossy(), path_env);

        println!(
            "{} Spawning {} with Architect context...",
            Icon::Success,
            shell
        );

        let mut child = Command::new(&shell)
            .env("PATH", new_path)
            .env(
                "ARCHITECT_PROJECT_ROOT",
                absolute_root.to_string_lossy().to_string(),
            )
            .spawn()
            .context(format!("Failed to spawn shell: {}", shell))?;

        let status = child.wait()?;

        if status.success() {
            println!("\n{} Shell exited successfully.", Icon::Success);
        } else {
            println!("\n{} Shell exited with error.", Icon::Cross);
        }

        Ok(())
    }
}
