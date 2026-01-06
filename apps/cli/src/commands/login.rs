use anyhow::{Context, Result};
use clap::Args;
use cliclack::{intro, log, outro, spinner};
use keyring::Entry;
use reqwest::Client;
use shared::dto::{AuthDeviceResponse, TokenResponse};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

#[derive(Args, Debug)]
pub struct LoginCommand {
    /// The registry URL to login to
    #[arg(long, default_value = "http://localhost:3000")]
    pub registry: String,
}

impl LoginCommand {
    #[allow(dead_code)]
    pub async fn execute(&self) -> Result<()> {
        intro(console::style("EnvArchitect Login").bold())?;

        let client = Client::new();
        let registry_url = Url::parse(&self.registry)?;

        // 1. Initiate Device Flow
        let initiate_url = registry_url.join("/oauth/device/code")?;
        log::info(format!("Connecting to {}...", initiate_url))?;

        let initiate_res: reqwest::Response = client
            .get(initiate_url)
            .send()
            .await
            .context("Failed to connect to registry")?;

        if !initiate_res.status().is_success() {
            let err = initiate_res.text().await?;
            return Err(anyhow::anyhow!("Registry error: {}", err));
        }

        let device_code_data: AuthDeviceResponse = initiate_res.json().await?;

        // 2. Display User Code and Open Browser
        println!(
            "\nFirst, copy your one-time code: {}",
            console::style(&device_code_data.user_code).bold().yellow()
        );
        println!(
            "Then press {} to open the browser at {}",
            console::style("Enter").bold(),
            console::style(&device_code_data.verification_uri).underlined()
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let portal_url = format!(
            "{}?user_code={}",
            device_code_data.verification_uri, device_code_data.user_code
        );
        if open::that(&portal_url).is_err() {
            log::warning(format!(
                "Failed to open browser. Please visit manually: {}",
                portal_url
            ))?;
        }

        // 3. Polling for Token
        let poll_url = registry_url.join("/oauth/token")?;
        let s = spinner();
        s.start("Waiting for authorization...");

        let token_response = loop {
            let res: reqwest::Response = client
                .get(poll_url.clone())
                .query(&[("device_code", &device_code_data.device_code)])
                .send()
                .await?;

            if res.status().is_success() {
                let body: serde_json::Value = res.json().await?;
                if body.get("status").and_then(|s| s.as_str()) == Some("pending") {
                    sleep(Duration::from_secs(device_code_data.interval)).await;
                    continue;
                }

                let token: TokenResponse = serde_json::from_value(body)?;
                break token;
            } else {
                let err = res.text().await?;
                return Err(anyhow::anyhow!("Auth failed: {}", err));
            }
        };

        s.stop("Authorized!");

        // 4. Securely store tokens
        self.store_tokens(
            &registry_url,
            &token_response.access_token,
            token_response.refresh_token.as_deref().unwrap_or(""),
        )
        .await?;

        // 5. Automatic Key Generation & Registration
        if let Err(e) = self
            .register_signing_key(&registry_url, &token_response.access_token)
            .await
        {
            log::warning(format!("Failed to register signing key: {}", e))?;
            log::info("You can still use the registry, but publishing will not work until you register a key.")?;
        } else {
            log::success("Signing key registered successfully!")?;
        }

        success_outro()?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn store_tokens(
        &self,
        registry_url: &Url,
        id_token: &str,
        refresh_token: &str,
    ) -> Result<()> {
        let domain = registry_url.host_str().unwrap_or("localhost");

        // Use Keyring for Access Token
        let entry = Entry::new("env-architect", domain)?;
        entry
            .set_password(id_token)
            .context("Failed to save token to keyring")?;

        // Fallback/Secondary storage (hosts.toml) - Simplified for now
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".config").join("env-architect");
        std::fs::create_dir_all(&config_dir)?;

        // We still save a hosts.toml for metadata/refresh tokens if needed
        let hosts_path = config_dir.join("hosts.toml");

        // Fetch username from server if possible, or use a placeholder/session info
        // For now, let's assume the server includes the username in TokenResponse or we fetch it
        // Actually, TokenResponse doesn't have it yet. I should add it or fetch it.
        // Let's fetch it via /auth/me since we have the token now.
        let client = Client::new();
        let me_url = registry_url.join("/auth/me")?;
        let username = if let Ok(res) = client
            .get(me_url)
            .header("Authorization", format!("Bearer {}", id_token))
            .send()
            .await
        {
            if let Ok(user) = res.json::<database::models::User>().await {
                user.username
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        let content = format!(
            "[\"{}\"]\nuser = \"{}\"\noauth_token = \"{}\"\nrefresh_token = \"{}\"\n",
            domain, username, id_token, refresh_token
        );
        std::fs::write(hosts_path, content)?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn register_signing_key(&self, registry: &Url, token: &str) -> Result<()> {
        use crate::keys::{generate_or_load_signing_key, get_public_key_base64};
        use shared::dto::{RegisterKeyRequest, RegisterKeyResponse};

        log::step("Registering signing key...")?;

        // Generate or load existing key from OS Keychain
        let signing_key = generate_or_load_signing_key()
            .context("Failed to generate/load signing key from keychain")?;

        let public_key = get_public_key_base64(&signing_key);

        // Register with server
        let client = Client::new();
        let register_url = registry.join("/auth/register-key")?;

        let response: reqwest::Response = client
            .post(register_url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&RegisterKeyRequest { public_key })
            .send()
            .await
            .context("Failed to send key registration request")?;

        if response.status().is_success() {
            let _result: RegisterKeyResponse = response
                .json()
                .await
                .context("Failed to parse registration response")?;
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Key registration failed ({}): {}",
                status,
                error_text
            ))
        }
    }
}

#[allow(dead_code)]
fn success_outro() -> Result<()> {
    outro(console::style("Login successful! You can now publish and search for plugins.").green())?;
    Ok(())
}
