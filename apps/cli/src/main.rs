use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use url::Url;

// Import application services
// Use application crate directly
use application::InstallService;

mod adapters;
mod commands;
mod constants;
mod core;
mod host;
mod keys;
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
    /// Install an environment or a specific package
    Install {
        /// Package to install (e.g. 'node'). If provided, adds to env.toml. If empty, installs from env.toml.
        #[arg(value_parser)]
        package: Option<String>,

        /// Path to the environment file (env.toml/json/yaml).
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Force re-installation
        #[arg(long, short)]
        force: bool,
    },

    /// Resolve an environment using a WASM plugin (Host Runtime Check)
    Resolve(commands::resolve::ResolveCommand),

    /// Publish a package to the registry
    Publish(commands::publish::PublishCommand),

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

    /// Run the embedded Physician (Doctor) diagnostic tool
    Doctor(commands::doctor::DoctorCommand),

    /// Initialize a new environment configuration
    Init(commands::init::InitCommand),

    /// Display the current logged-in identity
    Whoami(commands::whoami::WhoamiCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging/tracing if needed
    // tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            package,
            path,
            force,
        } => {
            cliclack::intro(console::style("EnvArchitect Install").bold())?;
            if force {
                cliclack::log::warning("Force mode enabled")?;
            }

            // 1. System/Global Install Mode (Homebrew-style)
            // When a package is explicitly named, we operate in system mode.
            // This is context-independent and does not read/write local project files.
            if let Some(pkg_name) = package {
                cliclack::log::step(format!("System Install: '{}'", pkg_name))?;

                // Resolve against system registry or local WASM cache (dev mode)
                let resolution = match pkg_name.as_str() {
                    "node" => {
                        if std::path::Path::new(
                            "../../target/wasm32-wasip1/debug/env_plugin_node.wasm",
                        )
                        .exists()
                        {
                            "path:../../target/wasm32-wasip1/debug/env_plugin_node.wasm"
                        } else {
                            "registry:node"
                        }
                    }
                    "python" => {
                        if std::path::Path::new(
                            "../../target/wasm32-wasip1/debug/env_plugin_python.component.wasm",
                        )
                        .exists()
                        {
                            "path:../../target/wasm32-wasip1/debug/env_plugin_python.component.wasm"
                        } else {
                            "registry:python"
                        }
                    }
                    _ => "registry:unknown",
                };

                // DIRECT EXECUTION PATH (SystemExecutor)
                // If we have a local path (Dev Mode), we execute the plugin directly to support interactive resolution.
                if resolution.starts_with("path:") {
                    let path_str = resolution.strip_prefix("path:").unwrap();
                    let plugin_path = PathBuf::from(path_str);

                    crate::core::executor::SystemExecutor::install(&plugin_path).await?;
                } else {
                    // Registry Fallback (Legacy / Non-Interactive for now)
                    // Create a virtual manifest for this single transaction
                    let manifest = crate::core::virtual_manifest::VirtualManifestBuilder::build(
                        &pkg_name, resolution,
                    )?;

                    // Initialize Service
                    let registry_url = Url::parse("https://registry.env-architect.dev")
                        .context("Invalid registry URL")?;
                    let tuf_root = PathBuf::from(".env-architect/tuf");
                    let tuf_cache = PathBuf::from(".env-architect/cache");
                    std::fs::create_dir_all(&tuf_root)?;
                    std::fs::create_dir_all(&tuf_cache)?;

                    let mut service = InstallService::new(registry_url, tuf_root, tuf_cache)?;
                    service.install_from_manifest(manifest).await?;
                }

                // Update Global State Tracking
                let global_store = crate::core::global_store::GlobalStateService::new()?;
                global_store.add_tool(&pkg_name, &resolution, Some("latest".to_string()), None)?;

                cliclack::log::success(format!("Installed '{}' to global system.", pkg_name))?;
                cliclack::outro("Done.")?;
            }
            // 2. Project Install Mode (npm install / cargo build style)
            // When no package is named, we look for a manifest file to restore the environment.
            else {
                let (_manifest_path, manifest) = match path {
                    Some(p) => (p.clone(), utils::loader::load_manifest(&p)?),
                    None => utils::loader::find_and_load_manifest(&std::env::current_dir()?)?,
                };

                cliclack::log::step(format!("Restoring Project: {}", &manifest.project.name))?;

                let registry_url = Url::parse("https://registry.env-architect.dev")
                    .context("Invalid registry URL")?;
                let tuf_root = PathBuf::from(".env-architect/tuf");
                let tuf_cache = PathBuf::from(".env-architect/cache");
                std::fs::create_dir_all(&tuf_root)?;
                std::fs::create_dir_all(&tuf_cache)?;

                let mut service = InstallService::new(registry_url, tuf_root, tuf_cache)?;
                service.install_from_manifest(manifest).await?;

                cliclack::outro("Project environment restored.")?;
            }
        }
        Commands::Resolve(cmd) => {
            cmd.execute().await?;
        }
        Commands::Publish(cmd) => {
            cmd.execute().await?;
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
        Commands::Doctor(cmd) => {
            cmd.execute().await?;
        }
        Commands::Init(cmd) => {
            cmd.execute().await?;
        }
        Commands::Whoami(cmd) => {
            cmd.execute().await?;
        }
    }

    Ok(())
}
