// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Command execution logic

use crate::ResourceLimits;
use crate::types::{ExecutionRequest, ExecutionResponse, ExecutionStatus, JobInfo};
use songbird_types::{SongbirdError, SongbirdResult};
use std::process::Stdio;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Result of running a command (before mapping to `ExecutionResponse`)
type CommandRunResult =
    Result<(std::process::ExitStatus, Option<u32>, String, String), SongbirdError>;

/// Executes commands and manages their lifecycle
pub struct CommandExecutor {
    resource_limits: ResourceLimits,
}

impl CommandExecutor {
    /// Create a new command executor
    #[must_use]
    pub const fn new(resource_limits: ResourceLimits) -> Self {
        Self {
            resource_limits,
        }
    }

    /// Execute a command (foreground)
    ///
    /// # Errors
    ///
    /// Returns an error if command parsing fails, spawn fails, or wait fails
    pub async fn execute(&self, request: ExecutionRequest) -> SongbirdResult<ExecutionResponse> {
        let job_id = request.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let started_at = SystemTime::now();

        info!("Executing command: {} (job: {})", request.command, job_id);

        // Parse command and arguments
        let (cmd, args) = Self::parse_command(&request.command)?;

        // Create tokio command
        let mut command = Command::new(cmd);
        command.args(&args);

        // Set working directory if specified
        if let Some(ref dir) = request.working_dir {
            command.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &request.env {
            command.env(key, value);
        }

        // Configure output capture
        if request.capture_output {
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
        } else {
            command.stdout(Stdio::null());
            command.stderr(Stdio::null());
        }

        // Execute with timeout
        let timeout_duration = Duration::from_secs(
            request.timeout_seconds.unwrap_or(self.resource_limits.default_timeout_seconds),
        );

        let result = timeout(timeout_duration, self.run_command_inner(command, &request)).await;

        let completed_at = SystemTime::now();
        let duration_ms = completed_at
            .duration_since(started_at)
            .ok()
            .and_then(|d| u64::try_from(d.as_millis()).ok());

        Ok(Self::handle_execution_result(
            result,
            &job_id,
            started_at,
            completed_at,
            duration_ms,
            timeout_duration,
        ))
    }

    async fn run_command_inner(
        &self,
        mut command: Command,
        request: &ExecutionRequest,
    ) -> Result<(std::process::ExitStatus, Option<u32>, String, String), SongbirdError> {
        let mut child = command.spawn().map_err(|e| SongbirdError::Runtime {
            message: format!("Failed to spawn command: {e}"),
            component: Some("command_executor".to_string()),
            debug_info: None,
        })?;

        let pid = child.id();
        debug!("Command spawned with PID: {:?}", pid);

        let (stdout, stderr) = if request.capture_output {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let stdout_task = stdout.map(|s| self.read_stream(s));
            let stderr_task = stderr.map(|s| self.read_stream(s));
            let (stdout_result, stderr_result) = tokio::join!(
                async {
                    match stdout_task {
                        Some(task) => task.await,
                        None => String::new(),
                    }
                },
                async {
                    match stderr_task {
                        Some(task) => task.await,
                        None => String::new(),
                    }
                }
            );
            (stdout_result, stderr_result)
        } else {
            (String::new(), String::new())
        };

        let status = child.wait().await.map_err(|e| SongbirdError::Runtime {
            message: format!("Failed to wait for command: {e}"),
            component: Some("command_executor".to_string()),
            debug_info: None,
        })?;

        Ok((status, pid, stdout, stderr))
    }

    fn handle_execution_result(
        result: Result<CommandRunResult, tokio::time::error::Elapsed>,
        job_id: &str,
        started_at: SystemTime,
        completed_at: SystemTime,
        duration_ms: Option<u64>,
        timeout_duration: Duration,
    ) -> ExecutionResponse {
        match result {
            Ok(Ok((status, pid, stdout, stderr))) => {
                let exit_code = status.code();
                let execution_status = if status.success() {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed
                };
                info!(
                    "Command completed with status: {} (exit code: {:?}, job: {})",
                    execution_status, exit_code, job_id
                );
                ExecutionResponse {
                    job_id: job_id.to_string(),
                    status: execution_status,
                    pid,
                    exit_code,
                    stdout,
                    stderr,
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                }
            }
            Ok(Err(e)) => {
                error!("Command execution error: {} (job: {})", e, job_id);
                ExecutionResponse {
                    job_id: job_id.to_string(),
                    status: ExecutionStatus::Failed,
                    pid: None,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("Execution error: {e}"),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                }
            }
            Err(_) => {
                warn!("Command timed out (job: {})", job_id);
                ExecutionResponse {
                    job_id: job_id.to_string(),
                    status: ExecutionStatus::Timeout,
                    pid: None,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!(
                        "Command timed out after {} seconds",
                        timeout_duration.as_secs()
                    ),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                }
            }
        }
    }

    /// Execute a command in the background
    ///
    /// # Errors
    ///
    /// Returns an error if command parsing fails or spawn fails
    pub async fn execute_background(&self, request: ExecutionRequest) -> SongbirdResult<JobInfo> {
        tokio::task::yield_now().await;
        let job_id = request.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let started_at = SystemTime::now();

        info!("Starting background command: {} (job: {})", request.command, job_id);

        // Parse command and arguments
        let (cmd, args) = Self::parse_command(&request.command)?;

        // Create tokio command
        let mut command = Command::new(cmd);
        command.args(&args);

        // Set working directory if specified
        if let Some(ref dir) = request.working_dir {
            command.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &request.env {
            command.env(key, value);
        }

        // Always capture output for background jobs
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Spawn the command
        let child = command.spawn().map_err(|e| SongbirdError::Runtime {
            message: format!("Failed to spawn background command: {e}"),
            component: Some("command_executor".to_string()),
            debug_info: None,
        })?;

        let pid = child.id();
        debug!("Background command spawned with PID: {:?}", pid);

        Ok(JobInfo {
            id: job_id,
            request,
            status: ExecutionStatus::Running,
            pid,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            started_at,
            completed_at: None,
        })
    }

    /// Parse command string into program and arguments
    fn parse_command(command: &str) -> SongbirdResult<(String, Vec<String>)> {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return Err(SongbirdError::Validation {
                message: "Empty command".to_string(),
                field: Some("command".to_string()),
                suggestion: Some("Provide a valid command to execute".to_string()),
            });
        }

        let cmd = parts[0].to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();

        Ok((cmd, args))
    }

    /// Read all content from an async stream
    async fn read_stream<R: AsyncRead + Unpin>(&self, stream: R) -> String {
        let mut reader = BufReader::new(stream);
        let mut output = String::new();
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => output.push_str(&line),
                Err(e) => {
                    error!("Error reading stream: {}", e);
                    break;
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let executor = CommandExecutor::new(ResourceLimits::default());
        let request = ExecutionRequest::new("echo hello world");

        let response = executor.execute(request).await.unwrap();

        assert_eq!(response.status, ExecutionStatus::Completed);
        assert_eq!(response.exit_code, Some(0));
        assert!(response.stdout.contains("hello world"));
    }

    #[tokio::test]
    async fn test_execute_with_env() {
        let executor = CommandExecutor::new(ResourceLimits::default());
        let request = ExecutionRequest::new("printenv TEST_VAR").with_env("TEST_VAR", "test_value");

        let response = executor.execute(request).await.unwrap();

        assert_eq!(response.status, ExecutionStatus::Completed);
        assert!(response.stdout.contains("test_value"));
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let executor = CommandExecutor::new(ResourceLimits::default());
        let request = ExecutionRequest::new("false"); // Command that always fails

        let response = executor.execute(request).await.unwrap();

        assert_eq!(response.status, ExecutionStatus::Failed);
        assert_ne!(response.exit_code, Some(0));
    }

    #[test]
    fn test_parse_command() {
        let (cmd, args) = CommandExecutor::parse_command("ls -la /tmp").unwrap();
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
    }
}
