//! Execution manager for coordinating remote executions

use super::broadcast::{BroadcastExecutor, BroadcastOptions, BroadcastResult};
use super::client::{ExecutionClient, ExecutionRequest, ExecutionResponse};
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
    pub fn new() -> Self {
        Self {
            client: ExecutionClient::new(),
            broadcast: Arc::new(RwLock::new(BroadcastExecutor::new())),
        }
    }
    
    /// Register a tower for execution
    pub async fn register_tower(&self, tower_id: String, endpoint: String) {
        let mut broadcast = self.broadcast.write().await;
        broadcast.register_tower(tower_id, endpoint);
    }
    
    /// Execute command on a single tower
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

impl Default for ExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_manager_creation() {
        let manager = ExecutionManager::new();
        // Just verify it constructs
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_register_tower() {
        let manager = ExecutionManager::new();
        manager.register_tower("test-tower".to_string(), "http://localhost:9020".to_string()).await;
        // Verify no panic
        assert!(true);
    }
}

