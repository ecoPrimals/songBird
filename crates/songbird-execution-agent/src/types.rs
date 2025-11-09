//! Type definitions for remote execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

/// Request to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique request ID
    pub id: Option<String>,
    
    /// Command to execute
    pub command: String,
    
    /// Working directory (optional)
    pub working_dir: Option<PathBuf>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Run in background
    pub background: bool,
    
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
    
    /// Capture output
    pub capture_output: bool,
}

/// Response from execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    /// Job ID (for tracking)
    pub job_id: String,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Process ID (if running)
    pub pid: Option<u32>,
    
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
    
    /// Standard output
    pub stdout: String,
    
    /// Standard error
    pub stderr: String,
    
    /// Start time
    pub started_at: SystemTime,
    
    /// End time (if completed)
    pub completed_at: Option<SystemTime>,
    
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    /// Job is queued
    Queued,
    
    /// Job is running
    Running,
    
    /// Job completed successfully
    Completed,
    
    /// Job failed
    Failed,
    
    /// Job timed out
    Timeout,
    
    /// Job was stopped
    Stopped,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Timeout => write!(f, "timeout"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    /// Job ID
    pub id: String,
    
    /// Execution request
    pub request: ExecutionRequest,
    
    /// Current status
    pub status: ExecutionStatus,
    
    /// Process ID (if running)
    pub pid: Option<u32>,
    
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
    
    /// Captured stdout
    pub stdout: String,
    
    /// Captured stderr
    pub stderr: String,
    
    /// Start time
    pub started_at: SystemTime,
    
    /// End time (if completed)
    pub completed_at: Option<SystemTime>,
}

/// Request to stop a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobRequest {
    /// Signal to send (e.g., "SIGTERM", "SIGKILL")
    pub signal: Option<String>,
}

/// Response from stopping a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobResponse {
    /// Job ID
    pub job_id: String,
    
    /// Status after stop
    pub status: ExecutionStatus,
    
    /// Signal sent
    pub signal: String,
}

/// Broadcast execution request (multiple towers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastRequest {
    /// Tower IDs to execute on
    pub tower_ids: Vec<String>,
    
    /// Command to execute
    pub command: String,
    
    /// Working directory (optional)
    pub working_dir: Option<PathBuf>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Wait for completion before returning
    pub wait_for_completion: bool,
    
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Broadcast execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResponse {
    /// Broadcast ID
    pub broadcast_id: String,
    
    /// Results from each tower
    pub results: Vec<TowerExecutionResult>,
}

/// Result from a single tower in broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerExecutionResult {
    /// Tower ID
    pub tower_id: String,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
    
    /// Standard output
    pub stdout: String,
    
    /// Standard error  
    pub stderr: String,
    
    /// Duration in milliseconds
    pub duration_ms: u64,
    
    /// Error message (if failed)
    pub error: Option<String>,
}

impl ExecutionRequest {
    /// Create a new execution request with defaults
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            id: Some(Uuid::new_v4().to_string()),
            command: command.into(),
            working_dir: None,
            env: HashMap::new(),
            background: false,
            timeout_seconds: None,
            capture_output: true,
        }
    }
    
    /// Set working directory
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
    
    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    
    /// Set background execution
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_request_builder() {
        let req = ExecutionRequest::new("echo hello")
            .with_working_dir("/tmp")
            .with_env("KEY", "value")
            .with_background(true)
            .with_timeout(30);
        
        assert_eq!(req.command, "echo hello");
        assert_eq!(req.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(req.env.get("KEY"), Some(&"value".to_string()));
        assert!(req.background);
        assert_eq!(req.timeout_seconds, Some(30));
    }
    
    #[test]
    fn test_execution_status_display() {
        assert_eq!(ExecutionStatus::Running.to_string(), "running");
        assert_eq!(ExecutionStatus::Completed.to_string(), "completed");
        assert_eq!(ExecutionStatus::Failed.to_string(), "failed");
    }
}

