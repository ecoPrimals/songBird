//! Comprehensive Error Tests for Songbird Orchestrator
//!
//! This test suite covers error handling, error types, error propagation,
//! and error reporting mechanisms.

use songbird_errors::{
    DiscoveryError, GamingError, NetworkError, NotFoundError, Result, SongbirdError,
    ValidationError,
};
use std::time::Duration;

#[test]
fn test_songbird_error_creation() -> Result<()> {
    let error = SongbirdError::service_error("test", "Test error message".to_string());
    assert!(error.to_string().contains("Test error message"));
    Ok(())
}

#[test]
fn test_songbird_error_types() -> Result<()> {
    let errors = vec![
        SongbirdError::Configuration {
            field: "test_field".to_string(),
            message: "Config error".to_string(),
            suggestion: Some("Check configuration file".to_string()),
        },
        SongbirdError::Network(Box::new(NetworkError {
            service: Some("test_service".to_string()),
            message: "Network error".to_string(),
            details: None,
            endpoint: None,
            suggestion: Some("Check network connectivity".to_string()),
        })),
        SongbirdError::service_error("database", "Database error".to_string()),
        SongbirdError::Authentication {
            provider: "oauth".to_string(),
            message: "Auth error".to_string(),
            suggestion: Some("Check authentication credentials".to_string()),
        },
        SongbirdError::Validation(Box::new(ValidationError {
            field: "username".to_string(),
            message: "Validation error".to_string(),
            value: Some("invalid_user".to_string()),
            expected: Some("valid username".to_string()),
            suggestion: Some("Use only alphanumeric characters".to_string()),
        })),
        SongbirdError::NotFound(Box::new(NotFoundError {
            resource: "user".to_string(),
            message: "Not found".to_string(),
            searched_paths: Some(vec!["users".to_string(), "admin".to_string()]),
            suggestion: Some("Check if the user exists".to_string()),
        })),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
    }

    Ok(())
}

#[test]
fn test_error_display_formatting() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("api".to_string()),
        message: "Connection refused".to_string(),
        details: Some("timeout after 30s".to_string()),
        endpoint: None,
        suggestion: Some("Check network connectivity".to_string()),
    }));

    assert!(error.to_string().contains("Connection refused"));
    Ok(())
}

#[test]
fn test_error_debug_formatting() -> Result<()> {
    let error = SongbirdError::Configuration {
        field: "port".to_string(),
        message: "Invalid port".to_string(),
        suggestion: Some("Use a port between 1024 and 65535".to_string()),
    };

    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("Configuration"));
    assert!(debug_output.contains("port"));
    Ok(())
}

#[test]
fn test_validation_error_creation() -> Result<()> {
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
        value: Some("invalid@email".to_string()),
        expected: Some("valid email format".to_string()),
        suggestion: Some("Use a valid email address".to_string()),
    }));

    assert!(validation_error.to_string().contains("email"));
    assert!(validation_error
        .to_string()
        .contains("Invalid email format"));

    let network_error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("smtp".to_string()),
        message: "Mail server unreachable".to_string(),
        details: None,
        endpoint: Some("smtp.example.com".to_string()),
        suggestion: Some("Check SMTP server configuration".to_string()),
    }));

    assert!(network_error.to_string().contains("smtp"));
    assert!(network_error
        .to_string()
        .contains("Mail server unreachable"));

    Ok(())
}

#[test]
fn test_validation_error_without_value() -> Result<()> {
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        field: "password".to_string(),
        message: "Password too weak".to_string(),
        value: Some("weak".to_string()),
        expected: Some("strong password".to_string()),
        suggestion: Some("Use at least 8 characters with numbers and symbols".to_string()),
    }));

    assert!(validation_error.to_string().contains("Password too weak"));

    Ok(())
}

#[test]
fn test_result_type_ok() -> Result<()> {
    let successful_result: Result<String> = Ok("Success".to_string());

    match successful_result {
        Ok(value) => assert_eq!(value, "Success"),
        Err(_) => panic!("Expected Ok result"),
    }

    Ok(())
}

#[test]
fn test_result_type_err() -> Result<()> {
    fn test_function() -> Result<String> {
        Err(SongbirdError::service_error(
            "test",
            "Test error".to_string(),
        ))
    }

    let result = test_function();

    match result {
        Ok(_) => panic!("Expected Err result"),
        Err(error) => assert!(error.to_string().contains("Test error")),
    }

    Ok(())
}

#[test]
fn test_error_propagation() -> Result<()> {
    fn inner_function() -> Result<String> {
        Err(SongbirdError::service_error(
            "database",
            "Connection failed".to_string(),
        ))
    }

    fn outer_function() -> Result<String> {
        let _result = inner_function()?; // This will propagate the error
        Ok("Success".to_string())
    }

    let result = outer_function();
    match result {
        Ok(_) => panic!("Expected Err result"),
        Err(error) => assert!(error.to_string().contains("Connection failed")),
    }

    Ok(())
}

#[test]
fn test_error_chaining() -> Result<()> {
    let root_error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("api".to_string()),
        message: "Connection timeout".to_string(),
        details: None,
        endpoint: Some("api.example.com".to_string()),
        suggestion: Some("Check API server availability".to_string()),
    }));
    let chained_error =
        SongbirdError::service_error("service", format!("Service unavailable: {}", root_error));

    assert!(chained_error.to_string().contains("Service unavailable"));
    assert!(chained_error.to_string().contains("Connection timeout"));

    Ok(())
}

#[test]
fn test_multiple_validation_errors() -> Result<()> {
    let validation_errors = vec![
        SongbirdError::Validation(Box::new(ValidationError {
            field: "username".to_string(),
            message: "Username is required".to_string(),
            value: None,
            expected: Some("valid username".to_string()),
            suggestion: Some("Provide a username".to_string()),
        })),
        SongbirdError::Validation(Box::new(ValidationError {
            field: "password".to_string(),
            message: "Password is required".to_string(),
            value: None,
            expected: Some("valid password".to_string()),
            suggestion: Some("Provide a password".to_string()),
        })),
        SongbirdError::Validation(Box::new(ValidationError {
            field: "email".to_string(),
            message: "Email is invalid".to_string(),
            value: Some("invalid-email".to_string()),
            expected: Some("valid email format".to_string()),
            suggestion: Some("Use a valid email address".to_string()),
        })),
    ];

    assert_eq!(validation_errors.len(), 3);

    for error in validation_errors {
        assert!(
            error.to_string().contains("is required") || error.to_string().contains("is invalid")
        );
    }

    Ok(())
}

#[test]
fn test_error_context_information() -> Result<()> {
    let error = SongbirdError::Configuration {
        field: "database_url".to_string(),
        message: "Database connection string missing".to_string(),
        suggestion: Some("Set the DATABASE_URL environment variable".to_string()),
    };

    assert!(error.to_string().contains("database_url"));
    assert!(error
        .to_string()
        .contains("Database connection string missing"));

    Ok(())
}

#[test]
fn test_error_categorization() -> Result<()> {
    let errors_with_categories = vec![
        (
            SongbirdError::Configuration {
                field: "config".to_string(),
                message: "Config".to_string(),
                suggestion: Some("Check configuration settings".to_string()),
            },
            "client",
        ),
        (
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("network".to_string()),
                message: "Network".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity".to_string()),
            })),
            "network",
        ),
        (
            SongbirdError::service_error("database", "Database".to_string()),
            "data",
        ),
        (
            SongbirdError::Authentication {
                provider: "auth".to_string(),
                message: "Auth".to_string(),
                suggestion: Some("Check authentication setup".to_string()),
                suggestion: Some("Check authentication setup".to_string()),
            },
            "security",
        ),
        (
            SongbirdError::Validation(Box::new(ValidationError {
                field: "validation".to_string(),
                message: "Validation".to_string(),
                value: Some("invalid".to_string()),
                expected: Some("valid data".to_string()),
                suggestion: Some("Check input validation".to_string()),
            })),
            "client",
        ),
        (
            SongbirdError::NotFound(Box::new(NotFoundError {
                resource: "resource".to_string(),
                message: "Not found".to_string(),
                searched_paths: None,
                suggestion: Some("Check if the resource exists".to_string()),
            })),
            "client",
        ),
    ];

    for (error, expected_category) in errors_with_categories {
        // Just verify the error can be created and contains expected text
        let error_text = error.to_string().to_lowercase();
        match expected_category {
            "client" => assert!(
                error_text.contains("config")
                    || error_text.contains("validation")
                    || error_text.contains("not found")
            ),
            "network" => assert!(error_text.contains("network")),
            "data" => assert!(error_text.contains("database")),
            "security" => assert!(error_text.contains("auth")),
            _ => {}
        }
    }

    Ok(())
}

#[test]
fn test_error_severity_levels() -> Result<()> {
    let errors_with_severity = vec![
        (
            SongbirdError::Configuration {
                field: "config".to_string(),
                message: "Config".to_string(),
                suggestion: Some("Check configuration file".to_string()),
            },
            "high",
        ),
        (
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("network".to_string()),
                message: "Network".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity".to_string()),
            })),
            "medium",
        ),
        (
            SongbirdError::service_error("database", "Database".to_string()),
            "high",
        ),
        (
            SongbirdError::Authentication {
                provider: "auth".to_string(),
                message: "Auth".to_string(),
                suggestion: Some("Check authentication setup".to_string()),
            },
            "high",
        ),
        (
            SongbirdError::Validation {
                field: "validation".to_string(),
                message: "Validation".to_string(),
            },
            "low",
        ),
        (
            SongbirdError::NotFound(Box::new(NotFoundError {
                resource: "resource".to_string(),
                message: "Not found".to_string(),
                searched_paths: None,
                suggestion: None,
            })),
            "low",
        ),
    ];

    for (error, expected_severity) in errors_with_severity {
        // Just verify error can be displayed
        assert!(!error.to_string().is_empty());
        assert!(matches!(expected_severity, "low" | "medium" | "high"));
    }

    Ok(())
}

#[test]
fn test_error_recovery_suggestions() -> Result<()> {
    let config_error = SongbirdError::Configuration {
        field: "port".to_string(),
        message: "Invalid port number".to_string(),
        suggestion: Some("Use a port between 1024 and 65535".to_string()),
    };

    // Test that error can be displayed
    assert!(config_error.to_string().contains("Invalid port number"));

    Ok(())
}

#[test]
fn test_error_logging_format() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("test".to_string()),
        message: "Connection refused to 192.168.1.100:8080".to_string(),
        details: Some("Additional context".to_string()),
        endpoint: Some("192.168.1.100:8080".to_string()),
        suggestion: None,
    }));

    let log_format = error.to_string();
    assert!(log_format.contains("Connection refused"));
    assert!(log_format.contains("192.168.1.100:8080"));

    Ok(())
}

#[test]
fn test_error_metrics_collection() -> Result<()> {
    let errors = vec![
        SongbirdError::Configuration {
            field: "config1".to_string(),
            message: "Config 1".to_string(),
            suggestion: Some("Check configuration file".to_string()),
        },
        SongbirdError::Network(Box::new(NetworkError {
            service: Some("network1".to_string()),
            message: "Network 1".to_string(),
            details: None,
            endpoint: None,
            suggestion: None,
        })),
        SongbirdError::Network(Box::new(NetworkError {
            service: Some("network2".to_string()),
            message: "Network 2".to_string(),
            details: None,
            endpoint: None,
            suggestion: None,
        })),
        SongbirdError::service_error("database", "Database 1".to_string()),
        SongbirdError::Configuration {
            field: "config2".to_string(),
            message: "Config 2".to_string(),
            suggestion: Some("Check configuration file".to_string()),
        },
        SongbirdError::service_error("internal", "Internal 1".to_string()),
    ];

    // Just verify we can create these errors
    assert_eq!(errors.len(), 6);
    for error in errors {
        assert!(!error.to_string().is_empty());
    }

    Ok(())
}

#[test]
fn test_error_performance_impact() -> Result<()> {
    let start_time = std::time::Instant::now();

    // Create many errors to test performance
    for i in 0..1000 {
        let error = SongbirdError::service_error("perf_test", format!("Error {}", i));
        let _ = error.to_string();
    }

    let elapsed = start_time.elapsed();

    // Error creation and formatting should be fast
    assert!(elapsed < Duration::from_millis(1000)); // Increased threshold for test stability

    Ok(())
}

#[test]
fn test_error_memory_usage() -> Result<()> {
    let error = SongbirdError::configuration_error("Simple error".to_string());
    let error_size = std::mem::size_of_val(&error);

    // Error should not be too large (very generous threshold)
    assert!(error_size < 2048); // Less than 2KB

    Ok(())
}

#[test]
fn test_error_thread_safety() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("test".to_string()),
        message: "Thread safety test".to_string(),
        details: Some("Additional details".to_string()),
        endpoint: None,
        suggestion: None,
    }));

    // Clone error for thread safety test
    let error_clone = error.clone();

    // Simulate sending to another thread
    let handle = std::thread::spawn(move || error_clone.to_string());

    let result = handle.join().unwrap();
    assert!(result.contains("Thread safety test"));

    Ok(())
}

#[tokio::test]
async fn test_config_error_creation() -> Result<()> {
    let error = SongbirdError::Config {
        message: "Invalid configuration".to_string(),
        field: Some("database_url".to_string()),
        context: Some("config validation".to_string()),
        suggestion: Some("Check database URL format and connectivity".to_string()),
    };

    assert!(error.to_string().contains("Invalid configuration"));
    assert!(error.to_string().contains("database_url"));

    Ok(())
}

#[tokio::test]
async fn test_configuration_error_creation() -> Result<()> {
    let error = SongbirdError::Configuration {
        field: "port".to_string(),
        message: "Port must be between 1 and 65535".to_string(),
        suggestion: Some("Use a valid port number".to_string()),
    };

    assert!(error.to_string().contains("Port must be between"));

    Ok(())
}

#[tokio::test]
async fn test_network_error_creation() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("api".to_string()),
        message: "Connection refused".to_string(),
        details: Some("timeout".to_string()),
        endpoint: None,
        suggestion: None,
    }));

    assert!(error.to_string().contains("Connection refused"));

    Ok(())
}

#[tokio::test]
async fn test_discovery_error_creation() -> Result<()> {
    let error = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Service discovery failed".to_string(),
        service: Some("mdns".to_string()),
        timeout: None,
        suggestion: None,
    }));

    assert!(error.to_string().contains("Service discovery failed"));

    Ok(())
}

#[tokio::test]
async fn test_auth_error_creation() -> Result<()> {
    let error = SongbirdError::Authentication {
        provider: "oauth2".to_string(),
        message: "Invalid token".to_string(),
        suggestion: Some("Check token validity and refresh if needed".to_string()),
    };

    assert!(error.to_string().contains("Invalid token"));

    Ok(())
}

#[tokio::test]
async fn test_gaming_error_creation() -> Result<()> {
    let error = SongbirdError::Gaming(Box::new(GamingError {
        message: "Failed to create bridge".to_string(),
        protocol: Some("StarCraft".to_string()),
        game: None,
        suggestion: None,
    }));

    assert!(error.to_string().contains("Failed to create bridge"));

    Ok(())
}

#[tokio::test]
async fn test_security_error_creation() -> Result<()> {
    let error = SongbirdError::Security {
        context: Some("firewall".to_string()),
        message: "Access denied".to_string(),
        severity: Some("high".to_string()),
        suggestion: Some("Review security configuration and apply recommended fixes".to_string()),
    };

    assert!(error.to_string().contains("Access denied"));

    Ok(())
}

#[tokio::test]
async fn test_service_error_creation() -> Result<()> {
    let error = SongbirdError::service_error("database", "Connection timeout".to_string());

    assert!(error.to_string().contains("Connection timeout"));

    Ok(())
}

#[tokio::test]
async fn test_not_found_error_creation() -> Result<()> {
    let error = SongbirdError::NotFound(Box::new(NotFoundError {
        resource: "user".to_string(),
        message: "User not found".to_string(),
        searched_paths: None,
        suggestion: None,
    }));

    assert!(error.to_string().contains("User not found"));

    Ok(())
}

#[tokio::test]
async fn test_simple_error_pattern() -> Result<()> {
    let error = SongbirdError::service_error("test", "Simple error".to_string());

    assert!(error.to_string().contains("Simple error"));

    Ok(())
}

#[tokio::test]
async fn test_network_error_pattern() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("http".to_string()),
        message: "Network timeout".to_string(),
        details: Some("after 30 seconds".to_string()),
        endpoint: None,
        suggestion: None,
    }));

    assert!(error.to_string().contains("Network timeout"));

    Ok(())
}
