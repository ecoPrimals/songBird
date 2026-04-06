// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! [`ServiceDiscovery`](crate::traits::ServiceDiscovery) trait implementation.

use std::collections::HashMap;

use tracing::{debug, info, warn};

use songbird_types::SongbirdResult;

use crate::traits::ServiceDiscovery;

use super::ProductionServiceDiscovery;
use super::conversions::{instance_to_service_info, service_info_to_instance};
use super::types::{RegisteredService, ServiceHealthStatus};

impl ServiceDiscovery for ProductionServiceDiscovery {
    async fn discover(
        &self,
        query: crate::traits::ServiceQuery,
    ) -> SongbirdResult<Vec<crate::traits::ServiceInfo>> {
        info!("Discovering services with query: {:?}", query);

        let discovered: Vec<crate::traits::ServiceInfo> = {
            let services = self.services.read().await;
            services
                .values()
                .filter(|service| {
                    service.health_status == ServiceHealthStatus::Healthy
                        || service.health_status == ServiceHealthStatus::Degraded
                })
                .filter(|service| {
                    if let Some(ref name) = query.name
                        && !service.instance.name.contains(name)
                    {
                        return false;
                    }
                    true
                })
                .map(|service| instance_to_service_info(&service.instance))
                .collect()
        };

        info!("Discovered {} services", discovered.len());
        Ok(discovered)
    }

    async fn register(&self, service: crate::traits::ServiceInfo) -> SongbirdResult<()> {
        let instance = service_info_to_instance(&service);
        let registered = RegisteredService {
            instance,
            registered_at: std::time::SystemTime::now(),
            last_heartbeat: None,
            health_status: ServiceHealthStatus::Unknown,
            retry_count: 0,
        };

        info!("Registering service: {}", service.service_id);
        {
            let mut services = self.services.write().await;
            services.insert(service.service_id, registered);
        }
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Deregistering service: {}", service_id);

        {
            let mut services = self.services.write().await;
            if services.remove(service_id).is_some() {
                info!("Service deregistered successfully: {}", service_id);
            } else {
                warn!("Attempted to deregister unknown service: {}", service_id);
            }
        }

        {
            let mut health_cache = self.health_cache.write().await;
            health_cache.remove(service_id);
        }

        Ok(())
    }

    async fn watch(
        &self,
        _query: crate::traits::ServiceQuery,
    ) -> SongbirdResult<
        std::pin::Pin<Box<dyn futures_util::Stream<Item = crate::traits::ServiceEvent> + Send>>,
    > {
        use futures_util::stream;
        // Periodic push-based updates would require extending `ServiceEvent`; keep API wired with an empty stream.
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(
        &self,
        service_id: &str,
        health: crate::traits::discovery::ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_id) {
                let internal_status = match health {
                    crate::traits::discovery::ServiceHealthStatus::Healthy => {
                        ServiceHealthStatus::Healthy
                    }
                    crate::traits::discovery::ServiceHealthStatus::Degraded => {
                        ServiceHealthStatus::Degraded
                    }
                    crate::traits::discovery::ServiceHealthStatus::Unhealthy => {
                        ServiceHealthStatus::Unhealthy
                    }
                    crate::traits::discovery::ServiceHealthStatus::Unknown => {
                        ServiceHealthStatus::Unknown
                    }
                };
                debug!(
                    "Updating health for {}: {:?} -> {:?}",
                    service_id, service.health_status, internal_status
                );
                service.health_status = internal_status;
                service.last_heartbeat = Some(std::time::SystemTime::now());
            } else {
                warn!("Health update requested for unknown service: {}", service_id);
            }
        }
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<crate::traits::ServiceInfo>> {
        let all: Vec<crate::traits::ServiceInfo> = {
            let services = self.services.read().await;
            services.values().map(|s| instance_to_service_info(&s.instance)).collect()
        };
        Ok(all)
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_id) {
                for (k, v) in &metadata {
                    service.instance.metadata.insert(k.clone(), v.clone());
                }
                debug!("Updated metadata for service {}: {} keys", service_id, metadata.len());
            } else {
                warn!("Metadata update requested for unknown service: {}", service_id);
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
