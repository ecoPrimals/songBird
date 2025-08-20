//! Consul service discovery backend

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::traits::discovery::{ServiceDiscovery, ServiceQuery, ServiceEvent, ServiceHealthStatus, HealthStatus, SortBy};
use crate::traits::service::ServiceInfo;
use songbird_errors::{DiscoveryError, Result, SongbirdError};

/// Consul service discovery backend
pub struct ConsulServiceDiscovery {
    consul_url: String,
    client: reqwest::Client,
}

impl ConsulServiceDiscovery {
    /// Create new Consul service discovery
    #[must_use]
    pub fn new(consul_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client for Consul");

        Self { consul_url, client }
    }

    /// Build service registration payload for Consul
    fn build_service_payload(&self, service: &ServiceInfo) -> Value {
        let mut check = json!({
            "HTTP": format!("http://{}:{}/health", service.host, service.port),
            "Interval": "10s",
            "Timeout": "5s"
        });

        // Use custom health check URL if provided
        if let Some(health_url) = &service.health_check_endpoint {
            check["HTTP"] = json!(health_url);
        }

        json!({
            "ID": service.service_id,
            "Name": service.name,
            "Tags": service.tags,
            "Address": service.host,
            "Port": service.port,
            "Meta": service.metadata,
            "Check": check
        })
    }

    /// Parse Consul service response into ServiceInfo
    fn parse_consul_service(&self, consul_service: &Value) -> Option<ServiceInfo> {
        let id = consul_service["ID"].as_str()?.to_string();
        let name = consul_service["Service"].as_str()?.to_string();

        let address = consul_service["Address"].as_str()?;
        let port = consul_service["Port"].as_u64()?;
        let socket_addr = format!("{address}:{port}").parse::<SocketAddr>().ok()?;

        let tags: Vec<String> = consul_service["Tags"]
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let metadata: HashMap<String, String> = consul_service["Meta"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        use chrono::Utc;
        use crate::traits::service::ServiceStatus;
        
        Some(ServiceInfo {
            service_id: id,
            name,
            version: "1.0.0".to_string(), // Default version
            service_type: "consul-service".to_string(),
            description: None,
            endpoints: vec![], // TODO: Extract from Consul service definition
            health_check_endpoint: None,
            metadata: metadata.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect(),
            tags,
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("consul-{}", uuid::Uuid::new_v4()),
            host: address.to_string(),
            port: port.try_into().unwrap_or(8080),
        })
    }
}

#[async_trait]
impl ServiceDiscovery for ConsulServiceDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<()> {
        tracing::info!("Registering service {} with Consul", service.name);

        let url = format!("{}/v1/agent/service/register", self.consul_url);
        let payload = self.build_service_payload(&service);

        let response = self
            .client
            .put(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to register service with Consul: {e}"),
                    service: Some(service.name.clone()),
                    timeout: None,
                    suggestion: Some("Check Consul connectivity and configuration".to_string()),
                }))
            })?;

        if response.status().is_success() {
            tracing::info!(
                "Successfully registered service {} with Consul",
                service.name
            );
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Consul registration failed: {error_text}"),
                service: Some(service.name),
                timeout: None,
                suggestion: Some("Check service configuration and Consul ACLs".to_string()),
            })))
        }
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!("Deregistering service {} from Consul", service_id);

        let url = format!(
            "{}/v1/agent/service/deregister/{}",
            self.consul_url, service_id
        );

        let response = self.client.put(&url).send().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to deregister service from Consul: {e}"),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check Consul connectivity".to_string()),
            }))
        })?;

        if response.status().is_success() {
            tracing::info!(
                "Successfully deregistered service {} from Consul",
                service_id
            );
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Consul deregistration failed: {error_text}"),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check if service exists and Consul ACLs".to_string()),
            })))
        }
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let url = match &query.name {
            Some(name) => format!("{}/v1/health/service/{}", self.consul_url, name),
            None => format!("{}/v1/agent/services", self.consul_url),
        };

        tracing::debug!("Discovering services from Consul: {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to discover services from Consul: {e}"),
                service: query.name.clone(),
                timeout: None,
                suggestion: Some("Check Consul connectivity".to_string()),
            }))
        })?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Consul discovery failed: {error_text}"),
                service: query.name.clone(),
                timeout: None,
                suggestion: Some("Check Consul query and ACLs".to_string()),
            })));
        }

        let consul_response: Value = response.json().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to parse Consul response: {e}"),
                service: query.name.clone(),
                timeout: None,
                suggestion: Some("Check Consul API version compatibility".to_string()),
            }))
        })?;

        let services: Vec<ServiceInfo> = if query.name.is_some() {
            // Health endpoint returns array of health checks
            consul_response
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|health_check| {
                    // Only include services that are passing health checks
                    let checks = health_check["Checks"].as_array()?;
                    let all_passing = checks
                        .iter()
                        .all(|check| check["Status"].as_str() == Some("passing"));

                    if all_passing {
                        self.parse_consul_service(&health_check["Service"])
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            // Services endpoint returns object of services
            consul_response
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .values()
                .filter_map(|service| self.parse_consul_service(service))
                .collect()
        };

        tracing::debug!("Discovered {} services from Consul", services.len());
        Ok(services)
    }

    /// Check health status of a specific service (internal method)

    async fn watch(
        &self,
        query: ServiceQuery,
    ) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = ServiceEvent> + Send>>> {
        // TODO: Implement Consul watch functionality
        use futures_util::stream;
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        tracing::info!("Updating health for service {} to {:?}", service_id, health);
        // TODO: Implement health update via Consul API
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        self.discover(ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        let query = ServiceQuery::new().with_service_id(service_id);
        let services = self.discover(query).await?;
        Ok(!services.is_empty())
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        tracing::info!("Updating metadata for service {}: {:?}", service_id, metadata);
        // TODO: Implement metadata update via Consul API
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ConsulServiceDiscovery {
    // Check health status of a specific service (internal method)
}
