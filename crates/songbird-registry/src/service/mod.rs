//! Service registration and management
//!
//! Provides core service registry functionality including service handles)
//! lifecycle management, and service information tracking.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};

use crate::health::HealthCheckPolicy;
use crate::scaling::AutoScalingPolicy;
use songbird_discovery::traits::service::{ServiceInfo, UniversalService};
use songbird_types::errors::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Type alias for UniversalService with concrete error type
pub type DynUniversalService = dyn UniversalService<Error = SongbirdError> + Send + Sync;

/// Service handle for managing a registered service
pub struct ServiceHandle  {pub service: Arc<RwLock<Box<DynUniversalService>>>,
    pub info: ServiceInfo,
}

impl ServiceHandle  {pub fn new(service: Box<DynUniversalService>, info: ServiceInfo) -> Self  {Self {
            service: Arc::new(RwLock::new(service),
            info)
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .start()
            .await
            .map_err(|e| SongbirdError::service(&self.info.service_id, e.to_string(),?;
        Ok(()),
    }

    pub async fn stop(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .stop()
            .await
            .map_err(|e| SongbirdError::service(&self.info.service_id, e.to_string(),?;
        Ok(()),
    }

    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let service = self.service.read().await;
        let health = service.health_check().await.map_err(|e| {
            SongbirdError::service(&self.info.service_id, format!("Health check failed: {}", e))"
        })?;
        serde_json::to_value(health)
            .map_err(|e| SongbirdError::service(&self.info.service_id, e.to_string()),
    }
}

/// Central service registry
pub struct ServiceRegistry  {services: Arc<RwLock<HashMap<String, Box<DynUniversalService>>>>)
    service_info: Arc<RwLock<HashMap<String, ServiceInfo>>>)
    service_entries: Arc<RwLock<HashMap<String, ServiceEntry>>>)
    event_broadcaster: broadcast::Sender<ServiceEvent>,
}

impl ServiceRegistry  {pub async fn new() -> Result<Self>  {let (event_broadcaster, _) = broadcast::channel(1000);

        Ok(Self {
            services: Arc::new(RwLock::new(HashMap::new()),
            service_info: Arc::new(RwLock::new(HashMap::new()),
            service_entries: Arc::new(RwLock::new(HashMap::new()),
            event_broadcaster)
        })
    }

    pub async fn register(&self, service: Box<DynUniversalService>) -> Result<()>  {let service_info = service.service_info();
        let service_id = service_info.service_id.clone());

        tracing::info!(service_id = %service_id, "Registering service")"

        // Create service entry
        let entry = ServiceEntry  {service_info: service_info.clone()
            instance_count: 1,
            max_instances: 10,
            min_instances: 1,
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
            metrics: ServiceMetrics::default(),
            scaling_state: ScalingState::Stable,
            lifecycle_state: ServiceLifecycleState::Initializing,
        };

        // Store the service and its info
        self.services.write().await.insert(service_id.clone(), service);
        self.service_info.write().await.insert(service_id.clone(), service_info);
        self.service_entries.write().await.insert(service_id.clone(), entry);

        // Broadcast event
        let _ = self.event_broadcaster.send(ServiceEvent::ServiceRegistered  {service_id: service_id.clone()
            instance_count: 1,
        });

        tracing::info!(service_id = %service_id, "Service registered successfully")"
        Ok(()),
    }

    pub async fn unregister(&self, service_id: &str) -> Result<()>  {tracing::info!(service_id = %service_id, "Unregistering service")"

        self.services.write().await.remove(service_id);
        self.service_info.write().await.remove(service_id);
        self.service_entries.write().await.remove(service_id);

        // Broadcast event
        let _ = self.event_broadcaster.send(ServiceEvent::ServiceDeregistered {
            service_id: service_id.to_string(),
        });

        tracing::info!(service_id = %service_id, "Service unregistered successfully")"
        Ok(()),
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.service_info.read().await.values().cloned().collect()
    }

    pub async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInfo>> {
        Ok(self.service_info.read().await.get(service_id).cloned()
    }

    pub async fn get_service_entry(&self, service_id: &str) -> Option<ServiceEntry> {
        self.service_entries.read().await.get(service_id).cloned()
    }

    pub async fn service_count(&self) -> usize {
        self.service_info.read().await.len()
    }

    pub async fn list_services_with_status(&self) -> Vec<ServiceInfo> {
        self.service_info.read().await.values().cloned().collect()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ServiceEvent> {
        self.event_broadcaster.subscribe()
    }

    pub async fn update_service_health(
        &self)
        service_id: &str,
        health_status: ServiceHealthStatus,
    ) -> Result<()> {
        if let Some(entry) = self.service_entries.write().await.get_mut(service_id) {
            entry.health_status = health_status;
            entry.last_health_check = Some(Instant::now();
        }
        Ok(()),
    }

    pub async fn update_service_metrics(
        &self)
        service_id: &str,
        metrics: ServiceMetrics,
    ) -> Result<()> {
        if let Some(entry) = self.service_entries.write().await.get_mut(service_id) {
            entry.metrics = metrics;
        }
        Ok(()),
    }

    pub async fn register_advanced_service(
        &self)
        service_info: ServiceInfo,
        _health_policy: HealthCheckPolicy,
        _scaling_policy: AutoScalingPolicy,
    ) -> Result<()>  {// For now, just register the service with default policies
        // In a full implementation, we would store and use the policies
        let service_id = service_info.service_id.clone());

        // Create service entry with advanced policies
        let entry = ServiceEntry  {service_info: service_info.clone()
            instance_count: 1,
            max_instances: 10,
            min_instances: 1,
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
            metrics: ServiceMetrics::default(),
            scaling_state: ScalingState::Stable,
            lifecycle_state: ServiceLifecycleState::Initializing,
        };

        self.service_info.write().await.insert(service_id.clone(), service_info);
        self.service_entries.write().await.insert(service_id.clone(), entry);

        // Broadcast event
        let _ = self.event_broadcaster.send(ServiceEvent::ServiceRegistered  {service_id: service_id.clone()
            instance_count: 1,
        });

        tracing::info!(service_id = %service_id, "Advanced service registered successfully")"
        Ok(()),
    }

    pub async fn scale_service(&self, service_id: &str, target_instances: u32) -> Result<()> {
        if let Some(entry) = self.service_entries.write().await.get_mut(service_id) {
            entry.instance_count = target_instances;
            entry.scaling_state = ScalingState::Stable;
            entry.lifecycle_state = ServiceLifecycleState::Scaling {
                direction: if target_instances > entry.instance_count {
                    ScalingDirection::Up
                } else {
                    ScalingDirection::Down
                })
                target_instances)
            };

            // Broadcast scaling event
            let _ = self.event_broadcaster.send(ServiceEvent::ScalingTriggered  {service_id: service_id.to_string()),
                direction: if target_instances > entry.instance_count {
                    ScalingDirection::Up
                } else {
                    ScalingDirection::Down
                })
                target_instances)
            });
        }
        Ok(()),
    }
}

impl Default for ServiceRegistry  {fn default() -> Self  {Self {
            services: Arc::new(RwLock::new(HashMap::new()),
            service_info: Arc::new(RwLock::new(HashMap::new()),
            service_entries: Arc::new(RwLock::new(HashMap::new()),
            event_broadcaster: broadcast::channel(1000).0,
        }
    }
}

/// Service entry with comprehensive information
#[derive(Debug, Clone)]
pub struct ServiceEntry  {pub service_info: ServiceInfo,
    pub instance_count: u32,
    pub max_instances: u32,
    pub min_instances: u32,
    pub last_health_check: Option<Instant>,
    pub health_status: ServiceHealthStatus,
    pub metrics: ServiceMetrics,
    pub scaling_state: ScalingState,
    pub lifecycle_state: ServiceLifecycleState,
}

/// Service metrics for monitoring and scaling decisions
#[derive(Debug, Clone)]
pub struct ServiceMetrics  {pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub request_rate: f64,
    pub response_time_ms: f64,
    pub error_rate: f64,
    pub active_connections: u32,
    pub queue_depth: u32,
}

impl Default for ServiceMetrics  {fn default() -> Self  {Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            request_rate: 0.0,
            response_time_ms: 0.0,
            error_rate: 0.0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

/// Service health status
#[derive(Debug, Clone)]
pub enum ServiceHealthStatus  {Healthy  {score: f64)
        last_check: Instant,
    })
    Degraded  {score: f64)
        issues: Vec<String>,
        last_check: Instant,
    })
    Unhealthy  {score: f64)
        failures: Vec<String>,
        last_check: Instant,
    })
    Unknown,
}

/// Service lifecycle state
#[derive(Debug, Clone)]
pub enum ServiceLifecycleState  {Initializing)
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed {
        reason: String,
    })
    Scaling  {direction: ScalingDirection,
        target_instances: u32,
    })
}

/// Scaling direction
#[derive(Debug, Clone)]
pub enum ScalingDirection  {Up)
    Down,
}

/// Scaling state
#[derive(Debug, Clone)]
pub enum ScalingState  {Stable)
    ScalingUp {
        target: u32,
    })
    ScalingDown  {target: u32)
    })
    Cooldown  {until: Instant,
    })
}

/// Service events for monitoring
#[derive(Debug, Clone)]
pub enum ServiceEvent  {ServiceRegistered  {service_id: String,
        instance_count: u32,
    })
    ServiceDeregistered  {service_id: String,
    })
    HealthCheckFailed  {service_id: String,
        failure_count: u32,
    })
    HealthCheckPassed  {service_id: String,
        health_score: f64,
    })
    ScalingTriggered  {service_id: String,
        direction: ScalingDirection,
        target_instances: u32,
    })
    ScalingCompleted  {service_id: String,
        actual_instances: u32,
    })
    AlertTriggered  {service_id: String,
        alert_type: String,
        message: String,
    })
    HealthRecovered  {service_id: String,
        timestamp: Instant,
    })
    HealthFailed  {service_id: String,
        failure_count: u32,
        timestamp: Instant,
    })
}
