//! Primal ecosystem type definitions
//!
//! **CANONICAL MODULE**: Single source of truth for primal configuration types
//!
//! This module consolidates primal-related types from:
//! - `config/agnostic_primals.rs` (archived - experimental, unused)
//! - `config/universal_primals.rs` (archived - 1 type extracted: `QosMetrics`)
//!
//! The simpler canonical approach proved more practical than the experimental
//! universal registry systems.

use serde::{Deserialize, Serialize};
use std::time::Duration;

// Removed unused SongbirdResult import
/// **CANONICAL**: Primal type classification in the ecosystem
///
/// Unified from multiple definitions across:
/// - `songbird-universal/src/adapters/types.rs`
/// - `songbird-universal-primals/src/types.rs`
/// - Various other locations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PrimalType {
    /// Compute and container orchestration providers
    Compute,
    /// Storage and file management providers
    Storage,
    /// Security and authentication providers
    Security,
    /// AI and machine learning providers
    AI,
    /// Songbird - Network orchestration and service mesh
    Orchestration,
    /// Gaming-specific primals
    Gaming,
    /// Communication and messaging providers
    Communication,
    /// Media processing and streaming providers
    Media,
    /// Database and data management providers
    Database,
    /// Analytics and monitoring providers
    Analytics,
    /// Development and CI/CD providers
    Development,
    /// `IoT` and edge computing providers
    IoT,
    /// Blockchain and distributed ledger providers
    Blockchain,
    /// Financial and payment processing providers
    Financial,
    /// Identity and access management providers
    Identity,
    /// Content delivery and CDN providers
    Cdn,
    /// Email and notification providers
    Email,
    /// Search and indexing providers
    Search,
    /// Backup and disaster recovery providers
    Backup,
    /// Compliance and governance providers
    Compliance,
    /// Custom or third-party primal types
    Custom(String),
    /// Unknown or unclassified primal type
    #[default]
    Unknown,
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compute => write!(f, "compute"),
            Self::Storage => write!(f, "storage"),
            Self::Security => write!(f, "security"),
            Self::AI => write!(f, "ai"),
            Self::Orchestration => write!(f, "orchestration"),
            Self::Gaming => write!(f, "gaming"),
            Self::Communication => write!(f, "communication"),
            Self::Media => write!(f, "media"),
            Self::Database => write!(f, "database"),
            Self::Analytics => write!(f, "analytics"),
            Self::Development => write!(f, "development"),
            Self::IoT => write!(f, "iot"),
            Self::Blockchain => write!(f, "blockchain"),
            Self::Financial => write!(f, "financial"),
            Self::Identity => write!(f, "identity"),
            Self::Cdn => write!(f, "cdn"),
            Self::Email => write!(f, "email"),
            Self::Search => write!(f, "search"),
            Self::Backup => write!(f, "backup"),
            Self::Compliance => write!(f, "compliance"),
            Self::Custom(name) => write!(f, "custom-{name}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for PrimalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compute" => Ok(Self::Compute),
            "storage" => Ok(Self::Storage),
            "security" => Ok(Self::Security),
            "ai" => Ok(Self::AI),
            "orchestration" => Ok(Self::Orchestration),
            "gaming" => Ok(Self::Gaming),
            "communication" => Ok(Self::Communication),
            "media" => Ok(Self::Media),
            "database" => Ok(Self::Database),
            "analytics" => Ok(Self::Analytics),
            "development" => Ok(Self::Development),
            "iot" => Ok(Self::IoT),
            "blockchain" => Ok(Self::Blockchain),
            "financial" => Ok(Self::Financial),
            "identity" => Ok(Self::Identity),
            "cdn" => Ok(Self::Cdn),
            "email" => Ok(Self::Email),
            "search" => Ok(Self::Search),
            "backup" => Ok(Self::Backup),
            "compliance" => Ok(Self::Compliance),
            "unknown" => Ok(Self::Unknown),
            custom if custom.starts_with("custom-") => {
                let custom_name = custom
                    .strip_prefix("custom-")
                    .unwrap_or(custom) // Safe fallback - if prefix removal fails, use original
                    .to_string();
                Ok(Self::Custom(custom_name))
            }
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
}

/// **CANONICAL**: Service category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServiceCategory {
    /// Core infrastructure services
    Infrastructure,
    /// Application-level services
    #[default]
    Application,
    /// Data processing services
    Data,
    /// User interface services
    UI,
    /// Integration and middleware services
    Integration,
    /// Monitoring and observability services
    Monitoring,
    /// Security and compliance services
    Security,
    /// Development and testing services
    Development,
    /// Analytics and reporting services
    Analytics,
    /// Communication services
    Communication,
    /// Custom service category
    Custom(String),
}

impl std::fmt::Display for ServiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infrastructure => write!(f, "infrastructure"),
            Self::Application => write!(f, "application"),
            Self::Data => write!(f, "data"),
            Self::UI => write!(f, "ui"),
            Self::Integration => write!(f, "integration"),
            Self::Monitoring => write!(f, "monitoring"),
            Self::Security => write!(f, "security"),
            Self::Development => write!(f, "development"),
            Self::Analytics => write!(f, "analytics"),
            Self::Communication => write!(f, "communication"),
            Self::Custom(name) => write!(f, "custom-{name}"),
        }
    }
}

/// **CANONICAL**: Quality of service metrics for capabilities
///
/// Extracted from `config/universal_primals.rs` (was the only actively used type)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosMetrics {
    /// Expected latency in milliseconds
    pub latency_ms: Option<f64>,

    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,

    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,

    /// Reliability score (0.0 to 1.0)
    pub reliability: Option<f64>,
}

/// **CANONICAL**: Connection settings for primal communication
///
/// Simplified from experimental `universal_primals.rs` patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    /// Connection timeout
    pub connection_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Keep-alive enabled
    pub keep_alive: bool,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            max_retries: 3,
            keep_alive: true,
        }
    }
}

/// **CANONICAL**: Health check configuration
///
/// Simplified from experimental patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
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

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            endpoint_path: "/health".to_string(),
            expected_status_codes: vec![200],
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
        }
    }
}

// ============================================================================
// PHASE 4: PRIMAL REGISTRY CONSOLIDATION (November 8, 2025)
// ============================================================================
// Consolidated from config/universal_primals.rs - simplified for production use

use std::collections::HashMap;

/// **CANONICAL**: Universal primal registry for dynamic primal management
///
/// Simplified registry system consolidating experimental patterns from:
/// - `config/universal_primals.rs` (archived)
/// - `unified/primals.rs` (archived)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrimalRegistry {
    /// Registered primals by their type identifier
    pub primals: HashMap<String, PrimalConfiguration>,
}

impl PrimalRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            primals: HashMap::new(),
        }
    }

    /// Register a primal configuration
    pub fn register_primal(&mut self, config: PrimalConfiguration) {
        self.primals.insert(config.primal_type.clone(), config);
    }

    /// Get a primal configuration by type
    #[must_use]
    pub fn get_primal(&self, primal_type: &str) -> Option<&PrimalConfiguration> {
        self.primals.get(primal_type)
    }

    /// Get all enabled primals
    #[must_use]
    pub fn get_enabled_primals(&self) -> Vec<&PrimalConfiguration> {
        self.primals.values().filter(|p| p.enabled).collect()
    }

    /// Check if a primal is registered
    #[must_use]
    pub fn is_registered(&self, primal_type: &str) -> bool {
        self.primals.contains_key(primal_type)
    }
}

/// **CANONICAL**: Universal configuration for any primal type
///
/// Simplified from experimental `universal_primals` patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration {
    /// Primal type identifier (e.g., "beardog", "nestgate", "toadstool", "squirrel")
    pub primal_type: String,

    /// Human-readable name
    pub display_name: String,

    /// Whether this primal is enabled
    pub enabled: bool,

    /// Primary endpoint configuration
    pub endpoint: PrimalEndpoint,

    /// Declared capabilities of this primal
    pub capabilities: Vec<PrimalCapability>,

    /// Connection and timeout settings
    pub connection_settings: ConnectionSettings,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl PrimalConfiguration {
    /// Create a new primal configuration template
    #[must_use]
    pub fn new_template(primal_type: &str, display_name: &str) -> Self {
        Self {
            primal_type: primal_type.to_string(),
            display_name: display_name.to_string(),
            enabled: false,
            endpoint: PrimalEndpoint::default(),
            capabilities: Vec::new(),
            connection_settings: ConnectionSettings::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// **CANONICAL**: Primal capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapability {
    /// Capability identifier (e.g., "security", "storage", "compute", "gaming")
    pub capability_type: String,

    /// Capability version
    pub version: String,

    /// Capability-specific parameters
    pub parameters: std::collections::HashMap<String, serde_json::Value>,

    /// Quality of service metrics
    pub qos_metrics: QosMetrics,
}

/// **CANONICAL**: Universal primal endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Primary URL for this primal
    pub primary_url: String,

    /// Whether to use TLS
    pub use_tls: bool,
}

impl Default for PrimalEndpoint {
    fn default() -> Self {
        Self {
            primary_url: String::new(),
            use_tls: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdResult;

    #[test]
    fn test_primal_type_parsing() -> SongbirdResult<()> {
        assert_eq!("compute".parse::<PrimalType>()?, PrimalType::Compute);
        assert_eq!("AI".parse::<PrimalType>()?, PrimalType::AI);
        assert_eq!("custom-test".parse::<PrimalType>()?, PrimalType::Custom("test".to_string()));
        Ok(())
    }

    #[test]
    fn test_primal_type_display() {
        assert_eq!(PrimalType::Compute.to_string(), "compute");
        assert_eq!(PrimalType::Gaming.to_string(), "gaming");
        assert_eq!(PrimalType::Custom("test".to_string()).to_string(), "custom-test");
    }

    #[test]
    fn test_service_category_display() {
        assert_eq!(ServiceCategory::Infrastructure.to_string(), "infrastructure");
        assert_eq!(ServiceCategory::Application.to_string(), "application");
        assert_eq!(ServiceCategory::Custom("test".to_string()).to_string(), "custom-test");
    }

    #[test]
    fn test_defaults() {
        assert_eq!(PrimalType::default(), PrimalType::Unknown);
        assert_eq!(ServiceCategory::default(), ServiceCategory::Application);
    }
}
