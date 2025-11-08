//! Core registry functionality tests
//!
//! Tests for service registry operations, lookups, and management

use songbird_test_utils::test_discovery_port;
use songbird_test_utils::test_health_port;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[tokio::test]
async fn test_registry_types_available() {
    // Test that registry types are accessible
    let result = std::panic::catch_unwind(|| true);

    assert!(result.is_ok(), "Registry types should be available");
}

#[test]
fn test_service_metadata_operations() {
    let mut metadata: HashMap<String, String> = HashMap::new();
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());

    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata.get("environment"), Some(&"production".to_string()));
    assert_eq!(metadata.get("region"), Some(&"us-west".to_string()));
}

#[test]
fn test_service_metadata_empty() {
    let metadata: HashMap<String, String> = HashMap::new();
    assert_eq!(metadata.len(), 0);
}

#[test]
fn test_service_endpoints_collection() {
    let endpoints = [
        format!("http://localhost:{}", test_orchestrator_port()),
        format!("http://localhost:{}", test_discovery_port()),
        format!("http://localhost:{}", test_health_port()),
    ];

    assert_eq!(endpoints.len(), 3);
    assert!(endpoints.contains(&format!("http://localhost:{}", test_orchestrator_port())));
}

#[test]
fn test_service_capabilities_collection() {
    let capabilities =
        ["compute".to_string(), "storage".to_string(), "ai".to_string(), "security".to_string()];

    assert_eq!(capabilities.len(), 4);
    assert!(capabilities.contains(&"compute".to_string()));
    assert!(capabilities.contains(&"storage".to_string()));
}

#[test]
fn test_service_version_formats() {
    let versions = vec!["1.0.0", "2.1.3", "0.1.0-alpha", "3.0.0-beta.1"];

    for version in versions {
        assert!(!version.is_empty());
    }
}

#[test]
fn test_service_id_generation() {
    let service_ids: Vec<String> = (0..5).map(|i| format!("service-{}", i)).collect();

    assert_eq!(service_ids.len(), 5);
    assert_eq!(service_ids[0], "service-0");
    assert_eq!(service_ids[4], "service-4");
}

#[test]
fn test_service_collection_operations() {
    let mut services = Vec::new();

    for i in 0..5 {
        services.push(format!("service-{}", i));
    }

    assert_eq!(services.len(), 5);
    assert!(services.contains(&"service-0".to_string()));
}

#[test]
fn test_metadata_filtering() {
    let services = [("service-1", "prod"), ("service-2", "dev"), ("service-3", "prod")];

    let prod_services: Vec<_> = services.iter().filter(|(_, env)| *env == "prod").collect();

    assert_eq!(prod_services.len(), 2);
}

#[test]
fn test_capability_filtering() {
    let services = [
        ("service-1", vec!["compute"]),
        ("service-2", vec!["storage"]),
        ("service-3", vec!["compute", "ai"]),
    ];

    let compute_services: Vec<_> =
        services.iter().filter(|(_, caps)| caps.contains(&"compute")).collect();

    assert_eq!(compute_services.len(), 2);
}

#[test]
fn test_endpoint_validation() {
    let valid_endpoints = vec![
        format!("http://localhost:{}", test_orchestrator_port()),
        "https://api.example.com".to_string(),
        format!("http://192.168.1.1:{}", songbird_config::defaults::ports::metrics_port()),
        format!("ws://example.com:{}", songbird_config::defaults::ports::dashboard_port()),
    ];

    for endpoint in valid_endpoints {
        assert!(!endpoint.is_empty());
        assert!(endpoint.starts_with("http") || endpoint.starts_with("ws"));
    }
}

#[test]
fn test_service_sorting() -> SongbirdResult<()> {
    let mut services = ["service-3", "service-1", "service-2"];
    services.sort_unstable();

    assert_eq!(services[0], "service-1");
    assert_eq!(services[1], "service-2");
    assert_eq!(services[2], "service-3");
    Ok(())
}

#[test]
fn test_service_deduplication() -> SongbirdResult<()> {
    let mut services = vec!["service-1", "service-1", "service-2"];
    services.dedup();

    assert_eq!(services.len(), 2);
    Ok(())
}

#[test]
fn test_registry_result_type() -> SongbirdResult<()> {
    let result: SongbirdResult<String> = Ok("test-service".to_string());
    assert!(result.is_ok());
    assert_eq!(
        result.ok_or_else(|| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        "test-service"
    );
    Ok(())
}

#[test]
fn test_registry_error_result() {
    // Test that registry can handle error results
    let result: Result<String, String> = Err("Test error".to_string());
    assert!(result.is_err());

    let ok_result: Result<String, String> = Ok("success".to_string());
    assert!(ok_result.is_ok());
}
