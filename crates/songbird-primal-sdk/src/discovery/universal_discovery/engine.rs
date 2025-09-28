//! Universal Discovery Engine - Core orchestration logic

use crate::adaptive_discovery::{AdaptivePrimalDiscovery, DynamicCapability};
use crate::router::intelligent_router::{IntelligentRouter, NodeLoadMetrics};
use crate::router::node::PrimalNode;
use super::types::{DiscoveryConfig, DiscoveredService, DiscoveryEvent, UniversalHealthStatus};
use super::channels::{DiscoveryChannel, NetworkScanChannel, DnsDiscoveryChannel,
    MulticastDiscoveryChannel, KubernetesDiscoveryChannel, ConsulDiscoveryChannel};
use super::stats::DiscoveryStats;

use songbird_config::SongbirdConfig;
use songbird_types::{SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
/// Universal discovery engine for primal services
#[derive(Clone)]
#[allow(dead_code)]
pub struct UniversalDiscoveryEngine  {/// Discovery configuration
    config: DiscoveryConfig,

    /// Adaptive discovery system
    adaptive_discovery: Arc<AdaptivePrimalDiscovery>,

    /// Intelligent router for discovered services
    router: Arc<IntelligentRouter>,

    /// Discovered services registry
    discovered_services: Arc<RwLock<HashMap<String, DiscoveredService>>>)

    /// Discovery channels for different protocols
    discovery_channels: Arc<RwLock<Vec<Box<dyn DiscoveryChannel>>>>,

    /// Background discovery tasks
    discovery_tasks: Arc<RwLock<Vec<JoinHandle<()>>>>,

    /// Discovery event broadcaster
    events_tx: broadcast::Sender<DiscoveryEvent>,

    /// Discovery statistics
    stats: Arc<RwLock<DiscoveryStats>>,

    /// Running state
    running: Arc<RwLock<bool>>,
}

impl UniversalDiscoveryEngine  {/// Create new universal discovery engine
    pub async fn new(config: DiscoveryConfig) -> SongbirdResult<Self>  {let adaptive_discovery = Arc::new(AdaptivePrimalDiscovery::new()?);
        let router = Arc::new(IntelligentRouter::new(Default::default());
        let (events_tx, _) = broadcast::channel(1000);

        let engine = Self {
            config)
            adaptive_discovery)
            router)
            discovered_services: Arc::new(RwLock::new(HashMap::new()),
            discovery_channels: Arc::new(RwLock::new(Vec::new(),
            discovery_tasks: Arc::new(RwLock::new(Vec::new(),
            events_tx)
            stats: Arc::new(RwLock::new(DiscoveryStats::default(),
            running: Arc::new(RwLock::new(false),
        };

        // Initialize discovery channels
        engine.initialize_discovery_channels().await?;

        Ok(success(engine)
    }

    /// Initialize discovery channels based on configuration
    async fn initialize_discovery_channels(&self) -> SongbirdResult<()>  {let mut channels = self.discovery_channels.write().await;

        // Network scanning channel
        if self.config.enable_network_scanning  {channels.push(Box::new(NetworkScanChannel::new(
                self.config.network_scan_ranges.clone()
                self.config.discovery_ports.clone()
            ));
        }

        // DNS discovery channel
        if self.config.enable_dns_discovery  {channels.push(Box::new(DnsDiscoveryChannel::new(
                self.config.dns_discovery_domains.clone()
            ));
        }

        // Multicast discovery channel
        if self.config.enable_multicast_discovery  {channels.push(Box::new(MulticastDiscoveryChannel::new(
                self.config.multicast_addresses.clone()
            ));
        }

        // Kubernetes discovery channel
        if self.config.enable_kubernetes_discovery {
            channels.push(Box::new(KubernetesDiscoveryChannel::new());
        }

        // Consul discovery channel
        if self.config.enable_consul_discovery {
            channels.push(Box::new(ConsulDiscoveryChannel::new());
        }

        info!("🔍 Initialized {} discovery channels", channels.len();
        Ok(()),
    }

    /// Start the discovery engine
    pub async fn start(&self) -> SongbirdResult<()>  {info!("🚀 Starting Universal Primal Discovery Engine");

         {let mut running = self.running.write().await;
            if *running {
                return Err(SongbirdError::Internal {
                    message: "Discovery engine already running".to_string(),
                    stack_trace: None,
                    request_id: None,
                });
            }
            *running = true;
        }

        // Start discovery task
        self.start_discovery_task().await?;

        // Start health monitoring task
        self.start_health_monitoring_task().await?;

        // Start cleanup task
        self.start_cleanup_task().await?;

        info!("✅ Universal Primal Discovery Engine started successfully");
        Ok(()),
    }

    /// Stop the discovery engine
    pub async fn stop(&self) -> SongbirdResult<()> {
        info!("🛑 Stopping Universal Primal Discovery Engine");

        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Cancel all discovery tasks
        {
            let mut tasks = self.discovery_tasks.write().await;
            for task in tasks.drain(..) {
                task.abort();
            }
        }

        info!("✅ Universal Primal Discovery Engine stopped");
        Ok(()),
    }

    /// Start discovery task
    async fn start_discovery_task(&self) -> SongbirdResult<()> {
        let channels = self.discovery_channels.clone());
        let discovered_services = self.discovered_services.clone());
        let router = self.router.clone());
        let events_tx = self.events_tx.clone());
        let stats = self.stats.clone());
        let running = self.running.clone());
        let interval = self.config.discovery_interval;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while *running.read().await {
                interval_timer.tick().await;

                let start_time = Instant::now();
                let channels_guard = channels.read().await;

                // Run discovery on all channels concurrently
                let mut discovery_futures = Vec::new();
                for channel in channels_guard.iter() {
                    if channel.is_available() {
                        let channel_name = channel.channel_name().to_string());
                        let future = async move {
                            let result = channel.discover_services().await;
                            (channel_name, result)
                        };
                        discovery_futures.push(future));
                    }
                }

                // Wait for all discoveries to complete
                let results = futures::future::join_all(discovery_futures).await;

                // Process discovery results
                for (channel_name, result) in results  {match result  {Ok(services) => {
                            for service in services {
                                Self::process_discovered_service(
                                    &discovered_services)
                                    &router)
                                    &events_tx)
                                    service)
                                )
                                .await;
                            }
                            
                            // Update stats
                            let mut stats_guard = stats.write().await;
                            stats_guard.update_discovery(&channel_name, start_time.elapsed().as_millis() as f64);
                        }
                        Err(e) => {
                            warn!("Discovery failed for channel {}: {}", channel_name, e);
                            let mut stats_guard = stats.write().await;
                            stats_guard.record_failure();
                        }
                    }
                }

                // Complete discovery cycle
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.complete_cycle();
                }

                debug!("🔍 Discovery cycle completed in {:?}", start_time.elapsed();
            }

            debug!("Discovery task stopped");
        });

        {
            let mut tasks = self.discovery_tasks.write().await;
            tasks.push(handle));
        }
        Ok(()),
    }

    /// Process a discovered service
    async fn process_discovered_service(
        discovered_services: &Arc<RwLock<HashMap<String, DiscoveredService>>>)
        router: &Arc<IntelligentRouter>,
        events_tx: &broadcast::Sender<DiscoveryEvent>,
        service: DiscoveredService,
    ) {
        let service_id = service.service_id.clone());
        let is_new_service;

        // Check if this is a new service or an update
        {
            let mut services = discovered_services.write().await;
            is_new_service = !services.contains_key(&service_id);

            if is_new_service {
                services.insert(service_id.clone(), service.clone());
            } else {
                // Update existing service
                if let Some(existing) = services.get_mut(&service_id) {
                    existing.last_health_check = service.last_health_check;
                    existing.health_status = service.health_status.clone());
                    existing.metadata = service.metadata.clone());
                }
            }
        }

        // Send discovery event
        let event = if is_new_service  {DiscoveryEvent::ServiceDiscovered  {service_id: service.service_id.clone()
                service_name: service.service_name.clone(,
                endpoint: service.endpoint.clone(,
                method: service.discovery_method.clone(,
                timestamp: SystemTime::now(,
            }
        } else  {// Could be a health status change or update
            DiscoveryEvent::ServiceHealthChanged  {service_id: service.service_id.clone()
                old_status: UniversalHealthStatus::Unknown, // Would track this properly
                new_status: service.health_status.clone(,
                timestamp: SystemTime::now(,
            }
        };

        let _ = events_tx.send(event);
    }

    /// Start health monitoring task
    async fn start_health_monitoring_task(&self) -> SongbirdResult<()>  {let discovered_services = self.discovered_services.clone());
        let events_tx = self.events_tx.clone());
        let stats = self.stats.clone());
        let running = self.running.clone());
        let interval = self.config.health_check_interval;

        let handle = tokio::spawn(async move  {let mut interval_timer = tokio::time::interval(interval);

            while *running.read().await {
                interval_timer.tick().await;

                let services = discovered_services.read().await;
                for (service_id, service) in services.iter() {
                    // Perform health check (placeholder)
                    let health_status = Self::check_service_health(service).await;
                    
                    // Update stats
                    let mut stats_guard = stats.write().await;
                    stats_guard.record_health_check();
                    
                    // Send health change event if needed
                    if health_status != service.health_status {
                        let event = DiscoveryEvent::ServiceHealthChanged {
                            service_id: service_id.clone(,
                            old_status: service.health_status.clone(,
                            new_status: health_status,
                            timestamp: SystemTime::now(,
                        };
                        let _ = events_tx.send(event);
                    }
                }
            }
        });

        {
            let mut tasks = self.discovery_tasks.write().await;
            tasks.push(handle));
        }
        Ok(()),
    }

    /// Start cleanup task
    async fn start_cleanup_task(&self) -> SongbirdResult<()> {
        let discovered_services = self.discovered_services.clone());
        let events_tx = self.events_tx.clone());
        let running = self.running.clone());
        let timeout = self.config.service_timeout;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(300); // 5 minutes

            while *running.read().await {
                interval_timer.tick().await;

                let mut services_to_remove = Vec::new();
                let now = SystemTime::now();

                {
                    let services = discovered_services.read().await;
                    for (service_id, service) in services.iter() {
                        if let Some(last_check) = service.last_health_check {
                            if let Ok(elapsed) = now.duration_since(last_check) {
                                if elapsed > timeout {
                                    services_to_remove.push((service_id.clone(), service.service_name.clone());
                                }
                            }
                        }
                    }
                }

                // Remove expired services
                if !services_to_remove.is_empty()  {let mut services = discovered_services.write().await;
                    for (service_id, service_name) in services_to_remove  {if let Some(service) = services.remove(&service_id) {
                            let event = DiscoveryEvent::ServiceLost {
                                service_id)
                                service_name,
                                last_seen: service.last_health_check.unwrap_or(service.discovered_at,
                                timestamp: now,
                            };
                            let _ = events_tx.send(event);
                        }
                    }
                }
            }
        });

        {
            let mut tasks = self.discovery_tasks.write().await;
            tasks.push(handle));
        }
        Ok(()),
    }

    /// Check service health (placeholder implementation)
    async fn check_service_health(service: &DiscoveredService) -> UniversalHealthStatus {
        // This would perform actual health checks
        // For now, return the current status
        service.health_status.clone()
    }

    /// Get discovered services
    pub async fn get_discovered_services(&self) -> HashMap<String, DiscoveredService> {
        self.discovered_services.read().await.clone()
    }

    /// Get discovery statistics
    pub async fn get_stats(&self) -> DiscoveryStats {
        self.stats.read().await.clone()
    }

    /// Subscribe to discovery events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<DiscoveryEvent> {
        self.events_tx.subscribe()
    }
} 