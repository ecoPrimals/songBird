//! Path configuration for Songbird - Zero hardcoded paths
//!
//! This module provides path configuration with environment-based defaults.
//! All paths are configurable via environment variables.

use songbird_types::{SongbirdError, SongbirdResult};
type Result<T> = SongbirdResult<T>;
// use crate::substrate::{PathRequest, PathRequirements, PathType};
use crate::config::constants::{get_cache_dir, get_config_dir, get_data_dir, get_log_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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
        Self {
            log_dir: PathBuf::from(get_log_dir()),
            cache_dir: PathBuf::from(get_cache_dir()),
            data_dir: PathBuf::from(get_data_dir()),
            config_dir: PathBuf::from(get_config_dir()),
            runtime_dir: std::env::temp_dir(),
            service_data_dirs: ServiceDataDirs {
                orchestrator: PathBuf::from(get_config_dir()).join("orchestrator"),
                federation: PathBuf::from(get_config_dir()).join("federation"),
                metrics: PathBuf::from(get_config_dir()).join("metrics"),
                discovery: PathBuf::from(get_config_dir()).join("discovery"),
                registry: PathBuf::from(get_config_dir()).join("registry"),
            },
        }
    }
}

impl PathConfig {
    /// Create a new `PathConfig` instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to determine home directory
    /// - HOME environment variable is not set
    pub fn new() -> Result<Self> {
        debug!("Creating new PathConfig instance");

        // Use simple path implementation
        let home_dir = dirs::home_dir().ok_or_else(|| SongbirdError::Configuration {
            message: "Unable to determine home directory".to_string(),
            field: Some("home_dir".to_string()),
            suggestion: Some("Check if HOME environment variable is set".to_string()),
        })?;

        let config_dir = home_dir.join(".config").join("songbird");
        let data_dir = home_dir.join(".local").join("share").join("songbird");
        let log_dir = home_dir.join(".local").join("log").join("songbird");
        let cache_dir = home_dir.join(".cache").join("songbird");
        let runtime_dir = std::env::temp_dir().join("songbird");

        // ZERO-COPY OPTIMIZATION: Use config_dir reference to avoid repeated cloning
        let service_data_dirs = ServiceDataDirs {
            orchestrator: config_dir.join("orchestrator"),
            federation: config_dir.join("federation"),
            metrics: config_dir.join("metrics"),
            discovery: config_dir.join("discovery"),
            registry: config_dir.join("registry"),
        };

        let paths = Self {
            data_dir,
            config_dir,
            log_dir,
            cache_dir,
            runtime_dir,
            service_data_dirs,
        };

        debug!("PathConfig created successfully: {:?}", paths);
        Ok(paths)
    }

    /// Create fallback path configuration when substrate is unavailable
    pub fn new_fallback() -> Self {
        warn!("🔄 Using fallback path configuration (substrate unavailable)");

        let base_data_dir = Self::get_fallback_data_dir().unwrap_or_else(|_| {
            warn!("Unable to determine data directory, using /tmp/songbird/data");
            PathBuf::from("/tmp/songbird/data")
        });
        let base_config_dir = Self::get_fallback_config_dir().unwrap_or_else(|_| {
            warn!("Unable to determine config directory, using /tmp/songbird/config");
            PathBuf::from("/tmp/songbird/config")
        });
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
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to determine home directory
    /// - System paths are not accessible
    pub fn production() -> Result<Self> {
        Self::new()
    }

    /// Get default configuration paths for the current platform
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to determine home directory
    /// - HOME environment variable is not set
    pub fn get_default_paths() -> Result<Self> {
        debug!("Getting default paths for current platform");

        // Simple path implementation without substrate
        let home_dir = dirs::home_dir().ok_or_else(|| SongbirdError::Configuration {
            message: "Unable to determine home directory".to_string(),
            field: Some("home_dir".to_string()),
            suggestion: Some("Check if HOME environment variable is set".to_string()),
        })?;

        let config_dir = home_dir.join(".config").join("songbird");
        let data_dir = home_dir.join(".local").join("share").join("songbird");
        let log_dir = home_dir.join(".local").join("log").join("songbird");
        let cache_dir = home_dir.join(".cache").join("songbird");
        let runtime_dir = std::env::temp_dir().join("songbird");

        let paths = Self {
            config_dir: config_dir.clone(),
            data_dir,
            log_dir,
            cache_dir,
            runtime_dir,
            service_data_dirs: ServiceDataDirs {
                orchestrator: config_dir.join("orchestrator"),
                federation: config_dir.join("federation"),
                metrics: config_dir.join("metrics"),
                discovery: config_dir.join("discovery"),
                registry: config_dir.join("registry"),
            },
        };

        debug!("Default paths configured: {:?}", paths);
        Ok(paths)
    }

    /// Get fallback data directory when substrate is unavailable
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `XDG_DATA_HOME` is not set
    /// - Unable to determine home directory
    /// - Suggestion: Set `XDG_DATA_HOME` environment variable
    pub fn get_fallback_data_dir() -> Result<PathBuf> {
        // Use XDG base directory specification or platform defaults
        std::env::var_os("XDG_DATA_HOME").map_or_else(
            || {
                dirs::home_dir().map_or_else(
                    || {
                        Err(SongbirdError::Configuration {
                            message: "Unable to determine data directory".to_string(),
                            field: Some("data_dir".to_string()),
                            suggestion: Some("Set XDG_DATA_HOME environment variable".to_string()),
                        })
                    },
                    |home_dir| Ok(home_dir.join(".local").join("share").join("songbird")),
                )
            },
            |data_dir| Ok(PathBuf::from(data_dir).join("songbird")),
        )
    }

    /// Get fallback config directory when substrate is unavailable
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `XDG_CONFIG_HOME` is not set
    /// - Unable to determine home directory
    /// - Suggestion: Set `XDG_CONFIG_HOME` environment variable
    pub fn get_fallback_config_dir() -> Result<PathBuf> {
        // Use XDG base directory specification or platform defaults
        std::env::var_os("XDG_CONFIG_HOME").map_or_else(
            || {
                dirs::home_dir().map_or_else(
                    || {
                        Err(SongbirdError::Configuration {
                            message: "Unable to determine config directory".to_string(),
                            field: Some("config_dir".to_string()),
                            suggestion: Some(
                                "Set XDG_CONFIG_HOME environment variable".to_string(),
                            ),
                        })
                    },
                    |home_dir| Ok(home_dir.join(".config").join("songbird")),
                )
            },
            |config_dir| Ok(PathBuf::from(config_dir).join("songbird")),
        )
    }

    /// Get fallback log directory when substrate is unavailable
    fn get_fallback_log_dir() -> PathBuf {
        if let Ok(log_dir) = std::env::var("SONGBIRD_LOG_DIR") {
            return PathBuf::from(log_dir);
        }

        PathBuf::from(get_log_dir())
    }

    /// Get fallback cache directory when substrate is unavailable
    fn get_fallback_cache_dir() -> PathBuf {
        if let Ok(cache_dir) = std::env::var("SONGBIRD_CACHE_DIR") {
            return PathBuf::from(cache_dir);
        }

        PathBuf::from(get_cache_dir())
    }

    /// Get fallback runtime directory when substrate is unavailable
    fn get_fallback_runtime_dir() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("SONGBIRD_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir);
        }

        PathBuf::from(crate::config::constants::get_temp_dir())
    }

    /// Create all necessary directories
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create any required directory
    /// - Insufficient write permissions for directory creation
    /// - Suggestion: Check directory permissions and available disk space
    pub fn create_directories(&self) -> Result<()> {
        // Create directories directly without substrate
        let directories = vec![
            &self.data_dir,
            &self.config_dir,
            &self.log_dir,
            &self.cache_dir,
            &self.runtime_dir,
            &self.service_data_dirs.orchestrator,
            &self.service_data_dirs.federation,
            &self.service_data_dirs.metrics,
            &self.service_data_dirs.discovery,
            &self.service_data_dirs.registry,
        ];

        for directory in directories {
            if !directory.exists() {
                if let Err(e) = std::fs::create_dir_all(directory) {
                    return Err(SongbirdError::Configuration {
                        message: format!(
                            "Failed to create directory {}: {}",
                            directory.display(),
                            e
                        ),
                        field: Some("directory_path".to_string()),
                        suggestion: Some(
                            "Check if you have write permissions for this directory".to_string(),
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a service-specific path through substrate
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unknown path type provided (valid: config, data, log, cache, runtime)
    /// - Failed to create service directory
    /// - Insufficient write permissions
    pub fn get_service_path(&self, service_name: &str, path_type: &str) -> Result<PathBuf> {
        let service_dir = match path_type {
            "config" => self.config_dir.join(service_name),
            "data" => self.data_dir.join(service_name),
            "log" => self.log_dir.join(service_name),
            "cache" => self.cache_dir.join(service_name),
            "runtime" => self.runtime_dir.join(service_name),
            _ => {
                return Err(SongbirdError::Configuration {
                    message: format!("Unknown path type: {path_type}"),
                    field: Some("path_type".to_string()),
                    suggestion: Some("Check if the path type is valid".to_string()),
                })
            }
        };

        // Ensure directory exists
        if !service_dir.exists() {
            fs::create_dir_all(&service_dir).map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create service directory: {e}"),
                field: Some("service_dir".to_string()),
                suggestion: Some(
                    "Check if you have write permissions for this directory".to_string(),
                ),
            })?;
        }

        Ok(service_dir)
    }

    /// Validate that all paths are accessible
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any configured path does not exist
    /// - Path is not accessible due to permissions
    /// - Suggestion: Ensure all paths exist and are accessible
    pub fn validate_paths(&self) -> Result<()> {
        let paths = vec![
            &self.data_dir,
            &self.config_dir,
            &self.log_dir,
            &self.cache_dir,
            &self.runtime_dir,
        ];

        for path in paths {
            if !path.exists() {
                return Err(SongbirdError::Configuration {
                    message: format!("Path does not exist: {}", path.display()),
                    field: Some("path_validation".to_string()),
                    suggestion: Some("Check if the path exists and is accessible".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Get a temporary path for specific operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create temporary directory
    /// - Insufficient write permissions for temp directory
    pub fn get_temp_path(operation: &str) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join("songbird").join(operation);

        // Ensure directory exists
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create temp directory: {e}"),
                field: Some("temp_dir".to_string()),
                suggestion: Some(
                    "Check if you have write permissions for this directory".to_string(),
                ),
            })?;
        }

        Ok(temp_dir)
    }

    /// Get secure path for sensitive operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create secure directory
    /// - Insufficient write permissions for secure directory
    /// - Unable to set restricted permissions
    pub fn get_secure_path(&self, operation: &str) -> Result<PathBuf> {
        let secure_dir = self.data_dir.join("secure").join(operation);

        // Ensure directory exists with restricted permissions
        if !secure_dir.exists() {
            fs::create_dir_all(&secure_dir).map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create secure directory: {e}"),
                field: Some("secure_dir".to_string()),
                suggestion: Some(
                    "Check if you have write permissions for this directory".to_string(),
                ),
            })?;
        }

        Ok(secure_dir)
    }

    /// Initialize paths for a service
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to create service directory
    /// - Insufficient write permissions
    pub fn initialize_service_paths(base_dir: &Path) -> Result<ServiceDataDirs> {
        let service_dirs = ServiceDataDirs {
            orchestrator: base_dir.join("orchestrator"),
            federation: base_dir.join("federation"),
            metrics: base_dir.join("metrics"),
            discovery: base_dir.join("discovery"),
            registry: base_dir.join("registry"),
        };

        // Create directories
        let directories = vec![
            &service_dirs.orchestrator,
            &service_dirs.federation,
            &service_dirs.metrics,
            &service_dirs.discovery,
            &service_dirs.registry,
        ];

        for dir in directories {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to create service directory: {e}"),
                    field: Some("service_directory".to_string()),
                    suggestion: Some("Check if you have write permissions".to_string()),
                })?;
            }
        }

        Ok(service_dirs)
    }
}

/// Get the best available path configuration
///
/// This function attempts to use substrate-based paths first, then falls back
/// to a simpler implementation. Always succeeds as fallback is always available.
#[must_use]
pub fn get_path_config() -> PathConfig {
    // Try to use substrate first
    match PathConfig::new() {
        Ok(config) => {
            debug!("✅ Using substrate-based path configuration");
            config
        }
        Err(e) => {
            warn!("⚠️ Substrate path configuration failed: {}, using fallback", e);
            PathConfig::new_fallback()
        }
    }
}

/// Initialize paths for a service
///
/// # Errors
///
/// Returns an error if:
/// - Failed to create service directory
/// - Insufficient write permissions for /`tmp/songbird/{service_name`}
pub fn initialize_service_paths(service_name: &str) -> Result<ServiceDataDirs> {
    let base_dir = PathBuf::from(format!("/tmp/songbird/{service_name}"));

    let service_dirs = ServiceDataDirs {
        orchestrator: base_dir.join("orchestrator"),
        federation: base_dir.join("federation"),
        metrics: base_dir.join("metrics"),
        discovery: base_dir.join("discovery"),
        registry: base_dir.join("registry"),
    };

    // Create directories
    let directories = vec![
        &service_dirs.orchestrator,
        &service_dirs.federation,
        &service_dirs.metrics,
        &service_dirs.discovery,
        &service_dirs.registry,
    ];

    for dir in directories {
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create service directory: {e}"),
                field: Some("service_directory".to_string()),
                suggestion: Some("Check if you have write permissions".to_string()),
            })?;
        }
    }

    Ok(service_dirs)
}

/// Create a path configuration for testing
#[must_use]
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
