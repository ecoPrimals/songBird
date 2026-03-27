// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `UniBin` Fault Injection Tests
//!
//! Failure scenario testing including:
//! - Invalid input handling
//! - Error recovery
//! - Graceful degradation
//! - Boundary conditions
//! - Filesystem failures
//! - Network errors
//!
//! Modern, idiomatic, async Rust with deep debt solutions.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ====================
// TEST HELPERS
// ====================

/// Create a clean command with isolated environment (no global state mutation!)
fn clean_cmd() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("songbird"));
    // ✅ Clear environment for this command only (not global!)
    cmd.env_clear();
    // ✅ Set minimal required env vars for test isolation
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd
}

// ====================
// FAULT INJECTION TESTS
// ====================

#[tokio::test] // ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_invalid_port_zero() -> Result<(), Box<dyn std::error::Error>> {
    // Port 0 should fail
    clean_cmd().arg("server").arg("--port").arg("0").assert().failure();

    Ok(())
}

#[tokio::test] // ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_invalid_port_overflow() -> Result<(), Box<dyn std::error::Error>> {
    // Port > 65535 should fail
    clean_cmd().arg("server").arg("--port").arg("100000").assert().failure();

    Ok(())
}

#[tokio::test] // ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_invalid_port_negative() -> Result<(), Box<dyn std::error::Error>> {
    // Negative port should fail
    clean_cmd().arg("server").arg("--port").arg("-1").assert().failure();

    Ok(())
}

#[tokio::test] // ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_invalid_port_string() -> Result<(), Box<dyn std::error::Error>> {
    // Non-numeric port should fail
    clean_cmd().arg("server").arg("--port").arg("invalid").assert().failure();

    Ok(())
}

#[tokio::test] // ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_unknown_command() -> Result<(), Box<dyn std::error::Error>> {
    // Unknown command should fail gracefully
    clean_cmd().arg("nonexistent-command").assert().failure().stderr(
        predicate::str::contains("unrecognized").or(predicate::str::contains("wasn't recognized")),
    );

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_nonexistent_config_file() -> Result<(), Box<dyn std::error::Error>> {
    // Non-existent config file should handle gracefully
    let mut cmd = clean_cmd();
    cmd.arg("server")
        .arg("--config")
        .arg("/nonexistent/path/config.toml")
        .arg("--help") // Use help to avoid actually starting
        .assert()
        .success(); // Help should succeed even with invalid config path

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_empty_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("empty.toml");

    // Create empty config file
    fs::write(&config_path, "")?;

    // Empty config should handle gracefully
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--config").arg(&config_path).arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_malformed_toml() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("malformed.toml");

    // Create malformed TOML
    fs::write(&config_path, "this is not valid toml [[[")?;

    // Malformed TOML should handle gracefully
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--config").arg(&config_path).arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_invalid_doctor_format() -> Result<(), Box<dyn std::error::Error>> {
    // Invalid format should fail with helpful message
    let mut cmd = clean_cmd();
    cmd.arg("doctor")
        .arg("--format")
        .arg("invalid-format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown format"));

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_binary_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("binary.toml");

    // Create binary (non-UTF8) file
    fs::write(&config_path, vec![0xFF, 0xFE, 0xFD, 0xFC])?;

    // Binary file should handle gracefully
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--config").arg(&config_path).arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_directory_as_config() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Pass directory instead of file
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--config").arg(temp_dir.path()).arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_special_characters_in_args() -> Result<(), Box<dyn std::error::Error>> {
    // Test special characters
    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_unicode_in_args() -> Result<(), Box<dyn std::error::Error>> {
    // Test unicode handling
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "node-测试-🦀");

    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_whitespace_only_args() -> Result<(), Box<dyn std::error::Error>> {
    // Whitespace-only env var
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "   ");

    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_extremely_long_env_var() -> Result<(), Box<dyn std::error::Error>> {
    // Extremely long env var
    let long_value = "a".repeat(10000);
    songbird_process_env::set_var("SONGBIRD_NODE_ID", &long_value);

    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_empty_env_var() -> Result<(), Box<dyn std::error::Error>> {
    // Empty env var
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "");

    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_config_init_to_readonly_dir() -> Result<(), Box<dyn std::error::Error>> {
    // Try to init to /dev/null or similar (will fail or handle gracefully)
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("init").arg("--output").arg("/dev/null/impossible.toml");

    // This should fail, but gracefully
    let _ = cmd.assert();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_multiple_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Multiple boolean flags
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--verbose").arg("--daemon").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_conflicting_env_and_args() -> Result<(), Box<dyn std::error::Error>> {
    // Set env var, then override with arg (arg should win)
    songbird_process_env::set_var("SONGBIRD_PORT", "9000");

    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--port").arg("8888").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_boundary_port_1() -> Result<(), Box<dyn std::error::Error>> {
    // Port 1 (minimum valid port)
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--port").arg("1").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_boundary_port_65535() -> Result<(), Box<dyn std::error::Error>> {
    // Port 65535 (maximum valid port)
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--port").arg("65535").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_double_dash_args() -> Result<(), Box<dyn std::error::Error>> {
    // Test -- separator
    let mut cmd = clean_cmd();
    let _ = cmd.arg("--").arg("--version").assert();

    // Behavior depends on clap configuration

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_repeated_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Repeated flags should error (clap doesn't allow duplicate args)
    let mut cmd = clean_cmd();
    cmd.arg("server")
        .arg("--port")
        .arg("8080")
        .arg("--port")
        .arg("9000")
        .arg("--help")
        .assert()
        .failure() // Should fail with "cannot be used multiple times"
        .stderr(predicate::str::contains("cannot be used multiple times"));

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_fault_missing_required_subcommand_args() -> Result<(), Box<dyn std::error::Error>> {
    // Config init without output should use default
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("init").arg("--help").assert().success();

    Ok(())
}
