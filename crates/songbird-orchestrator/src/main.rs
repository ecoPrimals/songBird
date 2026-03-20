// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird - Network Orchestration & Discovery Primal
//!
//! `UniBin` Architecture (Ecosystem Standard v1.0.0)
//! Main entry point with subcommand structure for different operational modes

use anyhow::Result;
use clap::{Parser, Subcommand};
use songbird_orchestrator::commands::{self, ConfigAction};

/// Songbird - Network Orchestration & Discovery Primal
///
/// `UniBin` Architecture: One binary, multiple modes
#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Network Orchestration & Discovery Primal", long_about = None)]
#[command(version)]
#[command(author = "ecoPrimals <contact@ecoprimals.dev>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird orchestrator in server mode
    ///
    /// This is the primary operational mode that runs the full orchestrator
    /// with discovery, federation, and network services.
    Server {
        /// HTTP server port (environment-aware)
        /// Respects `SONGBIRD_HTTP_PORT`, `SONGBIRD_PORT`, or PORT
        #[arg(long, short, default_value_t = songbird_orchestrator::env_config::http_port())]
        port: u16,

        /// Run as daemon (background process)
        #[arg(long, short)]
        daemon: bool,

        /// Configuration file path
        #[arg(long, short)]
        config: Option<String>,

        /// Enable verbose logging
        #[arg(long, short)]
        verbose: bool,
    },

    /// Run health diagnostics and system checks
    ///
    /// Validates configuration, checks connectivity, and verifies system health.
    Doctor {
        /// Run comprehensive checks (includes primal connectivity)
        #[arg(long, short)]
        comprehensive: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Configuration management commands
    ///
    /// View, validate, and initialize Songbird configuration.
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show sensitive values (API keys, etc.)
        #[arg(long)]
        show_secrets: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Validate configuration
    Validate,

    /// Generate default configuration template
    Init {
        /// Output path for generated config
        #[arg(long, default_value = "songbird.toml")]
        output: String,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            port,
            daemon,
            config,
            verbose,
        } => {
            commands::run_server(port, daemon, config, verbose).await?;
        }
        Commands::Doctor {
            comprehensive,
            format,
        } => {
            commands::run_doctor(comprehensive, &format).await?;
        }
        Commands::Config {
            config_cmd,
        } => {
            // Initialize minimal logging for config commands
            tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

            let action = match config_cmd {
                ConfigCommands::Show {
                    show_secrets,
                    format,
                } => ConfigAction::Show {
                    show_secrets,
                    format,
                },
                ConfigCommands::Validate => ConfigAction::Validate,
                ConfigCommands::Init {
                    output,
                    force,
                } => ConfigAction::Init {
                    output,
                    force,
                },
            };
            commands::run_config(action).await?;
        }
    }

    Ok(())
}
