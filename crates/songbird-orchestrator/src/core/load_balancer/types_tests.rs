// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for load balancer types

#[cfg(test)]
use songbird_types::{SongbirdError, SongbirdResult};
mod tests {
    #![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]

    use super::super::{LoadBalancingStrategy, ServiceInstance};

    #[test]
    fn test_round_robin_strategy() -> SongbirdResult<()> {
        let strategy = LoadBalancingStrategy::RoundRobin;
        assert_eq!(format!("{:?}", strategy), "RoundRobin");
        Ok(())
    }

    #[test]
    fn test_least_connections_strategy() -> SongbirdResult<()> {
        let strategy = LoadBalancingStrategy::LeastConnections;
        assert_eq!(format!("{:?}", strategy), "LeastConnections");
        Ok(())
    }

    #[test]
    fn test_weighted_round_robin_strategy() -> SongbirdResult<()> {
        let strategy = LoadBalancingStrategy::WeightedRoundRobin;
        assert_eq!(format!("{:?}", strategy), "WeightedRoundRobin");
        Ok(())
    }

    #[test]
    fn test_random_strategy() -> SongbirdResult<()> {
        let strategy = LoadBalancingStrategy::Random;
        assert_eq!(format!("{:?}", strategy), "Random");
        Ok(())
    }

    #[test]
    fn test_strategy_clone() -> SongbirdResult<()> {
        let strategy1 = LoadBalancingStrategy::RoundRobin;
        let strategy2 = strategy1.clone();
        assert_eq!(format!("{:?}", strategy1), format!("{:?}", strategy2));
        Ok(())
    }

    #[test]
    fn test_service_instance_creation() {
        let instance = ServiceInstance {
            id: "instance-1".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            weight: 1.0,
            active_connections: 0,
            healthy: true,
        };
        
        assert_eq!(instance.id, "instance-1");
        assert_eq!(instance.endpoint, "http://localhost:8080");
        assert_eq!(instance.weight, 1.0);
        assert!(instance.healthy);
    }

    #[test]
    fn test_service_instance_unhealthy() {
        let instance = ServiceInstance {
            id: "instance-2".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            weight: 1.0,
            active_connections: 5,
            healthy: false,
        };
        
        assert!(!instance.healthy);
        assert_eq!(instance.active_connections, 5);
    }

    #[test]
    fn test_service_instance_with_weight() {
        let instance = ServiceInstance {
            id: "high-priority".to_string(),
            endpoint: "http://localhost:8082".to_string(),
            weight: 5.0,
            active_connections: 0,
            healthy: true,
        };
        
        assert_eq!(instance.weight, 5.0);
    }

    #[test]
    fn test_service_instance_clone() {
        let instance1 = ServiceInstance {
            id: "test".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            weight: 1.0,
            active_connections: 2,
            healthy: true,
        };
        
        let instance2 = instance1.clone();
        assert_eq!(instance1.id, instance2.id);
        assert_eq!(instance1.endpoint, instance2.endpoint);
    }

    #[test]
    fn test_multiple_instances() {
        let instances = vec![
            ServiceInstance {
                id: "instance-1".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                weight: 1.0,
                active_connections: 0,
                healthy: true,
            },
            ServiceInstance {
                id: "instance-2".to_string(),
                endpoint: "http://localhost:8081".to_string(),
                weight: 2.0,
                active_connections: 5,
                healthy: true,
            },
            ServiceInstance {
                id: "instance-3".to_string(),
                endpoint: "http://localhost:8082".to_string(),
                weight: 1.0,
                active_connections: 10,
                healthy: false,
            },
        ];
        
        assert_eq!(instances.len(), 3);
        assert_eq!(instances[0].active_connections, 0);
        assert_eq!(instances[1].weight, 2.0);
        assert!(!instances[2].healthy);
    }

    #[test]
    fn test_service_instance_zero_weight() -> SongbirdResult<()> {
        let instance = ServiceInstance {
            id: "disabled".to_string(),
            endpoint: "http://localhost:8083".to_string(),
            weight: 0.0,
            active_connections: 0,
            healthy: true,
        };
        
        assert_eq!(instance.weight, 0.0);
        Ok(())
    }

    #[test]
    fn test_service_instance_high_connections() -> SongbirdResult<()> {
        let instance = ServiceInstance {
            id: "busy".to_string(),
            endpoint: "http://localhost:8084".to_string(),
            weight: 1.0,
            active_connections: 1000,
            healthy: true,
        };
        
        assert_eq!(instance.active_connections, 1000);
        Ok(())
    }

    #[test]
    fn test_load_balancing_strategy_debug() -> SongbirdResult<()> {
        let strategy = LoadBalancingStrategy::LeastConnections;
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("LeastConnections"));
        Ok(())
    }

    #[test]
    fn test_service_instance_debug() -> SongbirdResult<()> {
        let instance = ServiceInstance {
            id: "test-debug".to_string(),
            endpoint: "http://localhost:8085".to_string(),
            weight: 1.5,
            active_connections: 3,
            healthy: true,
        };
        
        let debug_str = format!("{:?}", instance);
        assert!(debug_str.contains("test-debug"));
        Ok(())
    }
}

