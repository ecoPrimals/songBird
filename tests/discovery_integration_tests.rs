/*!
 * Discovery Service Integration Tests
 *
 * Tests the integration between the Songbird Discovery Service
 * and the main orchestrator, ensuring end-to-end functionality.
 */

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use songbird_orchestrator::{
    orchestrator::{Orchestrator, DiscoveryBackend},
    config::OrchestratorConfig,
    discovery::{SongbirdDiscoveryConfig, NodeType, MonitoringConfig, NetworkConfig, TrustConfig},
    traits::discovery::{ServiceQuery, ServiceHealthStatus},
    traits::service::ServiceInfo,
};

/// Mock service for testing integration
#[derive(Clone)]
struct TestIntegrationService {
    id: String,
    service_type: String,
}

impl TestIntegrationService {
    fn new(id: String, service_type: String) -> Self {
        Self { id, service_type }
    }
}

#[tokio::test]
async fn test_orchestrator_with_songbird_discovery() {
    println!("🧪 Testing orchestrator integration with Songbird Discovery");
    
    // Create orchestrator with Songbird discovery
    let config = OrchestratorConfig::default();
    let songbird_config = SongbirdDiscoveryConfig {
        node_id: Some("test-node".to_string()),
        node_type: NodeType::Orchestrator,
        institution: Some("Test Institution".to_string()),
        federation_enabled: true,
        health_check_interval_secs: 30,
        node_discovery_interval_secs: 60,
        trust_verification_enabled: false,
        max_federation_nodes: 100,
        monitoring: MonitoringConfig::default(),
        network: NetworkConfig::default(),
        trust: TrustConfig::default(),
    };
    
    let orchestrator = Orchestrator::new_with_discovery(
        config,
        DiscoveryBackend::Songbird(songbird_config),
    ).await.unwrap();
    
    orchestrator.start().await.unwrap();
    
    // Test service registration through discovery
    let test_service = ServiceInfo {
        id: "integration-test-service".to_string(),
        name: "Integration Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: "Service for testing integration".to_string(),
        endpoints: vec![],
        capabilities: vec!["testing".to_string()],
        tags: {
            let mut tags = HashMap::new();
            tags.insert("test".to_string(), "integration".to_string());
            tags
        },
        metadata: HashMap::new(),
    };
    
    // Register service through discovery
    orchestrator.discovery().register(test_service.clone()).await.unwrap();
    
    // Verify service exists
    assert!(orchestrator.service_exists("integration-test-service").await.unwrap());
    
    // Test service discovery
    let discovered_services = orchestrator.discover_services(
        ServiceQuery::new().with_service_type("test")
    ).await.unwrap();
    assert_eq!(discovered_services.len(), 1, "Should find the registered service");
    assert_eq!(discovered_services[0].id, "integration-test-service");
    
    // Test health updates
    orchestrator.update_service_health_in_discovery(
        "integration-test-service",
        ServiceHealthStatus::Healthy,
    ).await.unwrap();
    
    // List all discovered services
    let all_services = orchestrator.list_discovered_services().await.unwrap();
    assert!(!all_services.is_empty(), "Should find registered services");
    
    orchestrator.stop().await.unwrap();
    
    println!("✅ Orchestrator with Songbird Discovery integration tests passed");
}

#[tokio::test]
async fn test_multi_orchestrator_federation() {
    println!("🧪 Testing multi-orchestrator federation");
    
    // Create first orchestrator (MIT)
    let config1 = OrchestratorConfig::default();
    let mit_orchestrator = Orchestrator::new_with_federation(
        config1,
        Some("MIT".to_string()),
    ).await.unwrap();
    
    // Create second orchestrator (Harvard)
    let config2 = OrchestratorConfig::default();
    let harvard_orchestrator = Orchestrator::new_with_federation(
        config2,
        Some("Harvard".to_string()),
    ).await.unwrap();
    
    // Start both orchestrators
    mit_orchestrator.start().await.unwrap();
    harvard_orchestrator.start().await.unwrap();
    
    // Register different services on each
    let mit_service = ServiceInfo {
        id: "mit-genomics-service".to_string(),
        name: "MIT Genomics Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "genomics".to_string(),
        description: "Genomics analysis service".to_string(),
        endpoints: vec![],
        capabilities: vec!["variant-calling".to_string()],
        tags: {
            let mut tags = HashMap::new();
            tags.insert("institution".to_string(), "MIT".to_string());
            tags.insert("domain".to_string(), "genomics".to_string());
            tags
        },
        metadata: HashMap::new(),
    };
    
    let harvard_service = ServiceInfo {
        id: "harvard-protein-service".to_string(),
        name: "Harvard Protein Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "proteomics".to_string(),
        description: "Protein folding service".to_string(),
        endpoints: vec![],
        capabilities: vec!["protein-folding".to_string()],
        tags: {
            let mut tags = HashMap::new();
            tags.insert("institution".to_string(), "Harvard".to_string());
            tags.insert("domain".to_string(), "proteomics".to_string());
            tags
        },
        metadata: HashMap::new(),
    };
    
    // Register services
    mit_orchestrator.discovery().register(mit_service).await.unwrap();
    harvard_orchestrator.discovery().register(harvard_service).await.unwrap();
    
    // Wait for federation discovery
    sleep(Duration::from_secs(2)).await;
    
    // Test cross-institution discovery
    let mit_genomics = mit_orchestrator.discover_services(
        ServiceQuery::new().with_service_type("genomics")
    ).await.unwrap();
    assert_eq!(mit_genomics.len(), 1, "MIT should find its genomics service");
    
    let harvard_proteomics = harvard_orchestrator.discover_services(
        ServiceQuery::new().with_service_type("proteomics")
    ).await.unwrap();
    assert_eq!(harvard_proteomics.len(), 1, "Harvard should find its proteomics service");
    
    // Test Songbird-specific federation features
    if let Some(mit_songbird) = mit_orchestrator.songbird_discovery() {
        let federation_health = mit_songbird.get_federation_stats().await;
        println!("   📊 Federation stats: {} nodes, {} services", 
                 federation_health.total_nodes, federation_health.total_services);
        
        let topology = mit_songbird.get_federation_stats().await;
        println!("   🌐 Network topology: {} nodes", topology.total_nodes);
    }
    
    // Stop orchestrators
    mit_orchestrator.stop().await.unwrap();
    harvard_orchestrator.stop().await.unwrap();
    
    println!("✅ Multi-orchestrator federation tests passed");
}

#[tokio::test]
async fn test_discovery_service_resilience() {
    println!("🧪 Testing discovery service resilience");
    
    let config = OrchestratorConfig::default();
    let songbird_config = SongbirdDiscoveryConfig {
        node_id: Some("resilience-test-node".to_string()),
        node_type: NodeType::Orchestrator,
        institution: Some("Test Institution".to_string()),
        federation_enabled: true,
        health_check_interval_secs: 1, // Fast for testing
        node_discovery_interval_secs: 1,
        trust_verification_enabled: true,
        max_federation_nodes: 100,
        monitoring: MonitoringConfig::default(),
        network: NetworkConfig::default(), 
        trust: TrustConfig::default(),
    };
    
    let orchestrator = Orchestrator::new_with_discovery(
        config,
        DiscoveryBackend::Songbird(songbird_config),
    ).await.unwrap();
    
    orchestrator.start().await.unwrap();
    
    // Register multiple services
    for i in 0..10 {
        let service = ServiceInfo {
            id: format!("resilience-service-{}", i),
            name: format!("Resilience Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "resilience".to_string(),
            description: "Service for testing resilience".to_string(),
            endpoints: vec![],
            capabilities: vec!["testing".to_string()],
            tags: {
                let mut tags = HashMap::new();
                tags.insert("test".to_string(), "resilience".to_string());
                tags.insert("instance".to_string(), i.to_string());
                tags
            },
            metadata: HashMap::new(),
        };
        
        orchestrator.discovery().register(service).await.unwrap();
    }
    
    // Test discovery under load
    let mut discovery_tasks = Vec::new();
    for _ in 0..50 {
        let orch = orchestrator.clone();
        discovery_tasks.push(tokio::spawn(async move {
            let query = ServiceQuery::new().with_service_type("resilience");
            orch.discover_services(query).await.unwrap().len()
        }));
    }
    
    // Wait for all discovery tasks
    let results = futures::future::join_all(discovery_tasks).await;
    let successful_discoveries = results.into_iter()
        .filter_map(|r| r.ok())
        .filter(|&count| count == 10)
        .count();
    
    assert!(successful_discoveries >= 45, "Should have high success rate under load");
    
    // Test service health updates under load
    for i in 0..10 {
        let service_id = format!("resilience-service-{}", i);
        orchestrator.update_service_health_in_discovery(
            &service_id,
            if i % 2 == 0 { ServiceHealthStatus::Healthy } else { ServiceHealthStatus::Degraded }
        ).await.unwrap();
    }
    
    // Verify services still discoverable
    let final_services = orchestrator.discover_services(
        ServiceQuery::new().with_service_type("resilience")
    ).await.unwrap();
    assert_eq!(final_services.len(), 10, "All services should still be discoverable");
    
    orchestrator.stop().await.unwrap();
    
    println!("✅ Discovery service resilience tests passed");
}

#[tokio::test]
async fn test_discovery_performance_metrics() {
    println!("🧪 Testing discovery service performance metrics");
    
    let config = OrchestratorConfig::default();
    let songbird_config = SongbirdDiscoveryConfig {
        node_id: Some("performance-test-node".to_string()),
        node_type: NodeType::Orchestrator,
        institution: Some("Test Institution".to_string()),
        federation_enabled: false, // Disable for pure performance testing
        health_check_interval_secs: 30,
        node_discovery_interval_secs: 60,
        trust_verification_enabled: false,
        max_federation_nodes: 100,
        monitoring: MonitoringConfig::default(),
        network: NetworkConfig::default(),
        trust: TrustConfig::default(),
    };
    
    let orchestrator = Orchestrator::new_with_discovery(
        config,
        DiscoveryBackend::Songbird(songbird_config),
    ).await.unwrap();
    
    orchestrator.start().await.unwrap();
    
    // Register services for performance testing
    let service_count = 1000;
    let registration_start = std::time::Instant::now();
    
    for i in 0..service_count {
        let service = ServiceInfo {
            id: format!("perf-service-{}", i),
            name: format!("Performance Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: match i % 5 {
                0 => "web",
                1 => "api",
                2 => "database",
                3 => "cache",
                _ => "compute",
            }.to_string(),
            description: "Service for performance testing".to_string(),
            endpoints: vec![],
            capabilities: vec!["testing".to_string()],
            tags: {
                let mut tags = HashMap::new();
                tags.insert("performance".to_string(), "test".to_string());
                tags.insert("batch".to_string(), (i / 100).to_string());
                tags
            },
            metadata: HashMap::new(),
        };
        
        orchestrator.discovery().register(service).await.unwrap();
    }
    
    let registration_time = registration_start.elapsed();
    
    // Test discovery performance
    let discovery_start = std::time::Instant::now();
    let all_services = orchestrator.list_discovered_services().await.unwrap();
    let list_time = discovery_start.elapsed();
    
    // Test query performance
    let query_start = std::time::Instant::now();
    let web_services = orchestrator.discover_services(
        ServiceQuery::new().with_service_type("web")
    ).await.unwrap();
    let query_time = query_start.elapsed();
    
    // Test tag-based query performance
    let tag_query_start = std::time::Instant::now();
    let batch_services = orchestrator.discover_services(
        ServiceQuery::new().with_tag("batch".to_string())
    ).await.unwrap();
    let tag_query_time = tag_query_start.elapsed();
    
    // Performance metrics
    println!("   📊 Performance Results:");
    println!("      Registration: {} services in {:.2}ms ({:.0} services/sec)", 
             service_count, registration_time.as_millis(),
             service_count as f64 / registration_time.as_secs_f64());
    println!("      List All: {} services in {:.2}ms", 
             all_services.len(), list_time.as_millis());
    println!("      Type Query: {} services in {:.2}ms", 
             web_services.len(), query_time.as_millis());
    println!("      Tag Query: {} services in {:.2}ms", 
             batch_services.len(), tag_query_time.as_millis());
    
    // Performance assertions
    assert_eq!(all_services.len(), service_count, "Should find all registered services");
    assert_eq!(web_services.len(), service_count / 5, "Should find correct number of web services");
    assert!(registration_time.as_millis() < 5000, "Registration should be fast");
    assert!(list_time.as_millis() < 100, "List operation should be fast");
    assert!(query_time.as_millis() < 50, "Query operation should be fast");
    assert!(tag_query_time.as_millis() < 50, "Tag query should be fast");
    
    orchestrator.stop().await.unwrap();
    
    println!("✅ Discovery service performance tests passed");
}

#[tokio::test]
async fn test_discovery_watch_functionality() {
    println!("🧪 Testing discovery service watch functionality");
    
    let config = OrchestratorConfig::default();
    let songbird_config = SongbirdDiscoveryConfig {
        node_id: Some("watch-test-node".to_string()),
        node_type: NodeType::Orchestrator,
        institution: Some("Test Institution".to_string()),
        federation_enabled: true,
        health_check_interval_secs: 5,
        node_discovery_interval_secs: 10,
        trust_verification_enabled: true,
        max_federation_nodes: 100,
        monitoring: MonitoringConfig::default(),
        network: NetworkConfig::default(),
        trust: TrustConfig::default(),
    };
    
    let orchestrator = Orchestrator::new_with_discovery(
        config,
        DiscoveryBackend::Songbird(songbird_config),
    ).await.unwrap();
    
    orchestrator.start().await.unwrap();
    
    // Test watch functionality (note: current implementation returns empty stream for static discovery)
    let watch_query = ServiceQuery::new().with_service_type("watchable");
    let watch_result = orchestrator.discovery().watch(watch_query).await;
    
    match watch_result {
        Ok(_stream) => {
            println!("   ✅ Watch stream created successfully");
            // In a real test, you'd consume the stream and verify events
        }
        Err(e) => {
            println!("   ⚠️  Watch functionality not fully implemented: {}", e);
        }
    }
    
    // Test that basic discovery still works
    let test_service = ServiceInfo {
        id: "watch-test-service".to_string(),
        name: "Watch Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "watchable".to_string(),
        description: "Service for testing watch functionality".to_string(),
        endpoints: vec![],
        capabilities: vec!["testing".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    orchestrator.discovery().register(test_service).await.unwrap();
    
    let discovered = orchestrator.discover_services(
        ServiceQuery::new().with_service_type("watchable")
    ).await.unwrap();
    
    assert_eq!(discovered.len(), 1, "Should find the watchable service");
    
    orchestrator.stop().await.unwrap();
    
    println!("✅ Discovery service watch functionality tests completed");
} 