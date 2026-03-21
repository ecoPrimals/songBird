// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core adapter: capability-based service registry, routing, and request handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use crate::errors::{SongbirdError, SongbirdResult};
use crate::traits::canonical::{
    HealthStatus as CanonicalHealthStatus, ProviderType as CanonicalProviderType,
};

use super::routing::{
    CanonicalCircuitBreaker, CanonicalLoadBalancer, CanonicalProtocolHandler,
    CanonicalProtocolRouter,
};
use super::types::{
    CanonicalAdapterConfig, CanonicalAdapterMetrics, CanonicalAdapterRequest,
    CanonicalAdapterResponse, CanonicalRegisteredService, CanonicalRequestPriority,
    CanonicalServicePerformance,
};

// ============================================================================
// SERVICE REGISTRY
// ============================================================================

/// Service registry for managing discovered services.
#[derive(Debug, Default)]
pub struct CanonicalServiceRegistry {
    services_by_capability: HashMap<String, Vec<CanonicalRegisteredService>>,
    services_by_type: HashMap<CanonicalProviderType, Vec<CanonicalRegisteredService>>,
    all_services: HashMap<String, CanonicalRegisteredService>,
    health_cache: HashMap<String, (CanonicalHealthStatus, SystemTime)>,
}

// ============================================================================
// UNIVERSAL ADAPTER
// ============================================================================

/// Single universal adapter for all service types.
///
/// Consolidates capability-based routing, protocol handling, load balancing,
/// and circuit breaking into one coherent entry point.
#[derive(Debug)]
pub struct CanonicalUniversalAdapter {
    registry: Arc<RwLock<CanonicalServiceRegistry>>,
    protocol_router: Arc<CanonicalProtocolRouter>,
    load_balancer: Arc<CanonicalLoadBalancer>,
    circuit_breaker: Arc<CanonicalCircuitBreaker>,
    metrics: Arc<RwLock<CanonicalAdapterMetrics>>,
}

fn health_status_from_probe(err: &SongbirdError) -> CanonicalHealthStatus {
    match err {
        SongbirdError::Service {
            message,
            ..
        } if message.contains("not supported") => CanonicalHealthStatus::Unknown,
        _ => CanonicalHealthStatus::Unhealthy,
    }
}

impl CanonicalUniversalAdapter {
    /// Create a new canonical universal adapter.
    #[must_use]
    pub fn new(config: CanonicalAdapterConfig) -> Self {
        let CanonicalAdapterConfig {
            load_balancing,
            circuit_breaker,
            ..
        } = config;
        Self {
            registry: Arc::new(RwLock::new(CanonicalServiceRegistry::default())),
            protocol_router: Arc::new(CanonicalProtocolRouter::new()),
            load_balancer: Arc::new(CanonicalLoadBalancer::new(load_balancing)),
            circuit_breaker: Arc::new(CanonicalCircuitBreaker::new(circuit_breaker)),
            metrics: Arc::new(RwLock::new(CanonicalAdapterMetrics::default())),
        }
    }

    /// Register a service with the adapter.
    ///
    /// # Errors
    ///
    /// Currently infallible; returns `Result` for forward compatibility.
    pub async fn register_service(
        &self,
        service: crate::traits::canonical::ServiceInfo,
        capabilities: Vec<String>,
    ) -> SongbirdResult<()> {
        let mut registry = self.registry.write().await;

        let service_id = service.id.clone();
        let registered_service = CanonicalRegisteredService {
            provider_type: service
                .metadata
                .get("provider_type")
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_else(|| CanonicalProviderType::Custom("unknown".to_string())),
            registered_at: SystemTime::now(),
            last_health_check: None,
            performance: CanonicalServicePerformance::default(),
            service,
            capabilities,
        };

        for capability in &registered_service.capabilities {
            registry
                .services_by_capability
                .entry(capability.clone())
                .or_default()
                .push(registered_service.clone());
        }

        registry
            .services_by_type
            .entry(registered_service.provider_type.clone())
            .or_default()
            .push(registered_service.clone());

        registry.all_services.insert(service_id, registered_service);

        drop(registry);
        Ok(())
    }

    /// Register a protocol handler on the embedded router (tests or custom transports).
    pub async fn register_protocol_handler(&self, handler: Arc<dyn CanonicalProtocolHandler>) {
        self.protocol_router.register_handler(handler).await;
    }

    /// Handle a capability request.
    ///
    /// # Errors
    ///
    /// Returns an error if no services match, the circuit breaker is open,
    /// or the protocol router fails.
    pub async fn handle_request(
        &self,
        request: CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        let start_time = SystemTime::now();

        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            *metrics.requests_by_capability.entry(request.capability.clone()).or_insert(0) += 1;
        }

        let services = self.find_services_by_capability(&request.capability).await?;

        if services.is_empty() {
            return Err(SongbirdError::Service {
                service: request.capability.clone(),
                message: "No services found with this capability".to_string(),
                suggested_alternatives: vec![],
                recovery_actions: vec![
                    "Check if services with this capability are registered".to_string(),
                ],
            });
        }

        let selected_service = self.load_balancer.select_service(&services, &request)?;

        if !self.circuit_breaker.can_execute(&selected_service.service.id).await {
            return Err(SongbirdError::Service {
                service: selected_service.service.id.clone(),
                message: "Circuit breaker is open".to_string(),
                suggested_alternatives: vec![],
                recovery_actions: vec!["Try again later".to_string()],
            });
        }

        let result = self.protocol_router.route_request(&selected_service.service, &request).await;

        if result.is_ok() {
            self.circuit_breaker.record_success(&selected_service.service.id).await;
            let mut metrics = self.metrics.write().await;
            metrics.successful_requests += 1;

            let processing_time = start_time.elapsed().unwrap_or(Duration::ZERO);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "truncation acceptable: nanos fit in u64 for running average window"
            )]
            let avg_nanos = metrics.avg_response_time.as_nanos() as u64;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "truncation acceptable: nanos fit in u64 for running average window"
            )]
            let processing_nanos = processing_time.as_nanos() as u64;
            let new_avg = (avg_nanos * (metrics.successful_requests - 1) + processing_nanos)
                / metrics.successful_requests;
            metrics.avg_response_time = Duration::from_nanos(new_avg);
        } else {
            self.circuit_breaker.record_failure(&selected_service.service.id).await;
            let mut metrics = self.metrics.write().await;
            metrics.failed_requests += 1;
        }

        result
    }

    /// Find services by capability.
    async fn find_services_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<CanonicalRegisteredService>> {
        let registry = self.registry.read().await;
        Ok(registry.services_by_capability.get(capability).cloned().unwrap_or_default())
    }

    /// Get adapter metrics.
    pub async fn get_metrics(&self) -> CanonicalAdapterMetrics {
        self.metrics.read().await.clone()
    }

    /// Perform health check on all registered services.
    ///
    /// Probes each service via [`CanonicalProtocolRouter::route_request`] using an internal
    /// liveness capability (`songbird.internal.health_probe`). TCP reachability is used for
    /// `http`/`https` endpoints unless a custom handler replaced the defaults.
    ///
    /// # Errors
    ///
    /// Currently infallible; returns `Result` for forward compatibility.
    pub async fn health_check_all(&self) -> SongbirdResult<HashMap<String, CanonicalHealthStatus>> {
        let snapshot: Vec<(String, CanonicalRegisteredService)> = {
            let registry = self.registry.read().await;
            registry.all_services.iter().map(|(id, svc)| (id.clone(), svc.clone())).collect()
        };

        let ping = create_adapter_request(
            "songbird.internal.health_probe",
            serde_json::json!({"kind": "liveness"}),
            CanonicalRequestPriority::Low,
        );

        let mut results = HashMap::new();
        for (service_id, registered) in snapshot {
            let status = match self.protocol_router.route_request(&registered.service, &ping).await
            {
                Ok(_) => CanonicalHealthStatus::Healthy,
                Err(e) => health_status_from_probe(&e),
            };

            results.insert(service_id.clone(), status.clone());

            let now = SystemTime::now();
            let mut registry = self.registry.write().await;
            if let Some(rs) = registry.all_services.get_mut(&service_id) {
                rs.last_health_check = Some(now);
                rs.service.health = status.clone();
                registry.health_cache.insert(service_id, (status, now));
            }
        }

        Ok(results)
    }
}

/// Create a new canonical universal adapter with default configuration.
#[must_use]
pub fn create_canonical_adapter() -> CanonicalUniversalAdapter {
    CanonicalUniversalAdapter::new(CanonicalAdapterConfig::default())
}

/// Create a canonical adapter request.
#[must_use]
pub fn create_adapter_request(
    capability: &str,
    payload: serde_json::Value,
    priority: super::types::CanonicalRequestPriority,
) -> CanonicalAdapterRequest {
    CanonicalAdapterRequest {
        id: uuid::Uuid::new_v4().to_string(),
        capability: capability.to_string(),
        payload,
        priority,
        timeout: None,
        metadata: HashMap::new(),
    }
}
