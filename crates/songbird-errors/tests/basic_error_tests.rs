//! Tests for the songbird-errors crate using the actual available API.

use songbird_errors::{SongbirdError, SongbirdResult};

#[test]
fn test_error_creation() {
    let config_error = SongbirdError::configuration("Missing config");
    let network_error = SongbirdError::network("Connection failed");
    let service_error = SongbirdError::service("api-service", "Service down");

    // Test error messages
    assert!(config_error.to_string().contains("Missing config"));
    assert!(network_error.to_string().contains("Connection failed"));
    assert!(service_error.to_string().contains("Service down"));
}

#[test]
fn test_result_handling() {
    let success_result: SongbirdResult<String> = Ok("test success".to_string());
    let error_result: SongbirdResult<String> = Err(SongbirdError::configuration("Invalid input"));

    assert!(success_result.is_ok());
    assert!(error_result.is_err());
}

#[test]
fn test_error_types() {
    let config_error = SongbirdError::configuration("Missing config");
    let network_error = SongbirdError::network("Connection failed");
    let service_error = SongbirdError::service("api-service", "Service down");
    let security_error = SongbirdError::security("Authentication failed");

    // Test that errors are created correctly
    assert!(config_error.to_string().contains("Missing config"));
    assert!(network_error.to_string().contains("Connection failed"));
    assert!(service_error.to_string().contains("Service down"));
    assert!(security_error.to_string().contains("Authentication failed"));
}

#[test]
fn test_error_handling() {
    fn operation(should_fail: bool) -> SongbirdResult<String> {
        if should_fail {
            Err(SongbirdError::network("Operation failed"))
        } else {
            Ok("Success".to_string())
        }
    }

    let success_result = operation(false);
    let error_result = operation(true);

    assert!(success_result.is_ok());
    assert!(error_result.is_err());

    if let Err(e) = error_result {
        assert!(e.to_string().contains("Operation failed"));
    }
}
