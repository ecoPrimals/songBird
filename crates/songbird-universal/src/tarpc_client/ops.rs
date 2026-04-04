// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Typed tarpc service calls (discovery, registration, health, version, protocols).

use tracing::debug;

use crate::tarpc_types::{
    HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration, VersionInfo,
};
use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

impl TarpcClient {
    /// Discover services by capability.
    pub async fn discover(&self, capability: &str) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering services with capability: {}", capability);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .discover(ctx, capability.to_string())
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Discover all available services.
    pub async fn discover_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering all services");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .discover_all(ctx)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Register a service with the remote registry.
    pub async fn register(
        &self,
        registration: ServiceRegistration,
    ) -> SongbirdResult<RegistrationResult> {
        debug!("Registering service: {}", registration.service_id);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .register(ctx, registration)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Unregister a service by id.
    pub async fn unregister(&self, service_id: &str) -> SongbirdResult<RegistrationResult> {
        debug!("Unregistering service: {}", service_id);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .unregister(ctx, service_id.to_string())
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Remote health status.
    pub async fn health(&self) -> SongbirdResult<HealthStatus> {
        debug!("Checking health status");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client.health(ctx).await.map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Protocol and build version info.
    pub async fn version(&self) -> SongbirdResult<VersionInfo> {
        debug!("Getting version information");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client.version(ctx).await.map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }

    /// Supported wire protocols and ports.
    pub async fn protocols(&self) -> SongbirdResult<Vec<ProtocolInfo>> {
        debug!("Getting available protocols");
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .protocols(ctx)
            .await
            .map_err(|e| SongbirdError::rpc(format!("tarpc call failed: {e}")))
    }
}
