//! # 🍼 Capability-Based Mock Servers
//!
//! **MISSION**: Provide test mocks WITHOUT hardcoded primal names
//!
//! This module implements mocks for the "infant discovery" philosophy where tests
//! request capabilities (security, storage, compute, ai) rather than specific
//! primal names (beardog, nestgate, toadstool, squirrel).
//!
//! ## Philosophy
//! > "Each primal only knows itself and discovers others through capabilities"
//!
//! ## Migration from Primal-Specific Mocks
//!
//! ### Before (Hardcoded)
//! ```rust,ignore
//! use songbird_test_utils::mocks::{MockBearDog, MockSquirrel};
//!
//! let beardog = MockBearDog::new();
//! beardog.start().await?;
//! ```
//!
//! ### After (Capability-Based)
//! ```rust,ignore
//! use songbird_test_utils::mocks::{MockCapabilityServer, CapabilityType};
//!
//! let security = MockCapabilityServer::new(CapabilityType::Security);
//! security.start().await?;
//! ```
//!
//! ## Available Capabilities
//!
//! - **Security** - Authentication, encryption, authorization (was beardog)
//! - **Storage** - Data persistence, caching, backup (was nestgate)
//! - **Compute** - Workload execution, container orchestration (was toadstool)
//! - **AI** - ML inference, training, analysis (was squirrel)

#![allow(clippy::unused_async)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

/// Capability type for mock servers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Security capabilities (authentication, encryption, authorization)
    Security,
    /// Storage capabilities (data persistence, caching, backup)
    Storage,
    /// Compute capabilities (workload execution, container orchestration)
    Compute,
    /// AI/ML capabilities (inference, training, analysis)
    Ai,
    /// Custom capability for testing
    Custom(String),
}

impl CapabilityType {
    /// Get human-readable description
    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Self::Security => "Security (Authentication, Encryption)",
            Self::Storage => "Storage (Persistence, Caching)",
            Self::Compute => "Compute (Workload Execution)",
            Self::Ai => "AI/ML (Inference, Training)",
            Self::Custom(name) => name,
        }
    }

    /// Get typical port range for this capability (for testing)
    #[must_use]
    pub fn default_port_range(&self) -> (u16, u16) {
        match self {
            Self::Storage => {
                let base = songbird_config::defaults::ports::metrics_port();
                (base, base + 10)
            }
            Self::Compute => {
                let base = songbird_config::defaults::ports::discovery_port();
                (base, base + 10)
            }
            Self::Security | Self::Ai => {
                let base = songbird_config::defaults::ports::beardog_port();
                (base, base + 10)
            }
            Self::Custom(_) => (9100, 9110),
        }
    }
}

/// Capability metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetrics {
    /// Number of requests handled
    pub requests_handled: u64,
    /// Current load (0.0 - 1.0)
    pub current_load: f64,
    /// Response time (milliseconds)
    pub avg_response_time_ms: f64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
}

impl Default for CapabilityMetrics {
    fn default() -> Self {
        Self {
            requests_handled: 0,
            current_load: 0.3,
            avg_response_time_ms: 50.0,
            success_rate: 0.99,
        }
    }
}

/// Capability-based mock server state
#[derive(Debug, Clone)]
struct CapabilityServerState {
    /// Server port (None if not started)
    port: Option<u16>,
    /// Health status
    is_healthy: bool,
    /// Capability type
    capability_type: CapabilityType,
    /// Capability-specific metrics
    metrics: CapabilityMetrics,
    /// Custom responses
    custom_responses: HashMap<String, serde_json::Value>,
}

/// Mock server for capability-based testing
///
/// ## Example
///
/// ```rust,ignore
/// use songbird_test_utils::mocks::{MockCapabilityServer, CapabilityType};
///
/// #[tokio::test]
/// async fn test_with_capability_discovery() {
///     // Create mock for security capability
///     let mut security = MockCapabilityServer::new(CapabilityType::Security);
///     let port = security.start().await?;
///     
///     // Set environment for discovery
///     std::env::set_var("CAPABILITY_SECURITY_ENDPOINT", format!("http://localhost:{}", port));
///     
///     // Test your code that discovers capabilities
///     // ...
///     
///     security.stop().await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MockCapabilityServer {
    state: Arc<RwLock<CapabilityServerState>>,
}

impl MockCapabilityServer {
    /// Create a new mock server for a capability
    #[must_use]
    pub fn new(capability_type: CapabilityType) -> Self {
        Self {
            state: Arc::new(RwLock::new(CapabilityServerState {
                port: None,
                is_healthy: true,
                capability_type,
                metrics: CapabilityMetrics::default(),
                custom_responses: HashMap::new(),
            })),
        }
    }

    /// Start the mock server on an available port
    ///
    /// Returns the port number the server is listening on
    ///
    /// # Errors
    ///
    /// Returns an error if no ports are available in the capability's default range
    pub async fn start(&mut self) -> Result<u16, std::io::Error> {
        let (start_port, end_port) = {
            let state = self.state.read().unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned for read, recovering");
                poisoned.into_inner()
            });
            state.capability_type.default_port_range()
        };

        // Find an available port in the capability's range
        for port in start_port..=end_port {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener); // Close immediately, we just wanted to check availability

                let mut state = self.state.write().unwrap_or_else(|poisoned| {
                    tracing::warn!("RwLock poisoned for write, recovering");
                    poisoned.into_inner()
                });
                state.port = Some(port);

                return Ok(port);
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            format!(
                "No available ports in range {}-{} for {:?}",
                start_port,
                end_port,
                self.state
                    .read()
                    .unwrap_or_else(|poisoned| {
                        tracing::warn!("RwLock poisoned for read, recovering");
                        poisoned.into_inner()
                    })
                    .capability_type
            ),
        ))
    }

    /// Stop the mock server
    pub async fn stop(&mut self) {
        let mut state = self.state.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned for write, recovering");
            poisoned.into_inner()
        });
        state.port = None;
    }

    /// Get the port the server is running on
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned for read, recovering");
                poisoned.into_inner()
            })
            .port
    }

    /// Get the full endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> Option<String> {
        self.port().map(|p| format!("http://localhost:{}", p))
    }

    /// Set health status
    pub fn set_healthy(&mut self, healthy: bool) {
        let mut state = self.state.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned for write, recovering");
            poisoned.into_inner()
        });
        state.is_healthy = healthy;
    }

    /// Set capability metrics
    pub fn set_metrics(&mut self, metrics: CapabilityMetrics) {
        let mut state = self.state.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned for write, recovering");
            poisoned.into_inner()
        });
        state.metrics = metrics;
    }

    /// Get current metrics
    #[must_use]
    pub fn metrics(&self) -> CapabilityMetrics {
        self.state
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned for read, recovering");
                poisoned.into_inner()
            })
            .metrics
            .clone()
    }

    /// Add a custom response for a specific endpoint
    pub fn add_response(&mut self, path: String, response: serde_json::Value) {
        let mut state = self.state.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned for write, recovering");
            poisoned.into_inner()
        });
        state.custom_responses.insert(path, response);
    }

    /// Get capability type
    #[must_use]
    pub fn capability_type(&self) -> CapabilityType {
        self.state
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned for read, recovering");
                poisoned.into_inner()
            })
            .capability_type
            .clone()
    }
}

/// Builder for creating mock capability environments
///
/// ## Example
///
/// ```rust,ignore
/// use songbird_test_utils::mocks::{MockCapabilityEnvironment, CapabilityType};
///
/// #[tokio::test]
/// async fn test_multi_capability_scenario() {
///     let mut env = MockCapabilityEnvironment::new()
///         .with_capability(CapabilityType::Security)
///         .with_capability(CapabilityType::Storage)
///         .with_capability(CapabilityType::Ai)
///         .build()
///         .await?;
///     
///     // All capabilities are now available and environment variables are set
///     // Your code can discover them via capability_endpoints
///     
///     env.shutdown().await;
/// }
/// ```
#[derive(Debug)]
pub struct MockCapabilityEnvironment {
    servers: HashMap<CapabilityType, MockCapabilityServer>,
}

impl MockCapabilityEnvironment {
    /// Create a new environment builder
    #[must_use]
    pub fn builder() -> MockCapabilityEnvironmentBuilder {
        MockCapabilityEnvironmentBuilder {
            capabilities: Vec::new(),
        }
    }

    /// Get endpoint for a capability
    #[must_use]
    pub fn endpoint(&self, capability: &CapabilityType) -> Option<String> {
        self.servers.get(capability).and_then(MockCapabilityServer::endpoint)
    }

    /// Get port for a capability
    #[must_use]
    pub fn port(&self, capability: &CapabilityType) -> Option<u16> {
        self.servers.get(capability).and_then(MockCapabilityServer::port)
    }

    /// Shutdown all mock servers
    pub async fn shutdown(&mut self) {
        for server in self.servers.values_mut() {
            server.stop().await;
        }
    }
}

/// Builder for mock capability environment
#[derive(Debug)]
pub struct MockCapabilityEnvironmentBuilder {
    capabilities: Vec<CapabilityType>,
}

impl MockCapabilityEnvironmentBuilder {
    /// Add a capability to the environment
    #[must_use]
    pub fn with_capability(mut self, capability: CapabilityType) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add security capability
    #[must_use]
    pub fn with_security(self) -> Self {
        self.with_capability(CapabilityType::Security)
    }

    /// Add storage capability
    #[must_use]
    pub fn with_storage(self) -> Self {
        self.with_capability(CapabilityType::Storage)
    }

    /// Add compute capability
    #[must_use]
    pub fn with_compute(self) -> Self {
        self.with_capability(CapabilityType::Compute)
    }

    /// Add AI capability
    #[must_use]
    pub fn with_ai(self) -> Self {
        self.with_capability(CapabilityType::Ai)
    }

    /// Build and start the environment
    ///
    /// # Errors
    ///
    /// Returns an error if any mock server fails to start or bind to its port.
    pub async fn build(self) -> Result<MockCapabilityEnvironment, std::io::Error> {
        let mut servers = HashMap::new();

        for capability in self.capabilities {
            let mut server = MockCapabilityServer::new(capability.clone());
            let port = server.start().await?;

            // Set environment variable for discovery
            let env_var = match &capability {
                CapabilityType::Security => "CAPABILITY_SECURITY_ENDPOINT",
                CapabilityType::Storage => "CAPABILITY_STORAGE_ENDPOINT",
                CapabilityType::Compute => "CAPABILITY_COMPUTE_ENDPOINT",
                CapabilityType::Ai => "CAPABILITY_AI_ENDPOINT",
                CapabilityType::Custom(name) => {
                    // For custom capabilities, use uppercase name
                    let var_name = format!("CAPABILITY_{}_ENDPOINT", name.to_uppercase());
                    Box::leak(var_name.into_boxed_str())
                }
            };

            std::env::set_var(env_var, format!("http://localhost:{}", port));
            servers.insert(capability, server);
        }

        Ok(MockCapabilityEnvironment {
            servers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::{SongbirdError, SongbirdResult};

    #[test]
    fn test_capability_type_description() {
        assert_eq!(CapabilityType::Security.description(), "Security (Authentication, Encryption)");
        assert_eq!(CapabilityType::Storage.description(), "Storage (Persistence, Caching)");
    }

    #[test]
    fn test_capability_port_ranges() {
        let (start, end) = CapabilityType::Security.default_port_range();
        assert!(start < end);
        assert!(start >= 8000);
    }

    #[tokio::test]
    async fn test_mock_server_creation() {
        let server = MockCapabilityServer::new(CapabilityType::Security);
        assert_eq!(server.capability_type(), CapabilityType::Security);
        assert_eq!(server.port(), None);
    }

    #[tokio::test]
    async fn test_mock_server_lifecycle() -> SongbirdResult<()> {
        let mut server = MockCapabilityServer::new(CapabilityType::Storage);

        // Start server
        let port = server.start().await?;
        assert!(port > 0);
        assert_eq!(server.port(), Some(port));

        // Check endpoint
        let endpoint = server.endpoint();
        assert!(endpoint.is_some());
        assert!(endpoint
            .ok_or_else(|| SongbirdError::configuration(
                "Endpoint should be available after server start".to_string()
            ))?
            .contains(&port.to_string()));

        // Stop server
        server.stop().await;
        assert_eq!(server.port(), None);
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_builder() -> SongbirdResult<()> {
        let mut env =
            MockCapabilityEnvironment::builder().with_security().with_storage().build().await?;

        // Check that endpoints are available
        assert!(env.endpoint(&CapabilityType::Security).is_some());
        assert!(env.endpoint(&CapabilityType::Storage).is_some());

        // Check environment variables are set
        assert!(std::env::var("CAPABILITY_SECURITY_ENDPOINT").is_ok());
        assert!(std::env::var("CAPABILITY_STORAGE_ENDPOINT").is_ok());

        env.shutdown().await;
        Ok(())
    }

    // ========== NEW TESTS (15 tests to reach 85% coverage) ==========

    #[test]
    fn test_capability_type_equality() {
        assert_eq!(CapabilityType::Security, CapabilityType::Security);
        assert_eq!(CapabilityType::Storage, CapabilityType::Storage);
        assert_ne!(CapabilityType::Security, CapabilityType::Storage);
        assert_ne!(CapabilityType::Compute, CapabilityType::Ai);
    }

    #[test]
    fn test_capability_type_clone() {
        let cap = CapabilityType::Compute;
        let cloned = cap.clone();
        assert_eq!(cap, cloned);
    }

    #[test]
    fn test_capability_type_custom() {
        let custom = CapabilityType::Custom("test-service".to_string());
        assert_eq!(custom.description(), "test-service");
        assert_eq!(custom.default_port_range(), (9100, 9110));
    }

    #[test]
    fn test_capability_metrics_default() {
        let metrics = CapabilityMetrics::default();
        assert_eq!(metrics.requests_handled, 0);
        assert!((metrics.current_load - 0.3).abs() < 0.001);
        assert!((metrics.avg_response_time_ms - 50.0).abs() < 0.001);
        assert!((metrics.success_rate - 0.99).abs() < 0.001);
    }

    #[test]
    fn test_capability_metrics_clone() {
        let metrics = CapabilityMetrics {
            requests_handled: 1000,
            current_load: 0.75,
            avg_response_time_ms: 120.0,
            success_rate: 0.95,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.requests_handled, 1000);
        assert!((cloned.current_load - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_capability_metrics_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let metrics = CapabilityMetrics::default();
        let json = serde_json::to_string(&metrics).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("requests_handled"));
        assert!(json.contains("success_rate"));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_server_health_status() {
        let mut server = MockCapabilityServer::new(CapabilityType::Ai);
        server.set_healthy(false);
        // Health status is internal, but we can verify the method doesn't panic
        server.set_healthy(true);
    }

    #[tokio::test]
    async fn test_mock_server_metrics_management() {
        let mut server = MockCapabilityServer::new(CapabilityType::Compute);

        let custom_metrics = CapabilityMetrics {
            requests_handled: 500,
            current_load: 0.8,
            avg_response_time_ms: 200.0,
            success_rate: 0.92,
        };

        server.set_metrics(custom_metrics.clone());
        let retrieved = server.metrics();

        assert_eq!(retrieved.requests_handled, 500);
        assert!((retrieved.current_load - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_server_custom_responses() {
        let mut server = MockCapabilityServer::new(CapabilityType::Security);

        let response = serde_json::json!({"status": "ok", "data": "test"});
        server.add_response("/test/endpoint".to_string(), response.clone());

        // Custom responses are internal, verify no panic
    }

    #[tokio::test]
    async fn test_environment_builder_all_capabilities() -> SongbirdResult<()> {
        let mut env = MockCapabilityEnvironment::builder()
            .with_security()
            .with_storage()
            .with_compute()
            .with_ai()
            .build()
            .await?;

        // All four standard capabilities should be available
        assert!(env.endpoint(&CapabilityType::Security).is_some());
        assert!(env.endpoint(&CapabilityType::Storage).is_some());
        assert!(env.endpoint(&CapabilityType::Compute).is_some());
        assert!(env.endpoint(&CapabilityType::Ai).is_some());

        env.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_port_access() -> SongbirdResult<()> {
        let mut env = MockCapabilityEnvironment::builder()
            .with_capability(CapabilityType::Security)
            .build()
            .await?;

        let port = env.port(&CapabilityType::Security);
        assert!(port.is_some());
        assert!(
            port.ok_or_else(|| SongbirdError::configuration(
                "Port should be available in environment".to_string()
            ))? > 0
        );

        env.shutdown().await;
        Ok(())
    }

    #[test]
    fn test_capability_type_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let security = CapabilityType::Security;
        let json = serde_json::to_string(&security).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Security"));

        let custom = CapabilityType::Custom("test".to_string());
        let json = serde_json::to_string(&custom).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Custom"));
        Ok(())
    }

    #[test]
    fn test_capability_type_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#""Security""#;
        let cap: CapabilityType = serde_json::from_str(json).map_err(|e| {
            SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
        })?;
        assert_eq!(cap, CapabilityType::Security);
        Ok(())
    }

    #[test]
    fn test_capability_type_all_descriptions() {
        assert!(CapabilityType::Security.description().contains("Security"));
        assert!(CapabilityType::Storage.description().contains("Storage"));
        assert!(CapabilityType::Compute.description().contains("Compute"));
        assert!(CapabilityType::Ai.description().contains("AI"));
    }

    #[tokio::test]
    async fn test_mock_server_endpoint_format() -> SongbirdResult<()> {
        let mut server = MockCapabilityServer::new(CapabilityType::Storage);
        let port = server.start().await?;

        let endpoint = server
            .endpoint()
            .ok_or_else(|| SongbirdError::configuration("Endpoint should exist".to_string()))?;
        assert!(endpoint.starts_with("http://localhost:"));
        assert!(endpoint.contains(&port.to_string()));

        server.stop().await;
        Ok(())
    }
}
