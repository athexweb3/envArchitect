use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use url::Url;

// Import application services
// Use application crate directly
use application::InstallService;

mod commands;
mod core;
mod host;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install an environment from a file (or registry)
    Install {
        /// Path to the environment file (env.toml/json/yaml). If not provided, searches current directory.
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Force re-resolution
        #[arg(long, short)]
        force: bool,
    },

    /// Resolve an environment using a WASM plugin (Host Runtime Check)
    Resolve(commands::resolve::ResolveCommand),

    /// Publish a package to the registry
    Publish {
        /// Path to the package manifest (env.toml)
        #[arg(default_value = "env.toml")]
        path: PathBuf,
    },

    /// Search for plugins
    Search {
        /// Query string
        query: String,
    },

    /// Audit the system
    Audit,

    /// Login to the Registry
    Login,

    /// Hot-reloading development loop for plugins
    Dev(commands::dev::DevCommand),

    /// Activate a project environment in a new shell
    Shell(commands::shell::ShellCommand),

    /// Run a command in the project context
    Run(commands::run::RunCommand),

    /// [Internal] The architect shim proxy
    #[command(hide = true)]
    Shim {
        /// The name of the tool being shimmed
        tool: String,
        /// Arguments for the tool
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging/tracing if needed
    // tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install { path, force } => {
            cliclack::intro(console::style("EnvArchitect Install").bold())?;
            if force {
                cliclack::log::warning("Force mode enabled")?;
            }

            let (_manifest_path, manifest) = match path {
                Some(p) => {
                    cliclack::log::step(format!("Loading manifest from: {:?}", p))?;
                    (p.clone(), utils::loader::load_manifest(&p)?)
                }
                None => {
                    cliclack::log::step("Searching for manifest in current directory...")?;
                    utils::loader::find_and_load_manifest(&std::env::current_dir()?)?
                }
            };

            cliclack::log::success(format!("Loaded project: {}", &manifest.project.name))?;

            let registry_url =
                Url::parse("https://registry.env-architect.dev").context("Invalid registry URL")?;

            let tuf_root = PathBuf::from(".env-architect/tuf");
            let tuf_cache = PathBuf::from(".env-architect/cache");

            // Create cache directories
            std::fs::create_dir_all(&tuf_root)?;
            std::fs::create_dir_all(&tuf_cache)?;

            let mut service = InstallService::new(registry_url, tuf_root, tuf_cache)?;
            
            // TODO: Wrap this in a spinner if possible, or let it log
            // Execute installation from manifest
            service.install_from_manifest(manifest).await?;
            
            cliclack::outro("Installation complete")?;
        }
        Commands::Resolve(cmd) => {
            cmd.execute().await?;
        }
        Commands::Publish { path } => {
            cliclack::intro("EnvArchitect Publish")?;
            cliclack::log::info(format!("TODO: Publish package from manifest: {:?}", path))?;
            cliclack::outro("Done")?;
        }
        Commands::Search { query } => {
            cliclack::intro("EnvArchitect Search")?;
            cliclack::log::info(format!("Searching for: {}", query))?;
            cliclack::outro("Done")?;
        }
        Commands::Audit => {
            cliclack::intro("EnvArchitect Audit")?;
            cliclack::log::info("Auditing system...")?;
            cliclack::outro("Audit complete")?;
        }
        Commands::Login => {
            cliclack::intro("Login")?;
            cliclack::log::info("Login flow initiated")?;
            cliclack::outro("You are logged in")?;
        }
        Commands::Dev(cmd) => {
            cmd.execute().await?;
        }
        Commands::Shell(cmd) => {
            cmd.execute().await?;
        }
        Commands::Run(cmd) => {
            cmd.execute().await?;
        }
        Commands::Shim { tool, args } => {
            commands::shim::execute_shim(tool, args).await?;
        }
    }

    Ok(())
}
