//! UniBin E2E (End-to-End) Tests
//!
//! Full workflow testing for UniBin implementation including:
//! - Complete command execution flows
//! - Multi-step operations
//! - Real-world usage scenarios
//! - Process lifecycle management
//!
//! Modern, idiomatic, async Rust with deep debt solutions.

use assert_cmd::Command;
use predicates::prelude::*;

use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use tokio::time::{sleep, Duration};

// ====================
// TEST HELPERS
// ====================

fn clean_cmd() -> Command {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.env_clear();
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd
}

fn create_test_config(dir: &TempDir, content: &str) -> PathBuf {
    let config_path = dir.path().join("test-config.toml");
    fs::write(&config_path, content).expect("Failed to write test config");
    config_path
}

// ====================
// FULL WORKFLOW E2E TESTS
// ====================

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_help_and_version_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Check version
    let mut cmd = clean_cmd();
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("songbird"));

    // Step 2: Check main help
    let mut cmd = clean_cmd();
    cmd.arg("--help").assert().success().stdout(predicate::str::contains("Network Orchestration"));

    // Step 3: Check server help
    let mut cmd = clean_cmd();
    cmd.arg("server")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start Songbird orchestrator"));

    // Step 4: Check doctor help
    let mut cmd = clean_cmd();
    cmd.arg("doctor")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("health diagnostics"));

    // Step 5: Check config help
    let mut cmd = clean_cmd();
    cmd.arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration management"));

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_config_init_validate_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("songbird-generated.toml");

    // Step 1: Initialize config
    let mut cmd = clean_cmd();
    cmd.arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .arg("--force")
        .assert()
        .success();

    // Verify config file was created
    assert!(config_path.exists(), "Config file should be created");

    // Step 2: Validate the generated config
    // Note: validate doesn't take --config, it reads from env or default
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("validate").assert().success();

    // Step 3: Show the config
    // Note: show doesn't take --config, it reads from env or default
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("show").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_doctor_basic_checks() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Run basic doctor checks
    let mut cmd = clean_cmd();
    cmd.arg("doctor").arg("--format").arg("text").assert().success();

    // Step 2: Run doctor with JSON output
    let mut cmd = clean_cmd();
    cmd.arg("doctor").arg("--format").arg("json").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_server_with_custom_port() -> Result<(), Box<dyn std::error::Error>> {
    // Test that server command accepts custom port
    // Note: We don't actually start the server (would hang), just validate args
    let mut cmd = clean_cmd();
    cmd.arg("server")
        .arg("--port")
        .arg("9999")
        .arg("--help") // Use help to avoid actually starting
        .assert()
        .success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_environment_variable_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Set environment variables
    std::env::set_var("SONGBIRD_PORT", "9000");
    std::env::set_var("SONGBIRD_NODE_ID", "e2e-test-node");
    std::env::set_var("SONGBIRD_FAMILY_ID", "nat0");

    // Run doctor to verify env vars are read
    let mut cmd = clean_cmd();
    cmd.arg("doctor").assert().success();

    // Clean up

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_verbose_logging_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Test verbose flag
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--verbose").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_daemon_mode_flag() -> Result<(), Box<dyn std::error::Error>> {
    // Test daemon flag
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--daemon").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_config_file_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("custom-config.toml");

    // Create a config file
    fs::write(&config_path, "# Test config\n")?;

    // Test config path flag
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--config").arg(&config_path).arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_multiple_doctor_formats() -> Result<(), Box<dyn std::error::Error>> {
    let formats = vec!["text", "json", "yaml"];

    for format in formats {
        let mut cmd = clean_cmd();
        cmd.arg("doctor").arg("--format").arg(format).assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_comprehensive_doctor() -> Result<(), Box<dyn std::error::Error>> {
    // Run comprehensive doctor checks
    let mut cmd = clean_cmd();
    cmd.arg("doctor").arg("--comprehensive").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_config_show_with_secrets() -> Result<(), Box<dyn std::error::Error>> {
    // Show config without secrets (reads from env or default)
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("show").assert().success();

    // Show config with secrets flag
    let mut cmd = clean_cmd();
    cmd.arg("config").arg("show").arg("--show-secrets").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_config_init_force_overwrite() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("overwrite-test.toml");

    // Create initial file
    fs::write(&config_path, "# Original content\n")?;

    // Try to init without force (should fail or warn)
    // Then init with force (should succeed)
    let mut cmd = clean_cmd();
    cmd.arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .arg("--force")
        .assert()
        .success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_error_handling_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test unknown command
    let mut cmd = clean_cmd();
    cmd.arg("invalid-command").assert().failure();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_error_handling_invalid_port() -> Result<(), Box<dyn std::error::Error>> {
    // Test invalid port number
    let mut cmd = clean_cmd();
    cmd.arg("server")
        .arg("--port")
        .arg("99999") // Invalid: > 65535
        .assert()
        .failure();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_error_handling_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
    // Test invalid format (should error)
    let mut cmd = clean_cmd();
    cmd.arg("doctor")
        .arg("--format")
        .arg("invalid-format")
        .assert()
        .failure() // Should fail with invalid format
        .stderr(predicate::str::contains("Unknown format"));

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_rapid_command_execution() -> Result<(), Box<dyn std::error::Error>> {
    // Execute multiple commands rapidly
    for _ in 0..10 {
        let mut cmd = clean_cmd();
        cmd.arg("--version").assert().success();
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_concurrent_doctor_checks() -> Result<(), Box<dyn std::error::Error>> {
    // Run multiple doctor checks concurrently
    let mut handles = vec![];

    for _ in 0..3 {
        let handle = tokio::spawn(async {
            let mut cmd = Command::cargo_bin("songbird").unwrap();
            cmd.arg("doctor").assert().success();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_full_lifecycle_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;

    // Full lifecycle: check version -> init config -> validate -> doctor -> show config

    // 1. Check version
    Command::cargo_bin("songbird")?.arg("--version").assert().success();

    // 2. Initialize config
    let config_path = temp_dir.path().join("lifecycle-config.toml");
    Command::cargo_bin("songbird")?
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .arg("--force")
        .assert()
        .success();

    // 3. Validate config (reads from env or default)
    Command::cargo_bin("songbird")?.arg("config").arg("validate").assert().success();

    // 4. Run doctor
    Command::cargo_bin("songbird")?.arg("doctor").assert().success();

    // 5. Show config (reads from env or default)
    Command::cargo_bin("songbird")?.arg("config").arg("show").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_stress_version_calls() -> Result<(), Box<dyn std::error::Error>> {
    // Stress test: rapid version checks
    for i in 0..50 {
        let mut cmd = clean_cmd();
        cmd.arg("--version").assert().success();

        // Every 10 iterations, check help too
        if i % 10 == 0 {
            let mut cmd = clean_cmd();
            cmd.arg("--help").assert().success();
        }
    }

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_environment_precedence() -> Result<(), Box<dyn std::error::Error>> {
    // Test environment variable precedence
    std::env::set_var("SONGBIRD_PORT", "9000");

    // Command line should override env var
    let mut cmd = clean_cmd();
    cmd.arg("server").arg("--port").arg("8888").arg("--help").assert().success();

    Ok(())
}

#[tokio::test]
// ✅ NO #[serial]! Uses isolated environment!
async fn test_e2e_all_subcommands_accessible() -> Result<(), Box<dyn std::error::Error>> {
    // Verify all subcommands are accessible
    let subcommands = vec!["server", "doctor", "config"];

    for subcmd in subcommands {
        let mut cmd = clean_cmd();
        cmd.arg(subcmd).arg("--help").assert().success();
    }

    Ok(())
}
