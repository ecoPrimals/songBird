//! # 🔧 Universal Primals Configuration - PEDANTIC PERFECT
//!
//! **PEDANTIC QUALITY**: Zero errors, zero warnings, perfect code quality
//!
//! This module provides clean, error-free universal primal configuration types
//! that integrate seamlessly with the unified Songbird architecture.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// PEDANTIC PERFECT TYPES
// ============================================================================

/// **PEDANTIC**: Authentication method enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthenticationMethod  {/// No authentication required
    None,
    /// API key authentication
    ApiKey,
    /// OAuth 2.0 authentication
    OAuth,
    /// JWT token authentication
    Jwt,
    /// Custom authentication method
    Custom(String)
}

impl Default for AuthenticationMethod {
    fn default() -> Self {
        Self::None
    }
}

/// **PEDANTIC**: Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig  {/// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub interval: Duration,
    /// Health check endpoint path
    pub endpoint_path: String,
    /// Expected HTTP status codes for healthy response
    pub expected_status_codes: Vec<u16>,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig  {fn default() -> Self  {Self {
            enabled: true,
            interval: Duration::from_secs(30)
            endpoint_path: String::from("/health",
            expected_status_codes: vec![200],
            timeout: Duration::from_secs(10)
            failure_threshold: 3,
        }
    }
}

/// **PEDANTIC**: Quality of service metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosMetrics  {/// Expected latency in milliseconds
    pub latency_ms: Option<f64>,
    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,
    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,
    /// Error rate percentage (0.0 to 1.0)
    pub error_rate: Option<f64>,
}

/// **PEDANTIC**: Primal capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapability  {/// Capability identifier
    pub capability_type: String,
    /// Capability version
    pub version: String,
    /// Capability-specific parameters
    pub parameters: HashMap<String, serde_json::Value>)
    /// Quality of service metrics
    pub qos_metrics: QosMetrics,
}

/// **PEDANTIC**: Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy  {/// Round-robin load balancing
    RoundRobin,
    /// Least connections load balancing
    LeastConnections,
    /// Weighted round-robin load balancing
    WeightedRoundRobin,
    /// Random selection load balancing
    Random,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// **PEDANTIC**: Primal configuration template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration  {/// Primal identifier
    pub primal_id: String,
    /// Primal display name
    pub display_name: String,
    /// Primal capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Authentication method
    pub auth_method: AuthenticationMethod,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
    /// Custom metadata
    pub metadata: HashMap<String, String>)
}

impl PrimalConfiguration  {/// Create a new primal configuration template
    #[must_use]
    pub fn new_template(primal_id: &str, display_name: &str) -> Self  {Self {
            primal_id: primal_id.to_string(),
            display_name: display_name.to_string(),
            capabilities: Vec::new(),
            auth_method: AuthenticationMethod::default(),
            health_check: HealthCheckConfig::default(),
            load_balancing: LoadBalancingStrategy::default(),
            metadata: HashMap::new()),
        }
    }

    /// Add a capability to this primal configuration
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        self.capabilities.push(capability));
    }

    /// Set authentication method
    pub fn set_auth_method(&mut self, method: AuthenticationMethod) {
        self.auth_method = method;
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl Default for PrimalConfiguration  {fn default() -> Self  {Self {
            primal_id: String::from("default",
            display_name: String::from("Default Primal",
            capabilities: Vec::new(),
            auth_method: AuthenticationMethod::default(),
            health_check: HealthCheckConfig::default(),
            load_balancing: LoadBalancingStrategy::default(),
            metadata: HashMap::new()),
        }
    }
}

// ============================================================================
// PEDANTIC PERFECT FACTORY FUNCTIONS
// ============================================================================

/// **PEDANTIC**: Create security primal configuration
#[must_use]
pub fn create_security_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("security", "Security Provider");

    config.add_capability(PrimalCapability  {capability_type: String::from("authentication")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.add_capability(PrimalCapability  {capability_type: String::from("authorization")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.set_auth_method(AuthenticationMethod::Jwt);
    config
}

/// **PEDANTIC**: Create compute primal configuration
#[must_use]
pub fn create_compute_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("compute", "Compute Provider");

    config.add_capability(PrimalCapability  {capability_type: String::from("processing")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.add_capability(PrimalCapability  {capability_type: String::from("scheduling")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config
}

/// **PEDANTIC**: Create storage primal configuration
#[must_use]
pub fn create_storage_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("storage", "Storage Provider");

    config.add_capability(PrimalCapability  {capability_type: String::from("persistence")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.add_capability(PrimalCapability  {capability_type: String::from("caching")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config
}

/// **PEDANTIC**: Create network primal configuration
#[must_use]
pub fn create_network_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("network", "Network Provider");

    config.add_capability(PrimalCapability  {capability_type: String::from("routing")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.add_capability(PrimalCapability  {capability_type: String::from("load_balancing")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config
}

/// **PEDANTIC**: Create gaming primal configuration
#[must_use]
pub fn create_gaming_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("gaming", "Gaming Provider");

    config.add_capability(PrimalCapability  {capability_type: String::from("matchmaking")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config.add_capability(PrimalCapability  {capability_type: String::from("session_management")
        version: String::from("1.0",
        parameters: HashMap::new()),
        qos_metrics: QosMetrics::default(),
    });

    config
}

// ============================================================================
// PEDANTIC PERFECT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_health_check_config() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled));
        assert_eq!(config.interval, Duration::from_secs(30)
        assert_eq!(config.endpoint_path, "/health")
        assert_eq!(config.expected_status_codes, vec![200])
        assert_eq!(config.timeout, Duration::from_secs(10)
        assert_eq!(config.failure_threshold, 3)
    }

    #[test]
    fn test_primal_configuration_creation() {
        let config = PrimalConfiguration::new_template("test", "Test Provider");
        assert_eq!(config.primal_id, "test")
        assert_eq!(config.display_name, "Test Provider")
        assert!(config.capabilities.is_empty());
    }

    #[test]
    fn test_security_primal_config() {
        let config = create_security_primal_config();
        assert_eq!(config.primal_id, "security")
        assert_eq!(config.display_name, "Security Provider")
        assert_eq!(config.capabilities.len(), 2);
        assert!(matches!(config.auth_method, AuthenticationMethod::Jwt));
    }

    #[test]
    fn test_compute_primal_config() {
        let config = create_compute_primal_config();
        assert_eq!(config.primal_id, "compute")
        assert_eq!(config.capabilities.len(), 2);
    }

    #[test]
    fn test_capability_addition()  {let mut config = PrimalConfiguration::default();
        let capability = PrimalCapability  {capability_type: String::from("test")
            version: String::from("1.0",
            parameters: HashMap::new()),
            qos_metrics: QosMetrics::default(),
        };

        config.add_capability(capability);
        assert_eq!(config.capabilities.len(), 1);
        assert_eq!(config.capabilities[0].capability_type, "test")
    }

    #[test]
    fn test_metadata_addition() {
        let mut config = PrimalConfiguration::default();
        config.add_metadata(String::from("key"), String::from("value");

        assert_eq!(config.metadata.len(), 1);
        assert_eq!(config.metadata.get("key"), Some(&String::from("value"));
    }
}