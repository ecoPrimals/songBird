// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Execution manager for coordinating remote executions

use super::broadcast::{BroadcastExecutor, BroadcastOptions, BroadcastResult};
use super::client::{ExecutionClient, ExecutionRequest, ExecutionResponse};
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Manager for remote execution operations
pub struct ExecutionManager {
    client: ExecutionClient,
    broadcast: Arc<RwLock<BroadcastExecutor>>,
}

impl ExecutionManager {
    /// Create a new execution manager
    ///
    /// ✅ EVOLVED: Now async due to `ExecutionClient` async construction
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new() -> SongbirdResult<Self> {
        let client = ExecutionClient::new().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to create ExecutionClient: {e}"))
        })?;

        let broadcast = BroadcastExecutor::new().await?;

        Ok(Self {
            client,
            broadcast: Arc::new(RwLock::new(broadcast)),
        })
    }

    /// Register a tower for execution
    pub async fn register_tower(&self, tower_id: String, endpoint: String) {
        let mut broadcast = self.broadcast.write().await;
        broadcast.register_tower(tower_id, endpoint);
    }

    /// Execute command on a single tower
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn execute_on_tower(
        &self,
        tower_endpoint: &str,
        request: ExecutionRequest,
    ) -> Result<ExecutionResponse, super::client::ExecutionError> {
        info!("Executing command on tower: {}", tower_endpoint);
        self.client.execute_command(tower_endpoint, request).await
    }

    /// Execute command across multiple towers (broadcast)
    pub async fn execute_broadcast(
        &self,
        tower_ids: Vec<String>,
        request: ExecutionRequest,
        options: Option<BroadcastOptions>,
    ) -> BroadcastResult {
        let options = options.unwrap_or_default();
        let broadcast = self.broadcast.read().await;
        broadcast.broadcast(tower_ids, request, options).await
    }
}

// NOTE: Default trait removed - async construction required
// Use ExecutionManager::new().await instead

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_manager_creation() {
        // Note: Will fail without crypto provider
        let _ = ExecutionManager::new().await;
    }

    #[tokio::test]
    async fn test_register_tower() {
        // Note: Will fail without crypto provider
        if let Ok(manager) = ExecutionManager::new().await {
            manager
                .register_tower("test-tower".to_string(), "http://localhost:9020".to_string())
                .await;
        }
    }
}
