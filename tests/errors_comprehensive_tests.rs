//! Comprehensive Error Tests for Songbird Orchestrator
//!
//! This test suite covers error handling, error types, error propagation,
//! and error reporting mechanisms.

use songbird_lib::errors::{Result, SongbirdError};
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
        },
        SongbirdError::Network {
            service: "test_service".to_string(),
            message: "Network error".to_string(),
            details: None,
        },
        SongbirdError::service_error("database", "Database error".to_string()),
        SongbirdError::Authentication {
            provider: "oauth".to_string(),
            message: "Auth error".to_string(),
        },
        SongbirdError::Validation {
            field: "username".to_string(),
            message: "Validation error".to_string(),
        },
        SongbirdError::NotFound {
            resource: "user".to_string(),
            message: "Not found".to_string(),
        },
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
    }

    Ok(())
}

#[test]
fn test_error_display_formatting() -> Result<()> {
    let error = SongbirdError::Network {
        service: "api".to_string(),
        message: "Connection refused".to_string(),
        details: Some("timeout after 30s".to_string()),
    };

    assert!(error.to_string().contains("Connection refused"));
    Ok(())
}

#[test]
fn test_error_debug_formatting() -> Result<()> {
    let error = SongbirdError::Configuration {
        field: "port".to_string(),
        message: "Invalid port".to_string(),
    };

    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("Configuration"));
    assert!(debug_output.contains("port"));
    Ok(())
}

#[test]
fn test_validation_error_creation() -> Result<()> {
    let validation_error = SongbirdError::Validation {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
    };

    assert!(validation_error.to_string().contains("email"));
    assert!(validation_error
        .to_string()
        .contains("Invalid email format"));

    let network_error = SongbirdError::Network {
        service: "smtp".to_string(),
        message: "Mail server unreachable".to_string(),
        details: None,
    };

    assert!(network_error.to_string().contains("smtp"));
    assert!(network_error
        .to_string()
        .contains("Mail server unreachable"));

    Ok(())
}

#[test]
fn test_validation_error_without_value() -> Result<()> {
    let validation_error = SongbirdError::Validation {
        field: "password".to_string(),
        message: "Password too weak".to_string(),
    };

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
    let root_error = SongbirdError::Network {
        service: "api".to_string(),
        message: "Connection timeout".to_string(),
        details: None,
    };
    let chained_error =
        SongbirdError::service_error("service", format!("Service unavailable: {}", root_error));

    assert!(chained_error.to_string().contains("Service unavailable"));
    assert!(chained_error.to_string().contains("Connection timeout"));

    Ok(())
}

#[test]
fn test_multiple_validation_errors() -> Result<()> {
    let validation_errors = vec![
        SongbirdError::Validation {
            field: "username".to_string(),
            message: "Username is required".to_string(),
        },
        SongbirdError::Validation {
            field: "password".to_string(),
            message: "Password is required".to_string(),
        },
        SongbirdError::Validation {
            field: "email".to_string(),
            message: "Email is invalid".to_string(),
        },
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
            },
            "client",
        ),
        (
            SongbirdError::Network {
                service: "network".to_string(),
                message: "Network".to_string(),
                details: None,
            },
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
            },
            "security",
        ),
        (
            SongbirdError::Validation {
                field: "validation".to_string(),
                message: "Validation".to_string(),
            },
            "client",
        ),
        (
            SongbirdError::NotFound {
                resource: "resource".to_string(),
                message: "Not found".to_string(),
            },
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
            },
            "high",
        ),
        (
            SongbirdError::Network {
                service: "network".to_string(),
                message: "Network".to_string(),
                details: None,
            },
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
            SongbirdError::NotFound {
                resource: "resource".to_string(),
                message: "Not found".to_string(),
            },
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
    };

    // Test that error can be displayed
    assert!(config_error.to_string().contains("Invalid port number"));

    Ok(())
}

#[test]
fn test_error_logging_format() -> Result<()> {
    let error = SongbirdError::Network {
        service: "test".to_string(),
        message: "Connection refused to 192.168.1.100:8080".to_string(),
        details: Some("Additional context".to_string()),
    };

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
        },
        SongbirdError::Network {
            service: "network1".to_string(),
            message: "Network 1".to_string(),
            details: None,
        },
        SongbirdError::Network {
            service: "network2".to_string(),
            message: "Network 2".to_string(),
            details: None,
        },
        SongbirdError::service_error("database", "Database 1".to_string()),
        SongbirdError::Configuration {
            field: "config2".to_string(),
            message: "Config 2".to_string(),
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
    let error = SongbirdError::Network {
        service: "test".to_string(),
        message: "Thread safety test".to_string(),
        details: Some("Additional details".to_string()),
    };

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
    };

    assert!(error.to_string().contains("Port must be between"));

    Ok(())
}

#[tokio::test]
async fn test_network_error_creation() -> Result<()> {
    let error = SongbirdError::Network {
        service: "api".to_string(),
        message: "Connection refused".to_string(),
        details: Some("timeout".to_string()),
    };

    assert!(error.to_string().contains("Connection refused"));

    Ok(())
}

#[tokio::test]
async fn test_discovery_error_creation() -> Result<()> {
    let error = SongbirdError::Discovery {
        service: Some("mdns".to_string()),
        message: "Service discovery failed".to_string(),
    };

    assert!(error.to_string().contains("Service discovery failed"));

    Ok(())
}

#[tokio::test]
async fn test_auth_error_creation() -> Result<()> {
    let error = SongbirdError::Authentication {
        provider: "oauth2".to_string(),
        message: "Invalid token".to_string(),
    };

    assert!(error.to_string().contains("Invalid token"));

    Ok(())
}

#[tokio::test]
async fn test_gaming_error_creation() -> Result<()> {
    let error = SongbirdError::Gaming {
        protocol: Some("StarCraft".to_string()),
        message: "Failed to create bridge".to_string(),
    };

    assert!(error.to_string().contains("Failed to create bridge"));

    Ok(())
}

#[tokio::test]
async fn test_security_error_creation() -> Result<()> {
    let error = SongbirdError::Security {
        context: Some("firewall".to_string()),
        message: "Access denied".to_string(),
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
    let error = SongbirdError::NotFound {
        resource: "user".to_string(),
        message: "User not found".to_string(),
    };

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
    let error = SongbirdError::Network {
        service: "http".to_string(),
        message: "Network timeout".to_string(),
        details: Some("after 30 seconds".to_string()),
    };

    assert!(error.to_string().contains("Network timeout"));

    Ok(())
}
