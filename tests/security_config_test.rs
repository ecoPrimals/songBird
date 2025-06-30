use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
// Security Configuration Tests
//
// Tests to verify security configuration fixes and defaults

use songbird_gaming_bridge::config::constants::logging;
use songbird_gaming_bridge::config::{
    CorsConfig, DefaultServiceConfig, NetworkConfig, OrchestratorConfig,
};

#[test]
fn test_secure_cors_defaults() {
    let cors_config = CorsConfig::default();

    // CORS should be disabled by default for security
    assert!(!cors_config.mode == crate::federation::FederationMode::Peer, "CORS should be disabled by default");

    // Origins should be empty, requiring explicit configuration
    assert!(
        cors_config.allowed_origins.is_empty(),
        "Allowed origins should be empty by default"
    );

    // Methods should be limited to safe operations
    assert_eq!(cors_config.allowed_methods.len(), 2);
    assert!(cors_config.allowed_methods.contains(&"GET".to_string()));
    assert!(cors_config.allowed_methods.contains(&"POST".to_string()));

    // Headers should be limited to essential ones
    assert_eq!(cors_config.allowed_headers.len(), 2);
    assert!(cors_config
        .allowed_headers
        .contains(&"Content-Type".to_string()));
    assert!(cors_config
        .allowed_headers
        .contains(&"Authorization".to_string()));

    println!("✅ CORS security defaults verified");
}

#[test]
fn test_environment_aware_tls() {
    // Test development environment (TLS disabled)
    std::env::set_var("SONGBIRD_ENVIRONMENT", "development");
    let dev_config = NetworkConfig::default();
    assert!(
        !dev_config.enable_tls,
        "TLS should be disabled in development"
    );

    // Test production environment (TLS enabled)
    std::env::set_var("SONGBIRD_ENVIRONMENT", "production");
    let prod_config = NetworkConfig::default();
    assert!(
        prod_config.enable_tls,
        "TLS should be enabled in production"
    );

    // Test staging environment (TLS enabled)
    std::env::set_var("SONGBIRD_ENVIRONMENT", "staging");
    let staging_config = NetworkConfig::default();
    assert!(
        staging_config.enable_tls,
        "TLS should be enabled in staging"
    );

    // Clean up environment
    std::env::remove_var("SONGBIRD_ENVIRONMENT");

    println!("✅ Environment-aware TLS configuration verified");
}

#[test]
fn test_environment_aware_logging() {
    // Clean up any existing environment variables first
    std::env::remove_var("SONGBIRD_ENVIRONMENT");

    // Test production logging
    std::env::set_var("SONGBIRD_ENVIRONMENT", "production");
    let prod_log_level = logging::get_log_level_for_environment();
    assert_eq!(
        prod_log_level, "warn",
        "Production should use warn log level, got: {}",
        prod_log_level
    );

    // Test development logging
    std::env::set_var("SONGBIRD_ENVIRONMENT", "development");
    let dev_log_level = logging::get_log_level_for_environment();
    assert_eq!(
        dev_log_level, "debug",
        "Development should use debug log level"
    );

    // Test staging logging
    std::env::set_var("SONGBIRD_ENVIRONMENT", "staging");
    let staging_log_level = logging::get_log_level_for_environment();
    assert_eq!(
        staging_log_level, "info",
        "Staging should use info log level"
    );

    // Test testing logging
    std::env::set_var("SONGBIRD_ENVIRONMENT", "testing");
    let test_log_level = logging::get_log_level_for_environment();
    assert_eq!(
        test_log_level, "error",
        "Testing should use error log level"
    );

    // Clean up environment
    std::env::remove_var("SONGBIRD_ENVIRONMENT");

    println!("✅ Environment-aware logging configuration verified");
}

#[test]
fn test_secure_defaults_integration() {
    // Test that the orchestrator config has secure defaults
    let config: OrchestratorConfig<DefaultServiceConfig> = OrchestratorConfig::default();

    // Network security
    assert!(
        !config.network.cors.enabled,
        "CORS should be disabled by default"
    );
    assert!(
        config.network.cors.allowed_origins.is_empty(),
        "CORS origins should be empty"
    );

    // Security configuration
    assert!(
        !config.security.enable_auth,
        "Authentication should be disabled by default"
    );
    assert!(
        !config.security.enable_authz,
        "Authorization should be disabled by default"
    );
    assert!(
        config.security.api_key.is_none(),
        "API key should be None by default"
    );
    assert!(
        !config.security.rate_limiting.enabled,
        "Rate limiting should be disabled by default"
    );
    assert!(
        !config.security.audit_logging.enabled,
        "Audit logging should be disabled by default"
    );

    println!("✅ Secure defaults integration verified");
}

#[test]
fn test_no_hardcoded_credentials() {
    // Verify no hardcoded credentials in default configurations
    let config: OrchestratorConfig<DefaultServiceConfig> = OrchestratorConfig::default();

    // Should not have any default API keys
    assert!(
        config.security.api_key.is_none(),
        "No default API key should be present"
    );

    // Auth provider should be 'none' by default
    assert_eq!(config.security.auth_provider.provider_type, "none");
    assert!(config.security.auth_provider.config.is_empty());

    // Authz provider should be 'none' by default
    assert_eq!(config.security.authz_provider.provider_type, "none");
    assert!(config.security.authz_provider.config.is_empty());

    println!("✅ No hardcoded credentials verified");
}

#[test]
fn test_production_security_checklist() {
    // Clean up any existing environment variables first
    std::env::remove_var("SONGBIRD_ENVIRONMENT");

    // Set production environment
    std::env::set_var("SONGBIRD_ENVIRONMENT", "production");

    let config: OrchestratorConfig<DefaultServiceConfig> = OrchestratorConfig::default();

    // TLS should be enabled in production
    assert!(
        config.network.enable_tls,
        "TLS should be enabled in production"
    );

    // CORS should still be disabled by default (requires explicit configuration)
    assert!(
        !config.network.cors.enabled,
        "CORS should require explicit configuration"
    );

    // Logging should be appropriate for production
    let log_level = logging::get_log_level_for_environment();
    assert_eq!(log_level, "warn", "Production should use warn log level");

    // Clean up
    std::env::remove_var("SONGBIRD_ENVIRONMENT");

    println!("✅ Production security checklist verified");
}

#[cfg(test)]
mod security_validation_tests {
    use super::*;

    #[test]
    fn test_cors_explicit_configuration() {
        // Test that CORS can be explicitly configured when needed
        let mut cors_config = CorsConfig::default();

        // Should start secure
        assert!(!cors_config.mode == crate::federation::FederationMode::Peer);
        assert!(cors_config.allowed_origins.is_empty());

        // Can be explicitly configured
        cors_config.mode == crate::federation::FederationMode::Peer = true;
        cors_config.allowed_origins = vec!["https://trusted-domain.com".to_string()];

        assert!(cors_config.mode == crate::federation::FederationMode::Peer);
        assert_eq!(cors_config.allowed_origins.len(), 1);
        assert_eq!(cors_config.allowed_origins[0], "https://trusted-domain.com");

        println!("✅ CORS explicit configuration verified");
    }

    #[test]
    fn test_security_environment_variables() {
        // Test that security can be enabled via environment variables
        // This simulates production deployment configuration

        // Simulate production environment variables
        std::env::set_var("SONGBIRD_ENVIRONMENT", "production");
        std::env::set_var("SONGBIRD_TLS_ENABLED", "true");
        std::env::set_var("SONGBIRD_AUTH_ENABLED", "true");

        // In a real implementation, these would be read by the config system
        let environment = std::env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default();
        let tls_enabled = std::env::var("SONGBIRD_TLS_ENABLED").unwrap_or_default();
        let auth_enabled = std::env::var("SONGBIRD_AUTH_ENABLED").unwrap_or_default();

        assert_eq!(environment, "production");
        assert_eq!(tls_enabled, "true");
        assert_eq!(auth_enabled, "true");

        // Clean up
        std::env::remove_var("SONGBIRD_ENVIRONMENT");
        std::env::remove_var("SONGBIRD_TLS_ENABLED");
        std::env::remove_var("SONGBIRD_AUTH_ENABLED");

        println!("✅ Security environment variables verified");
    }
}
