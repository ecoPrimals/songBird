// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical Universal Adapter System
//!
//! Consolidates all fragmented adapter implementations into a single,
//! capability-based routing system with protocol handling, load balancing,
//! and circuit breaking.
//!
//! ## Module layout
//!
//! - [`types`] — Configuration, DTOs, enums, and their `Default` impls
//! - [`routing`] — Protocol routing, load balancing, circuit breaking
//! - [`adapter`] — Core adapter, service registry, convenience constructors

pub mod adapter;
pub mod routing;
pub mod types;

// Re-export public API so existing `use canonical::*` paths keep working.
pub use adapter::{CanonicalUniversalAdapter, create_adapter_request, create_canonical_adapter};
pub use routing::{
    CanonicalCircuitBreaker, CanonicalLoadBalancer, CanonicalProtocolHandler,
    CanonicalProtocolRouter,
};
pub use types::{
    CanonicalAdapterConfig, CanonicalAdapterMetrics, CanonicalAdapterRequest,
    CanonicalAdapterResponse, CanonicalCircuitBreakerConfig, CanonicalCircuitState,
    CanonicalDiscoveryConfig, CanonicalHealthCheckConfig, CanonicalLoadBalancingConfig,
    CanonicalLoadBalancingStrategy, CanonicalMonitoringConfig, CanonicalRegisteredService,
    CanonicalRequestPriority, CanonicalRetryConfig, CanonicalServicePerformance,
    CanonicalTimeoutConfig,
};

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::traits::canonical::{
        Endpoint, HealthStatus, ProviderType, ServiceInfo as CanonicalServiceInfo, ServiceType,
    };
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    fn sample_service(id: &str, capability: &str) -> CanonicalServiceInfo {
        let mut metadata = HashMap::new();
        metadata.insert(
            "provider_type".to_string(),
            serde_json::to_string(&ProviderType::Service).expect("serialize ProviderType"),
        );
        CanonicalServiceInfo {
            id: id.to_string(),
            name: format!("service-{id}"),
            service_type: ServiceType::WebService,
            version: "1.0.0".to_string(),
            endpoints: vec![Endpoint {
                protocol: "http".to_string(),
                host: "127.0.0.1".to_string(),
                port: 8080,
                path: None,
                metadata: HashMap::new(),
            }],
            health: HealthStatus::Healthy,
            metadata,
            tags: vec![],
            capabilities: vec![capability.to_string()],
            last_updated: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn register_service_indexes_by_capability_and_type() {
        let adapter = create_canonical_adapter();
        adapter
            .register_service(sample_service("alpha", "compute"), vec!["compute".to_string()])
            .await
            .expect("register");

        let metrics = adapter.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn handle_request_errors_when_no_services_for_capability() {
        let adapter = create_canonical_adapter();
        let req =
            create_adapter_request("missing-cap", json!({}), CanonicalRequestPriority::Normal);
        let err = adapter.handle_request(req).await.expect_err("expected service error");
        let msg = err.to_string();
        assert!(
            msg.contains("No services found") || msg.contains("No services"),
            "unexpected message: {msg}"
        );
    }

    #[tokio::test]
    async fn handle_request_errors_when_protocol_handler_missing() {
        let mut svc = sample_service("s1", "compute");
        svc.endpoints = vec![Endpoint {
            protocol: "unregistered-protocol".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9,
            path: None,
            metadata: HashMap::new(),
        }];
        let adapter = create_canonical_adapter();
        adapter.register_service(svc, vec!["compute".to_string()]).await.expect("register");

        let req = create_adapter_request("compute", json!({}), CanonicalRequestPriority::Normal);
        let err = adapter.handle_request(req).await.expect_err("expected protocol error");
        assert!(
            err.to_string().contains("Protocol") || err.to_string().contains("protocol"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn load_balancer_select_empty_returns_error() {
        let lb = CanonicalLoadBalancer::new(CanonicalLoadBalancingConfig::default());
        let req = create_adapter_request("x", json!({}), CanonicalRequestPriority::Normal);
        let err = lb.select_service(&[], &req).expect_err("empty services");
        assert!(err.to_string().contains("No services available"));
    }

    #[tokio::test]
    async fn circuit_breaker_opens_after_failure() {
        let cb = CanonicalCircuitBreaker::new(CanonicalCircuitBreakerConfig::default());
        assert!(cb.can_execute("peer").await);
        cb.record_failure("peer").await;
        assert!(!cb.can_execute("peer").await);
        cb.record_success("peer").await;
        assert!(cb.can_execute("peer").await);
    }

    #[test]
    fn adapter_config_defaults_are_sensible() {
        let cfg = CanonicalAdapterConfig::default();
        assert!(cfg.monitoring.enabled);
        assert_eq!(cfg.load_balancing.strategy, CanonicalLoadBalancingStrategy::HealthAware);
    }

    fn registered_service(id: &str, capability: &str, avg_ms: u64) -> CanonicalRegisteredService {
        CanonicalRegisteredService {
            service: sample_service(id, capability),
            capabilities: vec![capability.to_string()],
            provider_type: ProviderType::Service,
            registered_at: SystemTime::now(),
            last_health_check: None,
            performance: CanonicalServicePerformance {
                avg_response_time: Duration::from_millis(avg_ms),
                success_rate: 1.0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                last_updated: SystemTime::now(),
            },
        }
    }

    #[test]
    fn load_balancer_least_response_time_picks_lowest_latency() {
        let lb = CanonicalLoadBalancer::new(CanonicalLoadBalancingConfig {
            strategy: CanonicalLoadBalancingStrategy::LeastResponseTime,
            ..CanonicalLoadBalancingConfig::default()
        });
        let req = create_adapter_request("compute", json!({}), CanonicalRequestPriority::Normal);
        let slow = registered_service("slow", "compute", 500);
        let fast = registered_service("fast", "compute", 5);
        let picked = lb
            .select_service(&[slow, fast], &req)
            .expect("select service with least response time");
        assert_eq!(picked.service.id, "fast");
    }

    #[derive(Debug)]
    struct MockHttpHandler;

    #[async_trait::async_trait]
    impl CanonicalProtocolHandler for MockHttpHandler {
        fn protocol_name(&self) -> &str {
            "http"
        }

        async fn handle_request(
            &self,
            service: &CanonicalServiceInfo,
            request: &CanonicalAdapterRequest,
        ) -> crate::errors::SongbirdResult<CanonicalAdapterResponse> {
            Ok(CanonicalAdapterResponse {
                request_id: request.id.clone(),
                service_id: service.id.clone(),
                payload: json!({"ok": true}),
                metadata: HashMap::new(),
                processing_time: Duration::from_millis(1),
                performance_info: CanonicalServicePerformance::default(),
            })
        }

        fn supports_service(&self, _service: &CanonicalServiceInfo) -> bool {
            true
        }

        fn get_metadata(&self) -> HashMap<String, String> {
            HashMap::new()
        }
    }

    #[tokio::test]
    async fn protocol_router_routes_registered_http_handler() {
        let router = CanonicalProtocolRouter::new();
        router.register_handler(Arc::new(MockHttpHandler)).await;
        let svc = sample_service("svc", "compute");
        let req = create_adapter_request("compute", json!({}), CanonicalRequestPriority::Normal);
        let res = router.route_request(&svc, &req).await.expect("route");
        assert_eq!(res.service_id, "svc");
    }

    #[test]
    fn load_balancer_round_robin_rotates() {
        let lb = CanonicalLoadBalancer::new(CanonicalLoadBalancingConfig {
            strategy: CanonicalLoadBalancingStrategy::RoundRobin,
            ..CanonicalLoadBalancingConfig::default()
        });
        let req = create_adapter_request("compute", json!({}), CanonicalRequestPriority::Normal);
        let pool =
            vec![registered_service("a", "compute", 1), registered_service("b", "compute", 1)];
        let first = lb.select_service(&pool, &req).expect("pick");
        let second = lb.select_service(&pool, &req).expect("pick");
        assert_ne!(first.service.id, second.service.id);
        let third = lb.select_service(&pool, &req).expect("pick");
        assert_eq!(first.service.id, third.service.id);
    }
}
