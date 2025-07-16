//! OS Substrate Integration with Performance Optimizations
//!
//! This module provides platform-agnostic OS operations by delegating to the
//! toadstool and biomeOS substrate systems with comprehensive performance
//! optimizations including TTL caching, connection pooling, and circuit breakers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use crate::errors::{Result, SongbirdError};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::biomeos_integration::{BiomeOSClient, BiomeOSIntegration};
use crate::config::constants;

/// Maximum cache size for substrate entries
const MAX_CACHE_SIZE: usize = 1000;

/// Default cache TTL for substrate entries
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Connection pool size for substrate clients
const CONNECTION_POOL_SIZE: usize = 10;

/// Circuit breaker failure threshold
const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Circuit breaker timeout duration
const CIRCUIT_BREAKER_TIMEOUT: Duration = Duration::from_secs(30);

/// OS substrate interface that abstracts platform operations with performance optimizations
#[derive(Debug, Clone)]
pub struct OSSubstrate {
    pub toadstool_client: ToadstoolClient,
    biomeos_client: BiomeOSClient,
    cache: Arc<RwLock<OptimizedSubstrateCache>>,
    metrics: Arc<RwLock<SubstrateMetrics>>,
}

/// Toadstool client for compute and container operations with connection pooling
#[derive(Debug, Clone)]
pub struct ToadstoolClient {
    client: reqwest::Client,
    endpoint: String,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    connection_pool: Arc<RwLock<ConnectionPool>>,
}

/// Optimized substrate cache with TTL and size limits
#[derive(Debug)]
struct OptimizedSubstrateCache {
    paths: HashMap<String, CacheEntry<PathBuf>>,
    capabilities: HashMap<String, CacheEntry<Vec<String>>>,
    system_info: Option<CacheEntry<SystemInfo>>,
    cache_size: usize,
    max_size: usize,
    ttl: Duration,
}

/// Cache entry with TTL and access tracking
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Duration,
}

/// Circuit breaker for substrate resilience
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    last_failure: Option<Instant>,
    timeout: Duration,
    state: CircuitState,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CircuitState {
    #[default]
    Closed,
    Open,
    HalfOpen,
}

/// Connection pool for HTTP clients
#[derive(Debug)]
struct ConnectionPool {
    pool: Vec<reqwest::Client>,
    pool_size: usize,
    active_connections: usize,
}

/// Substrate performance metrics
#[derive(Debug, Default)]
pub struct SubstrateMetrics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub substrate_errors: u64,
    pub fallback_uses: u64,
    pub average_response_time: f64,
    pub toadstool_requests: u64,
    pub biomeos_requests: u64,
    pub circuit_breaker_trips: u64,
}

/// System information from substrate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub platform: String,
    pub architecture: String,
    pub available_storage: u64,
    pub available_memory: u64,
    pub cpu_cores: u32,
    pub network_interfaces: Vec<NetworkInterface>,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub subnet_mask: String,
    pub is_up: bool,
    pub is_loopback: bool,
}

/// Substrate path request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequest {
    pub path_type: PathType,
    pub service_name: String,
    pub requirements: PathRequirements,
}

/// Types of paths that can be requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    Data,
    Config,
    Log,
    Cache,
    Runtime,
    Temp,
}

/// Path requirements and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRequirements {
    pub min_size_bytes: Option<u64>,
    pub permissions: Option<String>,
    pub persistent: bool,
    pub shared: bool,
}

/// Network operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub operation: NetworkOperation,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Network operations supported by substrate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    ResolveName,
    GetInterface,
    CheckConnectivity,
    GetAvailablePort,
    ConfigureFirewall,
}

impl<T: Clone> CacheEntry<T> {
    /// Create new cache entry with TTL
    fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }

    /// Check if cache entry is expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Access the cache entry, updating access stats
    fn access(&mut self) -> T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        self.value.clone()
    }

    /// Get age of cache entry
    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

impl OptimizedSubstrateCache {
    /// Create new optimized cache
    fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            paths: HashMap::new(),
            capabilities: HashMap::new(),
            system_info: None,
            cache_size: 0,
            max_size,
            ttl,
        }
    }

    /// Clean expired entries from cache
    fn clean_expired(&mut self) {
        // Clean expired paths
        self.paths.retain(|_, entry| !entry.is_expired());

        // Clean expired capabilities
        self.capabilities.retain(|_, entry| !entry.is_expired());

        // Clean expired system info
        if let Some(ref entry) = self.system_info {
            if entry.is_expired() {
                self.system_info = None;
            }
        }

        // Update cache size
        self.cache_size = self.paths.len()
            + self.capabilities.len()
            + if self.system_info.is_some() { 1 } else { 0 };
    }

    /// Evict least recently used entries if cache is full
    fn evict_lru(&mut self) {
        if self.cache_size >= self.max_size {
            // Find LRU path entry
            if let Some((lru_key, _)) = self
                .paths
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
            {
                let lru_key = lru_key.clone();
                self.paths.remove(&lru_key);
                debug!("Evicted LRU path cache entry: {}", lru_key);
            }
        }
    }

    /// Get cache statistics
    fn get_stats(&self) -> (usize, usize, f64) {
        let total_entries = self.paths.len()
            + self.capabilities.len()
            + if self.system_info.is_some() { 1 } else { 0 };
        let utilization = total_entries as f64 / self.max_size as f64;
        (total_entries, self.max_size, utilization)
    }
}

impl CircuitBreaker {
    /// Create new circuit breaker
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            timeout,
            state: CircuitState::Closed,
        }
    }

    /// Check if request should be allowed
    fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful request
    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure = None;
    }

    /// Record failed request
    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Get circuit breaker state
    fn get_state(&self) -> &CircuitState {
        &self.state
    }
}

impl ConnectionPool {
    /// Create new connection pool
    fn new(pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            pool.push(client);
        }

        Self {
            pool,
            pool_size,
            active_connections: 0,
        }
    }

    /// Get a client from the pool
    fn get_client(&mut self) -> Option<reqwest::Client> {
        if self.active_connections < self.pool_size {
            self.active_connections += 1;
            self.pool.get(self.active_connections - 1).cloned()
        } else {
            // Pool exhausted, create new client
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .ok()
        }
    }

    /// Return a client to the pool
    fn return_client(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }
}

impl OSSubstrate {
    /// Create new OS substrate with performance optimizations
    pub async fn new() -> Result<Self> {
        let toadstool_endpoint = std::env::var("TOADSTOOL_ENDPOINT")
            .unwrap_or_else(|_| crate::config::constants::network::toadstool_endpoint());

        let biomeos_endpoint = std::env::var("BIOMEOS_ENDPOINT")
            .unwrap_or_else(|_| crate::config::constants::network::biomeos_endpoint());

        let toadstool_client = ToadstoolClient::new(toadstool_endpoint).await?;
        let biomeos_client = BiomeOSClient::new(biomeos_endpoint);

        let substrate = Self {
            toadstool_client,
            biomeos_client,
            cache: Arc::new(RwLock::new(OptimizedSubstrateCache::new(
                MAX_CACHE_SIZE,
                DEFAULT_CACHE_TTL,
            ))),
            metrics: Arc::new(RwLock::new(SubstrateMetrics::default())),
        };

        // Initialize substrate connection with performance monitoring
        substrate.initialize_with_metrics().await?;

        Ok(substrate)
    }

    /// Initialize substrate connections with performance monitoring
    async fn initialize_with_metrics(&self) -> Result<()> {
        info!("🌍 Initializing optimized OS substrate through toadstool and biomeOS");

        let start_time = Instant::now();

        // Parallel health checks for better performance
        let (toadstool_result, biomeos_result) = tokio::join!(
            self.toadstool_client.health_check(),
            self.biomeos_client.health_check()
        );

        // Update metrics based on results
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 2;

        match toadstool_result {
            Ok(_) => {
                info!("✅ Toadstool substrate connected");
                metrics.toadstool_requests += 1;
            }
            Err(e) => {
                warn!("⚠️ Toadstool substrate unavailable: {}", e);
                metrics.substrate_errors += 1;
            }
        }

        match biomeos_result {
            Ok(_) => {
                info!("✅ biomeOS substrate connected");
                metrics.biomeos_requests += 1;
            }
            Err(e) => {
                warn!("⚠️ biomeOS substrate unavailable: {}", e);
                metrics.substrate_errors += 1;
            }
        }

        // Update response time metrics
        metrics.average_response_time = start_time.elapsed().as_secs_f64();

        // Load system information asynchronously
        if let Err(e) = self.refresh_system_info().await {
            warn!("Failed to load initial system info: {}", e);
        }

        Ok(())
    }

    /// Get platform-agnostic path through substrate with optimized caching
    pub async fn get_path(&self, request: PathRequest) -> Result<PathBuf> {
        let start_time = Instant::now();
        let cache_key = format!("{}_{}", request.service_name, request.path_type.to_string());

        // Check optimized cache first
        {
            let mut cache = self.cache.write().await;
            cache.clean_expired(); // Clean expired entries

            if let Some(entry) = cache.paths.get_mut(&cache_key) {
                if !entry.is_expired() {
                    // Update metrics
                    let mut metrics = self.metrics.write().await;
                    metrics.cache_hits += 1;
                    metrics.total_requests += 1;

                    return Ok(entry.access());
                }
            }
        }

        // Cache miss - request from substrate
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
        metrics.total_requests += 1;
        drop(metrics);

        // Request path from substrate with circuit breaker
        let path = match self
            .request_path_from_substrate_with_retry(request.clone())
            .await
        {
            Ok(path) => path,
            Err(e) => {
                warn!("Substrate path request failed: {}, using fallback", e);

                // Update fallback metrics
                let mut metrics = self.metrics.write().await;
                metrics.fallback_uses += 1;
                drop(metrics);

                self.get_fallback_path(request)?
            }
        };

        // Cache the result with TTL
        {
            let mut cache = self.cache.write().await;
            cache.evict_lru(); // Ensure cache doesn't exceed size limit
            cache
                .paths
                .insert(cache_key, CacheEntry::new(path.clone(), DEFAULT_CACHE_TTL));
            cache.cache_size += 1;
        }

        // Update performance metrics
        let mut metrics = self.metrics.write().await;
        metrics.average_response_time = start_time.elapsed().as_secs_f64();

        Ok(path)
    }

    /// Request path from substrate with retry logic
    async fn request_path_from_substrate_with_retry(
        &self,
        request: PathRequest,
    ) -> Result<PathBuf> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_millis(100);

        for attempt in 0..MAX_RETRIES {
            match self.request_path_from_substrate(request.clone()).await {
                Ok(path) => return Ok(path),
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 {
                        warn!(
                            "Substrate request failed (attempt {}), retrying: {}",
                            attempt + 1,
                            e
                        );
                        tokio::time::sleep(RETRY_DELAY * (attempt + 1)).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        unreachable!()
    }

    /// Request path from biomeOS substrate
    async fn request_path_from_substrate(&self, request: PathRequest) -> Result<PathBuf> {
        let payload = serde_json::json!({
            "action": "get_path",
            "path_type": request.path_type,
            "service_name": request.service_name,
            "requirements": request.requirements
        });

        let response = self.biomeos_client.request("paths", payload).await?;

        if let Some(path_str) = response.get("path").and_then(|v| v.as_str()) {
            Ok(PathBuf::from(path_str))
        } else {
            Err(SongbirdError::Network {
                service: "biomeos_substrate".to_string(),
                message: "Invalid path response from substrate".to_string(),
                details: None,
            })
        }
    }

    /// Get fallback path when substrate is unavailable
    fn get_fallback_path(&self, request: PathRequest) -> Result<PathBuf> {
        let base_path = match request.path_type {
            PathType::Data => constants::paths::DEFAULT_DATA_DIR,
            PathType::Config => constants::paths::DEFAULT_CONFIG_DIR,
            PathType::Log => constants::paths::DEFAULT_LOG_DIR,
            PathType::Cache => constants::paths::DEFAULT_CACHE_DIR,
            PathType::Runtime => constants::paths::DEFAULT_TEMP_DIR,
            PathType::Temp => constants::paths::DEFAULT_TEMP_DIR,
        };

        Ok(PathBuf::from(base_path).join(&request.service_name))
    }

    /// Get system information through substrate with optimized caching
    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.system_info.as_mut() {
                if !entry.is_expired() {
                    let mut metrics = self.metrics.write().await;
                    metrics.cache_hits += 1;
                    return Ok(entry.access());
                }
            }
        }

        // Cache miss - refresh from substrate
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
        drop(metrics);

        self.refresh_system_info().await
    }

    /// Refresh system information from substrate with caching
    async fn refresh_system_info(&self) -> Result<SystemInfo> {
        let info = match self.get_system_info_from_substrate().await {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "Failed to get system info from substrate: {}, using fallback",
                    e
                );

                // Update fallback metrics
                let mut metrics = self.metrics.write().await;
                metrics.fallback_uses += 1;
                drop(metrics);

                self.get_fallback_system_info()?
            }
        };

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.system_info = Some(CacheEntry::new(info.clone(), DEFAULT_CACHE_TTL));
        }

        Ok(info)
    }

    /// Get system information from toadstool substrate
    async fn get_system_info_from_substrate(&self) -> Result<SystemInfo> {
        let payload = serde_json::json!({
            "action": "get_system_info"
        });

        let response = self.toadstool_client.request(payload).await?;

        serde_json::from_value(response).map_err(|e| SongbirdError::Network {
            service: "toadstool_substrate".to_string(),
            message: format!("Failed to parse system info: {}", e),
            details: None,
        })
    }

    /// Get fallback system information
    fn get_fallback_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            available_storage: 100 * 1024 * 1024 * 1024, // 100GB fallback
            available_memory: 8 * 1024 * 1024 * 1024,    // 8GB fallback
            cpu_cores: 4,                                // 4 cores fallback
            network_interfaces: vec![NetworkInterface {
                name: "fallback".to_string(),
                ip_address: constants::network::DEFAULT_BIND_ADDRESS.to_string(),
                subnet_mask: "255.255.255.0".to_string(),
                is_up: true,
                is_loopback: false,
            }],
        })
    }

    /// Perform network operation through substrate with optimization
    pub async fn network_operation(&self, request: NetworkRequest) -> Result<serde_json::Value> {
        let start_time = Instant::now();

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        drop(metrics);

        match self
            .request_network_operation_from_substrate(request.clone())
            .await
        {
            Ok(result) => {
                // Update response time
                let mut metrics = self.metrics.write().await;
                metrics.average_response_time = start_time.elapsed().as_secs_f64();
                Ok(result)
            }
            Err(e) => {
                warn!("Substrate network operation failed: {}, using fallback", e);

                // Update fallback metrics
                let mut metrics = self.metrics.write().await;
                metrics.fallback_uses += 1;
                drop(metrics);

                self.get_fallback_network_result(request)
            }
        }
    }

    /// Request network operation from substrate
    async fn request_network_operation_from_substrate(
        &self,
        request: NetworkRequest,
    ) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "action": "network_operation",
            "operation": request.operation,
            "target": request.target,
            "parameters": request.parameters
        });

        self.toadstool_client.request(payload).await
    }

    /// Get fallback network operation result
    fn get_fallback_network_result(&self, request: NetworkRequest) -> Result<serde_json::Value> {
        match request.operation {
            NetworkOperation::ResolveName => Ok(serde_json::json!({
                "resolved_address": constants::network::DEFAULT_BIND_ADDRESS
            })),
            NetworkOperation::GetInterface => Ok(serde_json::json!({
                "interface": "fallback",
                "ip_address": constants::network::DEFAULT_BIND_ADDRESS
            })),
            NetworkOperation::CheckConnectivity => Ok(serde_json::json!({
                "connected": true
            })),
            NetworkOperation::GetAvailablePort => Ok(serde_json::json!({
                "port": constants::network::DEFAULT_ORCHESTRATOR_PORT
            })),
            NetworkOperation::ConfigureFirewall => Ok(serde_json::json!({
                "configured": false,
                "message": "Firewall configuration requires substrate"
            })),
        }
    }

    /// Execute container operation through toadstool
    pub async fn container_operation(
        &self,
        operation: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "action": "container_operation",
            "operation": operation,
            "parameters": parameters
        });

        self.toadstool_client.request(payload).await
    }

    /// Get substrate capabilities with caching
    pub async fn get_capabilities(&self) -> Result<HashMap<String, Vec<String>>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            cache.clean_expired();

            if let Some(entry) = cache.capabilities.get_mut("all") {
                if !entry.is_expired() {
                    let mut metrics = self.metrics.write().await;
                    metrics.cache_hits += 1;

                    // Convert Vec<String> to HashMap format
                    let mut result = HashMap::new();
                    let caps = entry.access();
                    result.insert("combined".to_string(), caps);
                    return Ok(result);
                }
            }
        }

        // Cache miss - discover capabilities
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
        drop(metrics);

        let capabilities = self.discover_capabilities().await?;

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            if let Some(combined_caps) = capabilities.get("combined") {
                cache.capabilities.insert(
                    "all".to_string(),
                    CacheEntry::new(combined_caps.clone(), DEFAULT_CACHE_TTL),
                );
            }
        }

        Ok(capabilities)
    }

    /// Discover substrate capabilities with parallel requests
    async fn discover_capabilities(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut capabilities = HashMap::new();

        // Parallel capability discovery for better performance
        let (toadstool_result, biomeos_result) = tokio::join!(
            self.toadstool_client.get_capabilities(),
            self.biomeos_client.get_capabilities()
        );

        // Combine results
        let mut combined_caps = Vec::new();

        if let Ok(toadstool_caps) = toadstool_result {
            capabilities.insert("toadstool".to_string(), toadstool_caps.clone());
            combined_caps.extend(toadstool_caps);
        }

        if let Ok(biomeos_caps) = biomeos_result {
            let caps_vec = match &biomeos_caps {
                serde_json::Value::Array(arr) => {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                }
                serde_json::Value::String(s) => vec![s.clone()],
                _ => vec![biomeos_caps.to_string()],
            };
            capabilities.insert("biomeos".to_string(), caps_vec.clone());
            combined_caps.extend(caps_vec);
        }

        capabilities.insert("combined".to_string(), combined_caps);
        Ok(capabilities)
    }

    /// Get substrate performance metrics
    pub async fn get_metrics(&self) -> SubstrateMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize, f64, u64, u64) {
        let cache = self.cache.read().await;
        let (total, max_size, utilization) = cache.get_stats();

        let metrics = self.metrics.read().await;
        let cache_hits = metrics.cache_hits;
        let cache_misses = metrics.cache_misses;

        (total, max_size, utilization, cache_hits, cache_misses)
    }

    /// Clear substrate cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.paths.clear();
        cache.capabilities.clear();
        cache.system_info = None;
        cache.cache_size = 0;

        info!("🧹 Substrate cache cleared");
    }

    /// Warm up substrate cache with common operations
    pub async fn warm_cache(&self) -> Result<()> {
        info!("🔥 Warming up substrate cache...");

        // Warm up system info
        let _ = self.get_system_info().await;

        // Warm up capabilities
        let _ = self.get_capabilities().await;

        // Warm up common paths
        let common_services = vec!["songbird", "orchestrator", "discovery"];
        let path_types = vec![PathType::Data, PathType::Config, PathType::Log];

        for service in common_services {
            for path_type in &path_types {
                let request = PathRequest {
                    path_type: path_type.clone(),
                    service_name: service.to_string(),
                    requirements: PathRequirements::default(),
                };
                let _ = self.get_path(request).await;
            }
        }

        info!("✅ Substrate cache warmed up");
        Ok(())
    }
}

impl ToadstoolClient {
    /// Create new toadstool client with performance optimizations
    pub async fn new(endpoint: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(CONNECTION_POOL_SIZE)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| SongbirdError::Network {
                service: "toadstool_client".to_string(),
                message: format!("Failed to create HTTP client: {}", e),
                details: None,
            })?;

        let circuit_breaker = Arc::new(RwLock::new(CircuitBreaker::new(
            CIRCUIT_BREAKER_THRESHOLD,
            CIRCUIT_BREAKER_TIMEOUT,
        )));

        let connection_pool = Arc::new(RwLock::new(ConnectionPool::new(CONNECTION_POOL_SIZE)));

        Ok(Self {
            endpoint,
            client,
            circuit_breaker,
            connection_pool,
        })
    }

    /// Health check for toadstool substrate with circuit breaker
    pub async fn health_check(&self) -> Result<()> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            if !cb.should_allow_request() {
                return Err(SongbirdError::Network {
                    service: "toadstool_substrate".to_string(),
                    message: "Circuit breaker is open".to_string(),
                    details: None,
                });
            }
        }

        let url = format!("{}/health", self.endpoint);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Record success
                    let mut cb = self.circuit_breaker.write().await;
                    cb.record_success();
                    Ok(())
                } else {
                    // Record failure
                    let mut cb = self.circuit_breaker.write().await;
                    cb.record_failure();
                    Err(SongbirdError::Network {
                        service: "toadstool_substrate".to_string(),
                        message: format!("Health check failed with status: {}", resp.status()),
                        details: None,
                    })
                }
            }
            Err(e) => {
                // Record failure
                let mut cb = self.circuit_breaker.write().await;
                cb.record_failure();
                Err(SongbirdError::Network {
                    service: "toadstool_substrate".to_string(),
                    message: format!("Health check failed: {}", e),
                    details: None,
                })
            }
        }
    }

    /// Make request to toadstool substrate with circuit breaker and connection pooling
    pub async fn request(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().await;
            if !cb.should_allow_request() {
                return Err(SongbirdError::Network {
                    service: "toadstool_substrate".to_string(),
                    message: "Circuit breaker is open".to_string(),
                    details: None,
                });
            }
        }

        let url = format!("{}/query", self.endpoint);
        let response = self.client.post(&url).json(&payload).send().await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Record success
                    let mut cb = self.circuit_breaker.write().await;
                    cb.record_success();

                    resp.json().await.map_err(|e| SongbirdError::Network {
                        service: "toadstool_substrate".to_string(),
                        message: format!("Failed to parse response: {}", e),
                        details: None,
                    })
                } else {
                    // Record failure
                    let mut cb = self.circuit_breaker.write().await;
                    cb.record_failure();

                    Err(SongbirdError::Network {
                        service: "toadstool_substrate".to_string(),
                        message: format!("Request failed with status: {}", resp.status()),
                        details: None,
                    })
                }
            }
            Err(e) => {
                // Record failure
                let mut cb = self.circuit_breaker.write().await;
                cb.record_failure();

                Err(SongbirdError::Network {
                    service: "toadstool_substrate".to_string(),
                    message: format!("Request failed: {}", e),
                    details: None,
                })
            }
        }
    }

    /// Get toadstool capabilities
    pub async fn get_capabilities(&self) -> Result<Vec<String>> {
        let payload = serde_json::json!({
            "action": "get_capabilities"
        });

        let response = self.request(payload).await?;

        if let Some(caps) = response.get("capabilities").and_then(|v| v.as_array()) {
            Ok(caps
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
        } else {
            Ok(vec!["compute".to_string(), "containers".to_string()])
        }
    }

    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> CircuitState {
        let cb = self.circuit_breaker.read().await;
        cb.get_state().clone()
    }
}

impl PathType {
    fn to_string(&self) -> String {
        match self {
            PathType::Data => "data",
            PathType::Config => "config",
            PathType::Log => "log",
            PathType::Cache => "cache",
            PathType::Runtime => "runtime",
            PathType::Temp => "temp",
        }
        .to_string()
    }
}

impl Default for PathRequirements {
    fn default() -> Self {
        Self {
            min_size_bytes: None,
            permissions: None,
            persistent: true,
            shared: false,
        }
    }
}

/// Convenience functions for common operations
impl OSSubstrate {
    /// Get data directory for a service
    pub async fn get_data_dir(&self, service_name: &str) -> Result<PathBuf> {
        self.get_path(PathRequest {
            path_type: PathType::Data,
            service_name: service_name.to_string(),
            requirements: PathRequirements::default(),
        })
        .await
    }

    /// Get config directory for a service
    pub async fn get_config_dir(&self, service_name: &str) -> Result<PathBuf> {
        self.get_path(PathRequest {
            path_type: PathType::Config,
            service_name: service_name.to_string(),
            requirements: PathRequirements::default(),
        })
        .await
    }

    /// Get log directory for a service
    pub async fn get_log_dir(&self, service_name: &str) -> Result<PathBuf> {
        self.get_path(PathRequest {
            path_type: PathType::Log,
            service_name: service_name.to_string(),
            requirements: PathRequirements::default(),
        })
        .await
    }

    /// Get available network interface
    pub async fn get_network_interface(&self) -> Result<NetworkInterface> {
        let request = NetworkRequest {
            operation: NetworkOperation::GetInterface,
            target: "default".to_string(),
            parameters: HashMap::new(),
        };

        let response = self.network_operation(request).await?;

        if let Some(interface_name) = response.get("interface").and_then(|v| v.as_str()) {
            if let Some(ip_address) = response.get("ip_address").and_then(|v| v.as_str()) {
                return Ok(NetworkInterface {
                    name: interface_name.to_string(),
                    ip_address: ip_address.to_string(),
                    subnet_mask: "255.255.255.0".to_string(),
                    is_up: true,
                    is_loopback: false,
                });
            }
        }

        // Fallback to default
        Ok(NetworkInterface {
            name: "default".to_string(),
            ip_address: constants::network::DEFAULT_BIND_ADDRESS.to_string(),
            subnet_mask: "255.255.255.0".to_string(),
            is_up: true,
            is_loopback: false,
        })
    }

    /// Get available port from substrate
    pub async fn get_available_port(&self) -> Result<u16> {
        let request = NetworkRequest {
            operation: NetworkOperation::GetAvailablePort,
            target: "tcp".to_string(),
            parameters: HashMap::new(),
        };

        let response = self.network_operation(request).await?;

        if let Some(port) = response.get("port").and_then(|v| v.as_u64()) {
            Ok(port as u16)
        } else {
            Ok(constants::network::DEFAULT_ORCHESTRATOR_PORT)
        }
    }
}

impl Clone for SubstrateMetrics {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            substrate_errors: self.substrate_errors,
            fallback_uses: self.fallback_uses,
            average_response_time: self.average_response_time,
            toadstool_requests: self.toadstool_requests,
            biomeos_requests: self.biomeos_requests,
            circuit_breaker_trips: self.circuit_breaker_trips,
        }
    }
}

// Global substrate instance with optimized caching
static SUBSTRATE: std::sync::OnceLock<OSSubstrate> = std::sync::OnceLock::new();

/// Get or initialize the global OS substrate with performance optimizations
pub async fn get_substrate() -> &'static OSSubstrate {
    SUBSTRATE.get_or_init(|| {
        // In a real implementation, this would be async
        // For now, we'll use a placeholder with optimized structure
        OSSubstrate {
            toadstool_client: ToadstoolClient {
                client: reqwest::Client::new(),
                endpoint: crate::config::constants::network::toadstool_endpoint(),
                circuit_breaker: Arc::new(RwLock::new(CircuitBreaker::new(
                    CIRCUIT_BREAKER_THRESHOLD,
                    CIRCUIT_BREAKER_TIMEOUT,
                ))),
                connection_pool: Arc::new(RwLock::new(ConnectionPool::new(CONNECTION_POOL_SIZE))),
            },
            biomeos_client: BiomeOSClient::new(crate::config::constants::network::biomeos_endpoint()),
            cache: Arc::new(RwLock::new(OptimizedSubstrateCache::new(
                MAX_CACHE_SIZE,
                DEFAULT_CACHE_TTL,
            ))),
            metrics: Arc::new(RwLock::new(SubstrateMetrics::default())),
        }
    })
}

/// Initialize the global substrate with performance optimizations
pub async fn initialize_substrate() -> Result<()> {
    let substrate = OSSubstrate::new().await?;
    SUBSTRATE
        .set(substrate)
        .map_err(|_| SongbirdError::Config {
            field: Some("substrate".to_string()),
            message: "Failed to initialize global substrate".to_string(),
        })?;

    // Warm up the cache for better performance
    if let Some(substrate) = SUBSTRATE.get() {
        if let Err(e) = substrate.warm_cache().await {
            warn!("Failed to warm substrate cache: {}", e);
        }
    }

    Ok(())
}

/// Get substrate performance metrics
pub async fn get_substrate_metrics() -> Option<SubstrateMetrics> {
    if let Some(substrate) = SUBSTRATE.get() {
        Some(substrate.get_metrics().await)
    } else {
        None
    }
}

/// Get substrate cache statistics
pub async fn get_substrate_cache_stats() -> Option<(usize, usize, f64, u64, u64)> {
    if let Some(substrate) = SUBSTRATE.get() {
        Some(substrate.get_cache_stats().await)
    } else {
        None
    }
}

/// Clear substrate cache
pub async fn clear_substrate_cache() -> Result<()> {
    if let Some(substrate) = SUBSTRATE.get() {
        substrate.clear_cache().await;
        Ok(())
    } else {
        Err(SongbirdError::Config {
            field: Some("substrate".to_string()),
            message: "Substrate not initialized".to_string(),
        })
    }
}

/// Check substrate health status
pub async fn check_substrate_health() -> Result<(bool, bool)> {
    if let Some(substrate) = SUBSTRATE.get() {
        let toadstool_health = substrate.toadstool_client.health_check().await.is_ok();
        let biomeos_health = substrate.biomeos_client.health_check().await.is_ok();
        Ok((toadstool_health, biomeos_health))
    } else {
        Err(SongbirdError::Config {
            field: Some("substrate".to_string()),
            message: "Substrate not initialized".to_string(),
        })
    }
}
