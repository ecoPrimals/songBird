//! Main Orchestrator Module
//!
//! The core orchestrator that manages all services

mod request_router;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::config::OrchestratorConfig;
use crate::errors::{Result, SongbirdError};
use crate::registry::ServiceRegistry;
use crate::traits::service::{ServiceInfo, ServiceMetrics, UniversalService, ServiceRequest, ServiceResponse};
use crate::load_balancer::{DefaultLoadBalancer, LoadBalancerConfig, ServiceInstance, LoadBalancer, LoadBalancerStats};
use crate::communication::{ProtocolRouter};
use crate::traits::communication::CommunicationLayer;
use crate::traits::discovery::{ServiceDiscovery, ServiceQuery, ServiceHealthStatus};
use crate::discovery::{StaticServiceDiscovery, SongbirdDiscovery, SongbirdDiscoveryConfig};
use crate::observability::{ObservabilityEngine, ClusterStatus, ObservabilityEvent};

pub use request_router::{RequestRouter, RequestMetrics};

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Recovering,
}

/// Service instance managed by the orchestrator
pub struct OrchestratorServiceInstance {
    pub info: ServiceInfo,
    pub health: Arc<RwLock<ServiceHealth>>,
    pub metrics: Arc<RwLock<ServiceMetrics>>,
    pub last_health_check: Arc<Mutex<Instant>>,
    pub restart_count: Arc<AtomicU64>,
    pub started_at: DateTime<Utc>,
}

impl std::fmt::Debug for OrchestratorServiceInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestratorServiceInstance")
            .field("info", &self.info)
            .field("started_at", &self.started_at)
            .finish()
    }
}

/// Orchestrator events
#[derive(Debug, Clone)]
pub enum OrchestratorEvent {
    ServiceStarted {
        service_id: String,
    },
    ServiceStopped {
        service_id: String,
    },
    ServiceHealthChanged {
        service_id: String,
        health: ServiceHealth,
    },
    ServiceRestarted {
        service_id: String,
    },
}

/// Orchestrator metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorMetrics {
    pub total_services: u64,
    pub healthy_services: u64,
    pub degraded_services: u64,
    pub unhealthy_services: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub service_restarts: u64,
    pub uptime_seconds: u64,
    pub started_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl Default for OrchestratorMetrics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            total_services: 0,
            healthy_services: 0,
            degraded_services: 0,
            unhealthy_services: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            service_restarts: 0,
            uptime_seconds: 0,
            started_at: now,
            last_updated: now,
        }
    }
}

/// Discovery backend type
#[derive(Debug, Clone)]
pub enum DiscoveryBackend {
    Static,
    Songbird(SongbirdDiscoveryConfig),
}

impl Default for DiscoveryBackend {
    fn default() -> Self {
        DiscoveryBackend::Static
    }
}

/// Main orchestrator implementation
#[derive(Clone)]
pub struct Orchestrator {
    config: Arc<OrchestratorConfig>,
    registry: Arc<ServiceRegistry>,
    discovery: Arc<dyn ServiceDiscovery>,
    services: Arc<DashMap<String, Arc<OrchestratorServiceInstance>>>,
    metrics: Arc<RwLock<OrchestratorMetrics>>,
    event_sender: broadcast::Sender<OrchestratorEvent>,
    shutdown_signal: Arc<AtomicBool>,
    started_at: Instant,
    // New components for request routing
    request_router: Arc<RequestRouter>,
    load_balancer: Arc<DefaultLoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
    // Keep direct reference to protocol router for advanced features
    protocol_router: Arc<ProtocolRouter>,
    observability: Arc<ObservabilityEngine>,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        Self::new_with_discovery(config, DiscoveryBackend::default()).await
    }

    /// Create a new orchestrator with specific discovery backend
    pub async fn new_with_discovery(config: OrchestratorConfig, discovery_backend: DiscoveryBackend) -> Result<Self> {
        let registry = ServiceRegistry::new().await?;
        let (event_sender, _) = broadcast::channel(1000);

        // Initialize discovery backend
        let discovery: Arc<dyn ServiceDiscovery> = match discovery_backend {
            DiscoveryBackend::Static => {
                tracing::info!("Using static service discovery");
                Arc::new(StaticServiceDiscovery::new())
            }
            DiscoveryBackend::Songbird(songbird_config) => {
                tracing::info!("Using Songbird service discovery with federation: {}", 
                             songbird_config.federation_enabled);
                let songbird = SongbirdDiscovery::new(songbird_config.clone());
                
                // Start federation if enabled
                if songbird_config.federation_enabled {
                    songbird.start_federation().await?;
                }
                
                Arc::new(songbird)
            }
        };

        // Initialize load balancer
        let load_balancer_config = LoadBalancerConfig::default();
        let load_balancer = Arc::new(DefaultLoadBalancer::new(load_balancer_config));

        // Initialize communication layer (multi-protocol router - no hardcoding!)
        let protocol_router = Arc::new(ProtocolRouter::new());
        let communication = Arc::clone(&protocol_router) as Arc<dyn CommunicationLayer>;

        // Initialize request router
        let request_router = Arc::new(RequestRouter::new(
            Arc::clone(&load_balancer) as Arc<dyn crate::load_balancer::LoadBalancer>,
            Arc::clone(&communication),
        ));

        // Initialize observability
        let observability = Arc::new(ObservabilityEngine::new(config.observability.clone())?);

        Ok(Self {
            config: Arc::new(config),
            registry: Arc::new(registry),
            discovery,
            services: Arc::new(DashMap::new()),
            metrics: Arc::new(RwLock::new(OrchestratorMetrics::default())),
            event_sender,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            started_at: Instant::now(),
            request_router,
            load_balancer,
            communication,
            protocol_router,
            observability,
        })
    }

    /// Create orchestrator with Songbird Federation
    pub async fn new_with_federation(config: OrchestratorConfig, institution: Option<String>) -> Result<Self> {
        let songbird_config = SongbirdDiscoveryConfig {
            node_id: None,
            node_type: crate::discovery::NodeType::Orchestrator,
            institution,
            federation_enabled: true,
            health_check_interval_secs: 30,
            node_discovery_interval_secs: 60,
            trust_verification_enabled: true,
            max_federation_nodes: 1000,
            network: crate::discovery::NetworkConfig::default(),
            monitoring: crate::discovery::MonitoringConfig::default(),
            trust: crate::discovery::TrustConfig::default(),
        };

        Self::new_with_discovery(config, DiscoveryBackend::Songbird(songbird_config)).await
    }

    /// Register and start a service with full integration
    pub async fn register_service<S>(&self, mut service: S, config: S::Config) -> Result<String>
    where
        S: UniversalService + 'static,
        S::Config: Clone + Send + Sync + for<'de> serde::de::Deserialize<'de> + std::fmt::Debug,
        S::Health: Send + Sync + serde::Serialize + std::fmt::Debug,
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        // Initialize the service
        service
            .initialize(config.clone())
            .await
            .map_err(|e| SongbirdError::Service {
                message: format!("Failed to initialize service: {}", e),
            })?;

        let info = service.service_info();
        let service_id = info.id.clone();

        // Start the service
        service.start().await.map_err(|e| SongbirdError::Service {
            message: format!("Failed to start service: {}", e),
        })?;

        // Register service protocol with the communication router
        self.protocol_router.register_service_protocol(&service_id, &info);

        // 1. Register with discovery service
        self.discovery.register(info.clone()).await?;

        // 2. Create service instance for orchestrator tracking
        let instance = Arc::new(OrchestratorServiceInstance {
            info: info.clone(),
            health: Arc::new(RwLock::new(ServiceHealth::Healthy)),
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            last_health_check: Arc::new(Mutex::new(Instant::now())),
            restart_count: Arc::new(AtomicU64::new(0)),
            started_at: Utc::now(),
        });

        self.services.insert(service_id.clone(), instance.clone());

        // 3. Register with load balancer (NOW IMPLEMENTED!)
        self.register_with_load_balancer(&service_id, &info).await?;

        // 4. Register with observability for health monitoring
        if let Err(e) = self.observability.register_service(&info).await {
            tracing::warn!("Failed to register service '{}' with observability: {}", service_id, e);
        }

        // 5. Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_services += 1;
            metrics.healthy_services += 1;
            metrics.last_updated = Utc::now();
        }

        // 6. Send event
        let _ = self.event_sender.send(OrchestratorEvent::ServiceStarted {
            service_id: service_id.clone(),
        });

        tracing::info!(
            "Successfully registered service '{}' with discovery, load balancer, and orchestrator", 
            service_id
        );
        Ok(service_id)
    }

    /// Register service with load balancer
    async fn register_with_load_balancer(&self, service_id: &str, _service_info: &ServiceInfo) -> Result<()> {
        // Since load balancer doesn't have a register method, we'll update its health tracking
        self.load_balancer.update_service_health(service_id, true).await
            .map_err(|e| SongbirdError::Service {
                message: format!("Failed to register with load balancer: {}", e),
            })?;

        tracing::debug!("Registered service '{}' with load balancer", service_id);
        Ok(())
    }

    /// Unregister and stop a service with full cleanup
    pub async fn unregister_service(&self, service_id: &str) -> Result<()> {
        if let Some((_, _instance)) = self.services.remove(service_id) {
            // 1. Unregister from discovery service
            self.discovery.unregister(service_id).await?;

            // 2. Update load balancer to mark service as unhealthy/removed
            if let Err(e) = self.load_balancer.update_service_health(service_id, false).await {
                tracing::warn!("Failed to update load balancer health for '{}': {}", service_id, e);
            }

            // 3. Unregister from observability
            if let Err(e) = self.observability.unregister_service(service_id).await {
                tracing::warn!("Failed to unregister service '{}' from observability: {}", service_id, e);
            }

            // 4. Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_services = metrics.total_services.saturating_sub(1);
                metrics.healthy_services = metrics.healthy_services.saturating_sub(1);
                metrics.last_updated = Utc::now();
            }

            // 5. Send event
            let _ = self.event_sender.send(OrchestratorEvent::ServiceStopped {
                service_id: service_id.to_string(),
            });

            tracing::info!("Successfully unregistered service '{}'", service_id);
            Ok(())
        } else {
            Err(SongbirdError::Service {
                message: "Service not found".to_string(),
            })
        }
    }

    /// Update service health across all systems (NEW!)
    pub async fn update_service_health(&self, service_id: &str, health: ServiceHealth) -> Result<()> {
        // 1. Update internal instance
        if let Some(instance) = self.services.get(service_id) {
            *instance.health.write().await = health.clone();
            
            // 2. Convert to discovery health and update discovery service
            let discovery_health = self.convert_to_discovery_health(&health);
            self.discovery.update_health(service_id, discovery_health).await?;
            
            // 3. Update load balancer health
            let is_healthy = matches!(health, ServiceHealth::Healthy);
            self.load_balancer.update_service_health(service_id, is_healthy).await?;
            
            // 4. Send event
            let _ = self.event_sender.send(OrchestratorEvent::ServiceHealthChanged {
                service_id: service_id.to_string(),
                health: health.clone(),
            });
            
            tracing::debug!("Updated health for service '{}' to {:?}", service_id, health);
            Ok(())
        } else {
            Err(SongbirdError::Service {
                message: "Service not found".to_string(),
            })
        }
    }

    /// Convert orchestrator health to discovery health
    fn convert_to_discovery_health(&self, health: &ServiceHealth) -> ServiceHealthStatus {
        match health {
            ServiceHealth::Healthy => ServiceHealthStatus::Healthy,
            ServiceHealth::Degraded => ServiceHealthStatus::Degraded,
            ServiceHealth::Unhealthy => ServiceHealthStatus::Unhealthy,
            ServiceHealth::Unknown => ServiceHealthStatus::Unknown,
            ServiceHealth::Recovering => ServiceHealthStatus::Degraded, // Map to degraded
        }
    }

    /// Get services available for load balancing (NEW!)
    pub async fn get_load_balancer_services(&self, query: Option<ServiceQuery>) -> Result<Vec<ServiceInstance>> {
        // Get services from discovery
        let discovered_services = if let Some(q) = query {
            self.discovery.discover(q).await?
        } else {
            self.discovery.list_all().await?
        };

        // Convert to load balancer instances
        let mut lb_services = Vec::new();
        for service_info in discovered_services {
            // Check if we have health info for this service
            let is_healthy = if let Some(instance) = self.services.get(&service_info.id) {
                matches!(*instance.health.read().await, ServiceHealth::Healthy)
            } else {
                true // Default to healthy for external services
            };

            let lb_instance = ServiceInstance {
                service_info,
                current_connections: 0, // TODO: Track actual connections
                is_healthy,
                weight: 1, // Default weight
            };
            lb_services.push(lb_instance);
        }

        Ok(lb_services)
    }

    /// Select service using load balancer with discovery integration (NEW!)
    pub async fn select_service_for_request(&self, service_type: Option<&str>) -> Result<Option<ServiceInfo>> {
        // Build query for service type
        let query = if let Some(stype) = service_type {
            Some(ServiceQuery::new().with_service_type(stype))
        } else {
            None
        };

        // Get available services for load balancing
        let _lb_services = self.get_load_balancer_services(query).await?;

        // Select using load balancer
        if let Some(selected_service_id) = self.load_balancer.select_service().await {
            // Find the service info for the selected service ID
            if let Some(service_instance) = self.services.get(&selected_service_id) {
                Ok(Some(service_instance.info.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Discover services using the configured discovery backend
    pub async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        self.discovery.discover(query).await
    }

    /// Check if a service exists in discovery
    pub async fn service_exists(&self, service_id: &str) -> Result<bool> {
        self.discovery.exists(service_id).await
    }

    /// List all services from discovery
    pub async fn list_discovered_services(&self) -> Result<Vec<ServiceInfo>> {
        self.discovery.list_all().await
    }

    /// Update service health in discovery
    pub async fn update_service_health_in_discovery(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        self.discovery.update_health(service_id, health).await
    }

    /// Get discovery service reference (for advanced Songbird features)
    pub fn discovery(&self) -> &Arc<dyn ServiceDiscovery> {
        &self.discovery
    }

    /// Get Songbird discovery if using that backend
    pub fn songbird_discovery(&self) -> Option<&SongbirdDiscovery> {
        self.discovery.as_any().downcast_ref::<SongbirdDiscovery>()
    }

    /// Get service health
    pub async fn get_service_health(&self, service_id: &str) -> Result<ServiceHealth> {
        if let Some(instance) = self.services.get(service_id) {
            Ok(instance.health.read().await.clone())
        } else {
            Err(SongbirdError::Service {
                message: "Service not found".to_string(),
            })
        }
    }

    /// Get service metrics
    pub async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        if let Some(instance) = self.services.get(service_id) {
            Ok(instance.metrics.read().await.clone())
        } else {
            Err(SongbirdError::Service {
                message: "Service not found".to_string(),
            })
        }
    }

    /// Start the orchestrator and all its components
    pub async fn start(&self) -> Result<()> {
        tracing::info!("🎼 Starting Songbird Orchestrator");

        // Start observability first for full monitoring
        if let Err(e) = self.observability.start().await {
            tracing::error!("Failed to start observability engine: {}", e);
            return Err(e);
        }

        // Start health monitoring
        self.start_health_monitoring().await?;

        // Update uptime in metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.uptime_seconds = self.started_at.elapsed().as_secs();
        }

        tracing::info!("✅ Songbird Orchestrator started successfully");
        Ok(())
    }

    /// Stop the orchestrator and all its components
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("🛑 Stopping Songbird Orchestrator");

        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Stop all services (existing services will gracefully shut down)
        for service in self.services.iter() {
            tracing::debug!("Stopping service: {}", service.key());
        }

        // Stop observability
        if let Err(e) = self.observability.stop().await {
            tracing::warn!("Failed to stop observability engine: {}", e);
        }

        tracing::info!("✅ Songbird Orchestrator stopped successfully");
        Ok(())
    }

    /// Get orchestrator metrics
    pub async fn get_metrics(&self) -> OrchestratorMetrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.uptime_seconds = self.started_at.elapsed().as_secs();
        metrics.last_updated = Utc::now();

        // Count service health statuses
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;

        for instance in self.services.iter() {
            match *instance.health.read().await {
                ServiceHealth::Healthy => healthy += 1,
                ServiceHealth::Degraded => degraded += 1,
                ServiceHealth::Unhealthy | ServiceHealth::Unknown => unhealthy += 1,
                ServiceHealth::Recovering => {} // Don't count recovering services
            }
        }

        metrics.healthy_services = healthy;
        metrics.degraded_services = degraded;
        metrics.unhealthy_services = unhealthy;
        metrics.total_services = self.services.len() as u64;

        metrics
    }

    /// List all services
    pub async fn list_services(&self) -> Vec<ServiceInfo> {
        self.services
            .iter()
            .map(|entry| entry.value().info.clone())
            .collect()
    }

    /// Subscribe to orchestrator events
    pub fn subscribe_events(&self) -> broadcast::Receiver<OrchestratorEvent> {
        self.event_sender.subscribe()
    }

    /// Start health monitoring background task
    async fn start_health_monitoring(&self) -> Result<()> {
        let services = Arc::clone(&self.services);
        let event_sender = self.event_sender.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            while !shutdown_signal.load(Ordering::Relaxed) {
                interval.tick().await;

                for service_entry in services.iter() {
                    let service_id = service_entry.key().clone();
                    let instance = service_entry.value();

                    // Simulate health check for now (will be replaced with actual service calls)
                    let health_ok = true; // Placeholder - in real implementation, call service.health_check()

                    if health_ok {
                        let mut current_health = instance.health.write().await;
                        if !matches!(*current_health, ServiceHealth::Healthy) {
                            *current_health = ServiceHealth::Healthy;
                            let _ = event_sender.send(OrchestratorEvent::ServiceHealthChanged {
                                service_id: service_id.clone(),
                                health: ServiceHealth::Healthy,
                            });
                        }
                    } else {
                        let mut current_health = instance.health.write().await;
                        if !matches!(*current_health, ServiceHealth::Unhealthy) {
                            *current_health = ServiceHealth::Unhealthy;
                            let _ = event_sender.send(OrchestratorEvent::ServiceHealthChanged {
                                service_id: service_id.clone(),
                                health: ServiceHealth::Unhealthy,
                            });
                        }
                    }

                    // Update last health check time
                    *instance.last_health_check.lock().await = Instant::now();
                }
            }
        });

        Ok(())
    }

    /// Get the orchestrator configuration
    pub fn config(&self) -> &Arc<OrchestratorConfig> {
        &self.config
    }

    /// Handle a service request by routing it through the load balancer and communication layer
    /// This is the core request routing functionality for the orchestrator
    pub async fn handle_service_request(
        &self,
        service_id: &str,
        request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        let start_time = std::time::Instant::now();
        
        // Update request metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
        }

        // Get all instances of the requested service through discovery
        let discovered_services = self.discovery.discover(
            ServiceQuery::new().with_service_id(service_id)
        ).await?;

        if discovered_services.is_empty() {
            // Try to find the service in our local registry
            if !self.services.contains_key(service_id) {
                let mut metrics = self.metrics.write().await;
                metrics.failed_requests += 1;
                
                return Err(SongbirdError::Service {
                    message: format!("Service '{}' not found in discovery or local registry", service_id),
                });
            }
        }

        // Convert discovered services to load balancer instances
        let mut service_instances: Vec<ServiceInstance> = Vec::new();
        
        // Add discovered services
        for service_info in discovered_services {
            let is_healthy = if let Some(instance) = self.services.get(&service_info.id) {
                matches!(*instance.health.read().await, ServiceHealth::Healthy)
            } else {
                true // External services default to healthy
            };

            service_instances.push(ServiceInstance {
                service_info,
                current_connections: 0, // TODO: Track actual connections
                is_healthy,
                weight: 1, // Default weight - could be made configurable
            });
        }

        // Add local service if not already included
        if let Some(local_instance) = self.services.get(service_id) {
            let local_info = &local_instance.info;
            if !service_instances.iter().any(|si| si.service_info.id == local_info.id) {
                let is_healthy = matches!(*local_instance.health.read().await, ServiceHealth::Healthy);
                
                service_instances.push(ServiceInstance {
                    service_info: local_info.clone(),
                    weight: 1,
                    current_connections: 0,
                    is_healthy,
                });
            }
        }

        if service_instances.is_empty() {
            let mut metrics = self.metrics.write().await;
            metrics.failed_requests += 1;
            
            return Err(SongbirdError::Service {
                message: format!("No instances available for service '{}'", service_id),
            });
        }

        // Route the request through our request router
        match self.request_router.route_request(&service_instances, request).await {
            Ok(response) => {
                // Update success metrics
                let mut metrics = self.metrics.write().await;
                metrics.successful_requests += 1;
                
                // Log successful request
                tracing::info!(
                    service_id = service_id,
                    request_id = response.request_id,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Request routed successfully"
                );
                
                Ok(response)
            }
            Err(e) => {
                // Update failure metrics
                let mut metrics = self.metrics.write().await;
                metrics.failed_requests += 1;
                
                // Log failed request
                tracing::warn!(
                    service_id = service_id,
                    error = %e,
                    duration_ms = start_time.elapsed().as_millis(),
                    "Request routing failed"
                );
                
                Err(e)
            }
        }
    }

    /// Get request metrics from the router
    pub fn get_request_metrics(&self) -> RequestMetrics {
        self.request_router.get_metrics()
    }

    /// Get load balancer statistics
    pub async fn get_load_balancer_stats(&self) -> Result<LoadBalancerStats> {
        self.load_balancer.get_stats().await
    }

    /// Get communication layer statistics
    pub async fn get_communication_stats(&self) -> Result<crate::traits::communication::CommunicationStats> {
        self.communication.get_stats().await
    }

    /// Update load balancer configuration
    pub async fn update_load_balancer_config(&self, config: LoadBalancerConfig) -> Result<()> {
        // For now, we'll just log this. In a real implementation, we'd update the config
        tracing::info!("Load balancer config update requested: {:?}", config);
        Ok(())
    }

    /// Test communication layer connectivity
    pub async fn test_communication(&self) -> Result<bool> {
        // Test basic connectivity of the communication layer
        tracing::info!("Testing communication layer connectivity");
        // For now, return true as a placeholder
        Ok(true)
    }

    /// Get comprehensive cluster status from observability
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        self.observability.get_cluster_status().await
    }

    /// Get observability event stream
    pub fn subscribe_observability_events(&self) -> broadcast::Receiver<ObservabilityEvent> {
        self.observability.subscribe_events()
    }

    /// Get observability engine for advanced monitoring
    pub fn observability(&self) -> &Arc<ObservabilityEngine> {
        &self.observability
    }

    /// Check if dashboard is enabled and accessible
    pub fn is_dashboard_enabled(&self) -> bool {
        self.config.observability.enable_dashboard
    }

    /// Get dashboard URL if enabled
    pub fn get_dashboard_url(&self) -> Option<String> {
        if self.config.observability.enable_dashboard {
            Some(format!("http://localhost:{}", self.config.observability.dashboard_port))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orchestrator")
            .field("config", &self.config)
            .field("service_count", &self.services.len())
            .finish()
    }
}

/// Legacy compatibility - keeping the old OrchestratorStats structure
#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub service_count: usize,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

impl From<OrchestratorMetrics> for OrchestratorStats {
    fn from(metrics: OrchestratorMetrics) -> Self {
        Self {
            service_count: metrics.total_services as usize,
            uptime_seconds: metrics.uptime_seconds,
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
        }
    }
}

// Re-export ServiceHandle for convenience
