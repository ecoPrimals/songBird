// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::canonical_test_framework::{MockService, TestEnvironment};
use songbird_types::{SongbirdError, errors::SongbirdResult};
/// Integration testing utilities
///
/// Provides utilities for end-to-end testing, integration testing)
/// and system-level testing across multiple components.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Integration test context for managing test services
#[derive(Debug)]
pub struct IntegrationTestContext {
    /// Test environment
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    environment: Arc<TestEnvironment>,
    /// Running services
    services: Arc<RwLock<HashMap<String, MockService>>>,
    /// Test configuration
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    config: IntegrationTestConfig,
}

/// Integration test configuration
#[derive(Debug, Clone)]
pub struct IntegrationTestConfig {
    /// Test timeout
    pub timeout: Duration,
    /// Service startup delay
    pub startup_delay: Duration,
    /// Maximum concurrent services
    pub max_services: usize,
}

/// Service status for integration tests
#[derive(Debug, Clone)]
pub enum ServiceStatus {
    /// Service is starting up
    Starting,
    /// Service is running normally
    Running,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service encountered an error
    Error(String),
}

impl IntegrationTestContext {
    /// Create a new integration test context
    #[must_use]
    pub fn new(environment: TestEnvironment) -> Self {
        Self {
            environment: Arc::new(environment),
            services: Arc::new(RwLock::new(HashMap::new())),
            config: IntegrationTestConfig::default(),
        }
    }

    /// Start a test service
    ///
    /// # Errors
    /// Returns an error if the service cannot be started.
    pub async fn start_service(&self, name: &str, port: u16) -> SongbirdResult<()> {
        let service = MockService {
            name: name.to_string(),
            port,
            healthy: true, // Default to healthy
        };

        self.services.write().await.insert(name.to_string(), service);

        tracing::info!("Started test service '{}' on port {}", name, port);
        Ok(())
    }

    /// Stop a test service
    ///
    /// # Errors
    /// Returns an error if the service cannot be stopped.
    pub async fn stop_service(&self, name: &str) -> SongbirdResult<()> {
        if let Some(service) = self.services.write().await.get_mut(name) {
            service.healthy = false; // Mark as unhealthy when stopped
            tracing::info!("Stopped test service '{}'", name);
        }
        Ok(())
    }

    /// Get service status
    ///
    /// # Errors
    /// Returns an error if the service is not found.
    pub async fn get_service_status(&self, name: &str) -> SongbirdResult<ServiceStatus> {
        self.services.read().await.get(name).map_or_else(
            || Err(SongbirdError::service("test-utils", format!("Service '{name}' not found"))),
            |service| {
                let status = if service.healthy {
                    ServiceStatus::Running
                } else {
                    ServiceStatus::Stopped
                };
                Ok(status)
            },
        )
    }

    /// Create a mock service for testing
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    fn create_mock_service(name: &str, port: u16) -> MockService {
        MockService {
            name: name.to_string(),
            port,
            healthy: true,
        }
    }
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            startup_delay: Duration::from_millis(100),
            max_services: 10,
        }
    }
}
