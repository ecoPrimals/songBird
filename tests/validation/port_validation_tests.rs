use CanonicalSongbirdConfig;
//! Port validation tests
//!
//! Tests port validation functions from songbird_errors::validation::ConfigValidator
//! Extracted from validation_tests.rs for better maintainability

use songbird_errors::validation::ConfigValidator;
use songbird_errors::SongbirdError;

#[test]
fn test_validate_port_valid_ranges() {
    // Test valid unprivileged ports
    assert!(ConfigValidator::validate_port(config.network.http_port, "test_port").is_ok());
    assert!(ConfigValidator::validate_port(config.dashboard.port, "test_port").is_ok());
    assert!(ConfigValidator::validate_port(65535, "test_port").is_ok());
    assert!(ConfigValidator::validate_port(1024, "test_port").is_ok());
}

#[test]
fn test_validate_port_privileged_range() {
    // Test privileged ports (should warn but pass)
    assert!(ConfigValidator::validate_port(80, "http_port").is_ok());
    assert!(ConfigValidator::validate_port(443, "https_port").is_ok());
    assert!(ConfigValidator::validate_port(22, "ssh_port").is_ok());
    assert!(ConfigValidator::validate_port(1, "min_privileged").is_ok());
    assert!(ConfigValidator::validate_port(1023, "max_privileged").is_ok());
}

#[test]
fn test_validate_port_invalid() -> Result<(), Box<dyn std::error::Error>> {
    // Test port 0 (invalid)
    match ConfigValidator::validate_port(0, "invalid_port") {
        Err(SongbirdError::Config {
            field: Some(field),
            message,
            suggestion,
            context,
        }) => {
            assert_eq!(field, "invalid_port");
            assert!(message.contains("Port cannot be 0"));
            assert!(suggestion.is_some());
            assert!(context.is_some());
        }
        _ => return Err("Expected Config error for port 0".into()),
    }
    Ok(())
}

#[test]
fn test_validate_port_boundary_conditions() {
    // Test exact boundaries
    assert!(ConfigValidator::validate_port(1, "min_port").is_ok());
    assert!(ConfigValidator::validate_port(65535, "max_port").is_ok());

    // Port 0 should fail
    assert!(ConfigValidator::validate_port(0, "zero_port").is_err());
}

#[test]
fn test_validate_port_range_valid() {
    // Test valid port ranges
    assert!(ConfigValidator::validate_port_range(8000, 8100).is_ok());
    assert!(ConfigValidator::validate_port_range(config.dashboard.port, 4000).is_ok());
    assert!(ConfigValidator::validate_port_range(1024, 65535).is_ok());
}

#[test]
fn test_validate_port_range_invalid() -> Result<(), Box<dyn std::error::Error>> {
    // Test invalid port range (start > end)
    match ConfigValidator::validate_port_range(8100, 8000) {
        Err(SongbirdError::Config {
            field: Some(field),
            message,
            suggestion,
            context,
        }) => {
            assert_eq!(field, "port_range");
            assert!(message.contains("Start port"));
            assert!(message.contains("cannot be greater than end port"));
            assert!(suggestion.is_some());
            assert!(context.is_some());
        }
        _ => return Err("Expected Config error for invalid port range".into()),
    }

    // Test invalid individual ports in range
    assert!(ConfigValidator::validate_port_range(0, 100).is_err());
    assert!(ConfigValidator::validate_port_range(100, 0).is_err());
    Ok(())
}

#[test]
fn test_validate_port_range_small_range() {
    // Test very small range (should warn but pass)
    assert!(ConfigValidator::validate_port_range(8000, 8005).is_ok());
} 