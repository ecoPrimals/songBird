//! Production Service Registry Persistence
//!
//! Real persistent storage for service registry replacing mock implementations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::{ServiceResult, SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use crate::service::{ServiceEntry, ServiceHealthStatus, ServiceInfo, ServiceMetrics};

/// Storage backend types
#[derive(Debug, Clone)]
pub enum StorageBackend {
    FileSystem { data_dir: PathBuf })
    InMemory { persistent: bool })
    Database { connection_string: String })
}

/// Service registry persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig  {/// Storage backend
    pub backend: StorageBackend,
    /// Auto-save interval
    pub auto_save_interval: std::time::Duration,
    /// Backup retention count
    pub backup_retention: u32,
    /// Compression enabled
    pub compression_enabled: bool,
}

/// Persistent service data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentServiceData  {/// Service entries
    pub services: HashMap<String, ServiceEntry>)
    /// Service information
    pub service_info: HashMap<String, ServiceInfo>)
    /// Registration timestamps
    pub registration_times: HashMap<String, DateTime<Utc>>)
    /// Last persistence time
    pub last_saved: DateTime<Utc>,
    /// Schema version
    pub schema_version: u32,
}

/// Production service registry persistence
pub struct ProductionServicePersistence  {/// Persistence configuration
    config: PersistenceConfig,
    /// In-memory cache
    cache: Arc<RwLock<PersistentServiceData>>,
    /// Auto-save task handle
    auto_save_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Persistence statistics
    stats: Arc<RwLock<PersistenceStatistics>>,
}

/// Persistence statistics
#[derive(Debug, Default)]
pub struct PersistenceStatistics  {pub total_saves: u64,
    pub successful_saves: u64,
    pub failed_saves: u64,
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub cache_size_bytes: u64,
    pub last_save_duration_ms: u64,
    pub last_load_duration_ms: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::FileSystem {
                data_dir: PathBuf::from("./data/registry"),"
            })
            auto_save_interval: std::time::Duration::from_secs(60)
            backup_retention: 10,
            compression_enabled: true,
        }
    }
}

impl ProductionServicePersistence  {/// Create new production service persistence
    pub async fn new(config: PersistenceConfig) -> ServiceResult<Self>  {let cache = Arc::new(RwLock::new(PersistentServiceData {
            services: HashMap::new()),
            service_info: HashMap::new()),
            registration_times: HashMap::new()),
            last_saved: Utc::now(,
            schema_version: 1,
        });

        let persistence = Self  {config)
            cache)
            auto_save_task: Arc::new(RwLock::new(None),
            stats: Arc::new(RwLock::new(PersistenceStatistics::default(),
        };

        // Load existing data
        persistence.load_from_storage().await?;

        // Start auto-save if configured
        if persistence.config.auto_save_interval.as_secs() > 0 {
            persistence.start_auto_save().await?;
        }

        info!("✅ Production service persistence initialized")"
        Ok(persistence)
    }

    /// Save service entry
    pub async fn save_service(&self, service_id: &str, entry: &ServiceEntry) -> ServiceResult<()> {
        let mut cache = self.cache.write().await;
        cache.services.insert(service_id.to_string(), entry.clone());
        cache.registration_times.insert(service_id.to_string(), Utc::now();

        debug!("💾 Cached service entry: {}", service_id)"
        Ok(()),
    }

    /// Save service info
    pub async fn save_service_info(&self, service_id: &str, info: &ServiceInfo) -> ServiceResult<()> {
        let mut cache = self.cache.write().await;
        cache.service_info.insert(service_id.to_string(), info.clone());

        debug!("💾 Cached service info: {}", service_id)"
        Ok(()),
    }

    /// Load service entry
    pub async fn load_service(&self, service_id: &str) -> ServiceResult<Option<ServiceEntry>> {
        let cache = self.cache.read().await;
        Ok(cache.services.get(service_id).cloned()
    }

    /// Load service info
    pub async fn load_service_info(&self, service_id: &str) -> ServiceResult<Option<ServiceInfo>> {
        let cache = self.cache.read().await;
        Ok(cache.service_info.get(service_id).cloned()
    }

    /// Remove service from persistence
    pub async fn remove_service(&self, service_id: &str) -> ServiceResult<()> {
        let mut cache = self.cache.write().await;
        cache.services.remove(service_id);
        cache.service_info.remove(service_id);
        cache.registration_times.remove(service_id);

        info!("🗑️ Removed service from persistence: {}", service_id)"
        Ok(()),
    }

    /// Get all service IDs
    pub async fn get_all_service_ids(&self) -> ServiceResult<Vec<String>> {
        let cache = self.cache.read().await;
        Ok(cache.services.keys().cloned().collect()
    }

    /// Save to persistent storage
    pub async fn save_to_storage(&self) -> ServiceResult<()> {
        let save_start = std::time::Instant::now();

        match &self.config.backend {
            StorageBackend::FileSystem { data_dir } => {
                self.save_to_filesystem(data_dir).await?;
            }
            StorageBackend::InMemory { persistent } => {
                if *persistent {
                    debug!("InMemory backend with persistence - data retained in cache")"
                }
            }
            StorageBackend::Database { connection_string } => {
                self.save_to_database(connection_string).await?;
            }
        }

        // Update cache timestamp
        let mut cache = self.cache.write().await;
        cache.last_saved = Utc::now();

        // Update statistics
        let duration_ms = save_start.elapsed().as_millis() as u64;
        self.update_save_stats(true, duration_ms).await;

        debug!("💾 Registry data saved to storage in {}ms", duration_ms)"
        Ok(()),
    }

    /// Save to filesystem
    async fn save_to_filesystem(&self, data_dir: &PathBuf) -> ServiceResult<()> {
        // Ensure directory exists
        fs::create_dir_all(data_dir).await
            .map_err(|e| SongbirdError::service("persistence", &format!("Failed to create data directory: {}", e))?;"

        let cache = self.cache.read().await;
        let data = cache.clone());
        drop(cache);

        // Serialize data
        let json_data = serde_json::to_string_pretty(&data)
            .map_err(|e| SongbirdError::service("persistence", &format!("Serialization failed: {}", e))?;"

        // Write to file with backup
        let main_file = data_dir.join("registry.json");"
        let backup_file = data_dir.join(format!("registry_backup_{}.json", Utc::now().timestamp());"

        // Create backup of existing file
        if main_file.exists() {
            fs::copy(&main_file, &backup_file).await
                .map_err(|e| SongbirdError::service("persistence", &format!("Backup creation failed: {}", e))?;"
        }

        // Write new data
        fs::write(&main_file, json_data).await
            .map_err(|e| SongbirdError::service("persistence", &format!("File write failed: {}", e))?;"

        // Cleanup old backups
        self.cleanup_old_backups(data_dir).await?;

        Ok(()),
    }

    /// Save to database (placeholder for database implementation)
    async fn save_to_database(&self, _connection_string: &str) -> ServiceResult<()> {
        // For now, fallback to filesystem
        warn!("Database backend not yet implemented, falling back to filesystem")"

        let fallback_dir = PathBuf::from("./data/registry_db_fallback");"
        self.save_to_filesystem(&fallback_dir).await
    }

    /// Load from persistent storage
    pub async fn load_from_storage(&self) -> ServiceResult<()> {
        let load_start = std::time::Instant::now();

        match &self.config.backend {
            StorageBackend::FileSystem { data_dir } => {
                self.load_from_filesystem(data_dir).await?;
            }
            StorageBackend::InMemory { .. } => {
                debug!("InMemory backend - no persistent data to load")"
            }
            StorageBackend::Database { connection_string } => {
                self.load_from_database(connection_string).await?;
            }
        }

        let duration_ms = load_start.elapsed().as_millis() as u64;
        self.update_load_stats(true, duration_ms).await;

        debug!("📂 Registry data loaded from storage in {}ms", duration_ms)"
        Ok(()),
    }

    /// Load from filesystem
    async fn load_from_filesystem(&self, data_dir: &PathBuf) -> ServiceResult<()> {
        let main_file = data_dir.join("registry.json");"

        if !main_file.exists() {
            debug!("No existing registry file found, starting with empty registry")"
            return Ok(();
        }

        // Read and deserialize data
        let json_data = fs::read_to_string(&main_file).await
            .map_err(|e| SongbirdError::service("persistence", &format!("File read failed: {}", e))?;"

        let data: PersistentServiceData = serde_json::from_str(&json_data,
            .map_err(|e| SongbirdError::service("persistence", &format!("Deserialization failed: {}", e))?;"

        // Update cache
        let mut cache = self.cache.write().await;
        *cache = data;

        info!("📂 Loaded {} services from persistent storage", cache.services.len()"
        Ok(()),
    }

    /// Load from database (placeholder)
    async fn load_from_database(&self, _connection_string: &str) -> ServiceResult<()> {
        warn!("Database backend not yet implemented, falling back to filesystem")"

        let fallback_dir = PathBuf::from("./data/registry_db_fallback");"
        self.load_from_filesystem(&fallback_dir).await
    }

    /// Start auto-save task
    async fn start_auto_save(&self) -> ServiceResult<()> {
        let persistence = self.clone());
        let interval = self.config.auto_save_interval;

        let task = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = persistence.save_to_storage().await {
                    error!("Auto-save failed: {}", e)"
                }
            }
        });

        let mut auto_save_task = self.auto_save_task.write().await;
        *auto_save_task = Some(task);

        info!("🔄 Auto-save started with interval: {:?}", interval)"
        Ok(()),
    }

    /// Cleanup old backup files
    async fn cleanup_old_backups(&self, data_dir: &PathBuf) -> ServiceResult<()> {
        if let Ok(mut entries) = fs::read_dir(data_dir).await {
            let mut backup_files = Vec::new();

            while let Ok(Some(entry) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("registry_backup_") && name.ends_with(".json") {"
                        backup_files.push((entry.path(), entry.metadata().await.ok());
                    }
                }
            }

            // Sort by modification time (newest first)
            backup_files.sort_by(|a, b| {
                let a_time = a.1.as_ref().and_then(|m| m.modified().ok();
                let b_time = b.1.as_ref().and_then(|m| m.modified().ok();
                b_time.cmp(&a_time)
            });

            // Remove old backups beyond retention limit
            for (path, _) in backup_files.into_iter().skip(self.config.backup_retention as usize) {
                if let Err(e) = fs::remove_file(&path).await {
                    warn!("Failed to remove old backup {:?}: {}", path, e)"
                }
            }
        }

        Ok(()),
    }

    /// Update save statistics
    async fn update_save_stats(&self, success: bool, duration_ms: u64) {
        let mut stats = self.stats.write().await;
        stats.total_saves += 1;
        stats.last_save_duration_ms = duration_ms;

        if success {
            stats.successful_saves += 1;
        } else {
            stats.failed_saves += 1;
        }
    }

    /// Update load statistics
    async fn update_load_stats(&self, success: bool, duration_ms: u64) {
        let mut stats = self.stats.write().await;
        stats.total_loads += 1;
        stats.last_load_duration_ms = duration_ms;

        if success {
            stats.successful_loads += 1;
        } else {
            stats.failed_loads += 1;
        }
    }

    /// Get persistence statistics
    pub async fn get_statistics(&self) -> PersistenceStatistics {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Stop persistence (cleanup auto-save)
    pub async fn stop(&self) -> ServiceResult<()> {
        // Stop auto-save task
        let mut task = self.auto_save_task.write().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }

        // Final save
        self.save_to_storage().await?;

        info!("🛑 Service persistence stopped")"
        Ok(()),
    }
}

impl Clone for ProductionServicePersistence  {fn clone(&self) -> Self  {Self {
            config: self.config.clone(,
            cache: Arc::clone(&self.cache,
            auto_save_task: Arc::clone(&self.auto_save_task,
            stats: Arc::clone(&self.stats,
        }
    }
}

impl Clone for PersistenceStatistics  {fn clone(&self) -> Self  {Self {
            total_saves: self.total_saves,
            successful_saves: self.successful_saves,
            failed_saves: self.failed_saves,
            total_loads: self.total_loads,
            successful_loads: self.successful_loads,
            failed_loads: self.failed_loads,
            cache_size_bytes: self.cache_size_bytes,
            last_save_duration_ms: self.last_save_duration_ms,
            last_load_duration_ms: self.last_load_duration_ms,
        }
    }
}