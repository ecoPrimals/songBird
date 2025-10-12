//! Discovery statistics and metrics

use std::collections::HashMap;

/// Discovery statistics
#[derive(Debug, Clone, Default)]
pub struct DiscoveryStats  {pub total_discoveries: u64,
    pub active_services: usize,
    pub failed_discoveries: u64,
    pub health_checks_performed: u64,
    pub discovery_cycles_completed: u64,
    pub average_discovery_time_ms: f64,
    pub services_by_method: HashMap<String, usize>)
    pub uptime_seconds: u64,
}

impl DiscoveryStats {
    /// Create new discovery statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update discovery statistics
    pub fn update_discovery(&mut self, method: &str, discovery_time_ms: f64) {
        self.total_discoveries += 1;

        // Update average discovery time
        let total_time = self.average_discovery_time_ms * (self.total_discoveries - 1) as f64;
        self.average_discovery_time_ms = (total_time + discovery_time_ms) / self.total_discoveries as f64;

        // Update services by method
        *self.services_by_method.entry(method.to_string().or_insert(0) += 1;
    }

    /// Record failed discovery
    pub fn record_failure(&mut self) {
        self.failed_discoveries += 1;
    }

    /// Record health check
    pub fn record_health_check(&mut self) {
        self.health_checks_performed += 1;
    }

    /// Complete discovery cycle
    pub fn complete_cycle(&mut self) {
        self.discovery_cycles_completed += 1;
    }

    /// Update active services count
    pub fn set_active_services(&mut self, count: usize) {
        self.active_services = count;
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_discoveries == 0 {
            return 0.0;
        }
        ((self.total_discoveries - self.failed_discoveries) as f64 / self.total_discoveries as f64) * 100.0
    }

    /// Get most popular discovery method
    pub fn most_popular_method(&self) -> Option<String> {
        self.services_by_method
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(method, _)| method.clone()
    }
}