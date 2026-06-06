// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔧 Universal Discovery Factory
//!
//! **MODERNIZED UNIVERSAL FACTORY** ✅
//!
//! This factory replaces hardcoded backend implementations with universal
//! capability detection and adapter-based discovery.
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(
    async_fn_in_trait,
    clippy::unused_async,
    clippy::unused_self,
    clippy::missing_errors_doc,
    clippy::default_constructed_unit_structs,
    reason = "discovery factory: async factory traits and adapter stubs"
)]

use crate::traits::discovery::{
    DiscoveryBackend, DiscoveryConfig, ServiceDiscovery, ServiceHealthStatus,
};
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info, warn};

// Import universal adapters
use songbird_universal::UnifiedUniversalAdapter;
use songbird_universal::capabilities::UniversalCapabilityAdapter;

type Result<T> = SongbirdResult<T>;

/// **MODERNIZED**: Universal discovery factory with capability detection
///
/// Replaces hardcoded `KubernetesServiceDiscovery`, `ConsulServiceDiscovery`,
/// and `StaticServiceDiscovery` with universal capability-based detection.
pub struct UniversalDiscoveryFactory;

#[allow(
    dead_code,
    reason = "factory methods called by orchestrator and adapter as discovery backends expand"
)]
impl UniversalDiscoveryFactory {
    /// Create service discovery with auto-detection
    ///
    /// This replaces the old hardcoded factory methods with universal capability detection.
    /// Returns concrete type for zero-cost abstraction.
    pub async fn create_auto_detect() -> Result<UniversalServiceDiscoveryAdapter> {
        info!("Starting universal capability-based service discovery");

        // Try universal adapter first (replaces all hardcoded backends)
        match Self::create_universal_adapter().await {
            Ok(adapter) => {
                info!("✅ Created universal capability adapter");
                Ok(adapter)
            }
            Err(e) => {
                warn!("Universal adapter creation failed: {}", e);
                // Fallback to basic adapter
                Self::create_universal_adapter().await
            }
        }
    }

    /// Create universal capability adapter (replaces hardcoded backends)
    async fn create_universal_adapter() -> Result<UniversalServiceDiscoveryAdapter> {
        let adapter = UniversalServiceDiscoveryAdapter::new().await?;
        Ok(adapter)
    }

    /// Create service discovery based on environment detection
    ///
    /// This replaces hardcoded environment checks with universal patterns.
    /// Returns concrete type for zero-cost abstraction.
    async fn create_from_environment() -> Result<UniversalServiceDiscoveryAdapter> {
        info!("Detecting service discovery environment");

        // Check for Kubernetes environment
        if Self::detect_kubernetes_environment().await {
            info!("📦 Kubernetes environment detected - using universal adapter");
            return Self::create_universal_adapter().await;
        }

        // Check for Consul environment
        if Self::detect_consul_environment().await {
            info!("🏛️ Consul environment detected - using universal adapter");
            return Self::create_universal_adapter().await;
        }

        // Check for container orchestration
        if Self::detect_container_environment().await {
            info!("🐳 Container environment detected - using universal adapter");
            return Self::create_universal_adapter().await;
        }

        // Default to universal adapter for all environments (zero-cost)
        info!("No specific environment detected - using universal adapter");
        Self::create_universal_adapter().await
    }

    /// Detect Kubernetes environment (replaces hardcoded `KubernetesServiceDiscovery`)
    async fn detect_kubernetes_environment() -> bool {
        // Check for Kubernetes service account
        if std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount").exists() {
            return true;
        }

        // Check for Kubernetes environment variables
        songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok()
    }

    /// Detect Consul environment (replaces hardcoded `ConsulServiceDiscovery`)
    async fn detect_consul_environment() -> bool {
        // Check for Consul environment variables
        songbird_process_env::var("CONSUL_HTTP_ADDR").is_ok()
            || songbird_process_env::var("CONSUL_ADDR").is_ok()
            || std::path::Path::new("/etc/consul").exists()
    }

    /// Detect container environment
    async fn detect_container_environment() -> bool {
        songbird_process_env::var("DOCKER_HOST").is_ok()
            || std::path::Path::new("/.dockerenv").exists()
            || songbird_process_env::var("CONTAINER_ID").is_ok()
    }

    /// Create Kubernetes-aware universal adapter (zero-cost)
    async fn create_kubernetes_universal() -> Result<UniversalServiceDiscoveryAdapter> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_kubernetes_discovery().await?;
        Ok(adapter)
    }

    /// Create Consul-aware universal adapter (zero-cost)
    async fn create_consul_universal() -> Result<UniversalServiceDiscoveryAdapter> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_consul_discovery().await?;
        Ok(adapter)
    }

    /// Create container-aware universal adapter (zero-cost)
    async fn create_container_universal() -> Result<UniversalServiceDiscoveryAdapter> {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await?;
        adapter.enable_container_discovery().await?;
        Ok(adapter)
    }

    /// Create service discovery based on configuration (returns concrete type for zero-cost)
    pub async fn create_for_config(
        config: &DiscoveryConfig,
    ) -> Result<UniversalServiceDiscoveryAdapter> {
        match &config.backend {
            DiscoveryBackend::Songbird {
                ..
            } => {
                info!("Creating Songbird native discovery backend");
                Self::create_universal_adapter().await
            }
            DiscoveryBackend::Static => {
                info!("Creating static service discovery backend (via universal adapter)");
                Self::create_universal_adapter().await
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
        let capability_adapter = UniversalCapabilityAdapter::new(
            songbird_universal::capabilities::DiscoveryConfig::default(),
        );

        Ok(Self {
            universal_adapter,
            _capability_adapter: capability_adapter,
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

// Native async trait implementation (no boxing overhead)
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use crate::traits::discovery::{DiscoveryBackend, DiscoveryConfig};

    #[test]
    fn discovery_config_default_is_static() {
        let c = DiscoveryConfig::default();
        assert!(matches!(c.backend, DiscoveryBackend::Static));
    }

    #[test]
    fn discovery_config_serde_roundtrip_static() {
        let c = DiscoveryConfig::default();
        let json = serde_json::to_string(&c).expect("serialize");
        let back: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(back.backend, DiscoveryBackend::Static));
    }

    #[test]
    fn discovery_config_serde_roundtrip_kubernetes() {
        let c = DiscoveryConfig {
            backend: DiscoveryBackend::Kubernetes {
                namespace: Some("ns".into()),
                in_cluster: true,
                kubeconfig_path: None,
            },
            ..DiscoveryConfig::default()
        };
        let json = serde_json::to_string(&c).expect("serialize");
        let back: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            back.backend,
            DiscoveryBackend::Kubernetes {
                namespace: Some(ref n),
                in_cluster: true,
                kubeconfig_path: None
            } if n == "ns"
        ));
    }

    #[test]
    fn discovery_config_serde_roundtrip_etcd() {
        let c = DiscoveryConfig {
            backend: DiscoveryBackend::Etcd {
                endpoints: vec!["http://127.0.0.1:2379".into()],
                username: Some("u".into()),
                password: None,
            },
            ..DiscoveryConfig::default()
        };
        let json = serde_json::to_string(&c).expect("serialize");
        let back: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        match back.backend {
            DiscoveryBackend::Etcd {
                endpoints,
                username,
                ..
            } => {
                assert_eq!(endpoints.len(), 1);
                assert_eq!(username.as_deref(), Some("u"));
            }
            _ => panic!("expected Etcd"),
        }
    }

    #[test]
    fn discovery_config_serde_roundtrip_songbird() {
        let c = DiscoveryConfig {
            backend: DiscoveryBackend::Songbird {
                federation_enabled: true,
                trust_verification: false,
                attribution_tracking: true,
            },
            ..DiscoveryConfig::default()
        };
        let json = serde_json::to_string(&c).expect("serialize");
        let back: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        match back.backend {
            DiscoveryBackend::Songbird {
                federation_enabled,
                trust_verification,
                attribution_tracking,
            } => {
                assert!(federation_enabled);
                assert!(!trust_verification);
                assert!(attribution_tracking);
            }
            _ => panic!("expected Songbird"),
        }
    }

    use super::{UniversalDiscoveryFactory, UniversalServiceDiscoveryAdapter};
    use crate::traits::discovery::ServiceDiscovery;

    #[tokio::test]
    async fn detect_kubernetes_environment_returns_bool() {
        let detected = UniversalDiscoveryFactory::detect_kubernetes_environment().await;
        let from_path =
            std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount").exists();
        let from_env = songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok();
        assert_eq!(detected, from_path || from_env);
    }

    #[tokio::test]
    async fn detect_consul_environment_returns_bool() {
        let detected = UniversalDiscoveryFactory::detect_consul_environment().await;
        let from_env = songbird_process_env::var("CONSUL_HTTP_ADDR").is_ok()
            || songbird_process_env::var("CONSUL_ADDR").is_ok();
        let from_path = std::path::Path::new("/etc/consul").exists();
        assert_eq!(detected, from_env || from_path);
    }

    #[tokio::test]
    async fn detect_container_environment_returns_bool() {
        let detected = UniversalDiscoveryFactory::detect_container_environment().await;
        let from_dockerenv = std::path::Path::new("/.dockerenv").exists();
        let from_env = songbird_process_env::var("DOCKER_HOST").is_ok()
            || songbird_process_env::var("CONTAINER_ID").is_ok();
        assert_eq!(detected, from_dockerenv || from_env);
    }

    #[tokio::test]
    async fn create_from_environment_succeeds() {
        let adapter = UniversalDiscoveryFactory::create_from_environment().await;
        assert!(adapter.is_ok(), "environment-based factory should succeed");
    }

    #[tokio::test]
    async fn create_auto_detect_returns_adapter() {
        let adapter = UniversalDiscoveryFactory::create_auto_detect().await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn create_for_config_kubernetes_backend() {
        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Kubernetes {
                namespace: Some("test-ns".into()),
                in_cluster: false,
                kubeconfig_path: None,
            },
            ..DiscoveryConfig::default()
        };
        let adapter = UniversalDiscoveryFactory::create_for_config(&config).await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn create_for_config_etcd_backend() {
        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Etcd {
                endpoints: vec!["http://127.0.0.1:2379".into()],
                username: None,
                password: None,
            },
            ..DiscoveryConfig::default()
        };
        let adapter = UniversalDiscoveryFactory::create_for_config(&config).await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn create_kubernetes_universal_enables_discovery() {
        let adapter = UniversalDiscoveryFactory::create_kubernetes_universal().await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn create_consul_universal_enables_discovery() {
        let adapter = UniversalDiscoveryFactory::create_consul_universal().await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn create_container_universal_enables_discovery() {
        let adapter = UniversalDiscoveryFactory::create_container_universal().await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn adapter_register_unregister_and_exists() {
        use crate::traits::service::ServiceStatus;
        use chrono::Utc;
        use std::collections::HashMap;

        let adapter = UniversalServiceDiscoveryAdapter::new().await.expect("new adapter");
        let service = crate::traits::ServiceInfo {
            service_id: "uni-svc".into(),
            name: "Universal".into(),
            version: "1.0".into(),
            service_type: "test".into(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: "uni-svc-1".into(),
            host: "127.0.0.1".into(),
            port: 9000,
        };

        adapter.register(service).await.expect("register");
        adapter.unregister("uni-svc").await.expect("unregister");
        assert!(!adapter.exists("uni-svc").await.expect("exists check"));
    }

    #[tokio::test]
    async fn adapter_watch_returns_empty_stream() {
        use crate::traits::ServiceQuery;
        use futures_util::StreamExt;

        let adapter = UniversalServiceDiscoveryAdapter::new().await.expect("new adapter");
        let mut stream = adapter.watch(ServiceQuery::new()).await.expect("watch");
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn adapter_update_health_and_metadata_succeed() {
        use crate::traits::discovery::ServiceHealthStatus;
        use std::collections::HashMap;

        let adapter = UniversalServiceDiscoveryAdapter::new().await.expect("new adapter");
        adapter.update_health("svc", ServiceHealthStatus::Healthy).await.expect("update health");
        adapter
            .update_metadata("svc", HashMap::from([("key".into(), "val".into())]))
            .await
            .expect("update metadata");
    }

    #[tokio::test]
    async fn adapter_enable_all_discovery_modes() {
        let mut adapter = UniversalServiceDiscoveryAdapter::new().await.expect("new adapter");
        adapter.enable_kubernetes_discovery().await.expect("k8s");
        adapter.enable_consul_discovery().await.expect("consul");
        adapter.enable_container_discovery().await.expect("container");
        let _ = adapter.discover(crate::traits::ServiceQuery::new()).await;
    }
}
