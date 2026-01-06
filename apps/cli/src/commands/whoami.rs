use anyhow::{Context, Result};
use clap::Args;
use cliclack::{intro, log, outro};
use database::models::User;
use keyring::Entry;
use reqwest::Client;
use url::Url;

#[derive(Args, Debug)]
pub struct WhoamiCommand {
    /// The registry URL to check
    #[arg(long, default_value = "http://localhost:3000")]
    pub registry: String,
}

#[allow(unused)]
impl WhoamiCommand {
    pub async fn execute(&self) -> Result<()> {
        intro(console::style("EnvArchitect Identity").bold())?;

        let registry_url = Url::parse(&self.registry)?;
        let domain = registry_url.host_str().unwrap_or("localhost");

        let entry = Entry::new("env-architect", domain)?;
        let token = match entry.get_password() {
            Ok(t) => t,
            Err(e) => {
                log::info(format!(
                    "Keyring retrieval failed ({}), checking hosts.json...",
                    e
                ))?;

                let home = dirs::home_dir().context("Could not find home directory")?;
                let hosts_path = home
                    .join(".config")
                    .join("env-architect")
                    .join("hosts.json");

                if hosts_path.exists() {
                    let content = std::fs::read_to_string(hosts_path)?;
                    let hosts: serde_json::Value = serde_json::from_str(&content)?;

                    if let Some(host_data) = hosts.get(domain) {
                        // Show local identity if available
                        if let Some(user) = host_data.get("user").and_then(|u| u.as_str()) {
                            log::info(format!(
                                "Identified locally as: {}",
                                console::style(user).bold().cyan()
                            ))?;
                        }

                        if let Some(t) = host_data.get("oauth_token").and_then(|t| t.as_str()) {
                            if t != "...keyring..." {
                                t.to_string()
                            } else {
                                log::warning("Token points back to keyring. Please login again.")?;
                                return Ok(());
                            }
                        } else {
                            log::warning("No token found in hosts.json for this domain.")?;
                            return Ok(());
                        }
                    } else {
                        log::warning(format!("No entry for {} found in hosts.json.", domain))?;
                        return Ok(());
                    }
                } else {
                    log::warning("No session found in keyring or hosts.json. Please login.")?;
                    return Ok(());
                }
            }
        };

        let client = Client::new();
        let me_url = registry_url.join("/auth/me")?;

        let res = client
            .get(me_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to connect to registry")?;

        if res.status().is_success() {
            let user: User = res.json().await?;
            log::info(format!(
                "Logged in as: {}",
                console::style(&user.username).bold().cyan()
            ))?;
            if let Some(email) = user.email {
                log::info(format!("Email: {}", email))?;
            }
            log::info(format!("GitHub ID: {}", user.github_id))?;
        } else {
            let err = res.text().await?;
            log::error(format!(
                "Failed to retrieve identity: {}. You might need to login again.",
                err
            ))?;
        }

        outro("Identity check complete.")?;
        Ok(())
    }
}
