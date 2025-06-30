use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Integration Tests for Songbird Orchestrator HPC System
//
// Tests end-to-end functionality of the HPC orchestration system
use async_trait::async_trait;
use serde_json::json;
use std::{collections::HashMap, sync::Once, time::Duration};

use songbird_gaming_bridge::{
    communication::ProtocolRouter,
    config::OrchestratorConfig,
    discovery::{ServiceDiscovery, StaticServiceDiscovery},
    errors::SongbirdError,
    orchestrator::Orchestrator,
    traits::{
        discovery::ServiceQuery, CommunicationLayer, ResponseStatus, ServiceEndpoint, ServiceInfo,
        ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService,
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

    async fn initialize(&mut self, _config: Self::Config) -> std::result::Result<()> {
        tracing::info!("Initializing test service_id: {}", self.name);
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<()> {
        tracing::info!("Starting test service_id: {}", self.name);
        self.is_running = true;
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<()> {
        tracing::info!("Stopping test service_id: {}", self.name);
        self.is_running = false;
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health> {
        Ok(json!({
            "status": if self.is_running { "healthy" } else { "stopped" },
            "service": self.name,
            "uptime": 100
        }))
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse> {
        let response = ServiceResponse {
            request_id: request.id.clone(),
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: json!({
                "service": self.name,
                "processed": true,
                "input_size": request.body.to_string().len()
            }),
            timestamp: chrono::Utc::now(),
            processing_time: Duration::from_millis(50),
            processing_time: std::time::Duration::from_millis(50),
            
        };
        Ok(response)
    }

    async fn update_config(
        &mut self,
        _config: Self::Config,
    ) -> std::result::Result<()> {
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics> {
        Ok(ServiceMetrics::default())
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_service_id: self.id.clone(),
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            service_type: "hpc-data-processor".to_string(),
            description: Some("HPC data processing service").to_string(),
            endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: "/process".to_string(),
                method: "POST".to_string(),
                description: Some("Process HPC data").to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }

    async fn can_handle_load(&self) -> std::result::Result<bool> {
        Ok(self.is_running)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64> {
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
        description: Some("HPC compute node for consumer tower").to_string(),
        endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
            path: "/compute".to_string(),
            method: "POST".to_string(),
            description: Some("Execute HPC computation").to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        tags: std::collections::HashMap::new(),
        tags: HashMap::new(),
        
    };

    let service2 = ServiceInfo {
        id: "hpc-node-2".to_string(),
        name: "HPC Storage Node".to_string(),
        version: "1.0.0".to_string(),
        service_type: "hpc-storage".to_string(),
        description: Some("HPC storage node for consumer tower").to_string(),
        endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
            path: "/store".to_string(),
            method: "POST".to_string(),
            description: Some("Store HPC data").to_string(),
            parameters: vec![],
            response_schema: None,
        }],
        tags: std::collections::HashMap::new(),
        tags: HashMap::new(),
        
    };

    // Initialize static discovery with HPC services
    let discovery = StaticServiceDiscovery::new();
    discovery
        .register(service1.clone())
        .await
        .expect("Test assertion failed");
    discovery
        .register(service2.clone())
        .await
        .expect("Test assertion failed");

    // Test service discovery - list all services
    let services = discovery.list_all().await.expect("Test assertion failed");
    assert_eq!(services.len(), 2);

    // Test service discovery with query for compute services
    let compute_query = ServiceQuery::new().with_service_type("hpc-compute");
    let compute_services = discovery
        .discover(compute_query)
        .await
        .expect("Test assertion failed");
    assert_eq!(compute_services.len(), 1);
    assert_eq!(compute_services[0].id, "hpc-node-1");

    // Test service discovery with query for storage services
    let storage_query = ServiceQuery::new().with_service_type("hpc-storage");
    let storage_services = discovery
        .discover(storage_query)
        .await
        .expect("Test assertion failed");
    assert_eq!(storage_services.len(), 1);
    assert_eq!(storage_services[0].id, "hpc-node-2");

    // Test service existence check
    assert!(discovery
        .exists("hpc-node-1")
        .await
        .expect("Test assertion failed"));
    assert!(discovery
        .exists("hpc-node-2")
        .await
        .expect("Test assertion failed"));
    assert!(!discovery
        .exists("non-existent")
        .await
        .expect("Test assertion failed"));

    tracing::info!("✅ HPC Static Discovery Integration Test Passed");
}

#[tokio::test]
async fn test_hpc_end_to_end_request_processing() {
    init_tracing();

    // Create HPC orchestrator
    let config = OrchestratorConfig::default();
    let _orchestrator = Orchestrator::new(config)
        .await
        .expect("Test assertion failed");

    // Create test HPC service
    let mut test_service = TestDataService::new(
        "hpc-data-processor-1".to_string(),
        "HPC Data Processor".to_string(),
    );

    // Initialize and start the service
    test_service
        .initialize(json!({}))
        .await
        .expect("Test assertion failed");
    test_service.start().await.expect("Test assertion failed");

    // Test health check
    let health = test_service
        .health_check()
        .await
        .expect("Test assertion failed");
    assert_eq!(health["status"], "healthy");

    // Test service request processing
    let request = ServiceRequest {
        service_id: "test-request-1".to_string(),
        method: "POST".to_string(),
        path: "/process".to_string(),
        headers: HashMap::new(),
        body: json!({
            "data": "test HPC computation data",
            "nodes": 4,
            "cores_per_node": 8
        }),
        timestamp: chrono::Utc::now(),
        timeout: Some(Duration::from_secs(30)),
        client_info: None,
        
    };

    let response = test_service
        .handle_request(request)
        .await
        .expect("Test assertion failed");

    match response.status {
        ResponseStatus::Success => {
            assert_eq!(response.body["processed"], true);
            assert_eq!(response.body["service"], "HPC Data Processor");
        }
        _ => panic!("Expected successful response"),
    }

    // Test load balancing capability
    assert!(test_service
        .can_handle_load()
        .await
        .expect("Test assertion failed"));
    let load_factor = test_service
        .get_load_factor()
        .await
        .expect("Test assertion failed");
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
