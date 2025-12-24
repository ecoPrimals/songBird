//! # 🔄 Agnostic Primal Configuration
//!
//! **ZERO PRIMAL NAME HARDCODING** - Evolution from specific to agnostic
//!
//! This module replaces hardcoded primal names (BearDog, Toadstool, NestGate, Squirrel)
//! with capability-based discovery.
//!
//! ## Evolution Pattern
//!
//! **BEFORE (Specific)**:
//! ```rust,ignore
//! let beardog = connect_to_beardog("localhost:8443");
//! let keys = beardog.generate_keys().await?;
//! ```
//!
//! **AFTER (Agnostic)**:
//! ```rust
//! # use songbird_config::agnostic_primal_config::*;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let coordinator = AgnosticPrimalConfig::from_environment()?;
//! let security = coordinator.request_capability("security").await?;
//! // security could be any primal that provides security capability
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

/// Agnostic primal configuration - NO hardcoded primal names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgnosticPrimalConfig {
    /// Capability-to-endpoint mapping (discovered or from env)
    pub capability_endpoints: HashMap<String, String>,
    
    /// Discovery configuration
    pub discovery: CapabilityDiscoveryConfig,
    
    /// Service mesh configuration (for primal-to-primal)
    pub service_mesh: ServiceMeshConfig,
}

/// Capability discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiscoveryConfig {
    /// Enable automatic discovery
    pub enabled: bool,
    
    /// Discovery methods to try (in order)
    pub methods: Vec<DiscoveryMethod>,
    
    /// Discovery timeout in seconds
    pub timeout_secs: u64,
    
    /// Cache discovered endpoints
    pub enable_cache: bool,
    
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

/// Discovery method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// Environment variables (CAPABILITY_SECURITY_ENDPOINT, etc.)
    Environment,
    
    /// DNS SRV records (_security._tcp.local, etc.)
    DnsSrv,
    
    /// HTTP service registry (Consul, Eureka, etc.)
    HttpRegistry,
    
    /// Container metadata (Kubernetes, Docker, etc.)
    ContainerMetadata,
    
    /// mDNS/Bonjour
    Mdns,
    
    /// Static configuration file
    StaticFile,
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Enable service mesh coordination
    pub enabled: bool,
    
    /// Mesh protocol (grpc, tarpc, http, etc.)
    pub protocol: String,
    
    /// Enable TLS for mesh connections
    pub enable_tls: bool,
    
    /// Mesh discovery interval in seconds
    pub discovery_interval_secs: u64,
}

impl AgnosticPrimalConfig {
    /// Create configuration from environment
    ///
    /// **ZERO HARDCODING**: All endpoints come from environment or discovery
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing
    pub fn from_environment() -> SongbirdResult<Self> {
        tracing::info!("🔄 Creating agnostic primal config (zero hardcoded primal names)");
        
        let capability_endpoints = Self::discover_capability_endpoints();
        let discovery = Self::create_discovery_config();
        let service_mesh = Self::create_service_mesh_config();
        
        Ok(Self {
            capability_endpoints,
            discovery,
            service_mesh,
        })
    }
    
    /// Discover capability endpoints from environment
    ///
    /// Pattern: `CAPABILITY_<TYPE>_ENDPOINT`
    /// Example: `CAPABILITY_SECURITY_ENDPOINT=https://localhost:8443`
    fn discover_capability_endpoints() -> HashMap<String, String> {
        let mut endpoints = HashMap::new();
        
        // Standard capabilities to check
        let capabilities = ["security", "compute", "storage", "ai", "discovery", "orchestration"];
        
        for capability in &capabilities {
            let env_var = format!("CAPABILITY_{}_ENDPOINT", capability.to_uppercase());
            if let Ok(endpoint) = std::env::var(&env_var) {
                tracing::info!("Discovered {} capability at: {}", capability, endpoint);
                endpoints.insert(capability.to_string(), endpoint);
            }
        }
        
        // Check for custom capabilities
        for (key, value) in std::env::vars() {
            if key.starts_with("CAPABILITY_") && key.ends_with("_ENDPOINT") {
                let capability = key
                    .trim_start_matches("CAPABILITY_")
                    .trim_end_matches("_ENDPOINT")
                    .to_lowercase();
                if !endpoints.contains_key(&capability) {
                    tracing::info!("Discovered custom {} capability at: {}", capability, value);
                    endpoints.insert(capability, value);
                }
            }
        }
        
        if endpoints.is_empty() {
            tracing::warn!("No capability endpoints discovered from environment");
        }
        
        endpoints
    }
    
    /// Create discovery configuration from environment
    fn create_discovery_config() -> CapabilityDiscoveryConfig {
        let enabled = std::env::var("CAPABILITY_DISCOVERY_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        
        let mut methods = Vec::new();
        
        // Always include environment discovery
        methods.push(DiscoveryMethod::Environment);
        
        // Add other methods based on environment
        if std::env::var("ENABLE_DNS_SRV_DISCOVERY").is_ok() {
            methods.push(DiscoveryMethod::DnsSrv);
        }
        if std::env::var("SERVICE_REGISTRY_ENDPOINT").is_ok() {
            methods.push(DiscoveryMethod::HttpRegistry);
        }
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            methods.push(DiscoveryMethod::ContainerMetadata);
        }
        if std::env::var("ENABLE_MDNS_DISCOVERY").is_ok() {
            methods.push(DiscoveryMethod::Mdns);
        }
        
        let timeout_secs = std::env::var("CAPABILITY_DISCOVERY_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        let cache_ttl_secs = std::env::var("CAPABILITY_CACHE_TTL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        
        CapabilityDiscoveryConfig {
            enabled,
            methods,
            timeout_secs,
            enable_cache: std::env::var("DISABLE_CAPABILITY_CACHE").is_err(),
            cache_ttl_secs,
        }
    }
    
    /// Create service mesh configuration
    fn create_service_mesh_config() -> ServiceMeshConfig {
        let enabled = std::env::var("SERVICE_MESH_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        
        let protocol = std::env::var("SERVICE_MESH_PROTOCOL")
            .unwrap_or_else(|_| "tarpc".to_string());
        
        let enable_tls = std::env::var("SERVICE_MESH_TLS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        
        let discovery_interval_secs = std::env::var("SERVICE_MESH_DISCOVERY_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        
        ServiceMeshConfig {
            enabled,
            protocol,
            enable_tls,
            discovery_interval_secs,
        }
    }
    
    /// Request an endpoint for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if the capability is not available
    pub async fn request_capability(&self, capability: &str) -> SongbirdResult<String> {
        // Check cached endpoints first
        if let Some(endpoint) = self.capability_endpoints.get(capability) {
            return Ok(endpoint.clone());
        }
        
        // If discovery is enabled, try to discover
        if self.discovery.enabled {
            tracing::debug!("Attempting to discover {} capability", capability);
            // TODO: Implement dynamic discovery
            // For now, return error if not in cache
        }
        
        Err(SongbirdError::Configuration {
            message: format!("Capability '{}' not available", capability),
            field: Some("capability".to_string()),
            suggestion: Some(format!("Set CAPABILITY_{}_ENDPOINT environment variable", capability.to_uppercase())),
        })
    }
}

impl Default for AgnosticPrimalConfig {
    fn default() -> Self {
        Self {
            capability_endpoints: HashMap::new(),
            discovery: CapabilityDiscoveryConfig {
                enabled: true,
                methods: vec![DiscoveryMethod::Environment],
                timeout_secs: 30,
                enable_cache: true,
                cache_ttl_secs: 300,
            },
            service_mesh: ServiceMeshConfig {
                enabled: true,
                protocol: "tarpc".to_string(),
                enable_tls: true,
                discovery_interval_secs: 60,
            },
        }
    }
}

/// Migration helper: Convert legacy primal-specific config to agnostic config
///
/// **USE THIS** to migrate from hardcoded `beardog_endpoint` to capability-based
pub struct PrimalConfigMigration;

impl PrimalConfigMigration {
    /// Migrate legacy environment variables to capability-based
    ///
    /// Maps:
    /// - `SONGBIRD_BEARDOG_ENDPOINT` → `CAPABILITY_SECURITY_ENDPOINT`
    /// - `SONGBIRD_TOADSTOOL_ENDPOINT` → `CAPABILITY_COMPUTE_ENDPOINT`
    /// - `SONGBIRD_NESTGATE_ENDPOINT` → `CAPABILITY_STORAGE_ENDPOINT`
    /// - `SONGBIRD_SQUIRREL_ENDPOINT` → `CAPABILITY_AI_ENDPOINT`
    pub fn migrate_legacy_env_vars() {
        let migrations = [
            ("SONGBIRD_BEARDOG_ENDPOINT", "CAPABILITY_SECURITY_ENDPOINT"),
            ("SONGBIRD_TOADSTOOL_ENDPOINT", "CAPABILITY_COMPUTE_ENDPOINT"),
            ("SONGBIRD_NESTGATE_ENDPOINT", "CAPABILITY_STORAGE_ENDPOINT"),
            ("SONGBIRD_SQUIRREL_ENDPOINT", "CAPABILITY_AI_ENDPOINT"),
        ];
        
        for (legacy_var, capability_var) in &migrations {
            if let Ok(value) = std::env::var(legacy_var) {
                if std::env::var(capability_var).is_err() {
                    tracing::warn!(
                        "🔄 Migrating {} to {} (value: {})",
                        legacy_var,
                        capability_var,
                        value
                    );
                    std::env::set_var(capability_var, value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discover_capability_endpoints() {
        // Set test environment variables
        std::env::set_var("CAPABILITY_SECURITY_ENDPOINT", "https://localhost:8443");
        std::env::set_var("CAPABILITY_COMPUTE_ENDPOINT", "http://localhost:8082");
        
        let endpoints = AgnosticPrimalConfig::discover_capability_endpoints();
        
        assert_eq!(endpoints.get("security"), Some(&"https://localhost:8443".to_string()));
        assert_eq!(endpoints.get("compute"), Some(&"http://localhost:8082".to_string()));
        
        // Cleanup
        std::env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
        std::env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    }
    
    #[test]
    fn test_discovery_config_creation() {
        std::env::set_var("CAPABILITY_DISCOVERY_ENABLED", "true");
        std::env::set_var("ENABLE_DNS_SRV_DISCOVERY", "1");
        
        let config = AgnosticPrimalConfig::create_discovery_config();
        
        assert!(config.enabled);
        assert!(config.methods.contains(&DiscoveryMethod::Environment));
        assert!(config.methods.contains(&DiscoveryMethod::DnsSrv));
        
        // Cleanup
        std::env::remove_var("CAPABILITY_DISCOVERY_ENABLED");
        std::env::remove_var("ENABLE_DNS_SRV_DISCOVERY");
    }
    
    #[test]
    fn test_migration_helper() {
        // Set legacy variables
        std::env::set_var("SONGBIRD_BEARDOG_ENDPOINT", "https://beardog:8443");
        std::env::set_var("SONGBIRD_TOADSTOOL_ENDPOINT", "http://toadstool:8082");
        
        // Migrate
        PrimalConfigMigration::migrate_legacy_env_vars();
        
        // Check new variables are set
        assert_eq!(
            std::env::var("CAPABILITY_SECURITY_ENDPOINT").ok(),
            Some("https://beardog:8443".to_string())
        );
        assert_eq!(
            std::env::var("CAPABILITY_COMPUTE_ENDPOINT").ok(),
            Some("http://toadstool:8082".to_string())
        );
        
        // Cleanup
        std::env::remove_var("SONGBIRD_BEARDOG_ENDPOINT");
        std::env::remove_var("SONGBIRD_TOADSTOOL_ENDPOINT");
        std::env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
        std::env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    }
}

