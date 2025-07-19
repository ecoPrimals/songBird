//! Main OS substrate implementation

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use songbird_config::constants;
use songbird_errors::{NetworkError, Result, SongbirdError};

use super::cache::OptimizedSubstrateCache;
use super::clients::ToadstoolClient;
use super::metrics::SubstrateMetrics;
use super::types::{NetworkRequest, NetworkResponse, PathRequest, SystemInfo};
use crate::biomeos::BiomeOSClient;

/// Maximum cache size for substrate entries
const MAX_CACHE_SIZE: usize = 1000;

/// Default cache TTL for substrate entries
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// OS substrate interface that abstracts platform operations with performance optimizations
#[derive(Debug, Clone)]
pub struct OSSubstrate {
    pub toadstool_client: ToadstoolClient,
    biomeos_client: BiomeOSClient,
    cache: Arc<RwLock<OptimizedSubstrateCache>>,
    metrics: Arc<RwLock<SubstrateMetrics>>,
}

impl OSSubstrate {
    /// Create new OS substrate with performance optimizations
    pub async fn new() -> Result<Self> {
        let toadstool_endpoint = std::env::var("TOADSTOOL_ENDPOINT")
            .unwrap_or_else(|_| constants::network::toadstool_endpoint());

        let biomeos_endpoint = std::env::var("BIOMEOS_ENDPOINT")
            .unwrap_or_else(|_| constants::network::biomeos_endpoint());

        let toadstool_client = ToadstoolClient::new(toadstool_endpoint).await?;
        let biomeos_client = BiomeOSClient::new(biomeos_endpoint);

        let substrate = Self {
            toadstool_client,
            biomeos_client,
            cache: Arc::new(RwLock::new(OptimizedSubstrateCache::new(
                MAX_CACHE_SIZE,
                DEFAULT_CACHE_TTL,
            ))),
            metrics: Arc::new(RwLock::new(SubstrateMetrics::new())),
        };

        // Initialize substrate connection with performance monitoring
        substrate.initialize_with_metrics().await?;

        Ok(substrate)
    }

    /// Initialize substrate connections with performance monitoring
    async fn initialize_with_metrics(&self) -> Result<()> {
        info!("🌍 Initializing optimized OS substrate through toadstool and biomeOS");

        let start_time = Instant::now();

        // Test toadstool connection
        match self.toadstool_client.health_check().await {
            Ok(true) => info!("✅ Toadstool client connected successfully"),
            Ok(false) => warn!("⚠️ Toadstool client connection unstable"),
            Err(e) => warn!("⚠️ Toadstool client connection failed: {}", e),
        }

        // Test biomeos connection (simplified)
        match self
            .biomeos_client
            .request("health", serde_json::json!({}))
            .await
        {
            Ok(_) => info!("✅ BiomeOS client connected successfully"),
            Err(e) => warn!("⚠️ BiomeOS client connection failed: {}", e),
        }

        let initialization_time = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_request(initialization_time);
        }

        info!("✅ OS substrate initialized in {:?}", initialization_time);
        Ok(())
    }

    /// Get system information with caching
    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        let start_time = Instant::now();

        // Try cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(cached_info) = cache.get_system_info() {
                let mut metrics = self.metrics.write().await;
                metrics.record_cache_hit();
                metrics.record_request(start_time.elapsed());
                debug!("📦 System info served from cache");
                return Ok(cached_info);
            }
        }

        // Cache miss - fetch from substrate
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_cache_miss();
        }

        let system_info = self
            .fetch_system_info_from_substrate()
            .await
            .unwrap_or_else(|_| self.get_fallback_system_info_sync());

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.cache_system_info(system_info.clone());
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.record_request(start_time.elapsed());
        }

        Ok(system_info)
    }

    /// Fetch system info from substrate services
    async fn fetch_system_info_from_substrate(&self) -> Result<SystemInfo> {
        // Try toadstool first
        match self
            .toadstool_client
            .request(serde_json::json!({"action": "system_info"}))
            .await
        {
            Ok(response) => {
                debug!("📊 System info from toadstool");
                return self.parse_system_info(response);
            }
            Err(e) => debug!("Failed to get system info from toadstool: {}", e),
        }

        // Fallback to biomeos
        match self
            .biomeos_client
            .request("system_info", serde_json::json!({}))
            .await
        {
            Ok(response) => {
                debug!("📊 System info from biomeOS");
                return self.parse_system_info(response);
            }
            Err(e) => debug!("Failed to get system info from biomeOS: {}", e),
        }

        Err(SongbirdError::service_error(
            "substrate".to_string(),
            "Failed to retrieve system info from any substrate".to_string(),
        ))
    }

    /// Parse system info from JSON response
    fn parse_system_info(&self, response: serde_json::Value) -> Result<SystemInfo> {
        serde_json::from_value(response)
            .map_err(|e| SongbirdError::io_error(format!("Failed to parse system info: {}", e)))
    }

    /// Get path with caching
    pub async fn get_path(&self, request: PathRequest) -> Result<PathBuf> {
        let start_time = Instant::now();
        let cache_key = format!("{}_{}", request.service_name, request.path_type);

        // Try cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(cached_path) = cache.get_path(&cache_key) {
                let mut metrics = self.metrics.write().await;
                metrics.record_cache_hit();
                metrics.record_request(start_time.elapsed());
                debug!("📦 Path served from cache: {:?}", cached_path);
                return Ok(cached_path);
            }
        }

        // Cache miss - fetch from substrate
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_cache_miss();
        }

        let path = self.fetch_path_from_substrate(&request).await?;

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.cache_path(cache_key, path.clone());
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.record_request(start_time.elapsed());
        }

        Ok(path)
    }

    /// Fetch path from substrate services
    async fn fetch_path_from_substrate(&self, request: &PathRequest) -> Result<PathBuf> {
        let request_payload = serde_json::json!({
            "action": "get_path",
            "path_type": request.path_type.to_string(),
            "service_name": request.service_name,
            "requirements": request.requirements
        });

        // Try toadstool first
        match self.toadstool_client.request(request_payload.clone()).await {
            Ok(response) => {
                if let Some(path_str) = response.get("path").and_then(|p| p.as_str()) {
                    return Ok(PathBuf::from(path_str));
                }
            }
            Err(e) => debug!("Failed to get path from toadstool: {}", e),
        }

        // Fallback to biomeos
        match self
            .biomeos_client
            .request("get_path", request_payload)
            .await
        {
            Ok(response) => {
                if let Some(path_str) = response.get("path").and_then(|p| p.as_str()) {
                    return Ok(PathBuf::from(path_str));
                }
            }
            Err(e) => debug!("Failed to get path from biomeOS: {}", e),
        }

        // Final fallback - generate default path
        let mut metrics = self.metrics.write().await;
        metrics.record_fallback_use();

        Ok(self.generate_fallback_path(&request.path_type.to_string(), &request.service_name))
    }

    /// Get capabilities with caching
    pub async fn get_capabilities(&self) -> Result<Vec<String>> {
        let start_time = Instant::now();
        let cache_key = "system_capabilities";

        // Try cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(cached_capabilities) = cache.get_capabilities(cache_key) {
                let mut metrics = self.metrics.write().await;
                metrics.record_cache_hit();
                metrics.record_request(start_time.elapsed());
                debug!("📦 Capabilities served from cache");
                return Ok(cached_capabilities);
            }
        }

        // Cache miss - fetch from substrate
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_cache_miss();
        }

        let capabilities = self
            .fetch_capabilities_from_substrate()
            .await
            .unwrap_or_else(|_| self.get_fallback_capabilities());

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.cache_capabilities(cache_key.to_string(), capabilities.clone());
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.record_request(start_time.elapsed());
        }

        Ok(capabilities)
    }

    /// Fetch capabilities from substrate services
    async fn fetch_capabilities_from_substrate(&self) -> Result<Vec<String>> {
        // Try toadstool first
        match self
            .toadstool_client
            .request(serde_json::json!({"action": "get_capabilities"}))
            .await
        {
            Ok(response) => {
                if let Some(caps) = response.get("capabilities").and_then(|c| c.as_array()) {
                    let capabilities: Vec<String> = caps
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    return Ok(capabilities);
                }
            }
            Err(e) => debug!("Failed to get capabilities from toadstool: {}", e),
        }

        // Fallback to biomeos
        match self
            .biomeos_client
            .request("get_capabilities", serde_json::json!({}))
            .await
        {
            Ok(response) => {
                if let Some(caps) = response.get("capabilities").and_then(|c| c.as_array()) {
                    let capabilities: Vec<String> = caps
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    return Ok(capabilities);
                }
            }
            Err(e) => debug!("Failed to get capabilities from biomeOS: {}", e),
        }

        Err(SongbirdError::service_error(
            "substrate".to_string(),
            "Failed to retrieve capabilities from any substrate".to_string(),
        ))
    }

    /// Perform network operation
    pub async fn network_operation(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        let start_time = Instant::now();

        let result = match request.operation_type.as_str() {
            "toadstool" => {
                let response = self.toadstool_client.request(request.payload).await?;
                Ok(NetworkResponse {
                    success: true,
                    data: response,
                    message: "Operation completed".to_string(),
                })
            }
            "biomeos" => {
                let response = self
                    .biomeos_client
                    .request("network", request.payload)
                    .await?;
                Ok(NetworkResponse {
                    success: true,
                    data: response,
                    message: "Operation completed".to_string(),
                })
            }
            _ => Err(SongbirdError::Network(Box::new(NetworkError {
                message: "Unknown operation type".to_string(),
                endpoint: None,
                port: None,
                protocol: None,
            }))),
        };

        let mut metrics = self.metrics.write().await;
        match &result {
            Ok(_) => metrics.record_request(start_time.elapsed()),
            Err(_) => {
                metrics.record_error();
                metrics.record_request(start_time.elapsed());
            }
        }

        result
    }

    /// Clear substrate cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("🧹 Substrate cache cleared");
    }

    /// Warm up substrate cache with common operations
    pub async fn warm_cache(&self) -> Result<()> {
        info!("🔥 Warming up substrate cache...");

        // Warm up system info
        let _ = self.get_system_info().await;

        // Warm up capabilities
        let _ = self.get_capabilities().await;

        info!("✅ Substrate cache warmed up");
        Ok(())
    }

    /// Get substrate metrics
    pub async fn get_metrics(&self) -> super::metrics::MetricsSummary {
        let metrics = self.metrics.read().await;
        metrics.summary()
    }

    /// Generate fallback path when substrate services are unavailable
    fn generate_fallback_path(&self, path_type: &str, service_name: &str) -> PathBuf {
        let base_path =
            std::env::var("SONGBIRD_DATA_DIR").unwrap_or_else(|_| "/tmp/songbird".to_string());

        PathBuf::from(base_path).join(service_name).join(path_type)
    }

    /// Get fallback system info when primary method fails
    fn get_fallback_system_info_sync(&self) -> SystemInfo {
        SystemInfo {
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            available_storage: 1_000_000_000, // 1GB fallback
            available_memory: 1_000_000_000,  // 1GB fallback
            cpu_cores: num_cpus::get() as u32,
            network_interfaces: vec![],
        }
    }

    /// Get fallback capabilities
    fn get_fallback_capabilities(&self) -> Vec<String> {
        vec![
            "basic_operations".to_string(),
            "file_system".to_string(),
            "network_access".to_string(),
        ]
    }
}
