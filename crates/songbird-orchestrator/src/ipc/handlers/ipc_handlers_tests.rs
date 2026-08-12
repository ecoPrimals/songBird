// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::IpcHandlers;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::registry::ServiceRegistry;
use songbird_http_client::SecurityRpcClient;
use songbird_types::json_rpc_method::MeshMethod;
use std::sync::Arc;

fn test_handlers() -> IpcHandlers {
    IpcHandlers::new(
        Arc::new(ServiceRegistry::new()),
        None,
        Arc::new(ConnectionManager::new()),
        Arc::new(SecurityRpcClient::new("/tmp/songbird-orchestrator-ipc-handlers-test.sock")),
    )
}

#[test]
fn new_preserves_registry_and_connection_manager_arcs() {
    let registry = Arc::new(ServiceRegistry::new());
    let connections = Arc::new(ConnectionManager::new());
    let security =
        Arc::new(SecurityRpcClient::new("/tmp/songbird-orchestrator-ipc-handlers-test.sock"));
    let registry_ptr = Arc::as_ptr(&registry);
    let connections_ptr = Arc::as_ptr(&connections);

    let handlers = IpcHandlers::new(registry, None, connections, security);

    assert!(handlers.discovery_listener.is_none());
    assert_eq!(Arc::as_ptr(&handlers.service_registry), registry_ptr);
    assert_eq!(Arc::as_ptr(&handlers.connection_manager), connections_ptr);
}

#[tokio::test]
async fn mesh_status_before_init_returns_awaiting() {
    let handlers = test_handlers();
    let result = handlers.mesh_dispatch(MeshMethod::Status, None).await;
    assert!(result.is_ok(), "mesh.status should succeed even before mesh.init");
    let val = result.unwrap();
    assert_eq!(val["initialized"], false);
    assert_eq!(val["status"], "awaiting_init");
}

#[tokio::test]
async fn mesh_status_works_after_init() {
    let handlers = test_handlers();
    let init_params = serde_json::json!({
        "node_id": "tower-status-test",
        "bootstrap_onions": []
    });
    handlers.mesh_dispatch(MeshMethod::Init, Some(init_params)).await.unwrap();
    let result = handlers.mesh_dispatch(MeshMethod::Status, None).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.is_object(), "mesh.status should return a JSON object");
}

#[tokio::test]
async fn mesh_init_succeeds_with_valid_params() {
    let handlers = test_handlers();
    let params = serde_json::json!({
        "node_id": "tower-test-node-12345678",
        "bootstrap_onions": []
    });
    let result = handlers.mesh_dispatch(MeshMethod::Init, Some(params)).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["initialized"], true);
}

#[tokio::test]
async fn mesh_init_fails_without_node_id() {
    let handlers = test_handlers();
    let result = handlers.mesh_dispatch(MeshMethod::Init, Some(serde_json::json!({}))).await;
    assert!(result.is_err());
}
