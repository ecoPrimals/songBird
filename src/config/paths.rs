//! Platform-Agnostic Paths Module
//!
//! Provides OS-appropriate paths through toadstool and biomeOS substrate
//! instead of direct platform-specific operations

use crate::errors::{Result, SongbirdError};
use crate::substrate::{OSSubstrate, PathRequest, PathRequirements, PathType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, warn};

/// Platform-agnostic path configuration using OS substrate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Data directory for persistent storage
    pub data_dir: PathBuf,

    /// Configuration directory
    pub config_dir: PathBuf,

    /// Log directory
    pub log_dir: PathBuf,

    /// Cache directory for temporary files
    pub cache_dir: PathBuf,

    /// Runtime directory for PID files, sockets, etc.
    pub runtime_dir: PathBuf,

    /// Service-specific data directories
    pub service_data_dirs: ServiceDataDirs,
}

/// Service-specific data directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDataDirs {
    pub orchestrator: PathBuf,
    pub federation: PathBuf,
    pub metrics: PathBuf,
    pub discovery: PathBuf,
    pub registry: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self::new_fallback()
    }
}

impl PathConfig {
    /// Create new path configuration using OS substrate
    pub async fn new() -> Result<Self> {
        let substrate = crate::substrate::get_substrate().await;

        debug!("🌍 Requesting paths from OS substrate (toadstool/biomeOS)");

        // Request paths from substrate
        let data_dir = substrate.get_data_dir("songbird").await?;
        let config_dir = substrate.get_config_dir("songbird").await?;
        let log_dir = substrate.get_log_dir("songbird").await?;
        let cache_dir = substrate
            .get_path(PathRequest {
                path_type: PathType::Cache,
                service_name: "songbird".to_string(),
                requirements: PathRequirements::default(),
            })
            .await?;
        let runtime_dir = substrate
            .get_path(PathRequest {
                path_type: PathType::Runtime,
                service_name: "songbird".to_string(),
                requirements: PathRequirements::default(),
            })
            .await?;

        // Request service-specific directories
        let service_data_dirs = ServiceDataDirs {
            orchestrator: substrate.get_data_dir("orchestrator").await?,
            federation: substrate.get_data_dir("federation").await?,
            metrics: substrate.get_data_dir("metrics").await?,
            discovery: substrate.get_data_dir("discovery").await?,
            registry: substrate.get_data_dir("registry").await?,
        };

        Ok(Self {
            data_dir,
            config_dir,
            log_dir,
            cache_dir,
            runtime_dir,
            service_data_dirs,
        })
    }

    /// Create fallback path configuration when substrate is unavailable
    pub fn new_fallback() -> Self {
        warn!("🔄 Using fallback path configuration (substrate unavailable)");

        let base_data_dir = Self::get_fallback_data_dir();
        let base_config_dir = Self::get_fallback_config_dir();
        let base_log_dir = Self::get_fallback_log_dir();
        let base_cache_dir = Self::get_fallback_cache_dir();
        let base_runtime_dir = Self::get_fallback_runtime_dir();

        Self {
            data_dir: base_data_dir.clone(),
            config_dir: base_config_dir,
            log_dir: base_log_dir,
            cache_dir: base_cache_dir,
            runtime_dir: base_runtime_dir,
            service_data_dirs: ServiceDataDirs {
                orchestrator: base_data_dir.join("orchestrator"),
                federation: base_data_dir.join("federation"),
                metrics: base_data_dir.join("metrics"),
                discovery: base_data_dir.join("discovery"),
                registry: base_data_dir.join("registry"),
            },
        }
    }

    /// Create development configuration (uses local directories)
    pub fn development() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let dev_dir = current_dir.join(".songbird");

        Self {
            data_dir: dev_dir.join("data"),
            config_dir: dev_dir.join("config"),
            log_dir: dev_dir.join("logs"),
            cache_dir: dev_dir.join("cache"),
            runtime_dir: dev_dir.join("runtime"),
            service_data_dirs: ServiceDataDirs {
                orchestrator: dev_dir.join("data").join("orchestrator"),
                federation: dev_dir.join("data").join("federation"),
                metrics: dev_dir.join("data").join("metrics"),
                discovery: dev_dir.join("data").join("discovery"),
                registry: dev_dir.join("data").join("registry"),
            },
        }
    }

    /// Create production configuration using substrate
    pub async fn production() -> Result<Self> {
        Self::new().await
    }

    /// Get fallback data directory when substrate is unavailable
    fn get_fallback_data_dir() -> PathBuf {
        // Check environment variable first
        if let Ok(data_dir) = std::env::var("SONGBIRD_DATA_DIR") {
            return PathBuf::from(data_dir);
        }

        // Use constants instead of direct platform detection
        PathBuf::from(crate::config::constants::paths::DEFAULT_DATA_DIR)
    }

    /// Get fallback config directory when substrate is unavailable
    fn get_fallback_config_dir() -> PathBuf {
        if let Ok(config_dir) = std::env::var("SONGBIRD_CONFIG_DIR") {
            return PathBuf::from(config_dir);
        }

        PathBuf::from(crate::config::constants::paths::DEFAULT_CONFIG_DIR)
    }

    /// Get fallback log directory when substrate is unavailable
    fn get_fallback_log_dir() -> PathBuf {
        if let Ok(log_dir) = std::env::var("SONGBIRD_LOG_DIR") {
            return PathBuf::from(log_dir);
        }

        PathBuf::from(crate::config::constants::paths::DEFAULT_LOG_DIR)
    }

    /// Get fallback cache directory when substrate is unavailable
    fn get_fallback_cache_dir() -> PathBuf {
        if let Ok(cache_dir) = std::env::var("SONGBIRD_CACHE_DIR") {
            return PathBuf::from(cache_dir);
        }

        PathBuf::from(crate::config::constants::paths::DEFAULT_CACHE_DIR)
    }

    /// Get fallback runtime directory when substrate is unavailable
    fn get_fallback_runtime_dir() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("SONGBIRD_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir);
        }

        PathBuf::from(crate::config::constants::paths::DEFAULT_TEMP_DIR)
    }

    /// Create all necessary directories through substrate
    pub async fn ensure_directories_exist(&self) -> Result<()> {
        let substrate = crate::substrate::get_substrate().await;

        // Request directory creation through substrate
        let directories = vec![
            ("data", &self.data_dir),
            ("config", &self.config_dir),
            ("log", &self.log_dir),
            ("cache", &self.cache_dir),
            ("runtime", &self.runtime_dir),
        ];

        for (dir_type, path) in directories {
            if let Err(e) = substrate
                .container_operation(
                    "ensure_directory",
                    serde_json::json!({
                        "path": path,
                        "type": dir_type,
                        "recursive": true
                    }),
                )
                .await
            {
                warn!(
                    "Failed to create directory {} through substrate: {}",
                    path.display(),
                    e
                );
                // Fallback to direct creation
                if let Err(create_err) = std::fs::create_dir_all(path) {
                    return Err(SongbirdError::Config {
                        message: format!(
                            "Failed to create directory {}: {}",
                            path.display(),
                            create_err
                        ),
                        field: Some("directory_path".to_string()),
                    });
                }
            }
        }

        // Create service-specific directories
        let service_directories = vec![
            ("orchestrator", &self.service_data_dirs.orchestrator),
            ("federation", &self.service_data_dirs.federation),
            ("metrics", &self.service_data_dirs.metrics),
            ("discovery", &self.service_data_dirs.discovery),
            ("registry", &self.service_data_dirs.registry),
        ];

        for (service, path) in service_directories {
            if let Err(e) = substrate
                .container_operation(
                    "ensure_directory",
                    serde_json::json!({
                        "path": path,
                        "type": "service_data",
                        "service": service,
                        "recursive": true
                    }),
                )
                .await
            {
                warn!(
                    "Failed to create service directory {} through substrate: {}",
                    path.display(),
                    e
                );
                // Fallback to direct creation
                if let Err(create_err) = std::fs::create_dir_all(path) {
                    return Err(SongbirdError::Config {
                        message: format!(
                            "Failed to create service directory {}: {}",
                            path.display(),
                            create_err
                        ),
                        field: Some("service_directory".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get path for a specific service through substrate
    pub async fn get_service_path(
        &self,
        service_name: &str,
        path_type: PathType,
    ) -> Result<PathBuf> {
        let substrate = crate::substrate::get_substrate().await;

        substrate
            .get_path(PathRequest {
                path_type,
                service_name: service_name.to_string(),
                requirements: PathRequirements::default(),
            })
            .await
    }

    /// Validate that all paths are accessible through substrate
    pub async fn validate_paths(&self) -> Result<()> {
        let substrate = crate::substrate::get_substrate().await;

        let paths_to_check = vec![
            ("data", &self.data_dir),
            ("config", &self.config_dir),
            ("log", &self.log_dir),
            ("cache", &self.cache_dir),
            ("runtime", &self.runtime_dir),
        ];

        for (path_type, path) in paths_to_check {
            match substrate
                .container_operation(
                    "validate_path",
                    serde_json::json!({
                        "path": path,
                        "type": path_type,
                        "check_writable": true
                    }),
                )
                .await
            {
                Ok(_) => debug!("✅ Path {} validated through substrate", path.display()),
                Err(e) => {
                    warn!(
                        "❌ Path {} validation failed through substrate: {}",
                        path.display(),
                        e
                    );
                    // Fallback to direct validation
                    if !path.exists() {
                        return Err(SongbirdError::Config {
                            message: format!(
                                "Path does not exist: {}",
                                path.display()
                            ),
                            field: Some("path_validation".to_string()),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Get temporary directory for a specific operation
    pub async fn get_temp_dir(&self, operation: &str) -> Result<PathBuf> {
        let substrate = crate::substrate::get_substrate().await;

        substrate
            .get_path(PathRequest {
                path_type: PathType::Temp,
                service_name: format!("songbird_{}", operation),
                requirements: PathRequirements {
                    min_size_bytes: Some(1024 * 1024), // 1MB minimum
                    permissions: Some("rw".to_string()),
                    persistent: false,
                    shared: false,
                },
            })
            .await
    }

    /// Clean up temporary directories through substrate
    pub async fn cleanup_temp_dirs(&self) -> Result<()> {
        let substrate = crate::substrate::get_substrate().await;

        match substrate
            .container_operation(
                "cleanup_temp",
                serde_json::json!({
                    "service": "songbird",
                    "max_age_hours": 24
                }),
            )
            .await
        {
            Ok(_) => debug!("✅ Temporary directories cleaned through substrate"),
            Err(e) => warn!(
                "❌ Failed to clean temporary directories through substrate: {}",
                e
            ),
        }

        Ok(())
    }
}

/// Get the best available path configuration
pub async fn get_path_config() -> Result<PathConfig> {
    // Try to use substrate first
    match PathConfig::new().await {
        Ok(config) => {
            debug!("✅ Using substrate-based path configuration");
            Ok(config)
        }
        Err(e) => {
            warn!(
                "⚠️ Substrate path configuration failed: {}, using fallback",
                e
            );
            Ok(PathConfig::new_fallback())
        }
    }
}

/// Initialize paths for a service through substrate
pub async fn initialize_service_paths(service_name: &str) -> Result<ServiceDataDirs> {
    let substrate = crate::substrate::get_substrate().await;

    Ok(ServiceDataDirs {
        orchestrator: substrate
            .get_data_dir(&format!("{}_orchestrator", service_name))
            .await?,
        federation: substrate
            .get_data_dir(&format!("{}_federation", service_name))
            .await?,
        metrics: substrate
            .get_data_dir(&format!("{}_metrics", service_name))
            .await?,
        discovery: substrate
            .get_data_dir(&format!("{}_discovery", service_name))
            .await?,
        registry: substrate
            .get_data_dir(&format!("{}_registry", service_name))
            .await?,
    })
}

/// Create a path configuration for testing
pub fn testing_config() -> PathConfig {
    let test_dir = std::env::temp_dir().join("songbird_test");

    PathConfig {
        data_dir: test_dir.join("data"),
        config_dir: test_dir.join("config"),
        log_dir: test_dir.join("logs"),
        cache_dir: test_dir.join("cache"),
        runtime_dir: test_dir.join("runtime"),
        service_data_dirs: ServiceDataDirs {
            orchestrator: test_dir.join("data").join("orchestrator"),
            federation: test_dir.join("data").join("federation"),
            metrics: test_dir.join("data").join("metrics"),
            discovery: test_dir.join("data").join("discovery"),
            registry: test_dir.join("data").join("registry"),
        },
    }
}
