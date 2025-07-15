//! Zero Touch Configuration Module
//!
//! Configuration generation and management for zero-touch deployment

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use songbird_errors::{Result, SongbirdError};
use super::ZeroTouchConfig;

/// Configuration generator for zero-touch deployment
pub struct ConfigGenerator {
    templates: HashMap<String, ConfigTemplate>,
    environment_overrides: HashMap<String, String>,
}

impl ConfigGenerator {
    /// Create a new configuration generator
    pub fn new() -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
            environment_overrides: HashMap::new(),
        };
        generator.load_default_templates();
        generator
    }

    /// Generate configuration based on environment discovery
    pub fn generate_config(
        &self,
        environment: &str,
        resources: &ResourceRequirements,
    ) -> Result<ZeroTouchConfig> {
        let template = self.templates.get(environment)
            .ok_or_else(|| SongbirdError::Config {
                message: format!("No template found for environment: {environment}"),
            })?;

        let config = ZeroTouchConfig {
            auto_discovery: template.auto_discovery,
            auto_configure: template.auto_configure,
            auto_deploy: template.auto_deploy,
            target_environment: environment.to_string(),
            deployment_timeout: template.deployment_timeout,
            rollback_on_failure: template.rollback_on_failure,
            config_templates: self.generate_config_templates(resources)?,
        };

        Ok(config)
    }

    /// Load default configuration templates
    fn load_default_templates(&mut self) {
        // Development template
        self.templates.insert("development".to_string(), ConfigTemplate {
            auto_discovery: true,
            auto_configure: true,
            auto_deploy: false,
            deployment_timeout: 300,
            rollback_on_failure: true,
            resource_requirements: ResourceRequirements::minimal(),
            network_config: NetworkTemplate::development(),
            security_config: SecurityTemplate::development(),
        });

        // Production template
        self.templates.insert("production".to_string(), ConfigTemplate {
            auto_discovery: true,
            auto_configure: true,
            auto_deploy: true,
            deployment_timeout: 600,
            rollback_on_failure: true,
            resource_requirements: ResourceRequirements::production(),
            network_config: NetworkTemplate::production(),
            security_config: SecurityTemplate::production(),
        });

        // Testing template
        self.templates.insert("testing".to_string(), ConfigTemplate {
            auto_discovery: true,
            auto_configure: true,
            auto_deploy: true,
            deployment_timeout: 180,
            rollback_on_failure: false,
            resource_requirements: ResourceRequirements::minimal(),
            network_config: NetworkTemplate::testing(),
            security_config: SecurityTemplate::testing(),
        });
    }

    /// Generate configuration templates based on resource requirements
    fn generate_config_templates(&self, resources: &ResourceRequirements) -> Result<HashMap<String, PathBuf>> {
        let mut templates = HashMap::new();

        // Generate service configuration
        templates.insert(
            "service".to_string(),
            PathBuf::from("config/service.yaml")
        );

        // Generate network configuration
        templates.insert(
            "network".to_string(),
            PathBuf::from("config/network.yaml")
        );

        // Generate security configuration
        templates.insert(
            "security".to_string(),
            PathBuf::from("config/security.yaml")
        );

        // Generate monitoring configuration if resources allow
        if resources.memory_mb >= 2048 {
            templates.insert(
                "monitoring".to_string(),
                PathBuf::from("config/monitoring.yaml")
            );
        }

        Ok(templates)
    }

    /// Add environment override
    pub fn add_override(&mut self, key: String, value: String) {
        self.environment_overrides.insert(key, value);
    }

    /// Get available templates
    pub fn get_available_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

/// Configuration template for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub auto_discovery: bool,
    pub auto_configure: bool,
    pub auto_deploy: bool,
    pub deployment_timeout: u64,
    pub rollback_on_failure: bool,
    pub resource_requirements: ResourceRequirements,
    pub network_config: NetworkTemplate,
    pub security_config: SecurityTemplate,
}

/// Resource requirements for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
}

impl ResourceRequirements {
    /// Minimal resource requirements
    pub fn minimal() -> Self {
        Self {
            cpu_cores: 1,
            memory_mb: 512,
            storage_gb: 10,
            network_bandwidth_mbps: 10,
        }
    }

    /// Production resource requirements
    pub fn production() -> Self {
        Self {
            cpu_cores: 4,
            memory_mb: 4096,
            storage_gb: 100,
            network_bandwidth_mbps: 1000,
        }
    }

    /// Check if current resources meet requirements
    pub fn meets_requirements(&self, available: &ResourceRequirements) -> bool {
        available.cpu_cores >= self.cpu_cores
            && available.memory_mb >= self.memory_mb
            && available.storage_gb >= self.storage_gb
            && available.network_bandwidth_mbps >= self.network_bandwidth_mbps
    }
}

/// Network configuration template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTemplate {
    pub bind_address: String,
    pub bind_port: u16,
    pub enable_ssl: bool,
    pub enable_proxy: bool,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

impl NetworkTemplate {
    /// Development network template
    pub fn development() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        Self {
            bind_address: env_config.bind_address.clone(),
            bind_port: env_config.bind_port,
            enable_ssl: env_config.require_tls,
            enable_proxy: false, // Development default
            max_connections: env_config.max_connections.min(100), // Reasonable dev limit
            timeout_seconds: env_config.connection_timeout_secs.min(30) as u32,
        }
    }

    /// Production network template
    pub fn production() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        Self {
            bind_address: env_config.bind_address.clone(),
            bind_port: 443,
            enable_ssl: true,
            enable_proxy: true,
            max_connections: 10000,
            timeout_seconds: 60,
        }
    }

    /// Testing network template
    pub fn testing() -> Self {
        Self {
            bind_address: "crate::config::constants::default_bind_address()".to_string(),
            bind_port: 8081,
            enable_ssl: false,
            enable_proxy: false,
            max_connections: 50,
            timeout_seconds: 10,
        }
    }
}

/// Security configuration template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTemplate {
    pub enable_authentication: bool,
    pub enable_authorization: bool,
    pub enable_encryption: bool,
    pub enable_audit_logging: bool,
    pub password_policy: PasswordPolicy,
    pub session_timeout_minutes: u32,
}

impl SecurityTemplate {
    /// Development security template
    pub fn development() -> Self {
        Self {
            enable_authentication: false,
            enable_authorization: false,
            enable_encryption: false,
            enable_audit_logging: false,
            password_policy: PasswordPolicy::lenient(),
            session_timeout_minutes: 480, // 8 hours
        }
    }

    /// Production security template
    pub fn production() -> Self {
        Self {
            enable_authentication: true,
            enable_authorization: true,
            enable_encryption: true,
            enable_audit_logging: true,
            password_policy: PasswordPolicy::strict(),
            session_timeout_minutes: 60, // 1 hour
        }
    }

    /// Testing security template
    pub fn testing() -> Self {
        Self {
            enable_authentication: false,
            enable_authorization: false,
            enable_encryption: false,
            enable_audit_logging: true,
            password_policy: PasswordPolicy::lenient(),
            session_timeout_minutes: 30,
        }
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: u32,
}

impl PasswordPolicy {
    /// Lenient password policy for development
    pub fn lenient() -> Self {
        Self {
            min_length: 4,
            require_uppercase: false,
            require_lowercase: false,
            require_numbers: false,
            require_special_chars: false,
            max_age_days: 365,
        }
    }

    /// Strict password policy for production
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: 90,
        }
    }
}

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a zero-touch configuration
    pub fn validate(config: &ZeroTouchConfig) -> Result<()> {
        if config.deployment_timeout == 0 {
            return Err(SongbirdError::Config {
                message: "Deployment timeout cannot be zero".to_string(),
            });
        }

        if config.target_environment.is_empty() {
            return Err(SongbirdError::Config {
                message: "Target environment cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Validate resource requirements
    pub fn validate_resources(requirements: &ResourceRequirements) -> Result<()> {
        if requirements.cpu_cores == 0 {
            return Err(SongbirdError::Config {
                message: "CPU cores cannot be zero".to_string(),
            });
        }

        if requirements.memory_mb == 0 {
            return Err(SongbirdError::Config {
                message: "Memory cannot be zero".to_string(),
            });
        }

        if requirements.storage_gb == 0 {
            return Err(SongbirdError::Config {
                message: "Storage cannot be zero".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for ZeroTouchConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        Self {
            enabled: false,
            // Use environment configuration - NO MORE HARDCODING!
            bind_port: env_config.bind_port,
            bind_address: env_config.bind_address.clone(),
            auto_discovery: false,
            auto_configure: false,
            auto_deploy: false,
            target_environment: "".to_string(),
            deployment_timeout: 0,
            rollback_on_failure: false,
            config_templates: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_generator_creation() {
        let generator = ConfigGenerator::new();
        let templates = generator.get_available_templates();
        
        assert!(templates.contains(&"development"));
        assert!(templates.contains(&"production"));
        assert!(templates.contains(&"testing"));
    }

    #[test]
    fn test_resource_requirements_minimal() {
        let minimal = ResourceRequirements::minimal();
        assert_eq!(minimal.cpu_cores, 1);
        assert_eq!(minimal.memory_mb, 512);
        assert_eq!(minimal.storage_gb, 10);
    }

    #[test]
    fn test_resource_requirements_production() {
        let production = ResourceRequirements::production();
        assert_eq!(production.cpu_cores, 4);
        assert_eq!(production.memory_mb, 4096);
        assert_eq!(production.storage_gb, 100);
    }

    #[test]
    fn test_resource_requirements_meets() {
        let minimal = ResourceRequirements::minimal();
        let production = ResourceRequirements::production();
        
        assert!(production.meets_requirements(&production));
        assert!(!minimal.meets_requirements(&production));
    }

    #[test]
    fn test_network_template_development() {
        let template = NetworkTemplate::development();
        let env_config = crate::config::environment::EnvironmentConfig::default();
        assert_eq!(template.bind_address, env_config.bind_address);
        assert_eq!(template.bind_port, env_config.bind_port);
    }

    #[test]
    fn test_network_template_production() {
        let template = NetworkTemplate::production();
        let env_config = crate::config::environment::EnvironmentConfig::default();
        assert_eq!(template.bind_address, env_config.bind_address);
        assert_eq!(template.bind_port, 443);
        assert!(template.enable_ssl);
    }

    #[test]
    fn test_security_template_development() {
        let template = SecurityTemplate::development();
        assert!(!template.enable_authentication);
        assert!(!template.enable_authorization);
        assert!(!template.enable_encryption);
    }

    #[test]
    fn test_security_template_production() {
        let template = SecurityTemplate::production();
        assert!(template.enable_authentication);
        assert!(template.enable_authorization);
        assert!(template.enable_encryption);
    }

    #[test]
    fn test_password_policy_lenient() {
        let policy = PasswordPolicy::lenient();
        assert_eq!(policy.min_length, 4);
        assert!(!policy.require_uppercase);
        assert!(!policy.require_numbers);
    }

    #[test]
    fn test_password_policy_strict() {
        let policy = PasswordPolicy::strict();
        assert_eq!(policy.min_length, 12);
        assert!(policy.require_uppercase);
        assert!(policy.require_numbers);
        assert!(policy.require_special_chars);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ZeroTouchConfig::default();
        assert!(ConfigValidator::validate(&config).is_ok());

        config.deployment_timeout = 0;
        assert!(ConfigValidator::validate(&config).is_err());

        config.deployment_timeout = 300;
        config.target_environment = "".to_string();
        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_resource_validation() {
        let mut resources = ResourceRequirements::minimal();
        assert!(ConfigValidator::validate_resources(&resources).is_ok());

        resources.cpu_cores = 0;
        assert!(ConfigValidator::validate_resources(&resources).is_err());

        resources.cpu_cores = 1;
        resources.memory_mb = 0;
        assert!(ConfigValidator::validate_resources(&resources).is_err());

        resources.memory_mb = 512;
        resources.storage_gb = 0;
        assert!(ConfigValidator::validate_resources(&resources).is_err());
    }

    #[test]
    fn test_generate_config() {
        let generator = ConfigGenerator::new();
        let resources = ResourceRequirements::minimal();
        
        let config = generator.generate_config("development", &resources).unwrap();
        assert_eq!(config.target_environment, "development");
        assert!(config.auto_discovery);
        assert!(config.auto_configure);
        assert!(!config.auto_deploy); // Development default
    }
} 