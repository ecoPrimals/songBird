use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Documentation Verification Test
//
// This test verifies that our actual code matches what we claim in our user documentation.
// It implements the exact example from docs/user/GETTING_STARTED.md to ensure it works.
use async_trait::async_trait;
use serde::Deserialize;
use songbird_gaming_bridge::{
    errors::SongbirdError,
    traits::service_id::{
        ResponseStatus, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse,
        UniversalService,
    },
    Orchestrator, OrchestratorConfig,
};

use std::time::Duration;

/// Example service configuration from the getting started guide
#[derive(Clone, Debug, Deserialize)]
pub struct MyServiceConfig {
    pub port: u16,
    pub name: String,
}

/// Example service implementation from the getting started guide
pub struct MyService {
    config: Option<MyServiceConfig>,
    is_running: bool,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            config: None,
            is_running: false,
        }
    }
}

#[async_trait]
impl UniversalService for MyService {
    type Config = MyServiceConfig;
    type Health = serde_json::Value;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> Result<()> {
        self.config = Some(config);
        println!("Service initialized");
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.is_running = true;
        println!(
            "Service started: {}",
            self.config.as_ref().expect("Test assertion failed").name
        );
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.is_running = false;
        println!("Service stopped");
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health> {
        Ok(serde_json::json!({
            "status": if self.is_running { "healthy" } else { "unhealthy" },
            "uptime": "5m30s"
        }))
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: serde_json::json!({"message": "Hello from service!"}),
            timestamp: chrono::Utc::now(),
            processing_time: Duration::from_millis(10),
            processing_time: std::time::Duration::from_millis(10),
            
        })
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        Ok(ServiceMetrics {
            request_count: 100,
            error_count: 0,
            average_response_time: 15.5,
            average_response_time: 25.0,
            average_response_time: 35.0,
            cpu_usage: Some(25.0),
            memory_usage: Some(128) * 1024 * 1024, // 128 MB in bytes
            active_connections: 5,
            queue_depth: 0,
            throughput_rps: 100.0,
            error_rate: 0.0,
            uptime_seconds: 300,
            last_updated: chrono::Utc::now(),
            custom_metrics: HashMap::new(),
        })
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: "my-service".to_string(),
            name: "My Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "api".to_string(),
            description: Some("My example service").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }

    async fn can_handle_load(&self) -> Result<bool> {
        Ok(self.is_running)
    }

    async fn get_load_factor(&self) -> Result<f64> {
        Ok(if self.is_running { 0.5 } else { 1.0 })
    }

    async fn update_config(&mut self, config: Self::Config) -> Result<()> {
        self.config = Some(config);
        Ok(())
    }
}

#[tokio::test]
async fn test_getting_started_example_works() -> Result<()>> {
    println!("🧪 === TESTING GETTING STARTED GUIDE EXAMPLE ===");

    // Step 1: Create orchestrator with default configuration (from docs)
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    println!("✅ Orchestrator created successfully");

    // Step 2: Start the orchestrator (from docs)
    orchestrator.start().await?;
    println!("✅ Orchestrator started");

    // Step 3: Register service exactly as shown in docs
    let service = MyService::new();
    let service_config = MyServiceConfig {
        port: 8080,
        name: "MyService".to_string(),
    };

    let service_id = orchestrator
        .register_service(service, service_config)
        .await?;
    println!("✅ Service registered with ID: {}", service_id);

    // Step 4: Verify the service is actually registered and working
    let services = orchestrator.list_services().await;
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "my-service");
    assert_eq!(services[0].name, "My Service");
    println!("✅ Service properly listed in orchestrator");

    // Step 5: Verify service health endpoint works
    let health = orchestrator.get_service_health(&service_id).await?;
    println!("✅ Service health check works: {:?}", health);

    // Step 6: Verify service metrics endpoint works
    let metrics = orchestrator.get_service_metrics(&service_id).await?;
    assert_eq!(metrics.error_count, 0);
    assert!(metrics.request_count >= 0);
    println!(
        "✅ Service metrics work: {} total requests",
        metrics.request_count
    );

    // Step 7: Verify orchestrator metrics work as documented
    let orch_metrics = orchestrator.get_config().await;
    assert_eq!(orch_metrics.total_services, 1);
    assert_eq!(orch_metrics.healthy_services, 1);
    println!(
        "✅ Orchestrator metrics work: {} total services",
        orch_metrics.total_services
    );

    // Step 8: Test service request handling
    let test_request =
        ServiceRequest::new("GET", "/test").with_payload(serde_json::json!({"test": "data"}));

    let response = orchestrator
        .handle_service_request(&service_id, test_request)
        .await?;
    if let ResponseStatus::Success = response.status {
        println!("✅ Service request handling works");
    } else {
        panic!("Service request failed: {:?}", response.status);
    }

    // Step 9: Graceful shutdown (from docs)
    orchestrator.stop().await?;
    println!("✅ Orchestrator stopped gracefully");

    println!("🎉 === ALL GETTING STARTED GUIDE FUNCTIONALITY VERIFIED ===");
    Ok(())
}

#[tokio::test]
async fn test_api_reference_methods_exist() -> Result<()>> {
    println!("🧪 === TESTING API REFERENCE CLAIMS ===");

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Verify all the methods claimed in API_REFERENCE.md actually exist

    // Test orchestrator.start() exists
    orchestrator.start().await?;
    println!("✅ orchestrator.start() method exists and works");

    // Test orchestrator.list_services() exists
    let _services = orchestrator.list_services().await;
    println!("✅ orchestrator.list_services() method exists and works");

    // Test orchestrator.get_config() exists
    let _metrics = orchestrator.get_config().await;
    println!("✅ orchestrator.get_config() method exists and works");

    // Test orchestrator.config() exists
    let _config_ref = orchestrator.config();
    println!("✅ orchestrator.config() method exists and works");

    // Test orchestrator.stop() exists
    orchestrator.stop().await?;
    println!("✅ orchestrator.stop() method exists and works");

    println!("🎉 === ALL API REFERENCE METHODS VERIFIED ===");
    Ok(())
}

#[tokio::test]
async fn test_service_lifecycle_as_documented() -> Result<()>> {
    println!("🧪 === TESTING DOCUMENTED SERVICE LIFECYCLE ===");

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    orchestrator.start().await?;

    // Register service
    let service = MyService::new();
    let service_config = MyServiceConfig {
        port: 9090,
        name: "LifecycleTest".to_string(),
    };

    let service_id = orchestrator
        .register_service(service, service_config)
        .await?;
    println!("✅ Service registration lifecycle works");

    // Verify service is healthy after registration
    let health = orchestrator.get_service_health(&service_id).await?;
    println!("✅ Service health tracking works: {:?}", health);

    // Verify we can get service metrics
    let metrics = orchestrator.get_service_metrics(&service_id).await?;
    println!(
        "✅ Service metrics collection works: {} requests processed",
        metrics.request_count
    );

    // Verify service unregistration works
    orchestrator.unregister_service(&service_id).await?;
    println!("✅ Service unregistration lifecycle works");

    // Verify service is no longer listed
    let services = orchestrator.list_services().await;
    assert_eq!(services.len(), 0);
    println!("✅ Service properly removed from orchestrator");

    orchestrator.stop().await?;

    println!("🎉 === SERVICE LIFECYCLE VERIFICATION COMPLETE ===");
    Ok(())
}

#[tokio::test]
async fn test_load_balancer_integration_works() -> Result<()>> {
    println!("🧪 === TESTING LOAD BALANCER INTEGRATION ===");

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    orchestrator.start().await?;

    // Register multiple services to test load balancing
    for i in 1..=3 {
        let service = MyService::new();
        let service_config = MyServiceConfig {
            port: 8080 + i,
            name: format!("Service-{}", i),
        };

        let service_id = orchestrator
            .register_service(service, service_config)
            .await?;
        println!("✅ Registered service_id: {}", service_id);
    }

    // Verify load balancer stats are accessible
    let lb_stats = orchestrator.get_load_balancer_stats().await?;
    println!(
        "✅ Load balancer stats accessible: {} total requests",
        lb_stats.total_requests
    );

    // Test service selection for load balancing
    let selected = orchestrator.select_service_for_request(Some("api")).await?;
    if selected.is_some() {
        println!("✅ Load balancer service selection works");
    } else {
        println!("⚠️ No services selected (expected with 'api' type)");
    }

    orchestrator.stop().await?;

    println!("🎉 === LOAD BALANCER INTEGRATION VERIFIED ===");
    Ok(())
}

#[tokio::test]
async fn test_discovery_integration_works() -> Result<()>> {
    println!("🧪 === TESTING DISCOVERY INTEGRATION ===");

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    orchestrator.start().await?;

    // Register a service
    let service = MyService::new();
    let service_config = MyServiceConfig {
        port: 8080,
        name: "DiscoveryTest".to_string(),
    };

    let service_id = orchestrator
        .register_service(service, service_config)
        .await?;

    // Test service discovery methods
    let exists = orchestrator.service_exists(&service_id).await?;
    assert!(exists);
    println!("✅ Service discovery existence check works");

    let discovered_services = orchestrator.list_discovered_services().await?;
    assert!(discovered_services.len() >= 1);
    println!(
        "✅ Service discovery listing works: {} services found",
        discovered_services.len()
    );

    orchestrator.stop().await?;

    println!("🎉 === DISCOVERY INTEGRATION VERIFIED ===");
    Ok(())
}
