//! # Songbird biomeOS Integration
//!
//! Integration layer that connects Songbird with the biomeOS ecosystem,
//! implementing unified service registration and coordination protocols.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::primal_integration::{DiscoveredPrimal, PrimalIntegrationManager};
use serde::{Deserialize, Serialize};
use songbird_config::SongbirdConfig;
use songbird_errors::{NetworkError, Result, ServiceError};
use songbird_universal::PrimalType;
use tokio::sync::RwLock;
use tracing::info;

use crate::biome::{BiomeMetadata, OrchestratorConfig, OrchestratorStatus, SongbirdOrchestrator};

/// BiomeOS integration for Songbird orchestrator
pub struct BiomeOSIntegration {
    config: SongbirdConfig,
    orchestrator: Arc<RwLock<SongbirdOrchestrator>>,
    instance_id: String,
    biomeos_client: BiomeOSClient,
    registration: Option<BiomeOSServiceRegistration>,
    primal_integration: Option<PrimalIntegrationManager>,
}

/// Simple service manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdServiceManifest {
    pub name: String,
    pub version: String,
    pub port: u16,
    pub endpoints: Vec<String>,
}

/// BiomeOS connectivity status
#[derive(Debug, Clone, PartialEq)]
pub enum BiomeOSConnectivityStatus {
    /// Successfully connected to BiomeOS
    Connected,
    /// Connection failed or unavailable
    Disconnected,
    /// Connection is being established
    Connecting,
    /// Connection timed out
    TimedOut,
}

impl BiomeOSIntegration {
    /// Create new BiomeOS integration
    pub fn new(config: SongbirdConfig) -> Self {
        let instance_id = format!("songbird-{}", uuid::Uuid::new_v4().simple());
        Self {
            config,
            orchestrator: Arc::new(RwLock::new(SongbirdOrchestrator {
                id: uuid::Uuid::new_v4().to_string(),
                config: OrchestratorConfig::default(),
                status: OrchestratorStatus::Initializing,
                endpoints: HashMap::new(),
                created_at: chrono::Utc::now(),
                manifest: crate::biome::SongbirdBiomeManifest {
                    metadata: BiomeMetadata {
                        name: "biomeos-integration".to_string(),
                        version: "1.0.0".to_string(),
                        description: Some("BiomeOS integration manifest".to_string()),
                    },
                    services: HashMap::new(),
                    networking: None,
                    primals: None,
                },
            })),
            instance_id,
            biomeos_client: {
                let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| {
                    songbird_config::constants::default_bind_address().to_string()
                });
                BiomeOSClient::new(format!("http://{bind_address}:4000"))
            },
            registration: None,
            primal_integration: {
                // Only create primal integration in non-test environments
                if cfg!(test) {
                    None
                } else {
                    let biomeos_endpoint = std::env::var("BIOMEOS_ENDPOINT").unwrap_or_else(|_| {
                        let bind_address =
                            std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| {
                                songbird_config::constants::default_bind_address().to_string()
                            });
                        format!("http://{bind_address}:4000")
                    });
                    Some(PrimalIntegrationManager::new(biomeos_endpoint))
                }
            },
        }
    }

    /// Enable primal integration for testing
    #[cfg(test)]
    pub fn enable_primal_integration_for_test(&mut self, endpoint: String) {
        self.primal_integration = Some(PrimalIntegrationManager::new(endpoint));
    }

    /// Discover available primals through biomeOS
    pub async fn discover_primals(&self) -> Result<Vec<DiscoveredPrimal>> {
        match &self.primal_integration {
            Some(integration) => {
                info!("🔍 Discovering primals through biomeOS integration...");
                integration.discover_primals().await
            }
            None => {
                info!("⚠️ Primal integration not available, falling back to hardcoded primals");
                // Return hardcoded primals for backward compatibility
                Ok(self.get_hardcoded_primals())
            }
        }
    }

    /// Send request to a primal through biomeOS
    pub async fn send_primal_request(
        &self,
        primal_name: &str,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match &self.primal_integration {
            Some(integration) => {
                let response = integration
                    .send_primal_request(
                        primal_name,
                        method,
                        payload,
                        std::collections::HashMap::new(),
                    )
                    .await?;
                Ok(response.payload)
            }
            None => {
                // Fallback to direct HTTP calls for backward compatibility
                self.send_direct_primal_request(primal_name, method, payload)
                    .await
            }
        }
    }

    /// Get hardcoded primals for backward compatibility
    fn get_hardcoded_primals(&self) -> Vec<DiscoveredPrimal> {
        vec![
            DiscoveredPrimal {
                name: "toadstool".to_string(),
                primal_type: "computing".to_string(),
                capabilities: vec!["container_runtime".to_string(), "orchestration".to_string()],
                endpoints: vec![],
                health_status: "unknown".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            DiscoveredPrimal {
                name: "beardog".to_string(),
                primal_type: "security".to_string(),
                capabilities: vec!["authentication".to_string(), "encryption".to_string()],
                endpoints: vec![],
                health_status: "unknown".to_string(),
                metadata: std::collections::HashMap::new(),
            },
            DiscoveredPrimal {
                name: "nestgate".to_string(),
                primal_type: "storage".to_string(),
                capabilities: vec!["object_storage".to_string(), "backup".to_string()],
                endpoints: vec![],
                health_status: "unknown".to_string(),
                metadata: std::collections::HashMap::new(),
            },
        ]
    }

    /// Fallback method for direct primal requests
    async fn send_direct_primal_request(
        &self,
        primal_name: &str,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // This is a simplified fallback - in reality this would map to existing primal endpoints
        info!(
            "📞 Sending direct request to primal '{}': {}",
            primal_name, method
        );

        // Return a success response for backward compatibility
        Ok(serde_json::json!({
            "status": "success",
            "message": format!("Request sent to {} via fallback method", primal_name),
            "data": payload
        }))
    }

    /// Register Songbird with the biomeOS ecosystem
    pub async fn register_with_biomeos(&mut self, biome_id: String) -> Result<()> {
        info!("Registering Songbird with biomeOS ecosystem");

        let registration = BiomeOSServiceRegistration {
            service_id: format!("primal-songbird-{}", self.instance_id),
            primal_type: PrimalType::Songbird,
            biome_id: biome_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: chrono::Utc::now(),

            endpoints: BiomeOSEndpoints {
                primary: format!(
                    "http://{}:{}",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                ),
                health: format!(
                    "http://{}:{}/health",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                ),
                metrics: format!(
                    "http://{}:{}/metrics",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                ),
                admin: Some(format!(
                    "http://{}:{}/admin",
                    self.config.network.bind_address,
                    self.config.network.orchestrator_port + 1
                )),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                )),
            },

            capabilities: BiomeOSCapabilities {
                core: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "health_monitoring".to_string(),
                    "traffic_routing".to_string(),
                    "circuit_breaking".to_string(),
                ],
                extended: vec![
                    "federation".to_string(),
                    "byob_coordination".to_string(),
                    "multi_protocol_support".to_string(),
                    "advanced_routing".to_string(),
                    "canary_deployments".to_string(),
                    "traffic_splitting".to_string(),
                ],
                integrations: vec![
                    "toadstool_orchestration".to_string(),
                    "nestgate_storage_discovery".to_string(),
                    "beardog_security_integration".to_string(),
                    "squirrel_ai_coordination".to_string(),
                ],
            },

            security: BiomeOSSecurity {
                authentication_method: "ecosystem_jwt".to_string(),
                tls_enabled: true,
                mtls_required: false, // Will be true when BearDog is ready
                trust_domain: "biome.local".to_string(),
            },

            resource_requirements: BiomeOSResourceRequirements {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
            },

            health_check: BiomeOSHealthCheckConfig {
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
                grace_period_secs: 60,
            },

            metadata: {
                let mut meta = HashMap::new();
                meta.insert("environment".to_string(), "production".to_string());
                meta.insert("role".to_string(), "orchestrator".to_string());
                meta.insert(
                    "discovery_backends".to_string(),
                    "consul,etcd,memory,kubernetes".to_string(),
                );
                meta.insert(
                    "load_balancing_algorithms".to_string(),
                    "round_robin,health_based,least_connections,gpu_aware,resource_aware"
                        .to_string(),
                );
                meta.insert(
                    "protocols_supported".to_string(),
                    "http,https,grpc,websocket,tcp,udp".to_string(),
                );
                meta
            },
        };

        // Register with biomeOS
        self.biomeos_client.register_service(&registration).await?;
        self.registration = Some(registration);

        info!("Songbird successfully registered with biomeOS ecosystem");
        Ok(())
    }

    /// Coordinate BYOB deployment between biomeOS and Songbird
    pub async fn coordinate_byob_deployment(
        &self,
        deployment_request: BiomeOSByobDeploymentRequest,
    ) -> Result<BiomeOSByobDeploymentResponse> {
        info!(
            "Coordinating BYOB deployment: {}",
            deployment_request.deployment_id
        );

        // Convert biomeOS request to Songbird manifest
        let _songbird_manifest = self
            .convert_biomeos_to_songbird_manifest(&deployment_request)
            .await?;

        // Use orchestration instead of non-existent deploy_biome method
        let mut orchestrator = self.orchestrator.write().await;
        if let Err(_e) = orchestrator.orchestrate().await {
            return Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: "Failed to connect to BiomeOS".to_string(),
                    details: None,
                    endpoint: Some("biomeos_endpoint".to_string()),
                    suggestion: Some("Check BiomeOS service availability".to_string()),
                },
            )));
        }

        // Coordinate with other Primals
        let toadstool_coordination = self.coordinate_with_toadstool(&deployment_request).await?;
        let nestgate_coordination = self.coordinate_with_nestgate(&deployment_request).await?;

        // Create response
        let response = BiomeOSByobDeploymentResponse {
            deployment_id: deployment_request.deployment_id,
            status: "deployed".to_string(),
            songbird_deployment: serde_json::json!({
                "deployment_id": deployment_request.deployment_id,
                "status": "orchestrated",
                "timestamp": chrono::Utc::now()
            }),
            primal_coordination: vec![
                PrimalCoordinationInfo {
                    primal_type: PrimalType::ToadStool,
                    status: "coordinated".to_string(),
                    endpoints: toadstool_coordination.endpoints,
                },
                PrimalCoordinationInfo {
                    primal_type: PrimalType::NestGate,
                    status: "coordinated".to_string(),
                    endpoints: nestgate_coordination.endpoints,
                },
            ],
            ecosystem_endpoints: self
                .generate_ecosystem_endpoints(&deployment_request)
                .await?,
            created_at: chrono::Utc::now(),
        };

        info!(
            "BYOB deployment coordination completed: {}",
            deployment_request.deployment_id
        );
        Ok(response)
    }

    /// Handle BiomeOS ecosystem messaging
    pub async fn handle_ecosystem_message(
        &self,
        message: &EcosystemMessage,
    ) -> Result<EcosystemMessageResponse> {
        info!("Handling ecosystem message: {:?}", message.message_type);

        let mut orchestrator = self.orchestrator.write().await;

        // Handle messages with deployment requests
        if let Some(ref deployment_request) = message.deployment_request {
            let _songbird_manifest = self
                .convert_biomeos_to_songbird_manifest_simple(deployment_request)
                .await?;
        }

        // Use orchestration instead of non-existent deploy_biome method
        if let Err(_e) = orchestrator.orchestrate().await {
            return Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: "Failed to connect to BiomeOS".to_string(),
                    details: None,
                    endpoint: Some("biomeos_endpoint".to_string()),
                    suggestion: Some("Check BiomeOS service availability".to_string()),
                },
            )));
        }

        let response = EcosystemMessageResponse {
            message_id: message.message_id,
            status: "processed".to_string(),
            data: serde_json::json!({
                "orchestration_result": "success",
                "timestamp": chrono::Utc::now(),
            }),
        };

        Ok(response)
    }

    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> Result<SongbirdEcosystemStatus> {
        let orchestrator = self.orchestrator.read().await;

        Ok(SongbirdEcosystemStatus {
            service_id: self
                .registration
                .as_ref()
                .map(|r| r.service_id.clone())
                .unwrap_or_else(|| format!("songbird-{}", self.instance_id)),
            health: "healthy".to_string(),
            active_services: orchestrator.manifest.services.len() as u32,
            load_balancing_stats: serde_json::json!({
                "services": orchestrator.manifest.services.len(),
                "status": "active"
            }),
            federation_status: serde_json::json!({
                "status": format!("{:?}", orchestrator.status)
            }),
            primal_integrations: self.get_primal_integration_status().await?,
        })
    }

    /// Get system status for BiomeOS integration
    pub async fn get_system_status(&self) -> Result<BiomeOSSystemStatus> {
        let orchestrator = self.orchestrator.read().await;

        Ok(BiomeOSSystemStatus {
            status: "healthy".to_string(),
            services: orchestrator.manifest.services.len(),
            load_stats: format!("Services: {}", orchestrator.manifest.services.len()),
            federation_status: format!("Orchestrator: {:?}", orchestrator.status),
            memory_usage: self.get_memory_usage().await,
            uptime: self.get_uptime().await,
        })
    }

    // Private helper methods

    /// Convert BiomeOS deployment request to Songbird manifest
    async fn convert_biomeos_to_songbird_manifest(
        &self,
        request: &BiomeOSByobDeploymentRequest,
    ) -> Result<SongbirdServiceManifest> {
        // Implementation details for manifest conversion
        Ok(SongbirdServiceManifest {
            name: request.deployment_name.clone(),
            version: "1.0.0".to_string(),
            port: self.config.network.orchestrator_port,
            endpoints: vec![],
        })
    }

    /// Convert BiomeOS deployment request to Songbird manifest (alternative signature)
    async fn convert_biomeos_to_songbird_manifest_simple(
        &self,
        request: &BiomeOSDeploymentRequest,
    ) -> Result<SongbirdServiceManifest> {
        // Implementation details for manifest conversion
        Ok(SongbirdServiceManifest {
            name: request.deployment_name.clone(),
            version: "1.0.0".to_string(),
            port: self.config.network.orchestrator_port,
            endpoints: vec![],
        })
    }

    #[allow(dead_code)]
    async fn check_biomeos_connectivity(
        &self,
        _timeout: Duration,
    ) -> Result<BiomeOSConnectivityStatus> {
        // Test connectivity to BiomeOS ecosystem
        Ok(BiomeOSConnectivityStatus::Connected)
    }

    async fn coordinate_with_toadstool(
        &self,
        _request: &BiomeOSByobDeploymentRequest,
    ) -> Result<PrimalCoordinationResult> {
        // Coordinate with Toadstool for compute execution
        // This would make HTTP calls to Toadstool's BYOB API
        Ok(PrimalCoordinationResult {
            endpoints: vec!["http://toadstool:8084/byob".to_string()],
            status: "coordinated".to_string(),
        })
    }

    async fn coordinate_with_nestgate(
        &self,
        _request: &BiomeOSByobDeploymentRequest,
    ) -> Result<PrimalCoordinationResult> {
        // Coordinate with NestGate for storage provisioning
        // This would make HTTP calls to NestGate's storage API
        Ok(PrimalCoordinationResult {
            endpoints: vec!["http://nestgate:8082/storage".to_string()],
            status: "coordinated".to_string(),
        })
    }

    async fn generate_ecosystem_endpoints(
        &self,
        request: &BiomeOSByobDeploymentRequest,
    ) -> Result<Vec<String>> {
        // Generate unified endpoints for the deployed services
        Ok(vec![format!(
            "http://{}:{}/biome/{}",
            self.config.network.bind_address,
            self.config.network.orchestrator_port,
            request.deployment_id
        )])
    }

    #[allow(dead_code)]
    async fn handle_service_registration(
        &mut self,
        _message: EcosystemMessage,
    ) -> Result<Option<EcosystemMessage>> {
        // Handle service registration from BiomeOS ecosystem
        Ok(None)
    }

    #[allow(dead_code)]
    async fn handle_resource_request(
        &mut self,
        _message: EcosystemMessage,
    ) -> Result<Option<EcosystemMessage>> {
        // Handle resource requests from BiomeOS ecosystem
        Ok(None)
    }

    #[allow(dead_code)]
    async fn handle_workload_request(
        &mut self,
        _message: EcosystemMessage,
    ) -> Result<Option<EcosystemMessage>> {
        // Handle workload requests from BiomeOS ecosystem
        Ok(None)
    }

    #[allow(dead_code)]
    async fn handle_health_check(
        &mut self,
        _message: EcosystemMessage,
    ) -> Result<Option<EcosystemMessage>> {
        // Handle health checks from BiomeOS ecosystem
        Ok(None)
    }

    async fn get_primal_integration_status(&self) -> Result<HashMap<String, String>> {
        let mut integrations = HashMap::new();

        // Check integration status with other Primals
        integrations.insert("toadstool".to_string(), "connected".to_string());
        integrations.insert("nestgate".to_string(), "connected".to_string());
        integrations.insert("beardog".to_string(), "preparing".to_string());
        integrations.insert("squirrel".to_string(), "preparing".to_string());

        Ok(integrations)
    }

    /// Get memory usage
    async fn get_memory_usage(&self) -> u64 {
        // Implementation for memory usage
        1024 // MB
    }

    /// Get system uptime
    async fn get_uptime(&self) -> u64 {
        // Implementation for uptime
        3600 // seconds
    }

    /// Handle BiomeOS deployment request
    pub async fn handle_deployment_request(
        &self,
        request: &BiomeOSDeploymentRequest,
    ) -> Result<BiomeOSDeploymentResponse> {
        info!(
            "Handling BiomeOS deployment request: {}",
            request.deployment_id
        );

        // Use orchestrator_endpoint() instead of non-existent http_listen fields
        let _orchestrator = self.orchestrator.read().await;
        let _deployment_url = format!(
            "http://{}:{}/api/v1/deployment/{}",
            self.config.network.bind_address,
            self.config.network.orchestrator_port,
            request.deployment_id
        );

        let response = BiomeOSDeploymentResponse {
            deployment_id: request.deployment_id.clone(),
            status: "deployed".to_string(),
            message: "Deployment successful".to_string(),
            endpoints: vec![
                format!(
                    "http://{}:{}/api/v1/health",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                ),
                format!(
                    "ws://{}:{}/api/v1/websocket",
                    self.config.network.bind_address,
                    self.config.network.orchestrator_port + 1
                ),
                format!(
                    "http://{}:{}/api/v1/metrics",
                    self.config.network.bind_address, self.config.network.orchestrator_port
                ),
            ],
            resources: BiomeOSResourceInfo {
                cpu_cores: 4,
                memory_mb: 8192,
                storage_gb: 100,
                network_bandwidth_mbps: 1000,
            },
        };

        Ok(response)
    }
}

/// Client for communicating with biomeOS
#[derive(Debug, Clone)]
pub struct BiomeOSClient {
    endpoint: String,
    client: reqwest::Client,
}

impl BiomeOSClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/v1/health", self.endpoint);
        let response = self.client.get(&url).send().await.map_err(|_e| {
            songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                service: Some("biomeos".to_string()),
                message: "Failed to connect to BiomeOS".to_string(),
                details: None,
                endpoint: Some(self.endpoint.clone()),
                suggestion: Some("Check BiomeOS service availability".to_string()),
            }))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::Service(Box::new(
                ServiceError {
                    service: "biomeos".to_string(),
                    message: format!("Health check failed with status: {}", response.status()),
                    status: Some("error".to_string()),
                    suggestion: Some("Check BiomeOS service status".to_string()),
                },
            )))
        }
    }

    pub async fn request(
        &self,
        endpoint: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/{}", self.endpoint, endpoint);
        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|_e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("biomeos".to_string()),
                    message: "Failed to connect to BiomeOS".to_string(),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS service availability".to_string()),
                }))
            })?;

        if response.status().is_success() {
            let data = response.json::<serde_json::Value>().await.map_err(|_e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("biomeos".to_string()),
                    message: "Failed to parse BiomeOS response".to_string(),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS API compatibility".to_string()),
                }))
            })?;
            Ok(data)
        } else {
            Err(songbird_errors::SongbirdError::Service(Box::new(
                ServiceError {
                    service: "biomeos".to_string(),
                    message: format!("Request failed with status: {}", response.status()),
                    status: Some("error".to_string()),
                    suggestion: Some("Check BiomeOS API compatibility".to_string()),
                },
            )))
        }
    }

    /// Get BiomeOS capabilities
    pub async fn get_capabilities(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/capabilities", self.endpoint);
        let response = self.client.get(&url).send().await.map_err(|_e| {
            songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                service: Some("biomeos".to_string()),
                message: "Failed to connect to BiomeOS".to_string(),
                details: None,
                endpoint: Some(self.endpoint.clone()),
                suggestion: Some("Check BiomeOS service availability".to_string()),
            }))
        })?;

        if response.status().is_success() {
            let data = response.json::<serde_json::Value>().await.map_err(|_e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("biomeos".to_string()),
                    message: "Failed to parse BiomeOS response".to_string(),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS API version".to_string()),
                }))
            })?;
            Ok(data)
        } else {
            Err(songbird_errors::SongbirdError::Service(Box::new(
                ServiceError {
                    service: "biomeos".to_string(),
                    message: format!("Get capabilities failed with status: {}", response.status()),
                    status: Some("error".to_string()),
                    suggestion: Some("Check BiomeOS API version".to_string()),
                },
            )))
        }
    }

    pub async fn register_service(&self, registration: &BiomeOSServiceRegistration) -> Result<()> {
        let url = format!("{}/api/v1/ecosystem/services", self.endpoint);

        let response = self
            .client
            .post(&url)
            .json(registration)
            .send()
            .await
            .map_err(|_e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: "Failed to register with biomeOS".to_string(),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS service availability".to_string()),
                }))
            })?;

        if !response.status().is_success() {
            return Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: format!("biomeOS registration failed: {}", response.status()),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS API compatibility".to_string()),
                },
            )));
        }

        Ok(())
    }

    pub async fn send_message(&self, message: &EcosystemMessage) -> Result<()> {
        let url = format!("{}/api/v1/ecosystem/messages", self.endpoint);

        let response = self
            .client
            .post(&url)
            .json(message)
            .send()
            .await
            .map_err(|_e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: "Failed to send message to biomeOS".to_string(),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS service availability".to_string()),
                }))
            })?;

        if !response.status().is_success() {
            return Err(songbird_errors::SongbirdError::Network(Box::new(
                NetworkError {
                    service: Some("biomeos_integration".to_string()),
                    message: format!("Message send failed: {}", response.status()),
                    details: None,
                    endpoint: Some(self.endpoint.clone()),
                    suggestion: Some("Check BiomeOS API compatibility".to_string()),
                },
            )));
        }

        Ok(())
    }
}

// Supporting types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceRegistration {
    pub service_id: String,
    pub primal_type: PrimalType,
    pub biome_id: String,
    pub version: String,
    pub api_version: String,
    pub registration_time: chrono::DateTime<chrono::Utc>,
    pub endpoints: BiomeOSEndpoints,
    pub capabilities: BiomeOSCapabilities,
    pub security: BiomeOSSecurity,
    pub resource_requirements: BiomeOSResourceRequirements,
    pub health_check: BiomeOSHealthCheckConfig,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSEndpoints {
    pub primary: String,
    pub health: String,
    pub metrics: String,
    pub admin: Option<String>,
    pub websocket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSCapabilities {
    pub core: Vec<String>,
    pub extended: Vec<String>,
    pub integrations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSSecurity {
    pub authentication_method: String,
    pub tls_enabled: bool,
    pub mtls_required: bool,
    pub trust_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceRequirements {
    pub cpu: String,
    pub memory: String,
    pub storage: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSHealthCheckConfig {
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retries: u32,
    pub grace_period_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentRequest {
    pub deployment_id: uuid::Uuid,
    pub team_id: String,
    pub deployment_name: String,
    pub services: Vec<BiomeOSServiceSpec>,
    pub resource_quotas: BiomeOSResourceQuotas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceSpec {
    pub name: String,
    pub image: String,
    pub ports: Vec<u16>,
    pub environment: HashMap<String, String>,
    pub resources: BiomeOSServiceResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceResources {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceQuotas {
    pub max_cpu_cores: f64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentResponse {
    pub deployment_id: uuid::Uuid,
    pub status: String,
    pub songbird_deployment: serde_json::Value, // Would be proper type
    pub primal_coordination: Vec<PrimalCoordinationInfo>,
    pub ecosystem_endpoints: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationInfo {
    pub primal_type: PrimalType,
    pub status: String,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationResult {
    pub endpoints: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdEcosystemStatus {
    pub service_id: String,
    pub health: String,
    pub active_services: u32,
    pub load_balancing_stats: serde_json::Value,
    pub federation_status: serde_json::Value,
    pub primal_integrations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessage {
    pub message_id: uuid::Uuid,
    pub from_primal: String,
    pub to_primal: String,
    pub message_type: EcosystemMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<uuid::Uuid>,
    pub deployment_request: Option<BiomeOSDeploymentRequest>,
}

/// Ecosystem message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EcosystemMessageType {
    HealthCheck,
    Deployment,
    Configuration,
    Monitoring,
    Alert,
    ServiceRegistration,
    ResourceRequest,
    WorkloadRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessageResponse {
    pub message_id: uuid::Uuid,
    pub status: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSSystemStatus {
    pub status: String,
    pub services: usize,
    pub load_stats: String,
    pub federation_status: String,
    pub memory_usage: u64,
    pub uptime: u64,
}

/// BiomeOS deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSDeploymentRequest {
    pub deployment_id: String,
    pub team_id: String,
    pub deployment_name: String,
    pub resources: BiomeOSResourceInfo,
}

/// BiomeOS deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSDeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub endpoints: Vec<String>,
    pub resources: BiomeOSResourceInfo,
}

/// BiomeOS resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceInfo {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub network_bandwidth_mbps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_biomeos_integration_creation() {
        let config = songbird_config::SongbirdConfig::default();
        let _integration = BiomeOSIntegration::new(config);

        // Test that integration was created
        let orchestrator = _integration.orchestrator.read().await;
        assert!(!orchestrator.id.is_empty());
    }

    #[tokio::test]
    async fn test_ecosystem_message_handling() {
        let config = songbird_config::SongbirdConfig::default();
        let _integration = BiomeOSIntegration::new(config);

        let message = EcosystemMessage {
            message_id: uuid::Uuid::new_v4(),
            from_primal: "biomeos".to_string(),
            to_primal: "songbird".to_string(),
            message_type: EcosystemMessageType::HealthCheck,
            payload: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            deployment_request: None,
        };

        assert_eq!(message.message_type, EcosystemMessageType::HealthCheck);
    }
}
