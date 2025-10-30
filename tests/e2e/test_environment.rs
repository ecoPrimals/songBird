// E2E Test Environment Infrastructure
// Created: October 30, 2025
// Purpose: Provide common test environment for E2E scenarios

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, Context};

/// Test environment for E2E scenarios
pub struct TestEnvironment {
    orchestrator: Option<OrchestratorHandle>,
    services: HashMap<String, ServiceHandle>,
    config: TestConfig,
    allocated_ports: Vec<u16>,
}

/// Handle to a running orchestrator instance
pub struct OrchestratorHandle {
    pub port: u16,
    pub health_endpoint: String,
    process: Option<Child>,
}

/// Handle to a running service instance
pub struct ServiceHandle {
    pub name: String,
    pub port: u16,
    pub capability: String,
    pub health_endpoint: String,
    process: Option<Child>,
}

/// Test configuration
#[derive(Clone)]
pub struct TestConfig {
    pub base_port: u16,
    pub timeout: Duration,
    pub cleanup_on_drop: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_port: 10000, // Use high ports for tests
            timeout: Duration::from_secs(30),
            cleanup_on_drop: true,
        }
    }
}

impl TestEnvironment {
    /// Create a new test environment
    pub async fn new() -> Result<Self> {
        Ok(Self {
            orchestrator: None,
            services: HashMap::new(),
            config: TestConfig::default(),
            allocated_ports: Vec::new(),
        })
    }

    /// Create with custom config
    pub async fn with_config(config: TestConfig) -> Result<Self> {
        Ok(Self {
            orchestrator: None,
            services: HashMap::new(),
            config,
            allocated_ports: Vec::new(),
        })
    }

    /// Allocate a unique test port
    fn allocate_port(&mut self) -> u16 {
        let port = self.config.base_port + self.allocated_ports.len() as u16;
        self.allocated_ports.push(port);
        port
    }

    /// Start the orchestrator
    pub async fn start_orchestrator(&mut self) -> Result<&OrchestratorHandle> {
        if self.orchestrator.is_some() {
            return Err(anyhow::anyhow!("Orchestrator already started"));
        }

        let port = self.allocate_port();
        let health_endpoint = format!("http://127.0.0.1:{}/health", port);

        // For now, create a handle without spawning actual process
        // In real implementation, this would spawn the orchestrator binary
        let handle = OrchestratorHandle {
            port,
            health_endpoint: health_endpoint.clone(),
            process: None,
        };

        // Wait for orchestrator to be healthy
        self.wait_for_health(&health_endpoint).await?;

        self.orchestrator = Some(handle);
        Ok(self.orchestrator.as_ref().unwrap())
    }

    /// Start a service with given name and capability
    pub async fn start_service(&mut self, name: &str, capability: &str) -> Result<&ServiceHandle> {
        if self.services.contains_key(name) {
            return Err(anyhow::anyhow!("Service {} already started", name));
        }

        let port = self.allocate_port();
        let health_endpoint = format!("http://127.0.0.1:{}/health", port);

        // For now, create a handle without spawning actual process
        // In real implementation, this would spawn the service binary
        let handle = ServiceHandle {
            name: name.to_string(),
            port,
            capability: capability.to_string(),
            health_endpoint: health_endpoint.clone(),
            process: None,
        };

        // Wait for service to be healthy
        self.wait_for_health(&health_endpoint).await?;

        self.services.insert(name.to_string(), handle);
        Ok(self.services.get(name).unwrap())
    }

    /// Stop a running service
    pub async fn stop_service(&mut self, name: &str) -> Result<()> {
        if let Some(mut service) = self.services.remove(name) {
            if let Some(mut process) = service.process.take() {
                process.kill().context("Failed to kill service process")?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service {} not found", name))
        }
    }

    /// Wait for a health endpoint to become healthy
    async fn wait_for_health(&self, endpoint: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let deadline = tokio::time::Instant::now() + self.config.timeout;

        loop {
            if tokio::time::Instant::now() > deadline {
                return Err(anyhow::anyhow!("Timeout waiting for health: {}", endpoint));
            }

            match client.get(endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get orchestrator reference
    pub fn orchestrator(&self) -> Result<&OrchestratorHandle> {
        self.orchestrator.as_ref().ok_or_else(|| anyhow::anyhow!("Orchestrator not started"))
    }

    /// Get service reference
    pub fn service(&self, name: &str) -> Result<&ServiceHandle> {
        self.services.get(name).ok_or_else(|| anyhow::anyhow!("Service {} not found", name))
    }

    /// Cleanup all resources
    pub async fn cleanup(mut self) -> Result<()> {
        // Stop all services
        let service_names: Vec<_> = self.services.keys().cloned().collect();
        for name in service_names {
            let _ = self.stop_service(&name).await;
        }

        // Stop orchestrator
        if let Some(mut orchestrator) = self.orchestrator.take() {
            if let Some(mut process) = orchestrator.process.take() {
                let _ = process.kill();
            }
        }

        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if self.config.cleanup_on_drop {
            // Attempt cleanup of any remaining processes
            for (_, service) in self.services.iter_mut() {
                if let Some(ref mut process) = service.process {
                    let _ = process.kill();
                }
            }

            if let Some(ref mut orchestrator) = self.orchestrator {
                if let Some(ref mut process) = orchestrator.process {
                    let _ = process.kill();
                }
            }
        }
    }
}

impl OrchestratorHandle {
    /// Send a capability request to the orchestrator
    pub async fn request_capability(&self, capability: &str, data: serde_json::Value) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/capability/{}", self.port, capability);
        
        client
            .post(&url)
            .json(&data)
            .send()
            .await
            .context("Failed to send capability request")
    }

    /// Check orchestrator health
    pub async fn health_check(&self) -> Result<bool> {
        let client = reqwest::Client::new();
        match client.get(&self.health_endpoint).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

impl ServiceHandle {
    /// Send a direct request to the service
    pub async fn send_request(&self, path: &str, data: serde_json::Value) -> Result<reqwest::Response> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}{}", self.port, path);
        
        client
            .post(&url)
            .json(&data)
            .send()
            .await
            .context("Failed to send service request")
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<bool> {
        let client = reqwest::Client::new();
        match client.get(&self.health_endpoint).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_creation() {
        let env = TestEnvironment::new().await;
        assert!(env.is_ok());
    }

    #[tokio::test]
    async fn test_port_allocation() {
        let mut env = TestEnvironment::new().await.unwrap();
        let port1 = env.allocate_port();
        let port2 = env.allocate_port();
        
        assert_ne!(port1, port2);
        assert!(port1 >= 10000);
        assert!(port2 >= 10000);
    }
}

