//! Error trait implementation tests

use songbird_errors::SongbirdError;
use std::error::Error;

#[test]
fn test_error_std_error_trait() {
    let error = SongbirdError::Config {
        field: Some("test".to_string()),
        message: "Test error".to_string(),
        context: Some("Test context".to_string()),
        suggestion: Some("Check the test configuration".to_string()),
    };

    // Test Error trait
    let error_trait: &dyn Error = &error;
    assert!(!error_trait.to_string().is_empty());
    
    // Test source chain (should be None for this error type)
    assert!(error_trait.source().is_none());
}

#[test]
fn test_error_send_trait() {
    fn assert_send<T: Send>(_: T) {}
    
    let error = SongbirdError::config_error("field", "message");
    assert_send(error);
}

#[test]
fn test_error_sync_trait() {
    fn assert_sync<T: Sync>(_: T) {}
    
    let error = SongbirdError::config_error("field", "message");
    assert_sync(error);
}

#[test]
fn test_error_display_trait() {
    let error = SongbirdError::config_error("database_url", "Invalid URL format");
    let display_output = format!("{error}");
    
    assert!(!display_output.is_empty());
    assert!(display_output.contains("Invalid URL format"));
    assert!(display_output.contains("database_url"));
}

#[test]
fn test_error_debug_trait() {
    let error = SongbirdError::config_error("field", "message");
    let debug_output = format!("{:?}", error);
    
    assert!(!debug_output.is_empty());
    // Debug output should be more verbose than Display
    assert!(debug_output.len() >= format!("{error}").len());
}

#[test]
fn test_error_clone_trait() {
    let original = SongbirdError::config_error("field", "message");
    let cloned = original.clone();
    
    // Both errors should format the same way
    assert_eq!(format!("{original}"), format!("{cloned}"));
    assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
}

#[test]
fn test_error_partial_eq_trait() {
    let error1 = SongbirdError::config_error("field", "message");
    let error2 = SongbirdError::config_error("field", "message");
    let error3 = SongbirdError::config_error("other_field", "message");
    
    // Same errors should be equal
    assert_eq!(format!("{error1}"), format!("{error2}"));
    
    // Different errors should not be equal
    assert_ne!(format!("{error1}"), format!("{error3}"));
}

#[test]
fn test_error_from_trait_io() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error: SongbirdError = io_error.into();
    
    assert!(matches!(songbird_error, SongbirdError::Io(_)));
}

#[test]
fn test_error_from_trait_string() {
    let string_error = String::from("String error message");
    let songbird_error: SongbirdError = string_error.into();
    
    let display = format!("{songbird_error}");
    assert!(display.contains("String error message"));
}

#[test]
fn test_error_from_trait_str() {
    let str_error = "Static str error message";
    let songbird_error: SongbirdError = str_error.into();
    
    let display = format!("{songbird_error}");
    assert!(display.contains("Static str error message"));
}

#[test]
fn test_error_downcast_support() {
    let error = SongbirdError::config_error("field", "message");
    let boxed_error: Box<dyn Error> = Box::new(error);
    
    // Should be able to downcast back to SongbirdError
    let downcast_result = boxed_error.downcast::<SongbirdError>();
    assert!(downcast_result.is_ok());
}

#[test]
fn test_error_any_trait() {
    use std::any::Any;
    
    let error = SongbirdError::config_error("field", "message");
    let any_ref: &dyn Any = &error;
    
    // Should be able to downcast via Any trait
    assert!(any_ref.downcast_ref::<SongbirdError>().is_some());
}

#[test]
fn test_error_thread_safety() {
    use std::sync::Arc;
    use std::thread;
    
    let error = Arc::new(SongbirdError::config_error("field", "message"));
    let error_clone = Arc::clone(&error);
    
    let handle = thread::spawn(move || {
        format!("{error_clone}")
    });
    
    let result = handle.join()
    .map_err(|e| SongbirdError::runtime_error(&format!("Thread join failed: {:?}", e)))?;
    assert!(!result.is_empty());
}

#[test]
fn test_error_source_chain_with_io() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let songbird_error: SongbirdError = io_error.into();
    
    // Test that the source chain is preserved
    let error_ref: &dyn Error = &songbird_error;
    assert!(error_ref.source().is_some());
    
    let source = error_ref.source().expect("Test operation should succeed");
    assert!(source.to_string().contains("Access denied"));
}

#[test]
fn test_error_boxed_trait_object() {
    fn handle_error(err: Box<dyn Error>) -> String {
        format!("Handled error: {}", err)
    }
    
    let error = SongbirdError::config_error("field", "message");
    let boxed_error = Box::new(error);
    
    let result = handle_error(boxed_error);
    assert!(result.contains("Handled error:"));
    assert!(result.contains("message"));
} 