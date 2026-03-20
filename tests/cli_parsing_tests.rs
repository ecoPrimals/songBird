// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Integration tests for the unified `songbird` CLI (`src/main.rs` / `src/lib.rs`).
//!
//! These tests exercise **parsing and validation only**: no servers, no long-running tasks,
//! and no reliance on `parse_delegated`'s `exit()` path (that behavior is covered indirectly
//! via [`songbird::try_parse_delegated`], which implements the same `clap::Parser::try_parse_from`
//! invocation).

#![expect(
    clippy::expect_used,
    reason = "test assertions use expect/expect_err for clear failure messages"
)]

use clap::Parser;
use clap::error::ErrorKind;
use songbird::{Cli, Commands, RendezvousRunError, try_parse_delegated, try_run_rendezvous};

// ---------------------------------------------------------------------------
// CLI parsing — valid subcommands
// ---------------------------------------------------------------------------

/// [`Cli`] should accept each top-level subcommand with minimal valid arguments.
#[test]
fn cli_parses_server_subcommand() {
    let cli =
        Cli::try_parse_from(["songbird", "server"]).expect("server should parse with defaults");
    assert!(matches!(cli.command, Commands::Server { .. }));
}

#[test]
fn cli_parses_doctor_subcommand() {
    let cli = Cli::try_parse_from(["songbird", "doctor"]).expect("doctor should parse");
    assert!(matches!(cli.command, Commands::Doctor { .. }));
}

#[test]
fn cli_parses_config_validate() {
    let cli = Cli::try_parse_from(["songbird", "config", "validate"])
        .expect("config validate should parse");
    assert!(matches!(cli.command, Commands::Config { .. }));
}

#[test]
fn cli_parses_hidden_cli_placeholder() {
    let cli = Cli::try_parse_from(["songbird", "cli"]).expect("cli placeholder should parse");
    assert!(matches!(cli.command, Commands::Cli { args } if args.is_empty()));
}

#[test]
fn cli_parses_compute_bridge_with_trailing_args() {
    let cli = Cli::try_parse_from(["songbird", "compute-bridge", "--", "--port", "9001"])
        .expect("compute-bridge should accept forwarded args");
    match cli.command {
        Commands::ComputeBridge {
            args,
        } => {
            assert_eq!(args, vec!["--port", "9001"]);
        }
        _ => panic!("expected ComputeBridge variant"),
    }
}

#[test]
fn cli_parses_deploy_with_trailing_args() {
    let cli = Cli::try_parse_from(["songbird", "deploy", "--", "list"])
        .expect("deploy should accept forwarded args");
    match cli.command {
        Commands::Deploy {
            args,
        } => assert_eq!(args, vec!["list"]),
        _ => panic!("expected Deploy variant"),
    }
}

#[test]
fn cli_parses_rendezvous_subcommand() {
    let cli = Cli::try_parse_from(["songbird", "rendezvous", "--", "--help"])
        .expect("rendezvous should accept forwarded args");
    match cli.command {
        Commands::Rendezvous {
            args,
        } => assert_eq!(args, vec!["--help"]),
        _ => panic!("expected Rendezvous variant"),
    }
}

// ---------------------------------------------------------------------------
// Help and version (clap error kinds)
// ---------------------------------------------------------------------------

#[test]
fn root_help_is_display_help_kind() {
    let err = Cli::try_parse_from(["songbird", "--help"]).expect_err("--help should not return Ok");
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);
}

#[test]
fn root_version_is_display_version_kind() {
    let err =
        Cli::try_parse_from(["songbird", "--version"]).expect_err("--version should not return Ok");
    assert_eq!(err.kind(), ErrorKind::DisplayVersion);
}

// ---------------------------------------------------------------------------
// Delegated parsers (`parse_delegated` / `try_parse_delegated`)
// ---------------------------------------------------------------------------

#[test]
fn try_parse_delegated_compute_bridge_accepts_defaults() {
    let parsed =
        try_parse_delegated::<songbird_compute_bridge::Args>("songbird-compute-bridge", vec![])
            .expect("compute-bridge should parse with default flags");
    let baseline = songbird_compute_bridge::Args::try_parse_from(["songbird-compute-bridge"])
        .expect("baseline parse");
    assert_eq!(format!("{parsed:?}"), format!("{baseline:?}"));
}

#[test]
fn try_parse_delegated_compute_bridge_rejects_unknown_flag() {
    let err = try_parse_delegated::<songbird_compute_bridge::Args>(
        "songbird-compute-bridge",
        vec!["--not-a-valid-bridge-flag".to_string()],
    )
    .expect_err("unknown flag should fail");
    assert_eq!(err.kind(), ErrorKind::UnknownArgument);
}

#[test]
fn try_parse_delegated_deploy_accepts_list_subcommand() {
    let parsed = try_parse_delegated::<songbird_remote_deploy::Args>(
        "songbird-deploy",
        vec!["list".to_string()],
    )
    .expect("deploy list should parse");
    let baseline = songbird_remote_deploy::Args::try_parse_from(["songbird-deploy", "list"])
        .expect("baseline deploy list");
    assert_eq!(format!("{parsed:?}"), format!("{baseline:?}"));
}

#[test]
fn try_parse_delegated_deploy_rejects_missing_subcommand() {
    let err = try_parse_delegated::<songbird_remote_deploy::Args>("songbird-deploy", vec![])
        .expect_err("subcommand required");
    assert!(
        matches!(
            err.kind(),
            ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ),
        "unexpected kind: {:?}",
        err.kind()
    );
}

// ---------------------------------------------------------------------------
// Rendezvous: binary missing (typical in test / dev trees)
// ---------------------------------------------------------------------------

/// When `songbird-rendezvous` is not present beside the test executable, delegation must
/// surface [`RendezvousRunError::BinaryNotFound`] instead of panicking.
#[test]
fn try_run_rendezvous_errors_when_sidecar_binary_missing() {
    if songbird::rendezvous_binary_path().is_some() {
        // Rare: a `songbird-rendezvous` binary exists next to the test runner; skip to avoid
        // spawning an unrelated process.
        return;
    }

    let err = try_run_rendezvous(vec![]).expect_err("expected missing binary");
    assert!(matches!(err, RendezvousRunError::BinaryNotFound));
}

// ---------------------------------------------------------------------------
// Unknown subcommands and missing required input
// ---------------------------------------------------------------------------

#[test]
fn unknown_subcommand_maps_to_invalid_subcommand() {
    let err =
        Cli::try_parse_from(["songbird", "not-a-real-subcommand"]).expect_err("unknown subcommand");
    assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
}

#[test]
fn missing_required_subcommand_maps_to_missing_subcommand() {
    let err = Cli::try_parse_from(["songbird"]).expect_err("subcommand required");
    assert!(
        matches!(
            err.kind(),
            ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ),
        "unexpected kind: {:?}",
        err.kind()
    );
}
