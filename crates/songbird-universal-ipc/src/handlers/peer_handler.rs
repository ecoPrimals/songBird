// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Peer Connection Handler for JSON-RPC
//!
//! Handles `peer.connect` method for direct peer-to-peer connections using UDP hole punching.
//!
//! ## Method
//! - `peer.connect` - Initiate direct connection to peer (hole punching)
//!
//! ## Architecture
//! Uses enum dispatch ([`PeerConnector`]) for production UDP and test doubles.

use crate::error::{IpcError, IpcResult};
use crate::handlers::mesh_handler::MeshHandler;
use serde_json::Value;
use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

pub use super::peer_types::{PeerChannel, PeerConnectParams, PeerConnectResult};

use super::udp_peer_connector::PeerConnector;

// ============================================================================
// Peer Handler
// ============================================================================

pub struct PeerHandler {
    connector: Arc<PeerConnector>,
    mesh_handler: Option<Arc<MeshHandler>>,
}

impl PeerHandler {
    /// Create new handler with given connector (no mesh registration).
    #[must_use]
    pub fn new(connector: Arc<PeerConnector>) -> Self {
        Self {
            connector,
            mesh_handler: None,
        }
    }

    /// Create handler with mesh registration capability.
    #[must_use]
    pub fn with_mesh(connector: Arc<PeerConnector>, mesh_handler: Arc<MeshHandler>) -> Self {
        Self {
            connector,
            mesh_handler: Some(mesh_handler),
        }
    }

    /// Handle peer.connect — connects to peer and optionally registers in mesh.
    pub async fn handle_connect(&self, params: Value) -> IpcResult<PeerConnectResult> {
        let params: PeerConnectParams =
            serde_json::from_value(params).map_err(|e| IpcError::InvalidParams(e.to_string()))?;

        let register_mesh = params.register_mesh.unwrap_or(true);

        info!(
            target_address = %params.target_address,
            node_id = ?params.node_id,
            register_mesh,
            "peer.connect: initiating"
        );

        let result = self
            .connector
            .connect(
                &params.target_address,
                params.our_binding.as_deref(),
                params.rendezvous_token.as_deref(),
            )
            .await
            .map_err(|e| IpcError::Internal(format!("Peer connection failed: {e}")))?;

        let mut node_id = params.node_id;
        let mut mesh_registered = false;

        if result.state == "connected"
            && register_mesh
            && let Some(ref mesh_handler) = self.mesh_handler
        {
            let reg_result = self
                .register_in_mesh(mesh_handler, &params.target_address, node_id.as_deref())
                .await;
            match reg_result {
                Ok(discovered_id) => {
                    node_id = Some(discovered_id);
                    mesh_registered = true;
                    info!(
                        target_address = %params.target_address,
                        node_id = ?node_id,
                        "peer.connect: registered in mesh"
                    );
                }
                Err(e) => {
                    debug!(
                        target_address = %params.target_address,
                        error = %e,
                        "peer.connect: mesh registration failed (peer still connected)"
                    );
                }
            }
        }

        match result.state.as_str() {
            "connected" => info!(connection_id = %result.connection_id, "peer connected"),
            "connecting" => info!(connection_id = %result.connection_id, "peer connecting"),
            "failed" => warn!(connection_id = %result.connection_id, "peer connection failed"),
            other => warn!(state = other, "peer unknown state"),
        }

        Ok(PeerConnectResult {
            connection_id: result.connection_id,
            state: result.state,
            channel: result.channel,
            node_id,
            mesh_registered,
        })
    }

    /// Perform federation probe and register peer in mesh.
    ///
    /// If `node_id` is provided, uses it directly. Otherwise discovers it via
    /// health.ping federation probe.
    async fn register_in_mesh(
        &self,
        mesh_handler: &MeshHandler,
        target_address: &str,
        provided_node_id: Option<&str>,
    ) -> Result<String, String> {
        let addr: std::net::SocketAddr =
            target_address.parse().map_err(|e| format!("invalid address: {e}"))?;

        // Discover node_id via federation probe if not provided
        let node_id = if let Some(id) = provided_node_id {
            id.to_string()
        } else {
            let probe_result = MeshHandler::probe_peer_full(addr, Duration::from_secs(5))
                .await
                .map_err(|e| format!("federation probe failed: {e}"))?;

            // Try to get node_id from identity.get
            let discovered =
                self.discover_node_id(addr).await.unwrap_or_else(|| format!("peer-{}", addr.ip()));

            debug!(
                addr = %addr,
                node_id = %discovered,
                version = ?probe_result.version,
                "peer.connect: federation probe succeeded"
            );
            discovered
        };

        // Register in mesh if initialized
        let mesh_guard = mesh_handler.mesh.read().await;
        let mesh = mesh_guard.as_ref().ok_or_else(|| String::from("mesh not initialized"))?;

        let endpoint = RelayEndpoint {
            node_id: node_id.clone(),
            endpoint_type: EndpointType::Direct {
                addr,
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: true,
        };

        mesh.add_endpoint(node_id.clone(), endpoint).await;
        Ok(node_id)
    }

    /// Attempt to discover `node_id` via `identity.get` RPC.
    async fn discover_node_id(&self, addr: std::net::SocketAddr) -> Option<String> {
        use songbird_types::constants::ribocipher;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let stream = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr))
            .await
            .ok()?
            .ok()?;

        let (reader, mut writer) = stream.into_split();

        writer.write_all(&ribocipher::MITO_PREFIX).await.ok()?;
        writer
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"identity.get\",\"id\":2}\n")
            .await
            .ok()?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        tokio::time::timeout(Duration::from_secs(3), buf_reader.read_line(&mut response))
            .await
            .ok()?
            .ok()?;

        let val: serde_json::Value = serde_json::from_str(&response).ok()?;
        // Try common identity response fields
        val["result"]["node_id"]
            .as_str()
            .or_else(|| val["result"]["primal"].as_str())
            .map(String::from)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::handlers::udp_peer_connector::MockPeerConnector;
    use serde_json::json;

    #[tokio::test]
    async fn test_connect_success() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());
        assert!(result.channel.is_some());

        let channel = result.channel.unwrap();
        assert_eq!(channel.remote_address, "203.0.113.45:54321");
        assert_eq!(channel.protocol, "udp");
        assert!(channel.latency_ms.is_some());
    }

    #[tokio::test]
    async fn test_connect_with_binding() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321",
            "our_binding": "0.0.0.0:5000"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        let channel = result.channel.unwrap();
        assert_eq!(channel.local_address, "0.0.0.0:5000");
    }

    #[tokio::test]
    async fn test_connect_with_rendezvous_token() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321",
            "rendezvous_token": "token-abc123"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());
    }

    #[tokio::test]
    async fn test_connect_failure() {
        let inner = MockPeerConnector::new();
        inner.set_should_succeed(false);
        let connector = Arc::new(PeerConnector::Mock(inner));

        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "failed");
        assert!(result.channel.is_none());
    }

    #[tokio::test]
    async fn test_connect_missing_params() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            // Missing target_address
        });

        let result = handler.handle_connect(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_all_params() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.100:6000",
            "our_binding": "192.168.1.10:5000",
            "rendezvous_token": "token-xyz789"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());

        let channel = result.channel.unwrap();
        assert_eq!(channel.local_address, "192.168.1.10:5000");
        assert_eq!(channel.remote_address, "203.0.113.100:6000");
    }

    #[test]
    fn peer_connect_params_json_roundtrip() {
        let p = PeerConnectParams {
            target_address: "198.51.100.2:4000".into(),
            our_binding: Some("0.0.0.0:5000".into()),
            rendezvous_token: Some("tok".into()),
            node_id: None,
            register_mesh: None,
        };
        let v = serde_json::to_value(&p).expect("serialize");
        let back: PeerConnectParams = serde_json::from_value(v).expect("deserialize");
        assert_eq!(back.target_address, p.target_address);
        assert_eq!(back.our_binding, p.our_binding);
        assert_eq!(back.rendezvous_token, p.rendezvous_token);
    }

    #[tokio::test]
    async fn connect_propagates_connector_error() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::ErrorSim));
        let err = handler
            .handle_connect(json!({ "target_address": "127.0.0.1:1" }))
            .await
            .expect_err("connector error");
        match err {
            IpcError::Internal(msg) => assert!(msg.contains("simulated")),
            _ => panic!("expected Internal error"),
        }
    }

    #[tokio::test]
    async fn connect_invalid_params_type_errors() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let err = handler
            .handle_connect(json!({ "target_address": 12345 }))
            .await
            .expect_err("type mismatch");
        assert!(matches!(err, IpcError::InvalidParams(_)));
    }

    #[test]
    fn peer_connect_result_serialization_shape() {
        let r = PeerConnectResult {
            connection_id: "cid".into(),
            state: "connecting".into(),
            channel: None,
            node_id: None,
            mesh_registered: false,
        };
        let v = serde_json::to_value(&r).expect("json");
        assert_eq!(v["state"], "connecting");
        assert!(v["channel"].is_null());
    }

    #[test]
    fn peer_channel_serialization_roundtrip() {
        let c = PeerChannel {
            local_address: "a".into(),
            remote_address: "b".into(),
            protocol: "udp".into(),
            latency_ms: Some(1),
        };
        let v = serde_json::to_value(&c).expect("ser");
        assert_eq!(v["protocol"], "udp");
    }

    #[tokio::test]
    async fn connect_accepts_optional_null_fields() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let params = json!({
            "target_address": "198.51.100.1:4000",
            "our_binding": null,
            "rendezvous_token": null
        });
        let r = handler.handle_connect(params).await.expect("ok");
        assert_eq!(r.state, "connected");
    }

    #[tokio::test]
    async fn connect_unknown_state_still_returns_ok() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Weird));
        let r = handler.handle_connect(json!({ "target_address": "1.1.1.1:1" })).await.expect("ok");
        assert_eq!(r.state, "negotiating");
    }

    #[tokio::test]
    async fn target_address_accepts_ipv6_bracket_form() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let r = handler
            .handle_connect(json!({ "target_address": "[2001:db8::1]:5000" }))
            .await
            .expect("ok");
        assert_eq!(r.channel.expect("ch").remote_address, "[2001:db8::1]:5000");
    }
}
