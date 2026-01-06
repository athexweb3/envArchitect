use super::super::BuildOptions;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn build_rust_project(dir: &Path, options: BuildOptions) -> Result<PathBuf> {
    let spinner = cliclack::spinner();
    spinner.start("Building Wasm binary (Rust)...");

    // Try wasm32-wasip1 first, then fallback
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--target").arg("wasm32-wasip1");

    if options.release {
        cmd.arg("--release");
    }

    let mut status = cmd.current_dir(dir).status();

    if status.is_err() || !status.as_ref().unwrap().success() {
        spinner.start("wasm32-wasip1 failed, trying legacy wasm32-wasi...");
        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--target").arg("wasm32-wasi");
        if options.release {
            cmd.arg("--release");
        }
        status = cmd.current_dir(dir).status();
    }

    if let Ok(s) = status {
        if !s.success() {
            spinner.error("Cargo build failed");
            anyhow::bail!("Cargo build failed");
        }
    } else {
        spinner.error("Failed to execute cargo build");
        anyhow::bail!("Failed to execute cargo build");
    }

    spinner.stop("Cargo build complete");

    let wasm_path = find_wasm_artifact(dir)?;

    let component_path = componentize(dir, &wasm_path).await?;

    Ok(component_path)
}

fn find_wasm_artifact(dir: &Path) -> Result<PathBuf> {
    // Parse Cargo.toml using cargo metadata to get package name
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(dir)
        .output()
        .context("Failed to run cargo metadata")?;

    let package_name = if output.status.success() {
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        json.get("packages")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|p| p.get("name"))
            .and_then(|s| s.as_str())
            .unwrap_or("plugin")
            .to_string()
    } else {
        "plugin".to_string()
    };

    let wasm_filename = format!("{}.wasm", package_name.replace('-', "_"));

    // Search paths (prioritize release if exists, then debug)
    let possible_paths = [
        dir.join("target/wasm32-wasip1/release")
            .join(&wasm_filename),
        dir.join("target/wasm32-wasip1/debug").join(&wasm_filename),
        dir.join("target/wasm32-wasi/release").join(&wasm_filename),
        dir.join("target/wasm32-wasi/debug").join(&wasm_filename),
        // Workspace paths
        dir.join("../../target/wasm32-wasip1/release")
            .join(&wasm_filename),
        dir.join("../../target/wasm32-wasip1/debug")
            .join(&wasm_filename),
        dir.join("../../target/wasm32-wasi/release")
            .join(&wasm_filename),
        dir.join("../../target/wasm32-wasi/debug")
            .join(&wasm_filename),
    ];

    for path in &possible_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    anyhow::bail!("Could not find built Wasm artifact for {}", package_name)
}

async fn componentize(dir: &Path, wasm_path: &Path) -> Result<PathBuf> {
    let component_path = wasm_path.with_extension("component.wasm");

    // If it's already a component (unlikely for raw cargo build), return
    // But we are converting .wasm -> .component.wasm

    let spinner = cliclack::spinner();
    spinner.start("Componentizing...");

    // Ensure WASI Adapter
    let adapter_dir = dir.join("target/adapters");
    let adapter_path = adapter_dir.join("wasi_snapshot_preview1.reactor.wasm");

    if !adapter_path.exists() {
        std::fs::create_dir_all(&adapter_dir).ok();
        let _ = Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg("-o")
            .arg(&adapter_path)
            .arg("https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.0/wasi_snapshot_preview1.reactor.wasm")
            .status();
    }

    // Strip
    let stripped_path = wasm_path.with_extension("stripped.wasm");
    let _ = Command::new("wasm-tools")
        .arg("strip")
        .arg("-a")
        .arg(wasm_path)
        .arg("-o")
        .arg(&stripped_path)
        .output();

    let input_path = if stripped_path.exists() {
        &stripped_path
    } else {
        wasm_path
    };

    // Find WIT
    let wit_path = find_wit_path(dir)?;

    // Embed WIT
    let embedded_path = wasm_path.with_extension("embed.wasm");
    let embed_status = Command::new("wasm-tools")
        .arg("component")
        .arg("embed")
        .arg(&wit_path)
        .arg(input_path)
        .arg("-o")
        .arg(&embedded_path)
        .arg("--world")
        .arg("plugin")
        .status();

    if embed_status.is_err() || !embed_status.as_ref().unwrap().success() {
        spinner.error("WIT Embedding failed");
        anyhow::bail!("WIT Embedding failed");
    }

    // New Component
    let status = Command::new("wasm-tools")
        .arg("component")
        .arg("new")
        .arg(&embedded_path)
        .arg("-o")
        .arg(&component_path)
        .arg("--adapt")
        .arg(format!("wasi_snapshot_preview1={}", adapter_path.display()))
        .status();

    if status.is_err() || !status.as_ref().unwrap().success() {
        spinner.error("Componentization failed");
        anyhow::bail!("Componentization failed");
    }

    spinner.stop("Component created");

    // Cleanup
    let _ = std::fs::remove_file(embedded_path);

    Ok(component_path)
}

fn find_wit_path(dir: &Path) -> Result<PathBuf> {
    let candidates = [
        dir.join("wit/plugin.wit"), // Local project wit
        dir.join("plugin.wit"),
        // Workspace relative paths
        dir.join("packages/sdks/wit/plugin.wit"),
        dir.join("../packages/sdks/wit/plugin.wit"),
        dir.join("../../packages/sdks/wit/plugin.wit"),
        dir.join("../../../packages/sdks/wit/plugin.wit"),
        // Absolute fallback from current dir
        std::env::current_dir().unwrap_or_default().join("packages/sdks/wit/plugin.wit"),
    ];

    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    anyhow::bail!("Could not find 'plugin.wit' file in any expected location. Please ensure the SDK is properly installed or the file exists.")
}
