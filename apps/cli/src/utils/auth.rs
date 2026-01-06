use anyhow::{Context, Result};
use keyring::Entry;
use url::Url;

/// Retrieves the authentication token for a given registry URL.
/// First checks the system keyring, then falls back to hosts.toml.
pub fn get_token(registry_url: &Url) -> Result<String> {
    let domain = registry_url.host_str().unwrap_or("localhost");

    // 1. Try Keyring
    let entry = Entry::new("env-architect", domain).context("Failed to access keyring")?;
    if let Ok(token) = entry.get_password() {
        return Ok(token);
    }

    // 2. Fallback to hosts.toml
    let home = dirs::home_dir().context("Could not find home directory")?;
    let hosts_path = home
        .join(".config")
        .join("env-architect")
        .join("hosts.toml");

    if hosts_path.exists() {
        let content = std::fs::read_to_string(&hosts_path)
            .with_context(|| format!("Failed to read hosts file at {:?}", hosts_path))?;

        let hosts: serde_json::Value =
            toml::from_str(&content).with_context(|| "Failed to parse hosts.toml")?;

        if let Some(host_data) = hosts.get(domain) {
            if let Some(token) = host_data.get("oauth_token") {
                if let Some(token_str) = token.as_str() {
                    return Ok(token_str.to_string());
                }
            }
        }
    }

    // 3. Last resort: check hosts.json (legacy)
    let hosts_json_path = home
        .join(".config")
        .join("env-architect")
        .join("hosts.json");

    if hosts_json_path.exists() {
        let content = std::fs::read_to_string(&hosts_json_path)?;
        let hosts: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(host_data) = hosts.get(domain) {
            if let Some(token) = host_data.get("oauth_token") {
                if let Some(token_str) = token.as_str() {
                    return Ok(token_str.to_string());
                }
            }
        }
    }

    anyhow::bail!(
        "Not logged in. Please run 'env login' to authenticate with {}",
        domain
    );
}
