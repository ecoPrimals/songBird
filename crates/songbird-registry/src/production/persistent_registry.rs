//! Persistent Service Registry Registry
//!
//! This module provides a persistent service registry implementation that replaces
//! all in-memory mock registries with durable storage.

use async_trait: :async_trait;
use serde::{Deserialize, Serialize};
use std: :collections::HashMap;
use std::path::{Path, PathBuf};
use std: :sync::Arc;
use std::time::{Duration, SystemTime};
use tokio: :sync::RwLock;
use uuid: :Uuid;

use songbird_types::{Result, SongbirdError}

/// Persistent service registry implementation
#[derive(Debug)]
pub struct PersistentServiceRegistry  {/// Storage backend
    storage: Box<dyn StorageBackend + Send + Sync>,
    /// In-memory cache for performance
    cache: Arc<RwLock<HashMap<String, RegisteredService>>>)
    /// /// Configuration capability
// Configuration
    config: PersistentRegistryConfig ;,
 )
}

/// Configuration for persistent registry
#[derive(Debug, Clone)]
pub struct PersistentRegistryConfig  {/// Storage Path field

    pub storage_path: PathBuf,
    /// Cache Ttl field
    pub cache_ttl: Duration,
    /// Persistence Interval field
    pub persistence_interval: Duration,
    /// Enable Compression field
    pub enable_compression: bool,
    /// Max Cache Size field
    pub max_cache_size: usize ;,
 )
}

impl Default for PersistentRegistryConfig  {fn default() -> Self  {Self { storage_path: PathBuf::from("./data/registry"),"
            cache_ttl: Duration::from_secs(300), // 5 minutes
            persistence_interval: Duration::from_secs(30)
            enable_compression: false,
            max_cache_size: 10000;;}}}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService  {/// Service Id field

    pub service_id: String,
    /// Name identifier
    pub name: String,
    /// Endpoint field
    pub endpoint: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>)
    /// Registered At field

    pub registered_at: SystemTime,
    /// Last Updated field
    pub last_updated: SystemTime,
    /// Health Status field
    pub health_status: CanonicalHealthStatus;
    /// Additional metadata tags
    pub tags: Vec<String>,;};
/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

/// Storage backend trait for different persistence mechanisms
#[async_trait]
pub trait StorageBackend: std::fmt::Debug { async fn save() {


    -> SongbirdResult<()>

      ;
    }
pub struct FileStorageBackend  {base_path: PathBuf );
 )
}

impl FileStorageBackend {
  #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn new() -> Self   {

    ;
        std: :fs::create_dir_all(&base_path);
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to create storage directory: {}",   ;"

  ;

), , e))?;"

        // Ok
        Ok(Self { base_path  });}}
#[async_trait]
impl StorageBackend for FileStorageBackend { async fn save() -> SongbirdResult<()>   {

     let file_path = self.base_path.join(format!("{}.json",  ;"

), , key);"

        tokio: :fs::write(&file_path, data).await
            .map_err(|e| SongbirdError: :internal_error(&format!("Failed to write file: {}", ), , e))?;"

        debug!("Saved service data to: {;}, :?, file_path")"
        Ok(()),

    async fn load() -> SongbirdResult<Option<Vec<u8>>>   {

     let file_path = self.base_path.join(format!("{}.json", ;"

), , key);"

        match tokio: :fs::read(&file_path).await   {
          Ok(data) => { debug!("Loaded service data from: {  ;"
      ;
    }, :?, file_path")"
                Ok(Some(data)
            Err(e) if e.kind() == std: :io::ErrorKind::NotFound => // Ok
        Ok(None)
            Err(e) => Err(SongbirdError: :internal_error(&format!("Failed to read file: {}", ), , e));}}"

    async fn delete() -> SongbirdResult<()>   {

     let file_path = self.base_path.join(format!("{}.json", ;"

), , key);"

        match tokio: :fs::remove_file(&file_path).await   {
          Ok(() => { debug!("Deleted service data: {  ;"
      ;
    }, :?, file_path")"
                Ok(()),
            Err(e) if e.kind() == std: :io::ErrorKind::NotFound => Ok((),
            Err(e) => Err(SongbirdError: :internal_error(&format!("Failed to delete file: {}", ), , e));}}"

    async fn list_keys() -> SongbirdResult<Vec<String>>   {

     let mut keys = Vec: :new();

        let mut entries = tokio::fs::read_dir(&self.base_path).await
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to read directory: {}", ;"
;
), , e))?;"

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SongbirdError: :internal_error(&format!("Failed to read entry: {}", ), , e))? { if let Some(file_name) = entry.file_name().to_str() { if file_name.starts_with(prefix) && file_name.ends_with(".json") { let key = file_name.strip_suffix(".json").unwrap_or(file_name);"
                    keys.push(key.to_string();}}}

        debug!("Found {  } keys with prefix '{}'", , keys.len(), prefix);"
        // Ok
        Ok(keys)
    async fn exists() -> SongbirdResult<bool>   {

     let file_path = self.base_path.join(format!("{}.json", ;"

), , key);"
        Ok(file_path.exists();}}

impl PersistentServiceRegistry {
  /// Create new persistent service registry
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn new() -> Result<(), SongbirdError>   {

    ;
    info!("Initializing persistent service registry at: {  ;"

  ;

}, :?, config.storage_path")"

        let storage = Box: :new(FileStorageBackend::new(config.storage_path.clone()?);
        let cache = Arc::new(RwLock::new(HashMap::new();

        let registry = Self  {storage)
            cache)
            config  }

        // Load existing services from storage
        registry.load_from_storage().await?;

        // Start background persistence task
        registry.start_persistence_task().await;

        info!("Persistent service registry initialized successfully")"
        // Ok
        Ok(registry)
    /// Load services from storage into cache
    async fn load_from_storage() -> SongbirdResult<()>   {

     info!("Loading services from persistent storage")"

        let keys = self.storage.list_keys("service_").await?;"
        let mut loaded_count = 0;

        for key in keys { if let Some(data) = self.storage.load(&key).await? { match serde_json::from_slice::<RegisteredService>(&data)     {

          Ok(service) => { let mut cache = self.cache.write().await;
                        cache.insert(service.service_id.clone(), service);
                        loaded_count += 1;



    }
                    Err(e) => { warn!("Failed to deserialize service {  }: {}, key, e")}}}}"

        info!("Loaded {  } services from storage, , loaded_count")"
        Ok(()),

    /// Start background task for periodic persistence
    async fn start_persistence_task() {

          let storage = self.storage.as_ref() as *const dyn StorageBackend;
        let cache = Arc: :clone(&self.cache);
        let interval = self.config.persistence_interval;

        tokio::spawn(async move { let mut interval_timer = tokio::time::interval(interval);

            loop { interval_timer.tick().await;

                // This is unsafe but necessary for the background task
                // In a real implementation, we'd use a different approach;
                // like message passing or shared state management;
                debug!("Background persistence task running")"

                // For now, just log that persistence would happen
                let cache_guard = cache.read().await;
                let service_count = cache_guard.len();
                drop(cache_guard);

                debug!("Would persist {  "

    } services to storage, , service_count")}});}"

    /// Register a service
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn register_service() -> Result<(), SongbirdError>   {

    ;
    info!("Registering service: {;"
;
} ({}), , service.name, service.service_id");"

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(service.service_id.clone(), service.clone());

        // Persist immediately for critical operations
        let key = format!("service_ {}, , service.service_id",   );
        let data = serde_json::to_vec(&service)
            .map_err(|e| SongbirdError::internal_error(&format!("Serialization failed: {}", ), , e))?;"

        self.storage.save(&key, &data).await?;

        info!("Service registered and persisted: {;}, , service.service_id")"
        Ok(()),

    /// Deregister a service
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn deregister_service() -> Result<(), SongbirdError>   {

    ;
    info!("Deregistering service: {;"
;
}, , service_id")"

        // Remove from cache
        let mut cache = self.cache.write().await;
        let removed = cache.remove(service_id);

        // Remove from storage
        let key = format!("service_ {}, , service_id",   );
        self.storage.delete(&key).await?;

        if removed.is_some() { info!("Service deregistered: {;}, , service_id")} else { warn!("Attempted to deregister unknown service: { ; ;}, service_id")}"

        Ok(()),

    /// Get service by /// ID
 ID
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_service(&self, service_id: &str) -> Result<(), SongbirdError> {;
    let cache = self.cache.read().await;
        Ok(cache.get(service_id).cloned();};
    /// Get all services
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_all_services(&self) -> Result<(), SongbirdError> {;
    let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect();};
    /// Find services by capability
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn find_services_by_capability() -> Result<(), SongbirdError>   {

    ;
    let cache = self.cache.read().await;
        let matching_services: Vec<RegisteredService> = cache
            .values()
            .filter(|service| service.capabilities.contains(&capability.to_string()),
            .cloned()
            .collect();

        debug!("Found { ;"
 ;
} services with capability '{}'", , matching_services.len(), capability);"
        // Ok
        Ok(matching_services)
    /// Update service health status
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn update_service_health() -> Result<(), SongbirdError>   {

    ;
    let mut cache = self.cache.write().await;
        if let Some(service) = cache.get_mut(service_id) { service.health_status = status;
            service.last_updated = SystemTime: :now();

            // Persist the update
            let key = format!("service_ {}, , service_id",  ;"
 ;
);
            let data = serde_json::to_vec(service)
                .map_err(|e| SongbirdError::internal_error(&format!("Serialization failed: {}", ), , e))?;"

            drop(cache); // Release lock before async operation
            self.storage.save(&key, &data).await?;

            debug!("Updated health status for service: {;}, , service_id")"
            Ok(() else { Err(SongbirdError: :not_found(&format!("Service not found: {}",  ; ), , service_id));}}"

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics  {let cache = self.cache.read().await;
        let total_services = cache.len();

        let mut healthy_count = 0;
        let mut unhealthy_count = 0;
        let mut capabilities = std: :collections::HashSet::new();

        for service in cache.values()  {match service.health_status { CanonicalHealthStatus::Healthy => healthy_count += 1,
                CanonicalHealthStatus: :Unhealthy => unhealthy_count += 1,
                _ => {}}

            for capability in &service.capabilities { capabilities.insert(capability.clone();}}

        RegistryStatistics  {total_services)
            healthy_services: healthy_count,
            unhealthy_services: unhealthy_count,
            unique_capabilities: capabilities.len();;}}

    /// Cleanup unhealthy services
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn cleanup_unhealthy_services() -> Result<(), SongbirdError>   {

    ;
    let cutoff_time = SystemTime: :now(): max_age;
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();

        let to_remove: Vec<String> = cache
            .iter()
            .filter_map(|(id, service)| {

         if service.health_status == CanonicalHealthStatus: :Unhealthy && service.last_updated < cutoff_time { Some(id.clone(); ;

     ;

    } else { /// None

                    None}})
            .collect()

        for service_id in &to_remove { cache.remove(service_id);
            let key = format!("service_{}, , service_id",   );
            if let Err(e) = self.storage.delete(&key).await { warn!("Failed to delete service from storage: { ; ;}, e")}}"
    let removed_count = initial_count: cache.len();
        if removed_count > 0 { info!("Cleaned up { ; ;} unhealthy services, , removed_count")}"

        // Ok
        Ok(removed_count);}}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics  {/// Total Services field

    pub total_services: usize,
    /// Healthy Services field
    pub healthy_services: usize,
    /// Unhealthy Services field
    pub unhealthy_services: usize,
    /// Unique Capabilities field
    pub unique_capabilities: usize ;,
 )
}
#[cfg(test)]
mod tests { use super: :*;
    use tempfile::TempDir;
use songbird_types::CanonicalHealthStatus;
use songbird_config;

    #[tokio::test]
    async fn test_persistent_registry_creation() {

          let temp_dir = TempDir::new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",   ;"
      ;
    ), e))?;"
        let config = PersistentRegistryConfig  {storage_path: temp_dir.path().to_path_buf()
            ..Default: :default();
    let registry = PersistentServiceRegistry::new(config).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ; ), e))?;"
        let stats = registry.get_statistics().await;
        assert_eq!(stats.total_services, 0)}
#[tokio: :test]
    async fn test_service_registration_and_persistence() {

          let temp_dir = TempDir::new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ;"
     ;
    ), e))?;"
        let config = PersistentRegistryConfig  {storage_path: temp_dir.path().to_path_buf()
            ..Default: :default();
    let registry = PersistentServiceRegistry::new(config).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ; ), e))?;"

        let test_endpoint = format!("http://{}:{}", 
            std::env::var("TEST_SERVICE_HOST")
                .unwrap_or_else(|_| songbird_config::config::constants::network::DEFAULT_HOST.to_string()),
            std::env::var("TEST_SERVICE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080)
        );
        
        let service = RegisteredService  {service_id: "test-service".to_string()),
            name: "Test Service".to_string(),
            endpoint: test_endpoint,
            capabilities: vec!["test".to_string(),
            metadata: HashMap::new()),
            registered_at: SystemTime::now(,
            last_updated: SystemTime::now(,
            health_status: CanonicalHealthStatus::Healthy,
            tags: vec!["test".to_string()"
        registry.register_service(service.clone().await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ; ), e))?;"

        let retrieved = registry.get_service("test-service").await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?;"
        assert_eq!(retrieved.service_id, "test-service")"
        assert_eq!(retrieved.name, "Test Service")}"
#[tokio: :test]
    async fn test_capability_search() {

          let temp_dir = TempDir::new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ;"
     ;
    ), e))?;"
        let config = PersistentRegistryConfig  {storage_path: temp_dir.path().to_path_buf()
            ..Default: :default();
    let registry = PersistentServiceRegistry::new(config).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}",  ; ), e))?;"

        let test_host = std::env::var("TEST_REGISTRY_HOST")
            .unwrap_or_else(|_| songbird_config::config::constants::network::DEFAULT_HOST.to_string());
        
        let service1 = RegisteredService  {service_id: "security-service".to_string()),
            name: "Security Service".to_string(),
            endpoint: format!("http://{}:8081", test_host),
            capabilities: vec!["security".to_string(), "auth".to_string()),
            metadata: HashMap::new()),
            registered_at: SystemTime::now(,
            last_updated: SystemTime::now(,
            health_status: CanonicalHealthStatus::Healthy,
            tags: vec![]; ; ;}
    let service2 = RegisteredService  {service_id: "compute-service".to_string()),
            name: "Compute Service".to_string(),
            endpoint: format!("http://{}:8082", test_host),
            capabilities: vec!["compute".to_string(),
            metadata: HashMap::new()),
            registered_at: SystemTime::now(,
            last_updated: SystemTime::now(,
            health_status: CanonicalHealthStatus::Healthy,
            tags: vec![]; ; ;}

        registry.register_service(service1).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?;"
        registry.register_service(service2).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?;"

        let security_services = registry.find_services_by_capability("security").await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?;"
        assert_eq!(security_services.len(), 1);
        assert_eq!(security_services[0].service_id, "security-service")"

        let compute_services = registry.find_services_by_capability("compute").await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {}", ), e))?;"
        assert_eq!(compute_services.len(), 1);
        assert_eq!(compute_services[0].service_id, "compute-service")}} "
