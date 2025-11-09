//! Command execution logic

use crate::types::{ExecutionRequest, ExecutionResponse, ExecutionStatus, JobInfo};
use crate::ResourceLimits;
use songbird_types::{SongbirdError, SongbirdResult};
use std::process::Stdio;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Executes commands and manages their lifecycle
pub struct CommandExecutor {
    resource_limits: ResourceLimits,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(resource_limits: ResourceLimits) -> Self {
        Self { resource_limits }
    }
    
    /// Execute a command (foreground)
    pub async fn execute(&self, request: ExecutionRequest) -> SongbirdResult<ExecutionResponse> {
        let job_id = request.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let started_at = SystemTime::now();
        
        info!("Executing command: {} (job: {})", request.command, job_id);
        
        // Parse command and arguments
        let (cmd, args) = self.parse_command(&request.command)?;
        
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
            request.timeout_seconds
                .unwrap_or(self.resource_limits.default_timeout_seconds)
        );
        
        let result = timeout(timeout_duration, async {
            let mut child = command.spawn().map_err(|e| {
                SongbirdError::Runtime {
                    message: format!("Failed to spawn command: {}", e),
                    component: Some("command_executor".to_string()),
                    debug_info: None,
                }
            })?;
            
            let pid = child.id();
            debug!("Command spawned with PID: {:?}", pid);
            
            // Capture output if enabled
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
            
            // Wait for command to complete
            let status = child.wait().await.map_err(|e| {
                SongbirdError::Runtime {
                    message: format!("Failed to wait for command: {}", e),
                    component: Some("command_executor".to_string()),
                    debug_info: None,
                }
            })?;
            
            Ok::<_, SongbirdError>((status, pid, stdout, stderr))
        })
        .await;
        
        let completed_at = SystemTime::now();
        let duration_ms = completed_at.duration_since(started_at).ok()
            .map(|d| d.as_millis() as u64);
        
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
                
                Ok(ExecutionResponse {
                    job_id,
                    status: execution_status,
                    pid,
                    exit_code,
                    stdout,
                    stderr,
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                })
            }
            Ok(Err(e)) => {
                error!("Command execution error: {} (job: {})", e, job_id);
                Ok(ExecutionResponse {
                    job_id,
                    status: ExecutionStatus::Failed,
                    pid: None,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("Execution error: {}", e),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                })
            }
            Err(_) => {
                warn!("Command timed out (job: {})", job_id);
                Ok(ExecutionResponse {
                    job_id,
                    status: ExecutionStatus::Timeout,
                    pid: None,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("Command timed out after {} seconds", timeout_duration.as_secs()),
                    started_at,
                    completed_at: Some(completed_at),
                    duration_ms,
                })
            }
        }
    }
    
    /// Execute a command in the background
    pub async fn execute_background(&self, request: ExecutionRequest) -> SongbirdResult<JobInfo> {
        let job_id = request.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        let started_at = SystemTime::now();
        
        info!("Starting background command: {} (job: {})", request.command, job_id);
        
        // Parse command and arguments
        let (cmd, args) = self.parse_command(&request.command)?;
        
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
        let child = command.spawn().map_err(|e| {
            SongbirdError::Runtime {
                message: format!("Failed to spawn background command: {}", e),
                component: Some("command_executor".to_string()),
                debug_info: None,
            }
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
    fn parse_command(&self, command: &str) -> SongbirdResult<(String, Vec<String>)> {
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
        let request = ExecutionRequest::new("printenv TEST_VAR")
            .with_env("TEST_VAR", "test_value");
        
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
        let executor = CommandExecutor::new(ResourceLimits::default());
        
        let (cmd, args) = executor.parse_command("ls -la /tmp").unwrap();
        assert_eq!(cmd, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
    }
}

