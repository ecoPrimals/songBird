// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Integration tests for Songbird Execution Agent

use songbird_execution_agent::{
    AgentConfig, CommandExecutor, ExecutionRequest, JobManager, ResourceLimits, SecurityConfig,
    SecurityRequest, SovereignSecurityValidator,
};
use std::sync::Arc;

/// Helper: Create default resource limits for testing
const fn test_resource_limits() -> ResourceLimits {
    ResourceLimits {
        max_memory_mb: Some(4096),
        max_cpu_time_seconds: Some(3600),
        default_timeout_seconds: 3600,
    }
}

/// Test: Basic command execution (foreground)
#[tokio::test]
async fn test_command_execution_foreground() {
    let executor = CommandExecutor::new(test_resource_limits());

    let request = ExecutionRequest::new("echo 'Hello, Songbird!'").with_timeout(60);

    let response = executor.execute(request).await.expect("test precondition");

    assert_eq!(response.exit_code, Some(0));
    assert!(response.stdout.contains("Hello, Songbird!"));
    assert!(response.stderr.is_empty());
    assert!(response.duration_ms.is_some());
}

/// Test: Command execution with environment variables (simpler test)
#[tokio::test]
async fn test_command_with_env_vars() {
    let executor = CommandExecutor::new(test_resource_limits());

    // Use printenv which should work more reliably
    let request = ExecutionRequest::new("printenv TEST_VAR")
        .with_env("TEST_VAR", "test_value")
        .with_timeout(60);

    let response = executor.execute(request).await.expect("test precondition");

    assert_eq!(response.exit_code, Some(0));
    assert!(response.stdout.trim().contains("test_value"));
}

/// Test: Command execution with custom working directory
#[tokio::test]
async fn test_command_with_working_dir() {
    let executor = CommandExecutor::new(test_resource_limits());

    let request = ExecutionRequest::new("pwd").with_working_dir("/tmp").with_timeout(60);

    let response = executor.execute(request).await.expect("test precondition");

    assert_eq!(response.exit_code, Some(0));
    assert!(response.stdout.contains("/tmp"));
}

/// Test: Command with non-zero exit code
#[tokio::test]
async fn test_command_with_error_exit_code() {
    let executor = CommandExecutor::new(test_resource_limits());

    // Use false command which always returns exit code 1
    let request = ExecutionRequest::new("false").with_timeout(60);

    let response = executor.execute(request).await.expect("test precondition");

    assert_eq!(response.exit_code, Some(1));
}

/// Test: Background job execution
#[tokio::test]
async fn test_background_job_execution() {
    let executor = CommandExecutor::new(test_resource_limits());

    let request = ExecutionRequest::new("sleep 2").with_background(true).with_timeout(60);

    let job = executor.execute_background(request).await.expect("test precondition");

    assert!(!job.id.is_empty());
    assert!(job.pid.is_some());
    assert!(job.started_at.elapsed().expect("test precondition").as_secs() < 1);
}

/// Test: Job manager - add and retrieve job
#[tokio::test]
async fn test_job_manager_add_retrieve() {
    let job_manager = Arc::new(JobManager::new(10, 3600));
    let executor = CommandExecutor::new(test_resource_limits());

    let request =
        ExecutionRequest::new("echo 'background job'").with_background(true).with_timeout(60);

    let job = executor.execute_background(request).await.expect("test precondition");
    let job_id = job.id.clone();

    job_manager.add_job(job).await.expect("test precondition");

    let retrieved = job_manager.get_job(&job_id).await.expect("should find expected value");
    assert_eq!(retrieved.id, job_id);
}

/// Test: Job manager - list jobs
#[tokio::test]
async fn test_job_manager_list_jobs() {
    let job_manager = Arc::new(JobManager::new(10, 3600));
    let executor = CommandExecutor::new(test_resource_limits());

    // Create multiple jobs
    for i in 0..3 {
        let request =
            ExecutionRequest::new(format!("echo 'job {i}'")).with_background(true).with_timeout(60);

        let job = executor.execute_background(request).await.expect("test precondition");
        job_manager.add_job(job).await.expect("test precondition");
    }

    let jobs = job_manager.list_jobs().await;
    assert!(jobs.len() >= 3);
}

/// Test: Job manager - concurrent job limit
#[tokio::test]
async fn test_job_manager_concurrent_limit() {
    let job_manager = Arc::new(JobManager::new(2, 3600)); // Max 2 concurrent
    let executor = CommandExecutor::new(test_resource_limits());

    // Add 2 jobs (should succeed)
    for i in 0..2 {
        let request = ExecutionRequest::new(format!("sleep 10 && echo '{i}'"))
            .with_background(true)
            .with_timeout(60);

        let job = executor.execute_background(request).await.expect("test precondition");
        job_manager.add_job(job).await.expect("test precondition");
    }

    // Try to add 3rd job (should fail due to limit)
    let request = ExecutionRequest::new("echo 'third job'").with_background(true).with_timeout(60);

    let job = executor.execute_background(request).await.expect("test precondition");
    let result = job_manager.add_job(job).await;

    assert!(result.is_err());
}

/// Test: Sovereign security - authentication success
#[tokio::test]
async fn test_security_auth_success() {
    let config = SecurityConfig {
        enable_auth: true,
        auth_tokens: vec!["secret123".to_string()],
        max_timeout_seconds: 3600,
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    let request = SecurityRequest {
        command: "echo hello".to_string(),
        auth_token: Some("secret123".to_string()),
        timeout_seconds: Some(60),
        requester: Some("test".to_string()),
    };

    let decision = validator.validate_request(&request).await.expect("test precondition");
    assert!(decision.allowed);
}

/// Test: Sovereign security - authentication failure
#[tokio::test]
async fn test_security_auth_failure() {
    let config = SecurityConfig {
        enable_auth: true,
        auth_tokens: vec!["secret123".to_string()],
        max_timeout_seconds: 3600,
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    let request = SecurityRequest {
        command: "echo hello".to_string(),
        auth_token: Some("wrong_token".to_string()),
        timeout_seconds: Some(60),
        requester: Some("test".to_string()),
    };

    let decision = validator.validate_request(&request).await.expect("test precondition");
    assert!(!decision.allowed);
}

/// Test: Sovereign security - dangerous command blocked
#[tokio::test]
async fn test_security_dangerous_command_blocked() {
    let config = SecurityConfig {
        enable_auth: false,
        auth_tokens: vec![],
        max_timeout_seconds: 3600,
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    let dangerous_commands =
        vec!["rm -rf /", ":(){ :|:& };:", "mkfs.ext4 /dev/sda", "dd if=/dev/zero of=/dev/sda"];

    for cmd in dangerous_commands {
        let request = SecurityRequest {
            command: cmd.to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.expect("test precondition");
        assert!(!decision.allowed, "Command '{cmd}' should be blocked");
    }
}

/// Test: Sovereign security - timeout limit enforcement
#[tokio::test]
async fn test_security_timeout_limit() {
    let config = SecurityConfig {
        enable_auth: false,
        auth_tokens: vec![],
        max_timeout_seconds: 3600, // 1 hour
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    // Request with excessive timeout
    let request = SecurityRequest {
        command: "echo hello".to_string(),
        auth_token: None,
        timeout_seconds: Some(7200), // 2 hours
        requester: Some("test".to_string()),
    };

    let decision = validator.validate_request(&request).await.expect("test precondition");
    assert!(!decision.allowed);
}

/// Test: Sovereign security - safe commands allowed
#[tokio::test]
async fn test_security_safe_commands_allowed() {
    let config = SecurityConfig {
        enable_auth: false,
        auth_tokens: vec![],
        max_timeout_seconds: 3600,
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    let safe_commands =
        vec!["echo hello", "ls -la", "pwd", "date", "python train.py", "cargo build"];

    for cmd in safe_commands {
        let request = SecurityRequest {
            command: cmd.to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.expect("test precondition");
        assert!(decision.allowed, "Command '{cmd}' should be allowed");
    }
}

/// Test: Agent configuration defaults
#[test]
fn test_agent_config_defaults() {
    let config = AgentConfig::default();

    assert_eq!(config.port, 9020);
    assert_eq!(config.bind_address, "0.0.0.0");
    assert_eq!(config.max_concurrent_jobs, 100);
    assert_eq!(config.log_retention_seconds, 86400);
    assert!(config.enable_auth);
}

/// Test: Command execution with stderr output
#[tokio::test]
async fn test_command_with_stderr() {
    let executor = CommandExecutor::new(test_resource_limits());

    // Use ls with invalid directory to generate stderr
    let request = ExecutionRequest::new("ls /nonexistent_directory_12345").with_timeout(60);

    let response = executor.execute(request).await.expect("test precondition");

    // ls with invalid directory returns non-zero
    assert_ne!(response.exit_code, Some(0));
    // Should have error message in stderr
    assert!(!response.stderr.is_empty());
}

/// Test: Multiple tokens in security config
#[tokio::test]
async fn test_security_multiple_tokens() {
    let config = SecurityConfig {
        enable_auth: true,
        auth_tokens: vec!["token1".to_string(), "token2".to_string(), "token3".to_string()],
        max_timeout_seconds: 3600,
        enable_security_provider_discovery: false,
    };

    let validator = SovereignSecurityValidator::new(config);

    // Test all tokens
    for token in &["token1", "token2", "token3"] {
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: Some(token.to_string()),
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.expect("test precondition");
        assert!(decision.allowed, "Token {token} should be valid");
    }
}
