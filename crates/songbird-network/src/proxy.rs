//! Connection proxy and request routing for Songbird Orchestrator

use axum::http::{HeaderMap, Method, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use songbird_discovery::traits::service::ServiceInfo;
use songbird_errors::SongbirdError;

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    LeastConnections,
    WeightedRoundRobin,
    HealthBased,
}

/// Connection proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub bind_address: String,
    pub port: u16,
    pub enable_logging: bool,
    pub request_timeout: u64,
    pub connection_timeout: u64,
    pub max_retries: u32,
    pub enable_circuit_breaker: bool,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: u64,
    pub enable_load_balancing: bool,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub enable_ssl: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
    pub enable_compression: bool,
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

/// Circuit breaker state
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure: Option<Instant>,
    pub next_retry: Option<Instant>,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Load balancer state
#[derive(Debug, Clone)]
pub struct LoadBalancerState {
    pub round_robin_counter: usize,
    pub connection_counts: HashMap<String, u32>,
    pub service_weights: HashMap<String, u32>,
}

/// Connection proxy
#[derive(Debug)]
pub struct ConnectionProxy {
    config: ProxyConfig,
    services: Arc<RwLock<HashMap<String, Vec<ServiceInfo>>>>,
    running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<ProxyStats>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
    load_balancer: Arc<RwLock<LoadBalancerState>>,
}

impl ConnectionProxy {
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

    pub async fn start(&self) -> Result<(), SongbirdError> {
        tracing::info!(
            "Starting connection proxy on {}:{}",
            self.config.bind_address,
            self.config.port
        );

        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        tracing::info!("Connection proxy started successfully");
        Ok(())
    }

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

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn update_services(&self, services: Vec<ServiceInfo>) -> Result<(), SongbirdError> {
        let mut service_map = self.services.write().await;
        service_map.clear();

        for service in services {
            let service_key = service.service_type.clone();
            service_map
                .entry(service_key)
                .or_insert_with(Vec::new)
                .push(service);
        }

        tracing::info!(
            "Updated proxy service registry with {} service types",
            service_map.len()
        );
        Ok(())
    }

    pub async fn route_request(
        &self,
        service_name: &str,
        _request: ProxyRequest,
    ) -> Result<ProxyResponse, SongbirdError> {
        let _start_time = Instant::now();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.active_connections += 1;
        }

        // Get service instance using load balancing
        let service_instance = self.select_service_instance(service_name).await?;

        // Simulate forwarding the request
        let response_time = Duration::from_millis(50);

        let result = Ok(ProxyResponse {
            status_code: StatusCode::OK,
            headers: HeaderMap::new(),
            body: format!(
                "Proxied to service: {} ({})",
                service_instance.name, service_instance.service_id
            )
            .into_bytes(),
            response_time,
        });

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            stats.successful_requests += 1;

            if let Ok(response) = &result {
                stats.bytes_transferred += response.body.len() as u64;
            }

            let total_requests = stats.total_requests as f64;
            let current_avg = stats.average_response_time_ms;
            let new_response_time = response_time.as_millis() as f64;
            stats.average_response_time_ms =
                (current_avg * (total_requests - 1.0) + new_response_time) / total_requests;

            stats.error_rate = (stats.failed_requests as f64 / stats.total_requests as f64) * 100.0;
        }

        result
    }

    async fn select_service_instance(
        &self,
        service_name: &str,
    ) -> Result<ServiceInfo, SongbirdError> {
        let services = self.services.read().await;
        let service_instances =
            services
                .get(service_name)
                .ok_or_else(|| SongbirdError::Configuration {
                    field: "service_name".to_string(),
                    message: format!("Service not found: {}", service_name),
                })?;

        if service_instances.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "service_instances".to_string(),
                message: format!("No instances available for service: {}", service_name),
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
                .ok_or_else(|| SongbirdError::Config {
                    message: "No service instances available for least connections selection"
                        .to_string(),
                    field: None,
                })?,
            _ => &service_instances[0],
        };
        *load_balancer
            .connection_counts
            .entry(selected_instance.service_id.clone())
            .or_insert(0) += 1;

        Ok(selected_instance.clone())
    }

    pub async fn get_stats(&self) -> ProxyStats {
        self.stats.read().await.clone()
    }

    pub async fn get_circuit_breaker_states(&self) -> HashMap<String, CircuitBreakerState> {
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
