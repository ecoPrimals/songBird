// Discovery Summary and Reporting
//
// This module handles summarizing discovery results and providing
// statistics and reporting functionality for the discovery process.

use crate::discovery::types::{DiscoveredPrimal, DiscoveryMethod};
use crate::{traits::PrimalCapability, traits::PrimalType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Add chrono for timestamp handling
use chrono;
/// Statistics for discovery operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryStats {
    pub total_attempts: u64,
    pub successful_discoveries: u64,
    pub failed_attempts: u64,
    pub average_discovery_time_ms: f64,
    pub capability_counts: HashMap<String, u64>,
    pub method_counts: HashMap<String, u64>,
}
impl DiscoveryStats {
    /// Increment capability counts for discovered primal
    pub fn increment_primal_capabilities(&mut self, capabilities: &[PrimalCapability]) {
        for capability in capabilities {
            let cap_type = capability.capability_type();
            *self.capability_counts.entry(cap_type).or_insert(0) += 1;
        }
    }
    /// Decrement capability counts when primal is removed
    pub fn decrement_primal_capabilities(&mut self, capabilities: &[PrimalCapability]) {
        for capability in capabilities {
            let cap_type = capability.capability_type();
            if let Some(count) = self.capability_counts.get_mut(&cap_type) {
                if *count > 0 {
                    *count -= 1;
                }
            }
        }
    }
}
/// Summary of discovery operations and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySummary {
    /// Total number of primals discovered
    pub total_primals_discovered: u64,
    /// Number of primals discovered by each method
    pub primals_by_method: HashMap<String, u64>,
    /// Number of primals discovered by each type
    pub primals_by_type: HashMap<String, u64>,
    /// Total time spent on discovery (in milliseconds)
    pub total_discovery_time_ms: u64,
    /// Success rate of discovery attempts (0.0 to 1.0)
    pub discovery_success_rate: f64,
    /// Total number of discovery attempts made
    pub total_attempts: u64,
    /// Successful discovery attempts
    pub successful_attempts: u64,
    /// Average time per discovery attempt (in milliseconds)
    pub average_attempt_time_ms: f64,
    /// Discovery configuration used
    pub discovery_config_summary: String,
    /// Timestamp when summary was generated
    pub generated_at: String,
}
impl DiscoverySummary {
    /// Create a new empty discovery summary
    pub fn new() -> Self {
        Self {
            total_primals_discovered: 0,
            primals_by_method: HashMap::new(),
            primals_by_type: HashMap::new(),
            total_discovery_time_ms: 0,
            discovery_success_rate: 0.0,
            total_attempts: 0,
            successful_attempts: 0,
            average_attempt_time_ms: 0.0,
            discovery_config_summary: "Default".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a summary from discovered primals and statistics
    pub fn from_discovered_primals(
        primals: &HashMap<String, DiscoveredPrimal>,
        stats: &DiscoveryStats,
        config_summary: String,
    ) -> Self {
        let mut primals_by_method = HashMap::new();
        let mut primals_by_type = HashMap::new();
        // Count primals by discovery method and type
        for primal in primals.values() {
            let method_name = match primal.discovery_method {
                DiscoveryMethod::NetworkScan => "network_scan",
                DiscoveryMethod::ConfigBased => "config_based",
                DiscoveryMethod::Filesystem => "filesystem",
                DiscoveryMethod::ServiceRegistry => "service_registry",
                DiscoveryMethod::Broadcast => "broadcast",
                DiscoveryMethod::Federation => "federation",
                DiscoveryMethod::Manual => "manual",
                DiscoveryMethod::Mdns => "mdns",
                _ => "other",
            };
            *primals_by_method
                .entry(method_name.to_string())
                .or_insert(0) += 1;
            let type_name = primal.primal_type.to_string();
            *primals_by_type.entry(type_name).or_insert(0) += 1;
        }

        // Calculate metrics from stats
        let total_attempts = stats.total_attempts;
        let successful_attempts = stats.successful_discoveries;
        let success_rate = if total_attempts > 0 {
            successful_attempts as f64 / total_attempts as f64
        } else {
            0.0
        };
        let average_time = if total_attempts > 0 {
            stats.average_discovery_time_ms
        } else {
            0.0
        };

        Self {
            total_primals_discovered: primals.len() as u64,
            primals_by_method,
            primals_by_type,
            total_discovery_time_ms: stats.average_discovery_time_ms as u64,
            discovery_success_rate: success_rate,
            total_attempts,
            successful_attempts,
            average_attempt_time_ms: average_time,
            discovery_config_summary: config_summary,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create example discovery summary with capability-based primals
    pub fn create_capability_example() -> DiscoverySummary {
        let _security_capabilities = vec![
            // Modern structured capability definitions
            PrimalCapability::security(),
            PrimalCapability::authentication(),
            PrimalCapability::encryption(),
        ];
        let storage_capabilities = vec![
            PrimalCapability::storage(),
            PrimalCapability::Storage {
                types: vec!["file".to_string()],
            }, // persistence -> file storage
            PrimalCapability::database(),
        ];
        let _compute_capabilities = vec![
            PrimalCapability::compute(),
            PrimalCapability::Compute {
                types: vec!["container".to_string()],
            }, // containers -> container compute
            PrimalCapability::Compute {
                types: vec!["serverless".to_string()],
            }, // serverless -> serverless compute
        ];
        let discovered_primals = vec![
            DiscoveredPrimal {
                primal_id: "security-provider-001".to_string(),
                primal_type: PrimalType::Security,
                capabilities: vec![
                    PrimalCapability::Security {
                        protocols: vec!["TLS".to_string()],
                    },
                    PrimalCapability::Authentication {
                        methods: vec!["JWT".to_string()],
                    },
                ],
                endpoint: "https://security-service.local:8443".to_string(),
                health_status: "healthy".to_string(),
                discovery_method: DiscoveryMethod::ConfigBased,
                last_seen: std::time::Instant::now(),
                metadata: std::collections::HashMap::from([
                    ("service_type".to_string(), "security".to_string()),
                    (
                        "discovery_method".to_string(),
                        "capability_based".to_string(),
                    ),
                ]),
                registration: crate::universal_registry::UniversalServiceRegistration::default(),
            },
            DiscoveredPrimal {
                primal_id: "storage-provider-001".to_string(),
                primal_type: PrimalType::Storage,
                capabilities: storage_capabilities,
                endpoint: "http://storage-service.local:9000".to_string(),
                health_status: "healthy".to_string(),
                discovery_method: DiscoveryMethod::NetworkScan,
                last_seen: std::time::Instant::now(),
                metadata: std::collections::HashMap::from([
                    ("service_type".to_string(), "storage".to_string()),
                    ("port_discovered".to_string(), "9000".to_string()),
                ]),
                registration: crate::universal_registry::UniversalServiceRegistration::default(),
            },
        ];

        DiscoverySummary {
            total_primals_discovered: discovered_primals.len() as u64,
            primals_by_method: HashMap::new(), // Simplified for example
            primals_by_type: HashMap::new(),   // Simplified for example
            total_discovery_time_ms: std::time::Duration::from_secs(5).as_millis() as u64,
            discovery_success_rate: 1.0,
            total_attempts: discovered_primals.len() as u64,
            successful_attempts: discovered_primals.len() as u64,
            average_attempt_time_ms: 100.0,
            discovery_config_summary: "Example Config".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Convert summary to JSON string
    pub fn to_json(&self) -> Result<String, songbird_errors::SongbirdError> {
        serde_json::to_string_pretty(self).map_err(|e| songbird_errors::SongbirdError::Internal {
            component: Some("discovery_summary".to_string()),
            message: format!("Failed to serialize discovery summary to JSON: {e}"),
            error_code: None,
            debug_info: None,
        })
    }

    /// Convert summary to formatted text report
    pub fn to_text_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Universal Primal Discovery Summary ===\n\n");
        report.push_str(&format!("Generated at: {}\n", self.generated_at));
        report.push_str(&format!(
            "Total primals discovered: {}\n",
            self.total_primals_discovered
        ));
        report.push_str(&format!(
            "Total discovery attempts: {}\n",
            self.total_attempts
        ));
        report.push_str(&format!(
            "Successful attempts: {}\n",
            self.successful_attempts
        ));
        report.push_str(&format!(
            "Success rate: {:.2}%\n",
            self.discovery_success_rate * 100.0
        ));
        report.push_str(&format!(
            "Total discovery time: {}ms\n",
            self.total_discovery_time_ms
        ));
        report.push_str(&format!(
            "Average time per attempt: {:.2}ms\n",
            self.average_attempt_time_ms
        ));
        report.push_str(&format!(
            "Discovery config: {}\n\n",
            self.discovery_config_summary
        ));

        if !self.primals_by_method.is_empty() {
            report.push_str("Primals by discovery method:\n");
            for (method, count) in &self.primals_by_method {
                report.push_str(&format!("  - {method}: {count}\n"));
            }
        }
        if !self.primals_by_type.is_empty() {
            report.push_str("Primals by type:\n");
            for (primal_type, count) in &self.primals_by_type {
                report.push_str(&format!("  - {primal_type}: {count}\n"));
            }
        }
        report.push_str("=== End Summary ===\n");
        report
    }

    /// Get performance metrics from the summary
    pub fn get_performance_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("success_rate".to_string(), self.discovery_success_rate);
        metrics.insert(
            "average_attempt_time_ms".to_string(),
            self.average_attempt_time_ms,
        );
        metrics.insert(
            "total_discovery_time_ms".to_string(),
            self.total_discovery_time_ms as f64,
        );
        metrics.insert("primals_per_minute".to_string(), {
            if self.total_discovery_time_ms > 0 {
                (self.total_primals_discovered as f64 * 60000.0)
                    / self.total_discovery_time_ms as f64
            } else {
                0.0
            }
        });
        metrics
    }

    /// Check if discovery performance meets minimum thresholds
    pub fn meets_performance_thresholds(
        &self,
        min_success_rate: f64,
        max_avg_time_ms: f64,
    ) -> bool {
        self.discovery_success_rate >= min_success_rate
            && self.average_attempt_time_ms <= max_avg_time_ms
    }
}

impl Default for DiscoverySummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovery performance analysis
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryPerformanceAnalysis {
    /// Overall performance rating (0.0 to 1.0)
    pub overall_rating: f64,
    /// Performance by discovery method
    pub method_performance: HashMap<String, f64>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Performance trends (if historical data available)
    pub trends: Option<HashMap<String, Vec<f64>>>,
}

impl DiscoveryPerformanceAnalysis {
    /// Analyze performance from multiple discovery summaries
    pub fn analyze_performance(summaries: &[DiscoverySummary]) -> Self {
        if summaries.is_empty() {
            return Self {
                overall_rating: 0.0,
                method_performance: HashMap::new(),
                recommendations: vec!["No discovery data available for analysis".to_string()],
                trends: None,
            };
        }

        let latest = match summaries.last() {
            Some(summary) => summary,
            None => {
                // This should never happen due to the is_empty() check above,
                // but we handle it gracefully anyway
                return Self {
                    overall_rating: 0.0,
                    method_performance: HashMap::new(),
                    recommendations: vec!["No discovery data available for analysis".to_string()],
                    trends: None,
                };
            }
        };
        let mut recommendations = Vec::new();
        let _method_performance: HashMap<String, f64> = HashMap::new();
        // Calculate overall rating based on latest summary
        let success_weight = 0.6;
        let speed_weight = 0.4;
        let speed_score = if latest.average_attempt_time_ms > 0.0 {
            1.0 / (1.0 + latest.average_attempt_time_ms / 1000.0).ln()
        } else {
            1.0
        };
        let overall_rating =
            (latest.discovery_success_rate * success_weight) + (speed_score * speed_weight);

        // Analyze method performance
        let mut method_performance = HashMap::new();
        for (method, count) in &latest.primals_by_method {
            let performance = (*count as f64) / (latest.total_primals_discovered as f64).max(1.0);
            method_performance.insert(method.clone(), performance);
        }

        // Generate recommendations
        if latest.discovery_success_rate < 0.8 {
            recommendations.push(
                "Consider enabling additional discovery methods to improve success rate"
                    .to_string(),
            );
        }

        if latest.average_attempt_time_ms > 5000.0 {
            recommendations.push("Discovery attempts are taking longer than optimal - consider optimizing network timeouts".to_string());
        }

        if latest.total_primals_discovered == 0 {
            recommendations.push(
                "No primals discovered - check network connectivity and configuration".to_string(),
            );
        }

        Self {
            overall_rating,
            method_performance,
            recommendations,
            trends: None, // Could be implemented with historical data
        }
    }
}

#[allow(dead_code)]
fn calculate_capability_distribution(primals: &Vec<DiscoveredPrimal>) -> HashMap<String, u64> {
    let mut distribution = HashMap::new();
    for primal in primals {
        for capability in &primal.capabilities {
            let capability_name = capability.capability_type();
            *distribution.entry(capability_name).or_insert(0u64) += 1;
        }
    }
    distribution
}

/// Calculate discovery method distribution
#[allow(dead_code)]
fn calculate_method_distribution(primals: &Vec<DiscoveredPrimal>) -> HashMap<String, u64> {
    let mut distribution = HashMap::new();
    for primal in primals {
        let method_name = match primal.discovery_method {
            DiscoveryMethod::NetworkScan => "network_scan",
            DiscoveryMethod::ConfigBased => "config_based",
            DiscoveryMethod::Filesystem => "filesystem",
            DiscoveryMethod::ServiceRegistry => "service_registry",
            DiscoveryMethod::Broadcast => "broadcast",
            DiscoveryMethod::Federation => "federation",
            DiscoveryMethod::Manual => "manual",
            DiscoveryMethod::Mdns => "mdns",
            _ => "other",
        };
        *distribution.entry(method_name.to_string()).or_insert(0) += 1;
    }
    distribution
}
// #[cfg(test)]
// mod tests {
//     use super::super::types::{DiscoveredPrimal, DiscoveryMethod};
//     use super::*;
//     #[test]
//     fn test_discovery_summary_creation() {
//         let summary = DiscoverySummary::new();
//         assert_eq!(summary.total_primals_discovered, 0);
//         assert!(summary.primals_by_method.is_empty());
//         assert!(summary.primals_by_type.is_empty());
//         assert_eq!(summary.discovery_success_rate, 0.0);
//         Ok(())
//     }
//
//     #[test]
//     fn test_discovery_summary_from_primals() {
//         let mut primals = HashMap::new();
//         let mut stats = DiscoveryStats::default();
//
//         // Add a mock primal
//         let test_primal = DiscoveredPrimal {
//             primal_id: "security-service-001".to_string(),
//             primal_type: PrimalType::Security,
//             capabilities: vec![
//                 PrimalCapability::Security {
//                     protocols: vec!["https".to_string()],
//                 },
//                 PrimalCapability::Authentication {
//                     methods: vec!["oauth2".to_string()],
//                 },
//             ],
//             endpoint: "https://security.local:8443".to_string(),
//             health_status: "healthy".to_string(),
//             discovery_method: DiscoveryMethod::ConfigBased,
//             last_seen: std::time::Instant::now(),
//             metadata: std::collections::HashMap::from([(
//                 "service_type".to_string(),
//                 "security".to_string(),
//             )]),
//         };
//
//         primals.insert("test-1".to_string(), test_primal);
//
//         // Set some stats
//         stats.total_attempts = 10;
//         stats.successful_discoveries = 8;
//         stats.average_discovery_time_ms = 500.0;
//
//         let summary =
//             DiscoverySummary::from_discovered_primals(&primals, &stats, "Test Config".to_string());
//
//         assert_eq!(summary.total_primals_discovered, 1);
//         assert_eq!(summary.total_attempts, 10);
//         assert_eq!(summary.successful_attempts, 8);
//         assert_eq!(summary.discovery_success_rate, 0.8);
//         assert_eq!(summary.average_attempt_time_ms, 500.0);
//         assert!(summary.primals_by_method.contains_key("config_based"));
//         assert!(summary.primals_by_type.contains_key("security-provider"));
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_summary_to_json() {
//         let summary = DiscoverySummary::new();
//         let json_result = summary.to_json();
//         assert!(json_result.is_ok());
//         let json_str = json_result.unwrap_or_else(|| {
//             tracing::error!("Operation failed");
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!("Operation failed - {}: {:?}", "unable to continue", e),
//             )
//             )?;
//         });
//         assert!(json_str.contains("total_primals_discovered"));
//         assert!(json_str.contains("generated_at"));
//         Ok(())
//     }
//
//     #[test]
//     fn test_summary_to_text_report() {
//         let summary = DiscoverySummary::new();
//         let report = summary.to_text_report();
//         assert!(report.contains("Universal Primal Discovery Summary"));
//         assert!(report.contains("Total primals discovered: 0"));
//         assert!(report.contains("Success rate"));
//         Ok(())
//     }
//
//     #[test]
//     fn test_performance_analysis() {
//         let summaries = vec![DiscoverySummary::new()];
//         let analysis = DiscoveryPerformanceAnalysis::analyze_performance(&summaries);
//         assert!(analysis.overall_rating >= 0.0);
//         assert!(analysis.overall_rating <= 1.0);
//         assert!(!analysis.recommendations.is_empty());
//         Ok(())
//     }
// }
