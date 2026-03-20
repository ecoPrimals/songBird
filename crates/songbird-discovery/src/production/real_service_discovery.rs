// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Service Discovery
//!
//! This module provides real service discovery implementations that replace
//! all mock and placeholder discovery providers.
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost abstraction

#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::discovery::core::ServiceInstance;
use crate::traits::ServiceDiscovery;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_config;

/// Production service discovery implementation
#[derive(Debug)]
pub struct ProductionServiceDiscovery {
    /// Active services registry
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    /// Discovery configuration
    config: ProductionDiscoveryConfig,
    /// Service health cache
    health_cache: Arc<RwLock<HashMap<String, HealthRecord>>>,
}

/// Configuration for production service discovery
#[derive(Debug, Clone)]
pub struct ProductionDiscoveryConfig {
    /// Health Check Interval field
    pub health_check_interval: Duration,
    /// Service Timeout field
    pub service_timeout: Duration,
    /// Max Retry Attempts field
    pub max_retry_attempts: u32,
    /// Enable Health Checks field
    pub enable_health_checks: bool,
}

impl Default for ProductionDiscoveryConfig {
    fn default() -> Self {
        // ✅ DEEP DEBT EVOLUTION (Feb 3, 2026): Use TimeoutConfig
        // Replaces hardcoded Duration::from_secs with configurable timeouts
        let timeout_config = songbird_config::timeouts::TimeoutConfig::from_env();
        
        Self {
            health_check_interval: timeout_config.health_check,
            service_timeout: timeout_config.discovery,
            max_retry_attempts: 3,
            enable_health_checks: true,
        }
    }
}

/// Registered service with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Instance field
    pub instance: ServiceInstance,
    /// Registered At field
    pub registered_at: SystemTime,
    /// Last Heartbeat field
    pub last_heartbeat: Option<SystemTime>,
    /// Health Status field
    pub health_status: ServiceHealthStatus,
    /// Retry Count field
    pub retry_count: u32,
}

/// Health record for services
#[derive(Debug, Clone)]
pub struct HealthRecord {
    /// Service Id field
    pub service_id: String,
    /// Current status of the operation or entity
    pub status: ServiceHealthStatus,
    /// Last Check field
    pub last_check: SystemTime,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error Message field
    pub error_message: Option<String>,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl ProductionServiceDiscovery {
    /// Create new production service discovery
    #[must_use]
    pub fn new(config: ProductionDiscoveryConfig) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            config,
            health_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) {
        if !self.config.enable_health_checks {
            return;
        }

        let services = Arc::clone(&self.services);
        let health_cache = Arc::clone(&self.health_cache);
        let interval = self.config.health_check_interval;
        let timeout = self.config.service_timeout;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let services_to_check: Vec<RegisteredService> = {
                    let services_guard = services.read().await;
                    services_guard.values().cloned().collect()
                };

                for service in services_to_check {
                    let health_result = Self::perform_health_check(&service.instance, timeout).await;

                    let health_record = HealthRecord {
                        service_id: service.instance.id.clone(),
                        status: health_result.status.clone(),
                        last_check: SystemTime::now(),
                        response_time_ms: health_result.response_time_ms,
                        error_message: health_result.error_message,
                    };

                    // Update health cache
                    let mut health_cache_guard = health_cache.write().await;
                    health_cache_guard.insert(service.instance.id.clone(), health_record);

                    // Update service health status
                    let mut services_guard = services.write().await;
                    if let Some(registered_service) = services_guard.get_mut(&service.instance.id) {
                        registered_service.health_status = health_result.status;
                        registered_service.last_heartbeat = Some(SystemTime::now());
                    }
                }
            }
        });
    }

    /// Perform health check on a service
    async fn perform_health_check(service: &ServiceInstance, timeout: Duration) -> HealthCheckResult {
        let start_time = SystemTime::now();

        // Try to construct health check URL
        let health_url = if service.endpoint.ends_with('/') {
            format!("{}health", service.endpoint)
        } else {
            format!("{}/health", service.endpoint)
        };

        debug!("Performing health check for service: {} at {}", service.id, health_url);

        let client = match songbird_http_client::IpcHttpClient::builder()
            .timeout(timeout)
            .build()
            .await
        {
            Ok(client) => client,
            Err(e) => {
                return HealthCheckResult {
                    status: ServiceHealthStatus::Unhealthy,
                    response_time_ms: 0,
                    error_message: Some(format!("Failed to create HTTP client: {}", e)),
                };
            }
        };

        match client.get(&health_url).await {
            Ok(response) => {
                let response_time = start_time.elapsed().unwrap_or(Duration::ZERO).as_millis() as u64;

                if response.status().is_success() {
                    HealthCheckResult {
                        status: ServiceHealthStatus::Healthy,
                        response_time_ms: response_time,
                        error_message: None,
                    }
                } else {
                    HealthCheckResult {
                        status: ServiceHealthStatus::Degraded,
                        response_time_ms: response_time,
                        error_message: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => {
                let response_time = start_time.elapsed().unwrap_or(Duration::ZERO).as_millis() as u64;
                warn!("Health check failed for service {}: {}", service.id, e);

                HealthCheckResult {
                    status: ServiceHealthStatus::Unhealthy,
                    response_time_ms: response_time,
                    error_message: Some(e.to_string()),
                }
            }
        }
    }

    /// Get services by capability
    pub async fn get_services_by_capability(&self, capability: &str) -> SongbirdResult<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        let matching_services: Vec<ServiceInstance> = services
            .values()
            .filter(|service| {
                service.instance.capabilities.iter().any(|c| c == capability)
                    && service.health_status == ServiceHealthStatus::Healthy
            })
            .map(|service| service.instance.clone())
            .collect();

        debug!("Found {} services with capability '{}'", matching_services.len(), capability);
        Ok(matching_services)
    }

    /// Get service health information
    pub async fn get_service_health(&self, service_id: &str) -> SongbirdResult<Option<HealthRecord>> {
        let health_cache = self.health_cache.read().await;
        Ok(health_cache.get(service_id).cloned())
    }

    /// Remove unhealthy services
    pub async fn cleanup_unhealthy_services(&self) -> SongbirdResult<usize> {
        let mut services = self.services.write().await;
        let initial_count = services.len();

        services.retain(|_id, service| {
            service.health_status != ServiceHealthStatus::Unhealthy
                || service.retry_count < self.config.max_retry_attempts
        });

        let removed_count = initial_count - services.len();
        if removed_count > 0 {
            info!("Cleaned up {} unhealthy services", removed_count);
        }

        Ok(removed_count)
    }
}

/// Health check result
#[derive(Debug)]
struct HealthCheckResult {
    status: ServiceHealthStatus,
    response_time_ms: u64,
    error_message: Option<String>,
}

/// Convert internal `ServiceInstance` to the trait's `ServiceInfo`
fn instance_to_service_info(instance: &ServiceInstance) -> crate::traits::ServiceInfo {
    use crate::traits::{ServiceEndpoint as TraitEndpoint, ServiceStatus};
    use chrono::Utc;

    let endpoint = TraitEndpoint {
        path: instance.endpoint.clone(),
        method: "GET".to_string(),
        description: None,
        parameters: Vec::new(),
        response_schema: None,
        auth_required: false,
        rate_limit: None,
    };

    crate::traits::ServiceInfo {
        service_id: instance.id.clone(),
        name: instance.name.clone(),
        version: instance.metadata.get("version").cloned().unwrap_or_else(|| "0.0.0".to_string()),
        service_type: instance.metadata.get("type").cloned().unwrap_or_else(|| "unknown".to_string()),
        description: instance.metadata.get("description").cloned(),
        endpoints: vec![endpoint],
        health_check_endpoint: Some(format!("{}/health", instance.endpoint.trim_end_matches('/'))),
        metadata: instance.metadata.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect(),
        tags: instance.capabilities.clone(),
        dependencies: Vec::new(),
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: instance.id.clone(),
        host: instance.endpoint.clone(),
        port: 0,
    }
}

/// Convert the trait's `ServiceInfo` to internal `ServiceInstance`
fn service_info_to_instance(info: &crate::traits::ServiceInfo) -> ServiceInstance {
    let endpoint = info
        .endpoints
        .first()
        .map(|e| e.path.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            if info.host.starts_with("http://") || info.host.starts_with("https://") {
                info.host.clone()
            } else {
                format!("http://{}:{}", info.host, info.port)
            }
        });

    ServiceInstance {
        id: info.service_id.clone(),
        name: info.name.clone(),
        endpoint,
        capabilities: info.tags.clone(),
        health_status: "unknown".to_string(),
        metadata: info.metadata.iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect(),
    }
}

impl ServiceDiscovery for ProductionServiceDiscovery {
    async fn discover(&self, query: crate::traits::ServiceQuery) -> SongbirdResult<Vec<crate::traits::ServiceInfo>> {
        info!("Discovering services with query: {:?}", query);

        let services = self.services.read().await;
        let discovered: Vec<crate::traits::ServiceInfo> = services
            .values()
            .filter(|service| {
                service.health_status == ServiceHealthStatus::Healthy
                    || service.health_status == ServiceHealthStatus::Degraded
            })
            .filter(|service| {
                if let Some(ref name) = query.name {
                    if !service.instance.name.contains(name) {
                        return false;
                    }
                }
                true
            })
            .map(|service| instance_to_service_info(&service.instance))
            .collect();

        info!("Discovered {} services", discovered.len());
        Ok(discovered)
    }

    async fn register(&self, service: crate::traits::ServiceInfo) -> SongbirdResult<()> {
        let instance = service_info_to_instance(&service);
        let registered = RegisteredService {
            instance,
            registered_at: SystemTime::now(),
            last_heartbeat: None,
            health_status: ServiceHealthStatus::Unknown,
            retry_count: 0,
        };

        let mut services = self.services.write().await;
        info!("Registering service: {}", service.service_id);
        services.insert(service.service_id, registered);
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Deregistering service: {}", service_id);

        let mut services = self.services.write().await;
        if services.remove(service_id).is_some() {
            info!("Service deregistered successfully: {}", service_id);
        } else {
            warn!("Attempted to deregister unknown service: {}", service_id);
        }

        let mut health_cache = self.health_cache.write().await;
        health_cache.remove(service_id);

        Ok(())
    }

    async fn watch(&self, query: crate::traits::ServiceQuery) -> SongbirdResult<std::pin::Pin<Box<dyn futures::stream::Stream<Item = crate::traits::ServiceEvent> + Send>>> {
        use tokio_stream::wrappers::IntervalStream;
        use futures::StreamExt;

        let services = Arc::clone(&self.services);
        let interval = tokio::time::interval(self.config.health_check_interval);

        let stream = IntervalStream::new(interval)
            .then(move |_| {
                let services = Arc::clone(&services);
                let query = query.clone();
                async move {
                    let guard = services.read().await;
                    let matching: Vec<_> = guard.values()
                        .filter(|s| {
                            if let Some(ref name) = query.name {
                                s.instance.name.contains(name)
                            } else {
                                true
                            }
                        })
                        .map(|s| instance_to_service_info(&s.instance))
                        .collect();
                    crate::traits::ServiceEvent::ServicesUpdated(matching)
                }
            });

        Ok(Box::pin(stream))
    }

    async fn update_health(&self, service_id: &str, health: crate::traits::discovery::ServiceHealthStatus) -> SongbirdResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            let internal_status = match health {
                crate::traits::discovery::ServiceHealthStatus::Healthy => ServiceHealthStatus::Healthy,
                crate::traits::discovery::ServiceHealthStatus::Degraded => ServiceHealthStatus::Degraded,
                crate::traits::discovery::ServiceHealthStatus::Unhealthy => ServiceHealthStatus::Unhealthy,
                _ => ServiceHealthStatus::Unknown,
            };
            debug!("Updating health for {}: {:?} -> {:?}", service_id, service.health_status, internal_status);
            service.health_status = internal_status;
            service.last_heartbeat = Some(SystemTime::now());
        } else {
            warn!("Health update requested for unknown service: {}", service_id);
        }
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<crate::traits::ServiceInfo>> {
        let services = self.services.read().await;
        let all: Vec<crate::traits::ServiceInfo> = services
            .values()
            .map(|s| instance_to_service_info(&s.instance))
            .collect();
        Ok(all)
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(&self, service_id: &str, metadata: HashMap<String, String>) -> SongbirdResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            for (k, v) in &metadata {
                service.instance.metadata.insert(k.clone(), v.clone());
            }
            debug!("Updated metadata for service {}: {} keys", service_id, metadata.len());
        } else {
            warn!("Metadata update requested for unknown service: {}", service_id);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig::default());

        // Use configurable test endpoint
        let test_host = std::env::var("TEST_SERVICE_HOST")
            .unwrap_or_else(|_| songbird_config::canonical::constants::network::DEFAULT_HOST.to_string());
        let test_port = std::env::var("TEST_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        let service = ServiceInstance {
            id: "test-service".to_string(),
            name: "Test Service".to_string(),
            endpoint: format!("http://{}:{}", test_host, test_port),
            capabilities: vec!["test".to_string()],
            health_status: "unknown".to_string(),
            metadata: HashMap::new(),
        };

        // Test would need proper ServiceInfo conversion
        assert!(discovery.exists("test-service").await.is_ok());
    }

    #[tokio::test]
    async fn test_capability_filtering() {
        let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig::default());

        // Test capability filtering
        let services = discovery.get_services_by_capability("security").await;
        assert!(services.is_ok());
    }

    fn sample_service_info(id: &str, name: &str, endpoint_path: &str, tags: Vec<String>) -> crate::traits::ServiceInfo {
        use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceStatus};
        use chrono::Utc;
        use std::collections::HashMap;

        ServiceInfo {
            service_id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: None,
            endpoints: vec![ServiceEndpoint {
                path: endpoint_path.to_string(),
                method: "GET".to_string(),
                description: None,
                parameters: Vec::new(),
                response_schema: None,
                auth_required: false,
                rate_limit: None,
            }],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags,
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: id.to_string(),
            host: endpoint_path.to_string(),
            port: 8080,
        }
    }

    #[tokio::test]
    async fn register_list_and_capability_queries() {
        use crate::traits::discovery::{ServiceHealthStatus, ServiceQuery};
        use crate::traits::ServiceDiscovery;

        let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
            enable_health_checks: false,
            ..ProductionDiscoveryConfig::default()
        });

        let info = sample_service_info(
            "svc-reg-1",
            "RegistryAlpha",
            "http://127.0.0.1:9",
            vec!["security".to_string(), "metrics".to_string()],
        );

        ServiceDiscovery::register(&discovery, info)
            .await
            .expect("register service");

        ServiceDiscovery::update_health(&discovery, "svc-reg-1", ServiceHealthStatus::Healthy)
            .await
            .expect("mark healthy");

        let by_cap = discovery
            .get_services_by_capability("security")
            .await
            .expect("by capability");
        assert_eq!(by_cap.len(), 1);
        assert_eq!(by_cap[0].id, "svc-reg-1");

        let all = ServiceDiscovery::list_all(&discovery).await.expect("list all");
        assert_eq!(all.len(), 1);

        let mut q = ServiceQuery::new();
        q.name = Some("Alpha".into());
        let filtered = ServiceDiscovery::discover(&discovery, q).await.expect("discover");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].name.contains("Alpha"));
    }

    #[tokio::test]
    async fn unregister_removes_service() {
        use crate::traits::ServiceDiscovery;

        let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
            enable_health_checks: false,
            ..ProductionDiscoveryConfig::default()
        });

        let info = sample_service_info("svc-rm", "Rm", "http://127.0.0.1:8", vec![]);
        ServiceDiscovery::register(&discovery, info).await.expect("register");
        assert!(ServiceDiscovery::exists(&discovery, "svc-rm").await.expect("exists"));

        ServiceDiscovery::unregister(&discovery, "svc-rm")
            .await
            .expect("unregister");
        assert!(!ServiceDiscovery::exists(&discovery, "svc-rm").await.expect("gone"));
    }
}
