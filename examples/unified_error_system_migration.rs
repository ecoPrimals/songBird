//! # Unified Error System Migration Example
//!
//! This example shows how to properly migrate unwrap/expect calls to use
//! Songbird's unified error system for production-safe error handling.

use songbird_errors::{
    SafeUnwrap, SafeUnwrapOption, SongbirdResult, 
    config_error, network_error, success
, SongbirdError};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

/// Example showing migration from unwrap() to unified error system
pub fn migrate_parsing_example() -> SongbirdResult<IpAddr> {
    // ❌ OLD (panic-prone):
    // let addr = "192.168.1.1".parse()
    .map_err(|e| songbird_errors::SongbirdError::network_error(
        "parse_failed",
        format!("Network address parsing failed: {}", e)
    ))?;
    
    // ✅ NEW (unified error system):
    "192.168.1.1"
        .parse()
        .or_config_error("ip_address_parsing")
}

/// Example showing migration from Option unwrap to unified error system
pub fn migrate_option_example() -> SongbirdResult<String> {
    let config_map = HashMap::new();
    
    // ❌ OLD (panic-prone):
    // let value = config_map.get("service_name").unwrap_or_else(|e| {
        tracing::error!("Test assertion failed: {:?}", e);
        panic!("Test assertion should not fail: {:?}", e);
    });
    
    // ✅ NEW (unified error system):
    config_map
        .get("service_name")
        .cloned()
        .or_config_error("service_name", "Service name is required in configuration")
}

/// Example showing migration from expect() to unified error system  
pub fn migrate_expect_example() -> SongbirdResult<serde_json::Value> {
    // ❌ OLD (panic-prone):
    // let json = serde_json::from_str(data).unwrap_or_else(|e| {
        tracing::error!("Expect failed ({}): {:?}", "JSON should be valid", e);
        panic!("Test assertion should not fail - {}: {:?}", "JSON should be valid", e);
    });
    
    // ✅ NEW (unified error system):
    let data = r#"{"name": "songbird", "version": "1.0"}"#;
    serde_json::from_str(data)
        .or_config_error("json_parsing")
}

/// Example showing network operations with unified error system
pub async fn migrate_network_example(&self) -> SongbirdResult<String> {
    // ❌ OLD (panic-prone):
    // let response = reqwest::get("http://example.com").await.unwrap_or_else(|e| {
        tracing::error!("Test assertion failed: {:?}", e);
        panic!("Test assertion should not fail: {:?}", e);
    });
    // let text = response.text().await.unwrap_or_else(|e| {
        tracing::error!("Test assertion failed: {:?}", e);
        panic!("Test assertion should not fail: {:?}", e);
    });
    
    // ✅ NEW (unified error system):
    let response = reqwest::get("http://example.com")
        .await
        .or_network_error("http_request")?;
        
    let text = response
        .text()
        .await
        .or_network_error("response_parsing")?;
        
    Ok(success(text))
}

/// Example showing service operation with unified error system
pub fn migrate_service_example() -> SongbirdResult<String> {
    // Simulate a service operation that might fail
    let result: SongbirdResult<String, std::io::Error> = Ok("service_data".to_string());
    
    // ❌ OLD (panic-prone):
    // let data = result.unwrap_or_else(|e| {
        tracing::error!("Test assertion failed: {:?}", e);
        panic!("Test assertion should not fail: {:?}", e);
    });
    
    // ✅ NEW (unified error system):
    result.or_service_error("data_processing")
}

/// Example showing environment variable access with unified error system
pub fn migrate_env_example() -> SongbirdResult<String> {
    use songbird_errors::SafeEnv;
    
    // ❌ OLD (panic-prone):
    // let port = std::env::var("PORT")
    .map_err(|e| songbird_errors::SongbirdError::configuration_error(
        "env_var_missing",
        format!("Environment variable '{}' not found: {}", "PORT", e)
    ))?;
    
    // ✅ NEW (unified error system):
    SafeEnv::get_or_config_error("PORT", "Port configuration is required")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_migration() {
        let result = migrate_parsing_example();
        assert!(result.is_ok());
    }

    #[test] 
    fn test_option_migration() {
        let result = migrate_option_example();
        // This should fail gracefully with a config error
        assert!(result.is_err());
        match result.unwrap_err() {
            songbird_errors::SongbirdError::Config { field, .. } => {
                assert_eq!(field, Some("service_name".to_string()));
            }
            _ => return Err(songbird_errors::SongbirdError::validation_error(
        "panic_converted",
        "Should be a config error"
    )),
        }
    }

    #[test]
    fn test_expect_migration() {
        let result = migrate_expect_example();
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_migration() {
        let result = migrate_service_example();
        assert!(result.is_ok());
    }

    #[test]
    fn test_env_migration() {
        // This will fail gracefully if PORT is not set
        let result = migrate_env_example();
        // Could be ok or error depending on environment
        match result {
            Ok(_) => println!("PORT environment variable is set"),
            Err(e) => println!("PORT not set, got graceful error: {}", e),
        }
    }
} 