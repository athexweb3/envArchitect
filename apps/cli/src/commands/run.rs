use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
pub struct RunCommand {
    /// Command to run
    pub command_name: String,

    /// Arguments for the command
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Path to the project root
    #[arg(long, short)]
    pub project_root: Option<PathBuf>,
}

impl RunCommand {
    pub async fn execute(self) -> Result<()> {
        let root = self
            .project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from(".")));

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root);
        let shims_dir = absolute_root.join(".architect").join("shims");

        // Update PATH to include project shims
        let path_env = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", shims_dir.to_string_lossy(), path_env);

        let mut child = Command::new(&self.command_name)
            .args(&self.args)
            .env("PATH", new_path)
            .env(
                "ARCHITECT_PROJECT_ROOT",
                absolute_root.to_string_lossy().to_string(),
            )
            .spawn()
            .context(format!("Failed to run command: {}", self.command_name))?;

        let status = child.wait()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
