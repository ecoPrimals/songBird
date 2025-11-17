//! Health monitoring

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct HealthMonitor {
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    start_time: Instant,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn get_metrics(&self) -> HealthMetrics {
        HealthMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub uptime_seconds: u64,
}
