// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Service lifecycle helpers for testing
//!
//! Provides utilities for starting, stopping, and managing test services.

use songbird_types::HealthStatus;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock service configuration
#[derive(Debug, Clone)]
pub struct MockServiceConfig {
    /// Service name
    pub name: String,
    /// Capabilities provided
    pub capabilities: Vec<String>,
    /// Initial health status
    pub initial_health: HealthStatus,
    /// Whether to auto-respond to health checks
    pub auto_health_check: bool,
}

impl MockServiceConfig {
    /// Create a new mock service configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capabilities: Vec::new(),
            initial_health: HealthStatus::Healthy,
            auto_health_check: true,
        }
    }

    /// Add a capability to the service
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set the initial health status
    pub fn with_health(mut self, health: HealthStatus) -> Self {
        self.initial_health = health;
        self
    }
}

/// Service helper for managing test services
pub struct ServiceHelper {
    name: String,
    port: u16,
    health: Arc<RwLock<HealthStatus>>,
    is_mock: bool,
}

impl ServiceHelper {
    /// Create a new mock service helper
    pub fn new_mock(name: &str, config: MockServiceConfig, base_port: u16) -> Self {
        // Calculate port based on service name hash
        let port_offset = name.bytes().fold(0u16, |acc, b| acc.wrapping_add(u16::from(b))) % 1000;
        
        Self {
            name: name.to_string(),
            port: base_port + port_offset,
            health: Arc::new(RwLock::new(config.initial_health)),
            is_mock: true,
        }
    }

    /// Start the service (test mock)
    ///
    /// Note: This is a test helper that simulates service startup.
    /// Real service implementations handle their own lifecycle management.
    pub async fn start(&self) -> Result<(), String> {
        // Mark service as healthy for test purposes
        // Actual implementations start HTTP servers, establish connections, etc.
        *self.health.write().await = HealthStatus::Healthy;
        Ok(())
    }

    /// Stop the service (test mock)
    ///
    /// Note: This is a test helper that simulates service shutdown.
    pub async fn stop(self) -> Result<(), String> {
        // Mark service as stopped for test purposes
        // Actual implementations clean up resources, close connections, etc.
        *self.health.write().await = HealthStatus::Unhealthy;
        Ok(())
    }

    /// Get the current health status
    pub async fn get_health(&self) -> Result<HealthStatus, String> {
        Ok(*self.health.read().await)
    }

    /// Set the health status (for testing)
    pub async fn set_health(&self, status: HealthStatus) {
        *self.health.write().await = status;
    }

    /// Get the service endpoint
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Get the service port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if this is a mock service
    pub fn is_mock(&self) -> bool {
        self.is_mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_service_creation() {
        let config = MockServiceConfig::new("test-service")
            .with_capability("compute")
            .with_health(HealthStatus::Healthy);

        let helper = ServiceHelper::new_mock("test-service", config, 19000);
        assert!(helper.is_mock());
        assert!(helper.port() >= 19000 && helper.port() < 20000);
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let config = MockServiceConfig::new("test-service");
        let helper = ServiceHelper::new_mock("test-service", config, 19000);

        // Start service
        helper.start().await.unwrap();
        assert_eq!(helper.get_health().await.unwrap(), HealthStatus::Healthy);

        // Change health
        helper.set_health(HealthStatus::Degraded).await;
        assert_eq!(helper.get_health().await.unwrap(), HealthStatus::Degraded);

        // Stop service
        helper.stop().await.unwrap();
    }

    #[test]
    fn test_endpoint_generation() {
        let config = MockServiceConfig::new("test-service");
        let helper = ServiceHelper::new_mock("test-service", config, 19000);
        let endpoint = helper.endpoint();
        assert!(endpoint.starts_with("http://localhost:"));
    }
}

