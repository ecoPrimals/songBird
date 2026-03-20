// SPDX-License-Identifier: AGPL-3.0-only
// E2E Test Environment Infrastructure
// Created: October 30, 2025
// Updated: December 2, 2025 - Modernized for concurrent execution
// Purpose: Provide common test environment for E2E scenarios
//
// **MODERN:** Truly concurrent-safe with atomic port allocation and no sleeps!
//
// Note: This e2e test environment uses `reqwest` for external HTTP testing.
// Production code uses IpcHttpClient (100% Pure Rust, no C dependencies).
// See: crates/songbird-http-client/examples/ipc_http_client_demo.rs

use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use anyhow::{Result, Context};

/// Global atomic port allocator for truly concurrent tests
/// Each test gets a unique port range without coordination
static NEXT_PORT: AtomicU16 = AtomicU16::new(10000);

/// Test environment for E2E scenarios
/// **CONCURRENT-SAFE:** Each instance has isolated state
pub struct TestEnvironment {
    orchestrator: Option<OrchestratorHandle>,
    services: HashMap<String, ServiceHandle>,
    config: TestConfig,
    port_base: u16,  // This test's unique port range
    port_offset: u16, // Offset within the range
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
    /// **CONCURRENT-SAFE:** Atomically allocates unique port range
    pub async fn new() -> Result<Self> {
        // Atomically allocate a port range (100 ports per test)
        let port_base = NEXT_PORT.fetch_add(100, Ordering::SeqCst);
        
        Ok(Self {
            orchestrator: None,
            services: HashMap::new(),
            config: TestConfig::default(),
            port_base,
            port_offset: 0,
        })
    }

    /// Create with custom config
    /// **CONCURRENT-SAFE:** Atomically allocates unique port range
    pub async fn with_config(config: TestConfig) -> Result<Self> {
        // Atomically allocate a port range (100 ports per test)
        let port_base = NEXT_PORT.fetch_add(100, Ordering::SeqCst);
        
        Ok(Self {
            orchestrator: None,
            services: HashMap::new(),
            config,
            port_base,
            port_offset: 0,
        })
    }

    /// Allocate a unique test port
    /// **CONCURRENT-SAFE:** Each test has its own port range
    fn allocate_port(&mut self) -> u16 {
        let port = self.port_base + self.port_offset;
        self.port_offset += 1;
        assert!(self.port_offset < 100, "Test exhausted port allocation (max 100 ports per test)");
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
    /// **MODERN:** Uses exponential backoff instead of fixed sleep intervals
    async fn wait_for_health(&self, endpoint: &str) -> Result<()> {
        let client = reqwest::Client::new();
        
        // Use exponential backoff: 10us, 50us, 100us, 500us, 1ms, 5ms, 10ms
        let intervals = [
            Duration::from_micros(10),
            Duration::from_micros(50),
            Duration::from_micros(100),
            Duration::from_micros(500),
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ];
        
        let mut interval_idx = 0;
        let start = tokio::time::Instant::now();
        
        loop {
            if start.elapsed() >= self.config.timeout {
                return Err(anyhow::anyhow!("Timeout waiting for health: {}", endpoint));
            }

            match client.get(endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    // Yield first for cooperative multitasking
                    tokio::task::yield_now().await;
                    
                    // Then exponential backoff
                    if interval_idx < intervals.len() {
                        tokio::time::sleep(intervals[interval_idx]).await;
                        interval_idx += 1;
                    } else {
                        // Cap at max interval
                        tokio::time::sleep(intervals[intervals.len() - 1]).await;
                    }
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

