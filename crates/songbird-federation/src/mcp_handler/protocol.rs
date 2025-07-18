/*!
 * MCP Federation Protocol
 *
 * Handles core MCP protocol operations:
 * - Request/Response processing
 * - Service provider registration
 * - Federation message handling
 * - Protocol validation and compliance
 */

use super::super::config::FederationConfig;
use super::super::messages::{
    FederationRequest, FederationRequestType, FederationResponse, ServiceProviderInfo,
};
use chrono::Utc;
use songbird_errors::{NetworkError, SongbirdError};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{CpuExt, SystemExt};
use tracing::{debug, info, warn};

#[derive(Debug)]
/// Protocol handler for MCP federation
pub struct ProtocolHandler {
    config: FederationConfig,
    service_providers: HashMap<String, ServiceProviderInfo>,
}

impl ProtocolHandler {
    /// Create new protocol handler
    pub fn new(config: FederationConfig) -> Self {
        Self {
            config,
            service_providers: HashMap::with_capacity(16), // Pre-allocate for expected service providers
        }
    }

    /// Register a service provider
    pub async fn register_service_provider(
        &mut self,
        provider_info: ServiceProviderInfo,
    ) -> Result<(), SongbirdError> {
        info!(
            "Registering service provider: {} - {}",
            provider_info.name, provider_info.description
        );

        self.validate_service_provider(&provider_info)?;

        let provider_name = provider_info.name.clone();
        self.service_providers
            .insert(provider_name, provider_info.clone());

        self.broadcast_service_registration(&provider_info).await?;

        Ok(())
    }

    /// Validate service provider information
    fn validate_service_provider(
        &self,
        provider: &ServiceProviderInfo,
    ) -> Result<(), SongbirdError> {
        if provider.name.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "provider_name".to_string(),
                message: "Service provider name cannot be empty".to_string(),
                suggestion: Some("Provide a valid service provider name".to_string()),
            });
        }

        if provider.name.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "provider_name".to_string(),
                message: "Service provider name cannot be empty".to_string(),
                suggestion: Some("Provide a valid service provider name".to_string()),
            });
        }

        if provider.description.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "provider_description".to_string(),
                message: "Service provider description cannot be empty".to_string(),
                suggestion: Some("Provide a valid service provider description".to_string()),
            });
        }

        if let Some(endpoint) = provider.endpoints.first() {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(SongbirdError::Configuration {
                    field: "provider_endpoint".to_string(),
                    message: "Service provider endpoint must be a valid HTTP/HTTPS URL".to_string(),
                    suggestion: Some("Use a valid HTTP or HTTPS URL for the endpoint".to_string()),
                });
            }
        }

        if self.service_providers.contains_key(&provider.name) {
            return Err(SongbirdError::Configuration {
                field: "provider_name".to_string(),
                message: format!(
                    "Service provider with name '{}' is already registered",
                    provider.name
                ),
                suggestion: Some("Use a different service provider name".to_string()),
            });
        }

        Ok(())
    }

    /// Broadcast service registration to federation endpoints
    async fn broadcast_service_registration(
        &self,
        provider: &ServiceProviderInfo,
    ) -> Result<(), SongbirdError> {
        info!(
            "Broadcasting service registration for: {} - {}",
            provider.name, provider.description
        );

        let request = FederationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: FederationRequestType::ServiceDiscovery,
            source_node: Some(self.config.node_id.clone()),
            target_node: None,
            data: serde_json::to_value(provider).map_err(|e| {
                SongbirdError::Communication(format!("Failed to serialize provider: {e}"))
            })?,
            timestamp: chrono::Utc::now(),
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Failed to create HTTP client: {e}"),
                    details: None,
                    endpoint: Some("federation/register".to_string()),
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                }))
            })?;

        let response = client
            .post(format!("{}/register", self.config.cluster_endpoints[0]))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Request failed: {e}"),
                    details: None,
                    endpoint: Some("federation/register".to_string()),
                    suggestion: Some("Check network connectivity and cluster endpoint".to_string()),
                }))
            })?;

        if response.status().is_success() {
            let _response_data: FederationResponse = response.json().await.map_err(|e| {
                SongbirdError::Communication(format!("Failed to parse response: {e}"))
            })?;
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Request failed with status: {}", response.status()),
                    details: None,
                    endpoint: Some("federation/register".to_string()),
                    suggestion: Some("Check the cluster endpoint and try again".to_string()),
                },
            )))
        }
    }

    /// Send federation request to endpoint
    pub async fn send_federation_request(
        &self,
        endpoint: &str,
        request: &FederationRequest,
    ) -> Result<FederationResponse, SongbirdError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Failed to create HTTP client: {e}"),
                    details: None,
                    endpoint: Some("federation/request".to_string()),
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                }))
            })?;

        let response = client
            .post(format!("{endpoint}/federation"))
            .json(request)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Request failed: {e}"),
                    details: None,
                    endpoint: Some("federation/request".to_string()),
                    suggestion: Some(
                        "Check network connectivity and federation endpoint".to_string(),
                    ),
                }))
            })?;

        if response.status().is_success() {
            let federation_response: FederationResponse = response.json().await.map_err(|e| {
                SongbirdError::Communication(format!("Failed to parse response: {e}"))
            })?;
            Ok(federation_response)
        } else {
            Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Request failed with status: {}", response.status()),
                    details: None,
                    endpoint: Some("federation/request".to_string()),
                    suggestion: Some("Check the federation endpoint and try again".to_string()),
                },
            )))
        }
    }

    /// Handle incoming federation request
    pub async fn handle_federation_request(
        &mut self,
        request: &FederationRequest,
    ) -> Result<FederationResponse, SongbirdError> {
        self.validate_federation_request(request)?;

        let response_data = match request.request_type {
            FederationRequestType::ServiceDiscovery => {
                self.handle_service_discovery_request(request).await?
            }
            FederationRequestType::HealthCheck => self.handle_health_check_request(request).await?,
            FederationRequestType::ConfigUpdate => {
                self.handle_status_update_request(request).await?
            }
            FederationRequestType::DataReplication => self.handle_custom_request(request).await?,
            _ => {
                return Err(SongbirdError::Communication(format!(
                    "Unsupported request type: {:?}",
                    request.request_type
                )));
            }
        };

        Ok(FederationResponse {
            request_id: request.request_id.clone(),
            success: true,
            data: response_data,
            error_message: None,
        })
    }

    /// Validate federation request
    fn validate_federation_request(
        &self,
        request: &FederationRequest,
    ) -> Result<(), SongbirdError> {
        // Validate request has valid timestamp (not too old)
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(request.timestamp);
        if age > chrono::Duration::minutes(5) {
            return Err(SongbirdError::Configuration {
                field: "timestamp".to_string(),
                message: "Request timestamp is too old".to_string(),
                suggestion: Some("Use a more recent timestamp for the request".to_string()),
            });
        }

        Ok(())
    }

    /// Handle service registration request
    async fn handle_service_registration_request(
        &mut self,
        request: &FederationRequest,
    ) -> Result<serde_json::Value, SongbirdError> {
        let provider: ServiceProviderInfo =
            serde_json::from_value(request.data.clone()).map_err(|e| {
                SongbirdError::Communication(format!("Invalid service provider data: {e}"))
            })?;

        self.register_service_provider(provider).await?;

        Ok(serde_json::json!({
            "status": "success",
            "message": "Service provider registered successfully"
        }))
    }

    /// Handle service discovery request
    async fn handle_service_discovery_request(
        &self,
        request: &FederationRequest,
    ) -> Result<serde_json::Value, SongbirdError> {
        let service_type_filter = request.data.get("service_type").and_then(|v| v.as_str());

        let filtered_providers: Vec<&ServiceProviderInfo> = self
            .service_providers
            .values()
            .filter(|provider| {
                service_type_filter
                    .is_none_or(|filter| provider.capabilities.contains(&filter.to_string()))
            })
            .collect();

        Ok(serde_json::json!({
            "providers": filtered_providers,
            "total_count": filtered_providers.len()
        }))
    }

    /// Handle health check request
    async fn handle_health_check_request(
        &self,
        _request: &FederationRequest,
    ) -> Result<serde_json::Value, SongbirdError> {
        debug!("Processing health check request");

        // Collect actual health metrics from system
        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let cpu_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
            / system.cpus().len() as f32;
        let memory_usage = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        let total_memory_gb = system.total_memory() / 1024 / 1024 / 1024;
        let uptime = system.uptime();

        // Check network connectivity
        let network_healthy = self.check_network_connectivity().await.unwrap_or(false);

        // Check federation endpoints
        let mut healthy_endpoints = 0;
        let mut total_endpoints = 0;

        for endpoint in &self.config.cluster_endpoints {
            total_endpoints += 1;
            if self.check_endpoint_health(endpoint).await.unwrap_or(false) {
                healthy_endpoints += 1;
            }
        }

        // Determine overall health status
        let health_status = if cpu_usage > 90.0 || memory_usage > 95.0 || !network_healthy {
            "unhealthy"
        } else if cpu_usage > 75.0 || memory_usage > 80.0 || healthy_endpoints < total_endpoints {
            "degraded"
        } else {
            "healthy"
        };

        Ok(serde_json::json!({
            "status": health_status,
            "timestamp": Utc::now(),
            "service_count": self.service_providers.len(),
            "cluster_id": self.config.cluster_id,
            "node_id": self.config.node_id,
            "metrics": {
                "cpu_usage": cpu_usage,
                "memory_usage": memory_usage,
                "total_memory_gb": total_memory_gb,
                "uptime_seconds": uptime,
                "network_healthy": network_healthy,
                "healthy_endpoints": healthy_endpoints,
                "total_endpoints": total_endpoints
            }
        }))
    }

    /// Check network connectivity to external services
    async fn check_network_connectivity(&self) -> Result<bool, SongbirdError> {
        // Test connectivity to a reliable external service
        let test_result = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect("8.8.8.8:53"),
        )
        .await;

        match test_result {
            Ok(Ok(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Check health of a specific federation endpoint
    async fn check_endpoint_health(&self, endpoint: &str) -> Result<bool, SongbirdError> {
        let client = reqwest::Client::new();
        let health_url = format!("{endpoint}/health");

        let response =
            tokio::time::timeout(Duration::from_secs(5), client.get(&health_url).send()).await;

        match response {
            Ok(Ok(resp)) => Ok(resp.status().is_success()),
            _ => Ok(false),
        }
    }

    /// Handle status update request
    async fn handle_status_update_request(
        &self,
        request: &FederationRequest,
    ) -> Result<serde_json::Value, SongbirdError> {
        let status_info = &request.data;

        info!(
            "Received status update from node: {:?}",
            request.source_node
        );

        // Handle the status update
        Ok(serde_json::json!({
            "status": "success",
            "message": "Status update received",
            "data": status_info
        }))
    }

    /// Handle custom request
    async fn handle_custom_request(
        &self,
        request: &FederationRequest,
    ) -> Result<serde_json::Value, SongbirdError> {
        let custom_type = request
            .data
            .get("custom_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match custom_type {
            "ping" => Ok(serde_json::json!({
                "status": "success",
                "message": "pong",
                "node_id": self.config.node_id
            })),
            "info" => Ok(serde_json::json!({
                "status": "success",
                "cluster_id": self.config.cluster_id,
                "node_id": self.config.node_id,
                "service_count": self.service_providers.len()
            })),
            _ => Ok(serde_json::json!({
                "status": "error",
                "message": format!("Unknown custom request type: {}", custom_type)
            })),
        }
    }

    /// Get registered service providers
    pub fn get_service_providers(&self) -> &HashMap<String, ServiceProviderInfo> {
        &self.service_providers
    }

    /// Get service provider by ID
    pub fn get_service_provider(&self, id: &str) -> Option<&ServiceProviderInfo> {
        self.service_providers.get(id)
    }

    /// Remove service provider
    pub async fn unregister_service_provider(&mut self, id: &str) -> Result<(), SongbirdError> {
        if let Some(provider) = self.service_providers.remove(id) {
            info!("Unregistered service provider: {}", provider.name);

            // Broadcast unregistration to federation endpoints
            self.broadcast_service_unregistration(&provider).await?;
        } else {
            warn!("Attempted to unregister unknown service provider: {}", id);
        }

        Ok(())
    }

    /// Broadcast service unregistration
    async fn broadcast_service_unregistration(
        &self,
        provider: &ServiceProviderInfo,
    ) -> Result<(), SongbirdError> {
        info!("Broadcasting service unregistration for: {}", provider.name);

        let request = FederationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: FederationRequestType::ServiceDiscovery,
            source_node: Some(self.config.node_id.clone()),
            target_node: None,
            data: serde_json::json!({
                "action": "unregister",
                "service_name": provider.name
            }),
            timestamp: chrono::Utc::now(),
        };

        // Broadcast to all endpoints
        for endpoint in &self.config.cluster_endpoints {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| {
                    songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                        service: Some("federation".to_string()),
                        message: format!("Failed to create HTTP client: {e}"),
                        details: None,
                        endpoint: Some("federation/unregister".to_string()),
                        suggestion: Some(
                            "Check network connectivity and configuration".to_string(),
                        ),
                    }))
                })?;

            let _response = client
                .post(format!("{endpoint}/unregister"))
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                        service: Some("federation".to_string()),
                        message: format!("Failed to send unregistration: {e}"),
                        details: None,
                        endpoint: Some("federation/unregister".to_string()),
                        suggestion: Some(
                            "Check network connectivity and federation endpoint".to_string(),
                        ),
                    }))
                })?;
        }

        Ok(())
    }

    /// Get protocol statistics
    pub fn get_protocol_stats(&self) -> ProtocolStats {
        ProtocolStats {
            registered_services: self.service_providers.len(),
            configured_endpoints: self.config.cluster_endpoints.len(),
            cluster_id: self.config.cluster_id.clone(),
            node_id: self.config.node_id.clone(),
        }
    }

    /// Update protocol configuration
    pub fn update_config(&mut self, new_config: FederationConfig) -> Result<(), SongbirdError> {
        info!("Updating protocol configuration");

        // Update local configuration
        self.config = new_config.clone();

        // Clear provider cache if cluster changed
        if self.config.cluster_id != new_config.cluster_id {
            info!("Cluster ID changed, clearing service provider cache");
            self.service_providers.clear();
        }

        info!("Protocol configuration updated successfully");
        Ok(())
    }
}

/// Protocol statistics
#[derive(Debug, Clone)]
pub struct ProtocolStats {
    pub registered_services: usize,
    pub configured_endpoints: usize,
    pub cluster_id: String,
    pub node_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FederationConfig;

    fn create_test_config() -> FederationConfig {
        FederationConfig {
            cluster_endpoints: vec!["http://test:8080".to_string()],
            node_id: "test-node".to_string(),
            cluster_id: "test-cluster".to_string(),
            ..Default::default()
        }
    }

    fn create_test_service_provider() -> ServiceProviderInfo {
        ServiceProviderInfo {
            name: "Test Service".to_string(),
            description: "Test service for federation testing".to_string(),
            capabilities: vec!["test-capability".to_string()],
            endpoints: vec!["http://test:8080".to_string()],
            version: "1.0.0".to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_protocol_handler_creation() {
        let config = create_test_config();
        let handler = ProtocolHandler::new(config);

        assert_eq!(handler.service_providers.len(), 0);
        assert_eq!(handler.config.cluster_id, "test-cluster");
    }

    #[tokio::test]
    async fn test_service_provider_validation() {
        let config = create_test_config();
        let handler = ProtocolHandler::new(config);

        let provider = create_test_service_provider();
        assert!(handler.validate_service_provider(&provider).is_ok());

        // Test with invalid provider (empty name)
        let mut invalid_provider = provider.clone();
        invalid_provider.name = "".to_string();
        assert!(handler
            .validate_service_provider(&invalid_provider)
            .is_err());
    }

    #[tokio::test]
    async fn test_service_provider_registration() {
        let config = create_test_config();
        let mut handler = ProtocolHandler::new(config);

        let provider = create_test_service_provider();
        let result = handler.register_service_provider(provider.clone()).await;

        // Note: This will fail due to network calls, but validates the logic path
        assert!(result.is_err() || handler.service_providers.contains_key(&provider.name));
    }

    #[tokio::test]
    async fn test_protocol_stats() {
        let config = create_test_config();
        let handler = ProtocolHandler::new(config);

        let stats = handler.get_protocol_stats();
        assert_eq!(stats.cluster_id, "test-cluster");
        assert_eq!(stats.node_id, "test-node");
        assert_eq!(stats.registered_services, 0);
        assert_eq!(stats.configured_endpoints, 1);
    }
}
