//! Consul service discovery backend

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;

use super::super::core::{ServiceDiscovery, ServiceInstance};
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
    fn build_service_payload(&self, service: &ServiceInstance) -> Value {
        let mut check = json!({
            "HTTP": format!("http://{}/health", service.address),
            "Interval": "10s",
            "Timeout": "5s"
        });

        // Use custom health check URL if provided
        if let Some(health_url) = &service.health_check_url {
            check["HTTP"] = json!(health_url);
        }

        json!({
            "ID": service.id,
            "Name": service.name,
            "Tags": service.tags,
            "Address": service.address.ip().to_string(),
            "Port": service.address.port(),
            "Meta": service.metadata,
            "Check": check
        })
    }

    /// Parse Consul service response into ServiceInstance
    fn parse_consul_service(&self, consul_service: &Value) -> Option<ServiceInstance> {
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

        Some(ServiceInstance {
            id,
            name,
            address: socket_addr,
            metadata,
            health_check_url: None, // Consul manages health checks
            tags,
        })
    }
}

#[async_trait]
impl ServiceDiscovery for ConsulServiceDiscovery {
    async fn register_service(&self, service: ServiceInstance) -> Result<()> {
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

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
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

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        let url = match service_name {
            Some(name) => format!("{}/v1/health/service/{}", self.consul_url, name),
            None => format!("{}/v1/agent/services", self.consul_url),
        };

        tracing::debug!("Discovering services from Consul: {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to discover services from Consul: {e}"),
                service: service_name.map(String::from),
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
                service: service_name.map(String::from),
                timeout: None,
                suggestion: Some("Check Consul query and ACLs".to_string()),
            })));
        }

        let consul_response: Value = response.json().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to parse Consul response: {e}"),
                service: service_name.map(String::from),
                timeout: None,
                suggestion: Some("Check Consul API version compatibility".to_string()),
            }))
        })?;

        let services: Vec<ServiceInstance> = if service_name.is_some() {
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

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        let url = format!("{}/v1/health/service/{}", self.consul_url, service_id);

        let response = self.client.get(&url).send().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to check service health in Consul: {e}"),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check Consul connectivity".to_string()),
            }))
        })?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let health_response: Value = response.json().await.map_err(|_| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: "Failed to parse Consul health response".to_string(),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check Consul API version".to_string()),
            }))
        })?;

        // Check if any service instances are healthy
        let is_healthy = health_response
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .any(|health_check| {
                let empty_vec = vec![];
                let checks = health_check["Checks"].as_array().unwrap_or(&empty_vec);
                checks
                    .iter()
                    .all(|check| check["Status"].as_str() == Some("passing"))
            });

        tracing::debug!("Health check for service {}: {}", service_id, is_healthy);
        Ok(is_healthy)
    }
}
