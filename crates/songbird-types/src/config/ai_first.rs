//! AI-First Citizen API /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};

/// **CANONICAL**: AI-First Citizen API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CanonicalAIFirstConfig {
    /// Enable AI-First API features
    /// Enabled field
    pub enabled: bool,
    /// Structured error context for automation
    pub structured_errors: bool,
    /// Enable capability discovery
    pub capability_discovery: bool,
    /// Comprehensive observability
    pub observability: bool,
}

impl Default for CanonicalAIFirstConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            structured_errors: true,
            capability_discovery: true,
            observability: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SongbirdError;

    #[test]
    fn test_default_ai_first_config() {
        let config = CanonicalAIFirstConfig::default();
        assert!(config.enabled);
        assert!(config.structured_errors);
        assert!(config.capability_discovery);
        assert!(config.observability);
    }

    #[test]
    fn test_custom_ai_first_config() {
        let config = CanonicalAIFirstConfig {
            enabled: false,
            structured_errors: false,
            capability_discovery: false,
            observability: false,
        };
        assert!(!config.enabled);
        assert!(!config.structured_errors);
        assert!(!config.capability_discovery);
        assert!(!config.observability);
    }

    #[test]
    fn test_partial_ai_first_config() {
        let config = CanonicalAIFirstConfig {
            enabled: true,
            structured_errors: true,
            capability_discovery: false,
            observability: true,
        };
        assert!(config.enabled);
        assert!(config.structured_errors);
        assert!(!config.capability_discovery);
        assert!(config.observability);
    }

    #[test]
    fn test_ai_first_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalAIFirstConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        assert!(json.contains("enabled"));
        assert!(json.contains("structured_errors"));
        assert!(json.contains("capability_discovery"));
        assert!(json.contains("observability"));
        Ok(())
    }

    #[test]
    fn test_ai_first_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"enabled":false,"structured_errors":true,"capability_discovery":false,"observability":true}"#;
        let config: CanonicalAIFirstConfig =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert!(!config.enabled);
        assert!(config.structured_errors);
        assert!(!config.capability_discovery);
        assert!(config.observability);
        Ok(())
    }

    #[test]
    fn test_ai_first_config_clone() {
        let config1 = CanonicalAIFirstConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.structured_errors, config2.structured_errors);
        assert_eq!(config1.capability_discovery, config2.capability_discovery);
        assert_eq!(config1.observability, config2.observability);
    }

    #[test]
    fn test_ai_first_config_debug() {
        let config = CanonicalAIFirstConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalAIFirstConfig"));
        assert!(debug_str.contains("enabled"));
    }
}
