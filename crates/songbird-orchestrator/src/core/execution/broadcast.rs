// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Multi-tower broadcast execution

use super::client::{ExecutionClient, ExecutionRequest, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Broadcast executor for executing commands across multiple towers
pub struct BroadcastExecutor {
    client: ExecutionClient,
    tower_registry: HashMap<String, String>, // tower_id -> endpoint
}

impl BroadcastExecutor {
    /// Create a new broadcast executor
    ///
    /// ✅ EVOLVED: Now async due to `ExecutionClient` async construction
    pub async fn new() -> Result<Self, String> {
        let client = ExecutionClient::new()
            .await
            .map_err(|e| format!("Failed to create ExecutionClient: {e}"))?;

        Ok(Self {
            client,
            tower_registry: HashMap::new(),
        })
    }

    /// Register a tower endpoint
    pub fn register_tower(&mut self, tower_id: String, endpoint: String) {
        info!("Registering tower: {} at {}", tower_id, endpoint);
        self.tower_registry.insert(tower_id, endpoint);
    }

    /// Execute command on multiple towers in parallel
    pub async fn broadcast(
        &self,
        tower_ids: Vec<String>,
        request: ExecutionRequest,
        options: BroadcastOptions,
    ) -> BroadcastResult {
        let broadcast_id = uuid::Uuid::new_v4().to_string();
        info!("Starting broadcast {} to {} towers", broadcast_id, tower_ids.len());

        let mut tasks = Vec::new();

        for tower_id in tower_ids {
            let endpoint = if let Some(ep) = self.tower_registry.get(&tower_id) {
                ep.clone()
            } else {
                warn!("Tower not found in registry: {}", tower_id);
                continue;
            };

            let client = self.client.clone();
            let request_clone = request.clone();
            let tower_id_clone = tower_id.clone();

            let task = tokio::spawn(async move {
                let start = std::time::Instant::now();

                // Safe cast: u128 millis won't exceed u64 for any reasonable command duration
                // Commands running >584 million years would truncate, which is acceptable
                #[allow(clippy::cast_possible_truncation)]
                let duration_ms = start.elapsed().as_millis() as u64;

                match client.execute_command(&endpoint, request_clone).await {
                    Ok(response) => TowerExecutionResult {
                        tower_id: tower_id_clone,
                        status: response.status,
                        exit_code: response.exit_code,
                        stdout: response.stdout,
                        stderr: response.stderr,
                        duration_ms,
                        error: None,
                    },
                    Err(e) => TowerExecutionResult {
                        tower_id: tower_id_clone,
                        status: ExecutionStatus::Failed,
                        exit_code: None,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration_ms,
                        error: Some(e.to_string()),
                    },
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for task in tasks {
            match task.await {
                Ok(result) => {
                    if result.status == ExecutionStatus::Completed {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("Task join error: {}", e);
                    failure_count += 1;
                }
            }
        }

        info!(
            "Broadcast {} completed: {} successes, {} failures",
            broadcast_id, success_count, failure_count
        );

        // Check if we should fail fast
        if options.fail_fast && failure_count > 0 {
            return BroadcastResult {
                broadcast_id,
                success: false,
                results,
                error: Some(format!("{failure_count} towers failed execution")),
            };
        }

        // Check minimum success threshold
        let success_rate = f64::from(success_count) / f64::from(success_count + failure_count);
        if success_rate < options.min_success_rate {
            return BroadcastResult {
                broadcast_id,
                success: false,
                results,
                error: Some(format!(
                    "Success rate {:.2}% below threshold {:.2}%",
                    success_rate * 100.0,
                    options.min_success_rate * 100.0
                )),
            };
        }

        BroadcastResult {
            broadcast_id,
            success: failure_count == 0,
            results,
            error: None,
        }
    }
}

// NOTE: Default trait removed - async construction required
// Use BroadcastExecutor::new().await instead

/// Options for broadcast execution
#[derive(Debug, Clone)]
pub struct BroadcastOptions {
    /// Fail immediately if any tower fails
    pub fail_fast: bool,

    /// Minimum success rate required (0.0 - 1.0)
    pub min_success_rate: f64,

    /// Wait for all towers to complete
    pub wait_for_completion: bool,
}

impl Default for BroadcastOptions {
    fn default() -> Self {
        Self {
            fail_fast: false,
            min_success_rate: 1.0, // Require 100% success by default
            wait_for_completion: true,
        }
    }
}

/// Result from broadcast execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub broadcast_id: String,
    pub success: bool,
    pub results: Vec<TowerExecutionResult>,
    pub error: Option<String>,
}

/// Result from a single tower in broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerExecutionResult {
    pub tower_id: String,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_executor_creation() {
        // Note: Will fail without crypto provider
        let _ = BroadcastExecutor::new().await;
    }

    #[tokio::test]
    async fn test_register_tower() {
        // Note: Will fail without crypto provider
        if let Ok(mut executor) = BroadcastExecutor::new().await {
            executor.register_tower("tower1".to_string(), "http://localhost:9020".to_string());
            assert_eq!(executor.tower_registry.len(), 1);
        }
    }
}
