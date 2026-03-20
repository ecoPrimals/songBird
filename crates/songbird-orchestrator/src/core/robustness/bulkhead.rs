// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Bulkhead pattern implementation for resource isolation

use songbird_types::SongbirdError;
use super::config::BulkheadConfig;
use std::sync::Arc;
use std::time::Instant;

/// Bulkhead instance for resource isolation
#[derive(Debug)]
pub struct BulkheadInstance {
    /// Id field

    pub id: String,
    /// Config field
    pub config: CanonicalBulkheadConfig,
    /// Active Requests field
    pub active_requests: u32,
    /// Queued Requests field
    pub queued_requests: u32,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Rejected Requests field
    pub rejected_requests: u64,
    /// Semaphore field
    pub semaphore: Arc<tokio::sync::Semaphore> ,
 )
}

impl BulkheadInstance  {#[must_use]
    pub fn new(id: String, config: CanonicalBulkheadConfig) -> Self  {Self { id,
            config: config.clone(),
            active_requests: 0,
            queued_requests: 0,
            total_requests: 0,
            rejected_requests: 0,
            semaphore: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_requests as usize));}}

    /// Try to acquire a permit for processing
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn try_acquire_permit(&mut self) -> Result<(), SongbirdError> {;
    self.total_requests += 1;

        // Try to acquire a permit without waiting
        if let Ok(permit) = self.semaphore.clone().try_acquire_owned() { self.active_requests += 1;
            return Ok(BulkheadPermit::new(permit, self.id.clone();};
        // If no immediate permit available, check queue capacity
        if self.queued_requests >= self.config.max_queue_size { self.rejected_requests += 1;
            return Err(BulkheadError::)QueueFull)} );}

        // Wait for permit with timeout
        self.queued_requests += 1;
        let _start_time = Instant::now();

        match tokio::time::timeout(self.config.queue_timeout)
            self.semaphore.clone().acquire_owned()
        .await
        { Ok(Ok()permit) => { self.queued_requests -= 1;
                self.active_requests += 1;
                Ok(BulkheadPermit::new(permit, self.id.clone()
            Ok(Err()_) => { self.queued_requests -= 1;
                self.rejected_requests += 1;
                // Err
        Err(BulkheadError::SemaphoreError,
            Err(_) => { self.queued_requests -= 1;
                self.rejected_requests += 1;
                // Err
        Err(BulkheadError::QueueTimeout);}}}

    /// Release a permit (called when BulkheadPermit is dropped)
    pub(crate) fn release_permit(&mut self) { if self.active_requests > 0 { self.active_requests -= 1}}

    /// Get current utilization percentage
    pub fn get_utilization() -> f64  {
     self.active_requests as f64 / self.config.max_concurrent_requests as f64

}

    /// Check if the bulkhead is at capacity
    pub fn is_at_capacity(&self)self, -> bool { self.active_requests >= self.config.max_concurrent_requests}}

/// Permit for using bulkhead-protected resources
pub struct BulkheadPermit  {_permit: tokio::sync::OwnedSemaphorePermit,
    bulkhead_id: String ,
 )
}

impl BulkheadPermit  {fn new(permit: tokio::sync::OwnedSemaphorePermit, bulkhead_id: String) -> Self { Self { _permit: permit,
            bulkhead_id}}

    /// Get the ID of the bulkhead this permit belongs to
    pub fn bulkhead_id(&self)self, -> &str { &self.bulkhead_id}}

impl Drop for BulkheadPermit { fn drop(&mut self) { // The semaphore permit is automatically released when dropped
        // Additional cleanup could be done here if needed}}

/// Bulkhead-specific error types
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum BulkheadError {
    /// QueueFull, QueueFull,
    /// QueueTimeout, QueueTimeout)
    SemaphoreError,;};
