// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection Manager Module
//!
//! Handles lifecycle of connections to primals:
//! - Establishing connections
//! - Health monitoring
//! - Connection maintenance
//! - Disconnection
//!
//! Part of the smart refactoring from monolithic adapter.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::super::HEALTH_PATH;
use super::super::connection::{ConnectionHealth, PrimalConnection};
use super::super::error::CapabilityError;
use super::super::types::PrimalType;

/// Connection manager component
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Active primal connections
    connections: Arc<RwLock<HashMap<String, PrimalConnection>>>,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Establish connection to a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the connection test fails or the primal health check fails
    pub async fn establish_connection(
        &self,
        primal_name: &str,
        endpoint: &str,
    ) -> Result<(), CapabilityError> {
        info!("🔌 Establishing connection to primal: {}", primal_name);

        // Create connection
        let connection = PrimalConnection {
            name: primal_name.to_string(),
            endpoint: endpoint.to_string(),
            primal_type: Self::infer_primal_type(primal_name),
            health: ConnectionHealth::Unknown,
            last_contact: chrono::Utc::now(),
            last_health_check: None,
            metadata: std::collections::HashMap::new(),
        };

        // Test connection health
        if self.test_primal_health(&connection).await.is_ok() {
            let mut connections = self.connections.write().await;
            connections.insert(primal_name.to_string(), connection);
            info!("✅ Connected to {}", primal_name);
            Ok(())
        } else {
            warn!("❌ Failed health check for {}", primal_name);
            Err(CapabilityError::NetworkError(format!("Health check failed for {primal_name}")))
        }
    }

    /// Test primal health
    async fn test_primal_health(
        &self,
        connection: &PrimalConnection,
    ) -> Result<(), CapabilityError> {
        let health_endpoint = format!("{}{}", connection.endpoint, HEALTH_PATH);
        debug!("🏥 Testing health: {}", health_endpoint);

        let client = songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| CapabilityError::NetworkError(format!("HTTP client error: {e}")))?;

        match client.get(&health_endpoint).await {
            Ok(response) if response.is_success() => {
                debug!("✅ Health check passed for {}", connection.name);
                Ok(())
            }
            Ok(response) => {
                warn!(
                    "⚠️  Health check returned non-success status for {}: {}",
                    connection.name,
                    response.status()
                );
                Err(CapabilityError::NetworkError(format!(
                    "Non-success status: {}",
                    response.status()
                )))
            }
            Err(e) => {
                warn!("❌ Health check failed for {}: {}", connection.name, e);
                Err(CapabilityError::NetworkError(format!("Health check error: {e}")))
            }
        }
    }

    /// Infer primal type from name using only capability terms.
    ///
    /// Primal-agnostic: matches on domain terminology rather than specific
    /// primal names. Provider identities are discovered at runtime.
    fn infer_primal_type(name: &str) -> PrimalType {
        let name_lower = name.to_lowercase();

        if name_lower.contains("security")
            || name_lower.contains("auth")
            || name_lower.contains("crypto")
        {
            PrimalType::Security
        } else if name_lower.contains("compute")
            || name_lower.contains("worker")
            || name_lower.contains("exec")
        {
            PrimalType::Compute
        } else if name_lower.contains("storage")
            || name_lower.contains("data")
            || name_lower.contains("persist")
        {
            PrimalType::Storage
        } else if name_lower.contains("ai")
            || name_lower.contains("ml")
            || name_lower.contains("inference")
        {
            PrimalType::AI
        } else if name_lower.contains("orchestrat") || name_lower.contains("coordinat") {
            PrimalType::Orchestration
        } else {
            PrimalType::Generic
        }
    }

    /// Get all active connections
    pub async fn get_all_connections(&self) -> Vec<PrimalConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Disconnect from a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not currently connected
    pub async fn disconnect_from_primal(&self, primal_name: &str) -> Result<(), CapabilityError> {
        info!("🔌 Disconnecting from primal: {}", primal_name);

        let mut connections = self.connections.write().await;
        if connections.remove(primal_name).is_some() {
            info!("✅ Disconnected from {}", primal_name);
            Ok(())
        } else {
            warn!("⚠️  Primal {} was not connected", primal_name);
            Err(CapabilityError::PrimalNotFound(format!("Primal {primal_name} not connected")))
        }
    }

    /// Update connection health for all primals
    pub async fn update_connection_health(&self) -> Result<(), CapabilityError> {
        debug!("🔄 Updating connection health for all primals");

        let connections_snapshot: Vec<PrimalConnection> = {
            let connections = self.connections.read().await;
            connections.values().cloned().collect()
        };

        for mut connection in connections_snapshot {
            let health_result = self.test_primal_health(&connection).await;

            connection.health = if health_result.is_ok() {
                ConnectionHealth::Healthy
            } else {
                ConnectionHealth::Unhealthy
            };
            connection.last_contact = chrono::Utc::now();

            // Update stored connection
            let mut connections = self.connections.write().await;
            connections.insert(connection.name.clone(), connection);
        }

        debug!("✅ Connection health updated");
        Ok(())
    }

    /// Get connection by primal name
    ///
    /// Future use: Will be used for connection pooling and reuse optimization.
    /// Currently connections are managed ephemerally, but this enables persistent connections.
    #[allow(dead_code, reason = "reserved for future connection pooling and reuse optimization")]
    pub async fn get_connection(&self, primal_name: &str) -> Option<PrimalConnection> {
        let connections = self.connections.read().await;
        connections.get(primal_name).cloned()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_infer_primal_type() {
        assert_eq!(ConnectionManager::infer_primal_type("security-provider"), PrimalType::Security);
        assert_eq!(ConnectionManager::infer_primal_type("auth-gateway"), PrimalType::Security);
        assert_eq!(ConnectionManager::infer_primal_type("compute-worker"), PrimalType::Compute);
        assert_eq!(ConnectionManager::infer_primal_type("data-storage"), PrimalType::Storage);
        assert_eq!(ConnectionManager::infer_primal_type("ml-inference"), PrimalType::AI);
        assert_eq!(ConnectionManager::infer_primal_type("unknown-service"), PrimalType::Generic);
    }

    #[test]
    fn test_infer_primal_type_orchestration() {
        assert_eq!(
            ConnectionManager::infer_primal_type("task-orchestrator"),
            PrimalType::Orchestration
        );
        assert_eq!(
            ConnectionManager::infer_primal_type("coordinator-node"),
            PrimalType::Orchestration
        );
    }

    #[test]
    fn test_infer_primal_type_crypto_and_persist() {
        assert_eq!(ConnectionManager::infer_primal_type("crypto-vault"), PrimalType::Security);
        assert_eq!(ConnectionManager::infer_primal_type("persist-layer"), PrimalType::Storage);
        assert_eq!(ConnectionManager::infer_primal_type("exec-runner"), PrimalType::Compute);
    }

    #[test]
    fn test_connection_manager_default() {
        let _ = ConnectionManager::default();
    }

    #[tokio::test]
    async fn test_disconnect_not_connected() {
        let manager = ConnectionManager::new();
        let err = manager.disconnect_from_primal("missing").await.expect_err("not connected");
        match err {
            CapabilityError::PrimalNotFound(msg) => assert!(msg.contains("missing")),
            _ => panic!("expected PrimalNotFound"),
        }
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let manager = ConnectionManager::new();

        // Initially no connections
        assert_eq!(manager.get_all_connections().await.len(), 0);

        // Can query non-existent connection
        assert!(manager.get_connection("test").await.is_none());
    }

    #[test]
    fn test_infer_primal_type_case_insensitive() {
        assert_eq!(ConnectionManager::infer_primal_type("SECURITY-GW"), PrimalType::Security);
        assert_eq!(ConnectionManager::infer_primal_type("Ai-Core"), PrimalType::AI);
    }

    #[test]
    fn test_infer_primal_type_boundary_keywords() {
        assert_eq!(ConnectionManager::infer_primal_type("my-authz"), PrimalType::Security);
        assert_eq!(ConnectionManager::infer_primal_type("worker-pool"), PrimalType::Compute);
        assert_eq!(ConnectionManager::infer_primal_type("data-lake"), PrimalType::Storage);
    }

    #[tokio::test]
    async fn test_update_connection_health_no_connections() {
        let manager = ConnectionManager::new();
        manager.update_connection_health().await.expect("no-op");
    }

    #[tokio::test]
    async fn test_get_all_connections_returns_vec() {
        let manager = ConnectionManager::new();
        let v = manager.get_all_connections().await;
        assert!(v.is_empty());
    }

    #[test]
    fn test_connection_manager_clone_shares_map() {
        let a = ConnectionManager::new();
        let b = a.clone();
        assert!(Arc::ptr_eq(&a.connections, &b.connections));
    }

    #[test]
    fn test_infer_primal_type_generic_when_no_keyword() {
        assert_eq!(ConnectionManager::infer_primal_type("zzz-unknown"), PrimalType::Generic);
    }

    #[tokio::test]
    async fn test_disconnect_twice_second_fails() {
        let manager = ConnectionManager::new();
        let err = manager.disconnect_from_primal("nope").await.expect_err("missing");
        match err {
            CapabilityError::PrimalNotFound(_) => {}
            _ => panic!("expected PrimalNotFound"),
        }
    }
}
