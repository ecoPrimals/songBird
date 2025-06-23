//! Service Registry Module
//!
//! Service registration and management

use crate::errors::{Result, SongbirdError};
use crate::traits::{ServiceInfo, UniversalService};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service handle for managing a registered service
pub struct ServiceHandle<S: UniversalService> {
    pub service: Arc<RwLock<S>>,
    pub info: ServiceInfo,
}

impl<S: UniversalService> ServiceHandle<S> {
    pub fn new(service: S, info: ServiceInfo) -> Self {
        Self {
            service: Arc::new(RwLock::new(service)),
            info,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .start()
            .await
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut service = self.service.write().await;
        service
            .stop()
            .await
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))?;
        Ok(())
    }

    pub async fn health_check(&self) -> Result<serde_json::Value> {
        let service = self.service.read().await;
        let health = service
            .health_check()
            .await
            .map_err(|e| SongbirdError::health_check_failed(&self.info.id, e.to_string()))?;

        serde_json::to_value(health)
            .map_err(|e| SongbirdError::service_error(&self.info.id, e.to_string()))
    }
}

/// Central service registry
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl ServiceRegistry {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn register(&self, info: ServiceInfo) -> Result<()> {
        let service_id = info.id.clone();

        tracing::info!(service_id = %service_id, "Registering service");

        self.services.write().await.insert(service_id.clone(), info);

        tracing::info!(service_id = %service_id, "Service registered successfully");

        Ok(())
    }

    pub async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!(service_id = %service_id, "Unregistering service");

        self.services.write().await.remove(service_id);

        tracing::info!(service_id = %service_id, "Service unregistered successfully");
        Ok(())
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.services.read().await.values().cloned().collect())
    }

    pub async fn get_service(&self, service_id: &str) -> Result<Option<ServiceInfo>> {
        Ok(self.services.read().await.get(service_id).cloned())
    }

    pub async fn service_count(&self) -> usize {
        self.services.read().await.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
