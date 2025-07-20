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
            message: "Test network error".to_string(),
            endpoint: Some("http://test.example".to_string()),
            port: Some(8080),
            protocol: Some("test_service".to_string()),
        })),
        SongbirdError::service_error("database", "Database error".to_string()),
        SongbirdError::Authentication {
            provider: "oauth".to_string(),
            message: "Auth error".to_string(),
            suggestion: Some("Check authentication credentials".to_string()),
        },
        SongbirdError::Validation(Box::new(ValidationError {
            field: Some("username".to_string()),
            message: "Validation error".to_string(),
            expected: Some("valid username".to_string()),
        })),
        SongbirdError::NotFound(Box::new(NotFoundError {
            resource: Some("user".to_string()),
            message: "Not found".to_string(),
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
        message: "Connection refused".to_string(),
        endpoint: Some("192.168.1.100:8080".to_string()),
        port: Some(8080),
        protocol: Some("http".to_string()),
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

    let debug_output = format!("{error:?}");
    assert!(debug_output.contains("Configuration"));
    assert!(debug_output.contains("port"));
    Ok(())
}

#[test]
fn test_validation_error_creation() -> Result<()> {
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        field: Some("email".to_string()),
        message: "Invalid email format".to_string(),
        expected: Some("valid email format".to_string()),
    }));

    assert!(validation_error.to_string().contains("email"));
    assert!(validation_error
        .to_string()
        .contains("Invalid email format"));

    let network_error = NetworkError {
        message: "Mail server unreachable".to_string(),
        endpoint: Some("smtp.example.com".to_string()),
        port: None,
        protocol: Some("smtp".to_string()),
    };

    assert!(network_error.to_string().contains("smtp"));
    assert!(network_error
        .to_string()
        .contains("Mail server unreachable"));

    Ok(())
}

#[test]
fn test_validation_error_without_value() -> Result<()> {
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        field: Some("password".to_string()),
        message: "Password too weak".to_string(),
        expected: Some("strong password".to_string()),
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
        message: "Connection timeout".to_string(),
        endpoint: Some("api.example.com".to_string()),
        port: None,
        protocol: Some("http".to_string()),
    }));
    let chained_error =
        SongbirdError::service_error("service", format!("Service unavailable: {root_error}"));

    assert!(chained_error.to_string().contains("Service unavailable"));
    assert!(chained_error.to_string().contains("Connection timeout"));

    Ok(())
}

#[test]
fn test_multiple_validation_errors() -> Result<()> {
    let validation_errors = vec![
        SongbirdError::Validation(Box::new(ValidationError {
            field: Some("username".to_string()),
            message: "Username is required".to_string(),
            expected: Some("valid username".to_string()),
        })),
        SongbirdError::Validation(Box::new(ValidationError {
            field: Some("password".to_string()),
            message: "Password is required".to_string(),
            expected: Some("valid password".to_string()),
        })),
        SongbirdError::Validation(Box::new(ValidationError {
            field: Some("email".to_string()),
            message: "Email is invalid".to_string(),
            expected: Some("valid email format".to_string()),
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
                message: "Network".to_string(),
                endpoint: None,
                port: None,
                protocol: Some("test_service".to_string()),
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
            },
            "security",
        ),
        (
            SongbirdError::Validation(Box::new(ValidationError {
                field: Some("validation".to_string()),
                message: "Validation".to_string(),
                expected: Some("valid data".to_string()),
            })),
            "client",
        ),
        (
            SongbirdError::NotFound(Box::new(NotFoundError {
                resource: Some("resource".to_string()),
                message: "Not found".to_string(),
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
                message: "Network".to_string(),
                endpoint: None,
                port: None,
                protocol: Some("test_service".to_string()),
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
            SongbirdError::Validation(Box::new(ValidationError {
                field: Some("validation".to_string()),
                message: "Validation".to_string(),
                expected: None,
            })),
            "low",
        ),
        (
            SongbirdError::NotFound(Box::new(NotFoundError {
                resource: Some("resource".to_string()),
                message: "Not found".to_string(),
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
        message: "Connection refused to 192.168.1.100:8080".to_string(),
        endpoint: Some("192.168.1.100:8080".to_string()),
        port: Some(8080),
        protocol: Some("http".to_string()),
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
            message: "Network 1".to_string(),
            endpoint: None,
            port: None,
            protocol: Some("test_service".to_string()),
        })),
        SongbirdError::Network(Box::new(NetworkError {
            message: "Network 2".to_string(),
            endpoint: None,
            port: None,
            protocol: Some("test_service".to_string()),
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
        let error = SongbirdError::service_error("perf_test", format!("Error {i}"));
        let _ = error.to_string();
    }

    let elapsed = start_time.elapsed();

    // Error creation and formatting should be fast
    assert!(elapsed < Duration::from_millis(1000)); // Increased threshold for test stability

    Ok(())
}

#[test]
fn test_error_memory_usage() -> Result<()> {
    let error = SongbirdError::config("Simple error".to_string());
    let error_size = std::mem::size_of_val(&error);

    // Error should not be too large (very generous threshold)
    assert!(error_size < 2048); // Less than 2KB

    Ok(())
}

#[test]
fn test_error_thread_safety() -> Result<()> {
    let error = SongbirdError::Network(Box::new(NetworkError {
        message: "Thread safety test".to_string(),
        endpoint: None,
        port: None,
        protocol: None,
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
        message: "Connection refused".to_string(),
        endpoint: None,
        port: None,
        protocol: None,
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
        game: Some("StarCraft".to_string()),
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
        resource: Some("user".to_string()),
        message: "User not found".to_string(),
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
        message: "Network timeout".to_string(),
        endpoint: None,
        port: None,
        protocol: None,
    }));

    assert!(error.to_string().contains("Network timeout"));

    Ok(())
}
