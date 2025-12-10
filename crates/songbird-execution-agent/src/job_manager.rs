//! Background job management and tracking

use crate::types::{ExecutionStatus, JobInfo};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manages background jobs
pub struct JobManager {
    jobs: Arc<RwLock<HashMap<String, JobInfo>>>,
    max_concurrent_jobs: usize,
    log_retention_seconds: u64,
}

impl JobManager {
    /// Create a new job manager
    pub fn new(max_concurrent_jobs: usize, log_retention_seconds: u64) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_jobs,
            log_retention_seconds,
        }
    }

    /// Add a new job
    pub async fn add_job(&self, job: JobInfo) -> SongbirdResult<()> {
        let mut jobs = self.jobs.write().await;

        // Check concurrent job limit
        let running_count = jobs
            .values()
            .filter(|j| matches!(j.status, ExecutionStatus::Running | ExecutionStatus::Queued))
            .count();

        if running_count >= self.max_concurrent_jobs {
            return Err(SongbirdError::Configuration {
                message: format!(
                    "Maximum concurrent jobs ({}) reached. Current: {}",
                    self.max_concurrent_jobs, running_count
                ),
                field: Some("max_concurrent_jobs".to_string()),
                suggestion: Some(
                    "Increase max_concurrent_jobs or wait for jobs to complete".to_string(),
                ),
            });
        }

        info!("Adding job: {} (status: {})", job.id, job.status);
        jobs.insert(job.id.clone(), job);
        Ok(())
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> SongbirdResult<JobInfo> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned().ok_or_else(|| SongbirdError::Registry {
            message: format!("Job not found: {}", job_id),
            service_name: Some(job_id.to_string()),
            operation: "get".to_string(),
        })
    }

    /// Update job status
    pub async fn update_job(
        &self,
        job_id: &str,
        update_fn: impl FnOnce(&mut JobInfo),
    ) -> SongbirdResult<()> {
        let mut jobs = self.jobs.write().await;

        let job = jobs.get_mut(job_id).ok_or_else(|| SongbirdError::Registry {
            message: format!("Job not found: {}", job_id),
            service_name: Some(job_id.to_string()),
            operation: "update".to_string(),
        })?;

        update_fn(job);
        debug!("Updated job: {} (status: {})", job_id, job.status);
        Ok(())
    }

    /// Mark job as completed
    pub async fn complete_job(
        &self,
        job_id: &str,
        exit_code: i32,
        stdout: String,
        stderr: String,
    ) -> SongbirdResult<()> {
        self.update_job(job_id, |job| {
            job.status = if exit_code == 0 {
                ExecutionStatus::Completed
            } else {
                ExecutionStatus::Failed
            };
            job.exit_code = Some(exit_code);
            job.stdout = stdout;
            job.stderr = stderr;
            job.completed_at = Some(SystemTime::now());
        })
        .await?;

        info!("Job completed: {} (exit code: {})", job_id, exit_code);
        Ok(())
    }

    /// Mark job as failed
    pub async fn fail_job(&self, job_id: &str, error: String) -> SongbirdResult<()> {
        self.update_job(job_id, |job| {
            job.status = ExecutionStatus::Failed;
            job.stderr = error;
            job.completed_at = Some(SystemTime::now());
        })
        .await?;

        warn!("Job failed: {}", job_id);
        Ok(())
    }

    /// Stop a running job
    pub async fn stop_job(&self, job_id: &str) -> SongbirdResult<u32> {
        let job = self.get_job(job_id).await?;

        let pid = job.pid.ok_or_else(|| SongbirdError::Runtime {
            message: "Job has no PID (not running?)".to_string(),
            component: Some("job_manager".to_string()),
            debug_info: None,
        })?;

        // Send SIGTERM signal
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let pid_nix = Pid::from_raw(pid as i32);
            kill(pid_nix, Signal::SIGTERM).map_err(|e| SongbirdError::Runtime {
                message: format!("Failed to send SIGTERM to process {}: {}", pid, e),
                component: Some("job_manager".to_string()),
                debug_info: None,
            })?;
        }

        #[cfg(not(unix))]
        {
            return Err(SongbirdError::Runtime {
                message: "Process stopping is only supported on Unix systems".to_string(),
                component: Some("job_manager".to_string()),
                debug_info: None,
            });
        }

        self.update_job(job_id, |job| {
            job.status = ExecutionStatus::Stopped;
            job.completed_at = Some(SystemTime::now());
        })
        .await?;

        info!("Stopped job: {} (PID: {})", job_id, pid);
        Ok(pid)
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Vec<JobInfo> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// List jobs with a specific status
    pub async fn list_jobs_by_status(&self, status: ExecutionStatus) -> Vec<JobInfo> {
        let jobs = self.jobs.read().await;
        jobs.values().filter(|job| job.status == status).cloned().collect()
    }

    /// Clean up old completed jobs
    pub async fn cleanup_old_jobs(&self) -> usize {
        let mut jobs = self.jobs.write().await;
        let now = SystemTime::now();
        let retention = std::time::Duration::from_secs(self.log_retention_seconds);

        let initial_count = jobs.len();

        jobs.retain(|_, job| {
            // Keep running/queued jobs
            if matches!(job.status, ExecutionStatus::Running | ExecutionStatus::Queued) {
                return true;
            }

            // Keep recent jobs
            if let Some(completed_at) = job.completed_at {
                if let Ok(elapsed) = now.duration_since(completed_at) {
                    return elapsed < retention;
                }
            }

            false
        });

        let removed = initial_count - jobs.len();
        if removed > 0 {
            info!("Cleaned up {} old jobs", removed);
        }

        removed
    }

    /// Get job statistics
    pub async fn get_stats(&self) -> JobStats {
        let jobs = self.jobs.read().await;

        let mut stats = JobStats {
            total: jobs.len(),
            ..Default::default()
        };

        for job in jobs.values() {
            match job.status {
                ExecutionStatus::Queued => stats.queued += 1,
                ExecutionStatus::Running => stats.running += 1,
                ExecutionStatus::Completed => stats.completed += 1,
                ExecutionStatus::Failed => stats.failed += 1,
                ExecutionStatus::Timeout => stats.timeout += 1,
                ExecutionStatus::Stopped => stats.stopped += 1,
            }
        }

        stats
    }
}

/// Job statistics
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobStats {
    pub total: usize,
    pub queued: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub timeout: usize,
    pub stopped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutionRequest;

    fn create_test_job(id: &str, status: ExecutionStatus) -> JobInfo {
        JobInfo {
            id: id.to_string(),
            request: ExecutionRequest::new("echo test"),
            status,
            pid: Some(12345),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            started_at: SystemTime::now(),
            completed_at: None,
        }
    }

    #[tokio::test]
    async fn test_add_and_get_job() {
        let manager = JobManager::new(10, 86400);
        let job = create_test_job("test-1", ExecutionStatus::Running);

        manager.add_job(job.clone()).await.unwrap();

        let retrieved = manager.get_job("test-1").await.unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_job_limit() {
        let manager = JobManager::new(2, 86400);

        manager.add_job(create_test_job("test-1", ExecutionStatus::Running)).await.unwrap();
        manager.add_job(create_test_job("test-2", ExecutionStatus::Running)).await.unwrap();

        let result = manager.add_job(create_test_job("test-3", ExecutionStatus::Running)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_job() {
        let manager = JobManager::new(10, 86400);
        let job = create_test_job("test-1", ExecutionStatus::Running);

        manager.add_job(job).await.unwrap();
        manager.complete_job("test-1", 0, "output".to_string(), String::new()).await.unwrap();

        let updated = manager.get_job("test-1").await.unwrap();
        assert_eq!(updated.status, ExecutionStatus::Completed);
        assert_eq!(updated.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_job_stats() {
        let manager = JobManager::new(10, 86400);

        manager.add_job(create_test_job("test-1", ExecutionStatus::Running)).await.unwrap();
        manager.add_job(create_test_job("test-2", ExecutionStatus::Completed)).await.unwrap();
        manager.add_job(create_test_job("test-3", ExecutionStatus::Failed)).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total, 3);
        assert_eq!(stats.running, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 1);
    }
}
