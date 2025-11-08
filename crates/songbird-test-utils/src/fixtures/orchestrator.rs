//! Orchestrator Test Fixtures
//!
//! Provides standardized fixtures for testing Songbird's orchestration capabilities.
//!
//! ## 🍼 MODERNIZED: Now uses capability-based mocks
//!
//! This fixture has been modernized to use `MockCapabilityServer` instead of
//! hardcoded primal-specific mocks, aligning with the zero-hardcoding philosophy.

use crate::mocks::{CapabilityMetrics, CapabilityType, MockCapabilityServer};
use std::collections::HashMap;

/// Complete test environment with all capability-based mocks
///
/// ## Modern Capability-Based Design
///
/// This environment now uses `MockCapabilityServer` instances instead of
/// hardcoded primal names. Tests discover services by capability rather than
/// by primal name.
///
/// ## Example
///
/// ```rust,ignore
/// let env = OrchestratorTestEnvironment::with_healthy_primals().await;
/// let compute_endpoint = env.compute_endpoint().await;
/// let security_endpoint = env.security_endpoint().await;
/// // ... use endpoints in your tests
/// env.cleanup().await;
/// ```
pub struct OrchestratorTestEnvironment {
    /// Mock servers by capability type
    servers: HashMap<CapabilityType, MockCapabilityServer>,
    /// Cleanup handlers
    cleanup_handlers: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl OrchestratorTestEnvironment {
    /// Create a new test environment with all capability mocks
    pub async fn new() -> Self {
        let mut servers = HashMap::new();

        // Create and start mock servers for each capability
        for capability in [
            CapabilityType::Compute,
            CapabilityType::Security,
            CapabilityType::Storage,
            CapabilityType::Ai,
        ] {
            let mut server = MockCapabilityServer::new(capability.clone());
            if let Ok(_port) = server.start().await {
                servers.insert(capability, server);
            }
        }

        Self {
            servers,
            cleanup_handlers: Vec::new(),
        }
    }

    /// Create an environment with only compute capability
    pub async fn with_compute_only() -> Self {
        let mut env = Self::new().await;

        // Set only compute as healthy, others as unhealthy
        if let Some(compute) = env.servers.get_mut(&CapabilityType::Compute) {
            compute.set_healthy(true);
            compute.set_metrics(CapabilityMetrics {
                current_load: 0.3,
                ..Default::default()
            });
        }

        for capability in [CapabilityType::Security, CapabilityType::Storage, CapabilityType::Ai] {
            if let Some(server) = env.servers.get_mut(&capability) {
                server.set_healthy(false);
            }
        }

        env
    }

    /// Create an environment with high load scenario
    pub async fn with_high_load() -> Self {
        let mut env = Self::new().await;

        // Simulate high load on all capabilities
        for server in env.servers.values_mut() {
            server.set_healthy(true);
            server.set_metrics(CapabilityMetrics {
                current_load: 0.85,
                avg_response_time_ms: 250.0,
                success_rate: 0.85,
                ..Default::default()
            });
        }

        env
    }

    /// Create a healthy multi-capability environment
    pub async fn with_healthy_primals() -> Self {
        let mut env = Self::new().await;

        // Set all capabilities to healthy state
        for server in env.servers.values_mut() {
            server.set_healthy(true);
            server.set_metrics(CapabilityMetrics {
                current_load: 0.3,
                avg_response_time_ms: 50.0,
                success_rate: 0.99,
                ..Default::default()
            });
        }

        env
    }

    /// Get compute capability endpoint URL (was toadstool)
    #[must_use]
    pub fn toadstool_endpoint(&self) -> String {
        self.servers
            .get(&CapabilityType::Compute)
            .and_then(MockCapabilityServer::endpoint)
            .unwrap_or_else(|| "http://localhost:0".to_string())
    }

    /// Get security capability endpoint URL (was beardog)
    #[must_use]
    pub fn beardog_endpoint(&self) -> String {
        self.servers
            .get(&CapabilityType::Security)
            .and_then(MockCapabilityServer::endpoint)
            .unwrap_or_else(|| "http://localhost:0".to_string())
    }

    /// Get storage capability endpoint URL (was nestgate)
    #[must_use]
    pub fn nestgate_endpoint(&self) -> String {
        self.servers
            .get(&CapabilityType::Storage)
            .and_then(MockCapabilityServer::endpoint)
            .unwrap_or_else(|| "http://localhost:0".to_string())
    }

    /// Get AI capability endpoint URL (was squirrel)
    #[must_use]
    pub fn squirrel_endpoint(&self) -> String {
        self.servers
            .get(&CapabilityType::Ai)
            .and_then(MockCapabilityServer::endpoint)
            .unwrap_or_else(|| "http://localhost:0".to_string())
    }

    /// Get endpoint by capability type
    #[must_use]
    pub fn capability_endpoint(&self, capability: &CapabilityType) -> Option<String> {
        self.servers.get(capability).and_then(MockCapabilityServer::endpoint)
    }

    /// Get server by capability type (mutable)
    pub fn get_server_mut(
        &mut self,
        capability: &CapabilityType,
    ) -> Option<&mut MockCapabilityServer> {
        self.servers.get_mut(capability)
    }

    /// Get server by capability type (immutable)
    #[must_use]
    pub fn get_server(&self, capability: &CapabilityType) -> Option<&MockCapabilityServer> {
        self.servers.get(capability)
    }

    /// Add a cleanup handler to run when environment is dropped
    pub fn add_cleanup<F>(&mut self, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.cleanup_handlers.push(Box::new(handler));
    }

    /// Cleanup all resources
    pub async fn cleanup(&mut self) {
        // Stop all mock servers
        for server in self.servers.values_mut() {
            server.stop().await;
        }

        // Run custom cleanup handlers
        for handler in &self.cleanup_handlers {
            handler();
        }
    }
}

impl Drop for OrchestratorTestEnvironment {
    fn drop(&mut self) {
        // Run cleanup handlers synchronously
        for handler in &self.cleanup_handlers {
            handler();
        }
    }
}

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[tokio::test]
    async fn test_orchestrator_environment_creation() {
        let mut env = OrchestratorTestEnvironment::new().await;

        // Verify all capabilities are accessible
        assert!(!env.toadstool_endpoint().is_empty());
        assert!(!env.beardog_endpoint().is_empty());
        assert!(!env.nestgate_endpoint().is_empty());
        assert!(!env.squirrel_endpoint().is_empty());

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_healthy_environment() {
        let mut env = OrchestratorTestEnvironment::with_healthy_primals().await;

        // All capabilities should be healthy with good metrics
        for capability in [
            CapabilityType::Compute,
            CapabilityType::Security,
            CapabilityType::Storage,
            CapabilityType::Ai,
        ] {
            if let Some(server) = env.get_server(&capability) {
                let metrics = server.metrics();
                assert!(metrics.current_load < 0.5, "Load should be low");
                assert!(metrics.success_rate > 0.95, "Success rate should be high");
            }
        }

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_high_load_environment() {
        let mut env = OrchestratorTestEnvironment::with_high_load().await;

        // All capabilities should show high load
        for capability in [
            CapabilityType::Compute,
            CapabilityType::Security,
            CapabilityType::Storage,
            CapabilityType::Ai,
        ] {
            if let Some(server) = env.get_server(&capability) {
                let metrics = server.metrics();
                assert!(metrics.current_load > 0.7, "Load should be high");
            }
        }

        env.cleanup().await;
    }
}
