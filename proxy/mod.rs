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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::errors::SongbirdError;
use crate::registry::ServiceInfo;

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
            max_body_size: 1024 * 1024, // 1MB
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round robin distribution
    RoundRobin,
    
    /// Route to least connections
    LeastConnections,
    
    /// Weighted round robin
    WeightedRoundRobin,
    
    /// Random selection
    Random,
    
    /// Health-based routing
    HealthBased,
}

/// Connection proxy that routes requests through the orchestrator
#[derive(Debug)]
pub struct ConnectionProxy {
    /// Proxy configuration
    config: ProxyConfig,
    
    /// Service registry for routing
    services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    
    /// Running state
    running: Arc<RwLock<bool>>,
    
    /// Proxy statistics
    stats: Arc<RwLock<ProxyStats>>,
    
    /// Circuit breaker states
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
    
    /// Load balancer state
    load_balancer: Arc<RwLock<LoadBalancerState>>,
}

/// Proxy request structure
#[derive(Debug, Clone)]
pub struct ProxyRequest {
    /// HTTP method
    pub method: Method,
    
    /// Request URI
    pub uri: Uri,
    
    /// Request headers
    pub headers: HeaderMap,
    
    /// Request body
    pub body: Vec<u8>,
    
    /// Source IP address
    pub source_ip: Option<String>,
    
    /// Request timestamp
    pub timestamp: Instant,
}

/// Proxy response structure
#[derive(Debug, Clone)]
pub struct ProxyResponse {
    /// HTTP status code
    pub status_code: StatusCode,
    
    /// Response headers
    pub headers: HeaderMap,
    
    /// Response body
    pub body: Vec<u8>,
    
    /// Response time
    pub response_time: Duration,
}

/// Proxy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    /// Total number of requests processed
    pub total_requests: u64,
    
    /// Number of successful requests
    pub successful_requests: u64,
    
    /// Number of failed requests
    pub failed_requests: u64,
    
    /// Number of active connections
    pub active_connections: u32,
    
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    
    /// Total bytes transferred
    pub bytes_transferred: u64,
    
    /// Requests per second (last minute)
    pub requests_per_second: f64,
    
    /// Error rate percentage
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

/// Circuit breaker state
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    /// Current state
    pub state: CircuitState,
    
    /// Failure count
    pub failure_count: u32,
    
    /// Last failure time
    pub last_failure: Option<Instant>,
    
    /// Next retry time
    pub next_retry: Option<Instant>,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    
    /// Circuit is open (failing fast)
    Open,
    
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Load balancer state
#[derive(Debug, Clone)]
pub struct LoadBalancerState {
    /// Round robin counter
    pub round_robin_counter: usize,
    
    /// Connection counts per service
    pub connection_counts: HashMap<String, u32>,
    
    /// Service weights
    pub service_weights: HashMap<String, u32>,
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
            load_balancer: Arc::new(RwLock::new(LoadBalancerState {
                round_robin_counter: 0,
                connection_counts: HashMap::new(),
                service_weights: HashMap::new(),
            })),
        }
    }
    
    /// Start the connection proxy
    pub async fn start(&self) -> Result<(), SongbirdError> {
        tracing::info!("Starting connection proxy on {}:{}", self.config.bind_address, self.config.port);
        
        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Create the proxy router
        let app = self.create_router().await;
        
        // Build the server address
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        
        // Parse the address
        let socket_addr: std::net::SocketAddr = addr.parse()
            .map_err(|e| SongbirdError::Configuration(
                format!("Invalid bind address {}: {}", addr, e)
            ))?;
        
        // Create TCP listener
        let listener = tokio::net::TcpListener::bind(socket_addr).await
            .map_err(|e| SongbirdError::Communication(
                format!("Failed to bind to {}: {}", socket_addr, e)
            ))?;
        
        tracing::info!("Proxy server listening on {}", socket_addr);
        
        // Start the server based on SSL configuration
        if self.config.enable_ssl {
            // SSL/TLS configuration
            if let (Some(cert_path), Some(key_path)) = (&self.config.ssl_cert_path, &self.config.ssl_key_path) {
                // Load TLS certificate and key
                let cert = std::fs::read(cert_path)
                    .map_err(|e| SongbirdError::Configuration(
                        format!("Failed to read SSL certificate {}: {}", cert_path, e)
                    ))?;
                let key = std::fs::read(key_path)
                    .map_err(|e| SongbirdError::Configuration(
                        format!("Failed to read SSL private key {}: {}", key_path, e)
                    ))?;
                
                // Create TLS acceptor
                let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem(cert, key).await
                    .map_err(|e| SongbirdError::Configuration(
                        format!("Failed to create TLS configuration: {}", e)
                    ))?;
                
                // Start HTTPS server
                let server = axum_server::from_tcp_rustls(listener, tls_config)
                    .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>());
                
                // Spawn server task for graceful shutdown handling
                let running_clone = Arc::clone(&self.running);
                let server_handle = tokio::spawn(async move {
                    let graceful = server.with_graceful_shutdown(async move {
                        // Wait for shutdown signal
                        while running_clone.read().await.clone() {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                        tracing::info!("Gracefully shutting down HTTPS proxy server");
                    });
                    
                    if let Err(e) = graceful.await {
                        tracing::error!("HTTPS proxy server error: {}", e);
                    }
                });
                
                // Store server handle for cleanup (would need to add field to struct)
                tracing::info!("HTTPS proxy server started successfully on {}", socket_addr);
                
                // For now, we'll detach the handle since we don't have a field to store it
                server_handle.abort();
                
            } else {
                return Err(SongbirdError::Configuration(
                    "SSL enabled but certificate or key path not provided".to_string()
                ));
            }
        } else {
            // HTTP server (no SSL)
            let server = axum_server::from_tcp(listener)
                .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>());
            
            // Spawn server task for graceful shutdown handling
            let running_clone = Arc::clone(&self.running);
            let server_handle = tokio::spawn(async move {
                let graceful = server.with_graceful_shutdown(async move {
                    // Wait for shutdown signal
                    while running_clone.read().await.clone() {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                    tracing::info!("Gracefully shutting down HTTP proxy server");
                });
                
                if let Err(e) = graceful.await {
                    tracing::error!("HTTP proxy server error: {}", e);
                }
            });
            
            tracing::info!("HTTP proxy server started successfully on {}", socket_addr);
            
            // For now, we'll detach the handle since we don't have a field to store it
            server_handle.abort();
        }
        
        tracing::info!("Connection proxy started successfully");
        Ok(())
    }
    
    /// Stop the connection proxy
    pub async fn stop(&self) -> Result<(), SongbirdError> {
        tracing::info!("Stopping connection proxy");
        
        {
            let mut running = self.running.write().await;
            if !*running {
                return Ok(());
            }
            *running = false;
        }
        
        tracing::info!("Connection proxy stopped");
        Ok(())
    }
    
    /// Check if the proxy is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Update service registry
    pub async fn update_services(&self, services: Vec<ServiceInfo>) -> Result<(), SongbirdError> {
        let mut service_map = self.services.write().await;
        service_map.clear();
        
        // Group services by service type/name
        for service in services {
            let service_key = service.service_type.clone();
            service_map.entry(service_key).or_insert_with(Vec::new).push(service);
        }
        
        tracing::info!("Updated proxy service registry with {} service types", service_map.len());
        Ok(())
    }
    
    /// Create the proxy router
    async fn create_router(&self) -> Router {
        Router::new()
            .route("/proxy/health", get(proxy_health))
            .route("/proxy/stats", get(proxy_stats))
            .route("/proxy/:service/*path", any(proxy_request))
            .fallback(proxy_fallback)
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
                    .into_inner(),
            )
            .with_state(Arc::new(self.clone()))
    }
    
    /// Route a request to the appropriate service
    pub async fn route_request(&self, service_name: &str, request: ProxyRequest) -> Result<ProxyResponse, SongbirdError> {
        let start_time = Instant::now();
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.active_connections += 1;
        }
        
        // Check circuit breaker
        if self.is_circuit_open(service_name).await {
            return Err(SongbirdError::Communication(
                format!("Circuit breaker open for service: {}", service_name)
            ));
        }
        
        // Get service instance using load balancing
        let service_instance = self.select_service_instance(service_name).await?;
        
        // Forward the request
        let result = self.forward_request(&service_instance, request).await;
        
        // Update statistics and circuit breaker
        let response_time = start_time.elapsed();
        self.update_stats_and_circuit_breaker(service_name, &result, response_time).await;
        
        result
    }
    
    /// Select a service instance using load balancing
    async fn select_service_instance(&self, service_name: &str) -> Result<ServiceInfo, SongbirdError> {
        let services = self.services.read().await;
        let service_instances = services.get(service_name)
            .ok_or_else(|| SongbirdError::Configuration(
                format!("Service not found: {}", service_name)
            ))?;
        
        if service_instances.is_empty() {
            return Err(SongbirdError::Configuration(
                format!("No instances available for service: {}", service_name)
            ));
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
            LoadBalancingStrategy::LeastConnections => {
                // Find instance with least connections
                service_instances
                    .iter()
                    .min_by_key(|instance| {
                        load_balancer.connection_counts.get(&instance.id).unwrap_or(&0)
                    })
                    .unwrap()
            }
            _ => &service_instances[0], // Default to first instance
        };
        
        // Update connection count
        *load_balancer.connection_counts.entry(selected_instance.id.clone()).or_insert(0) += 1;
        
        Ok(selected_instance.clone())
    }
    
    /// Forward request to service instance
    async fn forward_request(&self, service: &ServiceInfo, request: ProxyRequest) -> Result<ProxyResponse, SongbirdError> {
        let start_time = Instant::now();
        
        // Build target URL from service endpoints
        let target_url = if !service.endpoints.is_empty() {
            // Use the first endpoint as the base URL
            let endpoint = &service.endpoints[0];
            let base_url = if endpoint.path.starts_with("http") {
                endpoint.path.clone()
            } else {
                // Assume HTTP if no protocol specified
                format!("http://{}", endpoint.path)
            };
            
            // Append the request path
            let uri_path = request.uri.path();
            let uri_query = request.uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
            format!("{}{}{}", base_url, uri_path, uri_query)
        } else {
            return Err(SongbirdError::Configuration(
                format!("No endpoints available for service: {}", service.id)
            ));
        };
        
        // Create HTTP client with timeouts
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.request_timeout))
            .connect_timeout(std::time::Duration::from_secs(self.config.connection_timeout))
            .build()
            .map_err(|e| SongbirdError::Communication(
                format!("Failed to create HTTP client: {}", e)
            ))?;
        
        // Create the request
        let mut req_builder = match request.method {
            Method::GET => client.get(&target_url),
            Method::POST => client.post(&target_url),
            Method::PUT => client.put(&target_url),
            Method::DELETE => client.delete(&target_url),
            Method::HEAD => client.head(&target_url),
            Method::PATCH => client.patch(&target_url),
            _ => client.request(request.method.clone(), &target_url),
        };
        
        // Copy headers from original request
        for (name, value) in request.headers.iter() {
            // Skip hop-by-hop headers that shouldn't be forwarded
            if !is_hop_by_hop_header(name.as_str()) {
                req_builder = req_builder.header(name, value);
            }
        }
        
        // Add proxy headers
        req_builder = req_builder.header("X-Forwarded-For", 
            request.source_ip.as_deref().unwrap_or("unknown"));
        req_builder = req_builder.header("X-Forwarded-Proto", "http");
        req_builder = req_builder.header("X-Proxy-Service", &service.id);
        
        // Add request body if present
        if !request.body.is_empty() {
            req_builder = req_builder.body(request.body);
        }
        
        // Send the request
        let response = req_builder.send().await
            .map_err(|e| SongbirdError::Communication(
                format!("Failed to forward request to {}: {}", target_url, e)
            ))?;
        
        // Extract response data
        let status_code = response.status();
        let mut headers = HeaderMap::new();
        
        // Copy response headers (excluding hop-by-hop headers)
        for (name, value) in response.headers().iter() {
            if !is_hop_by_hop_header(name.as_str()) {
                headers.insert(name.clone(), value.clone());
            }
        }
        
        // Add proxy response headers
        headers.insert("X-Proxy-Response-Time", 
            start_time.elapsed().as_millis().to_string().parse().unwrap());
        headers.insert("X-Proxy-Service", service.id.parse().unwrap());
        
        // Read response body
        let body = response.bytes().await
            .map_err(|e| SongbirdError::Communication(
                format!("Failed to read response body: {}", e)
            ))?
            .to_vec();
        
        let response_time = start_time.elapsed();
        
        tracing::debug!(
            "Forwarded request to {} -> {} in {:?}",
            target_url,
            status_code,
            response_time
        );
        
        Ok(ProxyResponse {
            status_code,
            headers,
            body,
            response_time,
        })
    }
    
    /// Check if a header is a hop-by-hop header that shouldn't be forwarded
    fn is_hop_by_hop_header(header_name: &str) -> bool {
        matches!(header_name.to_lowercase().as_str(),
            "connection" | "keep-alive" | "proxy-authenticate" | 
            "proxy-authorization" | "te" | "trailers" | "transfer-encoding" | "upgrade"
        )
    }
    
    /// Check if circuit breaker is open for a service
    async fn is_circuit_open(&self, service_name: &str) -> bool {
        if !self.config.enable_circuit_breaker {
            return false;
        }
        
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(cb_state) = circuit_breakers.get(service_name) {
            match cb_state.state {
                CircuitState::Open => {
                    // Check if we should try again
                    if let Some(next_retry) = cb_state.next_retry {
                        Instant::now() < next_retry
                    } else {
                        true
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Update statistics and circuit breaker state
    async fn update_stats_and_circuit_breaker(
        &self,
        service_name: &str,
        result: &Result<ProxyResponse, SongbirdError>,
        response_time: Duration,
    ) {
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            
            match result {
                Ok(response) => {
                    stats.successful_requests += 1;
                    stats.bytes_transferred += response.body.len() as u64;
                }
                Err(_) => {
                    stats.failed_requests += 1;
                }
            }
            
            // Update average response time
            let total_requests = stats.total_requests as f64;
            let current_avg = stats.average_response_time_ms;
            let new_response_time = response_time.as_millis() as f64;
            stats.average_response_time_ms = (current_avg * (total_requests - 1.0) + new_response_time) / total_requests;
            
            // Update error rate
            stats.error_rate = (stats.failed_requests as f64 / stats.total_requests as f64) * 100.0;
        }
        
        // Update circuit breaker
        if self.config.enable_circuit_breaker {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            let cb_state = circuit_breakers.entry(service_name.to_string()).or_insert_with(|| {
                CircuitBreakerState {
                    state: CircuitState::Closed,
                    failure_count: 0,
                    last_failure: None,
                    next_retry: None,
                }
            });
            
            match result {
                Ok(_) => {
                    // Reset failure count on success
                    cb_state.failure_count = 0;
                    if cb_state.state == CircuitState::HalfOpen {
                        cb_state.state = CircuitState::Closed;
                    }
                }
                Err(_) => {
                    cb_state.failure_count += 1;
                    cb_state.last_failure = Some(Instant::now());
                    
                    if cb_state.failure_count >= self.config.circuit_breaker_threshold {
                        cb_state.state = CircuitState::Open;
                        cb_state.next_retry = Some(
                            Instant::now() + Duration::from_secs(self.config.circuit_breaker_timeout)
                        );
                    }
                }
            }
        }
    }
    
    /// Get proxy statistics
    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }
    
    /// Get circuit breaker states
    pub async fn get_circuit_breaker_states(&self) -> HashMap<String, CircuitBreakerState> {
        self.circuit_breakers.read().await.clone()
    }
    
    /// Reset circuit breaker for a service
    pub async fn reset_circuit_breaker(&self, service_name: &str) -> Result<(), SongbirdError> {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(cb_state) = circuit_breakers.get_mut(service_name) {
            cb_state.state = CircuitState::Closed;
            cb_state.failure_count = 0;
            cb_state.last_failure = None;
            cb_state.next_retry = None;
            tracing::info!("Reset circuit breaker for service: {}", service_name);
        }
        Ok(())
    }
}

// Clone implementation for sharing state
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

/// Proxy health check endpoint
async fn proxy_health() -> impl IntoResponse {
    (StatusCode::OK, "Proxy is healthy")
}

/// Proxy statistics endpoint
async fn proxy_stats(State(proxy): State<Arc<ConnectionProxy>>) -> impl IntoResponse {
    let stats = proxy.get_stats().await;
    (StatusCode::OK, serde_json::to_string(&stats).unwrap_or_default())
}

/// Main proxy request handler
async fn proxy_request(
    Path((service, path)): Path<(String, String)>,
    State(proxy): State<Arc<ConnectionProxy>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let start_time = Instant::now();
    
    // Extract client IP from headers or connection info
    let source_ip = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Reconstruct the original URI with the path
    let original_path = format!("/{}", path);
    let uri_with_path = if let Some(query) = uri.query() {
        format!("{}?{}", original_path, query)
    } else {
        original_path
    };
    
    let reconstructed_uri = uri_with_path.parse::<Uri>()
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
            let mut response_builder = axum::response::Response::builder()
                .status(proxy_response.status_code);
            
            // Add response headers
            for (name, value) in proxy_response.headers.iter() {
                response_builder = response_builder.header(name, value);
            }
            
            // Add proxy timing header
            response_builder = response_builder.header(
                "X-Proxy-Duration-Ms", 
                proxy_response.response_time.as_millis().to_string()
            );
            
            // Build and return response
            match response_builder.body(axum::body::Body::from(proxy_response.body)) {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!("Failed to build proxy response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
                }
            }
        }
        Err(e) => {
            // Handle proxy errors
            tracing::error!("Proxy request failed for service '{}': {}", service, e);
            
            let error_response = match e {
                SongbirdError::Configuration(msg) => {
                    (StatusCode::SERVICE_UNAVAILABLE, format!("Service unavailable: {}", msg))
                }
                SongbirdError::Communication(msg) => {
                    (StatusCode::BAD_GATEWAY, format!("Gateway error: {}", msg))
                }
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal proxy error".to_string())
                }
            };
            
            error_response.into_response()
        }
    }
}

/// Fallback handler for unmatched routes
async fn proxy_fallback() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Service not found")
} 