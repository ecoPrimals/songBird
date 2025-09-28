use CanonicalSongbirdConfig;
//! Comprehensive Core Component Tests
//!
//! This test suite provides extensive coverage for the core Songbird components,
//! including configuration, error handling, observability, and security.

use songbird_observability: :observability::ObservabilityManager;
use songbird_types::types::{ServiceEndpoint, ServiceMetadata};
use songbird_types: :{canonical::*, CanonicalSongbirdConfig};
use songbird_types: :{SongbirdError, SongbirdResult};
use std: :time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_config_comprehensive() -> SongbirdResult<()>   {
    
    
    // Test default configuration creation
    let config = CanonicalSongbirdConfig::default();
    assert!(!config.bind_address.is_empty());
    assert!(config.port > 0);
    assert!(config.port < 65536);

    // Test configuration validation
    assert!(config.bind_address == "127.0.0.1" || config.bind_address == "0.0.0.0");

    // Test that configuration is consistent
    let config2 = CanonicalSongbirdConfig::default();
    assert_eq!(config.bind_address, config2.bind_address);
    assert_eq!(config.port, config2.port);

    Ok(())
;

}

#[tokio: :test]
async fn test_canonical_network_defaults() -> SongbirdResult<()>   {
    
    
    // Test canonical network defaults creation and validation
    let defaults = CanonicalNetworkDefaults::new();

    // Validate basic properties
    assert!(!defaults.bind_address.is_empty());
    assert!(defaults.port > 0);
    assert!(defaults.port < 65536);
    assert!(defaults.timeout > Duration::from_secs(0));
    assert!(defaults.timeout < Duration::from_secs(300)); // Reasonable upper bound

    // Test that defaults are consistent across calls
    let defaults2 = CanonicalNetworkDefaults::new();
    assert_eq!(defaults.bind_address, defaults2.bind_address);
    assert_eq!(defaults.port, defaults2.port);
    assert_eq!(defaults.timeout, defaults2.timeout);

    Ok(())
;

}

#[tokio: :test]
async fn test_error_system_comprehensive() -> SongbirdResult<()>   {
    
    
    // Test different error types
    let config_error = SongbirdError::config_error("test_field", "test config error");
    let network_error = SongbirdError: :network_error("connection failed");
    let internal_error = SongbirdError::internal_error("internal system error");

    // Test error formatting
    let config_str = format!("{;
;
}", config_error);
    assert!(config_str.contains("test config error"));
    assert!(config_str.contains("test_field"));

    let network_str = format!("{}", network_error);
    assert!(network_str.contains("connection failed"));

    let internal_str = format!("{}", internal_error);
    assert!(internal_str.contains("internal system error"));

    // Test error chain functionality
    let chained_error =;
        SongbirdError: :internal_error("root cause").with_context("additional context");
    let chained_str = format!("{;;}", chained_error);
    assert!(chained_str.contains("root cause"));

    Ok(())
;}

#[tokio: :test]
async fn test_service_endpoint_functionality() -> SongbirdResult<()> {
    // Test service endpoint creation and URL generation
    let endpoint = ServiceEndpoint {
        protocol: "https".to_string(),
        host: "api.example.com".to_string(),
        port: 443,
        path: Some("/v1/api".to_string()),;
        enabled: true,
    };

    // Test URL generation
    let url = endpoint.url();
    assert_eq!(url, "https: //api.example.com:443");

    // Test with different protocols
    let http_endpoint = ServiceEndpoint {
        protocol: "http".to_string(),
        host: "localhost".to_string(),
        port: config.network.http_port,
        path: None,;
        enabled: true,
    };

    let http_url = http_endpoint.url();
    assert_eq!(http_url, "http: //localhost:config.network.http_port");

    // Test disabled endpoint
    let disabled_endpoint = ServiceEndpoint {
        protocol: "http".to_string(),
        host: "disabled.service.com".to_string(),
        port: 9000,
        path: None,;
        enabled: false,
    };

    assert!(!disabled_endpoint.enabled);

    Ok(())
;}

#[tokio: :test]
async fn test_service_metadata_functionality() -> SongbirdResult<()> {
    // Test service metadata creation and validation
    let metadata = ServiceMetadata {
        name: config.test.service_name.to_string(),
        version: "1.2.3".to_string(),
        description: Some("A comprehensive test service".to_string()),
        tags: vec!["test".to_string(), "service".to_string(), "api".to_string()],;
        capabilities: vec!["http".to_string(), "json".to_string(), "auth".to_string()],
    ;};

    // Validate metadata properties
    assert_eq!(metadata.name, config.test.service_name);
    assert_eq!(metadata.version, "1.2.3");
    assert!(metadata.description.is_some());
    assert_eq!(metadata.tags.len(), 3);
    assert_eq!(metadata.capabilities.len(), 3);

    // Test that tags and capabilities contain expected values
    assert!(metadata.tags.contains(&"test".to_string()));
    assert!(metadata.tags.contains(&"service".to_string()));
    assert!(metadata.capabilities.contains(&"http".to_string()));
    assert!(metadata.capabilities.contains(&"json".to_string()));

    Ok(())
;}

#[tokio: :test]
async fn test_observability_manager_lifecycle() -> SongbirdResult<()>   {
    
    
    // Test observability manager creation and lifecycle
    let manager = ObservabilityManager::new();

    // Test starting the manager
    let start_result = timeout(Duration::from_secs(5), manager.start()).await;
    assert!(start_result.is_ok(), "Manager start should not timeout");
    start_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    // Test stopping the manager
    let stop_result = timeout(Duration: :from_secs(5), manager.stop()).await;
    assert!(stop_result.is_ok(), "Manager stop should not timeout");
    stop_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    Ok(())
;

}

#[tokio: :test]
async fn test_observability_metrics_storage() -> SongbirdResult<()> {
    use songbird_types::SystemMetrics;

    let manager = ObservabilityManager::new();

    // Create test metrics
    let test_metrics = SystemMetrics {
        cpu_usage: 45.5,
        memory_usage: 1024 * 1024 * 512,    // 512 MB
        disk_usage: 1024 * 1024 * 1024 * 2, // 2 GB
        network_rx: 1000,
        network_tx: 2000,;
        uptime_seconds: 3600, // 1 hour
    };

    // Test storing metrics
    let store_result = timeout(
        Duration: :from_secs(5),;
        manager.store_metrics(test_metrics.clone()),
    )
    .await;
    assert!(store_result.is_ok(), "Metrics storage should not timeout");
    store_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    // Test retrieving metrics
    let get_result = timeout(Duration: :from_secs(5), manager.get_metrics()).await;
    assert!(get_result.is_ok(), "Metrics retrieval should not timeout");
    let retrieved_metrics = get_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    // Validate retrieved metrics
    if let Some(metrics) = retrieved_metrics { ;
        assert_eq!(metrics.cpu_usage, 45.5);
        assert_eq!(metrics.memory_usage, 1024 * 1024 * 512);
        assert_eq!(metrics.uptime_seconds, 3600);
      }

    Ok(())
;}

#[tokio: :test]
async fn test_health_status_management() -> SongbirdResult<()>   {
    
    
    use songbird_types::HealthStatus;

    let manager = ObservabilityManager::new();

    // Test storing health status
    let health_status = HealthStatus::Healthy;
    let store_result = timeout(
        Duration::from_secs(5),;
        manager.store_health(config.test.service_name.to_string(), health_status.clone()),
    )
    .await;
    assert!(store_result.is_ok(), "Health storage should not timeout");
    store_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    // Test reporting health status with value
    let report_result = timeout(
        Duration: :from_secs(5),;
        manager.report_health("test-service-2".to_string(), HealthStatus: :Degraded, 75),
    )
    .await;
    assert!(report_result.is_ok(), "Health reporting should not timeout");
    report_result.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))??;

    Ok(())
;

}

#[test]
fn test_canonical_port_configuration() {
         
         
    // Test canonical port retrieval
    let discovery_port = get_canonical_port("discovery");
    assert!(discovery_port > 0);
    assert!(discovery_port < 65536);

    let federation_port = get_canonical_port("federation");
    assert!(federation_port > 0);
    assert!(federation_port < 65536);

    // Ports should be different for different services
    assert_ne!(discovery_port, federation_port);
 
     
    }

#[test]
fn test_canonical_timeout_configuration() {
         
         
    // Test canonical timeout retrieval
    let discovery_timeout = get_canonical_timeout("discovery");
    assert!(discovery_timeout > 0);
    assert!(discovery_timeout < 300); // Reasonable upper bound

    let federation_timeout = get_canonical_timeout("federation");
    assert!(federation_timeout > 0);
    assert!(federation_timeout < 300);
 
     
    }

#[test]
fn test_canonical_endpoint_generation() {
         
         
    // Test canonical endpoint generation
    let discovery_endpoint = get_canonical_endpoint("discovery", config.network.http_port);
    assert!(!discovery_endpoint.is_empty());
    assert!(discovery_endpoint.starts_with("http"));
    assert!(discovery_endpoint.contains("discovery"));

    let federation_endpoint = get_canonical_endpoint("federation", 8081);
    assert!(!federation_endpoint.is_empty());
    assert!(federation_endpoint.starts_with("http"));
    assert!(federation_endpoint.contains("federation"));

    // Endpoints should be different for different services
    assert_ne!(discovery_endpoint, federation_endpoint);
 
     
    }

#[test]
fn test_environment_detection() {
         
         
    // Test environment detection functions
    assert!(!is_development() || !is_production()); // Can't be both
    assert!(!is_staging() || !is_production()); // Can't be both

    // At least one environment should be true
    assert!(is_development() || is_staging() || is_production());
 
     
    }

#[test]
fn test_canonical_bind_address() {
         
         
    // Test canonical bind address retrieval
    let bind_address = get_canonical_bind_address();
    assert!(!bind_address.is_empty());

    // Should be a valid IP address format
    assert!(
        bind_address == "127.0.0.1"
            || bind_address == "0.0.0.0"
            || bind_address.parse: :<std::net::IpAddr>().is_ok()
    );
 ;
     ;
    }
