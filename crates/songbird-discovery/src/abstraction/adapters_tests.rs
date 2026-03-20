// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for discovery adapters
//!
//! Comprehensive test coverage for static, kubernetes, and consul adapters.

#[cfg(test)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::super::adapters::static_adapter::StaticDiscoveryAdapter;
    use super::super::registry::UniversalServiceRegistry;
    use songbird_types::ServiceInfo;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_static_adapter_creation() {
        let adapter = StaticDiscoveryAdapter::new();
        assert!(adapter.services.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_static_adapter_register_service() {
        let adapter = StaticDiscoveryAdapter::new();
        let service = create_test_service("test-service", "http://localhost:8080");
        
        let result = adapter.register_service(service.clone()).await;
        assert!(result.is_ok());
        
        let services = adapter.services.read().await;
        assert_eq!(services.len(), 1);
        assert!(services.contains_key("test-service"));
    }

    #[tokio::test]
    async fn test_static_adapter_register_multiple_services() {
        let adapter = StaticDiscoveryAdapter::new();
        
        for i in 0..5 {
            let service = create_test_service(
                &format!("service-{}", i),
                &format!("http://localhost:{}", 8080 + i),
            );
            adapter.register_service(service).await.unwrap();
        }
        
        let services = adapter.services.read().await;
        assert_eq!(services.len(), 5);
    }

    #[tokio::test]
    async fn test_static_adapter_deregister_service() {
        let adapter = StaticDiscoveryAdapter::new();
        let service = create_test_service("test-service", "http://localhost:8080");
        
        adapter.register_service(service).await.unwrap();
        assert_eq!(adapter.services.read().await.len(), 1);
        
        let result = adapter.deregister_service("test-service").await;
        assert!(result.is_ok());
        assert!(adapter.services.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_static_adapter_deregister_nonexistent_service() {
        let adapter = StaticDiscoveryAdapter::new();
        let result = adapter.deregister_service("nonexistent").await;
        // Should succeed even if service doesn't exist
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_static_adapter_get_service() {
        let adapter = StaticDiscoveryAdapter::new();
        let service = create_test_service("test-service", "http://localhost:8080");
        
        adapter.register_service(service.clone()).await.unwrap();
        
        let retrieved = adapter.get_service("test-service").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-service");
    }

    #[tokio::test]
    async fn test_static_adapter_get_nonexistent_service() {
        let adapter = StaticDiscoveryAdapter::new();
        let result = adapter.get_service("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_static_adapter_list_all_services() {
        let adapter = StaticDiscoveryAdapter::new();
        
        for i in 0..3 {
            let service = create_test_service(
                &format!("service-{}", i),
                &format!("http://localhost:{}", 8080 + i),
            );
            adapter.register_service(service).await.unwrap();
        }
        
        let all_services = adapter.list_services().await.unwrap();
        assert_eq!(all_services.len(), 3);
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = UniversalServiceRegistry::new();
        assert!(registry.services.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_registry_register_service() {
        let registry = UniversalServiceRegistry::new();
        let service = create_test_service("test-service", "http://localhost:8080");
        
        let result = registry.register(service).await;
        assert!(result.is_ok());
        
        let count = registry.count().await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_registry_discover_services() {
        let registry = UniversalServiceRegistry::new();
        
        for i in 0..3 {
            let service = create_test_service(
                &format!("service-{}", i),
                &format!("http://localhost:{}", 8080 + i),
            );
            registry.register(service).await.unwrap();
        }
        
        let discovered = registry.discover().await.unwrap();
        assert_eq!(discovered.len(), 3);
    }

    #[tokio::test]
    async fn test_registry_deregister_service() {
        let registry = UniversalServiceRegistry::new();
        let service = create_test_service("test-service", "http://localhost:8080");
        
        registry.register(service).await.unwrap();
        assert_eq!(registry.count().await, 1);
        
        let result = registry.deregister("test-service").await;
        assert!(result.is_ok());
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_find_service_by_name() {
        let registry = UniversalServiceRegistry::new();
        let service = create_test_service("findme", "http://localhost:8080");
        
        registry.register(service).await.unwrap();
        
        let found = registry.find("findme").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "findme");
    }

    #[tokio::test]
    async fn test_registry_find_nonexistent_service() {
        let registry = UniversalServiceRegistry::new();
        let found = registry.find("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_registry_clear_all_services() {
        let registry = UniversalServiceRegistry::new();
        
        for i in 0..5 {
            let service = create_test_service(
                &format!("service-{}", i),
                &format!("http://localhost:{}", 8080 + i),
            );
            registry.register(service).await.unwrap();
        }
        
        assert_eq!(registry.count().await, 5);
        registry.clear().await.unwrap();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_concurrent_registrations() {
        use tokio::task::JoinSet;
        
        let registry = std::sync::Arc::new(UniversalServiceRegistry::new());
        let mut set = JoinSet::new();
        
        for i in 0..10 {
            let reg = registry.clone();
            set.spawn(async move {
                let service = create_test_service(
                    &format!("service-{}", i),
                    &format!("http://localhost:{}", 8080 + i),
                );
                reg.register(service).await
            });
        }
        
        while let Some(result) = set.join_next().await {
            assert!(result.unwrap().is_ok());
        }
        
        assert_eq!(registry.count().await, 10);
    }

    #[tokio::test]
    async fn test_registry_update_service() {
        let registry = UniversalServiceRegistry::new();
        let mut service = create_test_service("test-service", "http://localhost:8080");
        
        registry.register(service.clone()).await.unwrap();
        
        // Update service endpoint
        service.endpoint = "http://localhost:9090".to_string();
        registry.register(service).await.unwrap();
        
        let found = registry.find("test-service").await.unwrap().unwrap();
        assert_eq!(found.endpoint, "http://localhost:9090");
    }

    // Helper function
    fn create_test_service(name: &str, endpoint: &str) -> ServiceInfo {
        ServiceInfo {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            primal_type: songbird_types::PrimalType::new("test"),
            capabilities: vec![],
            health: songbird_types::HealthStatus::Healthy,
            metadata: HashMap::new(),
        }
    }
}

