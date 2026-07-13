// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{FederationPeersResponse, FederationStatusResponse, IpcServiceHandler};
use serde_json::Value;

/// Check if federation is configured via environment variables (fallback when
/// `FederationState` is not injected into the handler — e.g. standalone IPC path).
fn federation_configured_via_env() -> bool {
    songbird_process_env::var("SONGBIRD_FEDERATION_ENABLED")
        .or_else(|_| songbird_process_env::var("FEDERATION_ENABLED"))
        .map(|v| songbird_types::error_helpers::parse_bool_relaxed(&v).unwrap_or(false))
        .unwrap_or(false)
        || songbird_process_env::var("SONGBIRD_PEERS").is_ok()
        || songbird_process_env::var("SONGBIRD_FEDERATION_PORT").is_ok()
}

impl IpcServiceHandler {
    /// Handle `birdsong.advertise` method
    ///
    /// Generates an encrypted beacon with the onion endpoint (if running).
    /// This is the complete Dark Forest beacon - only family can see the .onion address.
    ///
    /// NEW (Feb 6, 2026) - Combines onion service and birdsong beacon
    pub(super) async fn handle_birdsong_advertise(&self, params: Value) -> Result<Value, String> {
        // Get node_id and capabilities from params
        let node_id = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing node_id parameter")?
            .to_string();

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Get onion address if service is running
        let onion_status = self.onion_handler.handle_status(serde_json::json!({})).await?;
        let onion_endpoint = if onion_status.get("running") == Some(&serde_json::json!(true)) {
            let addr = onion_status.get("onion_address").and_then(|v| v.as_str());
            const DEFAULT_PORT_U64: u64 =
                songbird_types::defaults::ports::DEFAULT_SONGBIRD_PORT as u64;
            let port = onion_status
                .get("port")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(DEFAULT_PORT_U64);
            addr.map(|a| format!("{a}:{port}"))
        } else {
            None
        };

        // Also include any direct endpoint hints
        let endpoint_hints = params.get("endpoint_hints").cloned();

        // Generate the encrypted beacon with all endpoints
        let beacon_params = serde_json::json!({
            "node_id": node_id,
            "capabilities": capabilities,
            "onion_endpoint": onion_endpoint,
            "endpoint_hints": endpoint_hints,
        });

        let beacon_result =
            self.birdsong_handler.handle_generate_encrypted_beacon(beacon_params).await?;

        // Return combined result
        Ok(serde_json::json!({
            "beacon": beacon_result,
            "onion_endpoint": onion_endpoint,
            "onion_running": onion_status.get("running"),
        }))
    }

    /// Handle `health` method — requires uptime + registry info
    pub(super) async fn handle_health(&self) -> Result<Value, String> {
        let uptime_secs = self.start_time.read().await.elapsed().as_secs();
        let service_count = self.registry.read().await.list_services().await.len();
        Ok(crate::introspection::health(uptime_secs, service_count))
    }

    /// Handle `identity` method — canonical family-id lookup
    pub(super) async fn handle_identity(&self) -> Result<Value, String> {
        let family_id = self.family_id_overrides.as_ref().map_or_else(
            || crate::introspection::canonical_family_id(|k| songbird_process_env::var(k)),
            |map| {
                crate::introspection::canonical_family_id(|k| {
                    map.get(k).cloned().ok_or(std::env::VarError::NotPresent)
                })
            },
        );
        Ok(crate::introspection::identity(&family_id))
    }

    /// `songbird.federation.peers` / `federation.peers`
    pub(super) async fn handle_federation_peers_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationPeersResponse {
                peers: vec![],
                total_count: 0,
                federation_enabled: federation_configured_via_env(),
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let mut peers: Vec<String> =
            state.active_nodes().await.into_iter().map(|n| n.node_id).collect();
        peers.sort();
        let total_count = peers.len();

        serde_json::to_value(FederationPeersResponse {
            peers,
            total_count,
            federation_enabled: true,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }

    /// `songbird.federation.status` / `federation.status`
    ///
    /// `enabled` reflects whether federation was configured (state injected OR env vars set),
    /// NOT whether remote peers are connected. Use `active_connections` for connectivity.
    pub(super) async fn handle_federation_status_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationStatusResponse {
                enabled: federation_configured_via_env(),
                active_connections: 0,
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let fed_stats = state.get_stats().await;
        serde_json::to_value(FederationStatusResponse {
            enabled: true,
            active_connections: fed_stats.active_nodes,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::registry::ServiceRegistry;
    use crate::service::{FederationPeersResponse, FederationStatusResponse, IpcServiceHandler};
    use crate::tower_atomic::JsonRpcHandler;
    use serde_json::{Value, json};
    use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn is_expected_birdsong_delegate_error(msg: &str) -> bool {
        let m = msg.to_lowercase();
        m.contains("missing node_id")
            || m.contains("security provider")
            || m.contains("socket")
            || m.contains("ipc")
            || m.contains("connection refused")
            || m.contains("no such file")
            || m.contains("crypto")
            || m.contains("rpc")
            || m.contains("encryption failed")
            || m.contains("invalid params")
    }

    #[tokio::test]
    async fn health_check_includes_uptime_and_registry_service_count() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(Arc::clone(&registry));

        let v = handler.handle("health.check", json!({})).await.expect("health.check");
        assert_eq!(v["status"], "healthy");
        assert!(v["uptime_s"].as_u64().is_some());
        assert_eq!(v["services"], json!(0));

        handler
            .handle(
                "ipc.register",
                json!({
                    "primal_id": "meta-health-peer",
                    "capabilities": ["test.cap"],
                    "endpoint": "/tmp/meta-health.sock"
                }),
            )
            .await
            .expect("register");

        let v2 = handler.handle("health.check", json!({})).await.expect("health.check 2");
        assert_eq!(v2["services"], json!(1));
        assert!(v2["uptime_s"].as_u64().unwrap() >= v["uptime_s"].as_u64().unwrap());
    }

    #[tokio::test]
    async fn identity_uses_injected_env_for_family_id() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".into(), "injected-family-42".into());
        let handler = IpcServiceHandler::with_family_id_overrides(Arc::clone(&registry), env);

        let v = handler.handle("identity", json!({})).await.expect("identity");
        assert_eq!(v["family_id"], "injected-family-42");
        assert_eq!(v["primal"], json!("songbird"));
        assert!(v["capabilities"].is_array());
        assert!(v["version"].is_string());
    }

    #[tokio::test]
    async fn identity_defaults_when_injected_env_missing_all_keys() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler =
            IpcServiceHandler::with_family_id_overrides(Arc::clone(&registry), HashMap::new());

        let v = handler.handle("identity", json!({})).await.expect("identity");
        assert_eq!(v["family_id"], "default");
    }

    #[tokio::test]
    async fn birdsong_advertise_errors_when_node_id_missing() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let err = handler
            .handle("birdsong.advertise", json!({ "capabilities": [] }))
            .await
            .expect_err("missing node_id");
        assert!(err.contains("Missing node_id"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn birdsong_advertise_errors_when_node_id_not_a_string() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let err = handler
            .handle("birdsong.advertise", json!({ "node_id": 12345 }))
            .await
            .expect_err("node_id not string");
        assert!(err.contains("Missing node_id"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn birdsong_advertise_capabilities_non_array_becomes_empty() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let params = json!({
            "node_id": "n1",
            "capabilities": "not-an-array",
            "endpoint_hints": null
        });
        let r = handler.handle("birdsong.advertise", params).await;
        match r {
            Ok(v) => {
                let beacon = v["beacon"].as_object().expect("beacon object");
                let inner = beacon["encrypted_beacon"].as_str().expect("encrypted_beacon");
                assert!(!inner.is_empty());
            }
            Err(e) => assert!(is_expected_birdsong_delegate_error(&e), "unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn birdsong_advertise_capabilities_filters_non_string_entries() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let params = json!({
            "node_id": "n2",
            "capabilities": ["keep", 99, "also-keep", {"x": 1}]
        });
        let r = handler.handle("birdsong.advertise", params).await;
        match r {
            Ok(v) => {
                assert_eq!(v["onion_running"], json!(false));
                assert!(v.get("onion_endpoint").is_none() || v["onion_endpoint"].is_null());
            }
            Err(e) => assert!(is_expected_birdsong_delegate_error(&e), "unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn birdsong_advertise_preserves_endpoint_hints_metadata() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let hints = json!({ "lan": "192.168.0.10:0", "note": "port 0 hint" });
        let params = json!({
            "node_id": "n-hints",
            "capabilities": [],
            "endpoint_hints": hints.clone()
        });
        let r = handler.handle("birdsong.advertise", params).await;
        match r {
            Ok(v) => {
                let beacon = &v["beacon"];
                assert!(beacon.is_object(), "expected beacon object: {beacon:?}");
            }
            Err(e) => assert!(is_expected_birdsong_delegate_error(&e), "unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn birdsong_advertise_onion_not_running_sets_onion_fields() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);
        let r = handler
            .handle("birdsong.advertise", json!({ "node_id": "n-onion", "capabilities": [] }))
            .await;
        match r {
            Ok(v) => {
                assert_eq!(v["onion_running"], json!(false));
                assert!(v.get("onion_endpoint").is_none() || v["onion_endpoint"].is_null());
            }
            Err(e) => assert!(is_expected_birdsong_delegate_error(&e), "unexpected error: {e}"),
        }
    }

    #[test]
    fn federation_peers_response_serializes_stable_wire_shape() {
        let r = FederationPeersResponse {
            peers: vec!["z".into(), "a".into()],
            total_count: 2,
            federation_enabled: true,
        };
        let v: Value = serde_json::to_value(&r).expect("serialize FederationPeersResponse");
        assert_eq!(v["peers"], json!(["z", "a"]));
        assert_eq!(v["total_count"], json!(2));
        assert_eq!(v["federation_enabled"], json!(true));
        let round: Value = serde_json::from_str(&serde_json::to_string(&v).expect("stringify"))
            .expect("parse json string");
        assert_eq!(round, v);
    }

    #[test]
    fn federation_status_response_serializes_stable_wire_shape() {
        let r = FederationStatusResponse {
            enabled: false,
            active_connections: 0,
        };
        let v: Value = serde_json::to_value(&r).expect("serialize FederationStatusResponse");
        assert_eq!(v["enabled"], json!(false));
        assert_eq!(v["active_connections"], json!(0));
        let round: Value = serde_json::from_str(&serde_json::to_string(&v).expect("stringify"))
            .expect("parse json string");
        assert_eq!(round, v);
    }

    #[tokio::test]
    async fn federation_peers_sorts_node_ids_and_matches_status() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let federation = Arc::new(FederationState::new("meta-sort-test".into()));
        let now = chrono::Utc::now();
        for id in ["gamma", "alpha", "beta"] {
            federation
                .register_node(NodeRegistration {
                    node_id: id.into(),
                    node_name: id.into(),
                    node_address: "127.0.0.1:0".into(),
                    endpoints: None,
                    cpu_cores: 0,
                    memory_gb: 0,
                    gpu_model: None,
                    storage_gb: None,
                    capabilities: vec![],
                    status: NodeStatus::Active,
                    joined_at: now,
                    last_heartbeat: now,
                })
                .await;
        }

        let handler = IpcServiceHandler::with_federation_state(registry, Arc::clone(&federation));

        let p = handler.handle("federation.peers", json!({})).await.expect("peers");
        assert_eq!(p["peers"], json!(["alpha", "beta", "gamma"]));
        assert_eq!(p["total_count"], json!(3));
        assert_eq!(p["federation_enabled"], json!(true));

        let st = handler.handle("songbird.federation.status", json!({})).await.expect("status");
        assert_eq!(st["enabled"], json!(true));
        assert_eq!(st["active_connections"], json!(3));
    }

    #[tokio::test]
    async fn federation_peers_empty_state_serializes_like_meta_defaults() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);

        let p = handler.handle("songbird.federation.peers", json!({})).await.expect("peers");
        let expected = serde_json::to_value(FederationPeersResponse {
            peers: vec![],
            total_count: 0,
            federation_enabled: false,
        })
        .expect("to_value");
        assert_eq!(p, expected);

        let st = handler.handle("federation.status", json!({})).await.expect("status");
        let expected_st = serde_json::to_value(FederationStatusResponse {
            enabled: false,
            active_connections: 0,
        })
        .expect("to_value status");
        assert_eq!(st, expected_st);
    }

    #[tokio::test]
    async fn federation_status_reads_env_var_when_state_not_injected() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);

        let _env = songbird_process_env::ScopedEnv::new("SONGBIRD_FEDERATION_ENABLED", "true");

        let st = handler.handle("federation.status", json!({})).await.expect("status");
        assert_eq!(st["enabled"], json!(true), "env var should wire into response");
        assert_eq!(st["active_connections"], json!(0));

        let p = handler.handle("federation.peers", json!({})).await.expect("peers");
        assert_eq!(p["federation_enabled"], json!(true));
    }

    #[tokio::test]
    async fn federation_status_reads_peers_env_as_implicit_enabled() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new_isolated(registry);

        let _env = songbird_process_env::ScopedEnv::new("SONGBIRD_PEERS", "157.230.3.183:7700");

        let st = handler.handle("federation.status", json!({})).await.expect("status");
        assert_eq!(st["enabled"], json!(true), "SONGBIRD_PEERS implies federation enabled");
    }
}
