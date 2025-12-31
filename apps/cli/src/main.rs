pub mod ui;
use clap::{Parser, Subcommand};
use env_architect::domain::entities::tool::Tool;
use env_architect::infrastructure::adapters::brew::BrewAdapter;
// use application::install_tool;

#[derive(Parser)]
#[command(name = "env-architect")]
#[command(about = "Architect your developer environment", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a specific tool
    Install { name: String },
    /// Audit the system
    Audit,
    /// Login to the Registry
    Login,
    /// Publish a plugin to the Registry
    Publish {
        /// Path to the plugin manifest (env.toml)
        #[arg(short, long, default_value = "env.toml")]
        manifest: String,
    },
    /// Search for plugins
    Search {
        /// Query string
        query: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize adapters
    let brew = BrewAdapter::new();

    match cli.command {
        Commands::Install { name } => {
            let tool = Tool::new(&name);
            // In real app, we would use the application layer here
            // application::install_tool(tool, &brew).await?;
            println!("Simulating install of: {}", name);
        }
        Commands::Audit => {
            println!("Auditing system...");
        }
        Commands::Login => {
            println!("TODO: Implement Login Flow");
        }
        Commands::Publish { manifest } => {
            println!("TODO: Publish plugin from manifest: {}", manifest);
        }
        Commands::Search { query } => {
            println!("TODO: Search for: {}", query);
        }
    }

    Ok(())
}
