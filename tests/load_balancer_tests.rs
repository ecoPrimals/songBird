use songbird_orchestrator::{
    load_balancer::{
        DefaultLoadBalancer, LoadBalancer, LoadBalancerConfig, LoadBalancerStrategy, ServiceInstance,
    },
    traits::service::{ServiceInfo, ServiceEndpoint},
};
use std::time::Duration;

fn create_test_service_instance(
    id: &str,
    name: &str,
    service_type: &str,
    weight: u32,
    connections: u32,
    healthy: bool,
) -> ServiceInstance {
    ServiceInstance {
        service_info: ServiceInfo {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: service_type.to_string(),
            description: format!("Test service {}", name),
            endpoints: vec![],
            capabilities: vec![],
            tags: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        },
        weight,
        current_connections: connections,
        is_healthy: healthy,
    }
}

#[tokio::test]
async fn test_round_robin_selection() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![
        create_test_service_instance("service1", "Test Service 1", "test", 1, 0, true),
        create_test_service_instance("service2", "Test Service 2", "test", 1, 0, true),
        create_test_service_instance("service3", "Test Service 3", "test", 1, 0, true),
    ];

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Should cycle through services in order
    let selected1 = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert_eq!(selected1.service_info.id, "service1");

    let selected2 = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert_eq!(selected2.service_info.id, "service2");

    let selected3 = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert_eq!(selected3.service_info.id, "service3");

    // Should cycle back to first service
    let selected4 = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert_eq!(selected4.service_info.id, "service1");
}

#[tokio::test]
async fn test_least_connections_load_balancing() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::LeastConnections,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![
        create_test_service_instance("service1", "Test Service 1", "test", 1, 0, true),
        create_test_service_instance("service2", "Test Service 2", "test", 1, 5, true), // More connections
        create_test_service_instance("service3", "Test Service 3", "test", 1, 2, true),
    ];

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Should select service with least connections (service1 with 0 connections)
    let selected = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert_eq!(selected.service_info.id, "service1");
    assert_eq!(selected.current_connections, 0);
}

#[tokio::test]
async fn test_weighted_round_robin_load_balancing() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::WeightedRoundRobin,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![
        create_test_service_instance("service1", "Test Service 1", "test", 1, 0, true), // Low weight
        create_test_service_instance("service2", "Test Service 2", "test", 5, 0, true), // High weight
        create_test_service_instance("service3", "Test Service 3", "test", 1, 0, true), // Low weight
    ];

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Run multiple selections and count results
    let mut selections = std::collections::HashMap::new();
    for _ in 0..100 {
        if let Ok(Some(selected)) = lb_trait.select_service(&services).await {
            *selections.entry(selected.service_info.id).or_insert(0) += 1;
        }
    }

    // Service2 should be selected more often due to higher weight
    let service2_count = selections.get("service2").unwrap_or(&0);
    let service1_count = selections.get("service1").unwrap_or(&0);

    assert!(
        *service2_count > *service1_count,
        "Service2 (weight 5) should be selected more than Service1 (weight 1)"
    );
}

#[tokio::test]
async fn test_random_load_balancing() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::Random,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![
        create_test_service_instance("service1", "Test Service 1", "test", 1, 0, true),
        create_test_service_instance("service2", "Test Service 2", "test", 1, 0, true),
        create_test_service_instance("service3", "Test Service 3", "test", 1, 0, true),
    ];

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Should only select healthy services
    let selected = lb_trait.select_service(&services).await.unwrap().unwrap();
    assert!(["service1", "service2", "service3"].contains(&selected.service_info.id.as_str()));
    assert!(selected.is_healthy);
}

#[tokio::test]
async fn test_empty_services_list() {
    let config = LoadBalancerConfig::default();
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![];
    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;
    
    let selected = lb_trait.select_service(&services).await.unwrap();
    assert!(selected.is_none());
}

#[tokio::test]
async fn test_load_balancer_stats() {
    let config = LoadBalancerConfig::default();
    let lb = DefaultLoadBalancer::new(config);

    let services = vec![create_test_service_instance(
        "service1",
        "Test Service 1",
        "test",
        1,
        0,
        true,
    )];

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Make some selections
    lb_trait.select_service(&services).await.unwrap();
    lb_trait.select_service(&services).await.unwrap();
    lb_trait.select_service(&vec![]).await.unwrap(); // This should return None

    let stats = lb_trait.get_stats().await.unwrap();
    assert_eq!(stats.total_requests, 3);
    assert_eq!(stats.successful_requests, 2);
    assert_eq!(stats.failed_requests, 1);
}

#[tokio::test]
async fn test_service_health_updates() {
    let config = LoadBalancerConfig::default();
    let lb = DefaultLoadBalancer::new(config);

    // Cast to trait to use trait method
    let lb_trait: &dyn LoadBalancer = &lb;

    // Test health update functionality
    let result = lb_trait.update_service_health("service1", true).await;
    assert!(result.is_ok());

    let result = lb_trait.update_service_health("service2", false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_balancer_config_defaults() {
    let config = LoadBalancerConfig::default();

    assert!(matches!(config.strategy, LoadBalancerStrategy::RoundRobin));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
}

#[tokio::test]
async fn test_service_instance_creation() {
    let instance = create_test_service_instance("test-service", "Test Service", "test", 1, 0, true);

    assert_eq!(instance.service_info.id, "test-service");
    assert_eq!(instance.service_info.name, "Test Service");
    assert_eq!(instance.service_info.version, "1.0.0");
    assert_eq!(instance.service_info.service_type, "test");
    assert_eq!(
        instance.service_info.description,
        "Test service Test Service"
    );
    assert_eq!(instance.service_info.endpoints, vec![]);
    assert_eq!(instance.service_info.capabilities, vec![] as Vec<String>);
    assert_eq!(instance.service_info.tags, std::collections::HashMap::new());
    assert_eq!(
        instance.service_info.metadata,
        std::collections::HashMap::new()
    );
}

#[tokio::test]
async fn test_load_balancer_config_validation() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(30),
        max_retries: 3,
        timeout: Duration::from_secs(10),
    };

    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.max_retries, 3);
}
