use anyhow::{Context, Result};
use env_architect::domain::entities::manifest::global::GlobalManifest;
use std::fs;
use std::path::PathBuf;

pub struct GlobalStateService {
    // config_dir: PathBuf, // Removed unused field
    manifest_path: PathBuf,
}

impl GlobalStateService {
    /// Initialize the service, ensuring the config directory exists.
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".env-architect");
        let manifest_path = config_dir.join("global.env.toml");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config dir")?;
        }

        Ok(Self { manifest_path })
    }

    /// Load the global manifest. Returns default if it doesn't exist.
    pub fn load(&self) -> Result<GlobalManifest> {
        if !self.manifest_path.exists() {
            return Ok(GlobalManifest::default());
        }

        let content =
            fs::read_to_string(&self.manifest_path).context("Failed to read global manifest")?;

        toml::from_str(&content).context("Failed to parse global manifest")
    }

    /// Save the global manifest to disk.
    pub fn save(&self, manifest: &GlobalManifest) -> Result<()> {
        let content =
            toml::to_string_pretty(manifest).context("Failed to serialize global manifest")?;

        fs::write(&self.manifest_path, content).context("Failed to write global manifest")?;

        Ok(())
    }

    /// Add a tool to the global registry.
    pub fn add_tool(
        &self,
        name: &str,
        source: &str,
        version: Option<String>,
        signature: Option<String>,
    ) -> Result<()> {
        let mut manifest = self.load()?;
        use env_architect::domain::entities::manifest::global::GlobalTool;

        let tool = GlobalTool {
            source: source.to_string(),
            version,
            signature,
            installed_at: Some(chrono::Utc::now().to_rfc3339()),
        };

        manifest.tools.insert(name.to_string(), tool);
        self.save(&manifest)?;
        Ok(())
    }
}
