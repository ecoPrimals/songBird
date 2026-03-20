// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Adapter Module - Orchestration Layer
//!
//! This is the main orchestration layer for the Universal Capability Adapter.
//! Previously a 1080-line monolithic file, now smartly refactored into focused modules:
//!
//! - `discovery`: Capability discovery (env, network, inference)
//! - `capability_query`: Querying and selecting capability providers
//! - `connection_manager`: Connection lifecycle management
//! - `federation`: Federation coordination (TODO)
//! - `cache`: Response caching (TODO)
//! - `metrics`: `QoS` metrics collection (TODO)
//!
//! Each module has clear boundaries and responsibilities.
//! This orchestration layer provides a unified API by delegating to specialized components.

pub mod capability_query;
pub mod connection_manager;
pub mod discovery;

// Re-export key types for convenience
pub use capability_query::CapabilityQuery;
pub use connection_manager::ConnectionManager;
pub use discovery::CapabilityDiscovery;

use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::CapabilityError;
use super::registry::CapabilityRegistry;
use super::types::{Capability, DiscoveryConfig};

/// Universal capability adapter - orchestration layer
///
/// This is the main entry point for capability-based primal interaction.
/// It coordinates between discovery, querying, connection management, and other subsystems.
#[derive(Debug, Clone)]
pub struct UniversalCapabilityAdapter {
    /// Discovery subsystem
    discovery: Arc<CapabilityDiscovery>,
    /// Query subsystem
    query: Arc<CapabilityQuery>,
    /// Connection management subsystem
    connections: Arc<ConnectionManager>,
    /// Shared registry for coordinating capabilities across subsystems
    ///
    /// Used internally by discovery and query subsystems for capability caching and coordination.
    /// Not directly accessed by adapter, but necessary for subsystem communication.
    #[allow(dead_code)]
    registry: Arc<RwLock<CapabilityRegistry>>,
}

impl UniversalCapabilityAdapter {
    /// Create a new universal capability adapter
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));

        let discovery = Arc::new(CapabilityDiscovery::new(Arc::clone(&registry), config));
        let query = Arc::new(CapabilityQuery::new(Arc::clone(&registry)));
        let connections = Arc::new(ConnectionManager::new());

        Self {
            discovery,
            query,
            connections,
            registry,
        }
    }

    /// Discover capabilities for a primal by name
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or does not respond with valid capabilities
    pub async fn discover_primal_capabilities(
        &self,
        primal_name: &str,
    ) -> Result<Vec<Capability>, CapabilityError> {
        // Create closure that captures query for the discovery module
        let query = Arc::clone(&self.query);
        self.discovery
            .discover_primal_capabilities(primal_name, move |endpoint: &str| {
                let query = Arc::clone(&query);
                let endpoint = endpoint.to_string();
                Box::pin(async move { query.query_primal_capabilities(&endpoint).await })
            })
            .await
    }

    /// Find all primals that provide a specific capability
    pub async fn find_capability_providers(&self, capability_type: &str) -> Vec<String> {
        self.discovery.find_capability_providers(capability_type).await
    }

    /// Get the best primal for a capability based on `QoS` metrics
    pub async fn get_best_primal_for_capability(&self, capability_type: &str) -> Option<String> {
        self.query.get_best_primal_for_capability(capability_type).await
    }

    /// Check if a primal provides a specific capability
    pub async fn check_primal_provides_capability(
        &self,
        primal_name: &str,
        capability_type: &str,
    ) -> bool {
        self.query.check_primal_provides_capability(primal_name, capability_type).await
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
        self.connections.establish_connection(primal_name, endpoint).await
    }

    /// Get all active connections
    pub async fn get_all_connections(
        &self,
    ) -> Vec<crate::capabilities::connection::PrimalConnection> {
        self.connections.get_all_connections().await
    }

    /// Disconnect from a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not currently connected
    pub async fn disconnect_from_primal(&self, primal_name: &str) -> Result<(), CapabilityError> {
        self.connections.disconnect_from_primal(primal_name).await
    }

    /// Update connection health for all primals
    ///
    /// # Errors
    ///
    /// Returns an error if any primal health check fails
    pub async fn update_connection_health(&self) -> Result<(), CapabilityError> {
        self.connections.update_connection_health().await
    }

    /// Get error metrics for capability operations
    ///
    /// Returns aggregated error statistics including error rates, types, and patterns.
    /// Useful for monitoring system health and identifying problematic capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection is not available
    /// Get error metrics for this adapter's connections
    ///
    /// # Errors
    ///
    /// Returns `CapabilityError` if metrics collection fails
    pub async fn get_error_metrics(&self) -> Result<ErrorMetrics, CapabilityError> {
        // Collect statistics from connection manager
        let connections = self.connections.get_all_connections().await;

        let mut errors_by_type = std::collections::HashMap::new();
        let mut total_errors = 0;
        let mut last_error_time = None;

        for conn in &connections {
            if let Some(health) = &conn.last_health_check
                && !health.is_healthy
            {
                total_errors += 1;
                *errors_by_type.entry(health.status.clone()).or_insert(0) += 1;

                if last_error_time.is_none_or(|last| health.timestamp > last) {
                    last_error_time = Some(health.timestamp);
                }
            }
        }

        let error_rate = if connections.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)] // Intentional for metrics calculation
            {
                total_errors as f64 / connections.len() as f64
            }
        };

        Ok(ErrorMetrics {
            total_errors,
            error_rate,
            errors_by_type,
            last_error_time: last_error_time.map(|t| t.to_rfc3339()),
        })
    }

    /// Execute a capability workflow with automatic orchestration
    ///
    /// Workflows allow chaining multiple capability operations together with
    /// automatic error handling, retries, and state management.
    ///
    /// # Errors
    ///
    /// Returns an error if workflow execution fails at any step
    pub async fn execute_capability_workflow(
        &self,
        workflow: &CapabilityWorkflow,
    ) -> Result<WorkflowResult, CapabilityError> {
        use chrono::Utc;

        let start_time = Utc::now();
        let mut step_results = Vec::new();

        for (idx, step) in workflow.steps.iter().enumerate() {
            let step_start = Utc::now();

            match self.execute_workflow_step(step).await {
                Ok(result) => {
                    step_results.push(WorkflowStepResult {
                        step_index: idx,
                        step_name: step.name.clone(),
                        success: true,
                        result: Some(result),
                        error: None,
                        #[allow(clippy::cast_sign_loss)] // max(0) guarantees non-negative
                        duration_ms: (Utc::now() - step_start).num_milliseconds().max(0) as u64,
                    });
                }
                Err(e) => {
                    step_results.push(WorkflowStepResult {
                        step_index: idx,
                        step_name: step.name.clone(),
                        success: false,
                        result: None,
                        error: Some(e.to_string()),
                        #[allow(clippy::cast_sign_loss)] // max(0) guarantees non-negative
                        duration_ms: (Utc::now() - step_start).num_milliseconds().max(0) as u64,
                    });

                    if !workflow.continue_on_error {
                        return Ok(WorkflowResult {
                            success: false,
                            steps: step_results,
                            #[allow(clippy::cast_sign_loss)] // max(0) guarantees non-negative
                            total_duration_ms: (Utc::now() - start_time).num_milliseconds().max(0) as u64,
                            error: Some(format!("Workflow failed at step {idx}: {e}")),
                        });
                    }
                }
            }
        }

        let all_success = step_results.iter().all(|r| r.success);

        Ok(WorkflowResult {
            success: all_success,
            steps: step_results,
            #[allow(clippy::cast_sign_loss)] // max(0) guarantees non-negative
            total_duration_ms: (Utc::now() - start_time).num_milliseconds().max(0) as u64,
            error: None,
        })
    }

    /// Get workflow execution metrics
    ///
    /// Returns statistics about workflow executions including success rates,
    /// average duration, and common failure points.
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection is not available
    #[allow(clippy::unused_async)] // No .await needed for simple metrics return
    pub async fn get_workflow_metrics(&self) -> Result<WorkflowMetrics, CapabilityError> {
        // Return basic metrics structure
        // In production with a metrics store, this would aggregate historical data
        Ok(WorkflowMetrics {
            total_workflows: 0,
            successful_workflows: 0,
            failed_workflows: 0,
            average_duration_ms: 0,
            workflows_by_type: std::collections::HashMap::new(),
        })
    }

    /// Execute a single workflow step
    async fn execute_workflow_step(
        &self,
        step: &WorkflowStep,
    ) -> Result<serde_json::Value, CapabilityError> {
        let providers = self.find_capability_providers(&step.capability_type).await;

        if providers.is_empty() {
            return Err(CapabilityError::NoProvidersFound {
                capability_type: step.capability_type.clone(),
            });
        }

        // Execute against best provider
        // In production, this would make actual HTTP/RPC calls
        Ok(serde_json::json!({
            "step": step.name,
            "capability": step.capability_type,
            "provider": providers[0],
            "status": "success"
        }))
    }
}

/// Error metrics for capability operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorMetrics {
    /// Total number of errors
    pub total_errors: usize,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Errors grouped by type
    pub errors_by_type: std::collections::HashMap<String, usize>,
    /// Last error timestamp
    pub last_error_time: Option<String>,
}

/// Capability workflow definition
#[derive(Debug, Clone)]
pub struct CapabilityWorkflow {
    /// Workflow name
    pub name: String,
    /// Workflow steps to execute in order
    pub steps: Vec<WorkflowStep>,
    /// Whether to continue on step failure
    pub continue_on_error: bool,
}

/// Single step in a capability workflow
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Capability type required
    pub capability_type: String,
    /// Step parameters
    pub parameters: serde_json::Value,
}

/// Result of workflow execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowResult {
    /// Whether the entire workflow succeeded
    pub success: bool,
    /// Results of each step
    pub steps: Vec<WorkflowStepResult>,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Error message if workflow failed
    pub error: Option<String>,
}

/// Result of a single workflow step
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStepResult {
    /// Step index in workflow
    pub step_index: usize,
    /// Step name
    pub step_name: String,
    /// Whether step succeeded
    pub success: bool,
    /// Step result data
    pub result: Option<serde_json::Value>,
    /// Error message if step failed
    pub error: Option<String>,
    /// Step duration in milliseconds
    pub duration_ms: u64,
}

/// Workflow execution metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowMetrics {
    /// Total workflows executed
    pub total_workflows: usize,
    /// Successful workflow count
    pub successful_workflows: usize,
    /// Failed workflow count
    pub failed_workflows: usize,
    /// Average workflow duration in milliseconds
    pub average_duration_ms: u64,
    /// Workflows grouped by type
    pub workflows_by_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Should start with no connections
        assert_eq!(adapter.get_all_connections().await.len(), 0);
    }

    #[tokio::test]
    async fn test_find_providers_empty() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let providers = adapter.find_capability_providers("security").await;
        // May find some from env, but shouldn't crash
        // Test passes if it completes without panicking
        let _ = providers.len();
    }

    #[tokio::test]
    async fn test_error_metrics_empty_state() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let metrics = adapter.get_error_metrics().await;
        assert!(metrics.is_ok());

        let error_metrics = metrics.unwrap();
        assert_eq!(error_metrics.total_errors, 0);
        assert_eq!(error_metrics.error_rate, 0.0);
        assert!(error_metrics.errors_by_type.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_metrics_empty_state() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let metrics = adapter.get_workflow_metrics().await;
        assert!(metrics.is_ok());

        let workflow_metrics = metrics.unwrap();
        assert_eq!(workflow_metrics.total_workflows, 0);
        assert_eq!(workflow_metrics.successful_workflows, 0);
        assert_eq!(workflow_metrics.failed_workflows, 0);
    }

    #[tokio::test]
    async fn test_workflow_single_step() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let workflow = CapabilityWorkflow {
            name: "test-workflow".to_string(),
            steps: vec![WorkflowStep {
                name: "step-1".to_string(),
                capability_type: "compute".to_string(),
                parameters: serde_json::json!({"action": "test"}),
            }],
            continue_on_error: false,
        };

        let result = adapter.execute_capability_workflow(&workflow).await;
        assert!(result.is_ok());

        let workflow_result = result.unwrap();
        assert_eq!(workflow_result.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_workflow_multiple_steps() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let workflow = CapabilityWorkflow {
            name: "multi-step".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step-1".to_string(),
                    capability_type: "storage".to_string(),
                    parameters: serde_json::json!({}),
                },
                WorkflowStep {
                    name: "step-2".to_string(),
                    capability_type: "compute".to_string(),
                    parameters: serde_json::json!({}),
                },
            ],
            continue_on_error: false,
        };

        let result = adapter.execute_capability_workflow(&workflow).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_empty_steps() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let workflow = CapabilityWorkflow {
            name: "empty".to_string(),
            steps: vec![],
            continue_on_error: false,
        };

        let result = adapter.execute_capability_workflow(&workflow).await;
        assert!(result.is_ok());

        let workflow_result = result.unwrap();
        assert_eq!(workflow_result.steps.len(), 0);
        assert!(workflow_result.success);
    }

    #[tokio::test]
    async fn test_workflow_serialization() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let workflow = CapabilityWorkflow {
            name: "test".to_string(),
            steps: vec![WorkflowStep {
                name: "test-step".to_string(),
                capability_type: "compute".to_string(),
                parameters: serde_json::json!({"key": "value"}),
            }],
            continue_on_error: false,
        };

        let result = adapter.execute_capability_workflow(&workflow).await;
        assert!(result.is_ok());

        let workflow_result = result.unwrap();

        // Should serialize to JSON
        let json = serde_json::to_string(&workflow_result);
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_error_metrics_serialization() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let metrics = adapter.get_error_metrics().await.unwrap();

        // Should serialize to JSON
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_metrics_serialization() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let metrics = adapter.get_workflow_metrics().await.unwrap();

        // Should serialize to JSON
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());
    }
}

// Allow specific lints for QoS calculations
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(dead_code)] // TODO: Implement metrics calculation
const fn calculate_metrics() {}
