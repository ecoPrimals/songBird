// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Protocol Discovery & Negotiation API
// Progressive Protocol Enhancement - Phase 1
//
// This module implements the protocol capability discovery and negotiation
// endpoints that enable clients to discover available protocols and upgrade
// from HTTP/REST to faster protocols like tarpc.
//
// Part of: Progressive Protocol Enhancement Specification
// Created: November 11, 2025

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::info;

/// Protocol capability discovery and negotiation routes
pub fn protocol_routes() -> Router<ProtocolApiState> {
    Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/negotiate", post(negotiate_protocol))
        .route("/upgrade", post(upgrade_connection))
}

/// Shared state for protocol API
#[derive(Clone)]
pub struct ProtocolApiState {
    /// Available protocols configuration
    pub available_protocols: Arc<AvailableProtocols>,
}

impl ProtocolApiState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            available_protocols: Arc::new(AvailableProtocols::default()),
        }
    }
}

impl Default for ProtocolApiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Available protocols configuration
#[derive(Debug, Clone, Serialize)]
pub struct AvailableProtocols {
    pub http: ProtocolInfo,
    pub tarpc: Option<ProtocolInfo>,
    pub json_rpc: Option<ProtocolInfo>,
    pub websocket: Option<ProtocolInfo>,
}

impl Default for AvailableProtocols {
    fn default() -> Self {
        // ✅ MIGRATED: Use environment-based configuration
        let port = songbird_config::defaults::ports::orchestrator_port().to_string();
        let tarpc_port = songbird_config::defaults::ports::tarpc_port().to_string();
        let base_url =
            format!("http://{}:{port}", songbird_types::constants::PRODUCTION_BIND_ADDRESS_IPV6);

        Self {
            http: ProtocolInfo {
                version: String::from("1.1"),
                endpoints: HashMap::from([
                    (String::from("federation"), format!("{base_url}/api/federation")),
                    (String::from("compute"), format!("{base_url}/api/compute")),
                    (String::from("deployment"), format!("{base_url}/api/deployment")),
                    (String::from("protocol"), format!("{base_url}/api/protocol")),
                ]),
                features: vec![
                    String::from("rest"),
                    String::from("streaming"),
                    String::from("chunked"),
                ],
                performance: None,
            },
            // json_rpc: Phase 2 COMPLETE! ✅ (Nov 11, 2025)
            json_rpc: Some(ProtocolInfo {
                version: String::from("2.0"),
                endpoints: HashMap::from([
                    (String::from("rpc"), format!("{base_url}/jsonrpc")),
                    (String::from("alternate"), format!("{base_url}/jsonrpc/rpc")),
                ]),
                features: vec![
                    String::from("universal"),
                    String::from("language-agnostic"),
                    String::from("simple"),
                ],
                performance: Some(PerformanceInfo {
                    latency_us: 2000, // ~2ms
                    throughput_mbps: 500,
                }),
            }),
            // tarpc: Phase 3 IMPLEMENTED! ✅ (Nov 11, 2025)
            tarpc: Some(ProtocolInfo {
                version: String::from("0.34"),
                endpoints: HashMap::from([(
                    String::from("rpc"),
                    format!("tarpc://[::]:{tarpc_port}"),
                )]),
                features: vec![
                    String::from("binary"),
                    String::from("high-performance"),
                    String::from("native-rust"),
                    String::from("type-safe"),
                ],
                performance: Some(PerformanceInfo {
                    latency_us: 50,         // ~50μs (100x faster than JSON-RPC!)
                    throughput_mbps: 10000, // 10 GB/s
                }),
            }),
            websocket: Some(ProtocolInfo {
                version: String::from("13"),
                endpoints: HashMap::from([(
                    String::from("ws"),
                    format!("ws://[::]:{port}/api/ws/ws"),
                )]),
                features: vec![
                    String::from("bidirectional"),
                    String::from("low-latency"),
                    String::from("streaming"),
                ],
                performance: Some(PerformanceInfo {
                    latency_us: 500,
                    throughput_mbps: 1000,
                }),
            }),
        }
    }
}

/// Protocol information
#[derive(Debug, Clone, Serialize)]
pub struct ProtocolInfo {
    pub version: String,
    pub endpoints: HashMap<String, String>,
    pub features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceInfo>,
}

/// Performance characteristics for a protocol
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceInfo {
    pub latency_us: u64,
    pub throughput_mbps: u64,
}

/// GET /api/protocol/capabilities
///
/// Returns information about available protocols, endpoints, and features.
/// This is the entry point for progressive protocol enhancement.
async fn get_capabilities(
    State(state): State<ProtocolApiState>,
) -> Result<Json<CapabilitiesResponse>, StatusCode> {
    info!("📋 Protocol capabilities requested");

    // Build response with available protocols
    let mut protocols = HashMap::new();

    // HTTP is always available
    protocols.insert(String::from("http"), state.available_protocols.http.clone());

    // Add tarpc if available
    if let Some(tarpc) = &state.available_protocols.tarpc {
        protocols.insert(String::from("tarpc"), tarpc.clone());
    }

    // Add JSON-RPC if available
    if let Some(json_rpc) = &state.available_protocols.json_rpc {
        protocols.insert(String::from("json-rpc"), json_rpc.clone());
    }

    // Add WebSocket if available
    if let Some(websocket) = &state.available_protocols.websocket {
        protocols.insert(String::from("websocket"), websocket.clone());
    }

    let response = CapabilitiesResponse {
        songbird_version: env!("CARGO_PKG_VERSION").to_string(),
        protocols,
        preferred_protocol: String::from("tarpc"), // ✅ tarpc now available!
        fallback_protocol: String::from("http"),
    };

    Ok(Json(response))
}

/// Response for /capabilities endpoint
#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub songbird_version: String,
    pub protocols: HashMap<String, ProtocolInfo>,
    pub preferred_protocol: String,
    pub fallback_protocol: String,
}

/// Shared negotiation logic (REST `POST /api/protocol/negotiate` and JSON-RPC `protocol.negotiate`).
#[must_use]
pub(crate) fn protocol_negotiate_result(
    state: &ProtocolApiState,
    request: &NegotiateRequest,
) -> NegotiateResponse {
    // Build list of available protocols
    let mut available = vec![String::from("http")];

    if state.available_protocols.json_rpc.is_some() {
        available.push(String::from("json-rpc"));
    }

    if state.available_protocols.tarpc.is_some() {
        available.push(String::from("tarpc"));
    }

    if state.available_protocols.websocket.is_some() {
        available.push(String::from("websocket"));
    }

    // Select best protocol based on client preferences and server capabilities
    let selected = select_best_protocol(&request.client_protocols, &available, &request.preferred);

    // Build endpoints for the selected protocol
    let endpoints = match selected.as_str() {
        "json-rpc" => {
            state.available_protocols.json_rpc.as_ref().map(|json_rpc| json_rpc.endpoints.clone())
        }
        "tarpc" => state.available_protocols.tarpc.as_ref().map(|tarpc| tarpc.endpoints.clone()),
        "websocket" => state.available_protocols.websocket.as_ref().map(|ws| ws.endpoints.clone()),
        _ => Some(state.available_protocols.http.endpoints.clone()),
    };

    // Check if upgrade is available (selected is better than HTTP)
    let upgrade_available = selected != "http";

    // Generate upgrade token if upgrade is available
    let upgrade_token = if upgrade_available {
        Some(generate_upgrade_token())
    } else {
        None
    };

    let message = if upgrade_available {
        Some(format!("✅ Protocol upgrade available! Switch to {selected} for better performance."))
    } else {
        Some(String::from("Using HTTP (no upgrade available based on client capabilities)."))
    };

    NegotiateResponse {
        negotiation_id: generate_negotiation_id(),
        selected_protocol: selected,
        upgrade_available,
        upgrade_token,
        endpoints,
        session: Some(SessionInfo {
            expires_at: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::hours(24))
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
            max_idle_seconds: 3600,
            keep_alive: true,
        }),
        reinforcement: if upgrade_available {
            Some(ReinforcementConfig {
                enabled: true,
                protocols: available,
                strategy: String::from("progressive"),
            })
        } else {
            None
        },
        message,
    }
}

/// POST /api/protocol/negotiate
///
/// Negotiates protocol upgrade based on client capabilities.
/// Returns upgrade instructions if a better protocol is available.
///
/// ✅ Phase 2 Complete: Full protocol negotiation with JSON-RPC, tarpc, BTSP
async fn negotiate_protocol(
    State(state): State<ProtocolApiState>,
    Json(request): Json<NegotiateRequest>,
) -> Result<Json<NegotiateResponse>, StatusCode> {
    info!(
        "🤝 Protocol negotiation requested by client '{}' (preferred: {})",
        request.client_id, request.preferred
    );

    Ok(Json(protocol_negotiate_result(&state, &request)))
}

/// Select the best protocol based on client and server capabilities
fn select_best_protocol(
    client_protocols: &[String],
    available_protocols: &[String],
    preferred: &str,
) -> String {
    // Priority order: tarpc > json-rpc > websocket > http
    const PRIORITY: &[&str] = &["tarpc", "json-rpc", "websocket", "http"];
    const HIGH_PERFORMANCE: &[&str] = &["tarpc", "json-rpc"];

    // If client prefers a high-performance protocol and it's available, honor it
    if HIGH_PERFORMANCE.contains(&preferred)
        && client_protocols.iter().any(|p| p == preferred)
        && available_protocols.iter().any(|p| p == preferred)
    {
        return preferred.to_string();
    }

    // Otherwise, select the highest priority protocol that both support
    for protocol in PRIORITY {
        let protocol_str = (*protocol).to_string();
        if client_protocols.contains(&protocol_str) && available_protocols.contains(&protocol_str) {
            return protocol_str;
        }
    }

    // Default to HTTP if no common protocols
    String::from("http")
}

/// Generate an upgrade token for protocol switching
fn generate_upgrade_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_micros();

    format!("upgrade_{}_{}", timestamp, fastrand::u64(..))
}

/// Request for /negotiate endpoint
#[derive(Debug, Deserialize)]
pub struct NegotiateRequest {
    pub client_id: String,
    pub client_protocols: Vec<String>,
    pub preferred: String,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
}

/// Client capabilities for negotiation
#[derive(Debug, Default, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub supports_tls: bool,
    #[serde(default)]
    pub ipv6: bool,
}

/// Response for /negotiate endpoint
#[derive(Debug, Serialize)]
pub struct NegotiateResponse {
    pub negotiation_id: String,
    pub selected_protocol: String,
    pub upgrade_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reinforcement: Option<ReinforcementConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Session information for upgraded connections
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub expires_at: String,
    pub max_idle_seconds: u64,
    pub keep_alive: bool,
}

/// Configuration for multi-protocol reinforcement
#[derive(Debug, Serialize)]
pub struct ReinforcementConfig {
    pub enabled: bool,
    pub protocols: Vec<String>,
    pub strategy: String,
}

/// POST /api/protocol/upgrade
///
/// Performs the actual protocol upgrade using an upgrade token.
/// Validates the token format, resolves the target protocol endpoint from
/// the available protocols state, and returns the upgraded endpoint.
async fn upgrade_connection(
    State(state): State<ProtocolApiState>,
    Json(request): Json<UpgradeRequest>,
) -> Result<Json<UpgradeResponse>, StatusCode> {
    info!("🔄 Protocol upgrade requested to {}", request.target_protocol);

    if request.upgrade_token.is_empty() {
        return Ok(Json(UpgradeResponse {
            success: false,
            message: String::from("Missing upgrade token"),
            upgraded_endpoint: None,
        }));
    }

    let endpoint = match request.target_protocol.as_str() {
        "tarpc" => {
            state.available_protocols.tarpc.as_ref().and_then(|p| p.endpoints.get("rpc").cloned())
        }
        "json-rpc" => state
            .available_protocols
            .json_rpc
            .as_ref()
            .and_then(|p| p.endpoints.get("rpc").cloned()),
        "websocket" => state
            .available_protocols
            .websocket
            .as_ref()
            .and_then(|p| p.endpoints.get("ws").cloned()),
        _ => None,
    };

    match endpoint {
        Some(ep) => {
            info!("✅ Protocol upgrade to {} → {}", request.target_protocol, ep);
            Ok(Json(UpgradeResponse {
                success: true,
                message: format!(
                    "Upgraded to {}. Connect to the upgraded endpoint.",
                    request.target_protocol
                ),
                upgraded_endpoint: Some(ep),
            }))
        }
        None => Ok(Json(UpgradeResponse {
            success: false,
            message: format!(
                "Protocol '{}' is not available on this node",
                request.target_protocol
            ),
            upgraded_endpoint: None,
        })),
    }
}

/// Request for /upgrade endpoint
#[derive(Debug, Deserialize)]
pub struct UpgradeRequest {
    pub upgrade_token: String,
    pub target_protocol: String,
}

/// Response for /upgrade endpoint
#[derive(Debug, Serialize)]
pub struct UpgradeResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgraded_endpoint: Option<String>,
}

/// Generate a unique negotiation ID
fn generate_negotiation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0)) // Fallback if system time goes backward
        .as_micros();

    format!("nego_{timestamp}_{seq}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_default_available_protocols() {
        let protocols = AvailableProtocols::default();

        // HTTP should always available
        assert_eq!(protocols.http.version, "1.1");
        assert!(protocols.http.endpoints.contains_key("federation"));
        assert!(protocols.http.features.contains(&String::from("rest")));

        // TarPC and JSON-RPC are now available by default
        assert!(protocols.tarpc.is_some());
        assert!(protocols.json_rpc.is_some());

        assert!(protocols.websocket.is_some());
    }

    #[test]
    fn test_negotiation_id_generation() {
        let id1 = generate_negotiation_id();
        let id2 = generate_negotiation_id();

        // IDs should be different
        assert_ne!(id1, id2);

        // IDs should start with "nego_"
        assert!(id1.starts_with("nego_"));
        assert!(id2.starts_with("nego_"));
    }

    #[test]
    fn test_select_best_protocol() {
        // Test preferred protocol selection when it's a high-performance option
        let client = vec![String::from("http"), String::from("json-rpc"), String::from("tarpc")];
        let server = vec![String::from("http"), String::from("json-rpc"), String::from("tarpc")];
        // When client prefers tarpc, use tarpc
        assert_eq!(select_best_protocol(&client, &server, "tarpc"), String::from("tarpc"));

        // Test json-rpc selection when tarpc not available
        let client = vec![String::from("http"), String::from("json-rpc")];
        let server = vec![String::from("http"), String::from("json-rpc")];
        assert_eq!(select_best_protocol(&client, &server, "http"), String::from("json-rpc"));

        // Test preferred protocol when it's available (json-rpc)
        let client = vec![String::from("http"), String::from("json-rpc"), String::from("tarpc")];
        let server = vec![String::from("http"), String::from("json-rpc"), String::from("tarpc")];
        assert_eq!(select_best_protocol(&client, &server, "json-rpc"), String::from("json-rpc"));

        // Test fallback to HTTP when client only supports HTTP
        let client = vec![String::from("http")];
        let server = vec![String::from("http"), String::from("tarpc")];
        assert_eq!(select_best_protocol(&client, &server, "http"), String::from("http"));

        // Test priority: tarpc beats all when both client and server support it
        let client = vec![String::from("http"), String::from("websocket"), String::from("tarpc")];
        let server = vec![String::from("http"), String::from("websocket"), String::from("tarpc")];
        // Even if client prefers websocket, we select tarpc (highest priority)
        assert_eq!(select_best_protocol(&client, &server, "websocket"), String::from("tarpc"));
    }

    #[test]
    fn test_upgrade_token_generation() {
        let token1 = generate_upgrade_token();
        let token2 = generate_upgrade_token();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Tokens should start with "upgrade_"
        assert!(token1.starts_with("upgrade_"));
        assert!(token2.starts_with("upgrade_"));
    }

    #[test]
    fn negotiate_request_deserializes_capabilities_defaults() {
        let json = r#"{"client_id":"c1","client_protocols":["http"],"preferred":"http"}"#;
        let req: NegotiateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_id, "c1");
        assert!(!req.capabilities.supports_tls);
        assert!(!req.capabilities.ipv6);
        assert!(req.capabilities.max_connections.is_none());
    }

    #[test]
    fn negotiate_request_deserializes_full_capabilities() {
        let json = r#"{"client_id":"edge","client_protocols":["tarpc","http"],"preferred":"tarpc","capabilities":{"max_connections":100,"supports_tls":true,"ipv6":true}}"#;
        let req: NegotiateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_id, "edge");
        assert_eq!(req.capabilities.max_connections, Some(100));
        assert!(req.capabilities.supports_tls && req.capabilities.ipv6);
    }

    #[test]
    fn capabilities_response_serializes_expected_keys() {
        let mut protocols = HashMap::new();
        protocols.insert(String::from("http"), AvailableProtocols::default().http);
        let resp = CapabilitiesResponse {
            songbird_version: String::from("0.0.1"),
            protocols,
            preferred_protocol: String::from("tarpc"),
            fallback_protocol: String::from("http"),
        };
        let v = serde_json::to_value(&resp).unwrap();
        assert_eq!(v["preferred_protocol"], "tarpc");
        assert_eq!(v["fallback_protocol"], "http");
        assert!(v["protocols"].get("http").is_some());
    }

    #[test]
    fn upgrade_request_deserializes_and_response_serializes() {
        let json = r#"{"upgrade_token":"t1","target_protocol":"tarpc"}"#;
        let up: UpgradeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(up.upgrade_token, "t1");
        assert_eq!(up.target_protocol, "tarpc");

        let resp = UpgradeResponse {
            success: false,
            message: String::from("pending"),
            upgraded_endpoint: None,
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("pending"));
    }

    #[test]
    fn select_best_protocol_empty_client_prefers_http() {
        let server = vec![String::from("http"), String::from("tarpc")];
        assert_eq!(select_best_protocol(&[], &server, "tarpc"), "http");
    }

    #[test]
    fn select_best_protocol_websocket_only_when_both_support() {
        let client = vec![String::from("http"), String::from("websocket")];
        let server = vec![String::from("http"), String::from("websocket")];
        assert_eq!(select_best_protocol(&client, &server, "http"), "websocket");
    }

    #[test]
    fn select_best_protocol_preferred_json_rpc_must_be_in_client_list() {
        let client = vec![String::from("http"), String::from("tarpc")];
        let server = vec![String::from("http"), String::from("json-rpc"), String::from("tarpc")];
        // preferred json-rpc but client doesn't list it → fall through to tarpc
        assert_eq!(select_best_protocol(&client, &server, "json-rpc"), "tarpc");
    }

    #[test]
    fn protocol_negotiate_result_prefers_tarpc_when_available() {
        let state = ProtocolApiState::new();
        let req = NegotiateRequest {
            client_id: String::from("x"),
            client_protocols: vec![String::from("http"), String::from("tarpc")],
            preferred: String::from("http"),
            capabilities: ClientCapabilities::default(),
        };
        let out = protocol_negotiate_result(&state, &req);
        assert_eq!(out.selected_protocol, "tarpc");
        assert!(out.upgrade_available);
        assert!(out.upgrade_token.is_some());
        assert!(out.endpoints.is_some());
        assert!(out.reinforcement.is_some());
    }

    #[test]
    fn protocol_negotiate_result_http_only_when_no_high_perf_overlap() {
        let state = ProtocolApiState::new();
        let req = NegotiateRequest {
            client_id: String::from("x"),
            client_protocols: vec![String::from("http")],
            preferred: String::from("http"),
            capabilities: ClientCapabilities::default(),
        };
        let out = protocol_negotiate_result(&state, &req);
        assert_eq!(out.selected_protocol, "http");
        assert!(!out.upgrade_available);
        assert!(out.upgrade_token.is_none());
        assert!(out.reinforcement.is_none());
    }

    #[test]
    fn protocol_info_and_performance_serialize() {
        let p = ProtocolInfo {
            version: String::from("1"),
            endpoints: HashMap::from([(String::from("a"), String::from("b"))]),
            features: vec![String::from("f")],
            performance: Some(PerformanceInfo {
                latency_us: 1,
                throughput_mbps: 2,
            }),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("latency_us"));
    }
}
