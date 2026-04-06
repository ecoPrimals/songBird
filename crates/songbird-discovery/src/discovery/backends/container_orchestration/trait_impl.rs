// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! [`crate::traits::ServiceDiscovery`] implementation for [`super::UniversalContainerOrchestration`].

use super::types::UniversalContainerOrchestration;
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::service::ServiceInfo;
use crate::traits::{ServiceDiscovery, ServiceEvent, ServiceQuery};
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::info;

// Native async trait implementation (no boxing overhead)
impl ServiceDiscovery for UniversalContainerOrchestration {
    async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover_services_universal(query).await
    }

    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        info!("Universal container service registration: {}", service.name);
        // In a real implementation, this would register with detected container orchestration systems
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Universal container service unregistration: {}", service_id);
        // In a real implementation, this would unregister from detected container orchestration systems
        Ok(())
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn futures_util::Stream<Item = ServiceEvent> + Send>>> {
        use futures_util::stream;

        // Return an empty stream for now - would implement real watching
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        info!("Universal container health update for service {}: {:?}", service_id, health);
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover(ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let services = self.list_all().await?;
        Ok(services.iter().any(|s| s.service_id == service_id))
    }

    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        info!("Universal container metadata update for service {}: {:?}", service_id, metadata);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
