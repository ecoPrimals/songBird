// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Service Registry Persistence
//!
//! Real persistent storage for service registry replacing mock implementations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::persistence::service_data::{RegistryServiceEntry, ServiceInfo};

/// Select how registry snapshots are stored and loaded.
#[derive(Debug, Clone)]
pub enum StorageBackend {
    /// Persist JSON snapshots under a directory tree.
    FileSystem {
        /// Root directory for `registry.json` and rolling backups.
        data_dir: PathBuf,
    },
    /// Keep state only in process memory (optional durability hint).
    InMemory {
        /// When true, retain cache across logical reloads within the process.
        persistent: bool,
    },
    /// Future database backend (currently falls back to filesystem).
    Database {
        /// Connection URI for the backing database.
        connection_string: String,
    },
}

/// Tune autosave cadence, retention, and backend selection for registry persistence.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Storage backend
    pub backend: StorageBackend,
    /// Interval between automatic `save_to_storage` runs (zero disables the task).
    pub auto_save_interval: std::time::Duration,
    /// Number of rotated JSON backups to keep on disk.
    pub backup_retention: u32,
    /// When true, compress snapshots before writing (reserved for future use).
    pub compression_enabled: bool,
}

/// Serializable registry snapshot written to `registry.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentServiceData {
    /// Service entries
    pub services: HashMap<String, RegistryServiceEntry>,
    /// Service information
    pub service_info: HashMap<String, ServiceInfo>,
    /// Registration timestamps
    pub registration_times: HashMap<String, DateTime<Utc>>,
    /// Last persistence time
    pub last_saved: DateTime<Utc>,
    /// Schema version
    pub schema_version: u32,
}

/// Owns the registry cache, optional autosave loop, and storage I/O.
pub struct ProductionServicePersistence {
    /// Persistence configuration
    config: PersistenceConfig,
    /// In-memory cache
    cache: Arc<RwLock<PersistentServiceData>>,
    /// Auto-save task handle
    auto_save_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Persistence statistics
    stats: Arc<RwLock<PersistenceStatistics>>,
}

/// Counters surfaced to operators for persistence health.
#[derive(Debug, Default, Clone)]
pub struct PersistenceStatistics {
    /// Total save attempts initiated.
    pub total_saves: u64,
    /// Saves that completed without error.
    pub successful_saves: u64,
    /// Saves that failed after persistence errors.
    pub failed_saves: u64,
    /// Total load attempts initiated.
    pub total_loads: u64,
    /// Loads that completed without error.
    pub successful_loads: u64,
    /// Loads that failed after I/O or parse errors.
    pub failed_loads: u64,
    /// Approximate serialized size of the in-memory cache.
    pub cache_size_bytes: u64,
    /// Duration of the most recent successful save in milliseconds.
    pub last_save_duration_ms: u64,
    /// Duration of the most recent successful load in milliseconds.
    pub last_load_duration_ms: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::FileSystem {
                data_dir: PathBuf::from("./data/registry"),
            },
            auto_save_interval: std::time::Duration::from_secs(60),
            backup_retention: 10,
            compression_enabled: true,
        }
    }
}

impl ProductionServicePersistence {
    /// Create new production service persistence
    pub async fn new(config: PersistenceConfig) -> SongbirdResult<Self> {
        let cache = Arc::new(RwLock::new(PersistentServiceData {
            services: HashMap::new(),
            service_info: HashMap::new(),
            registration_times: HashMap::new(),
            last_saved: Utc::now(),
            schema_version: 1,
        }));

        let persistence = Self {
            config,
            cache,
            auto_save_task: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(PersistenceStatistics::default())),
        };

        persistence.load_from_storage().await?;

        if persistence.config.auto_save_interval.as_secs() > 0 {
            persistence.start_auto_save().await?;
        }

        info!("Production service persistence initialized");
        Ok(persistence)
    }

    /// Save service entry
    pub async fn save_service(
        &self,
        service_id: &str,
        entry: &RegistryServiceEntry,
    ) -> SongbirdResult<()> {
        let mut cache = self.cache.write().await;
        cache.services.insert(service_id.to_string(), entry.clone());
        cache.registration_times.insert(service_id.to_string(), Utc::now());

        debug!("Cached service entry: {}", service_id);
        Ok(())
    }

    /// Save service info
    pub async fn save_service_info(
        &self,
        service_id: &str,
        info: &ServiceInfo,
    ) -> SongbirdResult<()> {
        let mut cache = self.cache.write().await;
        cache.service_info.insert(service_id.to_string(), info.clone());

        debug!("Cached service info: {}", service_id);
        Ok(())
    }

    /// Load service entry
    pub async fn load_service(
        &self,
        service_id: &str,
    ) -> SongbirdResult<Option<RegistryServiceEntry>> {
        let cache = self.cache.read().await;
        Ok(cache.services.get(service_id).cloned())
    }

    /// Load service info
    pub async fn load_service_info(&self, service_id: &str) -> SongbirdResult<Option<ServiceInfo>> {
        let cache = self.cache.read().await;
        Ok(cache.service_info.get(service_id).cloned())
    }

    /// Remove service from persistence
    pub async fn remove_service(&self, service_id: &str) -> SongbirdResult<()> {
        let mut cache = self.cache.write().await;
        cache.services.remove(service_id);
        cache.service_info.remove(service_id);
        cache.registration_times.remove(service_id);

        info!("Removed service from persistence: {}", service_id);
        Ok(())
    }

    /// Get all service IDs
    pub async fn get_all_service_ids(&self) -> SongbirdResult<Vec<String>> {
        let cache = self.cache.read().await;
        Ok(cache.services.keys().cloned().collect())
    }

    /// Save to persistent storage
    pub async fn save_to_storage(&self) -> SongbirdResult<()> {
        let save_start = std::time::Instant::now();

        match &self.config.backend {
            StorageBackend::FileSystem {
                data_dir,
            } => {
                self.save_to_filesystem(data_dir).await?;
            }
            StorageBackend::InMemory {
                persistent,
            } => {
                if *persistent {
                    debug!("InMemory backend with persistence - data retained in cache");
                }
            }
            StorageBackend::Database {
                connection_string,
            } => {
                self.save_to_database(connection_string).await?;
            }
        }

        let mut cache = self.cache.write().await;
        cache.last_saved = Utc::now();

        let duration_ms = u64::try_from(save_start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.update_save_stats(true, duration_ms).await;

        debug!("Registry data saved to storage in {}ms", duration_ms);
        Ok(())
    }

    /// Save to filesystem
    async fn save_to_filesystem(&self, data_dir: &PathBuf) -> SongbirdResult<()> {
        fs::create_dir_all(data_dir).await.map_err(|e| {
            SongbirdError::service("persistence", format!("Failed to create data directory: {e}"))
        })?;

        let cache = self.cache.read().await;
        let data = cache.clone();
        drop(cache);

        let json_data = serde_json::to_string_pretty(&data).map_err(|e| {
            SongbirdError::service("persistence", format!("Serialization failed: {e}"))
        })?;

        let main_file = data_dir.join("registry.json");
        let backup_file = data_dir.join(format!("registry_backup_{}.json", Utc::now().timestamp()));

        if main_file.exists() {
            fs::copy(&main_file, &backup_file).await.map_err(|e| {
                SongbirdError::service("persistence", format!("Backup creation failed: {e}"))
            })?;
        }

        fs::write(&main_file, json_data).await.map_err(|e| {
            SongbirdError::service("persistence", format!("File write failed: {e}"))
        })?;

        self.cleanup_old_backups(data_dir).await?;

        Ok(())
    }

    /// Save to database — falls back to filesystem until a DB driver is integrated
    async fn save_to_database(&self, _connection_string: &str) -> SongbirdResult<()> {
        warn!("Database backend not yet implemented, falling back to filesystem");

        let fallback_dir = PathBuf::from("./data/registry_db_fallback");
        self.save_to_filesystem(&fallback_dir).await
    }

    /// Load from persistent storage
    pub async fn load_from_storage(&self) -> SongbirdResult<()> {
        let load_start = std::time::Instant::now();

        match &self.config.backend {
            StorageBackend::FileSystem {
                data_dir,
            } => {
                self.load_from_filesystem(data_dir).await?;
            }
            StorageBackend::InMemory {
                ..
            } => {
                debug!("InMemory backend - no persistent data to load");
            }
            StorageBackend::Database {
                connection_string,
            } => {
                self.load_from_database(connection_string).await?;
            }
        }

        let duration_ms = u64::try_from(load_start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.update_load_stats(true, duration_ms).await;

        debug!("Registry data loaded from storage in {}ms", duration_ms);
        Ok(())
    }

    /// Load from filesystem
    async fn load_from_filesystem(&self, data_dir: &Path) -> SongbirdResult<()> {
        let main_file = data_dir.join("registry.json");

        if !main_file.exists() {
            debug!("No existing registry file found, starting with empty registry");
            return Ok(());
        }

        let json_data = fs::read_to_string(&main_file)
            .await
            .map_err(|e| SongbirdError::service("persistence", format!("File read failed: {e}")))?;

        let data: PersistentServiceData = serde_json::from_str(&json_data).map_err(|e| {
            SongbirdError::service("persistence", format!("Deserialization failed: {e}"))
        })?;

        let mut cache = self.cache.write().await;
        *cache = data;

        info!("Loaded {} services from persistent storage", cache.services.len());
        Ok(())
    }

    /// Load from database — falls back to filesystem until a DB driver is integrated
    async fn load_from_database(&self, _connection_string: &str) -> SongbirdResult<()> {
        warn!("Database backend not yet implemented, falling back to filesystem");

        let fallback_dir = PathBuf::from("./data/registry_db_fallback");
        self.load_from_filesystem(&fallback_dir).await
    }

    /// Start auto-save task
    async fn start_auto_save(&self) -> SongbirdResult<()> {
        let persistence = self.clone();
        let interval = self.config.auto_save_interval;

        let task = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if let Err(e) = persistence.save_to_storage().await {
                    error!("Auto-save failed: {}", e);
                }
            }
        });

        let mut auto_save_task = self.auto_save_task.write().await;
        *auto_save_task = Some(task);

        info!("Auto-save started with interval: {:?}", interval);
        Ok(())
    }

    /// Cleanup old backup files
    async fn cleanup_old_backups(&self, data_dir: &PathBuf) -> SongbirdResult<()> {
        if let Ok(mut entries) = fs::read_dir(data_dir).await {
            let mut backup_files = Vec::new();

            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str()
                    && name.starts_with("registry_backup_")
                    && std::path::Path::new(name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                {
                    backup_files.push((entry.path(), entry.metadata().await.ok()));
                }
            }

            backup_files.sort_by(|a, b| {
                let a_time = a.1.as_ref().and_then(|m| m.modified().ok());
                let b_time = b.1.as_ref().and_then(|m| m.modified().ok());
                b_time.cmp(&a_time)
            });

            for (path, _) in backup_files.into_iter().skip(self.config.backup_retention as usize) {
                if let Err(e) = fs::remove_file(&path).await {
                    warn!("Failed to remove old backup {:?}: {}", path, e);
                }
            }
        }

        Ok(())
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
    pub async fn stop(&self) -> SongbirdResult<()> {
        let mut task = self.auto_save_task.write().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }

        self.save_to_storage().await?;

        info!("Service persistence stopped");
        Ok(())
    }
}

impl Clone for ProductionServicePersistence {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: Arc::clone(&self.cache),
            auto_save_task: Arc::clone(&self.auto_save_task),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::persistence::service_data::{
        RegistryHealthStatus, RegistryServiceEntry, RegistryServiceMetrics,
    };
    use chrono::Utc;
    use songbird_discovery::traits::service::ServiceStatus;
    use std::collections::HashMap;

    fn sample_info(id: &str) -> ServiceInfo {
        let now = Utc::now();
        ServiceInfo {
            service_id: id.to_string(),
            name: "test-svc".to_string(),
            version: "1.0.0".to_string(),
            service_type: "unit".to_string(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
            instance_id: format!("{id}-inst"),
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }

    fn sample_entry(id: &str) -> RegistryServiceEntry {
        RegistryServiceEntry {
            service_info: sample_info(id),
            instance_count: 1,
            max_instances: 3,
            min_instances: 1,
            health_status: RegistryHealthStatus::Healthy,
            metrics: RegistryServiceMetrics::default(),
        }
    }

    fn in_memory_config() -> PersistenceConfig {
        PersistenceConfig {
            backend: StorageBackend::InMemory {
                persistent: false,
            },
            auto_save_interval: std::time::Duration::from_secs(0),
            backup_retention: 3,
            compression_enabled: false,
        }
    }

    #[tokio::test]
    async fn in_memory_save_load_remove_roundtrip() {
        let p =
            ProductionServicePersistence::new(in_memory_config()).await.expect("persistence init");

        p.save_service("svc-a", &sample_entry("svc-a")).await.expect("save entry");
        p.save_service_info("svc-a", &sample_info("svc-a")).await.expect("save info");

        let e = p.load_service("svc-a").await.expect("load entry");
        assert!(e.is_some());
        assert_eq!(e.expect("entry").service_info.service_id, "svc-a");

        let i = p.load_service_info("svc-a").await.expect("load info");
        assert_eq!(i.expect("info").port, 8080);

        let ids = p.get_all_service_ids().await.expect("ids");
        assert_eq!(ids.len(), 1);

        p.remove_service("svc-a").await.expect("remove");
        assert!(p.load_service("svc-a").await.expect("load").is_none());
    }

    #[tokio::test]
    async fn save_to_storage_in_memory_updates_stats() {
        let p =
            ProductionServicePersistence::new(in_memory_config()).await.expect("persistence init");

        p.save_service("x", &sample_entry("x")).await.expect("save");
        p.save_to_storage().await.expect("save to storage");

        let s = p.get_statistics().await;
        assert!(s.successful_saves >= 1);
        assert_eq!(s.total_saves, s.successful_saves);
    }

    #[test]
    fn persistent_service_data_json_roundtrip() {
        let mut data = PersistentServiceData {
            services: HashMap::new(),
            service_info: HashMap::new(),
            registration_times: HashMap::new(),
            last_saved: Utc::now(),
            schema_version: 1,
        };
        data.services.insert("k".to_string(), sample_entry("k"));
        data.service_info.insert("k".to_string(), sample_info("k"));

        let json = serde_json::to_string(&data).expect("serialize");
        let back: PersistentServiceData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.services.get("k").expect("entry").instance_count, 1);
        assert_eq!(back.schema_version, 1);
    }
}
