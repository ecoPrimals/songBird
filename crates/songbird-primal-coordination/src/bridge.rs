// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal Bridge - abstraction for connecting to any primal
//!
//! **ZERO HARDCODING**: No primal names, only capability discovery

use crate::error::Result;
use crate::types::{CapabilityType, PrimalCapabilities, PrimalRequest, PrimalResponse};
use async_trait::async_trait;
use songbird_http_client::IpcHttpClient;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connection to a specific primal
///
/// This represents an active connection managed by Songbird's networking layer.
/// It's agnostic to which primal it's connected to - only knows capabilities.
#[derive(Debug, Clone)]
pub struct PrimalConnection {
    /// Connection identifier
    pub connection_id: String,

    /// Endpoint (discovered, not hardcoded) — shared `Arc` avoids copying URLs across clones.
    pub endpoint: Arc<str>,

    /// Capabilities advertised by this primal
    pub capabilities: Arc<RwLock<PrimalCapabilities>>,

    /// Connection metadata
    pub metadata: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
}

impl PrimalConnection {
    /// Create a new primal connection
    #[must_use]
    pub fn new(
        connection_id: String,
        endpoint: impl Into<Arc<str>>,
        capabilities: PrimalCapabilities,
    ) -> Self {
        Self {
            connection_id,
            endpoint: endpoint.into(),
            capabilities: Arc::new(RwLock::new(capabilities)),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Send a request to the primal
    ///
    /// Implements network communication to send requests to remote primals.
    /// Uses HTTP as the primary transport protocol.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Response is invalid
    /// - Timeout occurs
    pub async fn send_request(&self, request: PrimalRequest) -> Result<PrimalResponse> {
        tracing::debug!(
            "Sending request to primal at {} (connection: {})",
            self.endpoint,
            self.connection_id
        );

        // Build HTTP client
        let client = IpcHttpClient::new().await.map_err(|e| {
            crate::error::PrimalCoordinationError::Internal(format!(
                "Failed to create HTTP client: {e}"
            ))
        })?;

        // Determine endpoint path based on request type
        let path = match &request {
            PrimalRequest::DiscoverCapabilities => "/api/v1/capabilities",
            PrimalRequest::Status => "/api/v1/status",
            PrimalRequest::GenerateKeys => "/api/v1/keys/generate",
            PrimalRequest::SignLineage {
                ..
            } => "/api/v1/lineage/sign",
            PrimalRequest::DeployWorkload(_) => "/api/v1/workload/deploy",
            PrimalRequest::Custom {
                ..
            } => "/api/v1/custom",
        };

        let url = format!("{}{}", self.endpoint, path);

        // Send POST request with JSON body
        let response = client.post(&url).await.json(&request)?.send().await.map_err(|e| {
            crate::error::PrimalCoordinationError::Internal(format!("Network request failed: {e}"))
        })?;

        // Check for HTTP errors
        if !response.is_success() {
            return Err(crate::error::PrimalCoordinationError::Internal(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Parse JSON response
        let primal_response: PrimalResponse = response.json().await.map_err(|e| {
            crate::error::PrimalCoordinationError::Internal(format!(
                "Failed to parse response: {e}"
            ))
        })?;

        tracing::debug!("Received response from primal");
        Ok(primal_response)
    }

    /// Check if this connection supports a capability
    pub async fn supports_capability(&self, capability: &CapabilityType) -> bool {
        self.capabilities.read().await.supports_capability(capability)
    }

    /// Get current capabilities
    pub async fn get_capabilities(&self) -> PrimalCapabilities {
        self.capabilities.read().await.clone()
    }

    /// Update capabilities (from periodic discovery)
    pub async fn update_capabilities(&self, new_capabilities: PrimalCapabilities) {
        *self.capabilities.write().await = new_capabilities;
    }
}

/// Primal Bridge trait - defines how to connect to and interact with a primal
///
/// **KEY PRINCIPLE**: This trait is capability-based, not primal-name-based.
/// Implementations discover and connect to primals by capability, not by hardcoded names.
#[async_trait]
pub trait PrimalBridge: Send + Sync {
    /// Connect to a primal that provides the requested capability
    ///
    /// # Errors
    ///
    /// Returns an error if no primal with the capability can be found or connection fails
    async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection>;

    /// Discover capabilities offered by primals in the environment
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    async fn discover_capabilities(
        &self,
        connection: &PrimalConnection,
    ) -> Result<PrimalCapabilities>;

    /// Get the capabilities this bridge can connect to
    ///
    /// Used for routing and capability matching
    fn supported_capabilities(&self) -> Vec<CapabilityType>;
}

/// Discovery-based primal bridge
///
/// Uses `songbird-discovery` to find primals by capability at runtime.
/// **ZERO HARDCODING** - discovers everything from environment/network.
pub struct DiscoveryBasedBridge {
    /// Discovery engine (from songbird-discovery)
    discovery: Arc<dyn PrimalDiscovery>,
}

impl DiscoveryBasedBridge {
    /// Create a new discovery-based bridge
    #[must_use]
    pub fn new(discovery: Arc<dyn PrimalDiscovery>) -> Self {
        Self {
            discovery,
        }
    }
}

#[async_trait]
impl PrimalBridge for DiscoveryBasedBridge {
    async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection> {
        tracing::info!("Discovering primal for capability: {}", capability);

        // Use discovery engine to find a primal
        let discovered = self.discovery.discover_by_capability(&capability).await?;

        tracing::info!("Found primal at {} for capability {}", discovered.endpoint, capability);

        // Create connection
        Ok(PrimalConnection::new(
            uuid::Uuid::new_v4().to_string(),
            discovered.endpoint,
            discovered.capabilities,
        ))
    }

    async fn discover_capabilities(
        &self,
        connection: &PrimalConnection,
    ) -> Result<PrimalCapabilities> {
        // Query the primal for its capabilities
        let response = connection.send_request(PrimalRequest::DiscoverCapabilities).await?;

        match response {
            PrimalResponse::Capabilities(caps) => Ok(caps),
            PrimalResponse::Error(e) => Err(crate::error::PrimalCoordinationError::PrimalError(e)),
            _ => Err(crate::error::PrimalCoordinationError::UnexpectedResponse(
                "Expected Capabilities response".to_string(),
            )),
        }
    }

    fn supported_capabilities(&self) -> Vec<CapabilityType> {
        // Discovery-based bridge supports all capabilities
        vec![
            CapabilityType::Security,
            CapabilityType::Compute,
            CapabilityType::Storage,
            CapabilityType::Ai,
            CapabilityType::Discovery,
            CapabilityType::Orchestration,
            CapabilityType::Networking,
        ]
    }
}

/// Trait for primal discovery (implemented by songbird-discovery)
#[async_trait]
pub trait PrimalDiscovery: Send + Sync {
    /// Discover a primal by capability
    ///
    /// # Errors
    ///
    /// Returns an error if no primal with the capability is found
    async fn discover_by_capability(&self, capability: &CapabilityType)
    -> Result<DiscoveredPrimal>;
}

/// Result of a capability-based discovery lookup.
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    /// Connectable base URL or address for the primal.
    pub endpoint: String,
    /// Advertised capabilities used to build a [`PrimalConnection`].
    pub capabilities: PrimalCapabilities,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::error::PrimalCoordinationError;
    use crate::{PrimalCapabilities, ServiceQuality};
    use std::sync::Arc;

    #[tokio::test(start_paused = true)]
    async fn test_primal_connection_creation() {
        let caps = PrimalCapabilities {
            services: vec!["security".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };

        let conn = PrimalConnection::new(
            "test-conn-1".to_string(),
            "http://localhost:8080".to_string(),
            caps,
        );

        assert_eq!(conn.connection_id, "test-conn-1");
        assert_eq!(&*conn.endpoint, "http://localhost:8080");
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(!conn.supports_capability(&CapabilityType::Compute).await);
    }

    #[tokio::test(start_paused = true)]
    async fn primal_connection_endpoint_accepts_arc_str() {
        let caps = PrimalCapabilities {
            services: vec!["compute".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let ep: Arc<str> = Arc::from("http://example.test/");
        let conn = PrimalConnection::new("id".into(), ep, caps);
        assert_eq!(&*conn.endpoint, "http://example.test/");
        let got = conn.get_capabilities().await;
        assert!(
            got.supports_capability(&CapabilityType::Compute),
            "get_capabilities should clone advertised services"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn send_request_returns_error_without_reachable_ipc_backend() {
        let caps = PrimalCapabilities {
            services: vec!["security".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let conn = PrimalConnection::new("ipc-test".into(), "http://127.0.0.1:9", caps);
        let err = conn
            .send_request(PrimalRequest::Status)
            .await
            .expect_err("IPC HTTP delegation should fail without a Songbird socket/server");
        assert!(
            matches!(err, PrimalCoordinationError::Internal(_)),
            "expected Internal error from client/network path, got {err:?}"
        );
    }

    struct StaticDiscovery {
        endpoint: String,
        caps: PrimalCapabilities,
    }

    #[async_trait::async_trait]
    impl PrimalDiscovery for StaticDiscovery {
        async fn discover_by_capability(
            &self,
            _capability: &CapabilityType,
        ) -> crate::error::Result<DiscoveredPrimal> {
            Ok(DiscoveredPrimal {
                endpoint: self.endpoint.clone(),
                capabilities: self.caps.clone(),
            })
        }
    }

    #[tokio::test(start_paused = true)]
    async fn discovery_based_bridge_connect_and_supported_capabilities() {
        let caps = PrimalCapabilities {
            services: vec!["security".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let bridge = DiscoveryBasedBridge::new(Arc::new(StaticDiscovery {
            endpoint: "http://127.0.0.1:1".into(),
            caps: caps.clone(),
        }));
        let sup = bridge.supported_capabilities();
        assert!(
            sup.contains(&CapabilityType::Security) && sup.contains(&CapabilityType::Compute),
            "discovery bridge advertises core capability set"
        );
        let conn = bridge
            .connect(CapabilityType::Security)
            .await
            .expect("connect should use discovery result");
        assert_eq!(conn.endpoint.as_ref(), "http://127.0.0.1:1");
        let live = conn.get_capabilities().await;
        assert_eq!(live.services, caps.services);
    }

    #[tokio::test(start_paused = true)]
    async fn discovery_based_bridge_discover_capabilities_follows_send_request() {
        let caps = PrimalCapabilities {
            services: vec!["security".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let bridge = DiscoveryBasedBridge::new(Arc::new(StaticDiscovery {
            endpoint: "http://127.0.0.1:9".into(),
            caps,
        }));
        let conn = bridge.connect(CapabilityType::Security).await.expect("connect");
        let err = bridge
            .discover_capabilities(&conn)
            .await
            .expect_err("discover_capabilities delegates to HTTP and should fail without IPC");
        assert!(
            matches!(
                err,
                PrimalCoordinationError::Internal(_) | PrimalCoordinationError::PrimalError(_)
            ),
            "unexpected error variant: {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn test_capability_update() {
        let initial_caps = PrimalCapabilities {
            services: vec!["security".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };

        let conn = PrimalConnection::new(
            "test-conn-2".to_string(),
            "http://localhost:8080".to_string(),
            initial_caps,
        );

        // Initially only supports security
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(!conn.supports_capability(&CapabilityType::Compute).await);

        // Update capabilities
        let new_caps = PrimalCapabilities {
            services: vec!["security".to_string(), "compute".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        conn.update_capabilities(new_caps).await;

        // Now supports both
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(conn.supports_capability(&CapabilityType::Compute).await);
    }

    #[tokio::test(start_paused = true)]
    async fn send_request_generate_keys_errors_without_ipc() {
        let caps = PrimalCapabilities {
            services: vec!["security".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let conn = PrimalConnection::new("gk".into(), "http://127.0.0.1:9/base", caps);
        let err =
            conn.send_request(PrimalRequest::GenerateKeys).await.expect_err("IPC unavailable");
        assert!(
            matches!(err, PrimalCoordinationError::Internal(_)),
            "expected Internal from HTTP client/network path, got {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn send_request_deploy_workload_errors_without_ipc() {
        let caps = PrimalCapabilities {
            services: vec!["compute".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let conn = PrimalConnection::new("dep".into(), "http://127.0.0.1:9/", caps);
        let w = crate::types::Workload {
            id: "w".into(),
            service_type: "compute".into(),
            requirements: std::collections::HashMap::new(),
            payload: serde_json::json!({}),
        };
        let err =
            conn.send_request(PrimalRequest::DeployWorkload(w)).await.expect_err("IPC unavailable");
        assert!(
            matches!(err, PrimalCoordinationError::Internal(_)),
            "expected Internal from client/network, got {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn primal_connection_metadata_starts_empty() {
        let caps = PrimalCapabilities {
            services: vec!["security".into()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        let conn = PrimalConnection::new("m".into(), "http://127.0.0.1/", caps);
        let m = conn.metadata.read().await;
        assert!(m.is_empty());
    }
}
