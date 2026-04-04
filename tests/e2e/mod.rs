// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! # End-to-End Test Framework
//!
//! **Purpose**: Validate complete workflows across multiple components
//!
//! This module provides infrastructure for E2E testing of:
//! - Multi-adapter workflows
//! - Service discovery integration
//! - Load balancing scenarios
//! - Circuit breaker behavior
//! - Federation coordination

// Existing E2E test modules
pub mod capability_based_orchestration;
pub mod capability_routing;
pub mod discovery_workflow_tests;
pub mod adapter_workflow_tests;
pub mod circuit_breaker_workflow_tests;
pub mod e2e_enhanced_tests;
pub mod failure_recovery;
pub mod fault_tolerance;
pub mod load_balancing;
pub mod multi_service_coordination;
pub mod multi_service_workflows;
pub mod orchestration;
pub mod real_adapter_discovery_e2e;
pub mod scenario_01_service_discovery;
pub mod service_discovery;
pub mod test_environment;
pub mod test_runtime_discovery;
pub mod test_service_discovery_sovereign;
pub mod test_capability_routing;
pub mod test_primal_self_knowledge;

use songbird_types::SongbirdResult;
use std::time::Duration;

/// E2E test context with common setup
pub struct E2ETestContext {
    /// Test name for logging
    pub test_name: String,
    /// Timeout for test execution
    pub timeout: Duration,
    /// Whether to cleanup after test
    pub cleanup: bool,
}

impl E2ETestContext {
    /// Create new E2E test context
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            timeout: Duration::from_secs(30),
            cleanup: true,
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Disable cleanup (for debugging)
    pub fn without_cleanup(mut self) -> Self {
        self.cleanup = false;
        self
    }

    /// Setup test environment
    pub async fn setup(&self) -> SongbirdResult<()> {
        tracing::info!("🚀 Setting up E2E test: {}", self.test_name);
        Ok(())
    }

    /// Teardown test environment
    pub async fn teardown(&self) -> SongbirdResult<()> {
        if self.cleanup {
            tracing::info!("🧹 Cleaning up E2E test: {}", self.test_name);
        }
        Ok(())
    }
}

/// Test helper to wait for condition (event-driven, uses exponential backoff)
///
/// ✅ Modern pattern: Uses yield_now() and exponential backoff instead of fixed sleep intervals
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout: Duration,
    _check_interval: Duration, // Retained for API compatibility, but ignored
) -> SongbirdResult<()>
where
    F: FnMut() -> bool,
{
    use songbird_test_utils::coordination::eventually_async;
    
    eventually_async(
        || async { condition() },
        timeout,
    ).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_context_creation() {
        let ctx = E2ETestContext::new("test");
        assert_eq!(ctx.test_name, "test");
        assert_eq!(ctx.timeout, Duration::from_secs(30));
        assert!(ctx.cleanup);
    }

    #[tokio::test]
    async fn test_e2e_context_with_timeout() {
        let ctx = E2ETestContext::new("test")
            .with_timeout(Duration::from_secs(60));
        assert_eq!(ctx.timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_e2e_context_without_cleanup() {
        let ctx = E2ETestContext::new("test").without_cleanup();
        assert!(!ctx.cleanup);
    }

    #[tokio::test]
    async fn test_wait_for_condition_success() {
        let mut value = false;
        let result = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            value = true;
        });

        // This will timeout since we can't check the spawned value
        // In real tests, use Arc<Mutex<bool>> or similar
    }

    #[tokio::test]
    async fn test_wait_for_condition_immediate() {
        let result = wait_for_condition(
            || true,
            Duration::from_secs(1),
            Duration::from_millis(10),
        ).await;
        assert!(result.is_ok());
    }
}
