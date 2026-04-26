// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for [`super::command_handler::CommandHandler`].

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::command_handler::CommandHandler;
use crate::SongbirdOrchestrator;
use crate::test_sync_env::{VarGuard, env_lock};
use songbird_types::config::CanonicalSongbirdConfig;

#[tokio::test]
async fn unknown_command_returns_message() -> anyhow::Result<()> {
    let port = songbird_test_utils::test_port("cmd_handler_sec");
    let url = format!("http://127.0.0.1:{port}");
    let _serial = env_lock();
    let _sec = VarGuard::set("SONGBIRD_SECURITY_PROVIDER", url.as_str());

    let orch = SongbirdOrchestrator::new(CanonicalSongbirdConfig::default()).await?;
    let handler = CommandHandler::new(&orch);
    let out = handler.handle("definitely-not-a-command").await?;
    assert_eq!(out, "Unknown command: definitely-not-a-command");
    Ok(())
}

#[tokio::test]
async fn status_command_formats_orchestrator_status() -> anyhow::Result<()> {
    let port = songbird_test_utils::test_port("cmd_handler_stat");
    let url = format!("http://127.0.0.1:{port}");
    let _serial = env_lock();
    let _sec = VarGuard::set("SONGBIRD_SECURITY_PROVIDER", url.as_str());

    let orch = SongbirdOrchestrator::new(CanonicalSongbirdConfig::default()).await?;
    let handler = CommandHandler::new(&orch);
    let out = handler.handle("status").await?;
    assert!(out.starts_with("Status: "), "expected Debug-formatted status prefix, got: {out}");
    Ok(())
}

#[tokio::test]
async fn health_command_returns_report_or_error_string() -> anyhow::Result<()> {
    let port = songbird_test_utils::test_port("cmd_handler_health");
    let url = format!("http://127.0.0.1:{port}");
    let _serial = env_lock();
    let _sec = VarGuard::set("SONGBIRD_SECURITY_PROVIDER", url.as_str());

    let orch = SongbirdOrchestrator::new(CanonicalSongbirdConfig::default()).await?;
    let handler = CommandHandler::new(&orch);
    let out = handler.handle("health").await?;
    assert!(
        out.starts_with("Health Check Status: ") || out.starts_with("Health check failed:"),
        "unexpected health output: {out}"
    );
    Ok(())
}
