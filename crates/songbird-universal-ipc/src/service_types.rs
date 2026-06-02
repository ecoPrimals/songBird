// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC request/response types for the IPC service wire protocol.
//!
//! These DTOs define the stable API surface for inter-primal communication.
//! Other primals use these types (indirectly, via JSON) when calling Songbird's
//! IPC broker over Unix sockets.
//!
//! For raw wire buffers (serialized JSON-RPC frames, opaque bodies), prefer
//! `Bytes` or `SharedBytes` on hot paths to avoid extra copies and to align
//! with the wateringHole zero-copy IPC convention.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `bytes::Bytes` re-export for zero-copy IPC payload fields and wire buffers.
pub use bytes::Bytes;
/// Reference-counted IPC byte buffer (see [`crate::service_types`] module docs).
pub use songbird_types::SharedBytes;

/// IPC service request parameters for registration
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterParams {
    pub primal_id: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

/// IPC service request parameters for resolution.
///
/// Accepts either `capability` (capability-based routing — **preferred**) or
/// `primal_id`/`name` (identity lookup). Capability-first is the standard:
/// callers should resolve by what a primal *does*, not what it *is*.
///
/// When `capability` is provided it takes precedence. If capability lookup
/// fails, the same string is tried as a primal name (graceful fallback for
/// callers who conflate the two).
///
/// `name` is an alias for `primal_id` — use whichever is more natural.
/// `ipc.resolve_by_name` is a convenience alias that routes here.
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveParams {
    #[serde(default, alias = "name")]
    pub primal_id: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
    /// When `true`, prefer the virtual relay endpoint over the native socket.
    /// Phase 1 (shadow mode): opt-in only. Phase 2 will make this the default.
    #[serde(default, rename = "virtual")]
    pub prefer_virtual: bool,
    /// When `true`, force native endpoint (bypass relay even in Phase 2+).
    #[serde(default)]
    pub native: bool,
}

/// IPC service request parameters for discovery
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoverParams {
    pub capability: String,
}

/// IPC service response for registration
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResult {
    pub virtual_endpoint: String,
    pub registered_at: String,
    /// Ed25519 signature over `signed_payload` (base64, via `BearDog` delegation).
    /// `None` in standalone mode (no `FAMILY_ID` / no crypto provider).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Canonical JSON payload that was signed (for consumer verification).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_payload: Option<String>,
}

/// IPC service response for resolution
#[derive(Debug, Clone, Serialize)]
pub struct ResolveResult {
    /// Bare socket/connect path — the recommended endpoint to connect to.
    /// When relay is active and requested, this points to the virtual relay socket.
    /// Otherwise, it's the native socket path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
    pub virtual_endpoint: String,
    /// Transport-qualified URI (e.g. `unix:///path/to/sock`, `tcp://127.0.0.1:8080`).
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
    /// Whether traffic flows through Songbird's virtual relay.
    pub relay: bool,
    /// Filesystem path to the virtual relay socket (if active).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_socket: Option<String>,
    /// Ed25519 signature from the original registration (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Canonical JSON payload that was signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_payload: Option<String>,
}

/// IPC service response for discovery
#[derive(Debug, Clone, Serialize)]
pub struct DiscoverResult {
    pub providers: Vec<ProviderInfo>,
}

/// Provider information
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub primal_id: String,
    /// Bare socket/connect path (e.g. `/run/user/1000/biomeos/security.sock`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
    pub virtual_endpoint: String,
    /// Transport-qualified URI (e.g. `unix:///path/to/sock`, `tcp://127.0.0.1:8080`).
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
    /// Ed25519 signature from the original registration (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Canonical JSON payload that was signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_payload: Option<String>,
}

/// IPC service response for listing
#[derive(Debug, Clone, Serialize)]
pub struct ListResult {
    pub services: Vec<ServiceInfo>,
}

/// Service information
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub capabilities: Vec<String>,
}

/// `capability.resolve` request — single-step routing by capability.
#[derive(Debug, Clone, Deserialize)]
pub struct CapabilityResolveParams {
    pub capability: String,
}

/// `capability.resolve` response — the best provider endpoint for a capability.
#[derive(Debug, Clone, Serialize)]
pub struct CapabilityResolveResult {
    pub primal_id: String,
    /// Bare socket/connect path (e.g. `/run/user/1000/biomeos/security.sock`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
    pub virtual_endpoint: String,
    /// Transport-qualified URI (e.g. `unix:///path/to/sock`, `tcp://127.0.0.1:8080`).
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
    /// Ed25519 signature from the original registration (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Canonical JSON payload that was signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_payload: Option<String>,
}

/// `capability.call` request — cross-gate capability dispatch.
///
/// Invokes an operation on a capability provider. Songbird resolves the provider
/// locally (same gate) or remotely (via mesh peer's Songbird instance). This is
/// the routing layer that biomeOS uses for multi-gate composition dispatch.
#[derive(Debug, Clone, Deserialize)]
pub struct CapabilityCallParams {
    /// Capability domain to target (e.g. `"crypto"`, `"content"`, `"compute"`).
    pub capability: String,
    /// Operation within the capability (e.g. `"generate_keypair"`, `"sign"`).
    pub operation: String,
    /// Operation-specific parameters (forwarded verbatim to the provider).
    #[serde(default)]
    pub params: Value,
    /// Preferred routing: `"local"` (same gate only), `"any"` (local or remote).
    /// Default: `"any"`.
    #[serde(default = "default_routing")]
    pub routing: String,
}

fn default_routing() -> String {
    "any".to_string()
}

/// `capability.call` response — result from the resolved provider.
#[derive(Debug, Clone, Serialize)]
pub struct CapabilityCallResult {
    /// The provider that handled the call.
    pub provider: String,
    /// Which gate served the request (`"local"` or a remote `node_id`).
    pub gate: String,
    /// The provider's response (operation-specific).
    pub result: Value,
}

/// `lifecycle.composition` response — current composition state for dashboards.
#[derive(Debug, Clone, Serialize)]
pub struct CompositionState {
    pub primals: Vec<CompositionPrimalInfo>,
    pub total_capabilities: usize,
    pub timestamp: String,
}

/// Per-primal composition entry for `lifecycle.composition`.
#[derive(Debug, Clone, Serialize)]
pub struct CompositionPrimalInfo {
    pub primal_id: String,
    pub capabilities: Vec<String>,
    pub virtual_endpoint: String,
    pub status: &'static str,
}

/// `lifecycle.validate_consumed` response — composition completeness check.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateConsumedResult {
    pub valid: bool,
    pub satisfied: Vec<String>,
    pub unsatisfied: Vec<String>,
}

/// JSON-RPC result for `songbird.federation.peers` / `federation.peers`
#[derive(Debug, Clone, Serialize)]
pub struct FederationPeersResponse {
    pub peers: Vec<String>,
    pub total_count: usize,
    pub federation_enabled: bool,
}

/// JSON-RPC result for `songbird.federation.status` / `federation.status`
#[derive(Debug, Clone, Serialize)]
pub struct FederationStatusResponse {
    pub enabled: bool,
    pub active_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_params_deserializes_from_json() {
        let json = r#"{"primal_id":"security","capabilities":["crypto","auth"],"endpoint":"/tmp/security.sock"}"#;
        let params: RegisterParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.primal_id, "security");
        assert_eq!(params.capabilities, vec!["crypto", "auth"]);
        assert_eq!(params.endpoint, "/tmp/security.sock");
    }

    #[test]
    fn register_params_empty_capabilities() {
        let json = r#"{"primal_id":"minimal","capabilities":[],"endpoint":"tcp://127.0.0.1:9000"}"#;
        let params: RegisterParams = serde_json::from_str(json).unwrap();
        assert!(params.capabilities.is_empty());
    }

    #[test]
    fn resolve_params_deserializes_primal_id() {
        let json = r#"{"primal_id":"ai-provider"}"#;
        let params: ResolveParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.primal_id.as_deref(), Some("ai-provider"));
        assert!(params.capability.is_none());
    }

    #[test]
    fn resolve_params_deserializes_capability() {
        let json = r#"{"capability":"crypto.sign"}"#;
        let params: ResolveParams = serde_json::from_str(json).unwrap();
        assert!(params.primal_id.is_none());
        assert_eq!(params.capability.as_deref(), Some("crypto.sign"));
    }

    #[test]
    fn resolve_params_deserializes_both() {
        let json = r#"{"primal_id":"security","capability":"crypto.sign"}"#;
        let params: ResolveParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.primal_id.as_deref(), Some("security"));
        assert_eq!(params.capability.as_deref(), Some("crypto.sign"));
    }

    #[test]
    fn resolve_params_deserializes_empty() {
        let json = r"{}";
        let params: ResolveParams = serde_json::from_str(json).unwrap();
        assert!(params.primal_id.is_none());
        assert!(params.capability.is_none());
    }

    #[test]
    fn discover_params_deserializes() {
        let json = r#"{"capability":"network.discovery"}"#;
        let params: DiscoverParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.capability, "network.discovery");
    }

    #[test]
    fn register_result_serializes() {
        let result = RegisterResult {
            virtual_endpoint: "/primal/security".to_string(),
            registered_at: "2026-03-27T12:00:00Z".to_string(),
            signature: None,
            signed_payload: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["virtual_endpoint"], "/primal/security");
        assert_eq!(json["registered_at"], "2026-03-27T12:00:00Z");
        assert!(json.get("signature").is_none(), "None fields should be omitted");
    }

    #[test]
    fn resolve_result_serializes_with_capabilities() {
        let result = ResolveResult {
            socket: Some("/tmp/security.sock".to_string()),
            virtual_endpoint: "/primal/security".to_string(),
            native_endpoint: "unix:///tmp/security.sock".to_string(),
            capabilities: vec!["crypto".to_string(), "auth".to_string()],
            relay: false,
            relay_socket: None,
            signature: None,
            signed_payload: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["socket"], "/tmp/security.sock");
        let caps = json["capabilities"].as_array().unwrap();
        assert_eq!(caps.len(), 2);
        assert!(json.get("signature").is_none(), "None fields should be omitted");
        assert_eq!(json["relay"], false);
    }

    #[test]
    fn discover_result_empty_providers() {
        let result = DiscoverResult {
            providers: vec![],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["providers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn discover_result_with_providers() {
        let result = DiscoverResult {
            providers: vec![ProviderInfo {
                primal_id: "songbird".to_string(),
                socket: Some("/tmp/songbird.sock".to_string()),
                virtual_endpoint: "/primal/songbird".to_string(),
                native_endpoint: "unix:///tmp/songbird.sock".to_string(),
                capabilities: vec!["network.discovery".to_string()],
                signature: None,
                signed_payload: None,
            }],
        };
        let json = serde_json::to_value(&result).unwrap();
        let providers = json["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0]["primal_id"], "songbird");
        assert_eq!(providers[0]["socket"], "/tmp/songbird.sock");
    }

    #[test]
    fn list_result_serializes() {
        let result = ListResult {
            services: vec![
                ServiceInfo {
                    primal_id: "security".to_string(),
                    virtual_endpoint: "/primal/security".to_string(),
                    capabilities: vec!["crypto".to_string()],
                },
                ServiceInfo {
                    primal_id: "songbird".to_string(),
                    virtual_endpoint: "/primal/songbird".to_string(),
                    capabilities: vec!["network.discovery".to_string(), "ipc.jsonrpc".to_string()],
                },
            ],
        };
        let json = serde_json::to_value(&result).unwrap();
        let services = json["services"].as_array().unwrap();
        assert_eq!(services.len(), 2);
        assert_eq!(services[0]["primal_id"], "security");
        assert_eq!(services[1]["capabilities"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn federation_peers_response_serializes() {
        let result = FederationPeersResponse {
            peers: vec!["node-a".to_string(), "node-b".to_string()],
            total_count: 2,
            federation_enabled: true,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["total_count"], 2);
        assert!(json["federation_enabled"].as_bool().unwrap());
        assert_eq!(json["peers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn federation_status_response_serializes() {
        let result = FederationStatusResponse {
            enabled: false,
            active_connections: 0,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(!json["enabled"].as_bool().unwrap());
        assert_eq!(json["active_connections"], 0);
    }

    #[test]
    fn provider_info_clone_is_independent() {
        let original = ProviderInfo {
            primal_id: "test".to_string(),
            socket: Some("/tmp/test.sock".to_string()),
            virtual_endpoint: "/primal/test".to_string(),
            native_endpoint: "unix:///tmp/test.sock".to_string(),
            capabilities: vec!["cap1".to_string()],
            signature: Some("sig123".to_string()),
            signed_payload: Some("payload".to_string()),
        };
        let cloned = original.clone();
        assert_eq!(original.primal_id, cloned.primal_id);
        assert_eq!(original.signature, cloned.signature);
        assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
    }

    #[test]
    fn register_params_rejects_missing_fields() {
        let json = r#"{"primal_id":"security"}"#;
        assert!(serde_json::from_str::<RegisterParams>(json).is_err());
    }

    #[test]
    fn register_result_with_signature_serializes() {
        let result = RegisterResult {
            virtual_endpoint: "/primal/nestgate".to_string(),
            registered_at: "2026-04-28T14:00:00Z".to_string(),
            signature: Some("c2lnbmF0dXJl".to_string()),
            signed_payload: Some(
                r#"{"c":["storage"],"e":"/tmp/ng.sock","p":"nestgate","t":"2026-04-28T14:00:00Z"}"#
                    .to_string(),
            ),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["signature"], "c2lnbmF0dXJl");
        assert!(json["signed_payload"].as_str().unwrap().contains("nestgate"));
    }

    #[test]
    fn resolve_result_with_signature_serializes() {
        let result = ResolveResult {
            socket: Some("/run/user/1000/biomeos/beardog.sock".to_string()),
            virtual_endpoint: "/primal/beardog".to_string(),
            native_endpoint: "unix:///run/user/1000/biomeos/beardog.sock".to_string(),
            capabilities: vec!["crypto".to_string()],
            relay: false,
            relay_socket: None,
            signature: Some("sig_b64".to_string()),
            signed_payload: Some("payload_json".to_string()),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["signature"], "sig_b64");
        assert_eq!(json["signed_payload"], "payload_json");
        assert_eq!(json["socket"], "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn capability_resolve_result_omits_none_signature() {
        let result = CapabilityResolveResult {
            primal_id: "songbird".to_string(),
            socket: Some("/tmp/songbird.sock".to_string()),
            virtual_endpoint: "/primal/songbird".to_string(),
            native_endpoint: "unix:///tmp/songbird.sock".to_string(),
            capabilities: vec!["network.discovery".to_string()],
            signature: None,
            signed_payload: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("signature").is_none());
        assert!(json.get("signed_payload").is_none());
        assert_eq!(json["socket"], "/tmp/songbird.sock");
    }
}
