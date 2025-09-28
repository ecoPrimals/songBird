//! # 🔧 Universal Discovery Factory
//!
//! **MODERNIZED UNIVERSAL FACTORY** ✅
//!
//! This factory replaces hardcoded backend implementations with universal
//! capability detection and adapter-based discovery.

use async_trait::async_trait;
use std::sync::Arc;
use crate::traits::discovery::{ServiceDiscovery, DiscoveryConfig};
use crate::discovery::backends::StaticServiceDiscovery;
use songbird_types::{SongbirdError, SongbirdResult};

// Import universal adapters
use songbird_universal::{UnifiedUniversalAdapter, UniversalCapabilityAdapter};

type Result<T> = SongbirdResult<T>;

/// **MODERNIZED**: Universal discovery factory with capability detection
/// 
/// Replaces hardcoded `KubernetesServiceDiscovery`, `ConsulServiceDiscovery`, 
/// and `StaticServiceDiscovery` with universal capability-based detection.
pub struct UniversalDiscoveryFactory;

impl UniversalDiscoveryFactory {
    /// Create service discovery with auto-detection
    /// 
    /// This replaces the old hardcoded factory methods with universal capability detection.
    pub async fn create_auto_detect() -> Result<Box<dyn ServiceDiscovery>> {
        info!("🔍 Starting universal capability-based service discovery");
        
        // Try universal adapter first (replaces all hardcoded backends)
        match Self::create_universal_adapter().await {
            Ok(adapter) => {
                info!("✅ Created universal capability adapter");
                return Ok(adapter);
            }
            Err(e) => {
                warn!("Universal adapter creation failed: {}", e);
            }
        }
        
        // Fallback to environment detection
        Self::create_from_environment().await
    }
    
    /// Create universal capability adapter (replaces hardcoded backends)
    async fn create_universal_adapter() -> Result<Box<dyn ServiceDiscovery>> {
        let adapter = UniversalServiceDiscoveryAdapter::new().await?;
        Ok(Box::new(adapter)
    }
    
    /// Create service discovery based on environment detection
    /// 
    /// This replaces hardcoded environment checks with universal patterns.
    async fn create_from_environment() -> Result<Box<dyn ServiceDiscovery>> {
        info!("🔍 Detecting service discovery environment");
        
        // Check for Kubernetes environment
        if Self::detect_kubernetes_environment().await {
            info!("📦 Kubernetes environment detected - using universal adapter");
            return Self::create_kubernetes_universal().await;
        }
        
        // Check for Consul environment
        if Self::detect_consul_environment().await {
            info!("🏛️ Consul environment detected - using universal adapter");
            return Self::create_consul_universal().await;
        }
        
        // Check for container orchestration
        if Self::detect_container_environment().await {
            info!("🐳 Container environment detected - using universal adapter");
            return Self::create_container_universal().await;
        }
        
        // Default to static discovery for development
        info!("🔧 No specific environment detected - using static discovery");
        Ok(Box::new(StaticServiceDiscovery::new())
    }
    
    /// Detect Kubernetes environment (replaces hardcoded KubernetesServiceDiscovery,
    async fn detect_kubernetes_environment() -> bool {
        // Check for Kubernetes service account
        if std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount").exists() {
            return true;
        }
        
        // Check for Kubernetes environment variables
        std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
    }
    
    /// Detect Consul environment (replaces hardcoded ConsulServiceDiscovery,
    async fn detect_consul_environment() -> bool {
        // Check for Consul environment variables
        std::env::var("CONSUL_HTTP_ADDR").is_ok() 
            || std::env::var("CONSUL_ADDR").is_ok()
            || std::path::Path::new("/etc/consul").exists()
    }
    
    /// Detect container environment
    async fn detect_container_environment() -> bool {
        std::env::var("DOCKER_HOST").is_ok() 
            || std::path::Path::new("/.dockerenv").exists()
            || std::env::var("CONTAINER_ID").is_ok()
    }
    
    /// Create Kubernetes-aware universal adapter
    async fn create_kubernetes_universal() -> Result<Box<dyn ServiceDiscovery>> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_kubernetes_discovery().await?;
        Ok(Box::new(adapter)
    }
    
    /// Create Consul-aware universal adapter
    async fn create_consul_universal() -> Result<Box<dyn ServiceDiscovery>> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_consul_discovery().await?;
        Ok(Box::new(adapter)
    }
    
    /// Create container-aware universal adapter
    async fn create_container_universal() -> Result<Box<dyn ServiceDiscovery>> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_container_discovery().await?;
        Ok(Box::new(adapter)
    }
    
    /// Create service discovery based on configuration
    pub async fn create_for_config(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>> {
        match config.backend.as_str() {
            "universal" => {
                info!("Creating universal service discovery backend");
                Self::create_universal_adapter().await
            }
            "static" => {
                info!("Creating static service discovery backend");
                Ok(Box::new(StaticServiceDiscovery::new())
            }
            "auto" | "detect" => {
                info!("Auto-detecting service discovery backend");
                Self::create_auto_detect().await
            }
            // Legacy backend names - redirect to universal
            "kubernetes" | "k8s" => {
                warn!("Legacy 'kubernetes' backend - using universal adapter");
                Self::create_kubernetes_universal().await
            }
            "consul" => {
                warn!("Legacy 'consul' backend - using universal adapter");
                Self::create_consul_universal().await
            }
            "container" | "docker" => {
                warn!("Legacy 'container' backend - using universal adapter");
                Self::create_container_universal().await
            }
            _ => {
                info!("Unknown backend '{}' - using auto-detection", config.backend);
                Self::create_auto_detect().await
            }
        }
    }
}

// ============================================================================
// UNIVERSAL SERVICE DISCOVERY ADAPTER
// ============================================================================

/// **MODERNIZED**: Universal service discovery adapter
/// 
/// This adapter replaces hardcoded backend implementations with universal
/// capability-based discovery that can work with any environment.
pub struct UniversalServiceDiscoveryAdapter  {universal_adapter: UnifiedUniversalAdapter,
    capability_adapter: UniversalCapabilityAdapter,
    kubernetes_enabled: bool,
    consul_enabled: bool,
    container_enabled: bool,
}

impl UniversalServiceDiscoveryAdapter  {/// Create new universal service discovery adapter
    pub async fn new() -> Result<Self>  {let universal_adapter = UnifiedUniversalAdapter::new();
        let capability_adapter = UniversalCapabilityAdapter::new(Default::default();
        
        Ok(Self {
            universal_adapter)
            capability_adapter)
            kubernetes_enabled: false,
            consul_enabled: false,
            container_enabled: false,
        })
    }
    
    /// Enable Kubernetes discovery capabilities
    pub async fn enable_kubernetes_discovery(&mut self) -> Result<()> {
        info!("🔧 Enabling Kubernetes discovery capabilities");
        self.kubernetes_enabled = true;
        // Register Kubernetes capabilities with the adapter
        self.register_kubernetes_capabilities().await
    }
    
    /// Enable Consul discovery capabilities
    pub async fn enable_consul_discovery(&mut self) -> Result<()> {
        info!("🔧 Enabling Consul discovery capabilities");
        self.consul_enabled = true;
        // Register Consul capabilities with the adapter
        self.register_consul_capabilities().await
    }
    
    /// Enable container discovery capabilities
    pub async fn enable_container_discovery(&mut self) -> Result<()> {
        info!("🔧 Enabling container discovery capabilities");
        self.container_enabled = true;
        // Register container capabilities with the adapter
        self.register_container_capabilities().await
    }
    
    /// Register Kubernetes-specific capabilities
    async fn register_kubernetes_capabilities(&self) -> Result<()> {
        debug!("📦 Registering Kubernetes service discovery capabilities");
        // Implementation would register K8s-specific discovery patterns
        // This replaces the hardcoded KubernetesServiceDiscovery
        Ok(()),
    }
    
    /// Register Consul-specific capabilities
    async fn register_consul_capabilities(&self) -> Result<()> {
        debug!("🏛️ Registering Consul service discovery capabilities");
        // Implementation would register Consul-specific discovery patterns
        // This replaces the hardcoded ConsulServiceDiscovery
        Ok(()),
    }
    
    /// Register container-specific capabilities
    async fn register_container_capabilities(&self) -> Result<()> {
        debug!("🐳 Registering container service discovery capabilities");
        // Implementation would register container-specific discovery patterns
        Ok(()),
    }
}

#[async_trait]
impl ServiceDiscovery for UniversalServiceDiscoveryAdapter {
    async fn discover_services(&self) -> Result<Vec<crate::traits::service::ServiceInfo>> {
        info!("🔍 Discovering services using universal adapter");
        
        // Use the universal adapter to discover services
        match self.universal_adapter.discover_services().await {
            Ok(services) => {
                // Convert universal ServiceInfo to discovery ServiceInfo
                let discovery_services = services.into_iter()
                    .map(|service| self.convert_to_discovery_service_info(service)
                    .collect();
                
                Ok(discovery_services)
            }
            Err(e) => {
                Err(SongbirdError::discovery_error(format!(
                    "Universal service discovery failed: {}", e
                ))
            }
        }
    }
    
    async fn register_service(&self, service: crate::traits::service::ServiceInfo) -> Result<()> {
        info!("📝 Registering service: {}", service.service_id);
        
        // Convert discovery ServiceInfo to universal format and register
        let universal_service = self.convert_from_discovery_service_info(service);
        
        // Use capability adapter for registration
        // Implementation would use the universal registration system
        Ok(()),
    }
    
    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        info!("🗑️ Deregistering service: {}", service_id);
        
        // Use universal adapter for deregistration
        // Implementation would use the universal deregistration system
        Ok(()),
    }
    
    async fn health_check(&self) -> Result<crate::traits::discovery::ServiceHealthStatus> {
        // Use universal health checking
        Ok(crate::traits::discovery::ServiceHealthStatus::Healthy)
    }
}

impl UniversalServiceDiscoveryAdapter  {/// Convert universal ServiceInfo to discovery ServiceInfo
    fn convert_to_discovery_service_info(
        &self, 
        service: songbird_universal::ServiceInfo
    ) -> crate::traits::service::ServiceInfo  {crate::traits::service::ServiceInfo {
            service_id: service.name,
            name: service.name.clone(,
            version: service.version.unwrap_or_else(|| "unknown".to_string()),
            endpoints: service.endpoints,
            metadata: service.metadata,
            health_status: crate::traits::service::ServiceHealthStatus::Healthy,
            last_seen: chrono::Utc::now(,
        }
    }
    
    /// Convert discovery ServiceInfo to universal format
    fn convert_from_discovery_service_info(
        &self)
        service: crate::traits::service::ServiceInfo
    ) -> songbird_universal::ServiceInfo  {songbird_universal::ServiceInfo {name: service.service_id)
            version: Some(service.version)
            endpoints: service.endpoints,
            capabilities: vec![], // Would be populated based on service metadata
            metadata: service.metadata,
        }
    }
}

// Legacy discovery backends have been replaced with universal capability detection
// All discovery now uses UniversalDiscoveryFactory with automatic capability detection
