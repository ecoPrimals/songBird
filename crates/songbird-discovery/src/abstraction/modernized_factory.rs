// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Modernized Discovery Factory
//!
//! Replaces the hardcoded `ServiceDiscoveryFactory` with an agnostic, configuration-driven approach

use std::collections::HashMap;

use songbird_types::{SongbirdError, SongbirdResult};

use crate::abstraction::adapters::{
    ConsulProviderFactory, KubernetesProviderFactory, ProviderFactory, ProviderFactoryImpl,
    StaticProviderFactory,
};
use crate::abstraction::delegation::{DelegationStrategy, DiscoveryDelegator};
use crate::abstraction::providers::ProviderConfig;
use crate::abstraction::registry::ProviderRegistry;

/// Modernized discovery factory that eliminates hardcoding
pub struct ModernizedDiscoveryFactory {
    registry: ProviderRegistry,
}

impl ModernizedDiscoveryFactory {
    /// Create a new modernized factory with default adapters
    pub async fn new() -> SongbirdResult<Self> {
        let registry = ProviderRegistry::new();

        registry.register_factory(ProviderFactoryImpl::Static(StaticProviderFactory)).await?;
        registry.register_factory(ProviderFactoryImpl::Consul(ConsulProviderFactory)).await?;
        registry
            .register_factory(ProviderFactoryImpl::Kubernetes(KubernetesProviderFactory))
            .await?;

        Ok(Self {
            registry,
        })
    }

    /// Create with custom registry (for testing or advanced use cases)
    #[must_use]
    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self {
            registry,
        }
    }

    /// Create providers from configuration (replaces hardcoded factory)
    pub async fn create_from_config(
        &self,
        configs: Vec<ProviderConfig>,
    ) -> SongbirdResult<DiscoveryDelegator> {
        for config in configs {
            let provider_type = config
                .parameters
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("static")
                .to_string();

            self.registry.create_provider(&provider_type, config).await?;
        }

        let delegator = DiscoveryDelegator::new(self.registry.clone())
            .with_strategy(DelegationStrategy::BestMatch);

        Ok(delegator)
    }

    /// Create from environment variables (modernized version of old `create_from_env`)
    pub async fn create_from_environment(&self) -> SongbirdResult<DiscoveryDelegator> {
        let mut configs = Vec::new();

        if songbird_process_env::var("SONGBIRD_DISCOVERY_STATIC").is_ok() {
            configs.push(self.create_static_config_from_env());
        }

        if let Ok(consul_url) = songbird_process_env::var("CONSUL_URL")
            .or_else(|_| songbird_process_env::var("CONSUL_HTTP_ADDR"))
        {
            configs.push(self.create_consul_config_from_env(consul_url));
        }

        if songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            let namespace = songbird_process_env::var("KUBERNETES_NAMESPACE")
                .unwrap_or_else(|_| String::from("default"));
            configs.push(self.create_kubernetes_config_from_env(namespace));
        }

        if configs.is_empty() {
            configs.push(self.create_static_config_from_env());
        }

        self.create_from_config(configs).await
    }

    /// Create from JSON/YAML configuration file
    pub async fn create_from_file(&self, config_path: &str) -> SongbirdResult<DiscoveryDelegator> {
        let config_content = std::fs::read_to_string(config_path).map_err(|e| {
            SongbirdError::configuration(format!("Failed to read config file {config_path}: {e}"))
        })?;

        let configs: Vec<ProviderConfig> = if std::path::Path::new(config_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
        {
            serde_yaml::from_str(&config_content).map_err(|e| {
                SongbirdError::configuration(format!("Failed to parse YAML config: {e}"))
            })?
        } else {
            serde_json::from_str(&config_content).map_err(|e| {
                SongbirdError::configuration(format!("Failed to parse JSON config: {e}"))
            })?
        };

        self.create_from_config(configs).await
    }

    /// Create multiple providers with different strategies
    pub async fn create_with_strategy(
        &self,
        configs: Vec<ProviderConfig>,
        strategy: DelegationStrategy,
    ) -> SongbirdResult<DiscoveryDelegator> {
        let delegator = self.create_from_config(configs).await?;
        Ok(delegator.with_strategy(strategy))
    }

    /// Get available provider types
    pub async fn available_provider_types(&self) -> Vec<String> {
        vec![String::from("static"), String::from("consul"), String::from("kubernetes")]
    }

    /// Validate configuration before creating providers
    pub async fn validate_configs(&self, configs: &[ProviderConfig]) -> SongbirdResult<()> {
        for config in configs {
            let provider_type =
                config.parameters.get("type").and_then(|v| v.as_str()).unwrap_or("static");

            match provider_type {
                "static" => StaticProviderFactory.validate_config(config)?,
                "consul" => ConsulProviderFactory.validate_config(config)?,
                "kubernetes" => KubernetesProviderFactory.validate_config(config)?,
                _ => {
                    return Err(SongbirdError::configuration(format!(
                        "Unknown provider type: {provider_type}"
                    )));
                }
            }
        }
        Ok(())
    }

    fn create_static_config_from_env(&self) -> ProviderConfig {
        let mut parameters = HashMap::new();

        if let Ok(services_json) = songbird_process_env::var("SONGBIRD_STATIC_SERVICES") {
            if let Ok(services) = serde_json::from_str::<serde_json::Value>(&services_json) {
                parameters.insert(String::from("services"), services);
            }
        }

        parameters.insert(String::from("type"), serde_json::Value::String(String::from("static")));

        ProviderConfig {
            id: String::from("env-static"),
            name: String::from("Environment Static Provider"),
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(1000),
            retry_config: None,
        }
    }

    fn create_consul_config_from_env(&self, consul_url: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters.insert(String::from("type"), serde_json::Value::String(String::from("consul")));
        parameters.insert(String::from("url"), serde_json::Value::String(consul_url));

        if let Ok(datacenter) = songbird_process_env::var("CONSUL_DATACENTER") {
            parameters.insert(String::from("datacenter"), serde_json::Value::String(datacenter));
        }

        let mut environment = HashMap::new();
        if let Ok(token) = songbird_process_env::var("CONSUL_TOKEN") {
            environment.insert(String::from("CONSUL_TOKEN"), token);
        }

        ProviderConfig {
            id: String::from("env-consul"),
            name: String::from("Environment Consul Provider"),
            parameters,
            environment,
            timeout_ms: Some(10_000),
            retry_config: None,
        }
    }

    fn create_kubernetes_config_from_env(&self, namespace: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters
            .insert(String::from("type"), serde_json::Value::String(String::from("kubernetes")));
        parameters.insert(String::from("namespace"), serde_json::Value::String(namespace));

        let mut environment = HashMap::new();
        if let Ok(kubeconfig) = songbird_process_env::var("KUBECONFIG") {
            environment.insert(String::from("KUBECONFIG"), kubeconfig);
        }

        ProviderConfig {
            id: String::from("env-kubernetes"),
            name: String::from("Environment Kubernetes Provider"),
            parameters,
            environment,
            timeout_ms: Some(30_000),
            retry_config: None,
        }
    }

    /// Get the underlying registry (for advanced use cases)
    #[must_use]
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }
}

impl Default for ModernizedDiscoveryFactory {
    fn default() -> Self {
        let registry = ProviderRegistry::new();
        Self {
            registry,
        }
    }
}

/// Configuration builder for easier setup
pub struct DiscoveryConfigBuilder {
    configs: Vec<ProviderConfig>,
}

impl DiscoveryConfigBuilder {
    /// New empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Add a static provider
    #[must_use]
    pub fn add_static(mut self, id: String, services: Vec<serde_json::Value>) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert(String::from("type"), serde_json::Value::String(String::from("static")));
        parameters.insert(String::from("services"), serde_json::Value::Array(services));

        self.configs.push(ProviderConfig {
            id,
            name: String::from("Static Provider"),
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(1000),
            retry_config: None,
        });

        self
    }

    /// Add a Consul provider
    #[must_use]
    pub fn add_consul(mut self, id: String, url: String) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert(String::from("type"), serde_json::Value::String(String::from("consul")));
        parameters.insert(String::from("url"), serde_json::Value::String(url));

        self.configs.push(ProviderConfig {
            id,
            name: String::from("Consul Provider"),
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(10_000),
            retry_config: None,
        });

        self
    }

    /// Add a Kubernetes provider
    #[must_use]
    pub fn add_kubernetes(mut self, id: String, namespace: String) -> Self {
        let mut parameters = HashMap::new();
        parameters
            .insert(String::from("type"), serde_json::Value::String(String::from("kubernetes")));
        parameters.insert(String::from("namespace"), serde_json::Value::String(namespace));

        self.configs.push(ProviderConfig {
            id,
            name: String::from("Kubernetes Provider"),
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(30_000),
            retry_config: None,
        });

        self
    }

    /// Build the configuration list
    #[must_use]
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
        let factory = ModernizedDiscoveryFactory::new().await.expect("create modernized factory");
        let types = factory.available_provider_types().await;

        assert!(types.contains(&String::from("static")));
        assert!(types.contains(&String::from("consul")));
        assert!(types.contains(&String::from("kubernetes")));
    }

    #[tokio::test]
    async fn test_config_builder() {
        use songbird_config::canonical::constants;

        let test_consul_url = format!(
            "http://{}:8500",
            songbird_process_env::var("TEST_CONSUL_HOST")
                .unwrap_or_else(|_| constants::network::DEFAULT_HOST.to_string())
        );

        let configs = DiscoveryConfigBuilder::new()
            .add_static(String::from("static-1"), vec![])
            .add_consul(String::from("consul-1"), test_consul_url)
            .build();

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].id, "static-1");
        assert_eq!(configs[1].id, "consul-1");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create modernized factory");

        let configs =
            DiscoveryConfigBuilder::new().add_static(String::from("test"), vec![]).build();

        assert!(factory.validate_configs(&configs).await.is_ok());
    }

    #[tokio::test]
    async fn create_from_config_registers_static_provider() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let configs =
            DiscoveryConfigBuilder::new().add_static("static-test".into(), vec![]).build();

        let delegator = factory.create_from_config(configs).await.expect("create delegator");
        assert_eq!(delegator.strategy(), &DelegationStrategy::BestMatch);

        let providers = factory.registry().list_providers().await;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "static-test");
    }

    #[tokio::test]
    async fn create_with_strategy_applies_delegation_strategy() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let configs = DiscoveryConfigBuilder::new().add_static("s1".into(), vec![]).build();

        let delegator = factory
            .create_with_strategy(configs, DelegationStrategy::FirstAvailable)
            .await
            .expect("create with strategy");
        assert_eq!(delegator.strategy(), &DelegationStrategy::FirstAvailable);
    }

    #[tokio::test]
    async fn validate_configs_rejects_unknown_provider_type() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let mut parameters = HashMap::new();
        parameters.insert("type".into(), serde_json::Value::String("unknown-backend".into()));
        let bad = ProviderConfig {
            id: "bad".into(),
            name: "Bad".into(),
            parameters,
            environment: HashMap::new(),
            timeout_ms: None,
            retry_config: None,
        };

        let err = factory.validate_configs(&[bad]).await.unwrap_err();
        assert!(err.to_string().contains("Unknown provider type"));
    }

    #[tokio::test]
    async fn validate_configs_rejects_consul_without_url() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let mut parameters = HashMap::new();
        parameters.insert("type".into(), serde_json::Value::String("consul".into()));
        let bad = ProviderConfig {
            id: "consul-bad".into(),
            name: "Consul".into(),
            parameters,
            environment: HashMap::new(),
            timeout_ms: None,
            retry_config: None,
        };

        assert!(factory.validate_configs(&[bad]).await.is_err());
    }

    #[tokio::test]
    async fn validate_configs_accepts_kubernetes_provider() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let configs =
            DiscoveryConfigBuilder::new().add_kubernetes("k8s-1".into(), "staging".into()).build();

        assert!(factory.validate_configs(&configs).await.is_ok());
    }

    #[tokio::test]
    async fn default_factory_has_empty_registry() {
        let factory = ModernizedDiscoveryFactory::default();
        assert!(factory.registry().list_providers().await.is_empty());
    }

    #[tokio::test]
    async fn with_registry_uses_provided_registry() {
        let registry = ProviderRegistry::new();
        let factory = ModernizedDiscoveryFactory::with_registry(registry);
        assert!(factory.registry().list_providers().await.is_empty());
    }

    #[test]
    fn config_builder_kubernetes_produces_valid_config() {
        let configs = DiscoveryConfigBuilder::new()
            .add_kubernetes("k8s-env".into(), "production".into())
            .build();

        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].parameters.get("type").and_then(|v| v.as_str()), Some("kubernetes"));
        assert_eq!(
            configs[0].parameters.get("namespace").and_then(|v| v.as_str()),
            Some("production")
        );
    }

    #[tokio::test]
    async fn create_from_config_multiple_backends() {
        use songbird_config::canonical::constants;

        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let consul_url = format!("http://{}:8500", constants::network::DEFAULT_HOST);
        let configs = DiscoveryConfigBuilder::new()
            .add_static("s".into(), vec![])
            .add_consul("c".into(), consul_url)
            .add_kubernetes("k".into(), "default".into())
            .build();

        factory.create_from_config(configs).await.expect("create multi-backend delegator");
        assert_eq!(factory.registry().list_providers().await.len(), 3);
    }

    #[tokio::test]
    async fn create_from_file_json_config() {
        let factory = ModernizedDiscoveryFactory::new().await.expect("create factory");
        let dir =
            std::env::temp_dir().join(format!("songbird-factory-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("providers.json");
        let json = r#"[{"id":"file-static","name":"From File","parameters":{"type":"static","services":[]},"environment":{},"timeout_ms":1000,"retry_config":null}]"#;
        std::fs::write(&path, json).expect("write config");

        let delegator = factory.create_from_file(path.to_str().unwrap()).await.expect("from file");
        assert_eq!(delegator.strategy(), &DelegationStrategy::BestMatch);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
