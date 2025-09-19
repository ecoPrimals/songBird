//! Bulkhead pattern implementation for resource isolation

use super::config::BulkheadConfig;
use songbird_errors::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::time::Instant;

/// Bulkhead instance for resource isolation
#[derive(Debug)]
pub struct BulkheadInstance {
    pub id: String,
    pub config: BulkheadConfig,
    pub active_requests: u32,
    pub queued_requests: u32,
    pub total_requests: u64,
    pub rejected_requests: u64,
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

impl BulkheadInstance {
    pub fn new(id: String, config: BulkheadConfig) -> Self {
        Self {
            id,
            config: config.clone(),
            active_requests: 0,
            queued_requests: 0,
            total_requests: 0,
            rejected_requests: 0,
            semaphore: Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_requests as usize,
            )),
        }
    }

    /// Try to acquire a permit for processing
    pub async fn try_acquire_permit(&mut self) -> SongbirdResult<BulkheadPermit> {
        self.total_requests += 1;

        // Try to acquire a permit without waiting
        if let Ok(permit) = self.semaphore.clone().try_acquire_owned() {
            self.active_requests += 1;
            return Ok(BulkheadPermit::new(permit, self.id.clone()));
        }

        // If no immediate permit available, check queue capacity
        if self.queued_requests >= self.config.max_queue_size {
            self.rejected_requests += 1;
            return Err(BulkheadError::QueueFull.into());
        }

        // Wait for permit with timeout
        self.queued_requests += 1;
        let _start_time = Instant::now();

        match tokio::time::timeout(
            self.config.queue_timeout,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => {
                self.queued_requests -= 1;
                self.active_requests += 1;
                Ok(BulkheadPermit::new(permit, self.id.clone()))
            }
            Ok(Err(_)) => {
                self.queued_requests -= 1;
                self.rejected_requests += 1;
                Err(BulkheadError::SemaphoreError.into())
            }
            Err(_) => {
                self.queued_requests -= 1;
                self.rejected_requests += 1;
                Err(BulkheadError::QueueTimeout.into())
            }
        }
    }

    /// Release a permit (called when BulkheadPermit is dropped)
    pub(crate) fn release_permit(&mut self) {
        if self.active_requests > 0 {
            self.active_requests -= 1;
        }
    }

    /// Get current utilization percentage
    pub fn get_utilization(&self) -> f64 {
        self.active_requests as f64 / self.config.max_concurrent_requests as f64
    }

    /// Check if the bulkhead is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.active_requests >= self.config.max_concurrent_requests
    }
}

/// Permit for using bulkhead-protected resources
pub struct BulkheadPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    bulkhead_id: String,
}

impl BulkheadPermit {
    fn new(permit: tokio::sync::OwnedSemaphorePermit, bulkhead_id: String) -> Self {
        Self {
            _permit: permit,
            bulkhead_id,
        }
    }

    /// Get the ID of the bulkhead this permit belongs to
    pub fn bulkhead_id(&self) -> &str {
        &self.bulkhead_id
    }
}

impl Drop for BulkheadPermit {
    fn drop(&mut self) {
        // The semaphore permit is automatically released when dropped
        // Additional cleanup could be done here if needed
    }
}

/// Bulkhead-specific error types
#[derive(Debug, Clone)]
pub enum BulkheadError {
    QueueFull,
    QueueTimeout,
    SemaphoreError,
}

impl From<BulkheadError> for SongbirdError {
    fn from(error: BulkheadError) -> Self {
        match error {
            BulkheadError::QueueFull => SongbirdError::Service {
                service: "Bulkhead".to_string(),
                message: "Bulkhead queue is full".to_string(),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Reduce load or increase bulkhead capacity".to_string()],
            },
            BulkheadError::QueueTimeout => SongbirdError::Service {
                service: "Bulkhead".to_string(),
                message: "Bulkhead queue timeout".to_string(),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Increase timeout or reduce load".to_string()],
            },
            BulkheadError::SemaphoreError => SongbirdError::Service {
                service: "Bulkhead".to_string(),
                message: "Bulkhead semaphore error".to_string(),
                suggested_alternatives: Vec::new(),
                recovery_actions: vec!["Check bulkhead configuration".to_string()],
            },
        }
    }
}
