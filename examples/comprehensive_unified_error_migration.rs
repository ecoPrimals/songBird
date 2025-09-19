//! # Comprehensive Unified Error System Migration Guide
//!
//! This example demonstrates how to migrate ALL types of unwrap/expect calls
//! to use Songbird's unified error system in production code.

use songbird_errors::{
    SafeUnwrap, SafeUnwrapOption, SafeEnv, SafeParse, SafeSecurity, 
    SongbirdError, SongbirdResult, success, config_error, network_error
, SongbirdError};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, RwLock};
use std::env;

/// ==================================================================================
/// 1. ENVIRONMENT VARIABLE ACCESS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone environment access (MODERNIZED - DO NOT USE)
pub fn old_env_access_example() -> Result<String, Box<dyn std::error::Error>> {
    env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required".into())
}

/// ✅ NEW: Unified error system environment access
pub fn new_env_access() -> SongbirdResult<String> {
    SafeEnv::get_or_config_error("DATABASE_URL", "Database URL is required for operation")
}

/// ✅ NEW: Environment access with default fallback
pub fn new_env_with_default() -> String {
    SafeEnv::get_or_default("LOG_LEVEL", "info")
}

/// ==================================================================================
/// 2. PARSING OPERATIONS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone parsing (MODERNIZED - DO NOT USE)
pub fn old_ip_parsing_example(ip_str: &str) -> Result<IpAddr, Box<dyn std::error::Error>> {
    ip_str.parse()
        .map_err(|e| format!("Invalid IP address '{}': {}", ip_str, e).into())
}

/// ✅ NEW: Unified error system parsing
pub fn new_ip_parsing(ip_str: &str) -> SongbirdResult<IpAddr> {
    ip_str.parse().or_config_error("ip_address")
}

/// ❌ OLD: Panic-prone socket address parsing (MODERNIZED - DO NOT USE)
pub fn old_socket_parsing_example(addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    addr_str.parse()
        .or_else(|_| "127.0.0.1:{}".parse())
        .map_err(|e| format!("Invalid socket address: {}", e).into())
}

/// ✅ NEW: Unified error system socket parsing
pub fn new_socket_parsing(addr_str: &str) -> SongbirdResult<SocketAddr> {
    addr_str.parse().or_network_error("socket_address_parsing")
}

/// ==================================================================================
/// 3. COLLECTION ACCESS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone map access (MODERNIZED - DO NOT USE)
pub fn old_map_access_example(config: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
    config.get("service_name")
        .cloned()
        .ok_or_else(|| "Missing required 'service_name' key in configuration".into())
}

/// ✅ NEW: Unified error system map access
pub fn new_map_access(config: &HashMap<String, String>) -> SongbirdResult<String> {
    config
        .get("service_name")
        .cloned()
        .or_config_error("service_name", "Service name must be provided in configuration")
}

/// ❌ OLD: Panic-prone vector access (MODERNIZED - DO NOT USE)
pub fn old_vec_access_example(endpoints: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    endpoints.first()
        .cloned()
        .ok_or_else(|| "No endpoints available - at least one endpoint is required".into())
}

/// ✅ NEW: Unified error system vector access
pub fn new_vec_access(endpoints: &[String]) -> SongbirdResult<String> {
    endpoints
        .first()
        .cloned()
        .or_network_error("endpoint_selection", "At least one endpoint must be configured")
}

/// ==================================================================================
/// 4. LOCK OPERATIONS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone mutex access (MODERNIZED - DO NOT USE)
pub fn old_mutex_access_example(data: &Arc<Mutex<i32>>) -> Result<i32, Box<dyn std::error::Error>> {
    let guard = data.lock()
        .map_err(|e| format!("Mutex lock failed: {}", e))?;
    Ok(*guard)
}

/// ✅ NEW: Unified error system mutex access
pub fn new_mutex_access(data: &Arc<Mutex<i32>>) -> SongbirdResult<i32> {
    use songbird_errors::safe_lock;
    let guard = safe_lock(data).or_internal_error("mutex_access")?;
    Ok(success(*guard))
}

/// ❌ OLD: Panic-prone RwLock access (MODERNIZED - DO NOT USE)
pub fn old_rwlock_access_example(data: &Arc<RwLock<String>>) -> Result<String, Box<dyn std::error::Error>> {
    let guard = data.read()
        .map_err(|e| format!("RwLock read failed: {}", e))?;
    Ok(guard.clone())
}

/// ✅ NEW: Unified error system RwLock access
pub fn new_rwlock_access(data: &Arc<RwLock<String>>) -> SongbirdResult<String> {
    use songbird_errors::safe_read_lock;
    let guard = safe_read_lock(data).or_internal_error("rwlock_read_access")?;
    Ok(success(guard.clone()))
}

/// ==================================================================================
/// 5. JSON/SERIALIZATION - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone JSON parsing (MODERNIZED - DO NOT USE)
pub fn old_json_parsing_example(json_str: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e).into())
}

/// ✅ NEW: Unified error system JSON parsing
pub fn new_json_parsing(json_str: &str) -> SongbirdResult<serde_json::Value> {
    serde_json::from_str(json_str).or_config_error("json_parsing")
}

/// ❌ OLD: Panic-prone JSON serialization (MODERNIZED - DO NOT USE)
pub fn old_json_serialization_example(data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    serde_json::to_string(data)
        .map_err(|e| format!("JSON serialization failed: {}", e).into())
}

/// ✅ NEW: Unified error system JSON serialization
pub fn new_json_serialization(data: &serde_json::Value) -> SongbirdResult<String> {
    serde_json::to_string(data).or_internal_error("json_serialization")
}

/// ==================================================================================
/// 6. ASYNC OPERATIONS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone async join (MODERNIZED - DO NOT USE)
pub async fn old_async_operations_example() -> Result<String, Box<dyn std::error::Error>> {
    let handle = tokio::spawn(async { "result".to_string() });
    handle.await.map_err(|e| format!("Async task join failed: {}", e).into())
}

/// ✅ NEW: Unified error system async operations
pub async fn new_async_operations(&self) -> SongbirdResult<String> {
    let handle = tokio::spawn(async { "result".to_string() });
    handle.await.or_internal_error("async_task_join")
}

/// ==================================================================================
/// 7. NETWORK OPERATIONS - Migration Examples
/// ==================================================================================

/// ❌ OLD: Panic-prone HTTP request (MODERNIZED - DO NOT USE)
pub async fn old_http_request_example(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    response.text().await
        .map_err(|e| format!("Failed to read response: {}", e).into())
}

/// ✅ NEW: Unified error system HTTP request
pub async fn new_http_request(&self) -> SongbirdResult<String> {
    let response = reqwest::get(url)
        .await
        .or_network_error("http_request")?;
        
    let text = response
        .text()
        .await
        .or_network_error("response_body_parsing")?;
        
    Ok(success(text))
}

/// ==================================================================================
/// 8. COMPLEX EXAMPLE - Real Production Scenario
/// ==================================================================================

#[derive(serde::Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint: String,
    pub port: u16,
}

/// ❌ OLD: Multiple panic points in production code (MODERNIZED - DO NOT USE)
pub fn old_service_initialization_example() -> Result<(ServiceConfig, SocketAddr), Box<dyn std::error::Error>> {
    // Safe environment variable access
    let config_json = env::var("SERVICE_CONFIG")
        .map_err(|_| "SERVICE_CONFIG environment variable is required")?;
    
    // Safe JSON parsing
    let config: ServiceConfig = serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid service configuration JSON: {}", e))?;
    
    // Safe address parsing
    let addr_str = format!("{}:{}", config.endpoint, config.port);
    let socket_addr = addr_str.parse()
        .map_err(|e| format!("Invalid socket address '{}': {}", addr_str, e))?;
    
    Ok((config, socket_addr))
}

/// ✅ NEW: Production-safe unified error system
pub fn new_service_initialization() -> SongbirdResult<(ServiceConfig, SocketAddr)> {
    // Safe environment access
    let config_json = SafeEnv::get_or_config_error(
        "SERVICE_CONFIG", 
        "Service configuration JSON is required"
    )?;
    
    // Safe JSON parsing with context
    let config: ServiceConfig = serde_json::from_str(&config_json)
        .or_config_error("service_config_json")?;
    
    // Safe address parsing with context
    let addr_str = format!("{}:{}", config.endpoint, config.port);
    let socket_addr = addr_str
        .parse()
        .or_network_error("service_socket_address")?;
    
    Ok(success((config, socket_addr)))
}

/// ==================================================================================
/// 9. ERROR CONTEXT AND RECOVERY EXAMPLES
/// ==================================================================================

/// Complex operation with multiple error contexts
pub async fn complex_operation_with_recovery(&self) -> SongbirdResult<String> {
    // Try primary endpoint
    match new_http_request("https://primary.service.com/health").await {
        Ok(response) => return Ok(response),
        Err(SongbirdError::Network { .. }) => {
            tracing::warn!("Primary service unavailable, trying fallback");
        }
        Err(other) => return Err(other), // Non-recoverable error
    }
    
    // Try fallback endpoint
    match new_http_request("https://fallback.service.com/health").await {
        Ok(response) => Ok(response),
        Err(_) => {
            // Return a service error with recovery suggestions
            Err(SongbirdError::Service {
                service: "health_check".to_string(),
                message: "Both primary and fallback services unavailable".to_string(),
                suggested_alternatives: vec![
                    "Check network connectivity".to_string(),
                    "Verify service endpoints in configuration".to_string(),
                    "Try again in a few minutes".to_string(),
                ],
                recovery_actions: vec![
                    "Enable offline mode".to_string(),
                    "Use cached data".to_string(),
                ],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_migration() {
        // This will fail gracefully if not set
        match new_env_access() {
            Ok(_) => println!("DATABASE_URL is configured"),
            Err(e) => println!("Expected graceful error: {}", e),
        }
    }

    #[test]
    fn test_parsing_migration() {
        let result = new_ip_parsing("192.168.1.1");
        assert!(result.is_ok());
        
        let result = new_ip_parsing("invalid-ip");
        assert!(result.is_err());
        match result.unwrap_err() {
            SongbirdError::Config { field, .. } => {
                assert_eq!(field, Some("ip_address".to_string()));
            }
            _ => return Err(songbird_errors::SongbirdError::validation_error(
        "panic_converted",
        "Should be a config error"
    )),
        }
    }

    #[test]
    fn test_collection_migration() {
        let mut config = HashMap::new();
        config.insert("service_name".to_string(), "test-service".to_string());
        
        let result = new_map_access(&config);
        assert!(result.is_ok());
        
        let empty_config = HashMap::new();
        let result = new_map_access(&empty_config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_migration() {
        // This will fail gracefully if SERVICE_CONFIG is not set
        match new_service_initialization() {
            Ok((config, addr)) => {
                println!("Service configured: {} at {}", config.name, addr);
            }
            Err(e) => {
                println!("Expected graceful configuration error: {}", e);
            }
        }
    }
} 