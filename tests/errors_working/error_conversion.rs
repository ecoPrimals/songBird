//! Error conversion and helper function tests

use songbird_errors::{SongbirdResult, SongbirdError};

#[test]
fn test_songbird_error_helper_methods() {
    // Test config error creation
    let config_err = SongbirdError::config_error("database_url", "Invalid URL");
    assert!(matches!(config_err, SongbirdError::Config { .. }));

    // Test network error creation
    let network_err = SongbirdError::network_error("Connection failed");
    assert!(matches!(network_err, SongbirdError::Network(_)));

    // Test communication error creation
    let comm_err = SongbirdError::communication_error("Message send failed");
    let error_string = format!("{comm_err}");
    assert!(error_string.contains("Message send failed"));
}

#[test]
fn test_songbird_error_from_str() {
    let error: SongbirdError = "Test error message".into();
    let error_string = format!("{error}");
    assert!(error_string.contains("Test error message"));
}

#[test]
fn test_songbird_error_from_string() {
    let error: SongbirdError = String::from("String error message").into();
    let error_string = format!("{error}");
    assert!(error_string.contains("String error message"));
}

#[test]
fn test_songbird_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error: SongbirdError = io_error.into();
    
    assert!(matches!(songbird_error, SongbirdError::Io(_)));
    
    let error_string = format!("{songbird_error}");
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_songbird_error_from_addr_parse_error() {
    let addr_error = "invalid_address:port".parse::<std::net::SocketAddr>().unwrap_err();
    let songbird_error: SongbirdError = addr_error.into();
    
    let error_string = format!("{songbird_error}");
    assert!(error_string.contains("invalid address"));
}

#[test]
fn test_songbird_error_from_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let songbird_error: SongbirdError = json_error.into();
    
    let error_string = format!("{songbird_error}");
    assert!(error_string.contains("expected"));
}

#[test]
fn test_result_type_ok() {
    let result: SongbirdResult<String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.ok_or_else(|| SongbirdError::internal(format!("Operation failed: {:?}", e)))?, "success");
}

#[test]
fn test_result_type_err() {
    let result: SongbirdResult<String> = Err(SongbirdError::config_error("test", "Failed"));
    assert!(result.is_err());
}

#[test]
fn test_songbird_error_conversion_chain() {
    // Test multiple conversion scenarios
    let original_io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let songbird_err: SongbirdError = original_io.into();
    
    match songbird_err {
        SongbirdError::Io(_) => {
            // Correct conversion
        }
        _ => panic!("Unexpected error type"),
    }
}

#[test]
fn test_error_display_formatting() {
    let error = SongbirdError::config_error("timeout", "Invalid timeout value");
    let display = format!("{error}");
    let debug = format!("{:?}", error);
    
    assert!(display.contains("Invalid timeout value"));
    assert!(!debug.is_empty());
    assert_ne!(display, debug); // Display and Debug should be different
}

#[test]
fn test_error_source_chain() {
    use std::error::Error;
    
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Original error");
    let songbird_error: SongbirdError = io_error.into();
    
    // Test error source chain
    let error_ref: &dyn Error = &songbird_error;
    assert!(error_ref.source().is_some());
}

#[test]
fn test_error_downcast() {
    use std::error::Error;
    
    let error = SongbirdError::config_error("field", "message");
    let error_box: Box<dyn Error> = Box::new(error);
    
    // Attempt to downcast back to SongbirdError
    let downcasted = error_box.downcast::<SongbirdError>();
    assert!(downcasted.is_ok());
}

#[test]
fn test_error_send_sync_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    
    assert_send::<SongbirdError>();
    assert_sync::<SongbirdError>();
}

#[test]
fn test_error_clone_eq() {
    let error1 = SongbirdError::config_error("field", "message");
    let error2 = error1.clone();
    
    // Errors should be equal after cloning
    assert_eq!(format!("{error1}"), format!("{error2}"));
}

#[test]
fn test_error_helper_convenience_methods() {
    // Test all helper methods are working
    let auth_err = SongbirdError::authentication_error("provider", "message");
    let security_err = SongbirdError::security_error("message");
    let network_err = SongbirdError::network_error("message");
    let comm_err = SongbirdError::communication_error("message");
    
    assert!(matches!(auth_err, SongbirdError::Authentication { .. }));
    assert!(!format!("{security_err}").is_empty());
    assert!(matches!(network_err, SongbirdError::Network(_)));
    assert!(!format!("{comm_err}").is_empty());
} 