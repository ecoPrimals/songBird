//! Service discovery factory for creating backend instances

use super::backends::{ConsulServiceDiscovery, KubernetesServiceDiscovery, StaticServiceDiscovery};
use super::core::{DiscoveryConfig, ServiceDiscovery};
use songbird_errors::{DiscoveryError, Result, SongbirdError};

/// Service discovery factory
pub struct ServiceDiscoveryFactory;

impl ServiceDiscoveryFactory {
    /// Create service discovery backend based on configuration
    pub fn create(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>> {
        match config.backend.as_str() {
            "static" => {
                tracing::info!("Creating static service discovery backend");
                Ok(Box::new(StaticServiceDiscovery::new()))
            }
            "consul" => {
                let consul_url = config.consul_url.as_ref().ok_or_else(|| {
                    SongbirdError::Discovery(Box::new(DiscoveryError {
                        message: "Consul URL not provided for consul backend".to_string(),
                        service: None,
                        timeout: None,
                        suggestion: Some("Set consul_url in discovery configuration".to_string()),
                    }))
                })?;

                tracing::info!("Creating Consul service discovery backend: {}", consul_url);
                Ok(Box::new(ConsulServiceDiscovery::new(consul_url.clone())))
            }
            "kubernetes" => {
                let namespace = config.kubernetes_namespace.as_ref().ok_or_else(|| {
                    SongbirdError::Discovery(Box::new(DiscoveryError {
                        message: "Kubernetes namespace not provided for kubernetes backend"
                            .to_string(),
                        service: None,
                        timeout: None,
                        suggestion: Some(
                            "Set kubernetes_namespace in discovery configuration".to_string(),
                        ),
                    }))
                })?;

                tracing::info!(
                    "Creating Kubernetes service discovery backend: {}",
                    namespace
                );
                Ok(Box::new(KubernetesServiceDiscovery::new(
                    namespace.clone(),
                )?))
            }
            _ => Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Unsupported discovery backend: {}", config.backend),
                service: None,
                timeout: None,
                suggestion: Some("Use 'static', 'consul', or 'kubernetes' backend".to_string()),
            }))),
        }
    }

    /// Create static backend with predefined services
    pub fn create_static_with_services(
        services: Vec<super::core::ServiceInstance>,
    ) -> Result<Box<dyn ServiceDiscovery>> {
        tracing::info!(
            "Creating static service discovery with {} predefined services",
            services.len()
        );

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Convert ServiceInstance to ServiceInfo
                let service_infos: Vec<crate::traits::service::ServiceInfo> = services
                    .into_iter()
                    .map(|service| {
                        use chrono::Utc;
                        use crate::traits::service::ServiceStatus;
                        
                        crate::traits::service::ServiceInfo {
                            service_id: service.id,
                            name: service.name,
                            version: "1.0.0".to_string(),
                            service_type: "static-service".to_string(),
                            description: None,
                            endpoints: vec![],
                            health_check_endpoint: service.health_check_url,
                            metadata: service.metadata.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect(),
                            tags: service.tags,
                            dependencies: vec![],
                            status: ServiceStatus::Running,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            instance_id: format!("static-{}", uuid::Uuid::new_v4()),
                            host: service.address.ip().to_string(),
                            port: service.address.port(),
                        }
                    })
                    .collect();
                
                let discovery = StaticServiceDiscovery::with_services(service_infos).await;
                Ok(Box::new(discovery) as Box<dyn ServiceDiscovery>)
            })
        })
    }

    /// Create backend from environment variables
    pub fn create_from_env() -> Result<Box<dyn ServiceDiscovery>> {
        let backend =
            std::env::var("SONGBIRD_DISCOVERY_BACKEND").unwrap_or_else(|_| "static".to_string());

        let config = match backend.as_str() {
            "consul" => {
                let consul_url = std::env::var("CONSUL_URL")
                    .or_else(|_| std::env::var("CONSUL_HTTP_ADDR"))
                    .unwrap_or_else(|_| "http://localhost:8500".to_string());

                DiscoveryConfig::consul_config(consul_url)
            }
            "kubernetes" => {
                let namespace =
                    std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "default".to_string());

                DiscoveryConfig::kubernetes_config(namespace)
            }
            _ => DiscoveryConfig::static_config(),
        };

        Self::create(&config)
    }

    /// Get available backend names
    pub fn available_backends() -> Vec<&'static str> {
        vec!["static", "consul", "kubernetes"]
    }

    /// Validate configuration
    pub fn validate_config(config: &DiscoveryConfig) -> Result<()> {
        match config.backend.as_str() {
            "static" => {
                // Static backend doesn't need additional validation
                Ok(())
            }
            "consul" => {
                if config.consul_url.is_none() {
                    return Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                        message: "Consul URL is required for consul backend".to_string(),
                        service: None,
                        timeout: None,
                        suggestion: Some("Set consul_url in configuration".to_string()),
                    })));
                }
                Ok(())
            }
            "kubernetes" => {
                if config.kubernetes_namespace.is_none() {
                    return Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                        message: "Kubernetes namespace is required for kubernetes backend"
                            .to_string(),
                        service: None,
                        timeout: None,
                        suggestion: Some("Set kubernetes_namespace in configuration".to_string()),
                    })));
                }
                Ok(())
            }
            _ => Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Unsupported backend: {}", config.backend),
                service: None,
                timeout: None,
                suggestion: Some(format!(
                    "Use one of: {}",
                    Self::available_backends().join(", ")
                )),
            }))),
        }
    }
}
