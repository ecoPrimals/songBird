// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! Test environment management for E2E and integration testing
//!
//! Provides a comprehensive test environment that manages service lifecycles,
//! mock services, and cleanup.

use super::{TestConfig, ServiceHelper, MockServiceConfig};
use songbird_config::SongbirdConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Comprehensive test environment for E2E testing
///
/// Manages the lifecycle of services, mocks, and test resources.
/// Automatically cleans up on drop.
pub struct TestEnvironment {
    config: TestConfig,
    songbird_config: SongbirdConfig,
    services: Arc<Mutex<HashMap<String, ServiceHelper>>>,
    // Registry for service discovery (in-memory for tests)
    service_registry: Arc<Mutex<HashMap<String, songbird_types::ServiceInfo>>>,
    cleanup_handlers: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestEnvironment {
    /// Create a new test environment with default configuration
    pub async fn new() -> Self {
        Self::with_config(TestConfig::default()).await
    }

    /// Create a new test environment with custom configuration
    pub async fn with_config(config: TestConfig) -> Self {
        let songbird_config = SongbirdConfig::test_defaults();
        
        let env = Self {
            config,
            songbird_config,
            services: Arc::new(Mutex::new(HashMap::new())),
            service_registry: Arc::new(Mutex::new(HashMap::new())),
            cleanup_handlers: Vec::new(),
        };

        // Initialize base services
        env.initialize().await;

        env
    }

    /// Initialize the test environment
    async fn initialize(&self) {
        // Initialize in-memory registry for test
        // Real orchestrator initialization would happen here in production
        tracing::info!("Test environment initialized with in-memory registry");
    }

    /// Start a mock service with the given configuration
    pub async fn start_mock_service(&mut self, name: &str, config: MockServiceConfig) -> Result<(), String> {
        let mut services = self.services.lock().await;
        
        if services.contains_key(name) {
            return Err(format!("Service '{}' already exists", name));
        }

        let helper = ServiceHelper::new_mock(name, config, self.config.base_port);
        helper.start().await?;
        
        services.insert(name.to_string(), helper);
        Ok(())
    }

    /// Stop a running service
    pub async fn stop_service(&mut self, name: &str) -> Result<(), String> {
        let mut services = self.services.lock().await;
        
        if let Some(helper) = services.remove(name) {
            helper.stop().await?;
            Ok(())
        } else {
            Err(format!("Service '{}' not found", name))
        }
    }

    /// Register a service for discovery
    pub async fn register_service(&self, service_info: songbird_types::ServiceInfo) -> Result<(), String> {
        // Validate service info
        if service_info.name.is_empty() {
            return Err("Service name cannot be empty".to_string());
        }
        
        // Register in our in-memory registry (simulating real registry)
        let mut registry = self.service_registry.lock().await;
        registry.insert(service_info.name.clone(), service_info.clone());
        
        tracing::debug!("Registered service: {} with {} capabilities", 
                       service_info.name, service_info.capabilities.len());
        
        Ok(())
    }

    /// Query for services by capability
    pub async fn discover_services(&self, capability: &str) -> Result<Vec<songbird_types::ServiceInfo>, String> {
        // Query our in-memory registry (simulating real registry)
        let registry = self.service_registry.lock().await;
        
        let matching_services: Vec<songbird_types::ServiceInfo> = registry
            .values()
            .filter(|service| service.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect();
        
        tracing::debug!("Discovered {} services for capability '{}'", 
                       matching_services.len(), capability);
        
        Ok(matching_services)
    }

    /// Get the health status of a service
    pub async fn get_service_health(&self, name: &str) -> Result<songbird_types::HealthStatus, String> {
        let services = self.services.lock().await;
        
        if let Some(helper) = services.get(name) {
            helper.get_health().await
        } else {
            Err(format!("Service '{}' not found", name))
        }
    }

    /// Wait for a service to become healthy
    pub async fn wait_for_healthy(&self, name: &str, timeout_secs: u64) -> Result<(), String> {
        let start = tokio::time::Instant::now();
        let timeout = tokio::time::Duration::from_secs(timeout_secs);

        loop {
            match self.get_service_health(name).await {
                Ok(songbird_types::HealthStatus::Healthy) => return Ok(()),
                Ok(_) => {
                    // Not healthy yet, continue waiting
                }
                Err(e) => return Err(e),
            }

            if start.elapsed() > timeout {
                return Err(format!("Timeout waiting for service '{}' to become healthy", name));
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Add a cleanup handler to run on drop
    pub fn add_cleanup<F>(&mut self, handler: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_handlers.push(Box::new(handler));
    }

    /// Get the base port for services
    pub fn base_port(&self) -> u16 {
        self.config.base_port
    }

    /// Get a service endpoint URL
    pub fn get_endpoint(&self, name: &str, port_offset: u16) -> String {
        format!("http://localhost:{}", self.config.base_port + port_offset)
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.config.cleanup {
            return;
        }

        // Run cleanup handlers
        for handler in self.cleanup_handlers.drain(..) {
            handler();
        }

        // Note: Async cleanup in Drop is complex, services will clean up when their
        // ServiceHelper instances are dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_creation() {
        let env = TestEnvironment::new().await;
        assert_eq!(env.base_port(), 19000);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = TestConfig {
            base_port: 20000,
            ..Default::default()
        };
        let env = TestEnvironment::with_config(config).await;
        assert_eq!(env.base_port(), 20000);
    }

    #[tokio::test]
    async fn test_endpoint_generation() {
        let env = TestEnvironment::new().await;
        let endpoint = env.get_endpoint("test-service", 10);
        assert_eq!(endpoint, "http://localhost:19010");
    }
}

