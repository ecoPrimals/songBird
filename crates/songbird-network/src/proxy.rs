/*!
 * Connection proxy and request routing for Songbird Orchestrator
 *
 * This module provides advanced connection proxying capabilities including:
 * - HTTP request routing and forwarding
 * - WebSocket connection proxying
 * - Load balancing across service instances
 * - Request/response transformation
 * - Metrics collection and monitoring
 * - Circuit breaker and retry logic
 * - SSL termination and security
 */

use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::any,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
// use tower::ServiceBuilder;
// use tower_http::cors::CorsLayer;

use songbird_discovery::traits::service::ServiceInfo;
use songbird_errors::SongbirdError;

/// Connection proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy server bind address
    pub bind_address: String,

    /// Proxy server port
    pub port: u16,

    /// Enable request logging
    pub enable_logging: bool,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,

    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout: u64,

    /// Enable load balancing
    pub enable_load_balancing: bool,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,

    /// Enable SSL termination
    pub enable_ssl: bool,

    /// SSL certificate path
    pub ssl_cert_path: Option<String>,

    /// SSL private key path
    pub ssl_key_path: Option<String>,

    /// Enable compression
    pub enable_compression: bool,

    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            enable_logging: true,
            request_timeout: 30,
            connection_timeout: 10,
            max_retries: 3,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            enable_load_balancing: true,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
            enable_ssl: false,
            ssl_cert_path: None,
            ssl_key_path: None,
            enable_compression: true,
            max_body_size: 1024 * 1024,
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
    WeightedRoundRobin,
    HealthBased,
}

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker state information
#[derive(Debug, Clone)]
pub struct CircuitState {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub last_failure_time: Option<Instant>,
    pub next_attempt_time: Option<Instant>,
}

/// Proxy request structure
#[derive(Debug, Clone)]
pub struct ProxyRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub source_ip: Option<String>,
    pub timestamp: Instant,
}

/// Proxy response structure
#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status_code: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub response_time: Duration,
}

/// Proxy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_connections: u32,
    pub average_response_time_ms: f64,
    pub bytes_transferred: u64,
    pub requests_per_second: f64,
    pub error_rate: f64,
}

impl Default for ProxyStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_connections: 0,
            average_response_time_ms: 0.0,
            bytes_transferred: 0,
            requests_per_second: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Load balancer state
#[derive(Debug, Default)]
pub struct LoadBalancerState {
    pub round_robin_counter: usize,
    pub connection_counts: HashMap<String, u32>,
}

/// Main connection proxy
#[derive(Debug)]
pub struct ConnectionProxy {
    config: ProxyConfig,
    services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<ProxyStats>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitState>>>,
    load_balancer: Arc<RwLock<LoadBalancerState>>,
}

impl ConnectionProxy {
    /// Create a new connection proxy
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(ProxyStats::default())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(RwLock::new(LoadBalancerState::default())),
        }
    }

    /// Start the proxy server
    pub async fn start(&self) -> Result<(), SongbirdError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SongbirdError::Configuration {
                field: "proxy_lifecycle".to_string(),
                message: "Proxy server already running".to_string(),
                suggestion: Some("Check proxy status before starting".to_string()),
            });
        }

        *running = true;
        tracing::info!(
            "Starting proxy server on {}:{}",
            self.config.bind_address,
            self.config.port
        );

        // Build the router
        let app = self.build_router().await;

        // Start the server
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            SongbirdError::Communication(format!("Failed to bind to {}: {}", addr, e))
        })?;

        tracing::info!("Proxy server listening on {}", addr);

        // Serve the application
        tokio::spawn(async move {
            if let Err(e) = axum::Server::from_tcp(listener.into_std().unwrap())
                .unwrap()
                .serve(app.into_make_service())
                .await
            {
                tracing::error!("Proxy server error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop the proxy server
    pub async fn stop(&self) -> Result<(), SongbirdError> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(SongbirdError::Configuration {
                field: "proxy_lifecycle".to_string(),
                message: "Proxy server not running".to_string(),
                suggestion: Some("Check proxy status before stopping".to_string()),
            });
        }

        *running = false;
        tracing::info!("Stopping proxy server");
        Ok(())
    }

    /// Register a service
    pub async fn register_service(
        &self,
        service_name: String,
        service_instances: Vec<ServiceInfo>,
    ) {
        let mut services = self.services.write().await;
        services.insert(service_name.clone(), service_instances);
        tracing::info!("Registered service: {}", service_name);
    }

    /// Unregister a service
    pub async fn unregister_service(&self, service_name: &str) {
        let mut services = self.services.write().await;
        services.remove(service_name);
        tracing::info!("Unregistered service: {}", service_name);
    }

    /// Route a request to a service
    pub async fn route_request(
        &self,
        service_name: &str,
        request: ProxyRequest,
    ) -> Result<ProxyResponse, SongbirdError> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // Check circuit breaker
        if self.is_circuit_breaker_open(service_name).await {
            return Err(SongbirdError::Communication(format!(
                "Circuit breaker open for service: {}",
                service_name
            )));
        }

        // Select service instance
        let service_instance = self.select_service_instance(service_name).await?;

        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.request_timeout))
            .build()
            .map_err(|e| {
                SongbirdError::Communication(format!("Failed to create HTTP client: {}", e))
            })?;

        // Build target URL
        let target_url = format!(
            "http://{}:{}{}",
            service_instance.host,
            service_instance.port,
            request
                .uri
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or("")
        );

        // Build request
        // Convert axum::http::Method to reqwest::Method by parsing string
        let reqwest_method = reqwest::Method::from_bytes(request.method.as_str().as_bytes())
            .map_err(|e| SongbirdError::Communication(format!("Invalid HTTP method: {}", e)))?;
        let mut req_builder = client.request(reqwest_method, &target_url);

        // Add headers
        for (name, value) in request.headers.iter() {
            // Convert axum headers to reqwest headers
            let header_name = name.as_str();
            let header_value = value.to_str().unwrap_or("");
            req_builder = req_builder.header(header_name, header_value);
        }

        // Add body
        if !request.body.is_empty() {
            req_builder = req_builder.body(request.body);
        }

        // Send request
        let start_time = Instant::now();
        let response = req_builder.send().await;

        match response {
            Ok(resp) => {
                // Convert reqwest StatusCode to axum StatusCode
                let status_code = axum::http::StatusCode::from_u16(resp.status().as_u16())
                    .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

                // Convert reqwest headers to axum headers
                let mut headers = axum::http::HeaderMap::new();
                for (name, value) in resp.headers().iter() {
                    if let (Ok(header_name), Ok(header_value)) = (
                        axum::http::HeaderName::from_bytes(name.as_str().as_bytes()),
                        axum::http::HeaderValue::from_bytes(value.as_bytes()),
                    ) {
                        headers.insert(header_name, header_value);
                    }
                }
                let body = resp.bytes().await.map_err(|e| {
                    SongbirdError::Communication(format!("Failed to read response body: {}", e))
                })?;

                let response_time = start_time.elapsed();

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.successful_requests += 1;
                    stats.bytes_transferred += body.len() as u64;

                    // Update average response time
                    let total_time =
                        stats.average_response_time_ms * (stats.successful_requests - 1) as f64;
                    stats.average_response_time_ms = (total_time
                        + response_time.as_millis() as f64)
                        / stats.successful_requests as f64;
                }

                // Reset circuit breaker on success
                self.reset_circuit_breaker(service_name).await;

                Ok(ProxyResponse {
                    status_code,
                    headers,
                    body: body.to_vec(),
                    response_time,
                })
            }
            Err(e) => {
                // Update failure stats
                {
                    let mut stats = self.stats.write().await;
                    stats.failed_requests += 1;
                    stats.error_rate = stats.failed_requests as f64 / stats.total_requests as f64;
                }

                // Update circuit breaker
                self.record_circuit_breaker_failure(service_name).await;

                Err(SongbirdError::Communication(format!(
                    "Request failed: {}",
                    e
                )))
            }
        }
    }

    /// Build the router
    async fn build_router(&self) -> Router {
        Router::new()
            .route("/*path", any(proxy_handler))
            .fallback(proxy_fallback)
            .with_state(self.clone())
    }

    /// Select a service instance based on load balancing strategy
    async fn select_service_instance(
        &self,
        service_name: &str,
    ) -> Result<ServiceInfo, SongbirdError> {
        let services = self.services.read().await;
        let service_instances =
            services
                .get(service_name)
                .ok_or_else(|| SongbirdError::Configuration {
                    message: format!("Service not found: {}", service_name),
                    suggestion: Some("Check if service is registered".to_string()),
                    field: "service_name".to_string(),
                })?;

        if service_instances.is_empty() {
            return Err(SongbirdError::Configuration {
                message: format!("No instances available for service: {}", service_name),
                suggestion: Some("Check service health and registration".to_string()),
                field: "service_instances".to_string(),
            });
        }

        let mut load_balancer = self.load_balancer.write().await;

        let selected_instance = match self.config.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                let index = load_balancer.round_robin_counter % service_instances.len();
                load_balancer.round_robin_counter += 1;
                &service_instances[index]
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let index = rand::thread_rng().gen_range(0..service_instances.len());
                &service_instances[index]
            }
            LoadBalancingStrategy::LeastConnections => service_instances
                .iter()
                .min_by_key(|instance| {
                    load_balancer
                        .connection_counts
                        .get(&instance.service_id)
                        .unwrap_or(&0)
                })
                .ok_or_else(|| SongbirdError::Configuration {
                    message: "No service instances available for least connections selection"
                        .to_string(),
                    suggestion: Some("Check proxy configuration settings".to_string()),
                    field: "service_instances".to_string(),
                })?,
            _ => &service_instances[0],
        };

        *load_balancer
            .connection_counts
            .entry(selected_instance.service_id.clone())
            .or_insert(0) += 1;

        Ok(selected_instance.clone())
    }

    /// Check if circuit breaker is open
    async fn is_circuit_breaker_open(&self, service_name: &str) -> bool {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(circuit_state) = circuit_breakers.get(service_name) {
            match circuit_state.state {
                CircuitBreakerState::Open => {
                    // Check if it's time to attempt a half-open state
                    if let Some(next_attempt) = circuit_state.next_attempt_time {
                        Instant::now() < next_attempt
                    } else {
                        true
                    }
                }
                CircuitBreakerState::HalfOpen => false,
                CircuitBreakerState::Closed => false,
            }
        } else {
            false
        }
    }

    /// Record a circuit breaker failure
    async fn record_circuit_breaker_failure(&self, service_name: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let circuit_state = circuit_breakers
            .entry(service_name.to_string())
            .or_insert_with(|| CircuitState {
                state: CircuitBreakerState::Closed,
                failure_count: 0,
                last_failure_time: None,
                next_attempt_time: None,
            });

        circuit_state.failure_count += 1;
        circuit_state.last_failure_time = Some(Instant::now());

        if circuit_state.failure_count >= self.config.circuit_breaker_threshold {
            circuit_state.state = CircuitBreakerState::Open;
            circuit_state.next_attempt_time =
                Some(Instant::now() + Duration::from_secs(self.config.circuit_breaker_timeout));
            tracing::warn!("Circuit breaker opened for service: {}", service_name);
        }
    }

    /// Reset circuit breaker on success
    async fn reset_circuit_breaker(&self, service_name: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(circuit_state) = circuit_breakers.get_mut(service_name) {
            circuit_state.state = CircuitBreakerState::Closed;
            circuit_state.failure_count = 0;
            circuit_state.last_failure_time = None;
            circuit_state.next_attempt_time = None;
        }
    }

    /// Get proxy statistics
    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }

    /// Get circuit breaker states
    pub async fn get_circuit_breaker_states(&self) -> HashMap<String, CircuitState> {
        self.circuit_breakers.read().await.clone()
    }
}

impl Clone for ConnectionProxy {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            services: Arc::clone(&self.services),
            running: Arc::clone(&self.running),
            stats: Arc::clone(&self.stats),
            circuit_breakers: Arc::clone(&self.circuit_breakers),
            load_balancer: Arc::clone(&self.load_balancer),
        }
    }
}

/// Proxy handler for incoming requests
async fn proxy_handler(
    State(proxy): State<ConnectionProxy>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let start_time = Instant::now();

    // Extract service name from path
    let service = path.split('/').next().unwrap_or("default");

    // Get source IP
    let source_ip = headers
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|hv| hv.to_str().ok())
                .map(|s| s.to_string())
        });

    // Build URI with path
    let uri_with_path = format!("/{}", path);
    let reconstructed_uri = uri_with_path
        .parse::<Uri>()
        .unwrap_or_else(|_| Uri::from_static("/"));

    // Create proxy request
    let proxy_request = ProxyRequest {
        method: method.clone(),
        uri: reconstructed_uri,
        headers: headers.clone(),
        body: body.to_vec(),
        source_ip,
        timestamp: start_time,
    };

    // Log the request if logging is enabled
    if proxy.config.enable_logging {
        tracing::info!(
            "Proxying {} {} to service '{}' from {}",
            method,
            proxy_request.uri,
            service,
            proxy_request.source_ip.as_deref().unwrap_or("unknown")
        );
    }

    // Route the request through the proxy
    match proxy.route_request(&service, proxy_request).await {
        Ok(proxy_response) => {
            // Convert ProxyResponse to axum Response
            let mut response_builder =
                axum::response::Response::builder().status(proxy_response.status_code);

            // Add response headers
            for (name, value) in proxy_response.headers.iter() {
                response_builder = response_builder.header(name, value);
            }

            // Add proxy timing header
            response_builder = response_builder.header(
                "X-Proxy-Duration-Ms",
                proxy_response.response_time.as_millis().to_string(),
            );

            // Build and return response
            match response_builder.body(axum::body::Body::from(proxy_response.body)) {
                Ok(response) => response.into_response(),
                Err(e) => {
                    tracing::error!("Failed to build proxy response: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to build response",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            // Handle proxy errors
            tracing::error!("Proxy request failed for service '{}': {}", service, e);

            let error_response = match e {
                SongbirdError::Configuration { .. } => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service unavailable".to_string(),
                ),
                SongbirdError::Communication { .. } => {
                    (StatusCode::BAD_GATEWAY, "Gateway error".to_string())
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal proxy error".to_string(),
                ),
            };

            error_response.into_response()
        }
    }
}

/// Fallback handler for unmatched routes
async fn proxy_fallback() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Service not found")
}
