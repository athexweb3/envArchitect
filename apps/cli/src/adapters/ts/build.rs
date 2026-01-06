use crate::adapters::BuildOptions;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn build_ts_project(dir: &Path, _options: BuildOptions) -> Result<PathBuf> {
    let spinner = cliclack::spinner();
    spinner.start("Building Wasm binary (TS)...");

    // Install deps if needed (heuristic: node_modules missing)
    if !dir.join("node_modules").exists() {
        spinner.start("Installing dependencies (npm install)...");
        let status = Command::new("npm")
            .arg("install")
            .current_dir(dir)
            .status()
            .context("Failed to run npm install")?;

        if !status.success() {
            anyhow::bail!("npm install failed");
        }
    }

    spinner.start("Running build script...");
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(dir)
        .status()
        .context("Failed to run npm run build")?;

    if !status.success() {
        anyhow::bail!("npm run build failed");
    }

    // Convention: dist/plugin.wasm or similar.

    let candidates = [
        dir.join("plugin.wasm"),
        dir.join("dist/plugin.wasm"),
        dir.join("target/plugin.wasm"),
    ];

    for path in candidates.iter() {
        if path.exists() {
            spinner.stop("Build complete.");
            // Optimization step could go here (jco opt?)
            return Ok(path.clone());
        }
    }

    // Fallback: search for *any* wasm file in dist/
    let dist = dir.join("dist");
    if dist.exists() {
        for entry in std::fs::read_dir(dist)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "wasm" {
                    spinner.stop("Build complete.");
                    return Ok(path);
                }
            }
        }
    }

    anyhow::bail!("Build completed but no .wasm artifact found. Expected 'plugin.wasm' or 'dist/plugin.wasm'.")
}
