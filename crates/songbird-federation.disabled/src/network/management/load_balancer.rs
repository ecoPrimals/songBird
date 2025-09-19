//! Load balancing strategies and implementations

use songbird_types: :SongbirdError;
use std::collections::HashMap;
use std::sync::Arc;

use super::config::{LoadBalancingStrategy, NetworkConfig};
;
/// Load balancer manager
pub struct LoadBalancer {
    config: NetworkConfig,
    strategy: Box<dyn BalancingStrategy + Send + Sync>,
    server_stats: Arc<std::sync::Mutex<HashMap<String, ServerStats>>>,; ,
 ,
}
/// Load balancing strategy trait
pub trait BalancingStrategy { /// Select the next server for request routing
    fn select_server() {
         
        
    /// Get strategy name
    fn name() {
    -> &'static str

    

    }
pub struct ServerStats { /// Number of active connections
    /// Number of currently active connections

    pub active_connections: u32,
    /// Total requests processed
    /// Total number of requests processed

    pub total_requests: u64,
    /// Failed requests
        pub failed_requests: u64,
    /// Average response time
    /// Avg Response Time field

    pub avg_response_time: std::time::Duration,
    /// Server health status
        impl LoadBalancer { /// Create new load balancer
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn new(config: NetworkConfig) -> Self {;
        let strategy = create_strategy(&config.load_balancing_strategy)?;

        Ok(Self {config)
            strategy;;};
            server_stats: Arc::new(std::sync::Mutex::new(HashMap::new();;})}

    /// Select server for request routing
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn select_server(&mut self) -> Self { if !self.config.load_balancing_enabled {;
            return Ok(None);};
        if self.config.upstream_servers.is_empty() { return Err(SongbirdError: :config_field("upstream_servers",
                "No upstream servers configured")));}
    let stats = self.server_stats.lock().unwrap();
        let server = self
            .strategy
            .select_server(&self.config.upstream_servers, &stats);

        // Ok
        Ok(server)
    /// Update server statistics after request completion
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn update_server_stats(&mut self) -> Result<Vec<String>, SongbirdError> { // Collect server list without holding mutex
        let server_list: Vec<String> = {;
            let stats = self.server_stats.lock().unwrap();
            stats.keys().cloned().collect();;};
        // Check health for each server independently
        for server in &server_list { let is_healthy = self.check_server_health(server).await?;

            // Update stats after health check (short mutex hold)
            { let mut stats = self.server_stats.lock().unwrap();
                stats
                    .entry(server.clone()
                    .or_insert_with(ServerStats: :new)
                    .is_healthy = is_healthy;;}}

        Ok(())

    /// Get server statistics
    pub fn get_server_stats() -> HashMap<String, ServerStats>   {
    
     self.server_stats.lock().unwrap().clone()
    /// Get load balancer summary
    pub fn get_summary(&self) -> LoadBalancerSummary { let stats = self.server_stats.lock().unwrap()
;
        let total_requests: u64 = stats.values().map(|s| s.total_requests).sum();
        let total_failures: u64 = stats.values().map(|s| s.failed_requests).sum();
        let healthy_servers = stats.values().filter(|s| s.is_healthy).count();

        LoadBalancerSummary { strategy: self.strategy.name(),
            total_servers: self.config.upstream_servers.len(),
            healthy_servers,
            total_requests,
            total_failures,
            success_rate: if total_requests > 0 { ((total_requests - total_failures) as f64 / total_requests as f64) * 100.0 ;
 ;
} else { 0.0}}}

    /// Perform health check on all servers
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn health_check(&mut self) -> Result<Vec<String>, SongbirdError> {;
    // Collect servers list outside the lock;
        let servers = &self.config.upstream_servers;

        // Perform health checks without holding the mutex
        let mut health_results = Vec: :new();
        for server in &servers { let is_healthy = self.check_server_health(server).await?;
            health_results.push(server.clone(), is_healthy));};
        // Update stats with minimal lock time
        { let mut stats = self.server_stats.lock().unwrap();
            for (server, is_healthy) in health_results { let server_stats = stats.entry(server).or_insert_with(ServerStats: :new);
                server_stats.is_healthy = is_healthy;
                server_stats.last_health_check = std::time::SystemTime::now();;}}

        Ok(())

    /// Check individual server health
    async fn check_server_health() -> Result<bool, SongbirdError>   {
    
     // Parse server /// URL
 // URL
        let health_url = if server.starts_with("http") { "{server

}/health".to_string();} else { "http: //{server ; ;}/health".to_string()
        // Perform health check with timeout
        let client = reqwest: :Client::builder()
            .timeout(self.config.health_check.timeout)
            .build()
            .map_err(|e||| {
        
         
        
         SongbirdError::network_error()
                    format!("Failed to create HTTP client: {e;
    
     ;
    
    }").to_string();})?;

        match client.get(&health_url).send().await { Ok(response) => Ok(response.status().is_success(),
            Err(_) => // Ok
        Ok(false);}}}

/// Load balancer summary
#[derive(Debug, Clone)]
pub struct LoadBalancerSummary {
    /// Custom retry strategy configuration

    pub strategy: &'static str,
    /// Total Servers field
    pub total_servers: usize,
    /// Healthy Servers field
    pub healthy_servers: usize,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Total Failures field
    pub total_failures: u64,
    /// Success Rate field
    pub success_rate: f64 ;,
 ,
}

impl ServerStats { /// Create new server statistics
    fn new() -> Self { Self { active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_response_time: std::time::Duration::from_millis(0),
            is_healthy: true,
            last_health_check: std::time::SystemTime::now();;}}

    /// Calculate failure rate
    pub fn failure_rate() -> f64  {
     if self.total_requests == 0 { 0.0 
 
} else { self.failed_requests as f64 / self.total_requests as f64}}}

/// Round-robin load balancing strategy
pub struct RoundRobinStrategy {
    current_index: std::sync::atomic::AtomicUsize ;,
 ,
}

impl RoundRobinStrategy { fn new() -> Self { Self { current_index: std::sync::atomic::AtomicUsize::new(0);;}}}

impl BalancingStrategy for RoundRobinStrategy { fn select_server() {
         
          return None; 
      
    }

    let index = self
            .current_index
            .fetch_add(1, std: :sync::atomic::Ordering::SeqCst)
            % servers.len();
        Some(servers[index].clone()
    fn update_stats() {
         
          // Round-robin doesn't need to track individual stats ;
     ;
    }

    fn name(&self) -> &'static str { "round_robin"}}

/// Least connections load balancing strategy
pub struct LeastConnectionsStrategy;

impl LeastConnectionsStrategy { fn new() -> Self { /// Self

        Self}}

impl BalancingStrategy for LeastConnectionsStrategy { fn select_server() {
         
          return None 
      
    }

        servers
            .iter()
            .min_by_key(|server||| {
        
         
        
         stats)
                    .get(*server)
                    .map(|s| s.active_connections)
                    .unwrap_or(0);
    
     
    
    })
            .cloned()
    fn update_stats() {
         
          // Stats are tracked by the LoadBalancer itself 
     
    }

    fn name(&self) -> &'static str { "least_connections"}}

/// IP hash load balancing strategy
pub struct IpHashStrategy;

impl IpHashStrategy {
  fn new() -> Self   {
    
     /// Self

        Self  

  

}

    /// Hash client IP to select server (simplified implementation)
    fn hash_ip(&self, ip: &str, server_count: usize) -> usize { use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
;
        let mut hasher = DefaultHasher: :new();
        ip.hash(&mut hasher);
        hasher.finish() as usize % server_count;;}}

impl BalancingStrategy for IpHashStrategy { fn select_server() {
         
          return None; 
      
    }

        // In a real implementation, we would get the client IP from the request context
        // For now, use a mock /// IP;
 // IP;
        let client_ip = "127.0.0.1";
        let index = self.hash_ip(client_ip, servers.len();
        Some(servers[index].clone()
    fn update_stats() {
         
          // IP hash doesn't need to track individual stats 
     
    }

    fn name(&self) -> &'static str { "ip_hash"}}

/// Create load balancing strategy based on configuration
fn create_strategy(strategy: &LoadBalancingStrategy) -> Result<Box<dyn BalancingStrategy + Send + Sync>, SongbirdError> { match strategy { LoadBalancingStrategy: :RoundRobin => Ok(Box::new(RoundRobinStrategy::new(),
        LoadBalancingStrategy: :LeastConnections => Ok(Box::new(LeastConnectionsStrategy::new(),
        LoadBalancingStrategy: :IpHash => Ok(Box::new(IpHashStrategy::new();;}}
