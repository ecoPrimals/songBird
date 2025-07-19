//! Main BiomeOS integration logic

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::client::BiomeOSClient;
use super::registration::ServiceRegistrationManager;
use super::types::*;
use crate::biome::{BiomeMetadata, OrchestratorConfig, OrchestratorStatus, SongbirdOrchestrator};
use crate::primal_integration::{DiscoveredPrimal, PrimalIntegrationManager};
use songbird_config::SongbirdConfig;
use songbird_errors::{Result, ServiceError};
use songbird_universal::PrimalType;

/// BiomeOS integration for Songbird orchestrator
pub struct BiomeOSIntegration {
    config: SongbirdConfig,
    orchestrator: Arc<RwLock<SongbirdOrchestrator>>,
    instance_id: String,
    biomeos_client: BiomeOSClient,
    registration_manager: ServiceRegistrationManager,
    primal_integration: Option<PrimalIntegrationManager>,
}

impl BiomeOSIntegration {
    /// Create new BiomeOS integration
    pub fn new(config: SongbirdConfig) -> Self {
        let instance_id = format!("songbird-{}", uuid::Uuid::new_v4().simple());

        let orchestrator = Arc::new(RwLock::new(SongbirdOrchestrator {
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
        }));

        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS")
            .unwrap_or_else(|_| songbird_config::constants::default_bind_address().to_string());

        let biomeos_client = BiomeOSClient::new(format!("http://{bind_address}:4000"));
        let registration_manager = ServiceRegistrationManager::new(biomeos_client.clone());

        let primal_integration = if cfg!(test) {
            None
        } else {
            let biomeos_endpoint = std::env::var("BIOMEOS_ENDPOINT")
                .unwrap_or_else(|_| format!("http://{bind_address}:4000"));
            Some(PrimalIntegrationManager::new(biomeos_endpoint))
        };

        Self {
            config,
            orchestrator,
            instance_id,
            biomeos_client,
            registration_manager,
            primal_integration,
        }
    }

    /// Enable primal integration for testing
    pub fn enable_primal_integration_for_test(&mut self, endpoint: String) {
        self.primal_integration = Some(PrimalIntegrationManager::new(endpoint));
    }

    /// Initialize BiomeOS integration
    pub async fn initialize(&mut self) -> Result<()> {
        info!(
            "Initializing BiomeOS integration for instance {}",
            self.instance_id
        );

        // Update orchestrator status
        {
            let mut orchestrator = self.orchestrator.write().await;
            orchestrator.status = OrchestratorStatus::Starting;
        }

        // Test BiomeOS connection
        let connectivity = self.biomeos_client.test_connection().await;
        if !connectivity.is_connected() {
            warn!("BiomeOS not available, running in standalone mode");
            return self.initialize_standalone_mode().await;
        }

        // Register with BiomeOS
        self.register_with_biomeos().await?;

        // Initialize primal integration if available
        if let Some(ref mut primal_integration) = self.primal_integration {
            match primal_integration.initialize().await {
                Ok(_) => info!("Primal integration initialized successfully"),
                Err(e) => warn!("Failed to initialize primal integration: {}", e),
            }
        }

        // Update orchestrator status
        {
            let mut orchestrator = self.orchestrator.write().await;
            orchestrator.status = OrchestratorStatus::Running;
        }

        info!("BiomeOS integration initialized successfully");
        Ok(())
    }

    /// Initialize in standalone mode when BiomeOS is not available
    async fn initialize_standalone_mode(&mut self) -> Result<()> {
        info!("Initializing in standalone mode");

        {
            let mut orchestrator = self.orchestrator.write().await;
            orchestrator.status = OrchestratorStatus::Running;
        }

        info!("Standalone mode initialized successfully");
        Ok(())
    }

    /// Register with BiomeOS
    async fn register_with_biomeos(&mut self) -> Result<()> {
        info!("Registering Songbird orchestrator with BiomeOS");

        let registration = self.create_service_registration().await?;
        self.registration_manager
            .register_service(registration)
            .await?;

        info!("Successfully registered with BiomeOS");
        Ok(())
    }

    /// Create service registration for BiomeOS
    async fn create_service_registration(&self) -> Result<BiomeOSServiceRegistration> {
        let orchestrator = self.orchestrator.read().await;
        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS")
            .unwrap_or_else(|_| songbird_config::constants::default_bind_address().to_string());

        let base_port = 8080; // Default server port

        Ok(BiomeOSServiceRegistration {
            service_id: self.instance_id.clone(),
            service_name: "songbird-orchestrator".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            endpoints: BiomeOSEndpoints {
                main: format!("http://{}:{}", bind_address, base_port),
                health: format!("http://{}:{}/health", bind_address, base_port),
                metrics: Some(format!("http://{}:{}/metrics", bind_address, base_port)),
                management: Some(format!("http://{}:{}/admin", bind_address, base_port)),
                additional: HashMap::new(),
            },
            capabilities: BiomeOSCapabilities::default(),
            security: BiomeOSSecurity {
                authentication_required: false,
                supported_auth_methods: vec!["bearer".to_string()],
                tls_enabled: false,
                certificate_info: None,
            },
            resource_requirements: BiomeOSResourceRequirements {
                cpu_cores: Some(1.0),
                memory_mb: Some(512),
                storage_gb: Some(10),
                network_bandwidth_mbps: Some(100),
            },
            health_check: BiomeOSHealthCheckConfig::default(),
            metadata: HashMap::from([
                ("orchestrator_id".to_string(), orchestrator.id.clone()),
                ("version".to_string(), env!("CARGO_PKG_VERSION").to_string()),
            ]),
            tags: vec![
                "orchestrator".to_string(),
                "songbird".to_string(),
                "biomeos-integrated".to_string(),
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Start BiomeOS integration
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting BiomeOS integration");

        self.initialize().await?;
        self.start_background_tasks().await?;

        info!("BiomeOS integration started successfully");
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        // Start heartbeat task
        self.start_heartbeat_task().await;

        // Start primal coordination task
        if self.primal_integration.is_some() {
            self.start_primal_coordination_task().await;
        }

        // Start status sync task
        self.start_status_sync_task().await;

        Ok(())
    }

    /// Start heartbeat task
    async fn start_heartbeat_task(&self) {
        let client = self.biomeos_client.clone();
        let service_id = self.instance_id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                if let Err(e) = client.send_heartbeat(&service_id).await {
                    debug!("Heartbeat failed: {}", e);
                }
            }
        });

        debug!("Heartbeat task started");
    }

    /// Start primal coordination task
    async fn start_primal_coordination_task(&self) {
        debug!("Starting primal coordination task");

        // Implementation would coordinate with discovered primals
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                debug!("Performing primal coordination check");
                // Coordinate with primals here
            }
        });
    }

    /// Start status synchronization task
    async fn start_status_sync_task(&self) {
        let client = self.biomeos_client.clone();
        let orchestrator = self.orchestrator.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(120));

            loop {
                interval.tick().await;

                // Send ecosystem status to BiomeOS
                let status = {
                    let orch = orchestrator.read().await;
                    SongbirdEcosystemStatus {
                        total_orchestrators: 1,
                        active_primals: 0, // Would be updated from primal integration
                        total_services: orch.endpoints.len() as u32,
                        health_score: 1.0, // Would be calculated from health checks
                        uptime_seconds: orch
                            .created_at
                            .signed_duration_since(chrono::Utc::now())
                            .num_seconds() as u64,
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        biomeos_connected: true,
                        last_update: chrono::Utc::now(),
                    }
                };

                let message = EcosystemMessage::new(
                    EcosystemMessageType::StatusUpdate,
                    "songbird-orchestrator".to_string(),
                    serde_json::to_value(&status).unwrap_or_default(),
                );

                if let Err(e) = client.send_ecosystem_message(&message).await {
                    debug!("Failed to send status update: {}", e);
                }
            }
        });

        debug!("Status sync task started");
    }

    /// Stop BiomeOS integration
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping BiomeOS integration");

        // Update orchestrator status
        {
            let mut orchestrator = self.orchestrator.write().await;
            orchestrator.status = OrchestratorStatus::Stopping;
        }

        // Deregister from BiomeOS
        if let Err(e) = self
            .registration_manager
            .deregister_service(&self.instance_id)
            .await
        {
            warn!("Failed to deregister from BiomeOS: {}", e);
        }

        // Stop primal integration
        if let Some(ref mut primal_integration) = self.primal_integration {
            if let Err(e) = primal_integration.stop().await {
                warn!("Failed to stop primal integration: {}", e);
            }
        }

        info!("BiomeOS integration stopped successfully");
        Ok(())
    }

    /// Get BiomeOS connectivity status
    pub async fn get_connectivity_status(&self) -> BiomeOSConnectivityStatus {
        self.biomeos_client.test_connection().await
    }

    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> Result<SongbirdEcosystemStatus> {
        let orchestrator = self.orchestrator.read().await;

        Ok(SongbirdEcosystemStatus {
            total_orchestrators: 1,
            active_primals: if let Some(ref primal) = self.primal_integration {
                primal.get_active_primal_count().await as u32
            } else {
                0
            },
            total_services: orchestrator.endpoints.len() as u32,
            health_score: 1.0, // Simplified for now
            uptime_seconds: orchestrator
                .created_at
                .signed_duration_since(chrono::Utc::now())
                .num_seconds() as u64,
            version: env!("CARGO_PKG_VERSION").to_string(),
            biomeos_connected: self.get_connectivity_status().await.is_connected(),
            last_update: chrono::Utc::now(),
        })
    }

    /// Deploy BYOB service
    pub async fn deploy_byob_service(
        &self,
        request: BiomeOSByobDeploymentRequest,
    ) -> Result<BiomeOSByobDeploymentResponse> {
        info!("Deploying BYOB service: {}", request.service_name);
        self.biomeos_client.deploy_byob_service(&request).await
    }

    /// Get orchestrator instance
    pub async fn get_orchestrator(&self) -> Arc<RwLock<SongbirdOrchestrator>> {
        self.orchestrator.clone()
    }

    /// Get instance ID
    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get BiomeOS client
    pub fn get_client(&self) -> &BiomeOSClient {
        &self.biomeos_client
    }
}
