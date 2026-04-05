// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration providers for Songbird components
//!
//! This module provides configuration provider functionality.

#![allow(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]
#![allow(missing_docs, reason = "generic provider trait; document at call sites")]

use serde::de::DeserializeOwned;
use songbird_types::{SongbirdError, SongbirdResult};
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
    #[must_use]
    pub const fn new(path: PathBuf, format: ConfigFormat) -> Self {
        Self {
            path,
            format,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    #[must_use]
    pub const fn format(&self) -> &ConfigFormat {
        &self.format
    }
}

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

        tokio::fs::write(&self.path, content).await.map_err(|e| SongbirdError::Configuration {
            message: format!("Failed to write config file: {e}"),
            field: Some("config_file".to_string()),
            suggestion: Some("Check if you have write permissions for this file".to_string()),
        })?;

        Ok(())
    }

    fn provider_info(&self) -> ConfigProviderInfo {
        ConfigProviderInfo {
            name: "FileConfigProvider".to_string(),
            description: format!("File-based configuration provider for {}", self.path.display()),
            format: self.format.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SampleCfg {
        name: String,
        count: u32,
    }

    #[test]
    fn config_format_variants_distinct() {
        assert_ne!(ConfigFormat::Toml, ConfigFormat::Json);
        assert_ne!(ConfigFormat::Json, ConfigFormat::Yaml);
    }

    #[tokio::test]
    async fn file_provider_roundtrip_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cfg.toml");
        let provider = FileConfigProvider::<SampleCfg>::new(path.clone(), ConfigFormat::Toml);
        let original = SampleCfg {
            name: "songbird".to_string(),
            count: 42,
        };
        provider.save(&original).await.expect("save");
        let loaded: SampleCfg = provider.load().await.expect("load");
        assert_eq!(loaded, original);
        let info = provider.provider_info();
        assert_eq!(info.format, ConfigFormat::Toml);
        assert!(info.description.contains(path.to_string_lossy().as_ref()));
    }

    #[tokio::test]
    async fn file_provider_roundtrip_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cfg.json");
        let provider = FileConfigProvider::<SampleCfg>::new(path, ConfigFormat::Json);
        let original = SampleCfg {
            name: "j".to_string(),
            count: 1,
        };
        provider.save(&original).await.expect("save");
        let loaded: SampleCfg = provider.load().await.expect("load");
        assert_eq!(loaded, original);
    }

    #[tokio::test]
    async fn file_provider_roundtrip_yaml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cfg.yaml");
        let provider = FileConfigProvider::<SampleCfg>::new(path, ConfigFormat::Yaml);
        let original = SampleCfg {
            name: "y".to_string(),
            count: 7,
        };
        provider.save(&original).await.expect("save");
        let loaded: SampleCfg = provider.load().await.expect("load");
        assert_eq!(loaded, original);
    }

    #[tokio::test]
    async fn file_provider_load_missing_file_errors() {
        let path = std::path::PathBuf::from("/nonexistent/songbird_cfg_provider_test_xyz");
        let provider = FileConfigProvider::<SampleCfg>::new(path, ConfigFormat::Json);
        let err = provider.load().await.expect_err("missing file");
        assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
    }

    #[test]
    fn file_provider_accessors() {
        let path = std::path::PathBuf::from("/tmp/x.toml");
        let p = FileConfigProvider::<()>::new(path.clone(), ConfigFormat::Toml);
        assert_eq!(p.path(), &path);
        assert_eq!(*p.format(), ConfigFormat::Toml);
    }
}
