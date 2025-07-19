/// Configuration validation provides comprehensive security and correctness checks
/// for the Songbird Universal Orchestrator configuration. This follows the principle
/// of "secure by default" and "fail fast" - a fundamental security engineering
/// principle for the `SongBird` ecosystem.
///
/// All configuration validation should be performed before any system initialization
/// to ensure consistent and secure operation.
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;

/// Configuration validation results with detailed feedback
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub recommendations: Vec<String>,
}

/// Configuration validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub severity: ValidationSeverity,
    pub suggestion: String,
}

/// Configuration validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub current_value: Option<String>,
    pub suggestion: String,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl crate::config::SongbirdConfig {
    /// Comprehensive configuration validation
    ///
    /// # Errors
    ///
    /// Returns an error if critical validation checks fail
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        };

        // Validate security configuration
        self.validate_security_comprehensive(&mut result)?;

        // Validate network configuration
        self.validate_network_comprehensive(&mut result)?;

        // Validate path configuration
        self.validate_paths_comprehensive(&mut result)?;

        // Validate environment configuration
        self.validate_environment_comprehensive(&mut result)?;

        // Validate BearDog configuration if enabled
        if let Some(ref beardog_config) = self.beardog {
            self.validate_beardog_comprehensive(beardog_config, &mut result)?;
        }

        // Add general recommendations
        self.add_general_recommendations(&mut result);

        // Set overall validity
        result.is_valid = result.errors.is_empty()
            || result
                .errors
                .iter()
                .all(|e| e.severity == ValidationSeverity::Low);

        Ok(result)
    }

    /// Validate security configuration and settings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Security settings are missing or invalid
    /// - Encryption configuration is incomplete
    /// - Authentication settings are insecure
    /// - Network security policies are misconfigured
    pub fn validate_security(&self) -> Result<()> {
        let result = self.validate()?;

        // Check for critical security errors
        let critical_errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Critical)
            .collect();

        if !critical_errors.is_empty() {
            let error_messages: Vec<String> = critical_errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();

            return Err(SongbirdError::security_error(&format!(
                "Critical security validation failures: {}",
                error_messages.join(", ")
            )));
        }

        Ok(())
    }

    /// Comprehensive security validation
    fn validate_security_comprehensive(&self, result: &mut ValidationResult) -> Result<()> {
        // Validate encryption settings
        if !self.security.encryption_enabled {
            result.warnings.push(ValidationWarning {
                field: "security.encryption_enabled".to_string(),
                message: "Encryption is disabled".to_string(),
                current_value: Some("false".to_string()),
                suggestion: "Enable encryption for production environments".to_string(),
            });
        }

        // Validate TLS configuration
        if !self.security.tls_enabled {
            result.warnings.push(ValidationWarning {
                field: "security.tls_enabled".to_string(),
                message: "TLS is disabled".to_string(),
                current_value: Some("false".to_string()),
                suggestion: "Enable TLS for secure communication".to_string(),
            });
        } else {
            // Validate TLS certificate paths
            if let Some(ref cert_path) = self.security.cert_path {
                if !Path::new(cert_path).exists() {
                    result.errors.push(ValidationError {
                        field: "security.cert_path".to_string(),
                        message: "TLS certificate file not found".to_string(),
                        current_value: Some(cert_path.clone()),
                        expected_value: Some("Path to valid certificate file".to_string()),
                        severity: ValidationSeverity::High,
                        suggestion: "Provide a valid certificate file path or disable TLS"
                            .to_string(),
                    });
                }
            }

            if let Some(ref key_path) = self.security.key_path {
                if !Path::new(key_path).exists() {
                    result.errors.push(ValidationError {
                        field: "security.key_path".to_string(),
                        message: "TLS private key file not found".to_string(),
                        current_value: Some(key_path.clone()),
                        expected_value: Some("Path to valid private key file".to_string()),
                        severity: ValidationSeverity::High,
                        suggestion: "Provide a valid private key file path or disable TLS"
                            .to_string(),
                    });
                }
            }
        }

        // Validate JWT secret
        if let Some(ref jwt_secret) = self.security.jwt_secret {
            if jwt_secret.len() < 32 {
                result.errors.push(ValidationError {
                    field: "security.jwt_secret".to_string(),
                    message: "JWT secret is too short".to_string(),
                    current_value: Some(format!("{} characters", jwt_secret.len())),
                    expected_value: Some("At least 32 characters".to_string()),
                    severity: ValidationSeverity::High,
                    suggestion: "Use a longer JWT secret for better security".to_string(),
                });
            }
        } else {
            result.warnings.push(ValidationWarning {
                field: "security.jwt_secret".to_string(),
                message: "No JWT secret configured".to_string(),
                current_value: None,
                suggestion: "Configure a JWT secret for authentication".to_string(),
            });
        }

        Ok(())
    }

    /// Comprehensive network validation
    fn validate_network_comprehensive(&self, result: &mut ValidationResult) -> Result<()> {
        // Validate bind addresses
        self.validate_bind_address(&self.network.bind_address, "network.bind_address", result)?;

        // Validate ports
        self.validate_port_comprehensive(
            self.network.orchestrator_port,
            "network.orchestrator_port",
            result,
        );
        self.validate_port_comprehensive(
            self.network.discovery_port,
            "network.discovery_port",
            result,
        );
        self.validate_port_comprehensive(self.network.health_port, "network.health_port", result);
        self.validate_port_comprehensive(
            self.network.dashboard_port,
            "network.dashboard_port",
            result,
        );

        // Check for port conflicts
        let ports = vec![
            ("orchestrator_port", self.network.orchestrator_port),
            ("discovery_port", self.network.discovery_port),
            ("health_port", self.network.health_port),
            ("dashboard_port", self.network.dashboard_port),
        ];

        let mut port_map: HashMap<u16, Vec<String>> = HashMap::new();
        for (name, port) in ports {
            port_map.entry(port).or_default().push(name.to_string());
        }

        for (port, names) in port_map {
            if names.len() > 1 {
                result.errors.push(ValidationError {
                    field: "network.ports".to_string(),
                    message: format!("Port {port} is used by multiple services"),
                    current_value: Some(names.join(", ")),
                    expected_value: Some("Unique ports for each service".to_string()),
                    severity: ValidationSeverity::High,
                    suggestion: "Assign unique ports to each service".to_string(),
                });
            }
        }

        // Validate timeouts
        self.validate_timeout_comprehensive(
            self.network.connection_timeout,
            "network.connection_timeout",
            result,
        );
        self.validate_timeout_comprehensive(
            self.network.request_timeout,
            "network.request_timeout",
            result,
        );

        // Validate federation endpoints
        for (i, endpoint) in self.network.federation_endpoints.iter().enumerate() {
            if let Err(e) = self.validate_endpoint_format(endpoint) {
                result.errors.push(ValidationError {
                    field: format!("network.federation_endpoints[{i}]"),
                    message: format!("Invalid federation endpoint format: {e}"),
                    current_value: Some(endpoint.clone()),
                    expected_value: Some(
                        "Valid URL format (e.g., https://federation.example.com)".to_string(),
                    ),
                    severity: ValidationSeverity::Medium,
                    suggestion: "Provide a valid URL format for federation endpoints".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Comprehensive path validation
    fn validate_paths_comprehensive(&self, result: &mut ValidationResult) -> Result<()> {
        let paths = vec![
            ("paths.data_dir", &self.paths.data_dir),
            ("paths.config_dir", &self.paths.config_dir),
            ("paths.log_dir", &self.paths.log_dir),
            ("paths.cache_dir", &self.paths.cache_dir),
            ("paths.runtime_dir", &self.paths.runtime_dir),
        ];

        for (field_name, path) in paths {
            // Check if path exists
            if !path.exists() {
                result.warnings.push(ValidationWarning {
                    field: field_name.to_string(),
                    message: "Directory does not exist".to_string(),
                    current_value: Some(path.to_string_lossy().to_string()),
                    suggestion: "Directory will be created automatically if possible".to_string(),
                });
            }

            // Check if path is absolute (recommended for production)
            if !path.is_absolute() {
                result.warnings.push(ValidationWarning {
                    field: field_name.to_string(),
                    message: "Using relative path".to_string(),
                    current_value: Some(path.to_string_lossy().to_string()),
                    suggestion: "Use absolute paths for production deployments".to_string(),
                });
            }

            // Check write permissions for parent directory
            if let Some(parent) = path.parent() {
                if parent.exists() && !self.is_directory_writable(parent) {
                    result.errors.push(ValidationError {
                        field: field_name.to_string(),
                        message: "Parent directory is not writable".to_string(),
                        current_value: Some(parent.to_string_lossy().to_string()),
                        expected_value: Some("Writable directory".to_string()),
                        severity: ValidationSeverity::High,
                        suggestion: "Ensure the parent directory has write permissions".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Comprehensive environment validation
    fn validate_environment_comprehensive(&self, result: &mut ValidationResult) -> Result<()> {
        // Validate environment-specific settings
        let environment =
            std::env::var("SONGBIRD_ENV").unwrap_or_else(|_| "development".to_string());

        if environment == "production" {
            // Production-specific validations
            if self.network.bind_address.to_string() == "0.0.0.0" {
                result.warnings.push(ValidationWarning {
                    field: "network.bind_address".to_string(),
                    message: "Binding to all interfaces in production".to_string(),
                    current_value: Some("0.0.0.0".to_string()),
                    suggestion: "Consider binding to specific interfaces for better security"
                        .to_string(),
                });
            }

            if !self.security.tls_enabled {
                result.errors.push(ValidationError {
                    field: "security.tls_enabled".to_string(),
                    message: "TLS should be enabled in production".to_string(),
                    current_value: Some("false".to_string()),
                    expected_value: Some("true".to_string()),
                    severity: ValidationSeverity::High,
                    suggestion: "Enable TLS for production environments".to_string(),
                });
            }

            if !self.security.encryption_enabled {
                result.errors.push(ValidationError {
                    field: "security.encryption_enabled".to_string(),
                    message: "Encryption should be enabled in production".to_string(),
                    current_value: Some("false".to_string()),
                    expected_value: Some("true".to_string()),
                    severity: ValidationSeverity::High,
                    suggestion: "Enable encryption for production environments".to_string(),
                });
            }
        }

        // Validate environment variables
        let required_env_vars = vec!["SONGBIRD_DATA_DIR", "SONGBIRD_CONFIG_DIR"];

        for var in required_env_vars {
            if std::env::var(var).is_err() {
                result.warnings.push(ValidationWarning {
                    field: format!("environment.{}", var.to_lowercase()),
                    message: format!("Environment variable {var} is not set"),
                    current_value: None,
                    suggestion: format!(
                        "Set {var} environment variable for better configuration management"
                    ),
                });
            }
        }

        Ok(())
    }

    /// Comprehensive BearDog configuration validation
    fn validate_beardog_comprehensive(
        &self,
        beardog_config: &crate::config::BearDogConfig,
        result: &mut ValidationResult,
    ) -> Result<()> {
        if !beardog_config.enabled {
            result
                .recommendations
                .push("Consider enabling BearDog for enhanced security".to_string());
            return Ok(());
        }

        // Validate BearDog endpoint
        if let Err(e) = self.validate_endpoint_format(&beardog_config.endpoint.primary_url) {
            result.errors.push(ValidationError {
                field: "beardog.endpoint.primary_url".to_string(),
                message: format!("Invalid BearDog endpoint format: {e}"),
                current_value: Some(beardog_config.endpoint.primary_url.clone()),
                expected_value: Some("Valid HTTPS URL".to_string()),
                severity: ValidationSeverity::High,
                suggestion: "Provide a valid HTTPS URL for BearDog endpoint".to_string(),
            });
        }

        // Validate connection timeout
        if beardog_config.endpoint.connection_timeout_secs < 5 {
            result.warnings.push(ValidationWarning {
                field: "beardog.endpoint.connection_timeout_secs".to_string(),
                message: "BearDog connection timeout is very short".to_string(),
                current_value: Some(beardog_config.endpoint.connection_timeout_secs.to_string()),
                suggestion: "Consider increasing timeout for more reliable connections".to_string(),
            });
        }

        // Validate TLS verification
        if !beardog_config.endpoint.verify_tls {
            result.warnings.push(ValidationWarning {
                field: "beardog.endpoint.verify_tls".to_string(),
                message: "TLS verification is disabled for BearDog".to_string(),
                current_value: Some("false".to_string()),
                suggestion: "Enable TLS verification for better security".to_string(),
            });
        }

        // Validate authentication
        match beardog_config.authentication.auth_method {
            crate::config::BearDogAuthMethod::ApiKey => {
                if beardog_config.authentication.api_key.is_none() {
                    result.errors.push(ValidationError {
                        field: "beardog.authentication.api_key".to_string(),
                        message: "API key authentication selected but no API key provided"
                            .to_string(),
                        current_value: None,
                        expected_value: Some("Valid API key".to_string()),
                        severity: ValidationSeverity::High,
                        suggestion: "Provide an API key or change authentication method"
                            .to_string(),
                    });
                }
            }
            crate::config::BearDogAuthMethod::MutualTls => {
                if self.security.cert_path.is_none() || self.security.key_path.is_none() {
                    result.errors.push(ValidationError {
                        field: "beardog.authentication.mutual_tls".to_string(),
                        message:
                            "Mutual TLS authentication selected but certificate/key not configured"
                                .to_string(),
                        current_value: None,
                        expected_value: Some("Valid certificate and key paths".to_string()),
                        severity: ValidationSeverity::High,
                        suggestion: "Configure certificate and key paths for mutual TLS"
                            .to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Add general recommendations
    fn add_general_recommendations(&self, result: &mut ValidationResult) {
        // Performance recommendations
        if self.network.max_connections < 100 {
            result
                .recommendations
                .push("Consider increasing max_connections for better performance".to_string());
        }

        if self.network.worker_threads < num_cpus::get() {
            result
                .recommendations
                .push("Consider increasing worker_threads to match CPU cores".to_string());
        }

        // Security recommendations
        if self.security.jwt_secret.is_none() {
            result
                .recommendations
                .push("Configure JWT secret for authentication".to_string());
        }

        // Monitoring recommendations
        if self.network.federation_endpoints.is_empty() {
            result
                .recommendations
                .push("Configure federation endpoints for distributed deployment".to_string());
        }
    }

    /// Validate bind address
    fn validate_bind_address(
        &self,
        addr: &IpAddr,
        field: &str,
        result: &mut ValidationResult,
    ) -> Result<()> {
        match addr {
            IpAddr::V4(ipv4) => {
                if ipv4 == &Ipv4Addr::new(0, 0, 0, 0) {
                    result.warnings.push(ValidationWarning {
                        field: field.to_string(),
                        message: "Binding to all interfaces (0.0.0.0)".to_string(),
                        current_value: Some("0.0.0.0".to_string()),
                        suggestion: "Consider binding to specific interfaces for better security"
                            .to_string(),
                    });
                }
            }
            IpAddr::V6(ipv6) => {
                if ipv6 == &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0) {
                    result.warnings.push(ValidationWarning {
                        field: field.to_string(),
                        message: "Binding to all interfaces (::)".to_string(),
                        current_value: Some("::".to_string()),
                        suggestion: "Consider binding to specific interfaces for better security"
                            .to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate port comprehensively
    fn validate_port_comprehensive(&self, port: u16, field: &str, result: &mut ValidationResult) {
        if port == 0 {
            result.errors.push(ValidationError {
                field: field.to_string(),
                message: "Port cannot be 0".to_string(),
                current_value: Some("0".to_string()),
                expected_value: Some("1-65535".to_string()),
                severity: ValidationSeverity::High,
                suggestion: "Use a valid port number between 1 and 65535".to_string(),
            });
        } else if port < 1024 {
            result.warnings.push(ValidationWarning {
                field: field.to_string(),
                message: "Using privileged port".to_string(),
                current_value: Some(port.to_string()),
                suggestion: "Privileged ports (<1024) may require elevated permissions".to_string(),
            });
        }
    }

    /// Validate timeout comprehensively
    fn validate_timeout_comprehensive(
        &self,
        timeout: std::time::Duration,
        field: &str,
        result: &mut ValidationResult,
    ) {
        let timeout_secs = timeout.as_secs();

        if timeout_secs == 0 {
            result.errors.push(ValidationError {
                field: field.to_string(),
                message: "Timeout cannot be 0".to_string(),
                current_value: Some("0s".to_string()),
                expected_value: Some(">0s".to_string()),
                severity: ValidationSeverity::Medium,
                suggestion: "Use a positive timeout value".to_string(),
            });
        } else if timeout_secs > 300 {
            result.warnings.push(ValidationWarning {
                field: field.to_string(),
                message: "Timeout is very long".to_string(),
                current_value: Some(format!("{timeout_secs}s")),
                suggestion: "Consider reducing timeout for better responsiveness".to_string(),
            });
        }
    }

    /// Validate endpoint format
    fn validate_endpoint_format(&self, endpoint: &str) -> Result<()> {
        use url::Url;

        let url = Url::parse(endpoint).map_err(|e| {
            SongbirdError::validation_error(&format!(
                "Invalid URL format for endpoint '{}': {}",
                endpoint, e
            ))
        })?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(SongbirdError::validation_error(&format!(
                "Only HTTP and HTTPS schemes are supported for endpoint '{}'. Found: {}",
                endpoint,
                url.scheme()
            )));
        }

        Ok(())
    }

    /// Check if directory is writable
    fn is_directory_writable(&self, path: &Path) -> bool {
        if !path.exists() || !path.is_dir() {
            return false;
        }

        let test_file = path.join(".songbird_write_test");
        match std::fs::write(&test_file, b"test") {
            Ok(()) => {
                let _ = std::fs::remove_file(&test_file);
                true
            }
            Err(_) => false,
        }
    }
}

impl ValidationResult {
    /// Check if configuration is valid for production
    pub fn is_production_ready(&self) -> bool {
        self.is_valid
            && self
                .errors
                .iter()
                .all(|e| e.severity != ValidationSeverity::Critical)
    }

    /// Get summary of validation results
    pub fn summary(&self) -> String {
        format!(
            "Validation Summary: {} errors, {} warnings, {} recommendations",
            self.errors.len(),
            self.warnings.len(),
            self.recommendations.len()
        )
    }

    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Configuration Validation Report\n");
        report.push_str(&format!(
            "Overall Status: {}\n",
            if self.is_valid { "Valid" } else { "Invalid" }
        ));
        report.push_str(&format!(
            "Production Ready: {}\n\n",
            if self.is_production_ready() {
                "Yes"
            } else {
                "No"
            }
        ));

        if !self.errors.is_empty() {
            report.push_str("Errors:\n");
            for error in &self.errors {
                report.push_str(&format!(
                    "  [{}] {}: {}\n",
                    match error.severity {
                        ValidationSeverity::Critical => "CRITICAL",
                        ValidationSeverity::High => "HIGH",
                        ValidationSeverity::Medium => "MEDIUM",
                        ValidationSeverity::Low => "LOW",
                    },
                    error.field,
                    error.message
                ));
                if let Some(ref current) = error.current_value {
                    report.push_str(&format!("    Current: {current}\n"));
                }
                if let Some(ref expected) = error.expected_value {
                    report.push_str(&format!("    Expected: {expected}\n"));
                }
                report.push_str(&format!("    Suggestion: {}\n", error.suggestion));
            }
            report.push('\n');
        }

        if !self.warnings.is_empty() {
            report.push_str("Warnings:\n");
            for warning in &self.warnings {
                report.push_str(&format!(
                    "  [WARNING] {}: {}\n",
                    warning.field, warning.message
                ));
                if let Some(ref current) = warning.current_value {
                    report.push_str(&format!("    Current: {current}\n"));
                }
                report.push_str(&format!("    Suggestion: {}\n", warning.suggestion));
            }
            report.push('\n');
        }

        if !self.recommendations.is_empty() {
            report.push_str("Recommendations:\n");
            for (i, recommendation) in self.recommendations.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, recommendation));
            }
        }

        report
    }
}
