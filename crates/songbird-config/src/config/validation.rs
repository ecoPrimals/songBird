// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration validation with comprehensive checks
//!
//! This module provides validation for the universal primal configuration system.

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use tracing::debug;

use crate::config::SongbirdConfig;
// Do not `use songbird_config` here (circular import within this crate).

/// Configuration validation results with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Critical errors that prevent system startup
    pub errors: Vec<ValidationError>,
    /// Non-critical warnings that should be addressed
    pub warnings: Vec<ValidationWarning>,
    /// Helpful suggestions for optimization
    pub recommendations: Vec<String>,
    /// Overall validation status
    pub is_valid: bool,
}

/// Configuration validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Field path where error occurred
    pub field: String,
    /// Human-readable error message
    pub message: String,
    /// Current value that caused the error
    pub current_value: Option<String>,
    /// Expected value or format
    pub expected_value: Option<String>,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggestion for fixing the error
    pub suggestion: String,
}

/// Configuration validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Field path where warning occurred
    pub field: String,
    /// Human-readable warning message
    pub message: String,
    /// Current value that caused the warning
    pub current_value: Option<String>,
    /// Recommended value or format
    pub recommended_value: Option<String>,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggestion for improvement
    pub suggestion: String,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl ValidationResult {
    /// Create a new validation result
    #[must_use]
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            is_valid: true,
        }
    }

    /// Check if validation passed (no critical errors)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total issue count
    #[must_use]
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration validation implementation
impl SongbirdConfig {
    /// Validate the entire configuration
    #[allow(
        deprecated,
        reason = "`SongbirdConfig::primal_registry` remains the validated surface until migration completes"
    )]
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Universal primal validation (replaces hardcoded primal validation)
        if let Some(registry) = &self.primal_registry {
            for (primal_name, primal_config) in &registry.primals {
                if primal_config.enabled {
                    debug!("Validating universal primal: {}", primal_name);
                    Self::validate_universal_primal_config(primal_name, primal_config, &mut result);
                }
            }
        }

        // Basic network validation
        self.validate_network_basic(&mut result);

        // Security validation
        self.validate_security_basic(&mut result);

        // Set overall validation status
        result.is_valid = result.errors.is_empty();

        Ok(result)
    }

    /// Universal primal configuration validation
    fn validate_universal_primal_config(
        primal_name: &str,
        primal_config: &crate::canonical::primals::PrimalConfiguration,
        result: &mut ValidationResult,
    ) {
        // Validate primal endpoint
        if primal_config.endpoint.primary_url.is_empty() {
            result.errors.push(ValidationError {
                field: format!("primal_registry.{primal_name}.endpoint.primary_url"),
                message: String::from("Primal endpoint URL cannot be empty"),
                current_value: Some(String::from("empty")),
                expected_value: Some(String::from("Valid URL (http:// or https://)")),
                severity: ValidationSeverity::Critical,
                suggestion: format!("Set endpoint URL for primal '{primal_name}'"),
            });
        }

        // Validate primal capabilities
        if primal_config.capabilities.is_empty() {
            result.warnings.push(ValidationWarning {
                field: format!("primal_registry.{primal_name}.capabilities"),
                message: String::from("Primal has no declared capabilities"),
                current_value: Some(String::from("empty")),
                recommended_value: Some(String::from("At least one capability")),
                severity: ValidationSeverity::Medium,
                suggestion: format!(
                    "Add capabilities for primal '{primal_name}' to enable capability-based routing"
                ),
            });
        }

        // Validate connection settings
        if primal_config.connection_settings.connection_timeout.as_secs() == 0 {
            result.errors.push(ValidationError {
                field: format!(
                    "primal_registry.{primal_name}.connection_settings.connection_timeout"
                ),
                message: String::from("Connection timeout cannot be zero"),
                current_value: Some(String::from("0")),
                expected_value: Some(String::from("Positive duration (e.g., 30s)")),
                severity: ValidationSeverity::High,
                suggestion: format!("Set a positive connection timeout for primal '{primal_name}'"),
            });
        }
    }

    /// Basic network configuration validation
    fn validate_network_basic(&self, result: &mut ValidationResult) {
        // Validate bind address is not empty
        if self.network.bind_address.is_empty() {
            result.errors.push(ValidationError {
                field: String::from("network.bind_address"),
                message: String::from("Network bind address cannot be empty"),
                current_value: Some(String::from("empty")),
                expected_value: Some(String::from("Valid IP address")),
                severity: ValidationSeverity::Critical,
                suggestion: String::from("Set a valid bind address (e.g., &crate::constants::network::DEFAULT_HOST or '0.0.0.0')"),
            });
        }

        // Validate port range
        if self.network.port_range.start >= self.network.port_range.end {
            result.errors.push(ValidationError {
                field: String::from("network.port_range"),
                message: String::from("Port range start must be less than end"),
                current_value: Some(format!(
                    "{}-{}",
                    self.network.port_range.start, self.network.port_range.end
                )),
                expected_value: Some(String::from("start < end")),
                severity: ValidationSeverity::High,
                suggestion: String::from("Ensure port range start is less than port range end"),
            });
        }
    }

    /// Basic security configuration validation
    fn validate_security_basic(&self, result: &mut ValidationResult) {
        if self.security.enabled {
            if !self.security.authentication.enabled {
                result.warnings.push(ValidationWarning {
                    field: String::from("security.authentication.enabled"),
                    message: String::from("Security is enabled but authentication is disabled"),
                    current_value: Some(String::from("false")),
                    recommended_value: Some(String::from("true")),
                    severity: ValidationSeverity::Medium,
                    suggestion: String::from("Enable authentication when security is enabled"),
                });
            }

            // Validate encryption settings
            if !self.security.encryption.at_rest && !self.security.encryption.in_transit {
                result.warnings.push(ValidationWarning {
                    field: String::from("security.encryption"),
                    message: String::from("Security enabled but no encryption configured"),
                    current_value: Some(String::from("no encryption")),
                    recommended_value: Some(String::from(
                        "at_rest or in_transit encryption enabled",
                    )),
                    severity: ValidationSeverity::Medium,
                    suggestion: String::from(
                        "Enable at_rest or in_transit encryption for security",
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SongbirdConfig;
    use std::time::Duration;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    #[test]
    fn validation_result_roundtrip_and_default() {
        let r = ValidationResult::default();
        assert!(r.errors.is_empty());
        assert_json_roundtrip(&r);
    }

    #[test]
    fn validation_error_and_warning_roundtrip() {
        let err = ValidationError {
            field: String::from("network.bind_address"),
            message: String::from("empty"),
            current_value: Some(String::new()),
            expected_value: Some(String::from("0.0.0.0")),
            severity: ValidationSeverity::Critical,
            suggestion: String::from("set bind"),
        };
        assert_json_roundtrip(&err);

        let warn = ValidationWarning {
            field: String::from("cap"),
            message: String::from("empty capabilities"),
            current_value: Some(String::from("[]")),
            recommended_value: Some(String::from("non-empty")),
            severity: ValidationSeverity::Medium,
            suggestion: String::from("add capabilities"),
        };
        assert_json_roundtrip(&warn);
    }

    #[test]
    fn validation_severity_roundtrip() {
        for s in [
            ValidationSeverity::Critical,
            ValidationSeverity::High,
            ValidationSeverity::Medium,
            ValidationSeverity::Low,
        ] {
            assert_json_roundtrip(&s);
        }
    }

    #[test]
    fn test_defaults_passes_validation() {
        let r = SongbirdConfig::test_defaults().validate().unwrap();
        assert!(r.errors.is_empty());
        assert!(r.is_valid);
    }

    #[test]
    fn empty_bind_address_fails() {
        let mut c = SongbirdConfig::test_defaults();
        c.network.bind_address = String::new();
        let r = c.validate().unwrap();
        assert!(
            r.errors.iter().any(|e| e.field == "network.bind_address"),
            "expected bind_address error"
        );
        assert!(!r.is_valid);
    }

    #[test]
    fn invalid_port_range_fails() {
        let mut c = SongbirdConfig::test_defaults();
        c.network.port_range.end = c.network.port_range.start;
        let r = c.validate().unwrap();
        assert!(
            r.errors.iter().any(|e| e.field == "network.port_range"),
            "expected port_range error"
        );
    }

    #[test]
    fn enabled_primal_empty_url_errors() {
        let mut c = SongbirdConfig::test_defaults();
        c.enable_primal("p1", "http://localhost:1");
        if let Some(reg) = &mut c.primal_registry
            && let Some(p) = reg.primals.get_mut("p1")
        {
            p.endpoint.primary_url.clear();
        }
        let r = c.validate().unwrap();
        assert!(
            r.errors.iter().any(|e| e.field.contains("primary_url")),
            "expected empty URL error"
        );
    }

    #[test]
    fn enabled_primal_zero_connection_timeout_errors() {
        let mut c = SongbirdConfig::test_defaults();
        c.enable_primal("p2", "http://localhost:2");
        if let Some(reg) = &mut c.primal_registry
            && let Some(p) = reg.primals.get_mut("p2")
        {
            p.connection_settings.connection_timeout = Duration::ZERO;
        }
        let r = c.validate().unwrap();
        assert!(
            r.errors.iter().any(|e| e.field.contains("connection_timeout")),
            "expected zero timeout error"
        );
    }

    #[test]
    fn enabled_primal_empty_capabilities_warns() {
        let mut c = SongbirdConfig::test_defaults();
        c.enable_primal("p3", "http://localhost:3");
        let r = c.validate().unwrap();
        assert!(
            r.warnings.iter().any(|w| w.field.contains("capabilities")),
            "expected capabilities warning"
        );
    }

    #[test]
    fn security_enabled_without_auth_warns() {
        let mut c = SongbirdConfig::test_defaults();
        c.security.enabled = true;
        c.security.authentication.enabled = false;
        let r = c.validate().unwrap();
        assert!(
            r.warnings.iter().any(|w| w.field == "security.authentication.enabled"),
            "expected auth warning"
        );
    }

    #[test]
    fn security_enabled_without_encryption_warns() {
        let mut c = SongbirdConfig::test_defaults();
        c.security.enabled = true;
        c.security.encryption.at_rest = false;
        c.security.encryption.in_transit = false;
        let r = c.validate().unwrap();
        assert!(
            r.warnings.iter().any(|w| w.field == "security.encryption"),
            "expected encryption warning"
        );
    }

    #[test]
    fn disabled_primals_skip_validation() {
        let c = SongbirdConfig::test_defaults();
        let r = c.validate().unwrap();
        assert!(
            !r.errors.iter().any(|e| e.field.contains("primal_registry")),
            "no primal errors when none enabled"
        );
    }
}
