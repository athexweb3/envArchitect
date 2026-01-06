use anyhow::{Context, Result};
use clap::Args;
use cliclack::{intro, log, outro};
use database::models::User;
use reqwest::Client;
use url::Url;

#[derive(Args, Debug)]
pub struct WhoamiCommand {
    /// The registry URL to check
    #[arg(long, default_value = "http://localhost:3000")]
    pub registry: String,
}

impl WhoamiCommand {
    #[allow(dead_code)]
    pub async fn execute(&self) -> Result<()> {
        intro(console::style("EnvArchitect Identity").bold())?;

        let registry_url = Url::parse(&self.registry)?;

        // 1. Retrieve Token
        let token = match crate::utils::auth::get_token(&registry_url) {
            Ok(t) => t,
            Err(e) => {
                log::warning(format!("{}", e))?;
                return Ok(());
            }
        };

        // 2. Call /auth/me
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
