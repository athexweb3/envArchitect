use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct InitCommand {
    /// Name of the plugin to use (e.g. node, python, rust)
    #[arg(long, short)]
    pub plugin: Option<String>,

    /// Environment name
    #[arg(long, default_value = "my-env")]
    pub name: String,

    /// Force overwrite existing env.toml
    #[arg(long, short)]
    pub force: bool,
}

impl InitCommand {
    pub async fn execute(self) -> Result<()> {
        let _terminal = cliclack::intro("EnvArchitect Initializer")?;

        let plugin = if let Some(p) = self.plugin {
            p
        } else {
            cliclack::select("Select a plugin language to initialize:")
                .item("ts", "TypeScript", "")
                .item("rust", "Rust (Wasm)", "")
                .interact()?
                .to_string()
        };

        let adapter: Box<dyn crate::adapters::PluginAdapter> = match plugin.as_str() {
            "ts" => Box::new(crate::adapters::ts::TsAdapter::new()),
            "rust" => Box::new(crate::adapters::rust::RustAdapter::new()),
            _ => anyhow::bail!("Unsupported language: {}", plugin),
        };

        cliclack::log::step(format!("Initializing {} project...", plugin))?;
        let cwd = std::env::current_dir()?;
        adapter.scaffold(&cwd, &self.name).await?;

        cliclack::outro(format!("Initialized {} plugin project!", plugin))?;

        Ok(())
    }
}
