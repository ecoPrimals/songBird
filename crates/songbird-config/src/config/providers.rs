//! Configuration providers for Songbird components
//!
//! This module provides configuration provider functionality.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use songbird_errors::{SongbirdError, SongbirdResult};
type Result<T> = SongbirdResult<T>;
use std::marker::PhantomData;
use std::path::PathBuf;

/// Configuration format types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider<T>: Send + Sync {
    async fn load(&self) -> Result<T>;
    async fn save(&self, config: &T) -> Result<()>;
    fn provider_info(&self) -> ConfigProviderInfo;
}

/// Configuration provider information
#[derive(Debug, Clone)]
pub struct ConfigProviderInfo {
    pub name: String,
    pub description: String,
    pub format: ConfigFormat,
}

/// File-based configuration provider
pub struct FileConfigProvider<T> {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub _phantom: PhantomData<T>,
}

impl<T> FileConfigProvider<T> {
    pub fn new(path: PathBuf, format: ConfigFormat) -> Self {
        Self {
            path,
            format,
            _phantom: PhantomData,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn format(&self) -> &ConfigFormat {
        &self.format
    }
}

#[async_trait]
impl<T> ConfigProvider<T> for FileConfigProvider<T>
where
    T: DeserializeOwned + serde::Serialize + Send + Sync,
{
    async fn load(&self) -> Result<T> {
        let content = tokio::fs::read_to_string(&self.path).await.map_err(|e| {
            SongbirdError::Configuration {
                message: format!("Failed to read config file: {e}"),
                field: Some("config_file".to_string()),
                suggestion: Some("Check if the file exists and is readable".to_string()),
            }
        })?;

        let config: T = match &self.format {
            ConfigFormat::Toml => {
                toml::from_str(&content).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to parse TOML config: {e}"),
                    field: Some("config_parsing".to_string()),
                    suggestion: Some("Check TOML syntax".to_string()),
                })?
            }
            ConfigFormat::Json => {
                serde_json::from_str(&content).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to parse JSON config: {e}"),
                    field: Some("config_parsing".to_string()),
                    suggestion: Some("Check JSON syntax".to_string()),
                })?
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(&content).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to parse YAML config: {e}"),
                    field: Some("config_parsing".to_string()),
                    suggestion: Some("Check YAML syntax".to_string()),
                })?
            }
        };

        Ok(config)
    }

    async fn save(&self, config: &T) -> Result<()> {
        let content = match &self.format {
            ConfigFormat::Toml => {
                toml::to_string_pretty(config).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to serialize config to TOML: {e}"),
                    field: Some("config_serialization".to_string()),
                    suggestion: Some("Check if the config structure is valid".to_string()),
                })?
            }
            ConfigFormat::Json => {
                serde_json::to_string_pretty(config).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to serialize config to JSON: {e}"),
                    field: Some("config_serialization".to_string()),
                    suggestion: Some("Check if the config structure is valid".to_string()),
                })?
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(config).map_err(|e| SongbirdError::Configuration {
                    message: format!("Failed to serialize config to YAML: {e}"),
                    field: Some("config_serialization".to_string()),
                    suggestion: Some("Check if the config structure is valid".to_string()),
                })?
            }
        };

        tokio::fs::write(&self.path, content)
            .await
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to write config file: {e}"),
                field: Some("config_file".to_string()),
                suggestion: Some("Check if you have write permissions for this file".to_string()),
            })?;

        Ok(())
    }

    fn provider_info(&self) -> ConfigProviderInfo {
        ConfigProviderInfo {
            name: "FileConfigProvider".to_string(),
            description: format!(
                "File-based configuration provider for {}",
                self.path.display()
            ),
            format: self.format.clone(),
        }
    }
}
