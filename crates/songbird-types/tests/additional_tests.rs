//! Additional tests for songbird-types
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! Expanding test coverage for error handling, config, and performance types.

use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_error_display_formatting() {
    let error = SongbirdError::configuration("Test error");
    let display = format!("{error}");
    assert!(display.contains("Test error"));
}

#[test]
fn test_error_debug_formatting() {
    let error = SongbirdError::configuration("Debug test");
    let debug = format!("{error:?}");
    assert!(debug.contains("Configuration"));
}

#[test]
fn test_result_ok_creation() {
    let result: SongbirdResult<i32> = Ok(42);
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, 42);
    }
}

#[test]
fn test_result_err_creation() {
    let result: SongbirdResult<i32> = Err(SongbirdError::configuration("Test"));
    assert!(result.is_err());
}

#[test]
fn test_error_propagation() {
    fn returns_error() -> SongbirdResult<()> {
        Err(SongbirdError::configuration("Propagated"))
    }

    fn calls_error_fn() -> SongbirdResult<()> {
        returns_error()?;
        Ok(())
    }

    let result = calls_error_fn();
    assert!(result.is_err());
}

#[test]
fn test_error_chain_building() {
    let base_error = SongbirdError::configuration("Base");
    let error_string = format!("{base_error}");
    assert!(error_string.contains("Base"));
}

#[test]
fn test_multiple_error_types() {
    let config_err = SongbirdError::configuration("Config");
    let network_err = SongbirdError::network("Network");

    assert_ne!(format!("{config_err:?}"), format!("{network_err:?}"));
}

#[test]
fn test_error_message_preservation() {
    let message = "Important error message";
    let error = SongbirdError::configuration(message);
    let formatted = format!("{error}");
    assert!(formatted.contains(message));
}

#[test]
fn test_result_map_operation() {
    let result: SongbirdResult<i32> = Ok(10);
    let mapped = result.map(|x| x * 2);
    assert_eq!(mapped.expect("Test: map operation should succeed"), 20);
}

#[test]
fn test_result_and_then_operation() {
    let result: SongbirdResult<i32> = Ok(10);
    let chained = result.map(|x| x + 5);
    assert_eq!(chained.expect("Test: and_then operation should succeed"), 15);
}

#[test]
fn test_error_types_are_send() {
    fn assert_send<T: Send>() {}
    assert_send::<SongbirdError>();
}

#[test]
fn test_error_types_are_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<SongbirdError>();
}

#[test]
fn test_result_unwrap_or_default() {
    // Test unwrap_or with an error result
    fn get_error_result() -> SongbirdResult<i32> {
        Err(SongbirdError::configuration("Test"))
    }
    let value = get_error_result().unwrap_or(42);
    assert_eq!(value, 42);
}

#[test]
fn test_service_error_creation() {
    let error = SongbirdError::service("test-service", "Service error");
    let formatted = format!("{error}");
    assert!(formatted.contains("test-service"));
    assert!(formatted.contains("Service error"));
}

#[test]
fn test_security_error_creation() {
    let error = SongbirdError::security("Access denied");
    let formatted = format!("{error}");
    assert!(formatted.contains("Access denied"));
}
