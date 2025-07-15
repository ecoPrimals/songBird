//! Configuration providers for different sources and formats

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use songbird_errors::{Result, SongbirdError};
use std::path::Path;

/// Configuration format types
#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider<T: DeserializeOwned + Send + Sync> {
    async fn load_config(&self) -> Result<T>;
    async fn watch_config(&self) -> impl futures_util::Stream<Item = Result<T>> {
        futures_util::stream::empty()
    }
}

/// Provider information
#[derive(Debug)]
pub struct ConfigProviderInfo {
    pub name: String,
    pub description: String,
    pub format: ConfigFormat,
}

/// File-based configuration provider
pub struct FileConfigProvider {
    path: std::path::PathBuf,
    format: ConfigFormat,
}

impl FileConfigProvider {
    pub fn new<P: AsRef<Path>>(path: P, format: ConfigFormat) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            format,
        }
    }
}

#[async_trait]
impl<T: DeserializeOwned + Send + Sync> ConfigProvider<T> for FileConfigProvider {
    async fn load_config(&self) -> Result<T> {
        let contents =
            tokio::fs::read_to_string(&self.path)
                .await
                .map_err(|e| SongbirdError::Config {
                    field: Some("file_path".to_string()),
                    message: format!("Failed to read config file {:?}: {}", self.path, e),
                    context: Some("File reading".to_string()),
                    suggestion: Some("Check file path and permissions".to_string()),
                })?;

        match self.format {
            ConfigFormat::Toml => {
                toml::from_str(&contents).map_err(|e| songbird_errors::SongbirdError::Config {
                    field: Some("toml_parse".to_string()),
                    message: format!("Failed to parse TOML config: {e}"),
                    context: Some("TOML parsing".to_string()),
                    suggestion: Some("Check TOML syntax and format".to_string()),
                })
            }
            ConfigFormat::Yaml => serde_yaml::from_str(&contents).map_err(|e| {
                songbird_errors::SongbirdError::Config {
                    field: Some("yaml_parse".to_string()),
                    message: format!("Failed to parse YAML config: {e}"),
                    context: Some("YAML parsing".to_string()),
                    suggestion: Some("Check YAML syntax and format".to_string()),
                }
            }),
            ConfigFormat::Json => serde_json::from_str(&contents).map_err(|e| {
                songbird_errors::SongbirdError::Config {
                    field: Some("json_parse".to_string()),
                    message: format!("Failed to parse JSON config: {e}"),
                    context: Some("JSON parsing".to_string()),
                    suggestion: Some("Check JSON syntax and format".to_string()),
                }
            }),
        }
    }
}

/// Environment-based configuration provider  
pub struct EnvConfigProvider;

#[async_trait]
impl<T: DeserializeOwned + Send + Sync> ConfigProvider<T> for EnvConfigProvider {
    async fn load_config(&self) -> Result<T> {
        // Environment config loaded from env vars - no file reading needed
        let config = std::env::var("SONGBIRD_CONFIG").unwrap_or_default();
        serde_json::from_str(&config).map_err(|e| songbird_errors::SongbirdError::Config {
            field: Some("env_parse".to_string()),
            message: format!("Failed to parse environment config: {e}"),
            context: Some("Environment parsing".to_string()),
            suggestion: Some("Check environment variables".to_string()),
        })
    }
}
