//! Universal Service Discovery Adapter
//!
//! Provides vendor-agnostic service discovery that can work with:
//! - Any HTTP-based service discovery (Consul, Eureka, etc.)
//! - Any file-based service registry
//! - Any environment-based service configuration
//! - Any capability-based service detection

use async_trait::async_trait;
use futures_util::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{debug, info};

use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::service::{ServiceInfo, ServiceStatus};
use crate::traits::{ServiceDiscovery, ServiceEvent, ServiceQuery};
use songbird_types::errors::SongbirdResult;

/// Universal service discovery adapter that works with any service discovery system
#[derive(Debug)]
pub struct UniversalServiceDiscovery {
    /// Discovered service registry endpoints
    registry_endpoints: Vec<String>,
    /// Capability-based service cache with TTL
    service_cache: HashMap<String, CachedServiceInfo>,
    /// Auto-detected discovery methods
    discovery_methods: Vec<DiscoveryMethod>,
    /// Cache configuration
    cache_config: CacheConfig,
}

/// Discovery methods that can be auto-detected
#[derive(Debug, Clone)]
enum DiscoveryMethod {
    /// HTTP-based discovery (works with Consul, Eureka, etc.)
    HttpRegistry {
        endpoint: String,
    },
    /// Environment-based discovery
    Environment,
    /// File-based discovery
    FileBased {
        path: String,
    },
    /// Network scanning discovery
    NetworkScan {
        subnet: String,
    },
}

/// Cached service information with TTL
#[derive(Debug, Clone)]
pub struct CachedServiceInfo {
    /// The actual service information
    pub service_info: ServiceInfo,
    /// When this cache entry was created
    pub cached_at: std::time::Instant,
    /// Time-to-live for this cache entry
    pub ttl: std::time::Duration,
}

/// Cache configuration for service discovery
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for cached services
    pub default_ttl: std::time::Duration,
    /// Maximum number of cached services
    pub max_cache_size: usize,
    /// Whether to enable cache
    pub enabled: bool,
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Total number of entries in the cache
    pub total_entries: usize,
    /// Number of valid (non-expired) entries
    pub valid_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Maximum capacity of the cache
    pub max_capacity: usize,
    /// Hit ratio (currently 0.0)
    pub hit_ratio: f64,
}

impl UniversalServiceDiscovery {
    /// Create a new universal service discovery adapter with auto-detection
    pub async fn new() -> SongbirdResult<Self> {
        let mut adapter = Self {
            registry_endpoints: Vec::new(),
            service_cache: HashMap::new(),
            discovery_methods: Vec::new(),
            cache_config: CacheConfig {
                default_ttl: std::time::Duration::from_secs(30),
                max_cache_size: 1000,
                enabled: true,
            },
        };

        // Auto-detect available discovery methods
        adapter.auto_detect_discovery_methods().await?;

        info!(
            "Universal service discovery initialized with {} methods",
            adapter.discovery_methods.len()
        );

        Ok(adapter)
    }

    /// Add service to cache
    pub fn cache_service(&mut self, service_id: &str, service_info: ServiceInfo) {
        if !self.cache_config.enabled {
            return;
        }

        // Check cache size limit
        if self.service_cache.len() >= self.cache_config.max_cache_size {
            // Remove oldest entry
            if let Some((oldest_key, _)) =
                self.service_cache.iter().min_by_key(|(_, cached)| cached.cached_at)
            {
                let oldest_key = oldest_key.clone();
                self.service_cache.remove(&oldest_key);
                debug!("Removed oldest cache entry: {}", oldest_key);
            }
        }

        let cached_info = CachedServiceInfo {
            service_info,
            cached_at: std::time::Instant::now(),
            ttl: self.cache_config.default_ttl,
        };

        self.service_cache.insert(service_id.to_string(), cached_info);
        debug!("Cached service: {} (TTL: {:?},", service_id, self.cache_config.default_ttl);
    }

    /// Get service from cache if valid
    pub fn get_cached_service(&self, service_id: &str) -> Option<ServiceInfo> {
        if !self.cache_config.enabled {
            return None;
        }

        if let Some(cached) = self.service_cache.get(service_id) {
            // Check if cache entry is still valid
            if cached.cached_at.elapsed() < cached.ttl {
                debug!("Cache hit for service: {}", service_id);
                return Some(cached.service_info.clone());
            }
            debug!("Cache expired for service: {}", service_id);
        }

        None
    }

    /// Remove expired entries from cache
    pub fn cleanup_cache(&mut self) {
        if !self.cache_config.enabled {
            return;
        }

        let now = std::time::Instant::now();
        let initial_size = self.service_cache.len();

        self.service_cache.retain(|service_id, cached| {
            let is_valid = now.duration_since(cached.cached_at) < cached.ttl;
            if !is_valid {
                debug!("Removing expired cache entry: {}", service_id);
            }
            is_valid
        });

        let removed_count = initial_size - self.service_cache.len();
        if removed_count > 0 {
            debug!("Cache cleanup completed: removed {} expired entries", removed_count);
        }
    }

    /// Get cache statistics
    #[must_use]
    pub fn get_cache_stats(&self) -> CacheStats {
        let now = std::time::Instant::now();
        let valid_entries = self
            .service_cache
            .values()
            .filter(|cached| now.duration_since(cached.cached_at) < cached.ttl)
            .count();

        CacheStats {
            total_entries: self.service_cache.len(),
            valid_entries,
            expired_entries: self.service_cache.len() - valid_entries,
            max_capacity: self.cache_config.max_cache_size,
            hit_ratio: 0.0, // Would need to track hits/misses for accurate ratio
        }
    }

    /// Auto-detect available service discovery methods
    async fn auto_detect_discovery_methods(&mut self) -> SongbirdResult<()> {
        // Check for HTTP-based service registries
        self.detect_http_registries().await;

        // Check for environment-based configuration
        self.detect_environment_services();

        // Check for file-based service definitions
        self.detect_file_based_services();

        // Check for network-based service discovery
        self.detect_network_services();

        Ok(())
    }

    /// Detect HTTP-based service registries (Consul, Eureka, etc.)
    async fn detect_http_registries(&mut self) {
        use songbird_config::config::constants;

        // Get configurable protocol
        let registry_protocol =
            std::env::var("SERVICE_REGISTRY_PROTOCOL").unwrap_or_else(|_| "http".to_string());

        // Build default endpoints using constants
        let consul_default = format!(
            "{}://{}:{}",
            registry_protocol,
            constants::network::DEFAULT_HOST,
            std::env::var("CONSUL_PORT").unwrap_or_else(|_| "8500".to_string())
        );
        let eureka_default = format!(
            "{}://{}:{}",
            registry_protocol,
            constants::network::DEFAULT_HOST,
            std::env::var("EUREKA_PORT").unwrap_or_else(|_| "8761".to_string())
        );

        let potential_endpoints = vec![
            std::env::var("SERVICE_REGISTRY_URL").unwrap_or_default(),
            std::env::var("CONSUL_HTTP_ADDR").unwrap_or_default(),
            std::env::var("EUREKA_SERVER_URL").unwrap_or_default(),
            consul_default, // Configurable Consul default
            eureka_default, // Configurable Eureka default
        ];

        for endpoint in potential_endpoints {
            if !endpoint.is_empty() && self.test_http_endpoint(&endpoint).await {
                debug!("Detected HTTP service registry: {}", endpoint);
                self.discovery_methods.push(DiscoveryMethod::HttpRegistry {
                    endpoint: endpoint.clone(),
                });
                self.registry_endpoints.push(endpoint);
            }
        }
    }

    /// Test if an HTTP endpoint is a valid service registry
    async fn test_http_endpoint(&self, endpoint: &str) -> bool {
        // Try common service registry endpoints
        let test_paths = vec!["/v1/catalog/services", "/eureka/apps", "/api/v1/services"];

        for path in test_paths {
            let url = format!("{}{}", endpoint.trim_end_matches('/'), path);
            if (reqwest::get(&url).await).is_ok() {
                return true;
            }
        }
        false
    }

    /// Detect environment-based service configuration
    fn detect_environment_services(&mut self) {
        let env_vars = std::env::vars().collect::<HashMap<_, _>>();

        // Look for service-related environment variables
        let service_patterns = ["_SERVICE_URL", "_ENDPOINT", "_HOST"];

        for (key, value) in &env_vars {
            if service_patterns.iter().any(|pattern| key.contains(pattern) && !value.is_empty()) {
                debug!("Detected environment service: {} = {}", key, value);
                if self.discovery_methods.iter().all(|m| !matches!(m, DiscoveryMethod::Environment))
                {
                    self.discovery_methods.push(DiscoveryMethod::Environment);
                }
            }
        }
    }

    /// Detect file-based service definitions
    fn detect_file_based_services(&mut self) {
        let config_paths =
            vec!["/etc/services.yaml", "/etc/services.json", "./services.yaml", "./services.json"];

        for path in config_paths {
            if std::path::Path::new(path).exists() {
                debug!("Detected file-based services: {}", path);
                self.discovery_methods.push(DiscoveryMethod::FileBased {
                    path: path.to_string(),
                });
            }
        }
    }

    /// Detect network-based service discovery
    fn detect_network_services(&mut self) {
        // Check if we're in a container environment (Kubernetes, Docker, etc.)
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::env::var("CONTAINER").is_ok()
            || std::path::Path::new("/.dockerenv").exists()
        {
            debug!("Detected container environment - enabling network scanning");
            self.discovery_methods.push(DiscoveryMethod::NetworkScan {
                subnet: "10.0.0.0/8".to_string(),
            });
        }
    }

    /// Discover services using all available methods
    async fn discover_services_universal(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut all_services = Vec::new();

        for method in &self.discovery_methods {
            match method {
                DiscoveryMethod::HttpRegistry {
                    endpoint,
                } => {
                    if let Ok(services) = self.discover_from_http_registry(endpoint, &query).await {
                        all_services.extend(services);
                    }
                }
                DiscoveryMethod::Environment => {
                    if let Ok(services) = self.discover_from_environment(&query).await {
                        all_services.extend(services);
                    }
                }
                DiscoveryMethod::FileBased {
                    path,
                } => {
                    if let Ok(services) = self.discover_from_file(path, &query).await {
                        all_services.extend(services);
                    }
                }
                DiscoveryMethod::NetworkScan {
                    subnet,
                } => {
                    if let Ok(services) = self.discover_from_network_scan(subnet, &query).await {
                        all_services.extend(services);
                    }
                }
            }
        }

        // Deduplicate services by service_id
        let mut unique_services = HashMap::new();
        for service in all_services {
            unique_services.insert(service.service_id.clone(), service);
        }

        Ok(unique_services.into_values().collect())
    }

    /// Discover services from HTTP registry (works with Consul, Eureka, etc.)
    async fn discover_from_http_registry(
        &self,
        endpoint: &str,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // This is a universal HTTP discovery that adapts to different registry APIs
        debug!("Discovering services from HTTP registry: {}", endpoint);

        // Try different API endpoints
        let api_paths = vec![
            "/v1/catalog/services", // Consul
            "/eureka/apps",         // Eureka
            "/api/v1/services",     // Generic
        ];

        for path in api_paths {
            let url = format!("{}{}", endpoint.trim_end_matches('/'), path);
            if let Ok(response) = reqwest::get(&url).await {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    return self.parse_universal_service_response(&data);
                }
            }
        }

        Ok(Vec::new())
    }

    /// Parse service response from any HTTP registry
    fn parse_universal_service_response(
        &self,
        data: &serde_json::Value,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();

        // Handle different response formats
        if let Some(consul_services) = data.as_object() {
            // Consul format
            for (name, _) in consul_services {
                services.push(self.create_service_info(name, "http-registry"));
            }
        } else if let Some(eureka_apps) =
            data.get("applications").and_then(|a| a.get("application").and_then(|a| a.as_array()))
        {
            // Eureka format
            for app in eureka_apps {
                if let Some(name) = app.get("name").and_then(|n| n.as_str()) {
                    services.push(self.create_service_info(name, "eureka"));
                }
            }
        } else if let Some(generic_services) = data.as_array() {
            // Generic array format
            for service in generic_services {
                if let Some(name) = service.get("name").and_then(|n| n.as_str()) {
                    services.push(self.create_service_info(name, "generic"));
                }
            }
        }

        Ok(services)
    }

    /// Discover services from environment variables
    async fn discover_from_environment(
        &self,
        query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        let env_vars = std::env::vars().collect::<HashMap<_, _>>();

        for (key, _value) in env_vars {
            if key.ends_with("_SERVICE_URL") || key.ends_with("_ENDPOINT") {
                let service_name =
                    key.replace("_SERVICE_URL", "").replace("_ENDPOINT", "").to_lowercase();

                // Apply query filters
                if let Some(name_filter) = &query.name {
                    if !service_name.contains(name_filter) {
                        continue;
                    }
                }

                services.push(self.create_service_info(&service_name, "environment"));
            }
        }

        Ok(services)
    }

    /// Discover services from file-based configuration
    async fn discover_from_file(
        &self,
        _path: &str,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // Implementation would read YAML/JSON service definitions
        debug!("File-based discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Discover services from network scanning
    async fn discover_from_network_scan(
        &self,
        _subnet: &str,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // Implementation would scan network for services
        debug!("Network scanning discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Create a `ServiceInfo` from discovered service data
    fn create_service_info(&self, name: &str, discovery_type: &str) -> ServiceInfo {
        use chrono::Utc;

        ServiceInfo {
            service_id: format!("{}-{}", name, uuid::Uuid::new_v4()),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: discovery_type.to_string(),
            description: Some(format!("Service discovered via {}", discovery_type)),
            endpoints: Vec::new(), // Would be populated from actual discovery
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![discovery_type.to_string()],
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("{}-instance", name),
            host: songbird_config::constants::network::DEFAULT_HOST.to_string(),
            port: 8080,
        }
    }
}

#[async_trait]
impl ServiceDiscovery for UniversalServiceDiscovery {
    async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover_services_universal(query).await
    }

    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        info!("Universal service registration: {}", service.name);
        // In a real implementation, this would register with detected service registries
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Universal service unregistration: {}", service_id);
        // In a real implementation, this would unregister from detected service registries
        Ok(())
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        use futures_util::stream;

        // Return an empty stream for now - would implement real watching
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        info!("Universal health update for service {}: {:?}", service_id, health);
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover(ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let services = self.list_all().await?;
        Ok(services.iter().any(|s| s.service_id == service_id))
    }

    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        info!("Universal metadata update for service {}: {:?}", service_id, metadata);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
