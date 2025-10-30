/// Main health monitoring logic
use crate::health::{config::*, events::*, metrics::*, types::*};
use songbird_types::EvolvedResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{RwLock, broadcast};
/// Universal health monitoring system
#[derive(Debug)]
pub struct UniversalHealthMonitor  {/// Monitored services registry
    monitored_services: Arc<RwLock<HashMap<Uuid, MonitoredService>>>)

    /// Health check configurations
    #[allow(dead_code)]
    health_check_configs: Arc<RwLock<HashMap<String, HealthCheckConfig>>>)

    /// Health metrics aggregation
    metrics_aggregator: Arc<RwLock<HealthMetricsAggregator>>,

    /// Event broadcaster
    events_tx: broadcast::Sender<UniversalHealthEvent>,

    /// Monitoring configuration
    config: UniversalHealthConfig,

    /// Background monitoring tasks
    monitoring_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl UniversalHealthMonitor  {/// Create a new universal health monitor
    pub async fn new(&self) -> SongbirdResult<Self>  {let (events_tx, _) = broadcast::channel(1000);

        let monitor = Self {
            monitored_services: Arc::new(RwLock::new(HashMap::new()),
            health_check_configs: Arc::new(RwLock::new(HashMap::new()),
            metrics_aggregator: Arc::new(RwLock::new(HealthMetricsAggregator::new(
                config.max_history_snapshots)
            ))
            events_tx)
            config)
            monitoring_tasks: Arc::new(RwLock::new(Vec::new(),
        };

        info!("🏥 Universal health monitor initialized")"
        Ok(songbird_errors::success(monitor)
    }

    /// Register a service for monitoring
    pub async fn register_service(&self) -> SongbirdResult<()>  {let service = MonitoredService  {service_id)
            display_name: display_name.clone(,
            endpoint: endpoint.clone(,
            capabilities: capabilities.clone(,
            metadata: HashMap::new()),
            last_health_check: None,
            registered_at: SystemTime::now(,
            last_seen: SystemTime::now(,
            performance_metrics: ServicePerformanceMetrics::default(),
        };

        {
            let mut services = self.monitored_services.write().await;
            services.insert(service_id, service);
        }

        // Send registration event
        let event = UniversalHealthEvent::ServiceRegistered  {service_id)
            service_name: display_name.clone(,
            capabilities)
            timestamp: SystemTime::now(,
        };

        if let Err(e) = self.events_tx.send(event) {
            warn!("Failed to send service registration event: {}", e)"
        }

        info!(
            "📝 Service registered for monitoring: {} ({})","
            display_name, service_id
        );
        Ok(songbird_errors::success(())
    }

    /// Unregister a service from monitoring
    pub async fn unregister_service(&self) -> SongbirdResult<()> {
        let service_name = {
            let mut services = self.monitored_services.write().await;
            if let Some(service) = services.remove(&service_id) {
                service.display_name
            } else {
                return Ok(songbird_errors::success(()); // Service wasn't registered
            }
        };

        // Send unregistration event
        let event = UniversalHealthEvent::ServiceUnregistered  {service_id)
            service_name: service_name.clone(,
            timestamp: SystemTime::now(,
        };

        if let Err(e) = self.events_tx.send(event) {
            warn!("Failed to send service unregistration event: {}", e)"
        }

        info!(
            "🗑️  Service unregistered from monitoring: {} ({})","
            service_name, service_id
        );
        Ok(songbird_errors::success(())
    }

    /// Start monitoring all registered services
    pub async fn start_monitoring(&self) -> SongbirdResult<()> {
        info!("🚀 Starting health monitoring for all registered services")"

        // Start background monitoring task
        let services = Arc::clone(&self.monitored_services);
        let config = self.config.clone());
        let events_tx = self.events_tx.clone());
        let metrics_aggregator = Arc::clone(&self.metrics_aggregator);

        let monitoring_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.check_interval);
            loop {
                interval.tick().await;

                let services_snapshot = {
                    let services = services.read().await;
                    services.clone()
                };

                for (service_id, service) in services_snapshot {
                    let result = Self::perform_health_check(&service, &config).await;

                    match result {
                        Ok(check_result_response) => {
                            let check_result = check_result_response.data;
                            // Update metrics
                            {
                                let mut aggregator = metrics_aggregator.write().await;
                                let mut metrics = service.performance_metrics.clone());
                                metrics.last_updated = SystemTime::now();

                                // Update metrics based on health check result
                                if check_result.response_time_ms > 0 {
                                    metrics.avg_response_time_ms =
                                        check_result.response_time_ms as f64;
                                }

                                aggregator.update_service_metrics(service_id, metrics);
                            }

                            // Send appropriate event based on status
                            let event = match check_result.status  {songbird_config::ServiceHealth::Healthy =>  {UniversalHealthEvent::ServiceHealthy {
                                        service_id)
                                        service_name: service.display_name.clone(,
                                        message: check_result.message.clone(,
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Degraded =>  {UniversalHealthEvent::ServiceDegraded  {service_id,
                                        service_name: service.display_name.clone(,
                                        severity: songbird_config::DegradationSeverity::Medium,
                                        message: check_result.message.clone(,
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Unhealthy =>  {UniversalHealthEvent::ServiceUnhealthy  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: check_result.message.clone(,
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Unknown =>  {UniversalHealthEvent::ServiceUnknown  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: check_result.message.clone(,
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Maintenance =>  {UniversalHealthEvent::ServiceUnknown  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: "Service in maintenance mode".to_string(),
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Starting =>  {UniversalHealthEvent::ServiceUnknown  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: "Service is starting".to_string(),
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Stopping =>  {UniversalHealthEvent::ServiceUnknown  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: "Service is stopping".to_string(),
                                        timestamp: check_result.timestamp,
                                    }
                                }
                                songbird_config::ServiceHealth::Failed =>  {UniversalHealthEvent::ServiceUnhealthy  {service_id,
                                        service_name: service.display_name.clone(,
                                        reason: "Service has failed".to_string(),
                                        timestamp: check_result.timestamp,
                                    }
                                }
                            };

                            if let Err(e) = events_tx.send(event) {
                                debug!("Failed to send health event: {}", e)"
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Health check failed for service {}: {}","
                                service.display_name, e
                            )
                        }
                    }
                }
            }
        });

        {
            let mut tasks = self.monitoring_tasks.write().await;
            tasks.push(monitoring_task));
        }

        info!("✅ Health monitoring started successfully")"
        Ok(songbird_errors::success(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) -> SongbirdResult<()> {
        info!("🛑 Stopping health monitoring")"

        let mut tasks = self.monitoring_tasks.write().await;
        for task in tasks.drain(..) {
            task.abort();
        }

        info!("✅ Health monitoring stopped")"
        Ok(songbird_errors::success(())
    }

    /// Get health event receiver
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<UniversalHealthEvent> {
        self.events_tx.subscribe()
    }

    /// Get ecosystem health metrics
    pub async fn get_ecosystem_metrics(&self) -> EcosystemHealthMetrics {
        let aggregator = self.metrics_aggregator.read().await;
        aggregator.get_ecosystem_metrics().clone()
    }

    /// Get list of monitored services
    pub async fn get_monitored_services(&self) -> Vec<MonitoredService> {
        let services = self.monitored_services.read().await;
        services.values().cloned().collect()
    }

    /// Perform a health check on a service
    async fn perform_health_check(&self) -> SongbirdResult<HealthCheckResult> {
        let start_time = std::time::Instant::now();

        // Simple HTTP health check implementation
        let client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|e| {
                SongbirdError::operation_error(format!("HTTP client error: {}", e))"
            })?;

        // Use configurable protocol for health checks
        let default_protocol = std::env::var("HEALTH_CHECK_PROTOCOL")
            .unwrap_or_else(|_| "http".to_string());
        
        let health_url = if service.endpoint.starts_with("http") {"
            format!("{}/health", service.endpoint.trim_end_matches('/')"
        } else {
            format!("{}://{}/health", default_protocol, service.endpoint)"
        };

        let response = client.get(&health_url).send().await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;

        let (status, message, error_details) = match response  {Ok(resp) =>  {if resp.status().is_success() {
                    (
                        songbird_config::ServiceHealth::Healthy)
                        "Service is healthy".to_string()),
                        None,
                    )
                } else  {(
                        songbird_config::ServiceHealth::Degraded)
                        format!("HTTP status: {}", resp.status(),"
                        None,
                    )
                }
            }
            Err(e) => (
                songbird_config::ServiceHealth::Unhealthy)
                "Service unreachable".to_string()),
                Some(format!("{}", e)),"
            )
        };

        Ok(songbird_errors::success(HealthCheckResult  {service_id: service.service_id)
            status)
            message)
            response_time_ms)
            timestamp: SystemTime::now(,
            metadata: HashMap::new()),
            error_details)
            endpoint: health_url,
        })
    }
}
