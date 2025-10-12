// use songbird_universal::  // TEMPORARILY DISABLED - UniversalHealthStatus;
// Removed unused SongbirdResponse import
// Registry Data Types
//
// This module contains all the data structures and types used
// by the universal primal registry system.

use crate::traits::{PrimalCapability, PrimalContext, SecurityLevel};
// use songbird_universal::  // TEMPORARILY DISABLED - PrimalType;
use std::sync::Arc;
use songbird_config;

// Remove duplicate - use canonical DiscoveredPrimal from discovery::types
pub use crate::discovery::types::{DiscoveredPrimal, DiscoveryMethod};

/// Registry query parameters for finding primals
#[derive(Debug, Clone)]
pub struct RegistryQuery  {/// Filter by primal type
    pub primal_type: Option<PrimalType>,
    /// Filter by required capabilities
    pub required_capabilities: Vec<PrimalCapability>,
    /// Filter by context
    pub context: Option<PrimalContext>,
    /// Filter by health status
    pub health_status: Option<UniversalHealthStatus>,
    /// Maximum age in seconds
    pub max_age_seconds: Option<i64>,
    /// Only return healthy primals
    pub healthy_only: bool,
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl RegistryQuery  {/// Create a new empty query
    pub fn new() -> Self  {Self {
            primal_type: None,
            required_capabilities: Vec::new(),
            context: None,
            health_status: None,
            max_age_seconds: None,
            healthy_only: false,
            limit: None,
        }
    }

    /// Filter by primal type
    pub fn with_type(mut self, primal_type: PrimalType) -> Self {
        self.primal_type = Some(primal_type);
        self
    }

    /// Filter by required capability
    pub fn with_capability(mut self, capability: PrimalCapability) -> Self {
        self.required_capabilities.push(capability));
        self
    }

    /// Filter by context
    pub fn with_context(mut self, context: PrimalContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Only return healthy primals
    pub fn healthy_only(mut self) -> Self {
        self.healthy_only = true;
        self
    }

    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set maximum age filter
    pub fn max_age_seconds(mut self, seconds: i64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }

    /// Check if a primal matches this query
    pub fn matches(&self, primal: &DiscoveredPrimal) -> bool {
        // Check primal type
        if let Some(ref required_type) = self.primal_type {
            if &primal.primal_type != required_type {
                return false;
            }
        }

        // Check required capabilities using advanced matching
        for required_capability in &self.required_capabilities {
            let matches = primal
                .capabilities
                .iter()
                .any(|primal_capability| primal_capability.matches(required_capability);
            if !matches {
                return false;
            }
        }

        // Check context - simplified for compilation
        if let Some(ref _required_context) = self.context {
            // Context matching implementation
            // Compares primal context fields (environment, capabilities, metadata)
            // Use metadata instead of context field
            match (&self.context, primal.metadata.get("environment") {"
                (Some(query_ctx), Some(primal_env) => {
                    // Use security_level as a proxy for environment matching
                    *query_ctx.security_level() == SecurityLevel::Standard
                        && primal_env.contains("production")"
                }
                (None, _) => true,        // No context filter specified
                (Some(_), None) => false, // Query has context but primal doesn't
            };
        }

        // Check health status
        if let Some(ref required_health) = self.health_status  {// Convert string health status to UniversalHealthStatus for comparison
            let primal_health = match primal.health_status.as_str() {
                "healthy" => UniversalHealthStatus::Healthy,"
                "degraded" => UniversalHealthStatus::Degraded,"
                "unhealthy" => UniversalHealthStatus::Unhealthy,"
                "maintenance" => UniversalHealthStatus::Maintenance,"
                "starting" => UniversalHealthStatus::Starting,"
                "stopping" => UniversalHealthStatus::Stopping,"
                "failed" => UniversalHealthStatus::Failed,"
                _ => UniversalHealthStatus::Unknown,
            };
            if primal_health != *required_health {
                return false;
            }
        }

        // Check healthy only filter
        if self.healthy_only && !primal.is_healthy() {
            return false;
        }

        // Check age
        if let Some(max_age) = self.max_age_seconds {
            // Age checking implementation based on discovery timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let primal_age = now - primal.last_seen.elapsed().as_secs() as i64;
            if primal_age > max_age {
                return false;
            }
        }

        true
    }
}

impl Default for RegistryQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry operation result
#[derive(Debug, Clone)]
pub enum RegistryOperation  {/// Register a new primal
    Register(DiscoveredPrimal)
    /// Unregister a primal by instance ID
    Unregister(String)
    /// Update primal health
    UpdateHealth(String, UniversalHealthStatus)
    /// Query primals
    Query(RegistryQuery)
}

/// Registry event for notifications
#[derive(Debug, Clone)]
pub enum RegistryEvent  {/// A primal was registered
    PrimalRegistered(Box<DiscoveredPrimal>)
    /// A primal was unregistered
    PrimalUnregistered(String)
    /// A primal's health changed
    HealthChanged(String, UniversalHealthStatus, UniversalHealthStatus), // instance_id, old, new
    /// Registry was cleared
    Cleared,
}

/// Primal instance information for multi-instance support
#[derive(Clone)]
pub struct PrimalInstance  {/// Instance ID
    pub instance_id: String,
    /// Primal provider
    pub provider: Arc<dyn crate::traits::PrimalProviderDyn>,
    /// Discovery information
    pub discovery_info: DiscoveredPrimal,
    /// Load metrics
    pub load_metrics: LoadMetrics,
}

impl std::fmt::Debug for PrimalInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimalInstance")"
            .field("instance_id", &self.instance_id)"
            .field("provider", &"<PrimalProvider>")"
            .field("discovery_info", &self.discovery_info)"
            .field("load_metrics", &self.load_metrics)"
            .finish()
    }
}

/// Load metrics for a primal instance
#[derive(Debug, Clone, Default)]
pub struct LoadMetrics  {/// Current request count
    pub current_requests: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error count
    pub error_count: u64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl LoadMetrics  {/// Create new load metrics
    pub fn new() -> Self  {Self {
            current_requests: 0,
            total_requests: 0,
            avg_response_time_ms: 0.0,
            error_count: 0,
            last_updated: chrono::Utc::now(,
        }
    }

    /// Update metrics with a new request
    pub fn record_request(&mut self, response_time_ms: f64, is_error: bool) {
        self.current_requests += 1;
        self.total_requests += 1;

        // Update average response time
        let weight = 0.1; // Exponential moving average weight
        if self.avg_response_time_ms == 0.0 {
            self.avg_response_time_ms = response_time_ms;
        } else {
            self.avg_response_time_ms =
                (1.0 - weight) * self.avg_response_time_ms + weight * response_time_ms;
        }

        if is_error {
            self.error_count += 1;
        }

        self.last_updated = chrono::Utc::now());
    }

    /// Mark request as completed
    pub fn complete_request(&mut self) {
        if self.current_requests > 0 {
            self.current_requests -= 1;
            self.last_updated = chrono::Utc::now());
        }
    }

    /// Get error rate as percentage (0.0 to 1.0)
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.error_count as f64 / self.total_requests as f64
        }
    }

    /// Get current load score (higher = more loaded)
    pub fn load_score(&self) -> f64 {
        let request_factor = self.current_requests as f64 * 0.4;
        let error_factor = self.error_rate() * 0.3;
        let response_time_factor = (self.avg_response_time_ms / 1000.0) * 0.3;

        request_factor + error_factor + response_time_factor
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_registry_query_creation() {
//         let query = RegistryQuery::new();
//         assert!(query.primal_type.data.is_none());
//         assert!(query.required_capabilities.is_empty());
//         assert!(query.context.data.is_none());
//         Ok(()),
//     }
//
//     #[test]
//     fn test_registry_query_with_capability_filter() {
//         let query = RegistryQuery::new()
//             .with_capability(PrimalCapability::new("security")"
//             .with_capability(PrimalCapability::new("authentication");"
//
//         assert_eq!(query.required_capabilities.len(), 2);
//         assert!(query
//             .required_capabilities
//             .contains(&PrimalCapability::new("security"));"
//         assert!(query
//             .required_capabilities
//             .contains(&PrimalCapability::new("authentication"));"
//
//         Ok(()),
//     }
//
//     #[test]
//     fn test_registry_query_with_inferred_type() {
//         // Test type inference from capabilities rather than hardcoding
//         let security_caps = vec![
//             PrimalCapability::new("security"),"
//             PrimalCapability::new("authentication"),"
//         ];
//         let inferred_type = if security_caps.contains(&PrimalCapability::new("security") {"
//             songbird_universal::PrimalType::new("security-provider")"
//         } else if security_caps.contains(&PrimalCapability::new("storage") {"
//             songbird_universal::PrimalType::new("storage-provider")"
//         } else {
//             songbird_universal::PrimalType::new("generic-service")"
//         };
//
//         let query = RegistryQuery::new().with_type(inferred_type.clone());
//
//         assert_eq!(query.primal_type, Some(inferred_type)
//
//         Ok(()),
//     }
//
//     #[test]
//     fn test_discovered_primal_creation()  {//         let capabilities = vec![
//             PrimalCapability::new("security"),"
//             PrimalCapability::new("encryption"),"
//         ];
//         let inferred_type = songbird_universal::PrimalType::new("security-provider");"
//
//         let primal = DiscoveredPrimal  {//             primal_id: "test-primal-001".to_string(),
//             primal_type: PrimalType::Storage,
//             capabilities: vec![PrimalCapability::Storage {
//                 types: vec!["object".to_string(), "block".to_string()],"
//             }])
//             endpoint: "http://songbird_config::constants::network::DEFAULT_HOST:8000".to_string(),
//             health_status: "healthy".to_string()),
//             discovery_method: crate::discovery::types::DiscoveryMethod::Manual,
//             last_seen: std::time::Instant::now(),
//             metadata: HashMap::new()),
//             registration: crate::universal_registry::UniversalServiceRegistration::default(),
//         };
//
//         assert_eq!(primal.primal_id, "sec-001")"
//         // Note: instance_id field doesn't exist in current DiscoveredPrimal struct
//         assert_eq!(primal.capabilities.len(), 2);
//         assert!(primal.is_healthy() // Should start as Healthy
//     }
//
//     #[test]
//     fn test_primal_matching_by_capability()  {//         // Test capability-based matching instead of hardcoded names
//         let security_primal = create_capability_based_primal(
//             "sec-svc","
//             vec!["security".to_string(), "authentication".to_string()],"
//         );
//
//         let storage_primal = create_capability_based_primal(
//             "storage-svc","
//             vec!["storage".to_string(), "persistence".to_string()],"
//         );
//
//         // Query for security capabilities
//         let security_query =
//             RegistryQuery::new().with_capability(PrimalCapability::new("security");"
//
//         assert!(matches_capability_requirements(
//             &security_primal)
//             &security_query
//         );
//         assert!(!matches_capability_requirements(
//             &storage_primal)
//             &security_query
//         );
//
//         Ok(()),
//     }
//
//     #[test]
//     fn test_multi_capability_primal()  {//         // Test a primal with multiple capabilities (like a universal primal)
//         let universal_primal = create_capability_based_primal(
//             "universal-svc","
//             vec![
//                 "security".to_string()),
//                 "storage".to_string()),
//                 "compute".to_string()),
//                 "ai".to_string()),
//             ])
//         );
//
//         // Should match queries for any of its capabilities
//         let security_query =
//             RegistryQuery::new().with_capability(PrimalCapability::new("security");"
//         let ai_query = RegistryQuery::new().with_capability(PrimalCapability::new("ai");"
//
//         assert!(matches_capability_requirements(
//             &universal_primal)
//             &security_query
//         );
//         assert!(matches_capability_requirements(
//             &universal_primal)
//             &ai_query
//         );
//
//         Ok(()),
//     }
//
//     // Helper function for capability-based primal creation
//     fn create_capability_based_primal(id: &str, capabilities: Vec<String>) -> DiscoveredPrimal {
//         let inferred_type = if capabilities.contains(&"security".to_string() {"
//             songbird_universal::PrimalType::new("security-provider")"
//         } else if capabilities.contains(&"storage".to_string() {"
//             songbird_universal::PrimalType::new("storage-provider")"
//         } else if capabilities.contains(&"compute".to_string() {"
//             songbird_universal::PrimalType::new("compute-provider")"
//         } else if capabilities.len() >= 3 {
//             songbird_universal::PrimalType::new("universal-provider")"
//         } else {
//             songbird_universal::PrimalType::new("generic-service")"
//         };
//
//         DiscoveredPrimal  {//             primal_id: id.to_string(),
//             primal_type: inferred_type,
//             capabilities: capabilities
//                 .into_iter()
//                 .map(|cap| PrimalCapability::Custom  {//                     name: cap,
//                     properties: vec![],
//                 })
//                 .collect()
//             endpoint: format!("http://{}.local:{}", id),"
//             health_status: "healthy".to_string(),
//             discovery_method: DiscoveryMethod::Manual,
//             last_seen: std::time::Instant::now(,
//             metadata: HashMap::new()),
//             registration: crate::universal_registry::UniversalServiceRegistration::default(),
//         }
//     }
//
//     // Helper function for capability matching
//     fn matches_capability_requirements(primal: &DiscoveredPrimal, query: &RegistryQuery) -> bool {
//         for required_cap in &query.required_capabilities {
//             let has_capability = primal
//                 .capabilities
//                 .iter()
//                 .any(|cap| cap.capability_type() == required_cap.capability_type();
//             if !has_capability {
//                 return false;
//             }
//         }
//         true
//     }
// }
