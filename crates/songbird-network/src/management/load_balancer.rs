//! Load balancing strategies and implementations

use songbird_errors::SongbirdError;
use std::collections::HashMap;
use std::sync::Arc;

use super::config::{LoadBalancingStrategy, NetworkConfig};

/// Load balancer manager
pub struct LoadBalancer {
    config: NetworkConfig,
    strategy: Box<dyn BalancingStrategy + Send + Sync>,
    server_stats: Arc<std::sync::Mutex<HashMap<String, ServerStats>>>,
}

/// Load balancing strategy trait
pub trait BalancingStrategy {
    /// Select the next server for request routing
    fn select_server(
        &mut self,
        servers: &[String],
        stats: &HashMap<String, ServerStats>,
    ) -> Option<String>;

    /// Update strategy state after request completion
    fn update_stats(&mut self, server: &str, success: bool, response_time: std::time::Duration);

    /// Get strategy name
    fn name(&self) -> &'static str;
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// Number of active connections
    pub active_connections: u32,
    /// Total requests processed
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time: std::time::Duration,
    /// Server health status
    pub is_healthy: bool,
    /// Last health check
    pub last_health_check: std::time::SystemTime,
}

impl LoadBalancer {
    /// Create new load balancer
    pub fn new(config: NetworkConfig) -> Result<Self, SongbirdError> {
        let strategy = create_strategy(&config.load_balancing_strategy)?;

        Ok(Self {
            config,
            strategy,
            server_stats: Arc::new(std::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Select server for request routing
    pub fn select_server(&mut self) -> Result<Option<String>, SongbirdError> {
        if !self.config.load_balancing_enabled {
            return Ok(None);
        }

        if self.config.upstream_servers.is_empty() {
            return Err(SongbirdError::config_field(
                "upstream_servers",
                "No upstream servers configured",
            ));
        }

        let stats = self.server_stats.lock().unwrap();
        let server = self
            .strategy
            .select_server(&self.config.upstream_servers, &stats);

        Ok(server)
    }

    /// Update server statistics after request completion
    pub fn update_server_stats(
        &mut self,
        server: &str,
        success: bool,
        response_time: std::time::Duration,
    ) {
        self.strategy.update_stats(server, success, response_time);

        let mut stats = self.server_stats.lock().unwrap();
        let server_stats = stats
            .entry(server.to_string())
            .or_insert_with(|| ServerStats::new());

        server_stats.total_requests += 1;
        if !success {
            server_stats.failed_requests += 1;
        }

        // Update average response time (simple moving average)
        let total_time = server_stats.avg_response_time.as_nanos() as u64
            * (server_stats.total_requests - 1)
            + response_time.as_nanos() as u64;
        server_stats.avg_response_time =
            std::time::Duration::from_nanos(total_time / server_stats.total_requests);
    }

    /// Get server statistics
    pub fn get_server_stats(&self) -> HashMap<String, ServerStats> {
        self.server_stats.lock().unwrap().clone()
    }

    /// Get load balancer summary
    pub fn get_summary(&self) -> LoadBalancerSummary {
        let stats = self.server_stats.lock().unwrap();

        let total_requests: u64 = stats.values().map(|s| s.total_requests).sum();
        let total_failures: u64 = stats.values().map(|s| s.failed_requests).sum();
        let healthy_servers = stats.values().filter(|s| s.is_healthy).count();

        LoadBalancerSummary {
            strategy: self.strategy.name(),
            total_servers: self.config.upstream_servers.len(),
            healthy_servers,
            total_requests,
            total_failures,
            success_rate: if total_requests > 0 {
                ((total_requests - total_failures) as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Perform health check on all servers
    pub async fn health_check(&mut self) -> Result<(), SongbirdError> {
        let mut stats = self.server_stats.lock().unwrap();

        for server in &self.config.upstream_servers {
            let is_healthy = self.check_server_health(server).await?;

            let server_stats = stats
                .entry(server.clone())
                .or_insert_with(|| ServerStats::new());
            server_stats.is_healthy = is_healthy;
            server_stats.last_health_check = std::time::SystemTime::now();
        }

        Ok(())
    }

    /// Check individual server health
    async fn check_server_health(&self, server: &str) -> Result<bool, SongbirdError> {
        // Parse server URL
        let health_url = if server.starts_with("http") {
            format!("{}/health", server)
        } else {
            format!("http://{}/health", server)
        };

        // Perform health check with timeout
        let client = reqwest::Client::builder()
            .timeout(self.config.health_check.timeout)
            .build()
            .map_err(|e| {
                SongbirdError::network_error(&format!("Failed to create HTTP client: {}", e))
            })?;

        match client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Load balancer summary
#[derive(Debug, Clone)]
pub struct LoadBalancerSummary {
    pub strategy: &'static str,
    pub total_servers: usize,
    pub healthy_servers: usize,
    pub total_requests: u64,
    pub total_failures: u64,
    pub success_rate: f64,
}

impl ServerStats {
    /// Create new server statistics
    fn new() -> Self {
        Self {
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_response_time: std::time::Duration::from_millis(0),
            is_healthy: true,
            last_health_check: std::time::SystemTime::now(),
        }
    }

    /// Calculate failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
}

/// Round-robin load balancing strategy
pub struct RoundRobinStrategy {
    current_index: std::sync::atomic::AtomicUsize,
}

impl RoundRobinStrategy {
    fn new() -> Self {
        Self {
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl BalancingStrategy for RoundRobinStrategy {
    fn select_server(
        &mut self,
        servers: &[String],
        _stats: &HashMap<String, ServerStats>,
    ) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        let index = self
            .current_index
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            % servers.len();
        Some(servers[index].clone())
    }

    fn update_stats(&mut self, _server: &str, _success: bool, _response_time: std::time::Duration) {
        // Round-robin doesn't need to track individual stats
    }

    fn name(&self) -> &'static str {
        "round_robin"
    }
}

/// Least connections load balancing strategy
pub struct LeastConnectionsStrategy;

impl LeastConnectionsStrategy {
    fn new() -> Self {
        Self
    }
}

impl BalancingStrategy for LeastConnectionsStrategy {
    fn select_server(
        &mut self,
        servers: &[String],
        stats: &HashMap<String, ServerStats>,
    ) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        servers
            .iter()
            .min_by_key(|server| {
                stats
                    .get(*server)
                    .map(|s| s.active_connections)
                    .unwrap_or(0)
            })
            .cloned()
    }

    fn update_stats(&mut self, _server: &str, _success: bool, _response_time: std::time::Duration) {
        // Stats are tracked by the LoadBalancer itself
    }

    fn name(&self) -> &'static str {
        "least_connections"
    }
}

/// IP hash load balancing strategy
pub struct IpHashStrategy;

impl IpHashStrategy {
    fn new() -> Self {
        Self
    }

    /// Hash client IP to select server (simplified implementation)
    fn hash_ip(&self, ip: &str, server_count: usize) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish() as usize % server_count
    }
}

impl BalancingStrategy for IpHashStrategy {
    fn select_server(
        &mut self,
        servers: &[String],
        _stats: &HashMap<String, ServerStats>,
    ) -> Option<String> {
        if servers.is_empty() {
            return None;
        }

        // In a real implementation, we would get the client IP from the request context
        // For now, use a mock IP
        let client_ip = "127.0.0.1";
        let index = self.hash_ip(client_ip, servers.len());
        Some(servers[index].clone())
    }

    fn update_stats(&mut self, _server: &str, _success: bool, _response_time: std::time::Duration) {
        // IP hash doesn't need to track individual stats
    }

    fn name(&self) -> &'static str {
        "ip_hash"
    }
}

/// Create load balancing strategy based on configuration
fn create_strategy(
    strategy: &LoadBalancingStrategy,
) -> Result<Box<dyn BalancingStrategy + Send + Sync>, SongbirdError> {
    match strategy {
        LoadBalancingStrategy::RoundRobin => Ok(Box::new(RoundRobinStrategy::new())),
        LoadBalancingStrategy::LeastConnections => Ok(Box::new(LeastConnectionsStrategy::new())),
        LoadBalancingStrategy::IpHash => Ok(Box::new(IpHashStrategy::new())),
    }
}
