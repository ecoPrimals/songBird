//! Kubernetes service discovery for cloud-native deployments

use async_trait::async_trait;
use serde_json::{json, Value};
use std::net::SocketAddr;

use super::super::core::{ServiceDiscovery, ServiceInstance};
use songbird_errors::{DiscoveryError, Result, SongbirdError};

/// Kubernetes service discovery for cloud-native deployments
pub struct KubernetesServiceDiscovery {
    namespace: String,
    client: reqwest::Client,
    api_server: String,
}

impl KubernetesServiceDiscovery {
    /// Create new Kubernetes service discovery
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created or if there are
    /// issues with the Kubernetes service account configuration.
    pub fn new(namespace: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to create HTTP client: {e}"),
                    service: Some("kubernetes".to_string()),
                    timeout: None,
                    suggestion: Some("Check Kubernetes service account configuration".to_string()),
                }))
            })?;

        let api_server = std::env::var("KUBERNETES_SERVICE_HOST")
            .map(|host| {
                let port =
                    std::env::var("KUBERNETES_SERVICE_PORT").unwrap_or_else(|_| "443".to_string());
                format!("https://{host}:{port}")
            })
            .unwrap_or_else(|_| "https://kubernetes.default.svc".to_string());

        Ok(Self {
            namespace,
            client,
            api_server,
        })
    }

    /// Get service account token
    async fn get_service_account_token(&self) -> Result<String> {
        tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to read service account token: {e}"),
                    service: Some("kubernetes".to_string()),
                    timeout: None,
                    suggestion: Some(
                        "Ensure running in Kubernetes cluster with proper service account"
                            .to_string(),
                    ),
                }))
            })
    }

    /// Build service definition for Kubernetes
    fn build_service_definition(&self, service: &ServiceInstance) -> Value {
        json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": service.name,
                "namespace": self.namespace,
                "labels": {
                    "app": service.name,
                    "managed-by": "songbird-orchestrator"
                }
            },
            "spec": {
                "selector": {
                    "app": service.name
                },
                "ports": [{
                    "port": service.address.port(),
                    "targetPort": service.address.port(),
                    "protocol": "TCP"
                }]
            }
        })
    }

    /// Parse Kubernetes service into ServiceInstance
    fn parse_kubernetes_service(&self, k8s_service: &Value) -> Option<ServiceInstance> {
        let metadata = k8s_service.get("metadata")?;
        let spec = k8s_service.get("spec")?;

        let name = metadata["name"].as_str()?.to_string();
        let id = format!("{}.{}", name, self.namespace);

        // Extract cluster IP and port
        let cluster_ip = spec["clusterIP"].as_str()?;
        let ports = spec["ports"].as_array()?;
        let port = ports.first()?["port"].as_u64()? as u16;

        let address = format!("{}:{}", cluster_ip, port)
            .parse::<SocketAddr>()
            .ok()?;

        // Extract labels as tags
        let tags = metadata["labels"]
            .as_object()
            .map(|labels| {
                labels
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("")))
                    .collect()
            })
            .unwrap_or_default();

        // Extract annotations as metadata
        let metadata_map = metadata["annotations"]
            .as_object()
            .map(|annotations| {
                annotations
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Some(ServiceInstance {
            id,
            name,
            address,
            metadata: metadata_map,
            health_check_url: None,
            tags,
        })
    }
}

#[async_trait]
impl ServiceDiscovery for KubernetesServiceDiscovery {
    async fn register_service(&self, service: ServiceInstance) -> Result<()> {
        tracing::info!(
            "Registering service {} in Kubernetes namespace {}",
            service.name,
            self.namespace
        );

        let service_def = self.build_service_definition(&service);
        let token = self.get_service_account_token().await?;

        let url = format!(
            "{}/api/v1/namespaces/{}/services",
            self.api_server, self.namespace
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&service_def)
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to register service with Kubernetes: {}", e),
                    service: Some(service.name.clone()),
                    timeout: None,
                    suggestion: Some("Check Kubernetes API connectivity and RBAC".to_string()),
                }))
            })?;

        if response.status().is_success() {
            tracing::info!(
                "Successfully registered service {} in Kubernetes",
                service.name
            );
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Kubernetes service registration failed: {}", error_text),
                service: Some(service.name),
                timeout: None,
                suggestion: Some("Check service definition and RBAC permissions".to_string()),
            })))
        }
    }

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        tracing::info!(
            "Deregistering service {} from Kubernetes namespace {}",
            service_id,
            self.namespace
        );

        let token = self.get_service_account_token().await?;

        // Extract service name from service_id (format: name.namespace)
        let service_name = service_id.split('.').next().unwrap_or(service_id);

        let url = format!(
            "{}/api/v1/namespaces/{}/services/{}",
            self.api_server, self.namespace, service_name
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to deregister service from Kubernetes: {}", e),
                    service: Some(service_id.to_string()),
                    timeout: None,
                    suggestion: Some("Check Kubernetes API connectivity".to_string()),
                }))
            })?;

        if response.status().is_success() {
            tracing::info!(
                "Successfully deregistered service {} from Kubernetes",
                service_id
            );
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Kubernetes service deregistration failed: {}", error_text),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check if service exists and RBAC permissions".to_string()),
            })))
        }
    }

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        let token = self.get_service_account_token().await?;

        let url = match service_name {
            Some(name) => format!(
                "{}/api/v1/namespaces/{}/services/{}",
                self.api_server, self.namespace, name
            ),
            None => format!(
                "{}/api/v1/namespaces/{}/services",
                self.api_server, self.namespace
            ),
        };

        tracing::debug!("Discovering services from Kubernetes: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to discover services from Kubernetes: {}", e),
                    service: service_name.map(String::from),
                    timeout: None,
                    suggestion: Some("Check Kubernetes API connectivity and RBAC".to_string()),
                }))
            })?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Kubernetes service discovery failed: {}", error_text),
                service: service_name.map(String::from),
                timeout: None,
                suggestion: Some("Check namespace and RBAC permissions".to_string()),
            })));
        }

        let k8s_response: Value = response.json().await.map_err(|e| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: format!("Failed to parse Kubernetes response: {}", e),
                service: service_name.map(String::from),
                timeout: None,
                suggestion: Some("Check Kubernetes API version compatibility".to_string()),
            }))
        })?;

        let services = if service_name.is_some() {
            // Single service response
            vec![k8s_response]
        } else {
            // Service list response
            k8s_response["items"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .cloned()
                .collect()
        };

        let parsed_services: Vec<ServiceInstance> = services
            .iter()
            .filter_map(|service| self.parse_kubernetes_service(service))
            .collect();

        tracing::debug!(
            "Discovered {} services from Kubernetes",
            parsed_services.len()
        );
        Ok(parsed_services)
    }

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        // For Kubernetes, we check if the service exists and has endpoints
        let token = self.get_service_account_token().await?;

        // Extract service name from service_id
        let service_name = service_id.split('.').next().unwrap_or(service_id);

        let endpoints_url = format!(
            "{}/api/v1/namespaces/{}/endpoints/{}",
            self.api_server, self.namespace, service_name
        );

        let response = self
            .client
            .get(&endpoints_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| {
                SongbirdError::Discovery(Box::new(DiscoveryError {
                    message: format!("Failed to check service endpoints in Kubernetes: {}", e),
                    service: Some(service_id.to_string()),
                    timeout: None,
                    suggestion: Some("Check Kubernetes API connectivity".to_string()),
                }))
            })?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let endpoints_response: Value = response.json().await.map_err(|_| {
            SongbirdError::Discovery(Box::new(DiscoveryError {
                message: "Failed to parse Kubernetes endpoints response".to_string(),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check Kubernetes API version".to_string()),
            }))
        })?;

        // Check if service has ready endpoints
        let has_ready_endpoints = endpoints_response["subsets"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .any(|subset| {
                subset["addresses"]
                    .as_array()
                    .map_or(false, |addrs| !addrs.is_empty())
            });

        tracing::debug!(
            "Health check for service {}: {}",
            service_id,
            has_ready_endpoints
        );
        Ok(has_ready_endpoints)
    }
}
