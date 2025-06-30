// Configuration providers
//! Configuration provider implementations
//!
//! Concrete implementations of the ConfigProvider trait for various sources.

use crate::errors::{Result, SongbirdError};
use crate::traits::config::{ConfigFormat, ConfigProvider, ConfigProviderInfo};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::path::PathBuf;
/// File-based configuration provider
pub struct FileConfigProvider<T> {
    path: PathBuf,
    format: ConfigFormat,
    _phantom: PhantomData<T>,
}
impl<T> FileConfigProvider<T> {
    pub fn new(path: PathBuf, format: ConfigFormat) -> Self {
        Self {
            path,
            format,
            _phantom: PhantomData,
        }
    }
    /// Get the configuration file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    /// Get the configuration format
    pub fn format(&self) -> &ConfigFormat {
        &self.format
    }
    /// Load configuration from file
    pub async fn load(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if !self.path.exists() {
            return Err(SongbirdError::Config {
                field: Some("config_file_path".to_string()),
                message: format!("Configuration file does not exist: {}", self.path.display()),
            });
        }
        let contents =
            tokio::fs::read_to_string(&self.path)
                .await
                .map_err(|e| SongbirdError::Config {
                    field: Some("config_file_read".to_string()),
                    message: format!("Failed to read configuration file: {}", e),
                })?;
        let config: T = if self.format == ConfigFormat::Toml {
            toml::from_str(&contents).map_err(|e| SongbirdError::Config {
                field: Some("toml_parsing".to_string()),
                message: format!("Failed to parse TOML configuration: {}", e),
            })?
        } else {
            serde_json::from_str(&contents).map_err(|e| SongbirdError::Config {
                field: Some("json_parsing".to_string()),
                message: format!("Failed to parse JSON configuration: {}", e),
            })?
        };
        Ok(config)
    }
}
#[async_trait]
impl<T> ConfigProvider<T> for FileConfigProvider<T>
where
    T: Clone + Send + Sync + for<'de> serde::Deserialize<'de>,
{
    async fn load_config(&self) -> Result<T> {
        // Read configuration from file
        let contents =
            tokio::fs::read_to_string(&self.path)
                .await
                .map_err(|e| SongbirdError::Config {
                    field: Some("file_path".to_string()),
                    message: format!(
                        "Failed to read config file '{}': {}",
                        self.path.display(),
                        e
                    ),
                })?;
        // Parse based on file extension
        let config = match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => toml::from_str(&contents).map_err(|e| SongbirdError::Config {
                field: Some("toml_parsing".to_string()),
                message: format!("Failed to parse TOML config: {}", e),
            })?,
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&contents).map_err(|e| SongbirdError::Config {
                    field: Some("yaml_parsing".to_string()),
                    message: format!("Failed to parse YAML config: {}", e),
                })?
            }
            Some("json") => serde_json::from_str(&contents).map_err(|e| SongbirdError::Config {
                field: Some("json_parsing".to_string()),
                message: format!("Failed to parse JSON config: {}", e),
            })?,
            _ => {
                return Err(SongbirdError::Config {
                    field: Some("file_extension".to_string()),
                    message: format!(
                        "Unsupported config file format for '{}'. Use .toml, .yaml, .yml, or .json",
                        self.path.display()
                    ),
                });
            }
        };
        Ok(config)
    }
    async fn reload_config(&self) -> Result<T> {
        self.load_config().await
    }
    async fn watch_config(&self) -> impl futures_util::Stream<Item = Result<T>> {
        futures_util::stream::empty()
    }
    async fn validate_config(&self, _config: &T) -> Result<()> {
        Ok(())
    }
    fn provider_info(&self) -> ConfigProviderInfo {
        ConfigProviderInfo {
            name: "File Config Provider".to_string(),
            version: "1.0.0".to_string(),
            provider_type: "file".to_string(),
            description: "Loads configuration from files".to_string(),
            supports_watch: false,
            supports_reload: true,
        }
    }
}
