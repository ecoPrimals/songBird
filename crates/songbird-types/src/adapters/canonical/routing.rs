// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Protocol routing, load balancing, and circuit breaking for the canonical adapter.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::errors::{SongbirdError, SongbirdResult};
use crate::traits::canonical::ServiceInfo as CanonicalServiceInfo;

use super::types::{
    CanonicalAdapterRequest, CanonicalAdapterResponse, CanonicalCircuitBreakerConfig,
    CanonicalCircuitState, CanonicalLoadBalancingConfig, CanonicalLoadBalancingStrategy,
    CanonicalRegisteredService,
};

// ============================================================================
// PROTOCOL HANDLER TRAIT
// ============================================================================

/// Protocol handler trait for different communication protocols.
#[async_trait]
pub trait CanonicalProtocolHandler: Send + Sync {
    /// Protocol name.
    fn protocol_name(&self) -> &str;

    /// Handle request using this protocol.
    async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse>;

    /// Check if service supports this protocol.
    fn supports_service(&self, service: &CanonicalServiceInfo) -> bool;

    /// Get protocol-specific metadata.
    fn get_metadata(&self) -> HashMap<String, String>;
}

// ============================================================================
// PROTOCOL ROUTER
// ============================================================================

/// TCP reachability probe for `http` / `https` logical protocols (TLS not negotiated; liveness only).
#[derive(Debug)]
struct TcpReachabilityHandler {
    protocol: &'static str,
}

impl TcpReachabilityHandler {
    const fn http() -> Self {
        Self {
            protocol: "http",
        }
    }

    const fn https() -> Self {
        Self {
            protocol: "https",
        }
    }
}

#[async_trait]
impl CanonicalProtocolHandler for TcpReachabilityHandler {
    fn protocol_name(&self) -> &str {
        self.protocol
    }

    async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        let start = std::time::Instant::now();
        let ep = service.endpoints.first().ok_or_else(|| SongbirdError::Service {
            service: service.id.clone(),
            message: "No endpoints to probe".to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec!["Register at least one endpoint for this service".to_string()],
        })?;

        let addr = format!("{}:{}", ep.host, ep.port);
        let timeout = Duration::from_secs(3);
        tokio::time::timeout(timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "TCP health probe to {addr} timed out after {timeout:?}"
                ))
            })?
            .map_err(|e| SongbirdError::network(format!("TCP connect to {addr} failed: {e}")))?;

        Ok(CanonicalAdapterResponse {
            request_id: request.id.clone(),
            service_id: service.id.clone(),
            payload: serde_json::json!({"status": "reachable", "protocol": self.protocol}),
            metadata: HashMap::new(),
            processing_time: start.elapsed(),
            performance_info: super::types::CanonicalServicePerformance::default(),
        })
    }

    fn supports_service(&self, service: &CanonicalServiceInfo) -> bool {
        service.endpoints.iter().any(|e| e.protocol.eq_ignore_ascii_case(self.protocol))
    }

    fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::from([("probe".to_string(), "tcp_connect".to_string())])
    }
}

/// Protocol router for handling different communication protocols.
pub struct CanonicalProtocolRouter {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn CanonicalProtocolHandler>>>>,
    default_protocol: String,
}

impl std::fmt::Debug for CanonicalProtocolRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanonicalProtocolRouter")
            .field("handlers", &"<async registry>")
            .field("default_protocol", &self.default_protocol)
            .finish()
    }
}

impl Default for CanonicalProtocolRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalProtocolRouter {
    /// Create a new protocol router.
    #[must_use]
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "http".to_string(),
            Arc::new(TcpReachabilityHandler::http()) as Arc<dyn CanonicalProtocolHandler>,
        );
        map.insert(
            "https".to_string(),
            Arc::new(TcpReachabilityHandler::https()) as Arc<dyn CanonicalProtocolHandler>,
        );
        Self {
            handlers: Arc::new(RwLock::new(map)),
            default_protocol: "http".to_string(),
        }
    }

    /// Register or replace a protocol handler (e.g. production HTTP client over TCP probe).
    pub async fn register_handler(&self, handler: Arc<dyn CanonicalProtocolHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.protocol_name().to_string(), handler);
    }

    /// Route request to the appropriate protocol handler.
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is unsupported or the handler fails.
    pub async fn route_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        let protocol: String = service
            .endpoints
            .first()
            .map_or_else(|| self.default_protocol.clone(), |e| e.protocol.clone());

        let handlers = self.handlers.read().await;
        let handler = handlers
            .get(&protocol)
            .or_else(|| handlers.get(&protocol.to_lowercase()))
            .ok_or_else(|| SongbirdError::Service {
                service: format!("protocol:{protocol}"),
                message: format!("Protocol '{protocol}' is not supported"),
                suggested_alternatives: handlers.keys().cloned().collect(),
                recovery_actions: vec![],
            })?;

        handler.handle_request(service, request).await
    }
}

// ============================================================================
// LOAD BALANCER
// ============================================================================

/// Load balancer for intelligent service selection.
#[derive(Debug)]
pub struct CanonicalLoadBalancer {
    strategy: CanonicalLoadBalancingStrategy,
    /// Round-robin cursor (global across [`select_service`] calls).
    round_robin_next: Arc<AtomicUsize>,
}

impl CanonicalLoadBalancer {
    /// Create a new load balancer.
    #[must_use]
    pub fn new(config: CanonicalLoadBalancingConfig) -> Self {
        Self {
            strategy: config.strategy,
            round_robin_next: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Select best service from available services.
    ///
    /// # Errors
    ///
    /// Returns an error if no services are available for the requested capability.
    pub fn select_service(
        &self,
        services: &[CanonicalRegisteredService],
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalRegisteredService> {
        if services.is_empty() {
            return Err(SongbirdError::Service {
                service: request.capability.clone(),
                message: "No services available for this capability".to_string(),
                suggested_alternatives: vec![],
                recovery_actions: vec![],
            });
        }

        match self.strategy {
            CanonicalLoadBalancingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..services.len());
                Ok(services[index].clone())
            }
            CanonicalLoadBalancingStrategy::RoundRobin
            | CanonicalLoadBalancingStrategy::WeightedRoundRobin => {
                let idx = self.round_robin_next.fetch_add(1, Ordering::Relaxed) % services.len();
                Ok(services[idx].clone())
            }
            CanonicalLoadBalancingStrategy::LeastResponseTime => services
                .iter()
                .min_by_key(|s| s.performance.avg_response_time)
                .ok_or_else(|| {
                    SongbirdError::service(
                        "load-balancer",
                        "No services available for LeastResponseTime strategy",
                    )
                })
                .cloned(),
            // LeastConnections, ConsistentHash, HealthAware — use first-available until wired to live stats.
            _ => Ok(services[0].clone()),
        }
    }
}

// ============================================================================
// CIRCUIT BREAKER
// ============================================================================

/// Circuit breaker for fault tolerance.
#[derive(Debug)]
pub struct CanonicalCircuitBreaker {
    states: Arc<RwLock<HashMap<String, CanonicalCircuitState>>>,
    /// Retained for threshold / timeout tuning when failure counting is wired.
    #[expect(dead_code, reason = "stored for future per-service breaker tuning")]
    config: CanonicalCircuitBreakerConfig,
}

impl CanonicalCircuitBreaker {
    /// Create a new circuit breaker.
    #[must_use]
    pub fn new(config: CanonicalCircuitBreakerConfig) -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if service can execute (circuit is closed or half-open).
    pub async fn can_execute(&self, service_id: &str) -> bool {
        let states = self.states.read().await;
        !matches!(states.get(service_id), Some(CanonicalCircuitState::Open))
    }

    /// Record successful execution.
    pub async fn record_success(&self, service_id: &str) {
        let mut states = self.states.write().await;
        states.insert(service_id.to_string(), CanonicalCircuitState::Closed);
    }

    /// Record failed execution.
    pub async fn record_failure(&self, service_id: &str) {
        let mut states = self.states.write().await;
        states.insert(service_id.to_string(), CanonicalCircuitState::Open);
    }
}
