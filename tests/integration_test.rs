//! Integration Tests for Songbird Orchestrator HPC System
//!
//! Tests end-to-end functionality of the HPC orchestration system

use async_trait::async_trait;
use serde_json::json;
use std::{collections::HashMap, time::Duration, sync::Once};

use songbird_orchestrator::{
    communication::ProtocolRouter,
    config::OrchestratorConfig,
    discovery::{StaticServiceDiscovery, ServiceDiscovery},
    errors::SongbirdError,
    orchestrator::Orchestrator,
    traits::{
        ServiceEndpoint, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse,
        UniversalService, ResponseStatus, discovery::ServiceQuery, CommunicationLayer,
    },
};

static INIT: Once = Once::new();

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt::init();
    });
}

/// Test service for HPC data processing
#[derive(Debug, Clone)]
pub struct TestDataService {
    id: String,
    name: String,
    is_running: bool,
}

impl TestDataService {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            is_running: false,
        }
    }
}

#[async_trait]
impl UniversalService for TestDataService {
    type Config = serde_json::Value;
    type Health = serde_json::Value;
    type Error = SongbirdError;

    async fn initialize(&mut self, _config: Self::Config) -> std::result::Result<(), Self::Error> {
        tracing::info!("Initializing test service: {}", self.name);
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        tracing::info!("Starting test service: {}", self.name);
        self.is_running = true;
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        tracing::info!("Stopping test service: {}", self.name);
        self.is_running = false;
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        Ok(json!({
            "status": if self.is_running { "healthy" } else { "stopped" },
            "service": self.name,
            "uptime": 100
        }))
    }

    async fn handle_request(&self, request: ServiceRequest) -> std::result::Result<ServiceResponse, Self::Error> {
        let response = ServiceResponse {
            request_id: request.id.clone(),
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            payload: json!({
                "service": self.name,
                "processed": true,
                "input_size": request.payload.to_string().len()
            }),
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(50),
            processing_time: 50,
            metadata: HashMap::new(),
        };
        Ok(response)
    }

    async fn update_config(&mut self, _config: Self::Config) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics::default())
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            service_type: "hpc-data-processor".to_string(),
            description: "HPC data processing service".to_string(),
            endpoints: vec![ServiceEndpoint {
                path: "/process".to_string(),
                method: "POST".to_string(),
                description: "Process HPC data".to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            capabilities: vec!["compute".to_string(), "data-processing".to_string()],
            tags: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(self.is_running)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        Ok(0.5) // 50% load
    }
}

#[tokio::test]
async fn test_hpc_static_discovery_integration() {
    init_tracing();

    // Create test services for HPC cluster
    let service1 = ServiceInfo {
        id: "hpc-node-1".to_string(),
        name: "HPC Compute Node 1".to_string(),
        version: "1.0.0".to_string(),
        service_type: "hpc-compute".to_string(),
        description: "HPC compute node for consumer tower".to_string(),
        endpoints: vec![ServiceEndpoint {
            path: "/compute".to_string(),
            method: "POST".to_string(),
            description: "Execute HPC computation".to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        capabilities: vec!["compute".to_string(), "parallel-processing".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };

    let service2 = ServiceInfo {
        id: "hpc-node-2".to_string(),
        name: "HPC Storage Node".to_string(),
        version: "1.0.0".to_string(),
        service_type: "hpc-storage".to_string(),
        description: "HPC storage node for consumer tower".to_string(),
        endpoints: vec![ServiceEndpoint {
            path: "/store".to_string(),
            method: "POST".to_string(),
            description: "Store HPC data".to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        capabilities: vec!["storage".to_string(), "distributed-storage".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };

    // Initialize static discovery with HPC services
    let discovery = StaticServiceDiscovery::new();
    discovery.register(service1.clone()).await.unwrap();
    discovery.register(service2.clone()).await.unwrap();

    // Test service discovery - list all services
    let services = discovery.list_all().await.unwrap();
    assert_eq!(services.len(), 2);

    // Test service discovery with query for compute services
    let compute_query = ServiceQuery::new().with_service_type("hpc-compute");
    let compute_services = discovery.discover(compute_query).await.unwrap();
    assert_eq!(compute_services.len(), 1);
    assert_eq!(compute_services[0].id, "hpc-node-1");

    // Test service discovery with query for storage services
    let storage_query = ServiceQuery::new().with_service_type("hpc-storage");
    let storage_services = discovery.discover(storage_query).await.unwrap();
    assert_eq!(storage_services.len(), 1);
    assert_eq!(storage_services[0].id, "hpc-node-2");

    // Test service existence check
    assert!(discovery.exists("hpc-node-1").await.unwrap());
    assert!(discovery.exists("hpc-node-2").await.unwrap());
    assert!(!discovery.exists("non-existent").await.unwrap());

    tracing::info!("✅ HPC Static Discovery Integration Test Passed");
}

#[tokio::test]
async fn test_hpc_end_to_end_request_processing() {
    init_tracing();

    // Create HPC orchestrator
    let config = OrchestratorConfig::default();
    let _orchestrator = Orchestrator::new(config).await.unwrap();

    // Create test HPC service
    let mut test_service = TestDataService::new(
        "hpc-data-processor-1".to_string(),
        "HPC Data Processor".to_string(),
    );

    // Initialize and start the service
    test_service.initialize(json!({})).await.unwrap();
    test_service.start().await.unwrap();

    // Test health check
    let health = test_service.health_check().await.unwrap();
    assert_eq!(health["status"], "healthy");

    // Test service request processing
    let request = ServiceRequest {
        id: "test-request-1".to_string(),
        method: "POST".to_string(),
        path: "/process".to_string(),
        headers: HashMap::new(),
        payload: json!({
            "data": "test HPC computation data",
            "nodes": 4,
            "cores_per_node": 8
        }),
        timestamp: chrono::Utc::now(),
        timeout: Some(Duration::from_secs(30)),
        client_info: None,
        metadata: HashMap::new(),
    };

    let response = test_service.handle_request(request).await.unwrap();
    
    match response.status {
        ResponseStatus::Success => {
            assert_eq!(response.payload["processed"], true);
            assert_eq!(response.payload["service"], "HPC Data Processor");
        }
        _ => panic!("Expected successful response"),
    }

    // Test load balancing capability
    assert!(test_service.can_handle_load().await.unwrap());
    let load_factor = test_service.get_load_factor().await.unwrap();
    assert!(load_factor >= 0.0 && load_factor <= 1.0);

    tracing::info!("✅ HPC End-to-End Request Processing Test Passed");
}

#[tokio::test]
async fn test_hpc_communication_layer() {
    init_tracing();

    // Create communication layer for HPC cluster
    let comm_layer = ProtocolRouter::new();

    // Test that communication layer can be connected
    assert!(comm_layer.connect().await.is_ok());
    assert!(comm_layer.is_connected().await);

    // Test getting stats
    assert!(comm_layer.get_stats().await.is_ok());

    tracing::info!("✅ HPC Communication Layer Test Passed");
} 