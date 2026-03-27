// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🍼 Zero-Touch Configuration System
//!
//! **MISSION**: Provide configuration with ZERO hardcoded knowledge
//!
//! This module implements the "infant discovery" philosophy where services start
//! with absolutely no hardcoded names, vendors, or ports. Everything is discovered
//! dynamically from the environment.
//!
//! ## Core Principles
//! 1. **No Hardcoded Primal Names**: Never reference beardog, toadstool, nestgate, squirrel
//! 2. **No Hardcoded Vendors**: Never reference kubernetes, consul, docker, redis, etc.
//! 3. **No Hardcoded Ports**: All ports come from environment or discovery
//! 4. **Capability-Based**: Request what you need (security, storage, compute, ai)
//! 5. **Dynamic Discovery**: Learn everything at runtime like an infant

#![allow(missing_docs, reason = "infant discovery structs are descriptive configuration bags")]

use crate::canonical::constants::read_process_env;

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tracing::{debug, info, warn};

/// **🍼 ZERO-TOUCH CONFIGURATION**
/// Contains NO hardcoded service names, primal names, or vendor references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTouchConfig {
    /// Self-identity (only thing a service knows about itself)
    pub self_identity: ServiceIdentity,

    /// Capability requirements (what this service needs, not who provides it)
    pub required_capabilities: Vec<CapabilityRequirement>,

    /// Optional capabilities (nice to have but not required)
    pub optional_capabilities: Vec<CapabilityRequirement>,

    /// Discovery configuration (how to find capability providers)
    pub discovery: DiscoveryConfig,

    /// Network configuration (ports from environment, not hardcoded)
    pub network: NetworkConfig,

    /// Bootstrap behavior
    pub bootstrap: BootstrapConfig,
}

/// Service's own identity - the ONLY thing it knows about itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentity {
    /// Service ID (from environment or generated)
    pub service_id: String,

    /// Capabilities this service PROVIDES (not what it consumes)
    pub provides_capabilities: Vec<String>,

    /// Service metadata from environment
    pub metadata: HashMap<String, String>,
}

/// Capability requirement (describes WHAT is needed, not WHO provides it)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Type of capability needed (e.g., "security", "storage", "compute", "ai")
    pub capability_type: String,

    /// Specific operations needed (e.g., `["encrypt", "decrypt"]` for security)
    pub required_operations: Vec<String>,

    /// Quality requirements
    pub quality_requirements: QualityRequirements,

    /// Fallback behavior if not available
    pub fallback_behavior: FallbackBehavior,
}

/// Quality requirements for capability providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms: Option<u64>,

    /// Minimum required availability (0.0 - 1.0)
    pub min_availability: Option<f64>,

    /// Required throughput (requests per second)
    pub min_throughput_rps: Option<f64>,

    /// Security requirements
    pub security_level: SecurityLevel,
}

/// Security level requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No specific security requirements
    None,
    /// Basic authentication required
    Basic,
    /// TLS/encryption required
    Encrypted,
    /// Mutual TLS and strong authentication
    StrongAuth,
    /// Maximum security with audit trails
    Maximum,
}

/// Fallback behavior when capability is unavailable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackBehavior {
    /// Fail immediately
    Fail,
    /// Retry with exponential backoff
    Retry {
        max_attempts: u32,
        backoff_ms: u64,
    },
    /// Use degraded mode
    DegradedMode {
        degraded_operations: Vec<String>,
    },
    /// Use local/mock implementation
    LocalFallback,
}

/// Discovery configuration for zero-touch infant discovery
///
/// **LOCAL DEFINITION**: Methods-based with caching support.
/// **ZERO-TOUCH**: Designed for automatic bootstrapping and infant mode.
/// Field mappings to canonical:
/// - `methods` (`Vec<Enum>`) → Can derive from enabled flags in canonical configs
/// - `timeout` (Duration) → `scan_timeout_secs` (u64, convert)
/// - `refresh_interval` (Duration) → `service_discovery.discovery_interval_secs` (u64, convert)
/// - `enable_cache` → `capability_discovery.enabled` (caching feature)
/// - `cache_ttl` (Duration) → `capability_discovery.cache_ttl_secs` (u64, convert)
///
/// **ARCHITECTURAL NOTE**: Methods-based pattern (`Vec<DiscoveryMethod>`) is an
/// alternative to flag-based. Both are valid approaches. Canonical uses nested
/// configs with individual enable flags for finer control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Discovery methods to try (in order of preference)
    /// Maps to: Multiple canonical flags based on method enum variants
    pub methods: Vec<DiscoveryMethod>,

    /// Discovery timeout (→ `scan_timeout_secs`)
    pub timeout: Duration,

    /// How often to refresh discovered services (→ `service_discovery.discovery_interval_secs`)
    pub refresh_interval: Duration,

    /// Cache discovered services (→ `capability_discovery.enabled` for caching)
    pub enable_cache: bool,

    /// Cache TTL (→ `capability_discovery.cache_ttl_secs`)
    pub cache_ttl: Duration,
}

/// Discovery method - describes HOW to discover, not WHAT vendor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Discover from environment variables
    /// Pattern: `CAPABILITY_<TYPE>_ENDPOINT`, e.g., `CAPABILITY_SECURITY_ENDPOINT`
    Environment {
        /// Variable name patterns to check
        patterns: Vec<String>,
    },

    /// Discover via HTTP service registry
    /// (Could be Consul, Eureka, or any HTTP-based registry)
    HttpRegistry {
        /// Registry endpoint from environment
        endpoint_env_var: String,
        /// API path pattern
        api_path: String,
    },

    /// Discover via DNS SRV records
    DnsSrv {
        /// DNS domain from environment
        domain_env_var: String,
    },

    /// Network scanning (for development)
    NetworkScan {
        /// IP ranges from environment
        ranges_env_var: String,
        /// Ports to probe from environment
        ports_env_var: String,
    },

    /// File-based configuration
    FileConfig {
        /// Config file path from environment
        path_env_var: String,
    },

    /// Container orchestration metadata
    /// (Could be Kubernetes, Docker Swarm, or any container system)
    ContainerMetadata {
        /// Metadata API endpoint from environment
        api_endpoint_env_var: String,
    },
}

/// Network configuration - NO hardcoded ports or addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address from environment (defaults to 127.0.0.1 for dev)
    pub bind_address: IpAddr,

    /// Service port from environment
    pub service_port: u16,

    /// Health check port from environment
    pub health_port: u16,

    /// Metrics port from environment  
    pub metrics_port: u16,

    /// Connection limits
    pub connection_limits: ConnectionLimits,

    /// Timeouts
    pub timeouts: NetworkTimeouts,
}

/// Connection limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_connections: u32,
    pub max_connections_per_ip: u32,
    pub connection_backlog: u32,
}

/// **SPECIALIZED**: Network timeouts for infant/zero-touch config
///
/// This is intentionally kept separate from `canonical::network::NetworkTimeouts` because:
/// 1. Different field names (`connection_timeout` vs connection)
/// 2. Different semantics (`idle_timeout` vs `health_check`)
/// 3. Zero-touch bootstrap has specific timeout requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub idle_timeout: Duration,
}

/// Bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// Enable infant discovery (zero-knowledge bootstrap)
    pub enable_infant_discovery: bool,

    /// Phases to complete during bootstrap
    pub discovery_phases: Vec<DiscoveryPhase>,

    /// Maximum time for bootstrap
    pub max_bootstrap_time: Duration,

    /// Fail if required capabilities not found
    pub fail_on_missing_required: bool,
}

/// Discovery phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryPhase {
    /// Scan environment variables
    EnvironmentScan,
    /// Probe network
    NetworkProbe,
    /// Query registries
    RegistryQuery,
    /// Test discovered capabilities
    CapabilityTest,
}

impl ZeroTouchConfig {
    /// Create zero-touch configuration from environment
    /// This is the ONLY entry point - no defaults with hardcoded values
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are not set or invalid
    pub fn from_environment() -> SongbirdResult<Self> {
        Self::from_environment_reader(read_process_env)
    }

    /// Same as [`from_environment`](Self::from_environment) with an injectable env reader.
    pub fn from_environment_reader(
        env: impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<Self> {
        info!("🍼 Creating zero-touch configuration from environment (no hardcoded knowledge)");

        let self_identity = Self::discover_self_identity(&env);
        let required_capabilities = Self::discover_required_capabilities(&env);
        let optional_capabilities = Self::discover_optional_capabilities(&env);
        let discovery = Self::create_discovery_config(&env);
        let network = Self::create_network_config(&env)?;
        let bootstrap = Self::create_bootstrap_config(&env);

        Ok(Self {
            self_identity,
            required_capabilities,
            optional_capabilities,
            discovery,
            network,
            bootstrap,
        })
    }

    /// Discover this service's own identity from environment
    fn discover_self_identity(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> ServiceIdentity {
        // Service ID from environment or generate one
        let service_id = env("SERVICE_ID")
            .or_else(|_| env("HOSTNAME"))
            .or_else(|_| env("POD_NAME"))
            .unwrap_or_else(|_| {
                let id = uuid::Uuid::new_v4().to_string();
                warn!("No SERVICE_ID found in environment, generated: {}", id);
                id
            });

        // Discover what capabilities THIS service provides
        let provides_capabilities = env("SERVICE_CAPABILITIES").map_or_else(
            |_| {
                debug!("No SERVICE_CAPABILITIES defined, service provides no capabilities");
                Vec::new()
            },
            |caps| caps.split(',').map(|s| s.trim().to_string()).collect(),
        );

        // Collect metadata from environment
        let metadata = Self::collect_service_metadata(env);

        info!("🔍 Discovered self identity: {} providing {:?}", service_id, provides_capabilities);

        ServiceIdentity {
            service_id,
            provides_capabilities,
            metadata,
        }
    }

    /// Discover required capabilities from environment
    fn discover_required_capabilities(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Vec<CapabilityRequirement> {
        let mut requirements = Vec::new();

        // Check for required capabilities in environment
        if let Ok(required) = env("REQUIRED_CAPABILITIES") {
            for cap_type in required.split(',') {
                let cap_type = cap_type.trim();
                if cap_type.is_empty() {
                    continue;
                }

                // Get operations for this capability
                let ops_var = format!("REQUIRED_OPERATIONS_{}", cap_type.to_uppercase());
                let operations = env(ops_var.as_str()).map_or_else(
                    |_| vec!["*".to_string()],
                    |ops| ops.split(',').map(|s| s.trim().to_string()).collect(),
                );

                requirements.push(CapabilityRequirement {
                    capability_type: cap_type.to_string(),
                    required_operations: operations,
                    quality_requirements: Self::parse_quality_requirements(env, cap_type),
                    fallback_behavior: Self::parse_fallback_behavior(env, cap_type),
                });
            }
        }

        debug!("🎯 Discovered {} required capabilities", requirements.len());
        requirements
    }

    /// Discover optional capabilities from environment
    fn discover_optional_capabilities(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Vec<CapabilityRequirement> {
        let mut requirements = Vec::new();

        if let Ok(optional) = env("OPTIONAL_CAPABILITIES") {
            for cap_type in optional.split(',') {
                let cap_type = cap_type.trim();
                if cap_type.is_empty() {
                    continue;
                }

                requirements.push(CapabilityRequirement {
                    capability_type: cap_type.to_string(),
                    required_operations: vec!["*".to_string()],
                    quality_requirements: QualityRequirements::default(),
                    fallback_behavior: FallbackBehavior::LocalFallback,
                });
            }
        }

        debug!("🎯 Discovered {} optional capabilities", requirements.len());
        requirements
    }

    /// Create discovery configuration from environment
    fn create_discovery_config(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> DiscoveryConfig {
        let mut methods = Vec::new();

        // Always include environment variable discovery
        methods.push(DiscoveryMethod::Environment {
            patterns: vec![
                "CAPABILITY_*_ENDPOINT".to_string(),
                "SERVICE_*_URL".to_string(),
                "*_PROVIDER_ENDPOINT".to_string(),
            ],
        });

        // Add HTTP registry if endpoint is provided
        if env("SERVICE_REGISTRY_ENDPOINT").is_ok() {
            methods.push(DiscoveryMethod::HttpRegistry {
                endpoint_env_var: "SERVICE_REGISTRY_ENDPOINT".to_string(),
                api_path: "/v1/services".to_string(),
            });
        }

        // Add DNS discovery if domain is provided
        if env("SERVICE_DISCOVERY_DOMAIN").is_ok() {
            methods.push(DiscoveryMethod::DnsSrv {
                domain_env_var: "SERVICE_DISCOVERY_DOMAIN".to_string(),
            });
        }

        // Add container metadata discovery if API endpoint is available
        if env("CONTAINER_METADATA_API").is_ok() {
            methods.push(DiscoveryMethod::ContainerMetadata {
                api_endpoint_env_var: "CONTAINER_METADATA_API".to_string(),
            });
        }

        // Add network scanning for development mode
        if env("ENABLE_NETWORK_DISCOVERY").is_ok() {
            warn!("🔍 Network scanning enabled (development mode only)");
            methods.push(DiscoveryMethod::NetworkScan {
                ranges_env_var: "DISCOVERY_IP_RANGES".to_string(),
                ports_env_var: "DISCOVERY_PORTS".to_string(),
            });
        }

        let timeout_secs =
            env("DISCOVERY_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);

        let refresh_secs =
            env("DISCOVERY_REFRESH_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);

        let cache_ttl_secs =
            env("DISCOVERY_CACHE_TTL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(300);

        DiscoveryConfig {
            methods,
            timeout: Duration::from_secs(timeout_secs),
            refresh_interval: Duration::from_secs(refresh_secs),
            enable_cache: env("DISABLE_DISCOVERY_CACHE").is_err(),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
        }
    }

    /// Create network configuration from environment (NO hardcoded ports)
    fn create_network_config(
        read_env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<NetworkConfig> {
        // Bind address from environment
        let bind_address =
            read_env("BIND_ADDRESS").ok().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                // Check if we're in production (container/cloud environment)
                if read_env("KUBERNETES_SERVICE_HOST").is_ok()
                    || read_env("DOCKER_HOST").is_ok()
                    || read_env("PRODUCTION").is_ok()
                {
                    IpAddr::V4(Ipv4Addr::UNSPECIFIED) // 0.0.0.0 for production
                } else {
                    IpAddr::V4(Ipv4Addr::LOCALHOST) // 127.0.0.1 for development
                }
            });

        // All ports from environment - NO defaults
        let service_port = read_env("SERVICE_PORT")
            .or_else(|_| read_env("PORT"))
            .map_err(|_| SongbirdError::Configuration {
                message: "SERVICE_PORT or PORT environment variable required".to_string(),
                field: Some("service_port".to_string()),
                suggestion: Some("Set SERVICE_PORT environment variable".to_string()),
            })?
            .parse()
            .map_err(|_| SongbirdError::Configuration {
                message: "Invalid SERVICE_PORT value - must be a valid port number".to_string(),
                field: Some("service_port".to_string()),
                suggestion: Some("Set SERVICE_PORT to a number between 1 and 65535".to_string()),
            })?;

        let health_port =
            read_env("HEALTH_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(service_port); // Use service port if not specified

        let metrics_port =
            read_env("METRICS_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(service_port); // Use service port if not specified

        let max_connections =
            read_env("MAX_CONNECTIONS").ok().and_then(|s| s.parse().ok()).unwrap_or(1000);

        let max_connections_per_ip =
            read_env("MAX_CONNECTIONS_PER_IP").ok().and_then(|s| s.parse().ok()).unwrap_or(100);

        let connection_timeout_secs =
            read_env("CONNECTION_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);

        let request_timeout_secs =
            read_env("REQUEST_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);

        info!(
            "🌐 Network config: {}:{} (health: {}, metrics: {})",
            bind_address, service_port, health_port, metrics_port
        );

        Ok(NetworkConfig {
            bind_address,
            service_port,
            health_port,
            metrics_port,
            connection_limits: ConnectionLimits {
                max_connections,
                max_connections_per_ip,
                connection_backlog: 128,
            },
            timeouts: NetworkTimeouts {
                connection_timeout: Duration::from_secs(connection_timeout_secs),
                request_timeout: Duration::from_secs(request_timeout_secs),
                idle_timeout: Duration::from_secs(300),
            },
        })
    }

    /// Create bootstrap configuration
    fn create_bootstrap_config(
        read_env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> BootstrapConfig {
        let enable_infant_discovery = read_env("ENABLE_INFANT_DISCOVERY")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        let discovery_phases = if enable_infant_discovery {
            vec![
                DiscoveryPhase::EnvironmentScan,
                DiscoveryPhase::RegistryQuery,
                DiscoveryPhase::NetworkProbe,
                DiscoveryPhase::CapabilityTest,
            ]
        } else {
            vec![DiscoveryPhase::EnvironmentScan, DiscoveryPhase::RegistryQuery]
        };

        let max_bootstrap_secs =
            read_env("MAX_BOOTSTRAP_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60);

        let fail_on_missing = read_env("FAIL_ON_MISSING_CAPABILITIES")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        BootstrapConfig {
            enable_infant_discovery,
            discovery_phases,
            max_bootstrap_time: Duration::from_secs(max_bootstrap_secs),
            fail_on_missing_required: fail_on_missing,
        }
    }

    /// Collect service metadata from environment
    fn collect_service_metadata(
        read_env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        // Collect standard metadata
        if let Ok(version) = read_env("SERVICE_VERSION") {
            metadata.insert("version".to_string(), version);
        }
        if let Ok(environment) = read_env("ENVIRONMENT") {
            metadata.insert("environment".to_string(), environment);
        }
        if let Ok(region) = read_env("REGION") {
            metadata.insert("region".to_string(), region);
        }
        if let Ok(az) = read_env("AVAILABILITY_ZONE") {
            metadata.insert("availability_zone".to_string(), az);
        }

        // Collect container metadata if available
        if let Ok(pod_name) = read_env("POD_NAME") {
            metadata.insert("pod_name".to_string(), pod_name);
        }
        if let Ok(namespace) = read_env("POD_NAMESPACE") {
            metadata.insert("namespace".to_string(), namespace);
        }
        if let Ok(node_name) = read_env("NODE_NAME") {
            metadata.insert("node_name".to_string(), node_name);
        }

        metadata
    }

    /// Parse quality requirements for a capability
    fn parse_quality_requirements(
        read_env: &impl Fn(&str) -> Result<String, std::env::VarError>,
        capability_type: &str,
    ) -> QualityRequirements {
        let prefix = format!("CAPABILITY_{}_", capability_type.to_uppercase());

        let max_response_key = format!("{prefix}MAX_RESPONSE_MS");
        let min_avail_key = format!("{prefix}MIN_AVAILABILITY");
        let min_tp_key = format!("{prefix}MIN_THROUGHPUT_RPS");
        let sec_key = format!("{prefix}SECURITY_LEVEL");

        let max_response_time = read_env(&max_response_key).ok().and_then(|s| s.parse().ok());

        let min_availability = read_env(&min_avail_key).ok().and_then(|s| s.parse().ok());

        let min_throughput = read_env(&min_tp_key).ok().and_then(|s| s.parse().ok());

        let security_level = read_env(&sec_key)
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "none" => Some(SecurityLevel::None),
                "basic" => Some(SecurityLevel::Basic),
                "encrypted" | "tls" => Some(SecurityLevel::Encrypted),
                "strong" | "strong_auth" => Some(SecurityLevel::StrongAuth),
                "maximum" | "max" => Some(SecurityLevel::Maximum),
                _ => None,
            })
            .unwrap_or(SecurityLevel::Basic);

        QualityRequirements {
            max_response_time_ms: max_response_time,
            min_availability,
            min_throughput_rps: min_throughput,
            security_level,
        }
    }

    /// Parse fallback behavior for a capability
    fn parse_fallback_behavior(
        read_env: &impl Fn(&str) -> Result<String, std::env::VarError>,
        capability_type: &str,
    ) -> FallbackBehavior {
        let var_name = format!("CAPABILITY_{}_FALLBACK", capability_type.to_uppercase());

        read_env(var_name.as_str())
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "fail" => Some(FallbackBehavior::Fail),
                "retry" => Some(FallbackBehavior::Retry {
                    max_attempts: 3,
                    backoff_ms: 1000,
                }),
                "degraded" => Some(FallbackBehavior::DegradedMode {
                    degraded_operations: vec!["*".to_string()],
                }),
                "local" => Some(FallbackBehavior::LocalFallback),
                _ => None,
            })
            .unwrap_or(FallbackBehavior::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
            })
    }
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            max_response_time_ms: Some(5000),
            min_availability: Some(0.95),
            min_throughput_rps: None,
            security_level: SecurityLevel::Basic,
        }
    }
}

#[cfg(test)]
#[path = "infant_config_tests.rs"]
mod tests;
