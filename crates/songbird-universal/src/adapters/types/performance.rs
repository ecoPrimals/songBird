//! Performance metrics and monitoring types for capability providers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Performance metrics for providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Success rate as percentage
    pub success_rate: f64,
    /// Current load as percentage
    pub current_load: f64,
    /// Last updated timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 100.0,
            success_rate: 100.0,
            current_load: 0.0,
            last_updated: Some(Utc::now()),
        }
    }
}

/// Performance metrics for primal selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalPerformanceMetrics {
    pub response_time_ms: u64,
    pub success_rate: f64,
    pub availability: f64,
}

/// Performance requirements for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms: u64,
    /// Minimum required success rate as percentage
    pub min_success_rate: f64,
    /// Maximum acceptable load as percentage
    pub max_load: f64,
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self {
            max_response_time_ms: 1000,
            min_success_rate: 95.0,
            max_load: 80.0,
        }
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(response_time_ms: f64, success_rate: f64, current_load: f64) -> Self {
        Self {
            avg_response_time_ms: response_time_ms,
            success_rate,
            current_load,
            last_updated: Some(Utc::now()),
        }
    }

    /// Check if metrics meet the given requirements
    pub fn meets_requirements(&self, requirements: &PerformanceRequirements) -> bool {
        self.avg_response_time_ms <= requirements.max_response_time_ms as f64
            && self.success_rate >= requirements.min_success_rate
            && self.current_load <= requirements.max_load
    }

    /// Calculate performance score (0.0 to 1.0, higher is better)
    pub fn performance_score(&self) -> f64 {
        let response_score = (1000.0 - self.avg_response_time_ms.min(1000.0)) / 1000.0;
        let success_score = self.success_rate / 100.0;
        let load_score = (100.0 - self.current_load.min(100.0)) / 100.0;

        (response_score + success_score + load_score) / 3.0
    }

    /// Update metrics with new values
    pub fn update(&mut self, response_time_ms: f64, success_rate: f64, current_load: f64) {
        self.avg_response_time_ms = response_time_ms;
        self.success_rate = success_rate;
        self.current_load = current_load;
        self.last_updated = Some(Utc::now());
    }
}

impl PrimalPerformanceMetrics {
    /// Create new primal performance metrics
    pub fn new(response_time_ms: u64, success_rate: f64, availability: f64) -> Self {
        Self {
            response_time_ms,
            success_rate,
            availability,
        }
    }

    /// Calculate overall performance score
    pub fn overall_score(&self) -> f64 {
        let response_score = (1000.0 - self.response_time_ms.min(1000) as f64) / 1000.0;
        let success_score = self.success_rate / 100.0;
        let availability_score = self.availability / 100.0;

        (response_score + success_score + availability_score) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.avg_response_time_ms, 100.0);
        assert_eq!(metrics.success_rate, 100.0);
        assert_eq!(metrics.current_load, 0.0);
        assert!(metrics.last_updated.is_some());
    }

    #[test]
    fn test_performance_score_calculation() {
        let metrics = PerformanceMetrics::new(100.0, 95.0, 50.0);
        let score = metrics.performance_score();

        // Should be a reasonable score for good metrics
        assert!(score > 0.8);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_meets_requirements() {
        let metrics = PerformanceMetrics::new(500.0, 98.0, 60.0);
        let requirements = PerformanceRequirements::default();

        assert!(metrics.meets_requirements(&requirements));

        let strict_requirements = PerformanceRequirements {
            max_response_time_ms: 100,
            min_success_rate: 99.0,
            max_load: 50.0,
        };

        assert!(!metrics.meets_requirements(&strict_requirements));
    }

    #[test]
    fn test_primal_performance_metrics() {
        let metrics = PrimalPerformanceMetrics::new(200, 95.0, 99.0);
        let score = metrics.overall_score();

        assert!(score > 0.8);
        assert!(score <= 1.0);
    }
}
