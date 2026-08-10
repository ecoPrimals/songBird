// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RiboCipher federation dispatch.
//!
//! Handles JSON-RPC method routing for connections arriving over a
//! `0xED`-signalled riboCipher channel on the federation port (`:7700`).
//! Routes through the full `IpcServiceHandler` when available, with
//! graceful fallback to intrinsic health/capability responses.

use songbird_universal_ipc::tower_atomic::JsonRpcHandler;

/// Dispatch a JSON-RPC method received over a riboCipher-signalled federation connection.
///
/// Routes through the full `IpcServiceHandler` when available (Tier 2 acceptance).
/// Falls back to inline health/capabilities responses when no handler is wired
/// (graceful degradation for early bootstrap).
pub(super) async fn dispatch_ribocipher_rpc(
    method: &str,
    params: serde_json::Value,
    id: serde_json::Value,
    tier: &str,
    handler: Option<&songbird_universal_ipc::service::IpcServiceHandler>,
) -> serde_json::Value {
    // Fast-path intrinsics that must always work (even without handler)
    match method {
        "health.liveness" | "health" | "ping" => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "status": "healthy",
                    "primal": songbird_types::primal_names::SELF_NAME,
                    "version": env!("CARGO_PKG_VERSION"),
                    "tier": tier,
                    "uptime_secs": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                },
                "id": id
            });
        }
        "system.capabilities" | "capabilities" => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "capabilities": [
                        "mesh.relay",
                        "federation.peer",
                        "health.liveness",
                        "birdsong.broadcast",
                        "capability.call",
                        "capability.resolve",
                    ],
                    "primal": songbird_types::primal_names::SELF_NAME,
                    "tier": tier,
                },
                "id": id
            });
        }
        _ => {}
    }

    // Tier 2: full dispatch through IpcServiceHandler
    if let Some(h) = handler {
        match h.handle(method, params).await {
            Ok(result) => serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            }),
            Err(msg) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32603, "message": msg},
                "id": id
            }),
        }
    } else {
        tracing::warn!(
            "riboCipher {tier}: no IpcServiceHandler wired — cannot dispatch '{method}'"
        );
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": format!("Method not available: {method}")},
            "id": id
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn dispatch_health_liveness_returns_healthy() {
        let resp = dispatch_ribocipher_rpc(
            "health.liveness",
            serde_json::Value::Null,
            serde_json::json!(1),
            "clear",
            None,
        )
        .await;
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["status"], "healthy");
        assert_eq!(resp["result"]["primal"], "songbird");
        assert_eq!(resp["result"]["tier"], "clear");
        assert!(resp["result"]["version"].as_str().is_some());
    }

    #[tokio::test]
    async fn dispatch_health_alias_returns_healthy() {
        let resp = dispatch_ribocipher_rpc(
            "health",
            serde_json::Value::Null,
            serde_json::json!("abc"),
            "mito",
            None,
        )
        .await;
        assert_eq!(resp["result"]["status"], "healthy");
        assert_eq!(resp["result"]["tier"], "mito");
    }

    #[tokio::test]
    async fn dispatch_ping_returns_healthy() {
        let resp = dispatch_ribocipher_rpc(
            "ping",
            serde_json::Value::Null,
            serde_json::json!(42),
            "clear",
            None,
        )
        .await;
        assert_eq!(resp["result"]["status"], "healthy");
        assert_eq!(resp["id"], 42);
    }

    #[tokio::test]
    async fn dispatch_capabilities_returns_list() {
        let resp = dispatch_ribocipher_rpc(
            "system.capabilities",
            serde_json::Value::Null,
            serde_json::json!(2),
            "clear",
            None,
        )
        .await;
        let caps = resp["result"]["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c == "health.liveness"));
        assert!(caps.iter().any(|c| c == "mesh.relay"));
        assert!(caps.iter().any(|c| c == "capability.call"));
        assert_eq!(resp["result"]["primal"], "songbird");
    }

    #[tokio::test]
    async fn dispatch_unknown_method_without_handler_returns_error() {
        let resp = dispatch_ribocipher_rpc(
            "custom.method",
            serde_json::json!({"key": "value"}),
            serde_json::json!(99),
            "nuclear",
            None,
        )
        .await;
        assert_eq!(resp["error"]["code"], -32601);
        assert!(resp["error"]["message"].as_str().unwrap().contains("custom.method"));
        assert_eq!(resp["id"], 99);
    }

    #[tokio::test]
    async fn dispatch_with_handler_routes_ipc_list() {
        let registry = Arc::new(tokio::sync::RwLock::new(
            songbird_universal_ipc::registry::ServiceRegistry::new(),
        ));
        let handler = songbird_universal_ipc::service::IpcServiceHandler::new(registry);
        let resp = dispatch_ribocipher_rpc(
            "ipc.list",
            serde_json::Value::Null,
            serde_json::json!(7),
            "mito",
            Some(&handler),
        )
        .await;
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 7);
        assert!(
            resp["result"].is_object() || resp["result"].is_array(),
            "expected structured result from ipc.list, got: {}",
            resp
        );
        assert!(resp.get("error").is_none());
    }

    #[tokio::test]
    async fn dispatch_mito_tier_health_via_handler() {
        let registry = Arc::new(tokio::sync::RwLock::new(
            songbird_universal_ipc::registry::ServiceRegistry::new(),
        ));
        let handler = songbird_universal_ipc::service::IpcServiceHandler::new(registry);
        let resp = dispatch_ribocipher_rpc(
            "health",
            serde_json::Value::Null,
            serde_json::json!(100),
            "mito",
            Some(&handler),
        )
        .await;
        assert_eq!(resp["result"]["status"], "healthy");
        assert_eq!(resp["result"]["tier"], "mito");
        assert_eq!(resp["id"], 100);
    }
}
