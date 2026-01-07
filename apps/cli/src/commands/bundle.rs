use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
pub struct BundleCommand {
    /// Path to the package manifest (e.g. Cargo.toml or directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Skip optimization step
    #[arg(long)]
    pub no_optimize: bool,
}

#[derive(Serialize, Deserialize)]
struct PluginMetadata {
    name: String,
    version: String,
    description: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
    capabilities: Option<Vec<String>>,
    dependencies: Option<Vec<DependencyInfo>>,
    repository: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DependencyInfo {
    purl: String,
    kind: String, // runtime, dev, build
    req: String,
}

impl BundleCommand {
    pub async fn execute(&self) -> Result<PathBuf> {
        // Returns the path to the bundle directory (dist)
        cliclack::intro("EnvArchitect Bundle (Enterprise)")?;

        let project_dir = if self.path.is_file() {
            self.path.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            self.path.clone()
        };

        // Ensure we use absolute paths for monorepo detection
        let project_dir = std::fs::canonicalize(&project_dir).context(format!(
            "Failed to resolve absolute path for {:?}",
            project_dir
        ))?;

        if !project_dir.exists() {
            return Err(anyhow::anyhow!(
                "Project directory does not exist: {:?}",
                project_dir
            ));
        }

        cliclack::log::info(format!("Project Directory: {:?}", project_dir))?;

        // 1. Build (Requires Cargo.toml for compilation, regardless of metadata source)
        self.build_component(&project_dir)?;

        // 2. Locate Artifacts
        let target_dir = self.get_target_dir(&project_dir)?;

        // Metadata Extraction (Prioritize env.toml/env.json)
        let manifest = self.parse_plugin_manifest(&project_dir)?;
        let package_name = manifest.name.clone();

        // 3. Bundling
        let bundle_dir = project_dir.join("dist");
        if bundle_dir.exists() {
            std::fs::remove_dir_all(&bundle_dir)?;
            cliclack::log::info("Cleaned previous dist artifacts")?;
        }
        std::fs::create_dir(&bundle_dir)?;

        // --- Step A: Wasm Artifact ---
        let wasm_filename = format!("{}.wasm", package_name.replace("-", "_"));
        let wasm_path = target_dir
            .join("wasm32-wasip1/release")
            .join(&wasm_filename);

        // Fallback search logic for hyphenated name
        let wasm_path = if wasm_path.exists() {
            wasm_path
        } else {
            let alt = target_dir
                .join("wasm32-wasip1/release")
                .join(format!("{}.wasm", package_name));
            if alt.exists() {
                alt
            } else {
                return Err(anyhow::anyhow!(
                    "Build failed: Wasm artifact not found. Expected: {:?} or hyphenated variant",
                    wasm_path
                ));
            }
        };

        cliclack::log::success(format!("Artifact found: {:?}", wasm_path))?;
        let dest = bundle_dir.join("artifact.wasm");

        if self.no_optimize {
            std::fs::copy(&wasm_path, &dest)?;
            cliclack::log::warning("Optimization skipped by user.")?;
        } else {
            match self.optimize_wasm(&wasm_path, &bundle_dir) {
                Ok(opt_path) => {
                    std::fs::rename(opt_path, &dest)?;
                    cliclack::log::success("Optimized with wasm-tools strip.")?;
                }
                Err(e) => {
                    cliclack::log::warning(format!(
                        "Optimization failed ({}). Using unoptimized.",
                        e
                    ))?;
                    std::fs::copy(&wasm_path, &dest)?;
                }
            }
        }

        // --- Step B: Metadata ---
        let metadata_path = bundle_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&metadata_path, metadata_json)?;
        cliclack::log::info("Generated metadata.json (from env.toml/Cargo.toml)")?;

        // --- Step C: SBOM ---
        self.generate_sbom(&project_dir, &bundle_dir)?;

        cliclack::log::success(format!("Bundle created at {:?}", bundle_dir))?;
        cliclack::outro("Bundle complete.")?;

        Ok(bundle_dir)
    }

    fn build_component(&self, dir: &Path) -> Result<()> {
        cliclack::log::step("Building component (release)...")?;
        let status = Command::new("cargo")
            .arg("component")
            .arg("build")
            .arg("--release")
            .current_dir(dir)
            .status()
            .context("Failed to execute 'cargo component build'")?;
        if !status.success() {
            return Err(anyhow::anyhow!("Build failed"));
        }
        Ok(())
    }

    fn get_target_dir(&self, dir: &Path) -> Result<PathBuf> {
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .arg("--no-deps")
            .current_dir(dir)
            .output()?;
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let target_dir = json
            .get("target_directory")
            .and_then(|v| v.as_str())
            .context("No target_directory")?;
        Ok(PathBuf::from(target_dir))
    }

    fn parse_plugin_manifest(&self, dir: &Path) -> Result<PluginMetadata> {
        // Priority 1: env.toml (for metadata)
        let env_toml = dir.join("env.toml");
        let cargo_toml = dir.join("Cargo.toml");

        if env_toml.exists() {
            let mut meta = self.parse_env_toml(&env_toml)?;

            // IMPORTANT: Also merge Cargo.toml dependencies!
            if cargo_toml.exists() {
                let cargo_content = std::fs::read_to_string(&cargo_toml)?;
                let cargo_val = cargo_content.parse::<toml::Value>()?;
                let cargo_deps = self.extract_deps_toml(&cargo_val);

                // Merge: env.toml deps + Cargo.toml deps
                if let Some(ref mut env_deps) = meta.dependencies {
                    env_deps.extend(cargo_deps);
                } else {
                    meta.dependencies = Some(cargo_deps);
                }

                // Merge: repository from Cargo.toml if missing in env.toml
                if meta.repository.is_none() {
                    let pkg = cargo_val.get_table("package");
                    if let Some(p) = pkg {
                        if let Some(repo) = p.get_str("repository") {
                            meta.repository = Some(repo.to_string());
                        }
                    }
                }
            }

            return Ok(meta);
        }

        // Priority 2: env.json
        let env_json = dir.join("env.json");
        if env_json.exists() {
            let content = std::fs::read_to_string(&env_json)?;
            let val: serde_json::Value = serde_json::from_str(&content)?;
            let mut meta = self.extract_metadata_from_value(&val)?;
            meta.dependencies = Some(self.extract_deps_json(&val));
            return Ok(meta);
        }

        // Fallback: Cargo.toml only
        self.parse_cargo_manifest(dir)
    }

    fn parse_env_toml(&self, path: &Path) -> Result<PluginMetadata> {
        let content = std::fs::read_to_string(path)?;
        let val = content.parse::<toml::Value>()?;
        let mut meta = self.extract_metadata_from_value(&val)?;

        // Extract env.toml dependencies (plugin-to-plugin deps)
        meta.dependencies = Some(self.extract_deps_toml(&val));

        Ok(meta)
    }

    fn extract_metadata_from_value<V: GetValue>(&self, val: &V) -> Result<PluginMetadata> {
        let pkg = val
            .get_table("package")
            .context("Missing [package] table")?;
        let name = pkg.get_str("name").unwrap_or("unknown").to_string();
        let version = pkg.get_str("version").unwrap_or("0.0.0").to_string();
        let description = pkg.get_str("description").map(|s| s.to_string());
        let authors = pkg.get_array_of_strings("authors");
        let license = pkg.get_str("license").map(|s| s.to_string());
        let repository = pkg.get_str("repository").map(|s| s.to_string());

        // Capabilities from [plugin] table
        let capabilities = val
            .get_table("plugin")
            .and_then(|t| t.get_array_of_strings("capabilities"));

        Ok(PluginMetadata {
            name,
            version,
            description,
            authors,
            license,
            capabilities,
            dependencies: None,
            repository,
        })
    }

    fn parse_cargo_manifest(&self, dir: &Path) -> Result<PluginMetadata> {
        let content = std::fs::read_to_string(dir.join("Cargo.toml"))?;
        let val = content.parse::<toml::Value>()?;
        let mut meta = self.extract_metadata_from_value(&val)?;
        meta.dependencies = Some(self.extract_deps_toml(&val));
        Ok(meta)
    }

    fn extract_deps_toml(&self, val: &toml::Value) -> Vec<DependencyInfo> {
        let mut deps = Vec::new();
        let kinds = [
            ("dependencies", "runtime"),
            ("dev-dependencies", "dev"),
            ("build-dependencies", "build"),
            ("peer-dependencies", "peer"),
        ];

        for (table_key, kind) in kinds {
            if let Some(table) = val.get(table_key).and_then(|t| t.as_table()) {
                for (name, dep_val) in table {
                    let req = if let Some(s) = dep_val.as_str() {
                        // Simple version string: serde = "1.0"
                        s.to_string()
                    } else if let Some(t) = dep_val.as_table() {
                        // Table format: { version = "1.0", features = [...] }
                        // OR { path = "..." }
                        if let Some(v) = t.get("version").and_then(|v| v.as_str()) {
                            v.to_string()
                        } else if t.get("path").is_some() {
                            // Path dependency - use "*" or "workspace"
                            "path".to_string()
                        } else {
                            "*".to_string()
                        }
                    } else {
                        "*".to_string()
                    };

                    let purl = format!("pkg:cargo/{}", name);
                    deps.push(DependencyInfo {
                        purl,
                        kind: kind.to_string(),
                        req,
                    });
                }
            }
        }
        deps
    }

    fn extract_deps_json(&self, val: &serde_json::Value) -> Vec<DependencyInfo> {
        let mut deps = Vec::new();
        let kinds = [
            ("dependencies", "runtime"),
            ("devDependencies", "dev"),
            ("peerDependencies", "peer"),
        ];

        for (key, kind) in kinds {
            if let Some(obj) = val.get(key).and_then(|v| v.as_object()) {
                for (name, req_val) in obj {
                    let req = req_val.as_str().unwrap_or("*").to_string();
                    let purl = format!("pkg:env/{}", name);
                    deps.push(DependencyInfo {
                        purl,
                        kind: kind.to_string(),
                        req,
                    });
                }
            }
        }
        deps
    }

    fn generate_sbom(&self, project_dir: &Path, bundle_dir: &Path) -> Result<()> {
        let sbom_path = bundle_dir.join("sbom.spdx.json");

        // Check for cargo-sbom
        let status = Command::new("cargo").arg("sbom").arg("--version").output();

        let should_generate = match status {
            Ok(o) => o.status.success(),
            Err(_) => {
                // Not installed, prompt?
                // For Bundle command used in CI/Automation, maybe we shouldn't block?
                // But Verify phase implied interactivity.
                // Assuming "Recoomnded" implies we try to install if interactive.
                // For now, if missing, we just skip to fallback to verify logic first.
                // Actually, reinstall logic:
                let install = cliclack::confirm("cargo-sbom missing. Install?")
                    .initial_value(true)
                    .interact()
                    .unwrap_or(false);
                if install {
                    let _ = Command::new("cargo")
                        .arg("install")
                        .arg("cargo-sbom")
                        .status();
                    true
                } else {
                    false
                }
            }
        };

        if should_generate {
            cliclack::log::step("Generating SBOM (cargo sbom)...")?;

            // Fix for monorepos: cargo-sbom needs Cargo.lock in the current dir
            let local_lock = project_dir.join("Cargo.lock");
            let mut copied_lock = false;
            if !local_lock.exists() {
                if let Some(root_lock) = self.find_workspace_root_lock(project_dir) {
                    if let Ok(_) = std::fs::copy(&root_lock, &local_lock) {
                        copied_lock = true;
                    }
                }
            }

            let output_file = std::fs::File::create(&sbom_path)?;
            let run_status = Command::new("cargo")
                .arg("sbom")
                .arg("--output-format")
                .arg("spdx_json_2_3")
                .stdout(output_file) // Redirect stdout to file
                .current_dir(project_dir)
                .status();

            if copied_lock {
                let _ = std::fs::remove_file(&local_lock);
            }

            match run_status {
                Ok(s) => {
                    // CAUTION: If status is success but file is empty?
                    // Verify size
                    let meta = std::fs::metadata(&sbom_path)?;
                    if s.success() && meta.len() > 0 {
                        cliclack::log::success("Generated sbom.spdx.json")?;
                        return Ok(());
                    } else {
                        // Failure logic
                        cliclack::log::warning("cargo-sbom failed or produced empty file.")?;
                    }
                }
                Err(_) => {
                    cliclack::log::warning("Failed to execute cargo-sbom")?;
                }
            }
        }

        // Final cleanup of 0-byte file (re-check)
        if sbom_path.exists() {
            let _ = std::fs::remove_file(&sbom_path);
        }

        // Fallback: Cargo.lock
        let lock = project_dir.join("Cargo.lock");
        let valid_lock = if lock.exists() {
            Some(lock)
        } else {
            // Search parent (Workspace root)
            let mut current = project_dir.parent();
            let mut found = None;
            for _ in 0..4 {
                if let Some(p) = current {
                    let l = p.join("Cargo.lock");
                    if l.exists() {
                        found = Some(l);
                        break;
                    }
                    current = p.parent();
                } else {
                    break;
                }
            }
            found
        };

        if let Some(l) = valid_lock {
            std::fs::copy(&l, bundle_dir.join("Cargo.lock"))?;
            cliclack::log::info("Included Cargo.lock (SBOM fallback)")?;
        } else {
            cliclack::log::warning("No Cargo.lock found in project or workspace root.")?;
        }
        Ok(())
    }

    // ... optimize_wasm same as before ...
    fn optimize_wasm(&self, input: &Path, output_dir: &Path) -> Result<PathBuf> {
        let output = output_dir.join("plugin.opt.wasm"); // Temp name

        cliclack::log::step("Optimizing Wasm (wasm-tools strip)...")?;

        match Command::new("wasm-tools").arg("--version").output() {
            Ok(_) => self.run_strip(input, &output),
            Err(_) => {
                let install = cliclack::confirm("wasm-tools missing. Install via Homebrew?")
                    .initial_value(true)
                    .interact()?;
                if install {
                    cliclack::log::step("Installing wasm-tools...")?;
                    let status = Command::new("brew")
                        .arg("install")
                        .arg("wasm-tools")
                        .status()?;
                    if status.success() {
                        self.run_strip(input, &output)
                    } else {
                        Err(anyhow::anyhow!("Failed to install wasm-tools"))
                    }
                } else {
                    Err(anyhow::anyhow!("Skipped installation"))
                }
            }
        }
    }

    fn run_strip(&self, input: &Path, output: &Path) -> Result<PathBuf> {
        let status = Command::new("wasm-tools")
            .arg("strip")
            .arg(input)
            .arg("-o")
            .arg(output)
            .status()?;

        if status.success() {
            Ok(output.to_path_buf())
        } else {
            Err(anyhow::anyhow!("wasm-tools strip failed"))
        }
    }

    fn find_workspace_root_lock(&self, project_dir: &Path) -> Option<PathBuf> {
        let mut current = project_dir.parent();
        for _i in 0..5 {
            if let Some(p) = current {
                let l = p.join("Cargo.lock");
                if l.exists() {
                    cliclack::log::info(format!("Found workspace lockfile at: {:?}", l)).ok();
                    return Some(l);
                }
                current = p.parent();
            } else {
                break;
            }
        }
        None
    }
}

// Helper trait to unify toml::Value and serde_json::Value access for our simple schema
trait GetValue {
    fn get_table(&self, key: &str) -> Option<&Self>;
    fn get_str(&self, key: &str) -> Option<&str>;
    fn get_array_of_strings(&self, key: &str) -> Option<Vec<String>>;
}

impl GetValue for toml::Value {
    fn get_table(&self, key: &str) -> Option<&Self> {
        self.get(key)
    }
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }
    fn get_array_of_strings(&self, key: &str) -> Option<Vec<String>> {
        self.get(key).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    }
}

impl GetValue for serde_json::Value {
    fn get_table(&self, key: &str) -> Option<&Self> {
        self.get(key)
    }
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }
    fn get_array_of_strings(&self, key: &str) -> Option<Vec<String>> {
        self.get(key).and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    }
}
