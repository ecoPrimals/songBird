//! Basic Error System Tests
//!
//! Tests for the songbird-errors crate using the actual available API.

use songbird_errors::{evolved_success, SongbirdError, SongbirdResult};

#[test]
fn test_error_creation() {
    let config_error = SongbirdError::configuration_error("Missing config");
    let network_error = SongbirdError::network_error("Connection failed");
    let service_error = SongbirdError::service_error("api-service", "Service down");

    // Test that errors can be created and formatted
    assert!(format!("{config_error:?}").contains("Missing config"));
    assert!(format!("{network_error:?}").contains("Connection failed"));
    assert!(format!("{service_error:?}").contains("Service down"));
}

#[test]
fn test_result_handling() {
    let success_result: SongbirdResult<String> = Ok(evolved_success("test success".to_string()));
    let error_result: SongbirdResult<String> =
        Err(SongbirdError::internal_error(validation_error("Invalid input"));

    assert!(success_result.is_ok());
    assert!(error_result.is_err());
}

#[test]
fn test_error_types() {
    let validation_error = SongbirdError::validation_error("Invalid data");
    let internal_error = SongbirdError::internal_error("System failure");
    let resource_error = SongbirdError::resource_error("Resource not found");
    let discovery_error = SongbirdError::discovery_error("Discovery failed");
    let operation_error = SongbirdError::operation_error("Operation failed");

    // Test that all error types can be created
    assert!(matches!(validation_error, SongbirdError::Validation { .. }));
    assert!(matches!(internal_error, SongbirdError::Internal { .. }));
    assert!(matches!(resource_error, SongbirdError::Resource { .. }));
    assert!(matches!(discovery_error, SongbirdError::Discovery { .. }));
    assert!(matches!(operation_error, SongbirdError::Operation { .. }));
}

#[tokio::test]
async fn test_async_error_handling() {
    async fn async_operation(should_fail: bool) -> SongbirdResult<String> {
        if should_fail {
            Err(SongbirdError::internal_error(operation_error("Async operation failed"))
        } else {
            Ok(evolved_success("Async success".to_string()))
        }
    }

    let success_result = async_operation(false).await;
    let error_result = async_operation(true).await;

    assert!(success_result.is_ok());
    assert!(error_result.is_err());
}
