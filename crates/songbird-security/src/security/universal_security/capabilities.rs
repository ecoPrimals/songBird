//! Security Capability Discovery Discovery
//!
//! This module handles discovery and management of security capabilities
//! across different primal implementations.

use crate::security::universal_security::types::{SecurityCapabilityInfo, SecurityLevel};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal_primals::{
    traits::PrimalCapability,
    universal_registry::UniversalServiceRegistry,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Security capability discovery and management
pub struct SecurityCapabilityDiscovery {
    /// Universal primal registry for capability-based discovery
    primal_registry: Arc<dyn UniversalServiceRegistry>,
    /// Cache of discovered security capabilities
    security_capabilities: RwLock<HashMap<String, SecurityCapabilityInfo>>,
    /// Last capability discovery time
    last_discovery: RwLock<SystemTime>,
    /// Discovery cache duration
    cache_duration: Duration,
}

impl SecurityCapabilityDiscovery {
    /// Create a new security capability discovery instance
    #[must_use]
    pub fn new(
        primal_registry: Arc<dyn UniversalServiceRegistry>,
        cache_duration: Duration,
    ) -> Self {
        Self {
            primal_registry,
            security_capabilities: RwLock::new(HashMap::new()),
            last_discovery: RwLock::new(SystemTime::UNIX_EPOCH),
            cache_duration,
        }
    }

    /// Discover available security capabilities
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub async fn discover_capabilities(&self) -> SongbirdResult<Vec<SecurityCapabilityInfo>> {
        let now = SystemTime::now();
        let last_discovery = *self.last_discovery.read().await;
        
        // Check if we need to refresh the cache
        if now.duration_since(last_discovery).unwrap_or(Duration::MAX) < self.cache_duration {
            let capabilities = self.security_capabilities.read().await;
            return Ok(capabilities.values().cloned().collect());
        }
        
        info!("Discovering security capabilities across the ecosystem");
        let mut discovered_capabilities = Vec::new();

        // Query the primal registry for security-capable services
        match self.discover_security_primals().await {
            Ok(primals) => {
                for primal_info in primals {
                    match self.analyze_security_capabilities(&primal_info).await {
                        Ok(capability_info) => {
                            discovered_capabilities.push(capability_info);
                        }
                        Err(e) => {
                            warn!("Failed to analyze security capabilities for {}: {}", 
                                 primal_info.primal_id, e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to discover security primals: {}", e);
            }
        }

        // Update cache
        {
            let mut capabilities = self.security_capabilities.write().await;
            capabilities.clear();
            for cap in &discovered_capabilities {
                capabilities.insert(cap.primal_id.clone(), cap.clone());
            }
        }

        // Update last discovery time
        *self.last_discovery.write().await = now;

        info!("Discovered {} security capabilities", discovered_capabilities.len());
        
        Ok(discovered_capabilities)
    }

    /// Get capabilities for a specific security level
    pub async fn get_capabilities_for_level(
        &self,
        required_level: SecurityLevel,
    ) -> SongbirdResult<Vec<SecurityCapabilityInfo>> {
        let all_capabilities = self.discover_capabilities().await?;
        let filtered_capabilities: Vec<SecurityCapabilityInfo> = all_capabilities
            .into_iter()
            .filter(|cap| cap.security_level >= required_level)
            .collect();

                debug!("Found {} capabilities for security level {:?}", filtered_capabilities.len(), required_level);
        
        Ok(filtered_capabilities)
    }

    /// Get the best capability for a specific operation
    pub async fn get_best_capability(
        &self,
        operation: &str,
        required_level: SecurityLevel,
    ) -> SongbirdResult<Option<SecurityCapabilityInfo>> {
        let capabilities = self.get_capabilities_for_level(required_level).await?;
        
        // Filter by operation support
        let suitable_capabilities: Vec<SecurityCapabilityInfo> = capabilities
            .into_iter()
            .filter(|cap| cap.capabilities.iter().any(|c| c.contains(operation)))
            .collect();

        if suitable_capabilities.is_empty() {
            return Ok(None);
        }

        // Select the best capability based on performance metrics
        let best_capability = suitable_capabilities
            .into_iter()
            .min_by(|a, b| {
                let a_latency = a.performance_metrics.get("avg_latency_ms").unwrap_or(&1000.0);
                let b_latency = b.performance_metrics.get("avg_latency_ms").unwrap_or(&1000.0);
                a_latency.partial_cmp(b_latency).unwrap_or(std::cmp::Ordering::Equal)
            });

        Ok(best_capability)
    }

    /// Force refresh of capability cache
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub async fn refresh_capabilities(&self) -> Result<(), SongbirdError> {
        *self.last_discovery.write().await = SystemTime::UNIX_EPOCH;
        self.discover_capabilities().await?;
        Ok(())
    }

    /// Discover security-capable primals from the registry
    async fn discover_security_primals(&self) -> SongbirdResult<Vec<SecurityPrimalInfo>> {
        // This would integrate with the actual primal registry
        // For now, return a placeholder implementation
        Ok(vec![])
    }

    /// Analyze security capabilities of a specific primal
    async fn analyze_security_capabilities(&self,
        primal_info: &SecurityPrimalInfo,
    ) -> SongbirdResult<SecurityCapabilityInfo> {
        // This would perform actual capability analysis
        // For now, return a placeholder implementation
        Ok(SecurityCapabilityInfo {
            primal_id: primal_info.primal_id.clone(),
            instance_id: primal_info.instance_id.clone(),
            capabilities: vec!["authentication".to_string(), "authorization".to_string()],
            endpoint: primal_info.endpoint.clone(),
            last_health_check: SystemTime::now(),
            security_level: SecurityLevel::Standard,
            performance_metrics: HashMap::new(),
        })
    }
}

/// Temporary structure for primal information during discovery
#[derive(Debug, Clone)]
struct SecurityPrimalInfo {
    /// Primal Id field

    pub primal_id: String,
    /// Instance Id field
    pub instance_id: String,
    /// Endpoint field
    pub endpoint: String,
    /// List of supported capabilities
    pub capabilities: Vec<PrimalCapability>,
}

#[cfg(test)]
mod tests { 
    use super::*;
    use std::time::Duration;

    // Test implementation for unit tests
    struct TestRegistry {
        services: std::sync::Arc<tokio::sync::RwLock<Vec<songbird_universal_primals::types::ServiceInfo>>>,
    }

    impl TestRegistry {
        fn new() -> Self {
            Self {
                services: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl UniversalServiceRegistry for TestRegistry {
        async fn register_service(&self, service: songbird_universal_primals::types::ServiceInfo) -> SongbirdResult<()> {
            self.services.write().await.push(service);
            Ok(())
        }

        async fn discover_services(&self, _capability: PrimalCapability) -> SongbirdResult<Vec<songbird_universal_primals::types::ServiceInfo>> {
            Ok(self.services.read().await.clone())
        }

        async fn get_service_health(&self, _service_id: &str) -> SongbirdResult<songbird_universal_primals::types::ServiceHealth> {
            Ok(songbird_universal_primals::types::ServiceHealth::Healthy)
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_creation() {
        let registry = Arc::new(TestRegistry::new());
        let discovery = SecurityCapabilityDiscovery::new(registry, Duration::from_secs(300));
        
        // Test that we can create the discovery instance
        let capabilities = discovery.discover_capabilities().await.unwrap();
        assert!(capabilities.is_empty()); // Expected for empty test registry
    }

    #[tokio::test]
    async fn test_capability_filtering_by_level() {
        let registry = Arc::new(TestRegistry::new());
        let discovery = SecurityCapabilityDiscovery::new(registry, Duration::from_secs(300));
        
        let capabilities = discovery.get_capabilities_for_level(SecurityLevel::High).await.unwrap();
        assert!(capabilities.is_empty()); // Expected for empty test registry
    }
} 
