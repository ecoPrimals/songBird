// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Protocol routing, load balancing, and circuit breaking for the canonical adapter.

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
// PROTOCOL HANDLER (enum dispatch)
// ============================================================================

/// TCP reachability probe for `http` / `https` logical protocols (TLS not negotiated; liveness only).
#[derive(Debug, Clone)]
pub struct TcpReachabilityHandler {
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

    async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        let start = std::time::Instant::now();
        let ep = service.endpoints.first().ok_or_else(|| SongbirdError::Service {
            service: service.id.clone(),
            message: String::from("No endpoints to probe"),
            suggested_alternatives: vec![],
            recovery_actions: vec![String::from("Register at least one endpoint for this service")],
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
}

/// Test-only HTTP stub for protocol router unit tests.
#[cfg(test)]
#[derive(Debug)]
pub struct MockHttpHandler;

#[cfg(test)]
impl MockHttpHandler {
    #[allow(
        clippy::unused_async,
        reason = "matches protocol handler async API; stub is synchronous"
    )]
    async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        Ok(CanonicalAdapterResponse {
            request_id: request.id.clone(),
            service_id: service.id.clone(),
            payload: serde_json::json!({"ok": true}),
            metadata: HashMap::new(),
            processing_time: Duration::from_millis(1),
            performance_info: super::types::CanonicalServicePerformance::default(),
        })
    }
}

/// Protocol handler for different communication protocols (enum dispatch; not `dyn`).
#[derive(Debug)]
pub enum CanonicalProtocolHandler {
    /// TCP connect probe for logical HTTP(S) endpoints.
    TcpReachability(TcpReachabilityHandler),
    /// Stub handler for crate tests.
    #[cfg(test)]
    MockHttp(MockHttpHandler),
}

impl CanonicalProtocolHandler {
    /// Protocol name key used for router registration.
    #[must_use]
    pub const fn protocol_name(&self) -> &'static str {
        match self {
            Self::TcpReachability(h) => h.protocol,
            #[cfg(test)]
            Self::MockHttp(_) => "http",
        }
    }

    /// Handle request using this protocol.
    pub async fn handle_request(
        &self,
        service: &CanonicalServiceInfo,
        request: &CanonicalAdapterRequest,
    ) -> SongbirdResult<CanonicalAdapterResponse> {
        match self {
            Self::TcpReachability(h) => h.handle_request(service, request).await,
            #[cfg(test)]
            Self::MockHttp(m) => m.handle_request(service, request).await,
        }
    }

    /// Check if service supports this protocol.
    #[must_use]
    pub fn supports_service(&self, service: &CanonicalServiceInfo) -> bool {
        match self {
            Self::TcpReachability(h) => {
                service.endpoints.iter().any(|e| e.protocol.eq_ignore_ascii_case(h.protocol))
            }
            #[cfg(test)]
            Self::MockHttp(_) => true,
        }
    }

    /// Protocol-specific metadata.
    #[must_use]
    pub fn get_metadata(&self) -> HashMap<String, String> {
        match self {
            Self::TcpReachability(_) => {
                HashMap::from([(String::from("probe"), String::from("tcp_connect"))])
            }
            #[cfg(test)]
            Self::MockHttp(_) => HashMap::new(),
        }
    }
}

// ============================================================================
// PROTOCOL ROUTER
// ============================================================================

/// Protocol router for handling different communication protocols.
pub struct CanonicalProtocolRouter {
    handlers: Arc<RwLock<HashMap<String, Arc<CanonicalProtocolHandler>>>>,
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
            String::from("http"),
            Arc::new(CanonicalProtocolHandler::TcpReachability(TcpReachabilityHandler::http())),
        );
        map.insert(
            String::from("https"),
            Arc::new(CanonicalProtocolHandler::TcpReachability(TcpReachabilityHandler::https())),
        );
        Self {
            handlers: Arc::new(RwLock::new(map)),
            default_protocol: String::from("http"),
        }
    }

    /// Register or replace a protocol handler (e.g. production HTTP client over TCP probe).
    pub async fn register_handler(&self, handler: Arc<CanonicalProtocolHandler>) {
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

        let handler = {
            let handlers = self.handlers.read().await;
            Arc::clone(
                handlers
                    .get(&protocol)
                    .or_else(|| handlers.get(&protocol.to_lowercase()))
                    .ok_or_else(|| SongbirdError::Service {
                        service: format!("protocol:{protocol}"),
                        message: format!("Protocol '{protocol}' is not supported"),
                        suggested_alternatives: handlers.keys().cloned().collect(),
                        recovery_actions: vec![],
                    })?,
            )
        };

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
                message: String::from("No services available for this capability"),
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
    #[allow(dead_code, reason = "stored for future per-service breaker tuning")]
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
