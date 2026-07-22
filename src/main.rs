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

/// Load relay credentials from file or `SONGBIRD_RELAY_CREDENTIALS` env var.
///
/// Format: one `username:hex_key` per line. Lines starting with `#` are comments.
fn load_relay_credentials(
    args: &songbird_stun::RelayArgs,
) -> Result<songbird_stun::StaticCredentialStore> {
    let mut store = songbird_stun::StaticCredentialStore::new();

    let content = if let Some(ref path) = args.credentials_file {
        Some(
            std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read credentials file {path}: {e}"))?,
        )
    } else {
        std::env::var("SONGBIRD_RELAY_CREDENTIALS").ok()
    };

    if let Some(text) = content {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((username, hex_key)) = line.split_once(':') else {
                tracing::warn!("Skipping malformed credential line (expected username:hex_key)");
                continue;
            };
            let key = hex_decode(hex_key.trim()).map_err(|e| {
                anyhow::anyhow!("Invalid hex key for user {}: {e}", username.trim())
            })?;
            store.insert(username.trim().to_string(), key);
        }
        tracing::info!("Loaded relay credentials ({} users)", store.len());
    } else {
        tracing::warn!("No relay credentials configured — running in open mode (testing only)");
    }

    Ok(store)
}

/// Decode a hex string to bytes.
fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        anyhow::bail!("Odd-length hex string");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| anyhow::anyhow!("Invalid hex at position {i}: {e}"))
        })
        .collect()
}

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
                Arc::new(load_relay_credentials(&args)?);
            let server = songbird_stun::TurnRelayServer::new(bind_addr, credentials);
            server.run().await.map_err(|e| anyhow::anyhow!("Relay server error: {e}"))?;
        }
        Commands::Benchmark {
            args,
        } => {
            tracing_subscriber::fmt::init();
            songbird::benchmark::run_benchmark(&args).await?;
        }
    }

    Ok(())
}
