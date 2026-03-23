// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Service Discovery
//!
//! This module provides real service discovery implementations that replace
//! all mock and placeholder discovery providers.
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for zero-cost abstraction

mod conversions;
mod health;
mod service_discovery_impl;
mod types;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::discovery::core::ServiceInstance;
use songbird_types::SongbirdResult;

pub use types::{HealthRecord, ProductionDiscoveryConfig, RegisteredService, ServiceHealthStatus};

/// Production service discovery implementation
#[derive(Debug)]
pub struct ProductionServiceDiscovery {
    /// Active services registry
    pub(super) services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    /// Discovery configuration
    config: ProductionDiscoveryConfig,
    /// Service health cache
    pub(super) health_cache: Arc<RwLock<HashMap<String, HealthRecord>>>,
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
                    let health_result =
                        health::perform_health_check(&service.instance, timeout).await;

                    let health_record = HealthRecord {
                        service_id: service.instance.id.clone(),
                        status: health_result.status,
                        last_check: SystemTime::now(),
                        response_time_ms: health_result.response_time_ms,
                        error_message: health_result.error_message,
                    };

                    // Update health cache (release before taking `services` write lock)
                    {
                        let mut health_cache_guard = health_cache.write().await;
                        health_cache_guard.insert(service.instance.id.clone(), health_record);
                    }

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

    /// Get services by capability
    pub async fn get_services_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceInstance>> {
        let matching_services: Vec<ServiceInstance> = {
            let services = self.services.read().await;
            services
                .values()
                .filter(|service| {
                    service.instance.capabilities.iter().any(|c| c == capability)
                        && service.health_status == ServiceHealthStatus::Healthy
                })
                .map(|service| service.instance.clone())
                .collect()
        };

        debug!("Found {} services with capability '{}'", matching_services.len(), capability);
        Ok(matching_services)
    }

    /// Get service health information
    pub async fn get_service_health(
        &self,
        service_id: &str,
    ) -> SongbirdResult<Option<HealthRecord>> {
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
        drop(services);
        if removed_count > 0 {
            info!("Cleaned up {} unhealthy services", removed_count);
        }

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests;
