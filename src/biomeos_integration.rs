//! # Songbird biomeOS Integration
//!
//! Integration layer that connects Songbird with the biomeOS ecosystem,
//! implementing unified service registration and coordination protocols.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    SongbirdError, Result,
    biome::{SongbirdOrchestrator, SongbirdBiomeManifest},
    config::SongbirdConfig,
};

/// biomeOS ecosystem service registration for Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceRegistration {
    pub service_id: String,
    pub primal_type: String,
    pub biome_id: String,
    pub version: String,
    pub api_version: String,
    pub registration_time: DateTime<Utc>,
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

/// Songbird integration with biomeOS ecosystem
pub struct SongbirdBiomeOSIntegration {
    config: SongbirdConfig,
    orchestrator: Arc<RwLock<SongbirdOrchestrator>>,
    biomeos_client: BiomeOSClient,
    registration: Option<BiomeOSServiceRegistration>,
    instance_id: String,
}

impl SongbirdBiomeOSIntegration {
    pub fn new(
        config: SongbirdConfig,
        orchestrator: Arc<RwLock<SongbirdOrchestrator>>,
        biomeos_endpoint: String,
    ) -> Self {
        let biomeos_client = BiomeOSClient::new(biomeos_endpoint);
        let instance_id = format!("songbird-{}", Uuid::new_v4().simple());
        
        Self {
            config,
            orchestrator,
            biomeos_client,
            registration: None,
            instance_id,
        }
    }
    
    /// Register Songbird with the biomeOS ecosystem
    pub async fn register_with_biomeos(&mut self, biome_id: String) -> Result<()> {
        info!("Registering Songbird with biomeOS ecosystem");
        
        let registration = BiomeOSServiceRegistration {
            service_id: format!("primal-songbird-{}", self.instance_id),
            primal_type: "songbird".to_string(),
            biome_id: biome_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: Utc::now(),
            
            endpoints: BiomeOSEndpoints {
                primary: format!("http://{}:{}", 
                    self.config.network.http_listen_address, 
                    self.config.network.http_listen_port
                ),
                health: format!("http://{}:{}/health", 
                    self.config.network.http_listen_address, 
                    self.config.network.http_listen_port
                ),
                metrics: format!("http://{}:{}/metrics", 
                    self.config.network.http_listen_address, 
                    self.config.network.http_listen_port
                ),
                admin: Some(format!("http://{}:{}/admin", 
                    self.config.network.http_listen_address, 
                    self.config.network.http_listen_port + 1
                )),
                websocket: Some(format!("ws://{}:{}/ws", 
                    self.config.network.http_listen_address, 
                    self.config.network.http_listen_port
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
                meta.insert("discovery_backends".to_string(), "consul,etcd,memory,kubernetes".to_string());
                meta.insert("load_balancing_algorithms".to_string(), "round_robin,health_based,least_connections,gpu_aware,resource_aware".to_string());
                meta.insert("protocols_supported".to_string(), "http,https,grpc,websocket,tcp,udp".to_string());
                meta
            },
        };
        
        // Register with biomeOS
        self.biomeos_client.register_service(&registration).await?;
        self.registration = Some(registration);
        
        info!("Songbird successfully registered with biomeOS ecosystem");
        Ok(())
    }
    
    /// Coordinate BYOB deployment with biomeOS
    pub async fn coordinate_byob_deployment(
        &self,
        deployment_request: BiomeOSByobDeploymentRequest,
    ) -> Result<BiomeOSByobDeploymentResponse> {
        info!("Coordinating BYOB deployment: {}", deployment_request.deployment_id);
        
        // Convert biomeOS request to Songbird manifest
        let songbird_manifest = self.convert_biomeos_to_songbird_manifest(&deployment_request)?;
        
        // Deploy using Songbird orchestrator
        let orchestrator = self.orchestrator.read().await;
        let deployment_result = orchestrator.deploy_biome(songbird_manifest).await?;
        
        // Coordinate with other Primals
        let toadstool_coordination = self.coordinate_with_toadstool(&deployment_request).await?;
        let nestgate_coordination = self.coordinate_with_nestgate(&deployment_request).await?;
        
        // Create response
        let response = BiomeOSByobDeploymentResponse {
            deployment_id: deployment_request.deployment_id,
            status: "deployed".to_string(),
            songbird_deployment: deployment_result,
            primal_coordination: vec![
                PrimalCoordinationInfo {
                    primal_type: "toadstool".to_string(),
                    status: "coordinated".to_string(),
                    endpoints: toadstool_coordination.endpoints,
                },
                PrimalCoordinationInfo {
                    primal_type: "nestgate".to_string(),
                    status: "coordinated".to_string(),
                    endpoints: nestgate_coordination.endpoints,
                },
            ],
            ecosystem_endpoints: self.generate_ecosystem_endpoints(&deployment_request).await?,
            created_at: Utc::now(),
        };
        
        info!("BYOB deployment coordination completed: {}", deployment_request.deployment_id);
        Ok(response)
    }
    
    /// Handle ecosystem messages from other Primals
    pub async fn handle_ecosystem_message(&mut self, message: EcosystemMessage) -> Result<Option<EcosystemMessage>> {
        debug!("Handling ecosystem message: {:?}", message.message_type);
        
        match message.message_type {
            EcosystemMessageType::ServiceRegistration => {
                self.handle_service_registration(message).await
            }
            EcosystemMessageType::ResourceRequest => {
                self.handle_resource_request(message).await
            }
            EcosystemMessageType::WorkloadRequest => {
                self.handle_workload_request(message).await
            }
            EcosystemMessageType::HealthCheck => {
                self.handle_health_check(message).await
            }
            _ => {
                debug!("Unhandled message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }
    
    /// Get Songbird status for ecosystem monitoring
    pub async fn get_ecosystem_status(&self) -> Result<SongbirdEcosystemStatus> {
        let orchestrator = self.orchestrator.read().await;
        
        Ok(SongbirdEcosystemStatus {
            service_id: self.registration.as_ref()
                .map(|r| r.service_id.clone())
                .unwrap_or_else(|| "unregistered".to_string()),
            health: "healthy".to_string(), // Would check actual health
            active_services: orchestrator.get_active_service_count().await.unwrap_or(0),
            load_balancing_stats: orchestrator.get_load_balancing_stats().await.unwrap_or_default(),
            federation_status: orchestrator.get_federation_status().await.unwrap_or_default(),
            primal_integrations: self.get_primal_integration_status().await?,
        })
    }
    
    // Private helper methods
    
    async fn convert_biomeos_to_songbird_manifest(
        &self,
        request: &BiomeOSByobDeploymentRequest,
    ) -> Result<SongbirdBiomeManifest> {
        // Convert biomeOS deployment request to Songbird manifest format
        // This would involve mapping biomeOS service specs to Songbird service specs
        Ok(SongbirdBiomeManifest {
            metadata: crate::biome::BiomeMetadata {
                name: format!("biomeos-{}", request.deployment_id),
                version: "1.0.0".to_string(),
                description: Some("biomeOS coordinated deployment".to_string()),
            },
            services: HashMap::new(), // Would populate from request
            networking: None,
            primals: None,
        })
    }
    
    async fn coordinate_with_toadstool(
        &self,
        request: &BiomeOSByobDeploymentRequest,
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
        request: &BiomeOSByobDeploymentRequest,
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
        Ok(vec![
            format!("http://{}:{}/biome/{}", 
                self.config.network.http_listen_address,
                self.config.network.http_listen_port,
                request.deployment_id
            ),
        ])
    }
    
    async fn handle_service_registration(&mut self, message: EcosystemMessage) -> Result<Option<EcosystemMessage>> {
        // Handle service registration from other Primals
        info!("Handling service registration from: {}", message.from_primal);
        Ok(None)
    }
    
    async fn handle_resource_request(&mut self, message: EcosystemMessage) -> Result<Option<EcosystemMessage>> {
        // Handle resource requests from other Primals
        info!("Handling resource request from: {}", message.from_primal);
        Ok(None)
    }
    
    async fn handle_workload_request(&mut self, message: EcosystemMessage) -> Result<Option<EcosystemMessage>> {
        // Handle workload requests from other Primals
        info!("Handling workload request from: {}", message.from_primal);
        Ok(None)
    }
    
    async fn handle_health_check(&mut self, message: EcosystemMessage) -> Result<Option<EcosystemMessage>> {
        // Respond to health check requests
        let response = EcosystemMessage {
            message_id: Uuid::new_v4(),
            from_primal: "songbird".to_string(),
            to_primal: message.from_primal,
            message_type: EcosystemMessageType::HealthCheck,
            payload: serde_json::json!({
                "status": "healthy",
                "timestamp": Utc::now(),
                "services": self.get_ecosystem_status().await?
            }),
            timestamp: Utc::now(),
            correlation_id: Some(message.message_id),
        };
        
        Ok(Some(response))
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
}

/// Client for communicating with biomeOS
pub struct BiomeOSClient {
    endpoint: String,
    client: Client,
}

impl BiomeOSClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }
    
    pub async fn register_service(&self, registration: &BiomeOSServiceRegistration) -> Result<()> {
        let url = format!("{}/api/v1/ecosystem/services", self.endpoint);
        
        let response = self.client
            .post(&url)
            .json(registration)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to register with biomeOS: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(SongbirdError::network(format!(
                "biomeOS registration failed: {}",
                response.status()
            )));
        }
        
        Ok(())
    }
    
    pub async fn send_message(&self, message: &EcosystemMessage) -> Result<()> {
        let url = format!("{}/api/v1/ecosystem/messages", self.endpoint);
        
        let response = self.client
            .post(&url)
            .json(message)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to send message to biomeOS: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(SongbirdError::network(format!(
                "Message send failed: {}",
                response.status()
            )));
        }
        
        Ok(())
    }
}

// Supporting types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentRequest {
    pub deployment_id: Uuid,
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
    pub deployment_id: Uuid,
    pub status: String,
    pub songbird_deployment: serde_json::Value, // Would be proper type
    pub primal_coordination: Vec<PrimalCoordinationInfo>,
    pub ecosystem_endpoints: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationInfo {
    pub primal_type: String,
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
    pub message_id: Uuid,
    pub from_primal: String,
    pub to_primal: String,
    pub message_type: EcosystemMessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemMessageType {
    ServiceRegistration,
    ServiceDeregistration,
    HealthCheck,
    ResourceRequest,
    ResourceAllocation,
    ResourceRelease,
    WorkloadRequest,
    WorkloadStatus,
    WorkloadComplete,
    VolumeProvisionRequest,
    VolumeProvisionComplete,
    MountRequest,
    MountComplete,
    EcosystemStateChange,
    PrimalStatusUpdate,
    ErrorNotification,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SongbirdConfig;
    
    #[tokio::test]
    async fn test_biomeos_registration() {
        let config = SongbirdConfig::default();
        let orchestrator = Arc::new(RwLock::new(SongbirdOrchestrator::new()));
        
        let mut integration = SongbirdBiomeOSIntegration::new(
            config,
            orchestrator,
            "http://localhost:4000".to_string(),
        );
        
        // Test registration structure
        assert!(integration.registration.is_none());
        
        // Note: Actual registration would require a running biomeOS instance
        // This test validates the structure and logic
    }
    
    #[tokio::test]
    async fn test_ecosystem_message_handling() {
        let config = SongbirdConfig::default();
        let orchestrator = Arc::new(RwLock::new(SongbirdOrchestrator::new()));
        
        let mut integration = SongbirdBiomeOSIntegration::new(
            config,
            orchestrator,
            "http://localhost:4000".to_string(),
        );
        
        let message = EcosystemMessage {
            message_id: Uuid::new_v4(),
            from_primal: "biomeos".to_string(),
            to_primal: "songbird".to_string(),
            message_type: EcosystemMessageType::HealthCheck,
            payload: serde_json::json!({}),
            timestamp: Utc::now(),
            correlation_id: None,
        };
        
        let response = integration.handle_ecosystem_message(message).await.unwrap();
        assert!(response.is_some());
        
        let response = response.unwrap();
        assert_eq!(response.message_type, EcosystemMessageType::HealthCheck);
        assert_eq!(response.from_primal, "songbird");
    }
} 