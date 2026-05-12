// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird - Network Orchestration & Discovery Primal
//!
//! `UniBin` Architecture (Ecosystem Standard v1.0.0)
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

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use songbird::{Cli, Commands, parse_delegated, run_interactive_cli, run_rendezvous};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server {
            args,
        } => {
            Box::pin(songbird_orchestrator::run_server(args)).await?;
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
            args,
        } => {
            if !args.is_empty() {
                eprintln!("songbird cli does not accept arguments yet: {args:?}");
                eprintln!("Run `songbird cli` with no extra tokens for the interactive shell.");
                eprintln!("See also: songbird server --help | doctor --help | config --help");
                std::process::exit(1);
            }
            run_interactive_cli()?;
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
        Commands::Relay {
            args,
        } => {
            tracing_subscriber::fmt::init();
            let bind_addr: std::net::SocketAddr = format!("{}:{}", args.bind, args.port)
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid relay bind address: {e}"))?;
            let credentials: Arc<dyn songbird_stun::CredentialStore> =
                Arc::new(songbird_stun::StaticCredentialStore::new());
            let server = songbird_stun::TurnRelayServer::new(bind_addr, credentials);
            server.run().await.map_err(|e| anyhow::anyhow!("Relay server error: {e}"))?;
        }
    }

    Ok(())
}
