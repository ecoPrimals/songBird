// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Shared CLI entry types and helpers for the `songbird` binary.
//!
//! Kept in the library target so integration tests can exercise parsing
//! without spawning servers or subprocesses.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};

pub mod benchmark;

/// Parse argv for a delegated clap parser (`--help` / `--version` exit via clap, success code 0).
///
/// Prefer [`try_parse_delegated`] in tests; this function terminates the process on parse failure.
#[must_use]
pub fn parse_delegated<A: Parser>(invocation: &str, args: Vec<String>) -> A {
    try_parse_delegated(invocation, args).unwrap_or_else(|e| e.exit())
}

/// Non-terminating variant of [`parse_delegated`], suitable for unit and integration tests.
///
/// # Errors
///
/// Returns a [`clap::Error`] if argument parsing fails.
pub fn try_parse_delegated<A: Parser>(
    invocation: &str,
    args: Vec<String>,
) -> Result<A, clap::Error> {
    A::try_parse_from(std::iter::once(invocation.to_string()).chain(args))
}

/// Songbird - Network Orchestration & Discovery Primal
///
/// `UniBin` Architecture: One binary, multiple modes
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
    /// Subcommand selecting the operational mode.
    #[command(subcommand)]
    pub command: Commands,
}

/// Operational modes exposed as clap subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start Songbird orchestrator (main service)
    ///
    /// This is the primary operational mode that runs the full orchestrator
    /// with discovery, federation, and network services.
    Server {
        /// Server configuration flags (port, bind address, etc.).
        #[command(flatten)]
        args: songbird_orchestrator::ServerArgs,
    },

    /// Run health diagnostics and system checks
    ///
    /// Validates configuration, checks connectivity, and verifies system health.
    Doctor {
        /// Diagnostic options (verbosity, checks to run, etc.).
        #[command(flatten)]
        args: songbird_orchestrator::DoctorArgs,
    },

    /// Configuration management commands
    ///
    /// View, validate, and initialize Songbird configuration.
    Config {
        /// Configuration subcommand (show, validate, init, etc.).
        #[command(subcommand)]
        config_cmd: songbird_orchestrator::ConfigCommands,
    },

    /// Interactive CLI (minimal REPL — type `help` or `exit`)
    ///
    /// Lightweight shell for quick exploration; use `server`, `doctor`, or `config` for operations.
    Cli {
        /// Extra tokens (not used; reserved for future subcommands)
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

    /// TURN relay server (RFC 5766 sovereign VPS relay)
    ///
    /// Starts a standalone TURN relay for NAT traversal. Songbird clients
    /// allocate through this server when direct and STUN-assisted connectivity fail.
    Relay {
        /// Relay server configuration.
        #[command(flatten)]
        args: songbird_stun::RelayArgs,
    },

    /// Run Tower Atomic / `WireGuard` parity benchmark
    ///
    /// Measures latency, connection setup, and throughput against a mesh peer.
    /// Outputs structured JSON for `primalSpring` parity assessment.
    Benchmark {
        /// Benchmark configuration.
        #[command(flatten)]
        args: crate::benchmark::BenchmarkArgs,
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

/// Resolve the path to `songbird-rendezvous` in the same directory as `current_exe`, if it exists.
#[must_use]
pub(crate) fn rendezvous_binary_path_next_to(
    current_exe: &std::path::Path,
) -> Option<std::path::PathBuf> {
    current_exe.parent().map(|p| p.join("songbird-rendezvous")).filter(|p| p.exists())
}

/// Resolve the path to `songbird-rendezvous` adjacent to the current executable, if it exists.
#[must_use]
pub fn rendezvous_binary_path() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok().and_then(|exe| rendezvous_binary_path_next_to(&exe))
}

/// Run rendezvous without exiting the process; callers map errors to [`std::process::exit`].
///
/// # Errors
///
/// Returns an error if the rendezvous binary is not found, fails to spawn, or exits non-zero.
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

/// Minimal interactive CLI: prints guidance and reads lines until `exit` / `quit`.
///
/// # Errors
///
/// Returns I/O errors from stdin/stdout.
pub fn run_interactive_cli() -> Result<()> {
    let stdin = io::stdin();
    run_interactive_cli_reader(&mut stdin.lock())
}

/// Same as [`run_interactive_cli`], but reads commands from `reader` (used by tests).
pub(crate) fn run_interactive_cli_reader<R: BufRead>(reader: &mut R) -> Result<()> {
    println!("Songbird interactive CLI — commands: help, exit, quit");
    println!("Operational modes: server, doctor, config, compute-bridge, deploy, rendezvous");
    println!("Try: songbird server --help | songbird doctor --help | songbird config --help\n");

    let mut stdout = io::stdout();
    for line in reader.lines() {
        let line = line?;
        match line.trim() {
            "" => {}
            "exit" | "quit" => break,
            "help" | "?" => {
                writeln!(
                    stdout,
                    "This REPL only lists modes. Examples:\n\
                     \tsongbird server --help\n\
                     \tsongbird doctor\n\
                     \tsongbird config show\n\
                     \tsongbird compute-bridge -- --help"
                )?;
            }
            other => {
                writeln!(stdout, "Unknown input: {other}")?;
                writeln!(
                    stdout,
                    "Type `help` for available top-level commands, or `exit` to leave."
                )?;
            }
        }
        stdout.flush()?;
    }
    Ok(())
}

/// Run rendezvous server by re-executing the binary.
///
/// Deep debt solution: Keep existing binaries working during migration phase.
/// Future: Integrate directly into this binary via library calls.
///
/// # Errors
///
/// Returns an error if the rendezvous binary fails to spawn.
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
            Err(anyhow::anyhow!("Failed to execute songbird-rendezvous: {e}"))
        }
        Err(RendezvousRunError::NonZeroExit(code)) => {
            std::process::exit(code);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use clap::Parser;
    use std::io::{BufReader, Cursor};

    #[test]
    fn try_parse_delegated_server() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["server".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Server { .. }));
    }

    #[test]
    fn try_parse_delegated_doctor() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["doctor".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Doctor { .. }));
    }

    #[test]
    fn try_parse_delegated_config_show() {
        let cli =
            try_parse_delegated::<Cli>("songbird", vec!["config".into(), "show".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Config { .. }));
    }

    #[test]
    fn try_parse_delegated_config_validate() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["config".into(), "validate".into()])
            .unwrap();
        assert!(matches!(cli.command, Commands::Config { .. }));
    }

    #[test]
    fn try_parse_delegated_cli_subcommand() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["cli".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Cli { args } if args.is_empty()));
    }

    #[test]
    fn try_parse_delegated_compute_bridge() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["compute-bridge".into()]).unwrap();
        assert!(matches!(cli.command, Commands::ComputeBridge { args } if args.is_empty()));
    }

    #[test]
    fn try_parse_delegated_deploy_list() {
        let cli =
            try_parse_delegated::<Cli>("songbird", vec!["deploy".into(), "list".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Deploy { .. }));
    }

    #[test]
    fn try_parse_delegated_rendezvous() {
        let cli = try_parse_delegated::<Cli>("songbird", vec!["rendezvous".into()]).unwrap();
        assert!(matches!(cli.command, Commands::Rendezvous { args } if args.is_empty()));
    }

    #[test]
    fn try_parse_delegated_invalid_server_port() {
        let err = try_parse_delegated::<Cli>(
            "songbird",
            vec!["server".into(), "--port".into(), "not-a-u16".into()],
        )
        .unwrap_err();
        assert!(err.kind() == clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn try_parse_delegated_unknown_flag() {
        let err =
            try_parse_delegated::<Cli>("songbird", vec!["server".into(), "--not-a-flag".into()])
                .unwrap_err();
        assert!(err.kind() == clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rendezvous_binary_path_next_to_none_when_binary_missing() {
        let dir = tempfile::tempdir().unwrap();
        let fake_exe = dir.path().join("songbird");
        std::fs::write(&fake_exe, b"x").unwrap();
        assert!(rendezvous_binary_path_next_to(fake_exe.as_path()).is_none());
    }

    #[test]
    fn rendezvous_binary_path_next_to_some_when_binary_present() {
        let dir = tempfile::tempdir().unwrap();
        let fake_exe = dir.path().join("songbird");
        std::fs::write(&fake_exe, b"x").unwrap();
        let rend = dir.path().join("songbird-rendezvous");
        std::fs::write(&rend, b"y").unwrap();
        assert_eq!(rendezvous_binary_path_next_to(fake_exe.as_path()), Some(rend));
    }

    #[test]
    fn try_run_rendezvous_binary_not_found() {
        let err = try_run_rendezvous(vec![]).unwrap_err();
        assert!(matches!(err, RendezvousRunError::BinaryNotFound));
    }

    #[test]
    fn run_interactive_cli_reader_help_and_exit() {
        let input = b"help\nexit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn run_interactive_cli_reader_unknown_then_exit() {
        let input = b"foo\nexit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn cli_parses_all_command_variants_via_try_parse() {
        let _ = Cli::try_parse_from(["songbird", "server"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "doctor"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "config", "validate"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "cli"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "compute-bridge"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "deploy", "list"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "rendezvous"]).unwrap();
        let _ = Cli::try_parse_from(["songbird", "relay"]).unwrap();
    }

    #[test]
    fn try_parse_delegated_relay_custom_port() {
        let cli = try_parse_delegated::<Cli>(
            "songbird",
            vec!["relay".into(), "--port".into(), "4000".into()],
        )
        .unwrap();
        match cli.command {
            Commands::Relay {
                args,
            } => {
                assert_eq!(args.port, 4000);
            }
            _ => panic!("expected Relay variant"),
        }
    }

    #[test]
    fn run_interactive_cli_reader_empty_then_exit() {
        let input = b"\n\n\nexit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn run_interactive_cli_reader_quit_variant() {
        let input = b"quit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn run_interactive_cli_reader_question_mark_help() {
        let input = b"?\nexit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn run_interactive_cli_reader_eof_without_exit() {
        let input = b"help\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn run_interactive_cli_reader_whitespace_only_lines() {
        let input = b"   \n\t\nexit\n";
        let mut reader = BufReader::new(Cursor::new(input));
        run_interactive_cli_reader(&mut reader).unwrap();
    }

    #[test]
    fn rendezvous_run_error_debug_format() {
        let err = RendezvousRunError::BinaryNotFound;
        let dbg = format!("{err:?}");
        assert!(dbg.contains("BinaryNotFound"));

        let err = RendezvousRunError::NonZeroExit(42);
        let dbg = format!("{err:?}");
        assert!(dbg.contains("42"));
    }

    #[test]
    fn try_parse_delegated_no_subcommand() {
        let err = try_parse_delegated::<Cli>("songbird", vec![]).unwrap_err();
        assert!(
            err.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                || err.kind() == clap::error::ErrorKind::MissingSubcommand
        );
    }

    #[test]
    fn try_parse_delegated_compute_bridge_with_trailing_args() {
        let cli = try_parse_delegated::<Cli>(
            "songbird",
            vec!["compute-bridge".into(), "--port".into(), "8080".into()],
        )
        .unwrap();
        match cli.command {
            Commands::ComputeBridge {
                args,
            } => {
                assert_eq!(args, vec!["--port", "8080"]);
            }
            _ => panic!("expected ComputeBridge variant"),
        }
    }

    #[test]
    fn try_parse_delegated_deploy_with_trailing_args() {
        let cli = try_parse_delegated::<Cli>(
            "songbird",
            vec!["deploy".into(), "status".into(), "--json".into()],
        )
        .unwrap();
        match cli.command {
            Commands::Deploy {
                args,
            } => {
                assert_eq!(args, vec!["status", "--json"]);
            }
            _ => panic!("expected Deploy variant"),
        }
    }

    #[test]
    fn try_parse_delegated_cli_with_trailing_args() {
        let cli = try_parse_delegated::<Cli>(
            "songbird",
            vec!["cli".into(), "extra".into(), "tokens".into()],
        )
        .unwrap();
        match cli.command {
            Commands::Cli {
                args,
            } => {
                assert_eq!(args, vec!["extra", "tokens"]);
            }
            _ => panic!("expected Cli variant"),
        }
    }
}
