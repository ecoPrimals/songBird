//! Storage /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalStorageConfig {
    /// Enable storage features
    /// Enabled field
    pub enabled: bool,
    /// Storage backend;
    /// Backend field
    pub backend: String,
}

impl Default for CanonicalStorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "memory".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SongbirdError;

    #[test]
    fn test_default_storage_config() {
        let config = CanonicalStorageConfig::default();
        assert!(config.enabled);
        assert_eq!(config.backend, "memory");
    }

    #[test]
    fn test_custom_storage_config() {
        let config = CanonicalStorageConfig {
            enabled: false,
            backend: "postgres".to_string(),
        };
        assert!(!config.enabled);
        assert_eq!(config.backend, "postgres");
    }

    #[test]
    fn test_storage_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalStorageConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        assert!(json.contains("enabled"));
        assert!(json.contains("backend"));
        assert!(json.contains("memory"));
        Ok(())
    }

    #[test]
    fn test_storage_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"enabled":true,"backend":"redis"}"#;
        let config: CanonicalStorageConfig =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert!(config.enabled);
        assert_eq!(config.backend, "redis");
        Ok(())
    }

    #[test]
    fn test_storage_config_clone() {
        let config1 = CanonicalStorageConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.backend, config2.backend);
    }

    #[test]
    fn test_storage_config_debug() {
        let config = CanonicalStorageConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalStorageConfig"));
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("backend"));
    }
}
