// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for execution agent types

use songbird_execution_agent::types::*;
use songbird_execution_agent::{AgentConfig, ResourceLimits};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

// ============================================================================
// AgentConfig Tests
// ============================================================================

#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();
    assert_eq!(config.port, 9020);
    assert_eq!(config.bind_address, "0.0.0.0");
    assert_eq!(config.max_concurrent_jobs, 100);
    assert_eq!(config.log_retention_seconds, 86400);
    assert!(config.enable_auth);
    assert!(config.auth_token.is_none());
}

#[test]
fn test_agent_config_clone() {
    let config = AgentConfig::default();
    let cloned = config.clone();
    assert_eq!(config.port, cloned.port);
    assert_eq!(config.bind_address, cloned.bind_address);
}

#[test]
fn test_agent_config_serialization() {
    let config = AgentConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("port"));
    assert!(json.contains("9020"));
}

#[test]
fn test_agent_config_deserialization() {
    let json = r#"{"port":8080,"bind_address":"127.0.0.1","max_concurrent_jobs":50,"log_retention_seconds":3600,"enable_auth":false,"auth_token":null,"resource_limits":{"max_memory_mb":2048,"max_cpu_time_seconds":1800,"default_timeout_seconds":1800}}"#;
    let config: AgentConfig = serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(config.port, 8080);
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.max_concurrent_jobs, 50);
    assert!(!config.enable_auth);
}

// ============================================================================
// ResourceLimits Tests
// ============================================================================

#[test]
fn test_resource_limits_default() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_memory_mb, Some(4096));
    assert_eq!(limits.max_cpu_time_seconds, Some(3600));
    assert_eq!(limits.default_timeout_seconds, 3600);
}

#[test]
fn test_resource_limits_clone() {
    let limits = ResourceLimits::default();
    let cloned = limits.clone();
    assert_eq!(limits.max_memory_mb, cloned.max_memory_mb);
    assert_eq!(limits.default_timeout_seconds, cloned.default_timeout_seconds);
}

#[test]
fn test_resource_limits_serialization() {
    let limits = ResourceLimits::default();
    let json = serde_json::to_string(&limits).expect("Serialization should succeed");
    assert!(json.contains("max_memory_mb"));
    assert!(json.contains("4096"));
}

// ============================================================================
// ExecutionRequest Tests
// ============================================================================

#[test]
fn test_execution_request_new() {
    let req = ExecutionRequest::new("echo hello");
    assert_eq!(req.command, "echo hello");
    assert!(req.id.is_some());
    assert!(req.working_dir.is_none());
    assert!(req.env.is_empty());
    assert!(!req.background);
    assert!(req.timeout_seconds.is_none());
    assert!(req.capture_output);
}

#[test]
fn test_execution_request_builder_chain() {
    let req = ExecutionRequest::new("ls -la")
        .with_working_dir("/home/user")
        .with_env("PATH", "/usr/bin")
        .with_env("HOME", "/home/user")
        .with_background(true)
        .with_timeout(60);

    assert_eq!(req.command, "ls -la");
    assert_eq!(req.working_dir, Some(PathBuf::from("/home/user")));
    assert_eq!(req.env.len(), 2);
    assert_eq!(req.env.get("PATH"), Some(&"/usr/bin".to_string()));
    assert!(req.background);
    assert_eq!(req.timeout_seconds, Some(60));
}

#[test]
fn test_execution_request_clone() {
    let req = ExecutionRequest::new("test command").with_timeout(30);
    let cloned = req.clone();
    assert_eq!(req.command, cloned.command);
    assert_eq!(req.timeout_seconds, cloned.timeout_seconds);
}

#[test]
fn test_execution_request_serialization() {
    let req = ExecutionRequest::new("echo test");
    let json = serde_json::to_string(&req).expect("Serialization should succeed");
    assert!(json.contains("echo test"));
    assert!(json.contains("command"));
}

// ============================================================================
// ExecutionStatus Tests
// ============================================================================

#[test]
fn test_execution_status_display_all_variants() {
    assert_eq!(ExecutionStatus::Queued.to_string(), "queued");
    assert_eq!(ExecutionStatus::Running.to_string(), "running");
    assert_eq!(ExecutionStatus::Completed.to_string(), "completed");
    assert_eq!(ExecutionStatus::Failed.to_string(), "failed");
    assert_eq!(ExecutionStatus::Timeout.to_string(), "timeout");
    assert_eq!(ExecutionStatus::Stopped.to_string(), "stopped");
}

#[test]
fn test_execution_status_equality() {
    assert_eq!(ExecutionStatus::Running, ExecutionStatus::Running);
    assert_ne!(ExecutionStatus::Running, ExecutionStatus::Completed);
}

#[test]
fn test_execution_status_clone() {
    let status = ExecutionStatus::Running;
    let cloned = status;
    assert_eq!(status, cloned);
}

#[test]
fn test_execution_status_serialization() {
    let status = ExecutionStatus::Completed;
    let json = serde_json::to_string(&status).expect("Serialization should succeed");
    assert!(json.contains("completed"));
}

#[test]
fn test_execution_status_deserialization() {
    let json = r#""running""#;
    let status: ExecutionStatus =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(status, ExecutionStatus::Running);
}

// ============================================================================
// ExecutionResponse Tests
// ============================================================================

#[test]
fn test_execution_response_creation() {
    let response = ExecutionResponse {
        job_id: "job-123".to_string(),
        status: ExecutionStatus::Completed,
        pid: Some(1234),
        exit_code: Some(0),
        stdout: "output".to_string(),
        stderr: String::new(),
        started_at: SystemTime::now(),
        completed_at: Some(SystemTime::now()),
        duration_ms: Some(100),
    };

    assert_eq!(response.job_id, "job-123");
    assert_eq!(response.status, ExecutionStatus::Completed);
    assert_eq!(response.pid, Some(1234));
    assert_eq!(response.exit_code, Some(0));
}

#[test]
fn test_execution_response_clone() {
    let response = ExecutionResponse {
        job_id: "job-456".to_string(),
        status: ExecutionStatus::Running,
        pid: Some(5678),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        started_at: SystemTime::now(),
        completed_at: None,
        duration_ms: None,
    };

    let cloned = response.clone();
    assert_eq!(response.job_id, cloned.job_id);
    assert_eq!(response.status, cloned.status);
}

// ============================================================================
// StopJobRequest Tests
// ============================================================================

#[test]
fn test_stop_job_request_with_signal() {
    let req = StopJobRequest {
        signal: Some("SIGTERM".to_string()),
    };
    assert_eq!(req.signal, Some("SIGTERM".to_string()));
}

#[test]
fn test_stop_job_request_without_signal() {
    let req = StopJobRequest {
        signal: None,
    };
    assert!(req.signal.is_none());
}

#[test]
fn test_stop_job_request_serialization() {
    let req = StopJobRequest {
        signal: Some("SIGKILL".to_string()),
    };
    let json = serde_json::to_string(&req).expect("Serialization should succeed");
    assert!(json.contains("SIGKILL"));
}

// ============================================================================
// StopJobResponse Tests
// ============================================================================

#[test]
fn test_stop_job_response_creation() {
    let response = StopJobResponse {
        job_id: "job-789".to_string(),
        status: ExecutionStatus::Stopped,
        signal: "SIGTERM".to_string(),
    };

    assert_eq!(response.job_id, "job-789");
    assert_eq!(response.status, ExecutionStatus::Stopped);
    assert_eq!(response.signal, "SIGTERM");
}

// ============================================================================
// BroadcastRequest Tests
// ============================================================================

#[test]
fn test_broadcast_request_creation() {
    let req = BroadcastRequest {
        tower_ids: vec!["tower-1".to_string(), "tower-2".to_string()],
        command: "echo broadcast".to_string(),
        working_dir: None,
        env: HashMap::new(),
        wait_for_completion: true,
        timeout_seconds: Some(120),
    };

    assert_eq!(req.tower_ids.len(), 2);
    assert_eq!(req.command, "echo broadcast");
    assert!(req.wait_for_completion);
    assert_eq!(req.timeout_seconds, Some(120));
}

#[test]
fn test_broadcast_request_clone() {
    let req = BroadcastRequest {
        tower_ids: vec!["t1".to_string()],
        command: "test".to_string(),
        working_dir: None,
        env: HashMap::new(),
        wait_for_completion: false,
        timeout_seconds: None,
    };

    let cloned = req.clone();
    assert_eq!(req.tower_ids, cloned.tower_ids);
    assert_eq!(req.command, cloned.command);
}

// ============================================================================
// TowerExecutionResult Tests
// ============================================================================

#[test]
fn test_tower_execution_result_success() {
    let result = TowerExecutionResult {
        tower_id: "tower-1".to_string(),
        status: ExecutionStatus::Completed,
        exit_code: Some(0),
        stdout: "success output".to_string(),
        stderr: String::new(),
        duration_ms: 500,
        error: None,
    };

    assert_eq!(result.tower_id, "tower-1");
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert_eq!(result.exit_code, Some(0));
    assert!(result.error.is_none());
}

#[test]
fn test_tower_execution_result_failure() {
    let result = TowerExecutionResult {
        tower_id: "tower-2".to_string(),
        status: ExecutionStatus::Failed,
        exit_code: Some(1),
        stdout: String::new(),
        stderr: "error message".to_string(),
        duration_ms: 100,
        error: Some("Command failed".to_string()),
    };

    assert_eq!(result.status, ExecutionStatus::Failed);
    assert!(result.error.is_some());
}

// ============================================================================
// JobInfo Tests
// ============================================================================

#[test]
fn test_job_info_creation() {
    let req = ExecutionRequest::new("echo job");
    let info = JobInfo {
        id: "job-abc".to_string(),
        request: req,
        status: ExecutionStatus::Queued,
        pid: None,
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        started_at: SystemTime::now(),
        completed_at: None,
    };

    assert_eq!(info.id, "job-abc");
    assert_eq!(info.status, ExecutionStatus::Queued);
    assert!(info.pid.is_none());
}

#[test]
fn test_job_info_clone() {
    let req = ExecutionRequest::new("test");
    let info = JobInfo {
        id: "j1".to_string(),
        request: req,
        status: ExecutionStatus::Running,
        pid: Some(100),
        exit_code: None,
        stdout: "out".to_string(),
        stderr: "err".to_string(),
        started_at: SystemTime::now(),
        completed_at: None,
    };

    let cloned = info.clone();
    assert_eq!(info.id, cloned.id);
    assert_eq!(info.status, cloned.status);
    assert_eq!(info.pid, cloned.pid);
}
