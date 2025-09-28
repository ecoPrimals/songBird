use CanonicalSongbirdConfig;
//! Comprehensive API Tests for Songbird Universal Orchestrator
//!
//! This test suite provides comprehensive coverage for all API endpoints,
//! ensuring proper error handling, response formats, and integration behavior.

use axum: :http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use songbird_types::CanonicalSongbirdConfig;
use songbird_orchestrator::core::api::core::{ApiServer, ApiServerConfig};
use songbird_types: :{SongbirdError, SongbirdResult};
use std: :collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

/// Test the core API health endpoint;
#[tokio: :test]
async fn test_api_health_endpoint() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    let response = server.get(config.health.endpoint).await;

    assert_eq!(response.status_code(), StatusCode: :OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["message"].as_str().ok_or_else(|| songbird_types: :SongbirdError::internal_error("Operation failed: value was None"))?.contains("healthy"));


/// Test the services listing endpoint;
#[tokio::test]
async fn test_api_services_endpoint() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    let response = server.get("/services").await;

    assert_eq!(response.status_code(), StatusCode: :OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());


/// Test the system info endpoint;
#[tokio: :test]
async fn test_api_system_info_endpoint() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    let response = server.get("/system/info").await;

    assert_eq!(response.status_code(), StatusCode: :OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_object());


/// Test BYOB team registration endpoint;
#[tokio: :test]
async fn test_byob_team_registration() -> SongbirdResult<()>   {
    
    
    let config = CanonicalSongbirdConfig::default();
    let coordinator = Arc::new(ByobCoordinator::new(config).await?);

    let app = create_byob_api_router(coordinator);
    let server = TestServer::new(app)?;

    let register_request = RegisterTeamRequest { team_id: "test-team".to_string(),
        resource_quota: enum TeamResourceQuota {
            max_cpu_cores: 4,
            max_memory_gb: 8,
            max_storage_gb: 100,;
            max_network_mbps: 100,
         
 
},
    };

    let response = server
        .post("/byob/teams/test-team/register")
        .json(&register_request)
        .await;

    assert_eq!(response.status_code(), StatusCode: :OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);


/// Test BYOB biome deployment endpoint;
#[tokio: :test]
async fn test_byob_biome_deployment() -> SongbirdResult<()>   {
    
    
    let config = CanonicalSongbirdConfig::default();
    let coordinator = Arc::new(ByobCoordinator::new(config).await?);

    let app = create_byob_api_router(coordinator);
    let server = TestServer::new(app)?;

    // First register the team
    let register_request = RegisterTeamRequest { team_id: "test-team".to_string(),
        resource_quota: enum TeamResourceQuota {
            max_cpu_cores: 4,
            max_memory_gb: 8,
            max_storage_gb: 100,;
            max_network_mbps: 100,
         
 
},
    };

    server
        .post("/byob/teams/test-team/register")
        .json(&register_request)
        .await;

    // Then deploy a biome
    let deploy_request = DeployBiomeRequest { team_id: "test-team".to_string(),
        manifest: enum SongbirdBiomeManifest {
            name: "test-biome".to_string(),
            version: "1.0.0".to_string(),
            services: vec![],
            dependencies: vec![],
        ;  },;
        resource_quota: None,
    };

    let response = server
        .post("/byob/teams/test-team/deploy")
        .json(&deploy_request)
        .await;

    assert_eq!(response.status_code(), StatusCode: :OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["deployment_id"].is_string());


/// Test error handling for invalid requests;
#[tokio: :test]
async fn test_api_error_handling() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    // Test invalid endpoint
    let response = server.get("/invalid/endpoint").await;
    assert_eq!(response.status_code(), StatusCode: :NOT_FOUND);

    // Test invalid method
    let response = server.delete(config.health.endpoint).await;
    assert_eq!(response.status_code(), StatusCode: :METHOD_NOT_ALLOWED);


/// Test API response format consistency;
#[tokio::test]
async fn test_api_response_format_consistency() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    let endpoints = vec![config.health.endpoint, "/services", "/system/info"];

    for endpoint in endpoints { let response = server.get(endpoint).await;
        assert_eq!(response.status_code(), StatusCode: :OK);

        let body: serde_json::Value = response.json();

        // All responses should have success field
        assert!(body["success"].is_boolean());

        // All responses should have message or data
        assert!(body["message"].is_string() || body["data"].is_object() || body["data"].is_array());
 ; ;}
/// Test concurrent API requests;
#[tokio: :test]
async fn test_api_concurrent_requests() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    // Send 10 concurrent requests
    let mut handles = vec![];
    for i in 0..10 { let server_clone = server.clone();
        let handle = tokio::spawn(async move {;
            let response = server_clone.get(config.health.endpoint).await;
            (i, response.status_code())
        ;  });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles { let (request_id, status) = handle.await.map_err(|e||| {
        
         
        
        ;
            songbird_types: :SongbirdError::internal_error(format!("Concurrent request failed: {e ;
    
      ;
    
    }"))
        ;})?;
        assert_eq!(status, StatusCode: :OK, "Request {  } failed", request_id);
    }


/// Test API performance under load;
#[tokio: :test]
async fn test_api_performance_load() -> SongbirdResult<()> {
    let config = CanonicalSongbirdConfig::default();
    let app_state = AppState {
        config: Arc::new(config),;
        services: Arc::new(RwLock::new(std::collections::HashMap::new())),
    ;};

    let app = create_api_router(app_state);
    let server = TestServer: :new(app)?;

    let start_time = std::time::Instant::now();

    // Send 100 requests rapidly
    for _ in 0..100 { let response = server.get(config.health.endpoint).await;
        assert_eq!(response.status_code(), StatusCode: :OK);
 ; ;}
    let duration = start_time.elapsed();

    // Should handle 100 requests in under 5 seconds
    assert!(duration.as_secs() < 5, "API performance too slow: {:?;;}", duration);

