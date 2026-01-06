use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use oci_client::client::ImageLayer;
use oci_client::manifest::{OciDescriptor, OciImageManifest};
use oci_client::{secrets::RegistryAuth, Client, Reference};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

use crate::commands::bundle::BundleCommand;

#[derive(Parser)]
pub struct PublishCommand {
    /// Path to the package manifest (e.g. Cargo.toml or directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Skip optimization step
    #[arg(long)]
    pub no_optimize: bool,

    /// Dry run (build and bundle only, do not publish)
    #[arg(long)]
    pub dry_run: bool,

    /// Target registry URL (e.g. ghcr.io/username/package)
    #[arg(long)]
    pub target: Option<String>,
}

use shared::dto::{DependencyPayload as DependencyInfo, PublishPayload};

#[derive(Deserialize, Serialize)]
struct MetadataFile {
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    dependencies: Option<Vec<DependencyInfo>>,
    #[serde(default)]
    repository: Option<String>,
}

#[allow(unused)]
impl PublishCommand {
    pub async fn execute(&self) -> Result<()> {
        // Delegate to BundleCommand
        let bundle_cmd = BundleCommand {
            path: self.path.clone(),
            no_optimize: self.no_optimize,
        };

        // This will Build + Optimize + Create dist/
        let bundle_dir = bundle_cmd.execute().await?;

        if self.dry_run {
            return Ok(());
        }

        cliclack::intro("EnvArchitect Publish")?;

        let access_token = self.get_access_token()?;

        let user = self.get_user_profile(&access_token).await?;
        let username = user.username;

        let package_name = self.get_package_name(&self.path)?;

        let meta_path = bundle_dir.join("metadata.json");
        let content = std::fs::read_to_string(&meta_path)?;
        let meta: MetadataFile = serde_json::from_str(&content)?;

        let mut ghcr_token = if let Ok(t) = std::env::var("GITHUB_TOKEN") {
            Some(t)
        } else {
            cliclack::log::step("Fetching GHCR token from server...")?;
            let api_url =
                std::env::var("REGISTRY_API_URL").unwrap_or("http://localhost:3000".to_string());
            let client = reqwest::Client::new();
            let res: reqwest::Response = client
                .get(format!("{}/auth/ghcr-token", api_url))
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if res.status().is_success() {
                let body: serde_json::Value = res.json().await?;
                body.get("token")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        };

        if ghcr_token.is_none() {
            cliclack::log::warning("GitHub Container Registry (GHCR) requires a Personal Access Token (PAT) for direct push.")?;
            cliclack::log::info("Why? GitHub restricts third-party Apps from pushing directly to namespaces for security.")?;

            let shortcut_url = "https://github.com/settings/tokens/new?scopes=write:packages&description=EnvArchitect+Publish+Token";
            cliclack::log::info(format!(
                "Please create a 'Classic PAT' with 'write:packages' scope here:\n  {}",
                shortcut_url
            ))?;

            let pat: String = cliclack::input("Paste your GitHub PAT:")
                .validate(|input: &String| {
                    if input.starts_with("ghp_") || input.starts_with("github_pat_") {
                        Ok(())
                    } else {
                        Err("Invalid token format. It should start with ghp_ or github_pat_")
                    }
                })
                .interact()?;

            // Upload PAT to server
            let api_url =
                std::env::var("REGISTRY_API_URL").unwrap_or("http://localhost:3000".to_string());
            let client = reqwest::Client::new();
            let res: reqwest::Response = client
                .put(format!("{}/auth/ghcr-token", api_url))
                .header("Authorization", format!("Bearer {}", access_token))
                .json(&serde_json::json!({ "pat": pat }))
                .send()
                .await?;

            if res.status().is_success() {
                cliclack::log::success("PAT securely stored on server.")?;
            } else {
                cliclack::log::error(
                    "Failed to store PAT on server. Using it for this session only.",
                )?;
            }
            ghcr_token = Some(pat);
        }

        let oci_ref = self
            .publish_to_registry(
                &username,
                &package_name,
                &meta.version,
                meta.description.clone(),
                meta.repository.clone(),
                &bundle_dir,
                ghcr_token,
            )
            .await?;

        self.upload_to_api(
            &package_name,
            &bundle_dir,
            meta,
            Some(oci_ref),
            &access_token,
        )
        .await?;

        cliclack::outro("Publishing complete! 🎉")?;
        Ok(())
    }

    fn get_access_token(&self) -> Result<String> {
        let api_url =
            std::env::var("REGISTRY_API_URL").unwrap_or("http://localhost:3000".to_string());

        let registry_url = url::Url::parse(&api_url)?;
        let domain = registry_url.host_str().unwrap_or("localhost");

        let entry =
            keyring::Entry::new("env-architect", domain).context("Failed to access keyring")?;
        match entry.get_password() {
            Ok(t) => Ok(t),
            Err(_) => {
                let home = dirs::home_dir().context("Could not find home directory")?;
                let hosts_path = home
                    .join(".config")
                    .join("env-architect")
                    .join("hosts.json");

                if hosts_path.exists() {
                    let content = std::fs::read_to_string(&hosts_path)?;
                    let hosts: serde_json::Value = serde_json::from_str(&content)?;
                    hosts
                        .get(domain)
                        .and_then(|h| h.get("oauth_token"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                        .context("No access token found in hosts.json")
                } else {
                    Err(anyhow::anyhow!("Not logged in. Please run 'env login'"))
                }
            }
        }
    }

    fn get_package_name(&self, path: &Path) -> Result<String> {
        let dir = if path.is_file() {
            path.parent().unwrap()
        } else {
            path
        };

        use crate::constants::MANIFEST_JSON;
        let env_json = dir.join(MANIFEST_JSON);
        if env_json.exists() {
            let content = std::fs::read_to_string(&env_json)?;
            let val: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(n) = val
                .get("project") // 'project' in json, 'package' in toml usually but schema says project
                .and_then(|p| p.get("name"))
                .and_then(|s| s.as_str())
            {
                return Ok(n.to_string());
            }
        }

        let pkg_json = dir.join("package.json");
        if pkg_json.exists() {
            let content = std::fs::read_to_string(&pkg_json)?;
            let val: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(n) = val.get("name").and_then(|s| s.as_str()) {
                return Ok(n.to_string());
            }
        }

        Err(anyhow::anyhow!(
            "Could not find package name in env.json or package.json"
        ))
    }

    async fn publish_to_registry(
        &self,
        username: &str,
        package_name: &str,
        package_version: &str,
        description: Option<String>,
        repository: Option<String>,
        bundle_dir: &Path,
        ghcr_token: Option<String>,
    ) -> Result<String> {
        let (registry_url, target_url) = if let Some(t) = &self.target {
            if t.contains('/') {
                (t.clone(), format!("{}:{}", t, package_version))
            } else {
                let full = format!("{}/{}:{}", t, package_name, package_version);
                (t.clone(), full)
            }
        } else {
            let user_default = format!("ghcr.io/{}/{}", username, package_name);
            let input: String = cliclack::input("Registry URL:")
                .placeholder(&user_default)
                .default_input(&user_default)
                .interact()?;
            (input.clone(), format!("{}:{}", input, package_version))
        };

        let auth = if let Some(t) = ghcr_token {
            RegistryAuth::Basic(username.to_string(), t)
        } else {
            cliclack::log::warning(
                "No GITHUB_TOKEN or server-provided token. Auth will likely fail for GHCR.",
            )?;
            RegistryAuth::Anonymous
        };

        cliclack::log::step(format!("Pushing to {}...", target_url))?;

        let reference: Reference = target_url.parse().context("Invalid registry URL")?;
        let client = Client::new(oci_client::client::ClientConfig::default());

        let mut annotations = BTreeMap::new();

        if let Some(repo) = repository {
            annotations.insert("org.opencontainers.image.source".to_string(), repo);
        } else {
            annotations.insert(
                "org.opencontainers.image.source".to_string(),
                format!("https://github.com/{}/envArchitect", username),
            );
        }

        if let Some(desc) = description {
            annotations.insert("org.opencontainers.image.description".to_string(), desc);
        } else {
            annotations.insert(
                "org.opencontainers.image.description".to_string(),
                format!("EnvArchitect Plugin: {}", package_name),
            );
        }

        let annotations_map = Some(annotations.clone());

        fn calculate_descriptor(
            media_type: String,
            data: &[u8],
            annotations: Option<BTreeMap<String, String>>,
        ) -> OciDescriptor {
            let digest = Sha256::digest(data);
            let digest_str = format!("sha256:{}", hex::encode(digest));
            OciDescriptor {
                media_type,
                digest: digest_str,
                size: data.len() as i64,
                annotations,
                urls: None,
            }
        }

        let config_data = b"{}";
        let config_desc = calculate_descriptor(
            "application/vnd.wasm.config.v0+json".to_string(),
            config_data,
            annotations_map.clone(),
        );

        let mut layers_data = Vec::new();
        let mut layers_desc = Vec::new();

        let wasm_path = bundle_dir.join("artifact.wasm");
        let wasm_data = std::fs::read(&wasm_path).context("Failed to read artifact.wasm")?;
        let wasm_desc = calculate_descriptor(
            "application/vnd.w3c.wasm.component.v1+wasm".to_string(),
            &wasm_data,
            None,
        );

        layers_data.push(ImageLayer {
            data: wasm_data,
            media_type: "application/vnd.w3c.wasm.component.v1+wasm".to_string(),
            annotations: None,
        });
        layers_desc.push(wasm_desc);

        let meta_path = bundle_dir.join("metadata.json");
        let meta_data = std::fs::read(&meta_path).context("Failed to read metadata.json")?;
        let meta_desc = calculate_descriptor(
            "application/vnd.env-architect.metadata.v1+json".to_string(),
            &meta_data,
            None,
        );

        layers_data.push(ImageLayer {
            data: meta_data,
            media_type: "application/vnd.env-architect.metadata.v1+json".to_string(),
            annotations: None,
        });
        layers_desc.push(meta_desc);

        let sbom_path = bundle_dir.join("sbom.spdx.json");
        if sbom_path.exists() {
            let sbom_data = std::fs::read(&sbom_path)?;
            let sbom_desc = calculate_descriptor(
                "application/vnd.env-architect.sbom.v1+json".to_string(),
                &sbom_data,
                None,
            );

            layers_data.push(ImageLayer {
                data: sbom_data,
                media_type: "application/vnd.env-architect.sbom.v1+json".to_string(),
                annotations: None,
            });
            layers_desc.push(sbom_desc);
        }

        let manifest = OciImageManifest {
            schema_version: 2,
            media_type: Some("application/vnd.oci.image.manifest.v1+json".to_string()),
            config: config_desc,
            layers: layers_desc,
            annotations: annotations_map.clone(),
            artifact_type: None,
        };

        let config = oci_client::client::Config {
            data: config_data.to_vec(),
            media_type: "application/vnd.wasm.config.v0+json".to_string(),
            annotations: annotations_map,
        };

        match client
            .push(&reference, &layers_data, config, &auth, Some(manifest))
            .await
        {
            Ok(res) => Ok(target_url),
            Err(e) => {
                cliclack::log::error(format!("Failed to push to registry: {}", e))?;
                Err(e.into())
            }
        }
    }

    async fn upload_to_api(
        &self,
        package_name: &str,
        bundle_dir: &Path,
        meta: MetadataFile,
        oci_reference: Option<String>,
        access_token: &str,
    ) -> Result<()> {
        use crate::keys::{generate_or_load_signing_key, sign_bytes_base64};

        cliclack::log::step("Preparing upload to registry...")?;

        let signing_key = generate_or_load_signing_key()
            .context("Failed to load signing key. Please run 'env login' again.")?;

        let wasm_path = bundle_dir.join("artifact.wasm");
        let wasm_bytes = std::fs::read(&wasm_path).context("Failed to read artifact.wasm")?;

        let signature_b64 = sign_bytes_base64(&signing_key, &wasm_bytes);

        let api_url =
            std::env::var("REGISTRY_API_URL").unwrap_or("http://localhost:3000".to_string());

        let version = meta.version.clone();
        let purl = format!("pkg:env/{}@{}", package_name, version);

        let payload = PublishPayload {
            name: meta.name.clone(),
            version,
            description: meta.description,
            ecosystem: "env".to_string(),
            purl: purl.clone(),
            dependencies: meta.dependencies.unwrap_or_default(),
            oci_reference,
        };

        let client = reqwest::Client::new();

        cliclack::log::step(format!("Publishing to {}...", api_url))?;

        let metadata_json = serde_json::to_string(&payload)?;

        let mut form = reqwest::multipart::Form::new()
            .text("metadata", metadata_json)
            .part(
                "file",
                reqwest::multipart::Part::bytes(wasm_bytes)
                    .file_name("artifact.wasm")
                    .mime_str("application/wasm")?,
            );

        let sbom_path = bundle_dir.join("sbom.spdx.json");
        if sbom_path.exists() {
            let sbom_bytes = std::fs::read(&sbom_path)?;
            form = form.part(
                "sbom",
                reqwest::multipart::Part::bytes(sbom_bytes)
                    .file_name("sbom.spdx.json")
                    .mime_str("application/json")?,
            );
        }

        let res: reqwest::Response = client
            .post(format!("{}/v1/publish", api_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("X-Signature", signature_b64)
            .multipart(form)
            .send()
            .await?;

        if res.status().is_success() {
            let response: serde_json::Value = res.json().await?;
            cliclack::log::success("Published successfully!")?;

            if let Some(id) = response.get("id") {
                cliclack::log::info(format!("Component ID: {}", id))?;
            }
            if let Some(msg) = response.get("message") {
                let msg_str = msg.as_str().unwrap_or("");
                if !msg_str.contains("Published successfully") {
                    cliclack::log::info(format!("{}", msg))?;
                }
            }

            cliclack::log::info("Artifact analysis initiated (Notary Scan).")?;
            cliclack::log::warning(
                "Note: Package is Private by default. Make it Public in GitHub Settings to share.",
            )?;
        } else {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_default();
            cliclack::log::error(format!("Failed to publish ({}): {}", status, error_text))?;
            return Err(anyhow::anyhow!("Publish failed"));
        }

        Ok(())
    }
    async fn get_user_profile(&self, access_token: &str) -> Result<database::models::User> {
        let api_url =
            std::env::var("REGISTRY_API_URL").unwrap_or("http://localhost:3000".to_string());
        let client = reqwest::Client::new();
        let res: reqwest::Response = client
            .get(format!("{}/auth/me", api_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to fetch user profile")?;

        if res.status().is_success() {
            let user: database::models::User = res.json().await?;
            Ok(user)
        } else {
            Err(anyhow::anyhow!("Failed to retrieve user profile"))
        }
    }
}
