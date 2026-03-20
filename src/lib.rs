// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Shared CLI entry types and helpers for the `songbird` binary.
//!
//! Kept in the library target so integration tests can exercise parsing
//! without spawning servers or subprocesses.

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Parse argv for a delegated clap parser (`--help` / `--version` exit via clap, success code 0).
///
/// Prefer [`try_parse_delegated`] in tests; this function terminates the process on parse failure.
pub fn parse_delegated<A: Parser>(invocation: &str, args: Vec<String>) -> A {
    try_parse_delegated(invocation, args).unwrap_or_else(|e| e.exit())
}

/// Non-terminating variant of [`parse_delegated`], suitable for unit and integration tests.
pub fn try_parse_delegated<A: Parser>(
    invocation: &str,
    args: Vec<String>,
) -> Result<A, clap::Error> {
    A::try_parse_from(std::iter::once(invocation.to_string()).chain(args))
}

/// Songbird - Network Orchestration & Discovery Primal
///
/// UniBin Architecture: One binary, multiple modes
#[derive(Parser, Debug)]
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
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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

/// Outcome of attempting to run the external `songbird-rendezvous` binary.
#[derive(Debug)]
pub enum RendezvousRunError {
    /// `songbird-rendezvous` was not found next to the current executable.
    BinaryNotFound,
    /// Failed to spawn the child process.
    SpawnFailed(std::io::Error),
    /// Child exited with non-zero status.
    NonZeroExit(i32),
}

/// Resolve the path to `songbird-rendezvous` adjacent to the current executable, if it exists.
pub fn rendezvous_binary_path() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("songbird-rendezvous")))
        .filter(|p| p.exists())
}

/// Run rendezvous without exiting the process; callers map errors to [`std::process::exit`].
pub fn try_run_rendezvous(args: Vec<String>) -> Result<(), RendezvousRunError> {
    let Some(path) = rendezvous_binary_path() else {
        return Err(RendezvousRunError::BinaryNotFound);
    };

    let status = std::process::Command::new(path)
        .args(args)
        .status()
        .map_err(RendezvousRunError::SpawnFailed)?;

    if !status.success() {
        return Err(RendezvousRunError::NonZeroExit(status.code().unwrap_or(1)));
    }

    Ok(())
}

/// Run rendezvous server by re-executing the binary
///
/// Deep debt solution: Keep existing binaries working during migration phase.
/// Future: Integrate directly into this binary via library calls.
pub fn run_rendezvous(args: Vec<String>) -> Result<()> {
    match try_run_rendezvous(args) {
        Ok(()) => Ok(()),
        Err(RendezvousRunError::BinaryNotFound) => {
            eprintln!("❌ Rendezvous server binary not found");
            eprintln!("💡 Build it with: cargo build --bin songbird-rendezvous");
            eprintln!("💡 Or use the full path to the binary");
            std::process::exit(1);
        }
        Err(RendezvousRunError::SpawnFailed(e)) => {
            Err(anyhow::anyhow!("Failed to execute songbird-rendezvous: {}", e))
        }
        Err(RendezvousRunError::NonZeroExit(code)) => {
            std::process::exit(code);
        }
    }
}
