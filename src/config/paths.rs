//! Platform-Agnostic Paths Module
//!
//! Provides OS-appropriate default paths without hardcoding platform-specific directories
//! Supports configuration override via environment variables

use crate::errors::{Result, SongbirdError};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Platform-agnostic path configuration
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
        Self::new()
    }
}

impl PathConfig {
    /// Create new path configuration with OS-appropriate defaults
    pub fn new() -> Self {
        let base_data_dir = Self::get_default_data_dir();
        let base_config_dir = Self::get_default_config_dir();
        let base_log_dir = Self::get_default_log_dir();
        let base_cache_dir = Self::get_default_cache_dir();
        let base_runtime_dir = Self::get_default_runtime_dir();

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
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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

    /// Create production configuration (uses system directories)
    pub fn production() -> Self {
        Self::new()
    }

    /// Get OS-appropriate default data directory
    fn get_default_data_dir() -> PathBuf {
        // Check environment variable first
        if let Ok(data_dir) = env::var("SONGBIRD_DATA_DIR") {
            return PathBuf::from(data_dir);
        }

        match std::env::consts::OS {
            "windows" => dirs::data_local_dir()
                .or_else(dirs::data_dir)
                .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
                .join("Songbird"),
            "macos" => {
                if Self::is_system_install() {
                    PathBuf::from("/usr/local/var/songbird")
                } else {
                    dirs::data_dir()
                        .unwrap_or_else(|| PathBuf::from("/usr/local/var"))
                        .join("Songbird")
                }
            }
            _ => {
                if Self::is_system_install() {
                    PathBuf::from("/var/lib/songbird")
                } else {
                    dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
                        .join("songbird")
                }
            }
        }
    }

    /// Get OS-appropriate default configuration directory
    fn get_default_config_dir() -> PathBuf {
        if let Ok(config_dir) = env::var("SONGBIRD_CONFIG_DIR") {
            return PathBuf::from(config_dir);
        }

        match std::env::consts::OS {
            "windows" => dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
                .join("Songbird"),
            "macos" => {
                if Self::is_system_install() {
                    PathBuf::from("/usr/local/etc/songbird")
                } else {
                    dirs::config_dir()
                        .unwrap_or_else(|| PathBuf::from("/usr/local/etc"))
                        .join("Songbird")
                }
            }
            _ => {
                if Self::is_system_install() {
                    PathBuf::from("/etc/songbird")
                } else {
                    dirs::config_dir()
                        .unwrap_or_else(|| PathBuf::from("~/.config"))
                        .join("songbird")
                }
            }
        }
    }

    /// Get OS-appropriate default log directory
    fn get_default_log_dir() -> PathBuf {
        if let Ok(log_dir) = env::var("SONGBIRD_LOG_DIR") {
            return PathBuf::from(log_dir);
        }

        match std::env::consts::OS {
            "windows" => dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
                .join("Songbird")
                .join("Logs"),
            "macos" => {
                if Self::is_system_install() {
                    PathBuf::from("/usr/local/var/log/songbird")
                } else {
                    dirs::data_dir()
                        .unwrap_or_else(|| PathBuf::from("/usr/local/var"))
                        .join("Songbird")
                        .join("Logs")
                }
            }
            _ => {
                if Self::is_system_install() {
                    PathBuf::from("/var/log/songbird")
                } else {
                    dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
                        .join("songbird")
                        .join("logs")
                }
            }
        }
    }

    /// Get OS-appropriate default cache directory
    fn get_default_cache_dir() -> PathBuf {
        if let Ok(cache_dir) = env::var("SONGBIRD_CACHE_DIR") {
            return PathBuf::from(cache_dir);
        }

        match std::env::consts::OS {
            "windows" => dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Local"))
                .join("Songbird"),
            "macos" => dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/var/cache"))
                .join("Songbird"),
            _ => dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("songbird"),
        }
    }

    /// Get OS-appropriate default runtime directory
    fn get_default_runtime_dir() -> PathBuf {
        if let Ok(runtime_dir) = env::var("SONGBIRD_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir);
        }

        match std::env::consts::OS {
            "windows" => dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
                .join("Songbird")
                .join("Runtime"),
            "macos" => {
                if Self::is_system_install() {
                    PathBuf::from("/usr/local/var/run/songbird")
                } else {
                    dirs::runtime_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join("songbird")
                }
            }
            _ => {
                if Self::is_system_install() {
                    PathBuf::from("/run/songbird")
                } else {
                    dirs::runtime_dir()
                        .unwrap_or_else(|| PathBuf::from("/tmp"))
                        .join("songbird")
                }
            }
        }
    }

    /// Check if this is a system-wide installation
    fn is_system_install() -> bool {
        env::var("SONGBIRD_SYSTEM_INSTALL").is_ok() || Self::is_running_as_privileged_user()
    }

    /// Check if running as privileged user (root on Unix, admin on Windows)
    fn is_running_as_privileged_user() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::getuid() == 0 }
        }
        #[cfg(windows)]
        {
            false
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    /// Ensure all directories exist
    pub async fn ensure_directories_exist(&self) -> Result<()> {
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

        for dir in directories {
            if let Err(e) = tokio::fs::create_dir_all(dir).await {
                return Err(SongbirdError::Io { message: format!(
                    "Failed to create directory {}: {}",
                    dir.display(),
                    e
                ) });
            }
        }

        Ok(())
    }

    /// Get configuration file path
    pub fn config_file_path(&self, filename: &str) -> PathBuf {
        self.config_dir.join(filename)
    }

    /// Get log file path
    pub fn log_file_path(&self, service: &str) -> PathBuf {
        self.log_dir.join(format!("{}.log", service))
    }

    /// Get PID file path
    pub fn pid_file_path(&self, service: &str) -> PathBuf {
        self.runtime_dir.join(format!("{}.pid", service))
    }

    /// Get socket file path (Unix only)
    #[cfg(unix)]
    pub fn socket_file_path(&self, service: &str) -> PathBuf {
        self.runtime_dir.join(format!("{}.sock", service))
    }

    /// Validate that all paths are accessible
    pub async fn validate(&self) -> Result<()> {
        for dir in [&self.data_dir, &self.config_dir, &self.log_dir].iter() {
            if let Some(parent) = dir.parent() {
                if !parent.exists() {
                    return Err(SongbirdError::Config {
                        field: Some("paths".to_string()),
                        message: format!("Parent directory {} does not exist", parent.display()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a summary of all configured paths
    pub fn summary(&self) -> String {
        format!(
            "Data: {}, Config: {}, Logs: {}, Cache: {}, Runtime: {}",
            self.data_dir.display(),
            self.config_dir.display(),
            self.log_dir.display(),
            self.cache_dir.display(),
            self.runtime_dir.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_paths() {
        let config = PathConfig::development();
        assert!(config.data_dir.to_string_lossy().contains(".songbird"));
        assert!(config.config_dir.to_string_lossy().contains(".songbird"));
    }

    #[test]
    fn test_path_utilities() {
        let config = PathConfig::development();
        let config_path = config.config_file_path("test.yaml");
        assert!(config_path.to_string_lossy().ends_with("test.yaml"));

        let log_path = config.log_file_path("orchestrator");
        assert!(log_path.to_string_lossy().ends_with("orchestrator.log"));
    }
}
