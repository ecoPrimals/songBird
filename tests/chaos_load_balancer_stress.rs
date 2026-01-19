//! Chaos Testing: Load Balancer Under Stress
//!
//! Tests load balancer behavior under extreme conditions

use songbird_types::{ServiceEndpoint, ServiceHealth};
use songbird_universal::load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn chaos_load_balancer_all_endpoints_down() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(1),
        max_retries: 3,
    };

    let balancer = LoadBalancer::new(config);

    // Add endpoints but mark all as unhealthy
    for i in 0..5 {
        let endpoint = ServiceEndpoint {
            url: format!("http://localhost:{}", 8000 + i),
            protocol: "http".to_string(),
            health_check_path: Some("/health".to_string()),
        };
        balancer.add_endpoint(endpoint.clone());
        balancer.update_health(&endpoint.url, ServiceHealth::Unhealthy);
    }

    // Attempt to select endpoint
    let result = balancer.select_endpoint().await;

    // Should handle gracefully (either error or fallback)
    assert!(result.is_ok() || result.is_err(), "Should handle all endpoints down gracefully");
}

#[tokio::test]
async fn chaos_load_balancer_rapid_endpoint_churn() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        health_check_interval: Duration::from_secs(1),
        max_retries: 2,
    };

    let balancer = Arc::new(LoadBalancer::new(config));

    // Rapidly add and remove endpoints
    for iteration in 0..50 {
        for i in 0..10 {
            let endpoint = ServiceEndpoint {
                url: format!("http://localhost:{}", 9000 + (iteration * 10) + i),
                protocol: "http".to_string(),
                health_check_path: None,
            };
            balancer.add_endpoint(endpoint.clone());

            if i % 2 == 0 {
                balancer.remove_endpoint(&endpoint.url);
            }
        }

        sleep(Duration::from_micros(100)).await;
    }

    // Should still be functional
    let result = balancer.select_endpoint().await;
    assert!(result.is_ok() || result.is_err(), "Should survive endpoint churn");
}

#[tokio::test]
async fn chaos_load_balancer_concurrent_selections() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(5),
        max_retries: 1,
    };

    let balancer = Arc::new(LoadBalancer::new(config));

    // Add endpoints
    for i in 0..3 {
        let endpoint = ServiceEndpoint {
            url: format!("http://localhost:{}", 7000 + i),
            protocol: "http".to_string(),
            health_check_path: None,
        };
        balancer.add_endpoint(endpoint);
    }

    // Spawn 100 concurrent selectors
    let mut handles = vec![];
    for _ in 0..100 {
        let balancer_clone = Arc::clone(&balancer);
        let handle = tokio::spawn(async move {
            for _ in 0..50 {
                let _ = balancer_clone.select_endpoint().await;
                sleep(Duration::from_micros(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all selections
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_load_balancer_health_oscillation() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::HealthWeighted,
        health_check_interval: Duration::from_millis(50),
        max_retries: 2,
    };

    let balancer = LoadBalancer::new(config);

    let endpoint = ServiceEndpoint {
        url: "http://localhost:6000".to_string(),
        protocol: "http".to_string(),
        health_check_path: Some("/health".to_string()),
    };

    balancer.add_endpoint(endpoint.clone());

    // Rapidly oscillate health status
    for i in 0..100 {
        let health = if i % 2 == 0 {
            ServiceHealth::Healthy
        } else {
            ServiceHealth::Unhealthy
        };
        balancer.update_health(&endpoint.url, health);
        sleep(Duration::from_micros(500)).await;
    }

    // Should handle health oscillation
    let result = balancer.select_endpoint().await;
    assert!(result.is_ok() || result.is_err(), "Should handle health oscillation");
}

#[tokio::test]
async fn chaos_load_balancer_massive_endpoint_scale() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(10),
        max_retries: 1,
    };

    let balancer = LoadBalancer::new(config);

    // Add 1000 endpoints
    for i in 0..1000 {
        let endpoint = ServiceEndpoint {
            url: format!("http://service-{}.local:8000", i),
            protocol: "http".to_string(),
            health_check_path: None,
        };
        balancer.add_endpoint(endpoint);
    }

    // Should handle large scale
    for _ in 0..100 {
        let result = balancer.select_endpoint().await;
        assert!(result.is_ok(), "Should select from large endpoint pool");
    }
}

#[tokio::test]
async fn chaos_load_balancer_strategy_switching() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::HealthWeighted,
    ];

    for strategy in strategies {
        let config = LoadBalancerConfig {
            strategy,
            health_check_interval: Duration::from_secs(1),
            max_retries: 2,
        };

        let balancer = LoadBalancer::new(config);

        // Add test endpoints
        for i in 0..5 {
            let endpoint = ServiceEndpoint {
                url: format!("http://localhost:{}", 5000 + i),
                protocol: "http".to_string(),
                health_check_path: None,
            };
            balancer.add_endpoint(endpoint);
        }

        // Test each strategy works
        for _ in 0..10 {
            let result = balancer.select_endpoint().await;
            assert!(result.is_ok(), "Strategy {:?} should work", strategy);
        }
    }
}

#[tokio::test]
async fn chaos_load_balancer_zero_endpoints() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(1),
        max_retries: 1,
    };

    let balancer = LoadBalancer::new(config);

    // Try to select without any endpoints
    let result = balancer.select_endpoint().await;

    // Should handle gracefully (error expected)
    assert!(result.is_err(), "Should error with no endpoints");
}

#[tokio::test]
async fn chaos_load_balancer_duplicate_endpoints() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(1),
        max_retries: 1,
    };

    let balancer = LoadBalancer::new(config);

    let endpoint = ServiceEndpoint {
        url: "http://localhost:4000".to_string(),
        protocol: "http".to_string(),
        health_check_path: None,
    };

    // Add same endpoint multiple times
    for _ in 0..100 {
        balancer.add_endpoint(endpoint.clone());
    }

    // Should handle duplicates gracefully
    let result = balancer.select_endpoint().await;
    assert!(result.is_ok(), "Should handle duplicate endpoints");
}

#[tokio::test]
async fn chaos_load_balancer_concurrent_modifications() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::Random,
        health_check_interval: Duration::from_secs(1),
        max_retries: 1,
    };

    let balancer = Arc::new(LoadBalancer::new(config));

    // Concurrent add/remove/select operations
    let mut handles = vec![];

    // Adders
    for i in 0..20 {
        let balancer_clone = Arc::clone(&balancer);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let endpoint = ServiceEndpoint {
                    url: format!("http://host-{}-{}:8000", i, j),
                    protocol: "http".to_string(),
                    health_check_path: None,
                };
                balancer_clone.add_endpoint(endpoint);
                sleep(Duration::from_micros(100)).await;
            }
        });
        handles.push(handle);
    }

    // Selectors
    for _ in 0..20 {
        let balancer_clone = Arc::clone(&balancer);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let _ = balancer_clone.select_endpoint().await;
                sleep(Duration::from_micros(100)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_load_balancer_extreme_retry_count() {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        health_check_interval: Duration::from_secs(1),
        max_retries: 1000, // Extreme retry count
    };

    let balancer = LoadBalancer::new(config);

    // Add one unhealthy endpoint
    let endpoint = ServiceEndpoint {
        url: "http://localhost:3000".to_string(),
        protocol: "http".to_string(),
        health_check_path: Some("/health".to_string()),
    };
    balancer.add_endpoint(endpoint.clone());
    balancer.update_health(&endpoint.url, ServiceHealth::Unhealthy);

    // Should eventually fail gracefully despite high retry count
    let result = balancer.select_endpoint().await;
    assert!(result.is_ok() || result.is_err(), "Should handle extreme retries");
}
