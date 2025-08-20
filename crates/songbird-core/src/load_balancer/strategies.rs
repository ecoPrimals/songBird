use crate::traits::load_balancer::{LoadBalancer, LoadBalancerStats, ServiceStats};
/// Load Balancer Strategies Module
///
/// Contains implementations of different load balancing algorithms
use songbird_errors::{SongbirdError, SongbirdResponse, SongbirdResult, success};
// use songbird_universal::  // TEMPORARILY DISABLED - {ServiceInfo, UniversalRequest as ServiceRequest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

/// Round-robin load balancer
#[derive(Debug)]
pub struct RoundRobinLoadBalancer {
    instances: Arc<RwLock<Vec<ServiceInfo>>>,
    current_index: Arc<RwLock<usize>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for RoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl RoundRobinLoadBalancer {
    /// Create a new round robin load balancer
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }

    /// Add an instance to the load balancer
    pub async fn add_instance(&self, instance: ServiceInfo) {
        let mut instances = self.instances.write().await;
        instances.push(instance);
    }

    /// Remove an instance from the load balancer
    pub async fn remove_instance(&self, instance_id: &str) {
        let mut instances = self.instances.write().await;
        instances.retain(|instance| instance.name != instance_id);

        // Reset index if it's out of bounds
        let mut index = self.current_index.write().await;
        if *index >= instances.len() && !instances.is_empty() {
            *index = 0;
        }
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {
    fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = SongbirdResult<ServiceInfo>> + Send + '_>>
    {
        let services = services.to_vec();
        Box::pin(async move {
            if services.is_empty() {
                return Err(SongbirdError::internal_error(Service {
                    service: "load_balancer".to_string(),
                    message: "No services available".to_string(),
                    suggested_alternatives: vec!["Register at least one service".to_string()],
                    recovery_actions: vec!["Check service registry".to_string()],
                });
            }

            // Filter for healthy services
            let healthy_services: Vec<_> = services
                .data
                .iter()
                .filter(|service| {
                    matches!(
                        service.health,
                        songbird_config::UniversalHealthStatus::Healthy
                    )
                })
                .collect();

            if healthy_services.is_empty() {
                return Err(SongbirdError::internal_error(Service {
                    service: "load_balancer".to_string(),
                    message: "No healthy services available".to_string(),
                    suggested_alternatives: vec!["Wait for services to become healthy".to_string()],
                    recovery_actions: vec!["Check service health status".to_string()],
                });
            }

            // Round-robin selection
            let mut current_index = self.current_index.write().await;
            let selected_index = *current_index % healthy_services.len();
            *current_index = (*current_index + 1) % healthy_services.len();

            Ok(songbird_errors::evolved_success(songbird_errors::success(
                healthy_services[selected_index].clone()),
            ))
        })
    }

    fn update_service_health(
        &mut self,
        service_id: &str,
        is_healthy: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = SongbirdResult<()>> + Send + '_>> {
        let service_id = service_id.to_string();
        Box::pin(async move {
            tracing::info!("Service {} health updated: {}", service_id, is_healthy);
            Ok(songbird_errors::success(()))
        })
    }

    fn get_stats(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = SongbirdResult<LoadBalancerStats>> + Send + '_>,
    > {
        Box::pin(async move {
            let stats = self.stats.read().await;
            Ok(songbird_errors::success(stats.clone()))
        })
    }

    fn reset_stats(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<SongbirdResponse<()>, SongbirdError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let mut stats = self.stats.write().await;
            stats.total_requests = 0;
            stats.successful_requests = 0;
            stats.failed_requests = 0;
            stats.average_response_time = 0.0;
            stats.active_connections = 0;
            stats.service_stats.clear();
            Ok(songbird_errors::success(()))
        })
    }

    fn record_request(
        &self,
        service_id: &str,
        success: bool,
        response_time: f64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let service_id = service_id.to_string();
        Box::pin(async move {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;

            if success {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }

            // Update service-specific stats
            let service_stats = stats
                .service_stats
                .entry(service_id)
                .or_insert(ServiceStats {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    average_response_time: 0.0,
                    active_connections: 0,
                    weight: 1.0,
                });

            service_stats.requests += 1;
            if success {
                service_stats.successes += 1;
            } else {
                service_stats.failures += 1;
            }

            // Update average response time using running average
            service_stats.average_response_time = (service_stats.average_response_time
                * (service_stats.requests - 1) as f64
                + response_time)
                / service_stats.requests as f64;

            // Update health score based on success rate
        })
    }
}

/// Least connections load balancer
#[derive(Debug)]
pub struct LeastConnectionsLoadBalancer {
    connection_counts: Arc<RwLock<HashMap<String, u32>>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for LeastConnectionsLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LeastConnectionsLoadBalancer {
    /// Create a new least connections load balancer
    pub fn new() -> Self {
        Self {
            connection_counts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }

    /// Increment connection count for an instance
    pub async fn increment_connections(&self, instance_id: &str) {
        let mut counts = self.connection_counts.write().await;
        *counts.entry(instance_id.to_string()).or_insert(0) += 1;
    }

    /// Decrement connection count for an instance
    pub async fn decrement_connections(&self, instance_id: &str) {
        let mut counts = self.connection_counts.write().await;
        if let Some(count) = counts.get_mut(instance_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

impl LoadBalancer for LeastConnectionsLoadBalancer {
    fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> impl std::future::Future<Output = SongbirdResult<ServiceInfo>> + Send {
        let services = services.to_vec();
        async move {
            if services.is_empty() {
                return Err(SongbirdError::internal_error(Service {
                    service: "load_balancer".to_string(),
                    message: "No services available".to_string(),
                    suggested_alternatives: vec!["Register at least one service".to_string()],
                    recovery_actions: vec!["Check service registry".to_string()],
                });
            }

            // Filter for healthy services
            let healthy_services: Vec<_> = services
                .data
                .iter()
                .filter(|service| {
                    matches!(
                        service.health,
                        songbird_config::UniversalHealthStatus::Healthy
                    )
                })
                .collect();

            if healthy_services.is_empty() {
                return Err(SongbirdError::internal_error(Service {
                    service: "load_balancer".to_string(),
                    message: "No healthy services available".to_string(),
                    suggested_alternatives: vec!["Wait for services to become healthy".to_string()],
                    recovery_actions: vec!["Check service health status".to_string()],
                });
            }

            // Find service with least connections
            let connection_counts = self.connection_counts.read().await;
            let selected_service = healthy_services
                .data
                .iter()
                .min_by_key(|service| connection_counts.get(&service.name).unwrap_or(&0))
                .ok_or_else(|| songbird_errors::SongbirdError::Service {
                    service: "load-balancer".to_string(),
                    message: "No healthy services available for least connections strategy"
                        .to_string(),
                    suggested_alternatives: vec!["Wait for services to become healthy".to_string()],
                    recovery_actions: vec!["Check service health status".to_string()],
                })?;

            Ok(songbird_errors::evolved_success(songbird_errors::success((*selected_service)).clone()))
        }
    }

    fn update_service_health(
        &mut self,
        service_id: &str,
        is_healthy: bool,
    ) -> impl std::future::Future<Output = SongbirdResult<()>> + Send {
        let service_id = service_id.to_string();
        async move {
            tracing::info!("Service {} health updated: {}", service_id, is_healthy);
            Ok(songbird_errors::success(()))
        }
    }

    fn get_stats(
        &self,
    ) -> impl std::future::Future<Output = SongbirdResult<LoadBalancerStats>> + Send {
        async move {
            let stats = self.stats.read().await;
            Ok(songbird_errors::success(stats.clone()))
        }
    }

    fn reset_stats(&mut self) -> impl std::future::Future<Output = SongbirdResult<()>> + Send {
        async move {
            let mut stats = self.stats.write().await;
            stats.total_requests = 0;
            stats.successful_requests = 0;
            stats.failed_requests = 0;
            stats.average_response_time = 0.0;
            stats.active_connections = 0;
            stats.service_stats.clear();
            Ok(songbird_errors::success(()))
        }
    }

    fn record_request(
        &self,
        service_id: &str,
        success: bool,
        response_time: f64,
    ) -> impl std::future::Future<Output = ()> + Send {
        let service_id = service_id.to_string();
        async move {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;

            if success {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }

            // Update service-specific stats (same as RoundRobin)
            let service_stats = stats
                .service_stats
                .entry(service_id)
                .or_insert(ServiceStats {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    average_response_time: 0.0,
                    active_connections: 0,
                    weight: 1.0,
                });

            service_stats.requests += 1;
            if success {
                service_stats.successes += 1;
            } else {
                service_stats.failures += 1;
            }

            service_stats.average_response_time = (service_stats.average_response_time
                * (service_stats.requests - 1) as f64
                + response_time)
                / service_stats.requests as f64;
        }
    }
}

/// Weighted round-robin load balancer
#[derive(Debug)]
pub struct WeightedRoundRobinLoadBalancer {
    weights: Arc<RwLock<HashMap<String, u32>>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for WeightedRoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl WeightedRoundRobinLoadBalancer {
    /// Create a new weighted round-robin load balancer
    pub fn new() -> Self {
        Self {
            weights: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }
}

impl LoadBalancer for WeightedRoundRobinLoadBalancer {
    fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> impl std::future::Future<Output = SongbirdResult<ServiceInfo>> + Send {
        let services = services.to_vec();
        async move {
            if services.is_empty() {
                return Err(SongbirdError::internal_error(Service {
                    service: "load_balancer".to_string(),
                    message: "No services available".to_string(),
                    suggested_alternatives: vec!["Register at least one service".to_string()],
                    recovery_actions: vec!["Check service registry".to_string()],
                });
            }

            // For now, just return the first service (simplified implementation)
            // In a real implementation, we would use weighted selection
            Ok(songbird_errors::success(services[0].clone()))
        }
    }

    fn update_service_health(
        &mut self,
        service_id: &str,
        is_healthy: bool,
    ) -> impl std::future::Future<Output = SongbirdResult<()>> + Send {
        let service_id = service_id.to_string();
        async move {
            tracing::info!("Service {} health updated: {}", service_id, is_healthy);
            Ok(songbird_errors::success(()))
        }
    }

    fn get_stats(
        &self,
    ) -> impl std::future::Future<Output = SongbirdResult<LoadBalancerStats>> + Send {
        async move {
            let stats = self.stats.read().await;
            Ok(songbird_errors::success(stats.clone()))
        }
    }

    fn reset_stats(&mut self) -> impl std::future::Future<Output = SongbirdResult<()>> + Send {
        async move {
            let mut stats = self.stats.write().await;
            stats.total_requests = 0;
            stats.successful_requests = 0;
            stats.failed_requests = 0;
            stats.average_response_time = 0.0;
            stats.active_connections = 0;
            stats.service_stats.clear();
            Ok(songbird_errors::success(()))
        }
    }

    fn record_request(
        &self,
        service_id: &str,
        success: bool,
        response_time: f64,
    ) -> impl std::future::Future<Output = ()> + Send {
        let service_id = service_id.to_string();
        async move {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;

            if success {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }

            // Update service-specific stats (same pattern)
            let service_stats = stats
                .service_stats
                .entry(service_id)
                .or_insert(ServiceStats {
                    requests: 0,
                    successes: 0,
                    failures: 0,
                    average_response_time: 0.0,
                    active_connections: 0,
                    weight: 1.0,
                });

            service_stats.requests += 1;
            if success {
                service_stats.successes += 1;
            } else {
                service_stats.failures += 1;
            }

            service_stats.average_response_time = (service_stats.average_response_time
                * (service_stats.requests - 1) as f64
                + response_time)
                / service_stats.requests as f64;
        }
    }
}
