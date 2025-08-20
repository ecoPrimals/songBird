//! # Agnostic Discovery Architecture Demo
//!
//! This example demonstrates how the new abstraction layer eliminates hard-coded
//! dependencies on specific external services like Consul or Kubernetes.

use std::collections::HashMap;
use songbird_discovery::abstraction::{
    capabilities::{CapabilityMatcher, CapabilityQuery, DiscoveryCapability},
    providers::{DiscoveryProvider, ProviderConfig, ProviderMetadata, ProviderFactory},
    registry::ProviderRegistry,
    delegation::{DiscoveryDelegator, DelegationStrategy},
};
use songbird_discovery::traits::{ServiceInfo, ServiceQuery};
use songbird_errors::Result;

/// Example: Creating a Redis-based discovery provider without hard-coding
struct RedisDiscoveryFactory;

#[async_trait::async_trait]
impl ProviderFactory for RedisDiscoveryFactory {
    fn provider_type(&self) -> &str {
        "redis"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<Box<dyn DiscoveryProvider>> {
        // Create a Redis provider from flexible configuration
        let redis_url = config.parameters.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("redis://localhost:6379");

        let provider = RedisDiscoveryProvider::new(redis_url.to_string());
        Ok(Box::new(provider))
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<()> {
        if config.parameters.get("url").is_none() {
            return Err(songbird_errors::SongbirdError::configuration_error(
                "Redis URL is required in parameters.url"
            ));
        }
        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters.insert("url".to_string(), serde_json::Value::String("redis://localhost:6379".to_string()));
        parameters.insert("db".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));

        ProviderConfig {
            id,
            name,
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(5000),
            retry_config: None,
        }
    }
}

/// Example Redis discovery provider (mock implementation)
struct RedisDiscoveryProvider {
    metadata: ProviderMetadata,
    _redis_url: String,
}

impl RedisDiscoveryProvider {
    fn new(redis_url: String) -> Self {
        let metadata = ProviderMetadata {
            id: "redis-provider".to_string(),
            name: "Redis Discovery Provider".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration,
                DiscoveryCapability::ServiceUnregistration,
                DiscoveryCapability::ServiceDiscovery,
                DiscoveryCapability::ServiceListing,
                DiscoveryCapability::ServiceExistence,
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "redis".to_string());
                meta.insert("protocol".to_string(), "redis".to_string());
                meta
            },
            healthy: true,
            load_score: 0.3,
        };

        Self {
            metadata,
            _redis_url: redis_url,
        }
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for RedisDiscoveryProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        println!("🔧 Initializing Redis discovery provider");
        // Here you would connect to Redis, set up key spaces, etc.
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        println!("🔌 Shutting down Redis discovery provider");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Here you would ping Redis
        Ok(true)
    }

    async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        println!("📝 Registering service {} in Redis", service.service_id);
        // Here you would store service info in Redis
        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        println!("🔍 Discovering services in Redis with query: {:?}", query.name);
        // Here you would query Redis for matching services
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Example: ETCD provider factory (completely different protocol)
struct EtcdDiscoveryFactory;

#[async_trait::async_trait]
impl ProviderFactory for EtcdDiscoveryFactory {
    fn provider_type(&self) -> &str {
        "etcd"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<Box<dyn DiscoveryProvider>> {
        let etcd_endpoints = config.parameters.get("endpoints")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_else(|| vec!["http://localhost:2379".to_string()]);

        let provider = EtcdDiscoveryProvider::new(etcd_endpoints);
        Ok(Box::new(provider))
    }

    fn validate_config(&self, _config: &ProviderConfig) -> Result<()> {
        // ETCD validation logic
        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters.insert("endpoints".to_string(), 
            serde_json::Value::Array(vec![
                serde_json::Value::String("http://localhost:2379".to_string())
            ]));

        ProviderConfig {
            id,
            name,
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(10000),
            retry_config: None,
        }
    }
}

struct EtcdDiscoveryProvider {
    metadata: ProviderMetadata,
    _endpoints: Vec<String>,
}

impl EtcdDiscoveryProvider {
    fn new(endpoints: Vec<String>) -> Self {
        let metadata = ProviderMetadata {
            id: "etcd-provider".to_string(),
            name: "ETCD Discovery Provider".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration,
                DiscoveryCapability::ServiceUnregistration,
                DiscoveryCapability::ServiceDiscovery,
                DiscoveryCapability::ServiceWatching, // ETCD supports watching!
                DiscoveryCapability::ServiceListing,
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "etcd".to_string());
                meta.insert("protocol".to_string(), "grpc".to_string());
                meta
            },
            healthy: true,
            load_score: 0.2, // Lower load than Redis
        };

        Self {
            metadata,
            _endpoints: endpoints,
        }
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for EtcdDiscoveryProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        println!("🔧 Initializing ETCD discovery provider");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        println!("🔌 Shutting down ETCD discovery provider");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        println!("📝 Registering service {} in ETCD", service.service_id);
        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        println!("🔍 Discovering services in ETCD with query: {:?}", query.name);
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Agnostic Discovery Architecture Demo");
    println!("========================================");

    // 1. Create a provider registry
    let registry = ProviderRegistry::new();

    // 2. Register provider factories (no hard-coding!)
    println!("\n📦 Registering provider factories...");
    registry.register_factory(Box::new(RedisDiscoveryFactory)).await?;
    registry.register_factory(Box::new(EtcdDiscoveryFactory)).await?;

    // 3. Create providers from configuration (completely flexible!)
    println!("\n🏭 Creating providers from configuration...");
    
    let redis_config = ProviderConfig {
        id: "redis-main".to_string(),
        name: "Main Redis Provider".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("url".to_string(), serde_json::Value::String("redis://prod-redis:6379".to_string()));
            params
        },
        environment: HashMap::new(),
        timeout_ms: Some(5000),
        retry_config: None,
    };

    let etcd_config = ProviderConfig {
        id: "etcd-cluster".to_string(),
        name: "Production ETCD Cluster".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("endpoints".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("http://etcd-1:2379".to_string()),
                serde_json::Value::String("http://etcd-2:2379".to_string()),
                serde_json::Value::String("http://etcd-3:2379".to_string()),
            ]));
            params
        },
        environment: HashMap::new(),
        timeout_ms: Some(10000),
        retry_config: None,
    };

    registry.create_provider("redis", redis_config).await?;
    registry.create_provider("etcd", etcd_config).await?;

    // 4. Create a delegator with different strategies
    println!("\n🎯 Creating discovery delegator...");
    let delegator = DiscoveryDelegator::new(registry)
        .with_strategy(DelegationStrategy::LeastLoad);

    // 5. Use discovery without knowing about specific providers!
    println!("\n🔍 Using discovery services...");

    // Find providers that can register services
    let registration_query = CapabilityQuery::new(
        CapabilityMatcher::new().require(DiscoveryCapability::ServiceRegistration)
    );

    println!("Providers that can register services:");
    if let Ok(providers) = delegator.registry.find_providers(&registration_query).await {
        for provider_id in providers {
            if let Ok(metadata) = delegator.registry.get_provider_metadata(&provider_id).await {
                println!("  - {} ({}): load={}", metadata.name, metadata.id, metadata.load_score);
            }
        }
    }

    // Find providers that can watch for changes (only ETCD in our example)
    let watching_query = CapabilityQuery::new(
        CapabilityMatcher::new().require(DiscoveryCapability::ServiceWatching)
    );

    println!("\nProviders that can watch for changes:");
    if let Ok(providers) = delegator.registry.find_providers(&watching_query).await {
        for provider_id in providers {
            if let Ok(metadata) = delegator.registry.get_provider_metadata(&provider_id).await {
                println!("  - {} ({})", metadata.name, metadata.id);
            }
        }
    } else {
        println!("  - No providers support watching");
    }

    // 6. Demonstrate registry statistics
    println!("\n📊 Registry Statistics:");
    let stats = delegator.registry.get_statistics().await;
    println!("  - Total providers: {}", stats.total_providers);
    println!("  - Healthy providers: {}", stats.healthy_providers);
    println!("  - Total factories: {}", stats.total_factories);
    println!("  - Capabilities distribution:");
    for (capability, count) in stats.capabilities_count {
        println!("    - {:?}: {} providers", capability, count);
    }

    println!("\n✅ Demo completed successfully!");
    println!("\n🎉 Key Benefits Demonstrated:");
    println!("  1. Zero hard-coding - providers configured from data");
    println!("  2. Runtime registration - add new provider types dynamically");
    println!("  3. Capability-based routing - requests go to capable providers");
    println!("  4. Protocol agnostic - Redis, ETCD, or any other system");
    println!("  5. Load balancing - automatic selection based on load/capabilities");
    println!("  6. Extensible - add new capabilities without changing core code");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agnostic_provider_creation() {
        let registry = ProviderRegistry::new();
        
        // Register Redis factory
        let redis_factory = Box::new(RedisDiscoveryFactory);
        assert!(registry.register_factory(redis_factory).await.is_ok());

        // Create provider from config
        let config = RedisDiscoveryFactory.default_config(
            "test-redis".to_string(),
            "Test Redis".to_string()
        );
        
        assert!(registry.create_provider("redis", config).await.is_ok());

        // Verify provider was registered
        let providers = registry.list_providers().await;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "redis-provider");
    }

    #[tokio::test]
    async fn test_capability_based_selection() {
        let registry = ProviderRegistry::new();
        
        // Register both factories
        registry.register_factory(Box::new(RedisDiscoveryFactory)).await.unwrap();
        registry.register_factory(Box::new(EtcdDiscoveryFactory)).await.unwrap();

        // Create both providers
        let redis_config = RedisDiscoveryFactory.default_config("redis".to_string(), "Redis".to_string());
        let etcd_config = EtcdDiscoveryFactory.default_config("etcd".to_string(), "ETCD".to_string());
        
        registry.create_provider("redis", redis_config).await.unwrap();
        registry.create_provider("etcd", etcd_config).await.unwrap();

        // Test capability-based selection
        let watching_query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceWatching)
        );

        // Only ETCD should support watching
        let providers = registry.find_providers(&watching_query).await.unwrap();
        assert_eq!(providers.len(), 1);
        
        let metadata = registry.get_provider_metadata(&providers[0]).await.unwrap();
        assert_eq!(metadata.name, "ETCD Discovery Provider");
    }
} 