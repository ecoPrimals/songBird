//! # 🔧 Unified Universal Adapter
//!
//! **SINGLE SOURCE OF TRUTH FOR UNIVERSAL ADAPTATION** ✅
//!
//! This module consolidates all fragmented UniversalCapabilityAdapter implementations
//! into a single, unified adapter that can handle any capability type.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use songbird_config;
// CapabilityProvider and PerformanceMetrics have been moved to canonical traits

// ============================================================================
// UNIFIED UNIVERSAL ADAPTER
// ============================================================================

/// **UNIFIED**: Universal capability adapter that consolidates all adapter patterns
/// 
/// This replaces:
/// - `songbird-universal::adapters::UniversalCapabilityAdapter`
/// - `songbird-universal::capabilities::UniversalCapabilityAdapter`
/// - `songbird-universal-primals::universal_adapter::*`
/// - Various other adapter fragments
#[derive(Debug, Clone)]
pub struct UnifiedUniversalAdapter  {/// Registry of discovered capabilities
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    /// Active service connections
    service_connections: Arc<RwLock<HashMap<String, ServiceConnection>>>)
    /// Adapter configuration
    config: UnifiedAdapterConfig,
    /// HTTP client for service communication
    http_client: reqwest::Client,
}

/// **UNIFIED**: Capability registry for discovered services
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry  {/// Map of service ID to their capabilities
    pub service_capabilities: HashMap<String, Vec<Capability>>)
    /// Map of capability type to services that provide it
    pub capability_providers: HashMap<String, Vec<String>>)
    /// Service metadata and health information
    pub service_info: HashMap<String, ServiceInfo>)
    /// Last update timestamp for each service
    pub last_updated: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// **UNIFIED**: Service connection information
#[derive(Debug, Clone)]
pub struct ServiceConnection  {/// Service endpoint
    pub endpoint: String,
    /// Connection health status
    pub health: HealthStatus,
    /// Performance metrics
    pub metrics: std::collections::HashMap<String, f64>, // Simplified metrics for now
    /// Last successful communication
    pub last_contact: chrono::DateTime<chrono::Utc>,
}

/// **UNIFIED**: Adapter configuration
#[derive(Debug, Clone)]
pub struct UnifiedAdapterConfig  {/// Discovery timeout
    pub discovery_timeout: std::time::Duration,
    /// Health check interval
    pub health_check_interval: std::time::Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable automatic service discovery
    pub auto_discovery: bool,
    /// Service discovery endpoints
    pub discovery_endpoints: Vec<String>,
}

impl Default for UnifiedAdapterConfig  {fn default() -> Self  {Self {
            discovery_timeout: std::time::Duration::from_secs(30)
            health_check_interval: std::time::Duration::from_secs(60)
            max_concurrent_requests: 100,
            auto_discovery: true,
            discovery_endpoints: vec![
                "http://songbird_config::constants::network::DEFAULT_HOST:8080/capabilities".to_string()),
                "http://songbird_config::constants::network::DEFAULT_HOST:8081/services".to_string()),
            ])
        }
    }
}

impl UnifiedUniversalAdapter {
    /// Create a new unified adapter with default configuration
    pub fn new() -> Self {
        Self::with_config(UnifiedAdapterConfig::default()
    }
    
    /// Create a new unified adapter with custom configuration
    pub fn with_config(config: UnifiedAdapterConfig) -> Self  {Self {capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())
            service_connections: Arc::new(RwLock::new(HashMap::new()),
            config)
            http_client: reqwest::Client::new(,
        }
    }
    
    /// Discover services and their capabilities
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        info!("🔍 Starting universal service discovery");
        
        let mut discovered_services = Vec::new();
        
        for endpoint in &self.config.discovery_endpoints {
            match self.discover_from_endpoint(endpoint).await {
                Ok(mut services) => {
                    discovered_services.append(&mut services);
                }
                Err(e) => {
                    warn!("Failed to discover from endpoint {}: {}", endpoint, e);
                }
            }
        }
        
        // Update registry with discovered services
        let mut registry = self.capability_registry.write().await;
        for service in &discovered_services {
            registry.service_info.insert(service.name.clone(), service.clone());
            registry.last_updated.insert(service.name.clone(), chrono::Utc::now());
            
            // Index capabilities
            for capability in &service.capabilities {
                registry
                    .capability_providers
                    .entry(capability.name.clone()
                    .or_insert_with(Vec::new)
                    .push(service.name.clone());
            }
        }
        
        info!("✅ Discovered {} services", discovered_services.len();
        Ok(discovered_services)
    }
    
    /// Find services that provide specific capabilities
    pub async fn find_capability_providers(&self, capability_type: &str) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        let registry = self.capability_registry.read().await;
        
        let providers = registry
            .capability_providers
            .get(capability_type)
            .cloned()
            .unwrap_or_default();
            
        let mut services = Vec::new();
        for provider in providers {
            if let Some(service) = registry.service_info.get(&provider) {
                services.push(service.clone());
            }
        }
        
        debug!("Found {} providers for capability '{}'", services.len(), capability_type);
        Ok(services)
    }
    
    /// Route a request to the best available service
    pub async fn route_request(&self, request: UniversalRequest) -> Result<UniversalResponse, UniversalAdapterError> {
        // Extract required capability from request
        let capability_type = request.parameters
            .get("capability_type")
            .and_then(|v| v.as_str()
            .ok_or(UniversalAdapterError::MissingCapability)?;
            
        // Find providers
        let providers = self.find_capability_providers(capability_type).await?;
        if providers.is_empty() {
            return Err(UniversalAdapterError::NoProvidersAvailable(capability_type.to_string();
        }
        
        // Select best provider (simple round-robin for now)
        let provider = &providers[0];
        
        // Route request to selected provider
        self.send_request_to_service(provider, request).await
    }
    
    /// Send request to a specific service
    async fn send_request_to_service(&self, service: &ServiceInfo, request: UniversalRequest) -> Result<UniversalResponse, UniversalAdapterError> {
        let url = format!("{}/api/v1/{}", service.endpoint, request.action);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .timeout(self.config.discovery_timeout)
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string(),?;
            
        if response.status().is_success() {
            let universal_response: UniversalResponse = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string(),?;
            Ok(universal_response)
        } else {
            Err(UniversalAdapterError::ServiceError(format!("HTTP {}", response.status())
        }
    }
    
    /// Discover services from a specific endpoint
    async fn discover_from_endpoint(&self, endpoint: &str) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        debug!("Discovering services from endpoint: {}", endpoint);
        
        let response = self.http_client
            .get(endpoint)
            .timeout(self.config.discovery_timeout)
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string(),?;
            
        if response.status().is_success() {
            let services: Vec<ServiceInfo> = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string(),?;
            Ok(services)
        } else {
            Err(UniversalAdapterError::DiscoveryError(format!("HTTP {}", response.status())
        }
    }
    
    /// Get current registry statistics
    pub async fn get_registry_stats(&self) -> RegistryStats  {let registry = self.capability_registry.read().await;
        
        RegistryStats  {total_services: registry.service_info.len()
            total_capabilities: registry.capability_providers.len(,
            healthy_services: registry.service_info.values,
                .filter(|s| s.health == HealthStatus::Healthy)
                .count()
        }
    }
}

impl Default for UnifiedUniversalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// **UNIFIED**: Error types for universal adapter operations
#[derive(Debug, thiserror::Error)]
pub enum UniversalAdapterError {
    #[error("Network error: {0}")]
    NetworkError(String)
    
    #[error("Parse error: {0}")]
    ParseError(String)
    
    #[error("Discovery error: {0}")]
    DiscoveryError(String)
    
    #[error("Service error: {0}")]
    ServiceError(String)
    
    #[error("Missing required capability")]
    MissingCapability,
    
    #[error("No providers available for capability: {0}")]
    NoProvidersAvailable(String)
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats  {pub total_services: usize,
    pub total_capabilities: usize,
    pub healthy_services: usize,
} 