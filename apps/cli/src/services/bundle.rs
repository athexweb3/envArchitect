use crate::adapters::{self, PluginMetadata};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct DependencyInfo {
    pub purl: String,
    pub kind: String, // runtime, dev, build
    pub req: String,
}

#[derive(Serialize, Deserialize)]
pub struct BundleMetadata {
    #[serde(flatten)]
    pub base: PluginMetadata,
    pub dependencies: Option<Vec<DependencyInfo>>,
}

pub struct BundleService;

impl BundleService {
    pub async fn execute(path: &Path, no_optimize: bool) -> Result<PathBuf> {
        // Returns the path to the bundle directory (dist)
        cliclack::intro("EnvArchitect Bundle (Enterprise)")?;

        let project_dir = if path.is_file() {
            path.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            path.to_path_buf()
        };

        let project_dir = std::fs::canonicalize(&project_dir).context(format!(
            "Failed to resolve absolute path for {:?}",
            project_dir
        ))?;

        if !project_dir.exists() {
            return Err(anyhow::anyhow!("Project directory does not exist"));
        }

        cliclack::log::info(format!("Project Directory: {:?}", project_dir))?;

        let adapter = adapters::get_adapter(&project_dir)?;
        cliclack::log::info(format!("Detected language: {}", adapter.name()))?;

        let build_opts = adapters::BuildOptions {
            release: !no_optimize,
        };
        let wasm_path = adapter.build(&project_dir, build_opts).await?;
        cliclack::log::success(format!("Build artifact: {:?}", wasm_path))?;

        // Start with Adapter metadata
        let base_meta = adapter.metadata(&project_dir)?;

        // Enhance with env.toml or package.json
        let mut dependencies: Option<Vec<DependencyInfo>> = None;

        if project_dir.join("package.json").exists() {
            let pkg_meta = Self::parse_package_json(&project_dir.join("package.json"))?;
            // Adapter metadata (adapters/ts/metadata.rs) already extracts basic fields.

            dependencies = pkg_meta.dependencies;
        }

        let bundle_dir = project_dir.join("dist");
        if bundle_dir.exists() {
            std::fs::remove_dir_all(&bundle_dir)?;
            cliclack::log::info("Cleaned previous dist artifacts")?;
        }
        std::fs::create_dir(&bundle_dir)?;

        // Move/Optimize Artifact
        let dest = bundle_dir.join("artifact.wasm");

        if wasm_path.exists() {
            std::fs::copy(&wasm_path, &dest)?;
        } else {
            cliclack::log::error(format!("Wasm artifact not found at {:?}", wasm_path))?;
            return Err(anyhow::anyhow!("Build failed to produce artifact"));
        }

        // Write Metadata
        let final_meta = BundleMetadata {
            base: base_meta,
            dependencies,
        };
        let metadata_path = bundle_dir.join("metadata.json");
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&final_meta)?)?;

        // SBOM
        Self::generate_sbom(&project_dir, &bundle_dir)?;

        cliclack::log::success(format!("Bundle created at {:?}", bundle_dir))?;
        cliclack::outro("Bundle complete.")?;

        Ok(bundle_dir)
    }

    // parse_env_toml removed

    fn parse_package_json(path: &Path) -> Result<BundleMetadata> {
        let content = std::fs::read_to_string(path)?;
        let val: serde_json::Value = serde_json::from_str(&content)?;

        // Most fields are covered by adapter metadata extraction
        // We focus on dependencies here
        let deps = Self::extract_deps_package_json(&val);

        Ok(BundleMetadata {
            base: PluginMetadata {
                name: "placeholder".to_string(), // Ignored in merge
                version: "0.0.0".to_string(),
                description: None,
                authors: None,
                license: None,
                repository: None,
                capabilities: None,
            },
            dependencies: Some(deps),
        })
    }

    fn extract_deps_package_json(val: &serde_json::Value) -> Vec<DependencyInfo> {
        let mut deps = Vec::new();
        let kinds = [("dependencies", "runtime"), ("devDependencies", "dev")];
        for (table_key, kind) in kinds {
            if let Some(table) = val.get(table_key).and_then(|t| t.as_object()) {
                for (name, dep_val) in table {
                    let req = dep_val.as_str().unwrap_or("*").to_string();
                    let purl = format!("pkg:npm/{}", name);
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

    fn generate_sbom(project_dir: &Path, bundle_dir: &Path) -> Result<()> {
        let lock = project_dir.join("Cargo.lock");
        if lock.exists() {
            std::fs::copy(lock, bundle_dir.join("Cargo.lock")).ok();
        }
        Ok(())
    }
}
