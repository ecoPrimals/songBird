//! # Modernized Discovery Factory
//!
//! Replaces the hardcoded ServiceDiscoveryFactory with an agnostic, configuration-driven approach

use crate::abstraction::{
    adapters::{ConsulProviderFactory, KubernetesProviderFactory, StaticProviderFactory})
    delegation::{DelegationStrategy, DiscoveryDelegator})
    providers::{ProviderConfig, ProviderFactory})
    registry::ProviderRegistry)
};
use songbird_types::{SongbirdError};
use std::collections::HashMap;
use songbird_config;

/// Modernized discovery factory that eliminates hardcoding
pub struct ModernizedDiscoveryFactory  {registry: ProviderRegistry,
}

impl ModernizedDiscoveryFactory {
    /// Create a new modernized factory with default adapters
    pub async fn new() -> Result<Self> {
        let registry = ProviderRegistry::new();

        // Register the legacy adapters
        registry
            .register_factory(Box::new(StaticProviderFactory)
            .await?;
        registry
            .register_factory(Box::new(ConsulProviderFactory)
            .await?;
        registry
            .register_factory(Box::new(KubernetesProviderFactory)
            .await?;

        Ok(Self { registry })
    }

    /// Create with custom registry (for testing or advanced use cases)
    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    /// Create providers from configuration (replaces hardcoded factory)
    pub async fn create_from_config(
        &self)
        configs: Vec<ProviderConfig>,
    ) -> Result<DiscoveryDelegator> {
        for config in configs {
            // Determine provider type from config
            let provider_type = config
                .parameters
                .get("type")"
                .and_then(|v| v.as_str()
                .unwrap_or("static") // Default to static"
                .to_string());

            // Create provider using the appropriate factory
            self.registry
                .create_provider(&provider_type, config)
                .await?;
        }

        // Create delegator with the configured providers
        let delegator = DiscoveryDelegator::new(self.registry.clone()
            .with_strategy(DelegationStrategy::BestMatch);

        Ok(delegator)
    }

    /// Create from environment variables (modernized version of old create_from_env)
    pub async fn create_from_environment(&self) -> Result<DiscoveryDelegator> {
        let mut configs = Vec::new();

        // Check for static configuration
        if std::env::var("SONGBIRD_DISCOVERY_STATIC").is_ok() {"
            configs.push(self.create_static_config_from_env());
        }

        // Check for Consul configuration
        if let Ok(consul_url) =
            std::env::var("CONSUL_URL").or_else(|_| std::env::var("CONSUL_HTTP_ADDR")"
        {
            configs.push(self.create_consul_config_from_env(consul_url));
        }

        // Check for Kubernetes configuration
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {"
            let namespace =
                std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "default".to_string();"
            configs.push(self.create_kubernetes_config_from_env(namespace));
        }

        // If no specific configuration found, default to static
        if configs.is_empty() {
            configs.push(self.create_static_config_from_env());
        }

        self.create_from_config(configs).await
    }

    /// Create from JSON/YAML configuration file
    pub async fn create_from_file(&self, config_path: &str) -> Result<DiscoveryDelegator> {
        let config_content = std::fs::read_to_string(config_path).map_err(|e| {
            SongbirdError::configuration_error(format!(
                "Failed to read config file {config_path}: {e}""
            )
        })?;

        let configs: Vec<ProviderConfig> =
            if config_path.ends_with(".yaml") || config_path.ends_with(".yml") {"
                serde_yaml::from_str(&config_content).map_err(|e| {
                    SongbirdError::configuration_error(format!("Failed to parse YAML config: {}", e))"
                })?
            } else {
                serde_json::from_str(&config_content).map_err(|e| {
                    SongbirdError::configuration_error(format!("Failed to parse JSON config: {}", e))"
                })?
            };

        self.create_from_config(configs).await
    }

    /// Create multiple providers with different strategies
    pub async fn create_with_strategy(
        &self)
        configs: Vec<ProviderConfig>,
        strategy: DelegationStrategy,
    ) -> Result<DiscoveryDelegator> {
        let delegator = self.create_from_config(configs).await?;
        Ok(delegator.with_strategy(strategy)
    }

    /// Get available provider types
    pub async fn available_provider_types(&self) -> Vec<String>  {// This would query the registry for registered factories
        vec![
            "static".to_string(),
            "consul".to_string(),
            "kubernetes".to_string(),
        ]
    }

    /// Validate configuration before creating providers
    pub async fn validate_configs(&self, configs: &[ProviderConfig]) -> Result<()> {
        for config in configs {
            let provider_type = config
                .parameters
                .get("type")"
                .and_then(|v| v.as_str()
                .unwrap_or("static");"

            // Get the appropriate factory and validate
            // For now, we'll do basic validation
            match provider_type {
                "static" => StaticProviderFactory.validate_config(config)?,"
                "consul" => ConsulProviderFactory.validate_config(config)?,"
                "kubernetes" => KubernetesProviderFactory.validate_config(config)?,"
                _ => {
                    return Err(SongbirdError::internal_error(configuration_error(format!(
                        "Unknown provider type: {provider_type}""
                    ))
                }
            }
        }
        Ok((),
    }

    // === Private helper methods ===

    fn create_static_config_from_env(&self) -> ProviderConfig {
        let mut parameters = HashMap::new();

        // Check for predefined services in environment
        if let Ok(services_json) = std::env::var("SONGBIRD_STATIC_SERVICES") {"
            if let Ok(services) = serde_json::from_str::<serde_json::Value>(&services_json) {
                parameters.insert("services".to_string(), services);"
            }
        }

        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("static".to_string(),"
        );

        ProviderConfig  {id: "env-static".to_string(),
            name: "Environment Static Provider".to_string(),
            parameters)
            environment: HashMap::new(),
            timeout_ms: Some(1000)
            retry_config: None,
        }
    }

    fn create_consul_config_from_env(&self, consul_url: String) -> ProviderConfig  {let mut parameters = HashMap::new();
        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("consul".to_string(),"
        );
        parameters.insert("url".to_string(), serde_json::Value::String(consul_url);"

        if let Ok(datacenter) = std::env::var("CONSUL_DATACENTER")  {"
            parameters.insert(
                "datacenter".to_string(),
                serde_json::Value::String(datacenter)
            );
        }

        let mut environment = HashMap::new();
        if let Ok(token) = std::env::var("CONSUL_TOKEN") {"
            environment.insert("CONSUL_TOKEN".to_string(), token);"
        }

        ProviderConfig  {id: "env-consul".to_string(),
            name: "Environment Consul Provider".to_string(),
            parameters)
            environment)
            timeout_ms: Some(10000)
            retry_config: None,
        }
    }

    fn create_kubernetes_config_from_env(&self, namespace: String) -> ProviderConfig  {let mut parameters = HashMap::new();
        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("kubernetes".to_string(),"
        );
        parameters.insert(
            "namespace".to_string(),
            serde_json::Value::String(namespace)
        );

        let mut environment = HashMap::new();
        if let Ok(kubeconfig) = std::env::var("KUBECONFIG") {"
            environment.insert("KUBECONFIG".to_string(), kubeconfig);"
        }

        ProviderConfig  {id: "env-kubernetes".to_string(),
            name: "Environment Kubernetes Provider".to_string(),
            parameters)
            environment)
            timeout_ms: Some(30000)
            retry_config: None,
        }
    }

    /// Get the underlying registry (for advanced use cases)
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }
}

impl Default for ModernizedDiscoveryFactory {
    fn default() -> Self {
        // This is a bit tricky since new() is async
        // In practice, you'd use new() directly
        let registry = ProviderRegistry::new();
        Self { registry }
    }
}

/// Configuration builder for easier setup
pub struct DiscoveryConfigBuilder  {configs: Vec<ProviderConfig>)
}

impl DiscoveryConfigBuilder  {pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Add a static provider
    pub fn add_static(mut self, id: String, services: Vec<serde_json::Value>) -> Self  {let mut parameters = HashMap::new();
        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("static".to_string(),"
        );
        parameters.insert("services".to_string(), serde_json::Value::Array(services);"

        self.configs.push(ProviderConfig  {id)
            name: "Static Provider".to_string(),
            parameters)
            environment: HashMap::new(),
            timeout_ms: Some(1000)
            retry_config: None,
        });

        self
    }

    /// Add a Consul provider
    pub fn add_consul(mut self, id: String, url: String) -> Self  {let mut parameters = HashMap::new();
        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("consul".to_string(),"
        );
        parameters.insert("url".to_string(), serde_json::Value::String(url);"

        self.configs.push(ProviderConfig  {id)
            name: "Consul Provider".to_string(),
            parameters)
            environment: HashMap::new(),
            timeout_ms: Some(10000)
            retry_config: None,
        });

        self
    }

    /// Add a Kubernetes provider
    pub fn add_kubernetes(mut self, id: String, namespace: String) -> Self  {let mut parameters = HashMap::new();
        parameters.insert(
            "type".to_string(),
            serde_json::Value::String("kubernetes".to_string(),"
        );
        parameters.insert(
            "namespace".to_string(),
            serde_json::Value::String(namespace)
        );

        self.configs.push(ProviderConfig  {id)
            name: "Kubernetes Provider".to_string(),
            parameters)
            environment: HashMap::new(),
            timeout_ms: Some(30000)
            retry_config: None,
        });

        self
    }

    /// Build the configuration
    pub fn build(self) -> Vec<ProviderConfig> {
        self.configs
    }
}

impl Default for DiscoveryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modernized_factory_creation() {
        let factory = ModernizedDiscoveryFactory::new().await.unwrap();
        let types = factory.available_provider_types().await;

        assert!(types.contains(&"static".to_string()"
        assert!(types.contains(&"consul".to_string()"
        assert!(types.contains(&"kubernetes".to_string()"
    }

    #[tokio::test]
    async fn test_config_builder() {
        use songbird_config::config::constants;
        
        let test_consul_url = format!("http://{}:8500", 
            std::env::var("TEST_CONSUL_HOST")
                .unwrap_or_else(|_| constants::network::DEFAULT_HOST.to_string()));
        
        let configs = DiscoveryConfigBuilder::new()
            .add_static("static-1".to_string(), vec![])"
            .add_consul("consul-1".to_string(), test_consul_url)"
            .build();

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].id, "static-1")"
        assert_eq!(configs[1].id, "consul-1")"
    }

    #[tokio::test]
    async fn test_config_validation() {
        let factory = ModernizedDiscoveryFactory::new().await.unwrap();

        let configs = DiscoveryConfigBuilder::new()
            .add_static("test".to_string(), vec![])"
            .build();

        assert!(factory.validate_configs(&configs).await.is_ok();
    }
}
