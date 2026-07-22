// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

type RpcHandler = Arc<dyn Fn(&str, &Value) -> Value + Send + Sync>;

async fn spawn_mock_security_server(path: PathBuf, handler: RpcHandler) {
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path).unwrap();
        let _ = ready_tx.send(());
        while let Ok((stream, _)) = listener.accept().await {
            let handler = Arc::clone(&handler);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }
                let request: Value =
                    serde_json::from_str(line.trim()).unwrap_or_else(|_| json!({}));
                let method = request["method"].as_str().unwrap_or("");
                let id = request["id"].clone();
                let result = handler(method, request.get("params").unwrap_or(&Value::Null));
                let response = json!({ "jsonrpc": "2.0", "result": result, "id": id });
                let mut stream = reader.into_inner();
                let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
            });
        }
    });
    ready_rx.await.unwrap();
}

fn temp_socket_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("songbird_prod_sec_{label}_{}.sock", uuid::Uuid::new_v4()))
}

fn sample_lineage_chain() -> LineageChain {
    LineageChain {
        root_id: "root".into(),
        node_id: "child".into(),
        links: vec![],
        depth: 0,
    }
}

fn sample_lineage_proof() -> LineageProof {
    LineageProof {
        chain: sample_lineage_chain(),
        claimer_signature: vec![1, 2, 3],
    }
}

#[tokio::test]
async fn test_production_provider_creation() {
    // Test with non-existent socket (should error gracefully)
    let result =
        ProductionSecurityProvider::new("/tmp/nonexistent_security_provider_test.sock").await;
    assert!(result.is_err(), "Should error when socket doesn't exist");
}

#[tokio::test]
async fn production_connects_to_bound_unix_socket_and_exposes_metadata() {
    let path =
        std::env::temp_dir().join(format!("songbird_prod_sec_test_{}.sock", uuid::Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    let accept_task = tokio::spawn(async move {
        listener.accept().await.unwrap();
    });

    let mut provider = ProductionSecurityProvider::new(&path).await.unwrap();
    accept_task.await.unwrap();

    assert_eq!(provider.version(), "production-unix-socket");
    assert_eq!(provider.get_visibility_level(0), AccessLevel::FullLineage);
    assert_eq!(provider.get_visibility_level(2), AccessLevel::SubMasked);
    assert_eq!(provider.get_visibility_level(7), AccessLevel::Masked);

    provider.set_family_id("family-x");
    provider.shutdown().await.unwrap();

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn with_family_id_stores_family_on_connect() {
    let path = temp_socket_path("family");
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    let accept_task = tokio::spawn(async move {
        listener.accept().await.unwrap();
    });

    let provider = ProductionSecurityProvider::with_family_id(&path, "family-test").await.unwrap();
    accept_task.await.unwrap();
    assert_eq!(provider.version(), "production-unix-socket");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn is_available_true_when_health_healthy() {
    let path = temp_socket_path("health_ok");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "health" {
                json!({ "status": "healthy" })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    assert!(provider.is_available().await);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn is_available_false_when_health_degraded() {
    let path = temp_socket_path("health_bad");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "health" {
                json!({ "status": "degraded" })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    assert!(!provider.is_available().await);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn verify_lineage_returns_rpc_valid_flag() {
    let path = temp_socket_path("verify");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "genetic.verify_lineage" {
                json!({ "valid": true })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    let proof = sample_lineage_proof();
    assert!(provider.verify_lineage(&proof).await.unwrap());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn generate_lineage_parses_chain_response() {
    let path = temp_socket_path("gen_lineage");
    let _ = std::fs::remove_file(&path);
    let chain = sample_lineage_chain();
    let chain_json = serde_json::to_value(&chain).unwrap();
    spawn_mock_security_server(
        path.clone(),
        Arc::new(move |method, _| {
            if method == "genetic.generate_lineage" {
                chain_json.clone()
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    let result = provider.generate_lineage("child", "root").await.unwrap();
    assert_eq!(result.root_id, "root");
    assert_eq!(result.node_id, "child");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn get_descendants_parses_node_list() {
    let path = temp_socket_path("descendants");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "genetic.get_descendants" {
                json!(["a", "b", "c"])
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    let nodes = provider.get_descendants("root").await.unwrap();
    assert_eq!(nodes, vec!["a", "b", "c"]);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn get_lineage_depth_parses_optional_depth() {
    let path = temp_socket_path("depth");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "genetic.get_lineage_depth" {
                json!({ "depth": 3 })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    assert_eq!(provider.get_lineage_depth("a", "d").await.unwrap(), Some(3));
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn rpc_error_surfaces_as_anyhow() {
    let path = temp_socket_path("rpc_err");
    let _ = std::fs::remove_file(&path);
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    tokio::spawn({
        let path = path.clone();
        async move {
            let listener = UnixListener::bind(&path).unwrap();
            let _ = ready_tx.send(());
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_err() || line.is_empty() {
                        return;
                    }
                    let request: Value =
                        serde_json::from_str(line.trim()).unwrap_or_else(|_| json!({}));
                    let id = request["id"].clone();
                    let method = request["method"].as_str().unwrap_or("");
                    let response = if method == "genetic.verify_lineage" {
                        json!({
                            "jsonrpc": "2.0",
                            "error": { "code": -1, "message": "denied" },
                            "id": id
                        })
                    } else {
                        json!({ "jsonrpc": "2.0", "result": json!({}), "id": id })
                    };
                    let mut stream = reader.into_inner();
                    let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
                });
            }
        }
    });
    ready_rx.await.unwrap();

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    let err = provider.verify_lineage(&sample_lineage_proof()).await.unwrap_err();
    assert!(err.to_string().contains("Security provider RPC error"));
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn decrypt_birdsong_returns_none_when_not_in_lineage() {
    let path = temp_socket_path("decrypt_none");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "birdsong.decrypt" {
                json!({ "success": false })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    let encrypted = EncryptedBirdSong {
        version: 1,
        ciphertext: vec![9],
        lineage_hint: LineageHint::Universal,
        timestamp: Utc::now(),
        signature: vec![],
        genesis_witness: None,
    };
    assert!(provider.decrypt_birdsong(&encrypted).await.unwrap().is_none());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn get_visibility_level_maps_lineage_depths() {
    let path = temp_socket_path("visibility");
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    let accept_task = tokio::spawn(async move {
        listener.accept().await.unwrap();
    });

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    accept_task.await.unwrap();
    assert_eq!(provider.get_visibility_level(0), AccessLevel::FullLineage);
    assert_eq!(provider.get_visibility_level(3), AccessLevel::SubMasked);
    assert_eq!(provider.get_visibility_level(5), AccessLevel::Masked);
    assert_eq!(provider.get_visibility_level(15), AccessLevel::Transport);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn encrypt_for_lineage_uses_explicit_family_id() {
    let path = temp_socket_path("encrypt");
    let _ = std::fs::remove_file(&path);
    let captured = Arc::new(std::sync::Mutex::new(None::<String>));
    let captured_clone = Arc::clone(&captured);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(move |method, params| {
            if method == "birdsong.encrypt" {
                *captured_clone.lock().unwrap() =
                    params.get("family_id").and_then(Value::as_str).map(str::to_string);
                json!({
                    "version": 1,
                    "ciphertext": vec![1, 2],
                    "lineage_hint": "Universal",
                    "timestamp": Utc::now(),
                    "signature": []
                })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::with_family_id(&path, "my-family").await.unwrap();
    let _ = provider.encrypt_for_lineage(b"hello", LineageHint::Universal).await.unwrap();
    assert_eq!(captured.lock().unwrap().as_deref(), Some("my-family"));
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn revoke_relay_succeeds_on_ok_rpc() {
    let path = temp_socket_path("revoke");
    let _ = std::fs::remove_file(&path);
    spawn_mock_security_server(
        path.clone(),
        Arc::new(|method, _| {
            if method == "relay.revoke" {
                json!({ "ok": true })
            } else {
                json!({})
            }
        }),
    )
    .await;

    let provider = ProductionSecurityProvider::new(&path).await.unwrap();
    provider.revoke_relay("sess-123").await.unwrap();
    let _ = std::fs::remove_file(&path);
}
