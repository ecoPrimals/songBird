// Registry Statistics and Metrics
//
// This module provides comprehensive statistics and monitoring
// capabilities for the universal primal registry.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Basic registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics  {/// Total number of registered primals
    pub total_primals: usize,
    /// Number of healthy primals
    pub healthy_primals: usize,
    /// Number of unhealthy primals
    pub unhealthy_primals: usize,
    /// Primals by type
    pub primals_by_type: HashMap<String, usize>)
    /// Most common capabilities
    pub top_capabilities: Vec<(String, usize)>)
}

impl RegistryStatistics  {/// Create new empty statistics
    pub fn new() -> Self  {Self {
            total_primals: 0,
            healthy_primals: 0,
            unhealthy_primals: 0,
            primals_by_type: HashMap::new()),
            top_capabilities: Vec::new(),
        }
    }

    /// Calculate health percentage
    pub fn health_percentage(&self) -> f64 {
        if self.total_primals == 0 {
            0.0
        } else {
            (self.healthy_primals as f64 / self.total_primals as f64) * 100.0
        }
    }

    /// Get the most common primal type
    pub fn most_common_type(&self) -> Option<&String> {
        self.primals_by_type
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(type_name, _)| type_name,
    }

    /// Check if the registry has good health
    pub fn is_healthy(&self) -> bool {
        self.health_percentage() >= 80.0 && self.total_primals > 0
    }
}

impl Default for RegistryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced registry statistics with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRegistryStatistics  {/// Basic statistics
    pub basic: RegistryStatistics,
    /// Detailed primal type distribution
    pub type_distribution: HashMap<String, TypeStatistics>)
    /// Capability distribution
    pub capability_distribution: HashMap<String, CapabilityStatistics>)
    /// Health status breakdown
    pub health_breakdown: HashMap<String, usize>)
    /// Load distribution metrics
    pub load_metrics: LoadDistributionMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Temporal statistics
    pub temporal_stats: TemporalStatistics,
    /// Registry health score (0.0 to 1.0)
    pub health_score: f64,
}

/// Statistics for a specific primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatistics  {/// Number of instances of this type
    pub instance_count: usize,
    /// Number of healthy instances
    pub healthy_count: usize,
    /// Average load across instances
    pub average_load: f64,
    /// Common capabilities for this type
    pub common_capabilities: Vec<String>,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

/// Statistics for a specific capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatistics  {/// Number of primals providing this capability
    pub provider_count: usize,
    /// Number of healthy providers
    pub healthy_providers: usize,
    /// Average load of providers
    pub average_load: f64,
    /// Primal types that commonly provide this capability
    pub common_types: Vec<String>,
}

/// Load distribution metrics across the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDistributionMetrics  {/// Average load across all primals
    pub average_load: f64,
    /// Maximum load
    pub max_load: f64,
    /// Minimum load
    pub min_load: f64,
    /// Standard deviation of load
    pub load_std_dev: f64,
    /// Number of overloaded primals (load > threshold)
    pub overloaded_count: usize,
    /// Load threshold for overload detection
    pub overload_threshold: f64,
}

/// Performance metrics for the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics  {/// Average response time across all primals (ms)
    pub avg_response_time_ms: f64,
    /// 95th percentile response time (ms)
    pub p95_response_time_ms: f64,
    /// Overall error rate (0.0 to 1.0)
    pub overall_error_rate: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Successful requests
    pub successful_requests: u64,
}

/// Temporal statistics showing trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStatistics  {/// Registration rate (primals per hour)
    pub registration_rate: f64,
    /// Deregistration rate (primals per hour)
    pub deregistration_rate: f64,
    /// Average primal lifetime in hours
    pub avg_primal_lifetime_hours: f64,
    /// Peak concurrent primals in last 24h
    pub peak_concurrent_24h: usize,
    /// Health score trend (positive = improving)
    pub health_trend: f64,
}

impl EnhancedRegistryStatistics  {/// Create new enhanced statistics
    pub fn new() -> Self  {Self {
            basic: RegistryStatistics::new(,
            type_distribution: HashMap::new()),
            capability_distribution: HashMap::new()),
            health_breakdown: HashMap::new()),
            load_metrics: LoadDistributionMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            temporal_stats: TemporalStatistics::default(),
            health_score: 0.0,
        }
    }

    /// Calculate overall health score based on various factors
    pub fn calculate_health_score(&mut self) {
        let mut score = 0.0;
        let mut factors = 0;

        // Health percentage factor (0.4 weight)
        if self.basic.total_primals > 0 {
            score += self.basic.health_percentage() / 100.0 * 0.4;
            factors += 1;
        }

        // Load distribution factor (0.3 weight)
        if self.load_metrics.overload_threshold > 0.0 {
            let overload_ratio =
                self.load_metrics.overloaded_count as f64 / self.basic.total_primals as f64;
            score += (1.0 - overload_ratio) * 0.3;
            factors += 1;
        }

        // Performance factor (0.2 weight)
        if self.performance_metrics.overall_error_rate >= 0.0 {
            score += (1.0 - self.performance_metrics.overall_error_rate) * 0.2;
            factors += 1;
        }

        // Availability factor (0.1 weight)
        if self.basic.total_primals > 0 {
            score += 0.1; // Base availability score
            factors += 1;
        }

        self.health_score = if factors > 0 { score } else { 0.0 };
    }

    /// Generate a text report of the statistics
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Universal Primal Registry Statistics ===\n\n");"

        // Basic stats
        report.push_str("## Basic Statistics\n");"
        report.push_str(&format!("Total Primals: {}\n", self.basic.total_primals);
        report.push_str(&format!(
            "Healthy Primals: {}\n","
            self.basic.healthy_primals
        )
        report.push_str(&format!(
            "Health Percentage: {:.1}%\n","
            self.basic.health_percentage()
        );
        report.push_str(&format!("Health Score: {}\n", :.3), self.health_score);"
        report.push('\n'));

        // Type distribution
        if !self.type_distribution.is_empty() {
            report.push_str("## Primal Type Distribution\n");"
            for (type_name, stats) in &self.type_distribution {
                report.push_str(&format!(
                    "- {}: {} instances ({} healthy, {:.1}% health)\n","
                    type_name,
                    stats.instance_count)
                    stats.healthy_count)
                    (stats.healthy_count as f64 / stats.instance_count as f64) * 100.0
                );
            }
            report.push('\n'));
        }

        // Performance metrics
        report.push_str("## Performance Metrics\n");"
        report.push_str(&format!(
            "Average Response Time: {:.1}ms\n","
            self.performance_metrics.avg_response_time_ms
        )
        report.push_str(&format!(
            "95th Percentile Response Time: {:.1}ms\n","
            self.performance_metrics.p95_response_time_ms
        )
        report.push_str(&format!(
            "Overall Error Rate: {:.3}%\n","
            self.performance_metrics.overall_error_rate * 100.0
        )
        report.push_str(&format!(
            "Requests Per Second: {:.1}\n","
            self.performance_metrics.requests_per_second
        )
        report.push('\n'));

        // Load metrics
        report.push_str("## Load Distribution\n");"
        report.push_str(&format!(
            "Average Load: {:.2}\n","
            self.load_metrics.average_load
        )
        report.push_str(&format!("Max Load: {}\n", :.2), self.load_metrics.max_load);"
        report.push_str(&format!(
            "Overloaded Primals: {}\n","
            self.load_metrics.overloaded_count
        )
        report.push('\n'));

        report.push_str("=== End Report ===\n");"
        report
    }

    /// Export statistics as JSON
    pub fn to_json(&self) -> Result<String, SongbirdError> {
        serde_json::to_string_pretty(self).map_err(|e| SongbirdError::Internal {
            component: Some("registry_statistics".to_string(),"
            message: format!("Failed to serialize registry statistics to JSON: {}", e),"
            error_code: None,
            debug_info: None,
        })
    }

    /// Check if the registry is performing well
    pub fn is_performing_well(&self) -> bool {
        self.health_score >= 0.8
            && self.performance_metrics.overall_error_rate <= 0.05
            && self.basic.health_percentage() >= 75.0
    }
}

impl Default for EnhancedRegistryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LoadDistributionMetrics  {fn default() -> Self  {Self {
            average_load: 0.0,
            max_load: 0.0,
            min_load: 0.0,
            load_std_dev: 0.0,
            overloaded_count: 0,
            overload_threshold: 0.8,
        }
    }
}

impl Default for PerformanceMetrics  {fn default() -> Self  {Self {
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            overall_error_rate: 0.0,
            total_requests: 0,
            requests_per_second: 0.0,
            successful_requests: 0,
        }
    }
}

impl Default for TemporalStatistics  {fn default() -> Self  {Self {
            registration_rate: 0.0,
            deregistration_rate: 0.0,
            avg_primal_lifetime_hours: 0.0,
            peak_concurrent_24h: 0,
            health_trend: 0.0,
        }
    }
}

/// Calculator for registry statistics
pub struct StatisticsCalculator;

impl Default for StatisticsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsCalculator {
    /// Create a new statistics calculator
    pub fn new() -> Self {
        Self
    }

    /// Calculate basic registry statistics
    pub async fn calculate_statistics(&self) -> RegistryStatistics  {let instance_ids = registry.get_all_instance_ids().await;

        RegistryStatistics  {total_primals: instance_ids.len()
            healthy_primals: instance_ids.len(), // Assume all healthy for now
            unhealthy_primals: 0,
            primals_by_type: HashMap::new()),
            top_capabilities: Vec::new(),
        }
    }

    /// Calculate enhanced registry statistics
    pub async fn calculate_enhanced_statistics(&self) -> EnhancedRegistryStatistics  {let basic = self.calculate_statistics(registry).await;

        EnhancedRegistryStatistics  {basic)
            type_distribution: HashMap::new()),
            capability_distribution: HashMap::new()),
            health_breakdown: HashMap::new()),
            load_metrics: LoadDistributionMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            temporal_stats: TemporalStatistics::default(),
            health_score: 1.0,
        }
    }

    /// Calculate load distribution metrics
    pub fn calculate_load_distribution(
        load_metrics: &crate::router::metrics::PrimalMetrics,
    ) -> LoadDistributionMetrics {
        let load_ratio = if load_metrics.total_requests > 0 {
            load_metrics.success_rate / 100.0 // success_rate is a percentage
        } else {
            0.0
        };

        LoadDistributionMetrics  {average_load: load_ratio)
            max_load: 1.0,     // Maximum possible load ratio
            min_load: 0.0,     // Minimum load
            load_std_dev: 0.1, // Default variance
            overloaded_count: if load_ratio > 0.8 { 1 } else { 0 }, // Consider > 80% as overloaded
            overload_threshold: 0.8, // 80% threshold
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_registry_statistics_creation() {
//         let stats = RegistryStatistics::new();
//         assert_eq!(stats.await.total_primals, 0)
//         assert_eq!(stats.await.healthy_primals, 0)
//         assert_eq!(stats.health_percentage(), 0.0);
//         assert!(!stats.is_healthy());
//         Ok(()),
//     }
//
//     #[test]
//     fn test_health_percentage_calculation() {
//         let mut stats = RegistryStatistics::new();
//         stats.total_primals = 10;
//         stats.healthy_primals = 8;
//
//         assert_eq!(stats.health_percentage(), 80.0);
//         assert!(stats.is_healthy());
//
//         Ok(()),
//     }
//
//     #[test]
//     fn test_enhanced_statistics_health_score() {
//         let mut stats = EnhancedRegistryStatistics::new();
//         stats.basic.total_primals = 10;
//         stats.basic.healthy_primals = 9;
//         stats.performance_metrics.overall_error_rate = 0.01;
//
//         stats.calculate_health_score();
//         assert!(stats.health_score > 0.5));
//         Ok(()),
//     }
//
//     #[tokio::test]
//     async fn test_statistics_calculator() -> Result<()>{
//         let registry = crate::universal_registry::memory_registry::MemoryServiceRegistry::new();
//         let calculator = StatisticsCalculator::new();
//
//         // Skip this test as it requires a full UniversalPrimalRegistry
//         // let stats = calculator.calculate_statistics(&registry).await;
//         // assert_eq!(stats.await.total_primals, 0)
//
//         // Skip this test as it requires a full UniversalPrimalRegistry
//         // let enhanced_stats = calculator.calculate_enhanced_statistics(&registry).await;
//         // assert_eq!(enhanced_stats.basic.total_primals, 0)
//         Ok(()),
//     }
//
//     #[test]
//     fn test_load_distribution_calculation()  {//         let load_metrics = LoadMetrics  {//             current_requests: 50,
//             total_requests: 100,
//             avg_response_time_ms: 200.0,
//             error_count: 5,
//             last_updated: chrono::Utc::now(,
//         };
//         let load_dist = StatisticsCalculator::calculate_load_distribution(&load_metrics);
//
//         assert_eq!(load_dist.average_load, 0.5) // 50/100 = 0.5
//         assert_eq!(load_dist.max_load, 1.0)
//
//         Ok(()),
//     }
// }
