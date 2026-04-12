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
/// Accepts either `primal_id` (identity lookup) or `capability` (capability-based
/// routing). When `capability` is provided, returns the best provider for that
/// capability domain — springs can resolve by capability without knowing primal names.
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveParams {
    #[serde(default)]
    pub primal_id: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
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
}

/// IPC service response for resolution
#[derive(Debug, Clone, Serialize)]
pub struct ResolveResult {
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
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
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
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
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
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
        let json = r#"{}"#;
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
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["virtual_endpoint"], "/primal/security");
        assert_eq!(json["registered_at"], "2026-03-27T12:00:00Z");
    }

    #[test]
    fn resolve_result_serializes_with_capabilities() {
        let result = ResolveResult {
            virtual_endpoint: "/primal/security".to_string(),
            native_endpoint: "/tmp/security.sock".to_string(),
            capabilities: vec!["crypto".to_string(), "auth".to_string()],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["native_endpoint"], "/tmp/security.sock");
        let caps = json["capabilities"].as_array().unwrap();
        assert_eq!(caps.len(), 2);
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
                virtual_endpoint: "/primal/songbird".to_string(),
                native_endpoint: "/tmp/songbird.sock".to_string(),
                capabilities: vec!["network.discovery".to_string()],
            }],
        };
        let json = serde_json::to_value(&result).unwrap();
        let providers = json["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0]["primal_id"], "songbird");
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
            virtual_endpoint: "/primal/test".to_string(),
            native_endpoint: "/tmp/test.sock".to_string(),
            capabilities: vec!["cap1".to_string()],
        };
        let cloned = original.clone();
        assert_eq!(original.primal_id, cloned.primal_id);
        assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
    }

    #[test]
    fn register_params_rejects_missing_fields() {
        let json = r#"{"primal_id":"security"}"#;
        assert!(serde_json::from_str::<RegisterParams>(json).is_err());
    }
}
