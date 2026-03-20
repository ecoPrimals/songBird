// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird - Network Orchestration & Discovery Primal
//!
//! UniBin Architecture (Ecosystem Standard v1.0.0)
//! Single binary with multiple operational modes
//!
//! ## Usage
//!
//! ```bash
//! # Main service (orchestrator)
//! songbird server [OPTIONS]
//! songbird doctor [--comprehensive]
//! songbird config <show|validate|init>
//!
//! # Interactive CLI
//! songbird cli <SUBCOMMAND>
//!
//! # Compute bridge service (delegates in-process; same flags as `songbird-compute-bridge`)
//! songbird compute-bridge -- [OPTIONS]
//!
//! # Remote deployment (delegates in-process; same subcommands as `songbird-deploy`)
//! songbird deploy -- <deploy|deploy-http|list|status> ...
//!
//! # Rendezvous server
//! songbird rendezvous [OPTIONS]
//!
//! # Standard commands
//! songbird --help
//! songbird --version
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Parse argv for a delegated clap parser (`--help` / `--version` exit via clap, success code 0).
fn parse_delegated<A: Parser>(invocation: &str, args: Vec<String>) -> A {
    match A::try_parse_from(std::iter::once(invocation.to_string()).chain(args)) {
        Ok(parsed) => parsed,
        Err(err) => err.exit(),
    }
}

/// Songbird - Network Orchestration & Discovery Primal
///
/// UniBin Architecture: One binary, multiple modes
#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Network Orchestration & Discovery Primal")]
#[command(version)]
#[command(author = "ecoPrimals <contact@ecoprimals.dev>")]
#[command(long_about = r#"Songbird - Network Orchestration & Discovery Primal

A unified binary providing multiple operational modes for network orchestration,
service discovery, and federation management.

UniBin Architecture (v1.0.0):
  • Single binary per primal
  • Multiple subcommands for different modes
  • Professional CLI with comprehensive help
  • Modern idiomatic Rust throughout

For detailed help on each mode, use:
  songbird <mode> --help
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird orchestrator (main service)
    ///
    /// This is the primary operational mode that runs the full orchestrator
    /// with discovery, federation, and network services.
    Server {
        #[command(flatten)]
        args: songbird_orchestrator::ServerArgs,
    },

    /// Run health diagnostics and system checks
    ///
    /// Validates configuration, checks connectivity, and verifies system health.
    Doctor {
        #[command(flatten)]
        args: songbird_orchestrator::DoctorArgs,
    },

    /// Configuration management commands
    ///
    /// View, validate, and initialize Songbird configuration.
    Config {
        #[command(subcommand)]
        config_cmd: songbird_orchestrator::ConfigCommands,
    },

    /// Interactive CLI (placeholder - future implementation)
    ///
    /// Interactive command-line interface for managing Songbird.
    /// Currently under development.
    #[command(hide = true)] // Hide until fully implemented
    Cli {
        /// CLI command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Compute bridge service
    ///
    /// Agnostic compute service bridge for Songbird federation.
    /// Enables any compute service to participate in the federation.
    ComputeBridge {
        /// All remaining arguments passed to compute bridge
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Deploy services to remote towers
    ///
    /// SSH-based or HTTP-based deployment tool for federation services.
    Deploy {
        /// All remaining arguments passed to deployment tool
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Rendezvous server
    ///
    /// Privacy-first rendezvous server for internet-wide federation.
    Rendezvous {
        /// All remaining arguments passed to rendezvous server
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            args,
        } => {
            songbird_orchestrator::run_server(args).await?;
        }
        Commands::Doctor {
            args,
        } => {
            songbird_orchestrator::run_doctor(args).await?;
        }
        Commands::Config {
            config_cmd,
        } => {
            songbird_orchestrator::run_config(config_cmd).await?;
        }
        Commands::Cli {
            args: _args,
        } => {
            eprintln!("❌ Interactive CLI is under development");
            eprintln!("💡 Use 'songbird server' for the main service");
            eprintln!("💡 Use 'songbird doctor' for health checks");
            eprintln!("💡 Use 'songbird config' for configuration management");
            std::process::exit(1);
        }
        Commands::ComputeBridge {
            args,
        } => {
            let bridge_args =
                parse_delegated::<songbird_compute_bridge::Args>("songbird-compute-bridge", args);
            songbird_compute_bridge::run(bridge_args).await?;
        }
        Commands::Deploy {
            args,
        } => {
            let deploy_args =
                parse_delegated::<songbird_remote_deploy::Args>("songbird-deploy", args);
            songbird_remote_deploy::run(deploy_args).await?;
        }
        Commands::Rendezvous {
            args,
        } => {
            // Re-exec rendezvous binary if it exists,
            // otherwise provide helpful error
            run_rendezvous(args)?;
        }
    }

    Ok(())
}

/// Run rendezvous server by re-executing the binary
///
/// Deep debt solution: Keep existing binaries working during migration phase.
/// Future: Integrate directly into this binary via library calls.
fn run_rendezvous(args: Vec<String>) -> Result<()> {
    let binary_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("songbird-rendezvous")))
        .filter(|p| p.exists());

    if let Some(path) = binary_path {
        let status = std::process::Command::new(path)
            .args(args)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute songbird-rendezvous: {}", e))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        Ok(())
    } else {
        eprintln!("❌ Rendezvous server binary not found");
        eprintln!("💡 Build it with: cargo build --bin songbird-rendezvous");
        eprintln!("💡 Or use the full path to the binary");
        std::process::exit(1);
    }
}
