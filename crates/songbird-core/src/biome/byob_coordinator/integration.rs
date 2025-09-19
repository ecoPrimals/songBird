//! BYOB Coordinator Integration with Universal Storage Capabilities
//!
//! This module provides BYOB (Bring Your Own Backend) coordination functionality
//! using the universal capability adapter system for storage primal discovery.

use songbird_config::config::constants::get_primal_endpoint;
use songbird_errors::SongbirdResult;
use songbird_universal::capabilities::UniversalCapabilityAdapter;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// BYOB Coordinator with universal storage capability integration
pub struct ByobCoordinator {
    /// Universal capability adapter for storage primal discovery
    capability_adapter: UniversalCapabilityAdapter,

    /// Active storage configurations (replaces hardcoded nestgate_config)
    storage_configs: HashMap<String, StorageConfig>,

    /// Current coordination status
    coordination_status: CoordinationStatus,

    /// Last capability refresh
    last_capability_refresh: Option<chrono::DateTime<chrono::Utc>>,
}

/// Universal storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub primal_name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub is_active: bool,
    pub config_data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct CoordinationStatus {
    pub active_storage_count: usize,
    pub total_storage_capacity: u64,
    pub coordination_health: HealthStatus,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl ByobCoordinator {
    /// Create new BYOB coordinator with universal storage discovery
    pub async fn new() -> SongbirdResult<Self> {
        info!("🚀 Initializing BYOB Coordinator with universal storage system");

        let discovery_config = songbird_universal::capabilities::DiscoveryConfig::default();
        let capability_adapter = UniversalCapabilityAdapter::new(discovery_config);

        let mut coordinator = Self {
            capability_adapter,
            storage_configs: HashMap::new(),
            coordination_status: CoordinationStatus {
                active_storage_count: 0,
                total_storage_capacity: 0,
                coordination_health: HealthStatus::Healthy,
            },
            last_capability_refresh: None,
        };

        // Discover storage primals (replaces hardcoded nestgate_config)
        coordinator.refresh_storage_capabilities().await?;

        info!(
            "✅ BYOB Coordinator initialized with {} storage configurations",
            coordinator.storage_configs.len()
        );

        Ok(coordinator)
    }

    /// Refresh storage capabilities discovery
    async fn refresh_storage_capabilities(&mut self) -> SongbirdResult<()> {
        info!("🔍 Discovering storage capability primals...");

        // Find all primals with storage capabilities
        let storage_primals = self
            .capability_adapter
            .find_capability_providers("storage")
            .await;

        self.storage_configs.clear();

        for primal_name in storage_primals {
            let endpoint = get_primal_endpoint(&primal_name);
            debug!("Found storage primal: {} at {}", primal_name, endpoint);

            // Create storage configuration for this primal
            let storage_config = StorageConfig {
                primal_name: primal_name.clone(),
                endpoint: endpoint.clone(),
                capabilities: vec!["storage".to_string(), "object-storage".to_string()],
                is_active: self.test_storage_connectivity(&endpoint).await,
                config_data: serde_json::json!({
                    "endpoint": endpoint,
                    "primal_type": primal_name,
                    "discovered_at": chrono::Utc::now(),
                }),
            };

            if storage_config.is_active {
                info!("✅ Connected to storage primal: {}", primal_name);
            } else {
                warn!("⚠️ Could not connect to storage primal: {}", primal_name);
            }

            self.storage_configs
                .insert(primal_name.clone(), storage_config);
        }

        // Fallback: Try capability-based discovery for storage providers
        if self.storage_configs.is_empty() {
            info!("🔄 No storage primals discovered, trying capability-based fallback...");

            let storage_providers = self
                .capability_adapter
                .find_capability_providers("storage")
                .await;

            for provider_name in storage_providers {
                let endpoint = get_primal_endpoint(&provider_name);
                let is_active = self.test_storage_connectivity(&endpoint).await;

                let storage_config = StorageConfig {
                    primal_name: provider_name.clone(),
                    endpoint: endpoint.clone(),
                    capabilities: vec!["storage".to_string(), "file_system".to_string()],
                    is_active,
                    config_data: serde_json::json!({
                        "endpoint": endpoint,
                        "primal_type": provider_name,
                        "discovery_method": "capability_based",
                    }),
                };

                if storage_config.is_active {
                    info!("✅ Connected to storage provider: {}", provider_name);
                    self.storage_configs.insert(provider_name, storage_config);
                    break; // Use the first working storage provider
                }
            }
        }

        // Update coordination status
        self.update_coordination_status();
        self.last_capability_refresh = Some(chrono::Utc::now());

        if self.storage_configs.is_empty() {
            warn!("⚠️ No storage capabilities available - coordination will be limited");
        }

        Ok(())
    }

    /// Test storage connectivity
    async fn test_storage_connectivity(&self, endpoint: &str) -> bool {
        debug!("🔍 Testing storage connectivity to {}", endpoint);

        match reqwest::Client::new()
            .get(&format!("{}/health", endpoint))
            .send()
            .await
        {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                debug!("Storage health check result: {}", is_healthy);
                is_healthy
            }
            Err(e) => {
                debug!("Storage connectivity test failed: {}", e);
                false
            }
        }
    }

    /// Update coordination status based on active storage configs
    fn update_coordination_status(&mut self) {
        let active_count = self
            .storage_configs
            .values()
            .filter(|config| config.is_active)
            .count();

        self.coordination_status = CoordinationStatus {
            active_storage_count: active_count,
            total_storage_capacity: active_count as u64 * 1000, // Placeholder calculation
            coordination_health: match active_count {
                0 => HealthStatus::Unhealthy,
                1 => HealthStatus::Degraded,
                _ => HealthStatus::Healthy,
            },
        };
    }

    /// Get storage configuration by name (universal replacement for nestgate_config access)
    pub fn get_storage_config(&self, primal_name: &str) -> Option<&StorageConfig> {
        self.storage_configs.get(primal_name)
    }

    /// Get all active storage configurations
    pub fn get_active_storage_configs(&self) -> Vec<&StorageConfig> {
        self.storage_configs
            .values()
            .filter(|config| config.is_active)
            .collect()
    }

    /// Get coordination status
    pub fn get_coordination_status(&self) -> &CoordinationStatus {
        &self.coordination_status
    }

    /// Enable storage configuration (universal replacement for nestgate_config = Some(config))
    pub async fn enable_storage_config(&mut self, primal_name: &str) -> SongbirdResult<()> {
        info!("🔧 Enabling storage configuration for: {}", primal_name);

        if let Some(config) = self.storage_configs.get(primal_name) {
            // Extract endpoint before mutable borrow
            let endpoint = config.endpoint.clone();
            let _ = config; // Release the immutable borrow

            // Test connectivity
            let is_active = self.test_storage_connectivity(&endpoint).await;

            // Now get mutable reference and update
            if let Some(config) = self.storage_configs.get_mut(primal_name) {
                config.is_active = is_active;

                if config.is_active {
                    info!("✅ Enabled storage configuration for: {}", primal_name);
                    self.update_coordination_status();
                    Ok(())
                } else {
                    warn!("❌ Failed to connect to storage primal: {}", primal_name);
                    Err(CoordinationError::StorageUnavailable(primal_name.to_string()).into())
                }
            } else {
                Err(CoordinationError::StorageUnavailable(format!(
                    "Storage config disappeared for: {}",
                    primal_name
                ))
                .into())
            }
        } else {
            warn!("❌ Storage configuration not found: {}", primal_name);
            Err(CoordinationError::ConfigNotFound(primal_name.to_string()).into())
        }
    }
}

/// Error types for coordination operations
#[derive(Debug)]
pub enum CoordinationError {
    StorageUnavailable(String),
    ConfigNotFound(String),
    NetworkError(String),
    CapabilityNotFound(String),
}

impl std::fmt::Display for CoordinationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinationError::StorageUnavailable(name) => {
                write!(f, "Storage unavailable: {}", name)
            }
            CoordinationError::ConfigNotFound(name) => write!(f, "Config not found: {}", name),
            CoordinationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            CoordinationError::CapabilityNotFound(cap) => {
                write!(f, "Capability not found: {}", cap)
            }
        }
    }
}

impl std::error::Error for CoordinationError {}

impl From<CoordinationError> for songbird_errors::SongbirdError {
    fn from(error: CoordinationError) -> Self {
        match error {
            CoordinationError::StorageUnavailable(name) => {
                songbird_errors::SongbirdError::Service {
                    service: "BYOB_Coordinator".to_string(),
                    message: format!("Storage unavailable: {}", name),
                    suggested_alternatives: vec!["Try alternative storage primal".to_string()],
                    recovery_actions: vec!["Check storage primal connectivity".to_string()],
                }
            }
            CoordinationError::ConfigNotFound(name) => {
                songbird_errors::SongbirdError::Configuration {
                    message: format!("Config not found: {}", name),
                    field: Some("primal_config".to_string()),
                    suggestion: Some("Check primal configuration".to_string()),
                }
            }
            CoordinationError::NetworkError(msg) => songbird_errors::SongbirdError::Network {
                message: format!("Network error: {}", msg),
                interface: None,
                suggestion: None,
            },
            CoordinationError::CapabilityNotFound(cap) => songbird_errors::SongbirdError::Service {
                service: "BYOB_Coordinator".to_string(),
                message: format!("Capability not found: {}", cap),
                suggested_alternatives: vec!["Check available capabilities".to_string()],
                recovery_actions: vec!["Update primal configuration".to_string()],
            },
        }
    }
}
