//! Platform-Agnostic Paths Module
//!
//! Provides OS-appropriate default paths without hardcoding platform-specific directories
//! Supports configuration override via environment variables

use serde::{Deserialize, Serialize};
use songbird_errors::{Result, SongbirdError};
use std::path::PathBuf;

/// Platform-agnostic path configuration that adapts to environment
///
/// Supports configuration override via environment variables
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Deserialize)]
pub struct PathConfig {
    /// Primary data storage directory
    pub data_dir: PathBuf,
    /// Configuration files directory
    pub config_dir: PathBuf,
    /// Log files directory
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
    /// Create new `PathConfig` with platform-appropriate defaults
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_dir: crate::config::environment::default_data_dir().into(),
            config_dir: crate::config::environment::default_config_dir().into(),
            log_dir: crate::config::environment::default_log_dir().into(),
            cache_dir: crate::config::environment::default_cache_dir().into(),
            runtime_dir: crate::config::environment::default_runtime_dir().into(),
            service_data_dirs: ServiceDataDirs {
                orchestrator: crate::config::environment::default_data_dir().into(),
                federation: crate::config::environment::default_data_dir().into(),
                metrics: crate::config::environment::default_data_dir().into(),
                discovery: crate::config::environment::default_data_dir().into(),
                registry: crate::config::environment::default_data_dir().into(),
            },
        }
    }

    /// Create development-focused path configuration
    #[must_use]
    pub fn development() -> Self {
        Self {
            data_dir: "./.songbird/data".into(),
            config_dir: "./.songbird/config".into(),
            log_dir: "./.songbird/logs".into(),
            cache_dir: "./.songbird/cache".into(),
            runtime_dir: "./.songbird/runtime".into(),
            service_data_dirs: ServiceDataDirs {
                orchestrator: "./.songbird/data".into(),
                federation: "./.songbird/data".into(),
                metrics: "./.songbird/data".into(),
                discovery: "./.songbird/data".into(),
                registry: "./.songbird/data".into(),
            },
        }
    }

    /// Create production-focused path configuration
    #[must_use]
    pub fn production() -> Self {
        Self {
            data_dir: "/var/lib/songbird".into(),
            config_dir: "/etc/songbird".into(),
            log_dir: "/var/log/songbird".into(),
            cache_dir: "/var/cache/songbird".into(),
            runtime_dir: "/run/songbird".into(),
            service_data_dirs: ServiceDataDirs {
                orchestrator: "/var/lib/songbird".into(),
                federation: "/var/lib/songbird".into(),
                metrics: "/var/lib/songbird".into(),
                discovery: "/var/lib/songbird".into(),
                registry: "/var/lib/songbird".into(),
            },
        }
    }

    /// Ensure all directories exist
    ///
    /// # Errors
    ///
    /// Returns an error if any directory cannot be created due to permissions or filesystem issues
    pub fn ensure_directories_exist(&self) -> Result<()> {
        for dir in &[&self.data_dir, &self.config_dir, &self.log_dir] {
            if let Err(e) = std::fs::create_dir_all(dir) {
                return Err(songbird_errors::SongbirdError::Config {
                    field: Some("directory_creation".to_string()),
                    message: format!("Failed to create directory {}: {}", dir.display(), e),
                    context: Some("Directory creation validation".to_string()),
                    suggestion: Some(
                        "Check directory permissions and available disk space".to_string(),
                    ),
                });
            }
        }
        Ok(())
    }

    /// Get configuration file path
    #[must_use]
    pub fn config_file_path(&self, filename: &str) -> PathBuf {
        self.config_dir.join(filename)
    }

    /// Get log file path
    #[must_use]
    pub fn log_file_path(&self, service: &str) -> PathBuf {
        self.log_dir.join(format!("{service}.log"))
    }

    /// Get PID file path
    #[must_use]
    pub fn pid_file_path(&self, service: &str) -> PathBuf {
        self.runtime_dir.join(format!("{service}.pid"))
    }

    /// Get socket file path (Unix only)
    #[cfg(unix)]
    #[must_use]
    pub fn socket_file_path(&self, service: &str) -> PathBuf {
        self.runtime_dir.join(format!("{service}.sock"))
    }

    /// Validate that all paths are accessible
    ///
    /// # Errors
    ///
    /// Returns an error if any path validation fails
    pub fn validate(&self) -> Result<()> {
        for dir in &[&self.data_dir, &self.config_dir, &self.log_dir] {
            if let Some(parent) = dir.parent() {
                if !parent.exists() {
                    return Err(SongbirdError::Config {
                        field: Some("paths".to_string()),
                        message: format!("Parent directory {} does not exist", parent.display()),
                        context: Some("Path validation".to_string()),
                        suggestion: Some(
                            "Ensure parent directories exist or create them first".to_string(),
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a summary of all configured paths
    #[must_use]
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
