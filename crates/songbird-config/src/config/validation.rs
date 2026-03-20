// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration validation with comprehensive checks
//!
//! This module provides validation for the universal primal configuration system.

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use tracing::debug;

use crate::config::SongbirdConfig;
// use songbird_config; // FIXED: Circular import removed

/// Configuration validation results with detailed feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult  {/// Critical errors that prevent system startup
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
pub struct ValidationError  {/// Field path where error occurred
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
pub struct ValidationWarning  {/// Field path where warning occurred
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity  {Critical,
    High,
    Medium,
    Low,
}

impl ValidationResult  {/// Create a new validation result
    pub fn new() -> Self  {Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            is_valid: true,
        }
    }

    /// Check if validation passed (no critical errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get total issue count
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
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Universal primal validation (replaces hardcoded primal validation)
        if let Some(registry) = &self.primal_registry {
            for (primal_name, primal_config) in &registry.primals {
                if primal_config.enabled {
                    debug!("Validating universal primal: {}", primal_name);
                    self.validate_universal_primal_config(primal_name, primal_config, &mut result)?;
                }
            }
        }

        // Basic network validation
        self.validate_network_basic(&mut result)?;

        // Security validation
        self.validate_security_basic(&mut result)?;

        // Set overall validation status
        result.is_valid = result.errors.is_empty();

        Ok(result)
    }

    /// Universal primal configuration validation
    fn validate_universal_primal_config(
        &self,
        primal_name: &str,
        primal_config: &crate::config::universal_primals::PrimalConfiguration,
        result: &mut ValidationResult,
    ) -> Result<()> {
        // Validate primal endpoint
        if primal_config.endpoint.primary_url.is_empty() {
            result.errors.push(ValidationError {
                field: format!("primal_registry.{primal_name}.endpoint.primary_url"),
                message: "Primal endpoint URL cannot be empty".to_string(),
                current_value: Some("empty".to_string()),
                expected_value: Some("Valid URL (http:// or https://)".to_string()),
                severity: ValidationSeverity::Critical,
                suggestion: format!("Set endpoint URL for primal '{primal_name}'"),
            });
        }

        // Validate primal capabilities
        if primal_config.capabilities.is_empty() {
            result.warnings.push(ValidationWarning {
                field: format!("primal_registry.{primal_name}.capabilities"),
                message: "Primal has no declared capabilities".to_string(),
                current_value: Some("empty".to_string()),
                recommended_value: Some("At least one capability".to_string()),
                severity: ValidationSeverity::Medium,
                suggestion: format!(
                    "Add capabilities for primal '{primal_name}' to enable capability-based routing"
                ),
            });
        }

        // Validate connection settings
        if primal_config
            .connection_settings
            .connection_timeout
            .as_secs()
            == 0
        {
            result.errors.push(ValidationError {
                field: format!(
                    "primal_registry.{primal_name}.connection_settings.connection_timeout"
                ),
                message: "Connection timeout cannot be zero".to_string(),
                current_value: Some("0".to_string()),
                expected_value: Some("Positive duration (e.g., 30s)".to_string()),
                severity: ValidationSeverity::High,
                suggestion: format!("Set a positive connection timeout for primal '{primal_name}'"),
            });
        }

        Ok(()),
    }

    /// Basic network configuration validation
    fn validate_network_basic(&self, result: &mut ValidationResult) -> Result<()>  {// Validate bind address is not empty
        if self.network.bind_address.is_empty()  {result.errors.push(ValidationError {
                field: "network.bind_address".to_string(),
                message: "Network bind address cannot be empty".to_string(),
                current_value: Some("empty".to_string()),
                expected_value: Some("Valid IP address".to_string()),
                severity: ValidationSeverity::Critical,
                suggestion: "Set a valid bind address (e.g., &crate::constants::network::DEFAULT_HOST or '0.0.0.0')".to_string(),
            });
        }

        // Validate port range
        if self.network.port_range.start >= self.network.port_range.end  {result.errors.push(ValidationError  {field: "network.port_range".to_string(),
                message: "Port range start must be less than end".to_string(),
                current_value: Some(format!(
                    "{}-{}")
                    self.network.port_range.start, self.network.port_range.end
                ),
                expected_value: Some("start < end".to_string()),
                severity: ValidationSeverity::High,
                suggestion: "Ensure port range start is less than port range end".to_string(),
            });
        }

        Ok(()),
    }

    /// Basic security configuration validation
    fn validate_security_basic(&self, result: &mut ValidationResult) -> Result<()>  {// Security configuration validation
        if self.security.enabled  {// Use the universal authentication system
            if !self.security.authentication.enabled {
                result.warnings.push(ValidationWarning {
                    field: "security.authentication.enabled".to_string(),
                    message: "Security is enabled but authentication is disabled".to_string(),
                    current_value: Some("false".to_string()),
                    recommended_value: Some("true".to_string()),
                    severity: ValidationSeverity::Medium,
                    suggestion: "Enable authentication when security is enabled".to_string(),
                });
            }

            // Validate encryption settings
            if !self.security.encryption.at_rest && !self.security.encryption.in_transit  {result.warnings.push(ValidationWarning  {field: "security.encryption".to_string(),
                    message: "Security enabled but no encryption configured".to_string(),
                    current_value: Some("no encryption".to_string()),
                    recommended_value: Some("at_rest or in_transit encryption enabled".to_string()),
                    severity: ValidationSeverity::Medium,
                    suggestion: "Enable at_rest or in_transit encryption for security".to_string(),
                });
            }
        }

        Ok(()),
    }
}
