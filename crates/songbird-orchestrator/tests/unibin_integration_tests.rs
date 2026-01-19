//! UniBin Integration Tests
//!
//! Tests for Songbird UniBin architecture compliance and functionality
//! Modern, idiomatic, async Rust tests

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_binary_exists() {
    // Verify binary is named 'songbird' (UniBin compliant!)
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("songbird"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Network Orchestration & Discovery Primal"))
        .stdout(predicate::str::contains("Usage: songbird <COMMAND>"))
        .stdout(predicate::str::contains("server"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("config"));
}

#[test]
fn test_server_help() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["server", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start Songbird orchestrator in server mode"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--daemon"))
        .stdout(predicate::str::contains("--config"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn test_doctor_help() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["doctor", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Run health diagnostics"))
        .stdout(predicate::str::contains("--comprehensive"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_config_help() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration management"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("init"));
}

#[test]
fn test_doctor_basic() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Health Diagnostics"))
        .stdout(predicate::str::contains("Binary Information"))
        .stdout(predicate::str::contains("Configuration"))
        .stdout(predicate::str::contains("Network Ports"))
        .stdout(predicate::str::contains("Filesystem"));
}

#[test]
fn test_config_validate() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "validate"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validating"))
        .stdout(predicate::str::contains("Configuration"));
}

#[test]
fn test_config_init() {
    use std::fs;
    use tempfile::TempDir;

    let tmp_dir = TempDir::new().unwrap();
    let config_path = tmp_dir.path().join("test-config.toml");

    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "init", "--output", config_path.to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::str::contains("Configuration template generated"));

    // Verify file was created
    assert!(config_path.exists());

    // Verify content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("SONGBIRD_PORT"));
    assert!(content.contains("SONGBIRD_NODE_ID"));
    assert!(content.contains("SONGBIRD_FAMILY_ID"));
}

#[test]
fn test_config_init_force() {
    use std::fs;
    use tempfile::TempDir;

    let tmp_dir = TempDir::new().unwrap();
    let config_path = tmp_dir.path().join("test-config-force.toml");

    // Create file first
    fs::write(&config_path, "old content").unwrap();

    // Try to init without --force (should fail)
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "init", "--output", config_path.to_str().unwrap()]);
    cmd.assert().failure();

    // Try with --force (should succeed)
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "init", "--output", config_path.to_str().unwrap(), "--force"]);
    cmd.assert().success();

    // Verify new content
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("SONGBIRD_PORT"));
    assert!(!content.contains("old content"));
}

#[test]
fn test_unknown_command() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.arg("unknown");
    cmd.assert().failure().stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_server_port_arg() {
    // Test that port argument is accepted
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["server", "--port", "9000", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_doctor_comprehensive() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["doctor", "--comprehensive"]);
    cmd.assert().success().stdout(predicate::str::contains("Comprehensive Checks"));
}

#[test]
fn test_doctor_json_format() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["doctor", "--format", "json"]);
    cmd.assert().success().stdout(predicate::str::contains("status"));
}

#[test]
fn test_config_show() {
    let mut cmd = Command::cargo_bin("songbird").unwrap();
    cmd.args(["config", "show"]);
    cmd.assert().success().stdout(predicate::str::contains("Configuration"));
}
