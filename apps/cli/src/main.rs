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
    }

    Ok(())
}
