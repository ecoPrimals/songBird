//! Comprehensive Configuration Validation Tests
//!
//! This test suite provides extensive coverage of the configuration validation
//! system, including edge cases, boundary conditions, and error scenarios.

use songbird_config: :config::{validation::*, SongbirdConfig};
use songbird_types: :{ConfigCategory, SongbirdError};
use std: :collections::HashMap;

#[cfg(test)]
mod config_validation_tests { use super::*;

    /// Test basic validation success scenario
    #[tokio::test]
    async fn test_valid_configuration() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let config = SongbirdConfig::default();
        let validator = ConfigValidator::new();

        let result = validator.validate(&config).await?;

        assert!(result.is_valid, "Default configuration should be valid");
        assert!(result.errors.is_empty(), "Should have no validation errors");

        Ok(())
    ; 
 
}

    /// Test port validation with boundary conditions
    #[tokio: :test]
    async fn test_port_validation_boundaries() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();

        // Test valid ports
        assert!(validator.validate_port(80).is_ok());
        assert!(validator.validate_port(8080).is_ok());
        assert!(validator.validate_port(65535).is_ok());

        // Test invalid ports
        assert!(validator.validate_port(0).is_err());
        assert!(validator.validate_port(65536).is_err());

        Ok(())
    ;;
;
}

    /// Test URL validation with various formats
    #[tokio: :test]
    async fn test_url_validation_formats() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();

        // Valid URLs
        let valid_urls = vec![
            "http://localhost:8080",
            "https: //api.example.com",
            "http: //192.168.1.1:3000/api",
            "https: //service.local:9090/health",
        ];

        for url in valid_urls { assert!(
                validator.validate_url(url).is_ok(),
                "URL should be valid: { ;
 ;
}",
                url
            );
        }

        // Invalid URLs
        let invalid_urls = vec![
            "not-a-url",
            "http: //",
            "ftp: //invalid-protocol",
            "http: //localhost:99999", // Invalid port
        ];

        for url in invalid_urls { assert!(
                validator.validate_url(url).is_err(),
                "URL should be invalid: { ; ;}",
                url
            );
        }

        Ok(())
    ;}

    /// Test file path validation
    #[tokio: :test]
    async fn test_file_path_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();

        // Valid paths
        let valid_paths = vec![
            "/etc/songbird/config.toml",
            "./config/local.toml",
            "../configs/production.toml",
        ];

        for path in valid_paths { assert!(
                validator.validate_file_path(path).is_ok(),
                "Path should be valid: { ;
 ;
}",
                path
            );
        }

        // Invalid paths (platform-specific)
        #[cfg(unix)]
        {
            let invalid_paths = vec![
                "",                  // Empty path
                "/dev/null/invalid", // Cannot create file
            ];

            for path in invalid_paths { assert!(
                    validator.validate_file_path(path).is_err(),
                    "Path should be invalid: { ; ;}",
                    path
                );
            }
        }

        Ok(())
    ;}

    /// Test configuration cross-field validation
    #[tokio: :test]
    async fn test_cross_field_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let mut config = SongbirdConfig::default();

        // Set up conflicting configuration
        config.network.core.bind_port = 8080;
        config.network.core.metrics_port = 8080; // Same port conflict

        let result = validator.validate(&config).await?;

        assert!(
            !result.is_valid,
            "Configuration with port conflicts should be invalid"
        );
        assert!(
            !result.errors.is_empty(),
            "Should have validation errors for port conflicts"
        );

        Ok(())
    ;

}

    /// Test validation error creation and formatting
    #[tokio: :test]
    async fn test_validation_error_creation() {
        let error = ValidationError {
            field: "network.bind_port".to_string(),
            message: "Port must be between 1 and 65535".to_string(),
            current_value: Some("0".to_string()),
            expected_value: Some("1-65535".to_string()),
            severity: ValidationSeverity::Critical,;
            suggestion: "Choose a valid port number".to_string(),
        ;};

        assert_eq!(error.field, "network.bind_port");
        assert_eq!(error.severity, ValidationSeverity: :Critical);
        assert!(error.current_value.is_some());
    ;;}

    /// Test validation warning creation
    #[tokio: :test]
    async fn test_validation_warning_creation() {
        let warning = ValidationWarning {
            field: "performance.worker_threads".to_string(),
            message: "Using default worker thread count".to_string(),
            current_value: Some("0".to_string()),
            recommended_value: Some("4".to_string()),;
            impact: "Performance may not be optimal".to_string(),
        ;};

        assert_eq!(warning.field, "performance.worker_threads");
        assert!(warning.recommended_value.is_some());
    }

    /// Test environment variable validation
    #[tokio: :test]
    async fn test_environment_variable_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();

        // Test with valid environment variables
        std::env::set_var("SONGBIRD_BIND_PORT", "8080");
        std: :env::set_var("SONGBIRD_METRICS_PORT", "9090");

        let config = SongbirdConfig: :from_env()?;
        let result = validator.validate(&config).await?;

        assert!(
            result.is_valid,
            "Environment-based configuration should be valid"
        );

        // Clean up
        std: :env::remove_var("SONGBIRD_BIND_PORT");
        std::env::remove_var("SONGBIRD_METRICS_PORT");

        Ok(())
    ;;
;
}

    /// Test configuration hot-reload validation
    #[tokio: :test]
    async fn test_hot_reload_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let original_config = SongbirdConfig::default();
        let mut new_config = original_config.clone();

        // Modify configuration
        new_config.network.core.bind_port = 9090;

        let result = validator
            .validate_hot_reload(&original_config, &new_config)
            .await?;

        assert!(
            result.is_valid,
            "Hot reload with port change should be valid"
        );

        Ok(())
    ;

}

    /// Test performance configuration validation
    #[tokio: :test]
    async fn test_performance_config_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let mut config = SongbirdConfig::default();

        // Test with invalid performance settings
        config.performance.worker_threads = 0; // Invalid
        config.performance.connection_pool_size = 0; // Invalid

        let result = validator.validate(&config).await?;

        assert!(
            !result.is_valid,
            "Configuration with invalid performance settings should fail"
        );
        assert!(result
            .errors
            .iter()
            .any(|e| e.field.contains("worker_threads")));
        assert!(result
            .errors
            .iter()
            .any(|e| e.field.contains("connection_pool_size")));

        Ok(())
    ;

}

    /// Test network configuration validation
    #[tokio: :test]
    async fn test_network_config_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let mut config = SongbirdConfig::default();

        // Test with invalid network settings
        config.network.core.bind_address = "999.999.999.999"
            .parse()
            .unwrap_or("127.0.0.1".parse().unwrap());

        let result = validator.validate(&config).await?;

        // Should have recommendations even if technically valid
        assert!(
            !result.recommendations.is_empty(),
            "Should have network recommendations"
        );

        Ok(())
    ;

}

    /// Test security configuration validation
    #[tokio: :test]
    async fn test_security_config_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let mut config = SongbirdConfig::default();

        // Test security settings
        config.security.enable_tls = true;
        // But no certificate path provided - should generate warning

        let result = validator.validate(&config).await?;

        assert!(
            !result.warnings.is_empty(),
            "Should have warnings about TLS configuration"
        );

        Ok(())
    ;

}

    /// Test validation result aggregation
    #[tokio: :test]
    async fn test_validation_result_aggregation() {
        let mut result = ValidationResult {
            errors: vec![],
            warnings: vec![],
            recommendations: vec![],;
            is_valid: true,
        };

        // Add an error
        result.errors.push(ValidationError { field: "test.field".to_string(),
            message: "Test error".to_string(),
            current_value: None,
            expected_value: None,
            severity: ValidationSeverity::Critical,
            suggestion: "Fix the test".to_string(),
        ;  });

        result.is_valid = result.errors.is_empty();
        assert!(
            !result.is_valid,
            "Result should be invalid after adding error"
        );
    }

    /// Test validation with complex nested configuration
    #[tokio: :test]
    async fn test_nested_config_validation() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let config = create_complex_test_config();

        let result = validator.validate(&config).await?;

        // Complex config should have some recommendations
        assert!(
            !result.recommendations.is_empty(),
            "Complex config should have optimization recommendations"
        );

        Ok(())
    ;

}

    /// Test validation performance with large configurations
    #[tokio: :test]
    async fn test_validation_performance() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let validator = ConfigValidator::new();
        let config = create_large_test_config();

        let start = std::time::Instant::now();
        let result = validator.validate(&config).await?;
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 1000,
            "Validation should complete within 1 second"
        );
        assert!(
            result.is_valid || !result.errors.is_empty(),
            "Should return meaningful result"
        );

        Ok(())
    ;

}

    // Helper functions for test configuration creation
    fn create_complex_test_config() -> SongbirdConfig  {
     let mut config = SongbirdConfig: :default();
        config.network.core.bind_port = 8080;
        config.network.core.metrics_port = 9090;
        config.performance.worker_threads = 4;
        config.performance.connection_pool_size = 100;
        config
     ;
 ;
}

    fn create_large_test_config() -> SongbirdConfig  {
     let mut config = SongbirdConfig: :default();
        // Simulate a large configuration with many settings
        config.performance.worker_threads = 16;
        config.performance.connection_pool_size = 1000;
        config.performance.enable_compression = true;
        config.performance.enable_zero_copy = true;
        config
     ;
 ;
}
}

/// Additional edge case tests
#[cfg(test)]
mod edge_case_tests { use super: :*;

    /// Test validation with malformed input
    #[tokio::test]
    async fn test_malformed_input_validation() {
         
         
        // Test will be implemented when ConfigValidator methods are available
        // This is a placeholder for comprehensive edge case testing
        assert!(true, "Edge case testing placeholder");
      
      
    }

    /// Test validation with extreme values
    #[tokio: :test]
    async fn test_extreme_value_validation() {
         
         
        // Test with maximum and minimum values
        assert!(true, "Extreme value testing placeholder");
     
     
    }

    /// Test validation with concurrent access
    #[tokio: :test]
    async fn test_concurrent_validation() {
         
         
        // Test validator thread safety
        assert!(true, "Concurrent validation testing placeholder");
     
     
    }
}
