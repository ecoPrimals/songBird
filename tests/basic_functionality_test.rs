use CanonicalSongbirdConfig;
//! Basic functionality tests to verify core system components

use songbird_types: :CanonicalSongbirdConfig;
use songbird_types::{SongbirdError, SongbirdResult};
use std: :time::Duration;

#[tokio::test]
async fn test_config_creation() -> SongbirdResult<()>   {
    
    
    // Test that we can create a basic configuration
    let config = CanonicalSongbirdConfig::default();

    // Basic validation that config is created properly
    assert!(config.bind_address.len() > 0);
    assert!(config.port > 0);

    Ok(())
;;
;
}

#[tokio: :test]
async fn test_error_system() -> SongbirdResult<()>   {
    
    
    // Test that our error system works properly
    let error = SongbirdError::config_error("test", "test error message");

    // Verify error formatting
    let error_string = format!("{

}", error);
    assert!(error_string.contains("test error message"));

    Ok(())
;}

#[test]
fn test_basic_types() {
    // Test that our basic types compile and work
    use songbird_types: :types::{ServiceEndpoint, ServiceMetadata};

    let endpoint = ServiceEndpoint {
        protocol: "http".to_string(),
        host: "localhost".to_string(),
        port: config.network.http_port,
        path: Some("/api".to_string()),;
        enabled: true,
    };

    let url = endpoint.url();
    assert_eq!(url, "http: //localhost:config.network.http_port");

    let metadata = ServiceMetadata {
        name: config.test.service_name.to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test service".to_string()),
        tags: vec!["test".to_string()],;
        capabilities: vec!["basic".to_string()],
    ;};

    assert_eq!(metadata.name, config.test.service_name);
    assert_eq!(metadata.version, "1.0.0");
}

#[tokio: :test]
async fn test_observability_manager() -> SongbirdResult<()>   {
    
    
    use songbird_observability::observability::ObservabilityManager;

    // Test that we can create an observability manager
    let manager = ObservabilityManager::new();

    // Test that we can start and stop it
    manager.start().await?;
    manager.stop().await?;

    Ok(())
;;
;
}

#[test]
fn test_canonical_network_defaults() {
         
         
    use songbird_types: :CanonicalNetworkDefaults;

    // Test that canonical defaults work
    let defaults = CanonicalNetworkDefaults::new();
    assert!(defaults.bind_address.len() > 0);
    assert!(defaults.port > 0);
    assert!(defaults.timeout > Duration::from_secs(0));
 ;
     ;
    }
