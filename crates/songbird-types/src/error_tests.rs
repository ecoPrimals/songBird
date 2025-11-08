//! Unit tests for error types

use songbird_types::{SongbirdError, SongbirdResult};
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use crate::error::*;

    #[test]
    fn test_songbird_error_network() {
        let error = SongbirdError::network("Connection failed");
        assert!(matches!(error, SongbirdError::Network(_)));
    }

    #[test]
    fn test_songbird_error_configuration() -> SongbirdResult<()> {
        let error = SongbirdError::configuration("Invalid config");
        assert!(matches!(error, SongbirdError::Configuration(_)));
        Ok(())
    }

    #[test]
    fn test_songbird_error_discovery() -> SongbirdResult<()> {
        let error = SongbirdError::discovery("Service not found");
        assert!(matches!(error, SongbirdError::Discovery(_)));
        Ok(())
    }

    #[test]
    fn test_songbird_error_adapters() -> SongbirdResult<()> {
        let error = SongbirdError::adapters("Adapter error");
        assert!(matches!(error, SongbirdError::Adapters(_)));
        Ok(())
    }

    #[test]
    fn test_songbird_error_display() -> SongbirdResult<()> {
        let error = SongbirdError::network("Test error");
        let display = format!("{error}");
        assert!(display.contains("Network error"));
        Ok(())
    }

    #[test]
    fn test_songbird_error_debug() -> SongbirdResult<()> {
        let error = SongbirdError::configuration("Test");
        let debug = format!("{error:?}");
        assert!(debug.contains("Configuration"));
        Ok(())
    }

    #[test]
    fn test_songbird_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error = SongbirdError::from(io_error);
        assert!(matches!(error, SongbirdError::Io(_)));
    }

    #[test]
    fn test_songbird_error_multiple_network() {
        let error1 = SongbirdError::network("Error 1");
        let error2 = SongbirdError::network("Error 2");
        
        let msg1 = format!("{error1}");
        let msg2 = format!("{error2}");
        
        assert!(msg1.contains("Error 1"));
        assert!(msg2.contains("Error 2"));
    }

    #[test]
    fn test_songbird_error_empty_message() {
        let error = SongbirdError::network("");
        let display = format!("{error}");
        assert!(!display.is_empty());
    }

    #[test]
    fn test_songbird_error_long_message() -> SongbirdResult<()> {
        let long_msg = "x".repeat(1000);
        let error = SongbirdError::network(&long_msg);
        let display = format!("{error}");
        assert!(display.len() > 500);
        Ok(())
    }

    #[test]
    fn test_songbird_error_unicode() -> SongbirdResult<()> {
        let error = SongbirdError::network("エラー: 日本語");
        let display = format!("{error}");
        assert!(display.contains("日本語"));
        Ok(())
    }

    #[test]
    fn test_songbird_error_special_chars() -> SongbirdResult<()> {
        let error = SongbirdError::network("Error: !@#$%^&*()");
        let display = format!("{error}");
        assert!(display.contains("!@#$%"));
        Ok(())
    }

    #[test]
    fn test_songbird_result_ok() -> SongbirdResult<()> {
        let result: SongbirdResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.map_err(|e| SongbirdError::configuration(format!("Result should be Ok: {}", e)))?, 42);
        Ok(())
    }

    #[test]
    fn test_songbird_result_err() {
        let result: SongbirdResult<i32> = Err(SongbirdError::network("Failed"));
        assert!(result.is_err());
    }

    #[test]
    fn test_songbird_error_chain() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "Not found");
        let error = SongbirdError::from(inner);
        let wrapped = SongbirdError::network(&format!("Wrapped: {error}"));
        
        let display = format!("{wrapped}");
        assert!(display.contains("Wrapped"));
    }

    #[test]
    fn test_error_variant_network() {
        let error = SongbirdError::Network("test".to_string());
        assert!(matches!(error, SongbirdError::Network(_)));
    }

    #[test]
    fn test_error_variant_configuration() {
        let error = SongbirdError::Configuration("test".to_string());
        assert!(matches!(error, SongbirdError::Configuration(_)));
    }

    #[test]
    fn test_error_variant_discovery() {
        let error = SongbirdError::Discovery("test".to_string());
        assert!(matches!(error, SongbirdError::Discovery(_)));
    }

    #[test]
    fn test_error_variant_adapters() {
        let error = SongbirdError::Adapters("test".to_string());
        assert!(matches!(error, SongbirdError::Adapters(_)));
    }
}

