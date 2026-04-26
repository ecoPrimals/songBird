// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(async_fn_in_trait, reason = "adapter enum dispatch uses AFIT; callers are internal")]

//! # Legacy Backend Adapters
//!
//! This module provides adapters that wrap the existing hardcoded Consul and Kubernetes
//! backends to work with the new agnostic provider system. This enables gradual migration
//! without breaking existing functionality.
//!
//! Enum dispatch ([`DiscoveryProviderImpl`], [`ProviderFactoryImpl`]) replaces `dyn` trait objects.

use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use futures_util::Stream;

use crate::abstraction::providers::{
    DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderMetadata, ServiceMetrics,
};
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::SongbirdResult;

pub mod consul_adapter;
pub mod kubernetes_adapter;
pub mod static_adapter;

pub use consul_adapter::{ConsulProviderAdapter, ConsulProviderFactory};
pub use kubernetes_adapter::{KubernetesProviderAdapter, KubernetesProviderFactory};
pub use static_adapter::{StaticProviderAdapter, StaticProviderFactory};

/// Factory for constructing [`DiscoveryProviderImpl`] values from [`ProviderConfig`].
pub trait ProviderFactory: Send + Sync {
    /// Registry key for this backend (e.g. `"static"`, `"consul"`).
    fn provider_type(&self) -> &'static str;

    /// Build a provider instance from configuration.
    async fn create_provider(
        &self,
        config: ProviderConfig,
    ) -> SongbirdResult<DiscoveryProviderImpl>;

    /// Validate configuration before creation.
    fn validate_config(&self, config: &ProviderConfig) -> SongbirdResult<()>;

    /// Default configuration for tooling and tests.
    fn default_config(&self, id: String, name: String) -> ProviderConfig;
}

/// Concrete discovery backend (enum dispatch).
pub enum DiscoveryProviderImpl {
    /// `HashiCorp` Consul.
    Consul(ConsulProviderAdapter),
    /// Kubernetes API.
    Kubernetes(KubernetesProviderAdapter),
    /// In-memory static list.
    Static(StaticProviderAdapter),
}

/// Concrete provider factory (enum dispatch).
#[derive(Debug, Clone, Copy)]
pub enum ProviderFactoryImpl {
    /// Consul factory.
    Consul(ConsulProviderFactory),
    /// Kubernetes factory.
    Kubernetes(KubernetesProviderFactory),
    /// Static factory.
    Static(StaticProviderFactory),
}

impl DiscoveryProvider for DiscoveryProviderImpl {
    fn metadata(&self) -> &ProviderMetadata {
        match self {
            Self::Consul(p) => p.metadata(),
            Self::Kubernetes(p) => p.metadata(),
            Self::Static(p) => p.metadata(),
        }
    }

    async fn initialize(&mut self, config: ProviderConfig) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.initialize(config).await,
            Self::Kubernetes(p) => p.initialize(config).await,
            Self::Static(p) => p.initialize(config).await,
        }
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.shutdown().await,
            Self::Kubernetes(p) => p.shutdown().await,
            Self::Static(p) => p.shutdown().await,
        }
    }

    async fn health_check(&self) -> SongbirdResult<bool> {
        match self {
            Self::Consul(p) => p.health_check().await,
            Self::Kubernetes(p) => p.health_check().await,
            Self::Static(p) => p.health_check().await,
        }
    }

    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.register(service).await,
            Self::Kubernetes(p) => p.register(service).await,
            Self::Static(p) => p.register(service).await,
        }
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.unregister(service_id).await,
            Self::Kubernetes(p) => p.unregister(service_id).await,
            Self::Static(p) => p.unregister(service_id).await,
        }
    }

    async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        match self {
            Self::Consul(p) => p.discover(query).await,
            Self::Kubernetes(p) => p.discover(query).await,
            Self::Static(p) => p.discover(query).await,
        }
    }

    async fn watch(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        match self {
            Self::Consul(p) => p.watch(query).await,
            Self::Kubernetes(p) => p.watch(query).await,
            Self::Static(p) => p.watch(query).await,
        }
    }

    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.update_health(service_id, health).await,
            Self::Kubernetes(p) => p.update_health(service_id, health).await,
            Self::Static(p) => p.update_health(service_id, health).await,
        }
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        match self {
            Self::Consul(p) => p.update_metadata(service_id, metadata).await,
            Self::Kubernetes(p) => p.update_metadata(service_id, metadata).await,
            Self::Static(p) => p.update_metadata(service_id, metadata).await,
        }
    }

    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        match self {
            Self::Consul(p) => p.list_all().await,
            Self::Kubernetes(p) => p.list_all().await,
            Self::Static(p) => p.list_all().await,
        }
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        match self {
            Self::Consul(p) => p.exists(service_id).await,
            Self::Kubernetes(p) => p.exists(service_id).await,
            Self::Static(p) => p.exists(service_id).await,
        }
    }

    async fn get_service_metrics(&self, service_id: &str) -> SongbirdResult<ServiceMetrics> {
        match self {
            Self::Consul(p) => p.get_service_metrics(service_id).await,
            Self::Kubernetes(p) => p.get_service_metrics(service_id).await,
            Self::Static(p) => p.get_service_metrics(service_id).await,
        }
    }

    async fn get_load_balancing_hints(
        &self,
        service_name: &str,
    ) -> SongbirdResult<LoadBalancingHints> {
        match self {
            Self::Consul(p) => p.get_load_balancing_hints(service_name).await,
            Self::Kubernetes(p) => p.get_load_balancing_hints(service_name).await,
            Self::Static(p) => p.get_load_balancing_hints(service_name).await,
        }
    }

    fn as_any(&self) -> &dyn Any {
        match self {
            Self::Consul(p) => p.as_any(),
            Self::Kubernetes(p) => p.as_any(),
            Self::Static(p) => p.as_any(),
        }
    }
}

impl ProviderFactory for ProviderFactoryImpl {
    fn provider_type(&self) -> &'static str {
        match self {
            Self::Consul(f) => ProviderFactory::provider_type(f),
            Self::Kubernetes(f) => ProviderFactory::provider_type(f),
            Self::Static(f) => ProviderFactory::provider_type(f),
        }
    }

    async fn create_provider(
        &self,
        config: ProviderConfig,
    ) -> SongbirdResult<DiscoveryProviderImpl> {
        match self {
            Self::Consul(f) => f.create_provider(config).await,
            Self::Kubernetes(f) => f.create_provider(config).await,
            Self::Static(f) => f.create_provider(config).await,
        }
    }

    fn validate_config(&self, config: &ProviderConfig) -> SongbirdResult<()> {
        match self {
            Self::Consul(f) => f.validate_config(config),
            Self::Kubernetes(f) => f.validate_config(config),
            Self::Static(f) => f.validate_config(config),
        }
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        match self {
            Self::Consul(f) => f.default_config(id, name),
            Self::Kubernetes(f) => f.default_config(id, name),
            Self::Static(f) => f.default_config(id, name),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::{
        ConsulProviderFactory, DiscoveryProviderImpl, KubernetesProviderFactory, ProviderFactory,
        ProviderFactoryImpl, StaticProviderAdapter, StaticProviderFactory,
    };
    use crate::abstraction::providers::{DiscoveryProvider, ProviderConfig};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn provider_factory_impl_dispatches_provider_type() {
        assert_eq!(ProviderFactoryImpl::Consul(ConsulProviderFactory).provider_type(), "consul");
        assert_eq!(
            ProviderFactoryImpl::Kubernetes(KubernetesProviderFactory).provider_type(),
            "kubernetes"
        );
        assert_eq!(ProviderFactoryImpl::Static(StaticProviderFactory).provider_type(), "static");
    }

    #[test]
    fn consul_validate_config_requires_url() {
        let factory = ProviderFactoryImpl::Consul(ConsulProviderFactory);
        let config = ProviderConfig {
            id: "c".into(),
            name: "n".into(),
            parameters: HashMap::new(),
            environment: HashMap::new(),
            timeout_ms: None,
            retry_config: None,
        };
        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn consul_validate_config_rejects_non_http_url() {
        let factory = ProviderFactoryImpl::Consul(ConsulProviderFactory);
        let mut parameters = HashMap::new();
        parameters.insert("url".into(), json!("ftp://consul:8500"));
        let config = ProviderConfig {
            id: "c".into(),
            name: "n".into(),
            parameters,
            environment: HashMap::new(),
            timeout_ms: None,
            retry_config: None,
        };
        assert!(factory.validate_config(&config).is_err());
    }

    #[test]
    fn static_and_kubernetes_validate_config_succeeds() {
        let empty = ProviderConfig {
            id: "x".into(),
            name: "y".into(),
            parameters: HashMap::new(),
            environment: HashMap::new(),
            timeout_ms: None,
            retry_config: None,
        };
        assert!(ProviderFactoryImpl::Static(StaticProviderFactory).validate_config(&empty).is_ok());
        assert!(
            ProviderFactoryImpl::Kubernetes(KubernetesProviderFactory)
                .validate_config(&empty)
                .is_ok()
        );
    }

    #[test]
    fn default_config_carries_ids_for_each_factory() {
        let id = "my-id".to_string();
        let name = "my-name".to_string();
        for factory in [
            ProviderFactoryImpl::Consul(ConsulProviderFactory),
            ProviderFactoryImpl::Kubernetes(KubernetesProviderFactory),
            ProviderFactoryImpl::Static(StaticProviderFactory),
        ] {
            let cfg = factory.default_config(id.clone(), name.clone());
            assert_eq!(cfg.id, id);
            assert_eq!(cfg.name, name);
        }
    }

    #[tokio::test]
    async fn discovery_provider_impl_as_any_round_trip_for_static() {
        let adapter = StaticProviderAdapter::new_native("aid".into(), vec![]);
        let provider = DiscoveryProviderImpl::Static(adapter);
        let any_ref = provider.as_any();
        assert!(any_ref.is::<StaticProviderAdapter>());
    }
}
