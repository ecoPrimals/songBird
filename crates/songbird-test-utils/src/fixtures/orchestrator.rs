//! Orchestrator Test Fixtures
//!
//! Provides standardized fixtures for testing Songbird's orchestration capabilities.

use crate::mocks::common::{HealthStatus, MockPrimalServer};
use crate::mocks::{MockBearDog, MockNestGate, MockSquirrel, MockToadStool};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Complete test environment with all mocked primals
pub struct OrchestratorTestEnvironment {
    /// Mock `ToadStool` compute primal
    pub toadstool: Arc<RwLock<MockToadStool>>,
    /// Mock `BearDog` security primal
    pub beardog: Arc<RwLock<MockBearDog>>,
    /// Mock `NestGate` storage primal  
    pub nestgate: Arc<RwLock<MockNestGate>>,
    /// Mock Squirrel AI primal
    pub squirrel: Arc<RwLock<MockSquirrel>>,
    /// Cleanup handlers
    cleanup_handlers: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl OrchestratorTestEnvironment {
    /// Create a new test environment with all mocked primals
    pub async fn new() -> Self {
        let mut toadstool = MockToadStool::new();
        let mut beardog = MockBearDog::new();
        let mut nestgate = MockNestGate::new();
        let mut squirrel = MockSquirrel::new();

        // Start all mock servers
        let _ = toadstool.start().await;
        let _ = beardog.start().await;
        let _ = nestgate.start().await;
        let _ = squirrel.start().await;

        Self {
            toadstool: Arc::new(RwLock::new(toadstool)),
            beardog: Arc::new(RwLock::new(beardog)),
            nestgate: Arc::new(RwLock::new(nestgate)),
            squirrel: Arc::new(RwLock::new(squirrel)),
            cleanup_handlers: Vec::new(),
        }
    }

    /// Create an environment with only compute capability (`ToadStool`)
    pub async fn with_compute_only() -> Self {
        let env = Self::new().await;

        // Configure only ToadStool as healthy
        let toadstool = env.toadstool.read().await;
        toadstool.simulate_idle();
        drop(toadstool);

        // Set others as unhealthy
        env.beardog.read().await.set_health(HealthStatus::Unhealthy);
        env.nestgate.read().await.set_health(HealthStatus::Unhealthy);
        env.squirrel.read().await.set_health(HealthStatus::Unhealthy);

        env
    }

    /// Create an environment with high load scenario
    pub async fn with_high_load() -> Self {
        let env = Self::new().await;

        // Simulate high load on all primals
        env.toadstool.read().await.simulate_high_load();
        env.beardog.read().await.simulate_security_incident();
        env.nestgate.read().await.simulate_near_capacity();
        env.squirrel.read().await.simulate_high_load();

        env
    }

    /// Create a healthy multi-primal environment
    pub async fn with_healthy_primals() -> Self {
        let env = Self::new().await;

        // Set all primals to healthy state
        env.toadstool.read().await.simulate_idle();
        env.beardog.read().await.simulate_normal_operation();
        env.nestgate.read().await.simulate_healthy_storage();
        env.squirrel.read().await.simulate_normal_operation();

        env
    }

    /// Get `ToadStool` endpoint URL
    pub async fn toadstool_endpoint(&self) -> String {
        let toadstool = self.toadstool.read().await;
        toadstool.endpoint()
    }

    /// Get `BearDog` endpoint URL
    pub async fn beardog_endpoint(&self) -> String {
        let beardog = self.beardog.read().await;
        beardog.endpoint()
    }

    /// Get `NestGate` endpoint URL
    pub async fn nestgate_endpoint(&self) -> String {
        let nestgate = self.nestgate.read().await;
        nestgate.endpoint()
    }

    /// Get Squirrel endpoint URL
    pub async fn squirrel_endpoint(&self) -> String {
        let squirrel = self.squirrel.read().await;
        squirrel.endpoint()
    }

    /// Add a cleanup handler to run when environment is dropped
    pub fn add_cleanup<F>(&mut self, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.cleanup_handlers.push(Box::new(handler));
    }

    /// Cleanup all resources
    pub async fn cleanup(&self) {
        // Stop all mock servers
        self.toadstool.read().await.stop().await;
        self.beardog.read().await.stop().await;
        self.nestgate.read().await.stop().await;
        self.squirrel.read().await.stop().await;

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
        let env = OrchestratorTestEnvironment::new().await;

        // Verify all primals are accessible
        assert!(!env.toadstool_endpoint().await.is_empty());
        assert!(!env.beardog_endpoint().await.is_empty());
        assert!(!env.nestgate_endpoint().await.is_empty());
        assert!(!env.squirrel_endpoint().await.is_empty());

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_healthy_environment() {
        let env = OrchestratorTestEnvironment::with_healthy_primals().await;

        use crate::mocks::common::HealthStatus;
        use crate::mocks::common::MockPrimalServer;

        // All primals should be healthy
        assert_eq!(env.toadstool.read().await.get_health(), HealthStatus::Healthy);
        assert_eq!(env.beardog.read().await.get_health(), HealthStatus::Healthy);
        assert_eq!(env.nestgate.read().await.get_health(), HealthStatus::Healthy);
        assert_eq!(env.squirrel.read().await.get_health(), HealthStatus::Healthy);

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_high_load_environment() {
        let env = OrchestratorTestEnvironment::with_high_load().await;

        use crate::mocks::common::HealthStatus;
        use crate::mocks::common::MockPrimalServer;

        // All primals should be degraded
        assert_eq!(env.toadstool.read().await.get_health(), HealthStatus::Degraded);
        assert_eq!(env.beardog.read().await.get_health(), HealthStatus::Degraded);
        assert_eq!(env.nestgate.read().await.get_health(), HealthStatus::Degraded);
        assert_eq!(env.squirrel.read().await.get_health(), HealthStatus::Degraded);

        env.cleanup().await;
    }
}
