//! Client for communicating with remote execution agents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Client for communicating with execution agents on remote towers
#[derive(Clone)]
pub struct ExecutionClient {
    http_client: reqwest::Client,
}

impl ExecutionClient {
    /// Create a new execution client
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }
    
    /// Execute a command on a remote tower
    pub async fn execute_command(
        &self,
        tower_endpoint: &str,
        request: ExecutionRequest,
    ) -> Result<ExecutionResponse, ExecutionError> {
        let url = format!("{}/api/v1/execution/command", tower_endpoint);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ExecutionError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExecutionError::Remote(error_text));
        }
        
        response.json()
            .await
            .map_err(|e| ExecutionError::Deserialization(e.to_string()))
    }
    
    /// Get job status from a remote tower
    pub async fn get_job_status(
        &self,
        tower_endpoint: &str,
        job_id: &str,
    ) -> Result<JobInfo, ExecutionError> {
        let url = format!("{}/api/v1/execution/jobs/{}", tower_endpoint, job_id);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExecutionError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExecutionError::Remote(error_text));
        }
        
        response.json()
            .await
            .map_err(|e| ExecutionError::Deserialization(e.to_string()))
    }
    
    /// Stop a running job on a remote tower
    pub async fn stop_job(
        &self,
        tower_endpoint: &str,
        job_id: &str,
        signal: Option<String>,
    ) -> Result<StopJobResponse, ExecutionError> {
        let url = format!("{}/api/v1/execution/jobs/{}/stop", tower_endpoint, job_id);
        
        let request = StopJobRequest { signal };
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ExecutionError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExecutionError::Remote(error_text));
        }
        
        response.json()
            .await
            .map_err(|e| ExecutionError::Deserialization(e.to_string()))
    }
}

impl Default for ExecutionClient {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export types from agent for convenience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub id: Option<String>,
    pub command: String,
    pub working_dir: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub background: bool,
    pub timeout_seconds: Option<u64>,
    pub capture_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub job_id: String,
    pub status: ExecutionStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Timeout,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub request: ExecutionRequest,
    pub status: ExecutionStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StopJobRequest {
    signal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopJobResponse {
    pub job_id: String,
    pub status: ExecutionStatus,
    pub signal: String,
}

/// Execution client errors
#[derive(Debug, Clone)]
pub enum ExecutionError {
    Network(String),
    Remote(String),
    Deserialization(String),
    TowerNotFound(String),
    MultipleFailures(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Remote(msg) => write!(f, "Remote error: {}", msg),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            Self::TowerNotFound(msg) => write!(f, "Tower not found: {}", msg),
            Self::MultipleFailures(msg) => write!(f, "Multiple failures: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_client_creation() {
        let client = ExecutionClient::new();
        assert!(true); // Just ensure it constructs
    }
}

