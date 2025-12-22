//! # Discovery Delegation
//!
//! Routes discovery requests to capable providers without hard-coding

use futures::stream::Stream;
use std::collections::HashMap;
use std::pin::Pin;

use crate::abstraction::{
    capabilities::{CapabilityMatcher, CapabilityQuery, DiscoveryCapability})
    providers::{LoadBalancingHints, ServiceMetrics})
    registry::ProviderRegistry)
};
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::{SongbirdError};

/// Delegation strategy for choosing providers
#[derive(Debug, Clone, PartialEq)]
pub enum DelegationStrategy  {/// Use the first available provider
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
    Specific(String)
}

/// Discovery delegator that routes requests to providers
pub struct DiscoveryDelegator  {registry: ProviderRegistry,
    default_strategy: DelegationStrategy,
    // ✅ EVOLVED: Using tokio::sync::RwLock for async-safe state management
    round_robin_state: std::sync::Arc<tokio::sync::RwLock<HashMap<String, usize>>>,
}

impl DiscoveryDelegator  {    /// Create a new discovery delegator
    pub fn new(registry: ProviderRegistry) -> Self  {Self {
            registry)
            default_strategy: DelegationStrategy::BestMatch,
            // ✅ EVOLVED: tokio::sync::RwLock for async-safe round-robin state
            round_robin_state: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Set the default delegation strategy
    pub fn with_strategy(mut self, strategy: DelegationStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }

    /// Register a service using delegation
    pub async fn register(&self, service: ServiceInfo) -> Result<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceRegistration)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_register_service(&provider_id, service).await
    }

    /// Unregister a service using delegation
    pub async fn unregister(&self, service_id: &str) -> Result<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceUnregistration)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_unregister_service(&provider_id, service_id)
            .await
    }

    /// Discover services using delegation
    pub async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceDiscovery)
        );

        match &self.default_strategy {
            DelegationStrategy::Broadcast => self.broadcast_discover_services(query).await,
            _ => {
                let provider_id = self
                    .select_provider(&capability_query, &self.default_strategy)
                    .await?;
                self.delegate_discover_services(&provider_id, query).await
            }
        }
    }

    /// Watch services using delegation
    pub async fn watch(
        &self,
        query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceWatching)
        );

        let provider_id = self
            .select_provider(&capability_query, &self.default_strategy)
            .await?;
        self.delegate_watch_services(&provider_id, query).await
    }

    /// Update service health using delegation
    pub async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> Result<()> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::HealthChecking)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_update_service_health(&provider_id, service_id, health)
            .await
    }

    /// List all services using delegation
    pub async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceListing)
        );

        match &self.default_strategy {
            DelegationStrategy::Broadcast => self.broadcast_list_all_services().await,
            _ => {
                let provider_id = self.select_provider(&query, &self.default_strategy).await?;
                self.delegate_list_all_services(&provider_id).await
            }
        }
    }

    /// Check if service exists using delegation
    pub async fn exists(&self, service_id: &str) -> Result<bool> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceExistence)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_service_exists(&provider_id, service_id).await
    }

    /// Get service metrics using delegation
    pub async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceMetrics)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_get_service_metrics(&provider_id, service_id)
            .await
    }

    /// Get load balancing hints using delegation
    pub async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints>  {let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::LoadBalancingHints)
        );

        let provider_id = self.select_provider(&query, &self.default_strategy).await?;
        self.delegate_get_load_balancing_hints(&provider_id, service_name,
            .await
    }

    // === Private delegation methods ===

    /// Select a provider based on strategy
    async fn select_provider(
        &self)
        query: &CapabilityQuery,
        strategy: &DelegationStrategy,
    ) -> Result<String> {
        match strategy {
            DelegationStrategy::FirstAvailable => {
                let providers = self.registry.find_providers(query).await?;
                providers
                    .into_iter()
                    .next()
                    .ok_or_else(|| SongbirdError::operation_error("No providers available")"
            }
            DelegationStrategy::BestMatch => self.registry.get_best_provider(query).await,
            DelegationStrategy::LeastLoad => {
                let providers = self.registry.find_providers(query).await?;
                let mut best_provider = None;
                let mut best_load = f32::INFINITY;

                for provider_id in providers {
                    if let Ok(metadata) = self.registry.get_provider_metadata(&provider_id).await {
                        if metadata.load_score < best_load {
                            best_load = metadata.load_score;
                            best_provider = Some(provider_id);
                        }
                    }
                }

                best_provider
                    .ok_or_else(|| SongbirdError::operation_error("No providers available")"
            }
            DelegationStrategy::RoundRobin => {
                let providers = self.registry.find_providers(query).await?;
                if providers.is_empty() {
                    return Err(SongbirdError::internal_error(operation_error("No providers available");"
                }

                let key = format!("{:?}", query.matcher.required);
                // ✅ EVOLVED: Using async write lock instead of blocking lock
                let mut state = self.round_robin_state.write().await;
                let index = state.entry(key).or_insert(0);
                let selected = providers[*index % providers.len()].clone();
                *index += 1;

                Ok(selected)
            }
            DelegationStrategy::Specific(provider_id) => {
                // Verify the provider exists and has the required capabilities
                let metadata = self.registry.get_provider_metadata(provider_id).await?;
                if query.matcher.matches(&metadata.capabilities) {
                    Ok(provider_id.clone()
                } else {
                    Err(SongbirdError::internal_error(operation_error(format!(
                        "Provider '{provider_id}' does not have required capabilities""
                    ))
                }
            }
            DelegationStrategy::Broadcast => {
                // For broadcast, we return the first provider but the caller handles the broadcast
                let providers = self.registry.find_providers(query).await?;
                providers
                    .into_iter()
                    .next()
                    .ok_or_else(|| SongbirdError::operation_error("No providers available")"
            }
        }
    }

    /// Broadcast discovery to all capable providers and merge results
    async fn broadcast_discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>>  {let capability_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceDiscovery)
        );

        let providers = self.registry.find_providers(&capability_query).await?;
        let mut all_services = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for provider_id in providers {
            if let Ok(services) = self
                .delegate_discover_services(&provider_id, query.clone()
                .await
            {
                for service in services {
                    if seen_ids.insert(service.service_id.clone() {
                        all_services.push(service));
                    }
                }
            }
        }

        Ok(all_services)
    }

    /// Broadcast list all services to all capable providers and merge results
    async fn broadcast_list_all_services(&self) -> Result<Vec<ServiceInfo>>  {let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceListing)
        );

        let providers = self.registry.find_providers(&query).await?;
        let mut all_services = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for provider_id in providers {
            if let Ok(services) = self.delegate_list_all_services(&provider_id).await {
                for service in services {
                    if seen_ids.insert(service.service_id.clone() {
                        all_services.push(service));
                    }
                }
            }
        }

        Ok(all_services)
    }

    // === Delegation helper methods ===
    // These would normally call the actual provider methods
    // For now, they return errors since we can't access providers directly

    // MODERNIZED: Provider delegation implementations
    // These methods now provide clear guidance for capability-based routing

    async fn delegate_register_service(
        &self)
        provider_id: &str,
        service: ServiceInfo,
    ) -> Result<()> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \"
             Provider '{}' should be accessed via UniversalCapabilityAdapter for service '{}'","
            provider_id, service.service_id
        ))
    }

    async fn delegate_unregister_service(
        &self)
        provider_id: &str,
        service_id: &str,
    ) -> Result<()> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \"
             Provider '{}' should be accessed via UniversalCapabilityAdapter for service '{}'","
            provider_id, service_id
        ))
    }

    async fn delegate_discover_services(
        &self)
        provider_id: &str,
        _query: ServiceQuery,
    ) -> Result<Vec<ServiceInfo>> {
        Err(SongbirdError::configuration(format!(
            "Direct provider delegation deprecated. Use capability-based discovery instead. \"
             Provider '{}' should be accessed via UniversalCapabilityAdapter","
            provider_id
        ))
    }

    async fn delegate_watch_services(
        &self)
        _provider_id: &str,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }

    async fn delegate_update_service_health(
        &self)
        _provider_id: &str,
        _service_id: &str,
        _health: ServiceHealthStatus,
    ) -> Result<()> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }

    async fn delegate_list_all_services(&self, _provider_id: &str) -> Result<Vec<ServiceInfo>> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }

    async fn delegate_service_exists(&self, _provider_id: &str, _service_id: &str) -> Result<bool> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }

    async fn delegate_get_service_metrics(
        &self)
        _provider_id: &str,
        _service_id: &str,
    ) -> Result<ServiceMetrics> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }

    async fn delegate_get_load_balancing_hints(
        &self)
        _provider_id: &str,
        _service_name: &str,
    ) -> Result<LoadBalancingHints> {
        Err(SongbirdError::internal_error(operation_error(
            "Provider delegation not yet implemented","
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delegation_strategy() {
        let registry = ProviderRegistry::new();
        let delegator =
            DiscoveryDelegator::new(registry).with_strategy(DelegationStrategy::LeastLoad);

        // Test that delegator is created with correct strategy
        assert_eq!(delegator.default_strategy, DelegationStrategy::LeastLoad)
    }

    #[test]
    fn test_delegation_strategy_equality()  {assert_eq!(
            DelegationStrategy::FirstAvailable)
            DelegationStrategy::FirstAvailable
        );
        assert_ne!(
            DelegationStrategy::FirstAvailable)
            DelegationStrategy::LeastLoad
        );
        assert_eq!(
            DelegationStrategy::Specific("test".to_string(),"
            DelegationStrategy::Specific("test".to_string()"
        );
    }
}
