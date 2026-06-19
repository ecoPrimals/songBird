// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Discovery Delegation
//!
//! Routes discovery requests to capable providers without hard-coding

use futures_util::Stream;
use std::collections::HashMap;
use std::pin::Pin;

use crate::abstraction::capabilities::{CapabilityMatcher, CapabilityQuery, DiscoveryCapability};
use crate::abstraction::providers::{LoadBalancingHints, ServiceMetrics};
use crate::abstraction::registry::ProviderRegistry;
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::{SongbirdError, SongbirdResult};

/// Delegation strategy for choosing providers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationStrategy {
    /// Use the first available provider
    FirstAvailable,
    /// Use the provider with the lowest load
    LeastLoad,
    /// Use the provider with the highest capability score
    BestMatch,
    /// Round-robin between available providers
    RoundRobin,
    /// Use all providers and merge results
    Broadcast,
    /// Custom strategy with provider ID
    Specific(String),
}

/// Discovery delegator that routes requests to providers
pub struct DiscoveryDelegator {
    registry: ProviderRegistry,
    default_strategy: DelegationStrategy,
    round_robin_state: std::sync::Arc<tokio::sync::RwLock<HashMap<String, usize>>>,
}

impl DiscoveryDelegator {
    /// Create a new discovery delegator
    #[must_use]
    pub fn new(registry: ProviderRegistry) -> Self {
        Self {
            registry,
            default_strategy: DelegationStrategy::BestMatch,
            round_robin_state: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Set the default delegation strategy
    #[must_use]
    pub fn with_strategy(mut self, strategy: DelegationStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Current default delegation strategy.
    #[must_use]
    pub fn strategy(&self) -> &DelegationStrategy {
        &self.default_strategy
    }

    /// Register a service using delegation
    pub async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceRegistration),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_register_service(&provider_id, service).await
    }

    /// Unregister a service using delegation
    pub async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceUnregistration),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_unregister_service(&provider_id, service_id).await
    }

    /// Discover services using delegation
    pub async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceDiscovery),
        );

        if self.default_strategy == DelegationStrategy::Broadcast {
            self.broadcast_discover_services(query).await
        } else {
            let provider_id =
                self.select_provider(&capability_query, &self.default_strategy).await?;
            self.delegate_discover_services(&provider_id, query).await
        }
    }

    /// Watch services using delegation
    pub async fn watch(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceWatching),
        );

        let provider_id = self.select_provider(&capability_query, &self.default_strategy).await?;
        self.delegate_watch_services(&provider_id, query).await
    }

    /// Update service health using delegation
    pub async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::HealthChecking),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_update_service_health(&provider_id, service_id, health).await
    }

    /// List all services using delegation
    pub async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceListing),
        );

        if self.default_strategy == DelegationStrategy::Broadcast {
            self.broadcast_list_all_services().await
        } else {
            let provider_id = self.select_provider(&query, &self.default_strategy).await?;
            self.delegate_list_all_services(&provider_id).await
        }
    }

    /// Check if service exists using delegation
    pub async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceExistence),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_service_exists(&provider_id, service_id).await
    }

    /// Get service metrics using delegation
    pub async fn get_service_metrics(&self, service_id: &str) -> SongbirdResult<ServiceMetrics> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceMetrics),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_get_service_metrics(&provider_id, service_id).await
    }

    /// Get load balancing hints using delegation
    pub async fn get_load_balancing_hints(
        &self,
        service_name: &str,
    ) -> SongbirdResult<LoadBalancingHints> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::LoadBalancingHints),
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_get_load_balancing_hints(&provider_id, service_name).await
    }

    async fn select_provider(
        &self,
        query: &CapabilityQuery,
        strategy: &DelegationStrategy,
    ) -> SongbirdResult<String> {
        match strategy {
            DelegationStrategy::FirstAvailable => {
                let providers = self.registry.find_providers(query).await?;
                providers
                    .into_iter()
                    .next()
                    .ok_or_else(|| SongbirdError::discovery("No providers available"))
            }
            DelegationStrategy::BestMatch => self.registry.get_best_provider(query).await,
            DelegationStrategy::LeastLoad => {
                let providers = self.registry.find_providers(query).await?;
                let mut best_provider = None;
                let mut best_load = f64::INFINITY;

                for provider_id in providers {
                    if let Ok(metadata) = self.registry.get_provider_metadata(&provider_id).await {
                        if metadata.load_score < best_load {
                            best_load = metadata.load_score;
                            best_provider = Some(provider_id);
                        }
                    }
                }

                best_provider.ok_or_else(|| SongbirdError::discovery("No providers available"))
            }
            DelegationStrategy::RoundRobin => {
                let providers = self.registry.find_providers(query).await?;
                if providers.is_empty() {
                    return Err(SongbirdError::discovery("No providers available"));
                }

                let key = format!("{:?}", query.matcher.required);
                let mut state = self.round_robin_state.write().await;
                let index = state.entry(key).or_insert(0);
                let selected = providers[*index % providers.len()].clone();
                *index += 1;

                Ok(selected)
            }
            DelegationStrategy::Specific(provider_id) => {
                let metadata = self.registry.get_provider_metadata(provider_id).await?;
                if query.matcher.matches(&metadata.capabilities) {
                    Ok(provider_id.clone())
                } else {
                    Err(SongbirdError::registry(
                        format!("Provider '{provider_id}' does not have required capabilities"),
                        "select_provider",
                    ))
                }
            }
            DelegationStrategy::Broadcast => {
                let providers = self.registry.find_providers(query).await?;
                providers
                    .into_iter()
                    .next()
                    .ok_or_else(|| SongbirdError::discovery("No providers available"))
            }
        }
    }

    async fn broadcast_discover_services(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceDiscovery),
        );

        let providers = self.registry.find_providers(&capability_query).await?;
        let mut all_services = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for provider_id in providers {
            if let Ok(services) = self.delegate_discover_services(&provider_id, query.clone()).await
            {
                for service in services {
                    if seen_ids.insert(service.service_id.clone()) {
                        all_services.push(service);
                    }
                }
            }
        }

        Ok(all_services)
    }

    async fn broadcast_list_all_services(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceListing),
        );

        let providers = self.registry.find_providers(&query).await?;
        let mut all_services = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for provider_id in providers {
            if let Ok(services) = self.delegate_list_all_services(&provider_id).await {
                for service in services {
                    if seen_ids.insert(service.service_id.clone()) {
                        all_services.push(service);
                    }
                }
            }
        }

        Ok(all_services)
    }

    async fn delegate_register_service(
        &self,
        provider_id: &str,
        service: ServiceInfo,
    ) -> SongbirdResult<()> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for service '{}'",
            service.service_id
        )))
    }

    async fn delegate_unregister_service(
        &self,
        provider_id: &str,
        service_id: &str,
    ) -> SongbirdResult<()> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for service '{service_id}'"
        )))
    }

    async fn delegate_discover_services(
        &self,
        provider_id: &str,
        _query: ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter"
        )))
    }

    async fn delegate_watch_services(
        &self,
        provider_id: &str,
        _query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for watch"
        )))
    }

    async fn delegate_update_service_health(
        &self,
        provider_id: &str,
        service_id: &str,
        _health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for health update on '{service_id}'"
        )))
    }

    async fn delegate_list_all_services(
        &self,
        provider_id: &str,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for listing"
        )))
    }

    async fn delegate_service_exists(
        &self,
        provider_id: &str,
        service_id: &str,
    ) -> SongbirdResult<bool> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for existence check on '{service_id}'"
        )))
    }

    async fn delegate_get_service_metrics(
        &self,
        provider_id: &str,
        service_id: &str,
    ) -> SongbirdResult<ServiceMetrics> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for metrics on '{service_id}'"
        )))
    }

    async fn delegate_get_load_balancing_hints(
        &self,
        provider_id: &str,
        service_name: &str,
    ) -> SongbirdResult<LoadBalancingHints> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \
             Provider '{provider_id}' should be accessed via UniversalCapabilityAdapter for load hints on '{service_name}'"
        )))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::abstraction::adapters::{
        ConsulProviderAdapter, DiscoveryProviderImpl, StaticProviderAdapter,
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_delegation_strategy() {
        let registry = ProviderRegistry::new();
        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::LeastLoad);

        assert_eq!(delegator.strategy(), &DelegationStrategy::LeastLoad);
    }

    #[test]
    fn test_delegation_strategy_equality() {
        assert_eq!(DelegationStrategy::FirstAvailable, DelegationStrategy::FirstAvailable);
        assert_ne!(DelegationStrategy::FirstAvailable, DelegationStrategy::LeastLoad);
        assert_eq!(
            DelegationStrategy::Specific("test".to_string()),
            DelegationStrategy::Specific("test".to_string())
        );
    }

    #[tokio::test]
    async fn discover_fails_when_registry_empty() {
        let delegator = DiscoveryDelegator::new(ProviderRegistry::new());
        let err = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err.to_string().contains("No providers") || err.to_string().contains("providers"));
    }

    #[tokio::test]
    async fn discover_returns_deprecated_delegation_error_when_provider_registered() {
        let registry = ProviderRegistry::new();
        let provider = DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
            "p-static".into(),
            vec![],
        ));
        registry.register_provider(provider).await.unwrap();

        let delegator = DiscoveryDelegator::new(registry);
        let err = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err.to_string().contains("UniversalCapabilityAdapter"));
    }

    #[tokio::test]
    async fn broadcast_discover_swallows_delegate_errors_and_returns_merged_ok() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "a".into(),
                vec![],
            )))
            .await
            .unwrap();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "b".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::Broadcast);
        let services = delegator.discover(ServiceQuery::new()).await.unwrap();
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn broadcast_list_all_returns_empty_when_delegate_errors() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "only".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::Broadcast);
        let services = delegator.list_all().await.unwrap();
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn specific_strategy_rejects_provider_missing_required_capability() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "no-watch".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator = DiscoveryDelegator::new(registry)
            .with_strategy(DelegationStrategy::Specific("no-watch".into()));
        match delegator.watch(ServiceQuery::new()).await {
            Err(e) => assert!(e.to_string().contains("does not have required capabilities")),
            Ok(_) => panic!("expected registry error for missing ServiceWatching capability"),
        }
    }

    #[tokio::test]
    async fn round_robin_rotates_between_two_providers() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "rr-a".into(),
                vec![],
            )))
            .await
            .unwrap();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "rr-b".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::RoundRobin);
        let err1 = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        let err2 = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err1.to_string().contains("rr-a") || err1.to_string().contains("rr-b"));
        assert!(err2.to_string().contains("rr-a") || err2.to_string().contains("rr-b"));
        assert_ne!(err1.to_string(), err2.to_string());
    }

    async fn registry_with_static_and_consul() -> ProviderRegistry {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "static-low-load".into(),
                vec![],
            )))
            .await
            .unwrap();
        let consul = ConsulProviderAdapter::new_native(
            "consul-high-load".into(),
            "http://127.0.0.1:8500".into(),
        )
        .await
        .expect("create consul adapter");
        registry.register_provider(DiscoveryProviderImpl::Consul(consul)).await.unwrap();
        registry
    }

    #[tokio::test]
    async fn default_strategy_is_best_match() {
        let delegator = DiscoveryDelegator::new(ProviderRegistry::new());
        assert_eq!(delegator.strategy(), &DelegationStrategy::BestMatch);
    }

    #[tokio::test]
    async fn first_available_selects_a_registered_provider() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "first-pick".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::FirstAvailable);
        let err = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err.to_string().contains("first-pick"));
    }

    #[tokio::test]
    async fn least_load_prefers_static_over_consul() {
        let registry = registry_with_static_and_consul().await;
        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::LeastLoad);
        let err = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err.to_string().contains("static-low-load"));
    }

    #[tokio::test]
    async fn register_fails_when_registry_empty() {
        let delegator = DiscoveryDelegator::new(ProviderRegistry::new());
        let service = ServiceInfo {
            service_id: "svc".into(),
            name: "Test".into(),
            version: "1".into(),
            service_type: "api".into(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: crate::traits::service::ServiceStatus::Running,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            instance_id: "svc-1".into(),
            host: "127.0.0.1".into(),
            port: 8080,
        };
        let err = delegator.register(service).await.unwrap_err();
        assert!(err.to_string().contains("providers") || err.to_string().contains("Providers"));
    }

    #[tokio::test]
    async fn list_all_fails_when_registry_empty() {
        let delegator = DiscoveryDelegator::new(ProviderRegistry::new());
        let err = delegator.list_all().await.unwrap_err();
        assert!(err.to_string().contains("providers") || err.to_string().contains("Providers"));
    }

    #[tokio::test]
    async fn exists_returns_deprecated_error_when_provider_registered() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "exists-provider".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator = DiscoveryDelegator::new(registry);
        let err = delegator.exists("any-id").await.unwrap_err();
        assert!(err.to_string().contains("UniversalCapabilityAdapter"));
    }

    #[tokio::test]
    async fn specific_strategy_accepts_provider_with_discovery_capability() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "capable".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator = DiscoveryDelegator::new(registry)
            .with_strategy(DelegationStrategy::Specific("capable".into()));
        let err = delegator.discover(ServiceQuery::new()).await.unwrap_err();
        assert!(err.to_string().contains("capable"));
    }

    #[tokio::test]
    async fn get_metrics_fails_when_all_providers_missing_capability() {
        let registry = ProviderRegistry::new();
        registry
            .register_provider(DiscoveryProviderImpl::Static(StaticProviderAdapter::new_native(
                "no-metrics".into(),
                vec![],
            )))
            .await
            .unwrap();

        let delegator = DiscoveryDelegator::new(registry);
        let err = delegator.get_service_metrics("svc").await.unwrap_err();
        assert!(
            err.to_string().contains("ServiceMetrics")
                || err.to_string().contains("capabilities")
                || err.to_string().contains("providers")
        );
    }

    #[tokio::test]
    async fn round_robin_three_providers_produces_three_distinct_errors() {
        let registry = ProviderRegistry::new();
        for id in ["rr-x", "rr-y", "rr-z"] {
            registry
                .register_provider(DiscoveryProviderImpl::Static(
                    StaticProviderAdapter::new_native(id.into(), vec![]),
                ))
                .await
                .unwrap();
        }

        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::RoundRobin);
        let mut messages = Vec::new();
        for _ in 0..3 {
            messages.push(delegator.discover(ServiceQuery::new()).await.unwrap_err().to_string());
        }
        assert_ne!(messages[0], messages[1]);
        assert_ne!(messages[1], messages[2]);
    }
}
