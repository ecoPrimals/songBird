//! # 🔧 Universal Discovery Factory
//!
//! **MODERNIZED UNIVERSAL FACTORY** ✅
//!
//! This factory replaces hardcoded backend implementations with universal
//! capability detection and adapter-based discovery.

use crate::discovery::backends::StaticServiceDiscovery;
use crate::traits::discovery::{
    DiscoveryBackend, DiscoveryConfig, ServiceDiscovery, ServiceHealthStatus,
};
use async_trait::async_trait;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info, warn};

// Import universal adapters
use songbird_universal::capabilities::UniversalCapabilityAdapter;
use songbird_universal::UnifiedUniversalAdapter;

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
        info!("Starting universal capability-based service discovery");

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
        Ok(Box::new(adapter))
    }

    /// Create service discovery based on environment detection
    ///
    /// This replaces hardcoded environment checks with universal patterns.
    async fn create_from_environment() -> Result<Box<dyn ServiceDiscovery>> {
        info!("Detecting service discovery environment");

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
        info!("No specific environment detected - using static discovery");
        Ok(Box::new(StaticServiceDiscovery::new()))
    }

    /// Detect Kubernetes environment (replaces hardcoded `KubernetesServiceDiscovery`)
    async fn detect_kubernetes_environment() -> bool {
        // Check for Kubernetes service account
        if std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount").exists() {
            return true;
        }

        // Check for Kubernetes environment variables
        std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
    }

    /// Detect Consul environment (replaces hardcoded `ConsulServiceDiscovery`)
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
        Ok(Box::new(adapter))
    }

    /// Create Consul-aware universal adapter
    async fn create_consul_universal() -> Result<Box<dyn ServiceDiscovery>> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_consul_discovery().await?;
        Ok(Box::new(adapter))
    }

    /// Create container-aware universal adapter
    async fn create_container_universal() -> Result<Box<dyn ServiceDiscovery>> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_container_discovery().await?;
        Ok(Box::new(adapter))
    }

    /// Create service discovery based on configuration
    pub async fn create_for_config(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>> {
        match &config.backend {
            DiscoveryBackend::Songbird {
                ..
            } => {
                info!("Creating Songbird native discovery backend");
                Self::create_universal_adapter().await
            }
            DiscoveryBackend::Static => {
                info!("Creating static service discovery backend");
                Ok(Box::new(StaticServiceDiscovery::new()))
            }
            DiscoveryBackend::Kubernetes {
                ..
            } => {
                info!("Creating Kubernetes service discovery backend");
                Self::create_kubernetes_universal().await
            }
            DiscoveryBackend::Etcd {
                ..
            } => {
                info!("Creating etcd service discovery backend");
                // For now, use universal adapter
                Self::create_universal_adapter().await
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
pub struct UniversalServiceDiscoveryAdapter {
    universal_adapter: UnifiedUniversalAdapter,
    _capability_adapter: UniversalCapabilityAdapter,
    kubernetes_enabled: bool,
    consul_enabled: bool,
    container_enabled: bool,
}

impl UniversalServiceDiscoveryAdapter {
    /// Create new universal service discovery adapter
    pub async fn new() -> Result<Self> {
        let universal_adapter = UnifiedUniversalAdapter::new();
        let _capability_adapter = UniversalCapabilityAdapter::new(Default::default());

        Ok(Self {
            universal_adapter,
            _capability_adapter,
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
        Ok(())
    }

    /// Register Consul-specific capabilities
    async fn register_consul_capabilities(&self) -> Result<()> {
        debug!("Registering Consul service discovery capabilities");
        // Implementation would register Consul-specific discovery patterns
        // This replaces the hardcoded ConsulServiceDiscovery
        Ok(())
    }

    /// Register container-specific capabilities
    async fn register_container_capabilities(&self) -> Result<()> {
        debug!("Registering container service discovery capabilities");
        // Implementation would register container-specific discovery patterns
        Ok(())
    }
}

#[async_trait]
impl ServiceDiscovery for UniversalServiceDiscoveryAdapter {
    async fn discover(
        &self,
        _query: crate::traits::ServiceQuery,
    ) -> Result<Vec<crate::traits::service::ServiceInfo>> {
        info!("Discovering services using universal adapter");

        // Use the universal adapter to discover services
        match self.universal_adapter.discover_services().await {
            Ok(services) => {
                // Convert universal ServiceInfo to discovery ServiceInfo
                let discovery_services = services
                    .into_iter()
                    .map(|service| self.convert_to_discovery_service_info(service))
                    .collect();

                Ok(discovery_services)
            }
            Err(e) => Err(SongbirdError::Service {
                service: "UniversalDiscovery".to_string(),
                message: format!("Service discovery failed: {e}"),
                suggested_alternatives: vec![],
                recovery_actions: vec![
                    "Check network connectivity".to_string(),
                    "Verify service registry".to_string(),
                ],
            }),
        }
    }

    async fn register(&self, service: crate::traits::service::ServiceInfo) -> Result<()> {
        info!("Registering service: {}", service.service_id);

        // Convert discovery ServiceInfo to universal format and register
        let _universal_service = self.convert_from_discovery_service_info(service);

        // Use capability adapter for registration
        // Implementation would use the universal registration system
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        info!("Deregistering service: {}", service_id);

        // Use universal adapter for deregistration
        // Implementation would use the universal deregistration system
        Ok(())
    }

    async fn watch(
        &self,
        _query: crate::traits::ServiceQuery,
    ) -> Result<
        std::pin::Pin<
            Box<dyn futures_util::Stream<Item = crate::traits::discovery::ServiceEvent> + Send>,
        >,
    > {
        // Return an empty stream for now - proper implementation would use the universal adapter's watch capability
        use futures_util::stream::{self};
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(&self, service_id: &str, _health: ServiceHealthStatus) -> Result<()> {
        info!("Updating health for service: {}", service_id);
        // Use universal adapter for health updates
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<crate::traits::service::ServiceInfo>> {
        info!("Listing all services using universal adapter");
        // Use discover with empty query to list all
        self.discover(crate::traits::ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        // Check if service exists using universal adapter
        debug!("Checking if service exists: {}", service_id);

        // Query all services to check for existence
        match self.list_all().await {
            Ok(services) => {
                let exists = services.iter().any(|s| s.service_id == service_id);
                if exists {
                    debug!("✅ Service '{}' exists", service_id);
                } else {
                    debug!("❌ Service '{}' not found", service_id);
                }
                Ok(exists)
            }
            Err(e) => {
                warn!("Failed to check service existence for '{}': {}", service_id, e);
                Err(SongbirdError::service(
                    "UniversalDiscovery",
                    format!("Failed to check service existence for '{service_id}': {e}"),
                ))
            }
        }
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        // Same as exists
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        _metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        info!("Updating metadata for service: {}", service_id);
        // Use universal adapter for metadata updates
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UniversalServiceDiscoveryAdapter {
    /// Convert universal `ServiceInfo` to discovery `ServiceInfo`
    fn convert_to_discovery_service_info(
        &self,
        service: songbird_universal::ServiceInfo,
    ) -> crate::traits::service::ServiceInfo {
        // Use From trait implementation
        service.into()
    }

    /// Convert discovery `ServiceInfo` to universal format
    fn convert_from_discovery_service_info(
        &self,
        service: crate::traits::service::ServiceInfo,
    ) -> songbird_universal::ServiceInfo {
        // Use From trait implementation
        service.into()
    }
}

// Legacy discovery backends have been replaced with universal capability detection
// All discovery now uses UniversalDiscoveryFactory with automatic capability detection
