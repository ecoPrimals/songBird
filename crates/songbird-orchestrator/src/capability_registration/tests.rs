// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::payload::{capability_registration_params, capability_unregister_params};
use super::*;
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixListener;

#[test]
fn test_for_testing_sets_explicit_paths() {
    let c = CapabilityRegistrationConfig::for_testing("/neural/path.sock", "/songbird/app.sock");
    assert_eq!(c.neural_socket, "/neural/path.sock");
    assert_eq!(c.songbird_socket, "/songbird/app.sock");
    assert_eq!(c.primal_id, "songbird");
}

#[test]
fn capability_registration_params_jsonrpc_and_operations() {
    let v = capability_registration_params("p1", "/sock/a", "fam-x", "9.9.9");
    assert_eq!(v["jsonrpc"], "2.0");
    assert_eq!(v["method"], "capability.register");
    assert_eq!(v["params"]["primal_id"], "p1");
    assert_eq!(v["params"]["capability"], "secure_http");
    assert_eq!(v["params"]["socket_path"], "/sock/a");
    let ops = v["params"]["operations"].as_array().expect("operations array");
    assert_eq!(ops.len(), 6);
    assert_eq!(ops[0], "http.get");
    assert_eq!(ops[5], "http.request");
    assert_eq!(v["params"]["metadata"]["family_id"], "fam-x");
    assert_eq!(v["params"]["metadata"]["version"], "9.9.9");
    assert_eq!(v["params"]["metadata"]["tls_version"], "1.3");
    assert_eq!(v["params"]["metadata"]["provider"], "songbird");
    assert_eq!(v["id"], 1);
}

#[test]
fn capability_unregister_params_matches_contract() {
    let v = capability_unregister_params("primal-z");
    assert_eq!(v["jsonrpc"], "2.0");
    assert_eq!(v["method"], "capability.unregister");
    assert_eq!(v["params"]["primal_id"], "primal-z");
    assert_eq!(v["params"]["capability"], "secure_http");
    assert_eq!(v["id"], 2);
}

#[test]
fn capability_registration_params_embeds_metadata_and_escapes_json() {
    let v = capability_registration_params("p\"id", "/path/with space", "fam:beta", "0.0.1-pre");
    assert_eq!(v["params"]["primal_id"], "p\"id");
    assert_eq!(v["params"]["socket_path"], "/path/with space");
    assert_eq!(v["params"]["metadata"]["family_id"], "fam:beta");
    assert_eq!(v["params"]["metadata"]["version"], "0.0.1-pre");
}

#[test]
fn capability_unregister_empty_primal_id_still_well_formed() {
    let v = capability_unregister_params("");
    assert_eq!(v["params"]["primal_id"], "");
    assert_eq!(v["method"], "capability.unregister");
}

#[test]
fn test_config_defaults() {
    let config = CapabilityRegistrationConfig {
        neural_socket: "/tmp/biomeos/neural-api.sock".to_string(),
        songbird_socket: "/tmp/songbird.sock".to_string(),
        primal_id: "songbird".to_string(),
    };
    assert_eq!(config.neural_socket, "/tmp/biomeos/neural-api.sock");
    assert_eq!(config.primal_id, "songbird");
}

#[tokio::test]
async fn test_registration_with_unavailable_neural_api_succeeds() {
    let config = CapabilityRegistrationConfig::for_testing(
        "/tmp/nonexistent-neural-api-sock-unique-12345.sock",
        "/tmp/test-songbird-unique.sock",
    );

    let result = register_capabilities_with(&config).await;
    if let Err(ref e) = result {
        eprintln!("Registration failed with error: {}", e);
    }
    assert!(
        result.is_ok(),
        "Registration should succeed gracefully even when Neural API is unavailable"
    );
}

#[tokio::test]
async fn test_unregistration_with_unavailable_neural_api_succeeds() {
    let result = unregister_capabilities_with("/tmp/nonexistent-neural-api.sock", "songbird").await;
    assert!(result.is_ok());
}

#[tokio::test]
#[cfg(unix)]
async fn test_check_neural_api_with_mock_server() {
    let socket_path = "/tmp/test-neural-api-check-concurrent.sock";

    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    tokio::task::yield_now().await;

    let available = check_neural_api_available_at(socket_path).await;
    assert!(available);

    let _ = std::fs::remove_file(socket_path);
}

#[test]
fn test_xdg_neural_api_socket_resolution() {
    let resolve = |xdg_dir: Option<&str>| -> String {
        match xdg_dir {
            Some(dir) => format!("{}/biomeos/neural-api.sock", dir),
            None => "/tmp/biomeos/neural-api.sock".to_string(),
        }
    };

    assert_eq!(resolve(Some("/run/user/1000")), "/run/user/1000/biomeos/neural-api.sock",);
    assert_eq!(resolve(None), "/tmp/biomeos/neural-api.sock",);
}

#[test]
fn test_primal_id_no_family_suffix() {
    let config = CapabilityRegistrationConfig {
        neural_socket: String::new(),
        songbird_socket: String::new(),
        primal_id: "songbird".to_string(),
    };
    assert_eq!(config.primal_id, "songbird");
    assert!(!config.primal_id.contains("-nat0"));
}

#[tokio::test]
#[cfg(unix)]
async fn test_xdg_registration_with_xdg_socket() {
    let temp_dir = std::env::temp_dir().join("test-cap-reg-xdg-concurrent");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let neural_socket = biomeos_dir.join("neural-api.sock");
    let songbird_socket = biomeos_dir.join("songbird.sock");

    let _ = std::fs::remove_file(&neural_socket);
    let _ = std::fs::remove_file(&songbird_socket);

    let neural_path = neural_socket.clone();
    let listener = UnixListener::bind(&neural_path).unwrap();
    let _server_task = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
            }
        }
    });

    tokio::task::yield_now().await;

    let config = CapabilityRegistrationConfig::for_testing(
        neural_socket.to_str().unwrap(),
        songbird_socket.to_str().unwrap(),
    );
    let result = register_capabilities_with(&config).await;
    assert!(result.is_ok(), "Registration with XDG socket should succeed");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
#[cfg(unix)]
async fn test_check_neural_api_with_xdg_socket() {
    let temp_dir = std::env::temp_dir().join("test-neural-check-xdg-concurrent");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let socket_path = biomeos_dir.join("neural-api.sock");
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    tokio::task::yield_now().await;

    let available = check_neural_api_available_at(socket_path.to_str().unwrap()).await;
    assert!(available, "Should find Neural API at XDG path");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
#[cfg(unix)]
async fn test_e2e_full_registration_lifecycle() {
    let socket_path = "/tmp/test-neural-e2e-concurrent.sock";
    let songbird_socket = "/tmp/test-songbird-e2e-concurrent.sock";

    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::remove_file(songbird_socket);

    let listener = UnixListener::bind(socket_path).unwrap();
    let server_task = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    assert!(line.contains("capability.register"));
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;

    let config = CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket);
    let result = register_capabilities_with(&config).await;
    assert!(result.is_ok(), "E2E registration should succeed");

    drop(server_task);
    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::remove_file(songbird_socket);
}

#[tokio::test]
#[cfg(unix)]
async fn test_e2e_registration_and_immediate_unregistration() {
    let socket_path = "/tmp/test-neural-e2e-unreg-concurrent.sock";
    let songbird_socket = "/tmp/test-songbird-e2e-unreg-concurrent.sock";

    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::remove_file(songbird_socket);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..12 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;

    let config = CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket);
    let reg_result = register_capabilities_with(&config).await;
    assert!(reg_result.is_ok());

    let unreg_result = unregister_capabilities_with(socket_path, "songbird").await;
    assert!(unreg_result.is_ok());

    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::remove_file(songbird_socket);
}

fn chaos_config(socket_path: &str, songbird_socket: &str) -> CapabilityRegistrationConfig {
    CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket)
}

#[tokio::test]
#[cfg(unix)]
async fn test_chaos_socket_disappears_during_registration() {
    let socket_path = "/tmp/test-neural-chaos-disappear-c.sock";
    let songbird_socket = "/tmp/test-songbird-chaos-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let server_task = tokio::spawn(async move {
        for i in 0..2 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
            if i == 1 {
                break;
            }
        }
    });

    tokio::task::yield_now().await;
    let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
    assert!(result.is_ok(), "Should handle socket disappearance gracefully");

    drop(server_task);
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
#[cfg(unix)]
async fn test_chaos_slow_neural_api_responses() {
    let socket_path = "/tmp/test-neural-chaos-slow-c.sock";
    let songbird_socket = "/tmp/test-songbird-chaos-slow-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;
    let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
    assert!(result.is_ok(), "Should handle slow responses");

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
#[cfg(unix)]
async fn test_chaos_neural_api_restarts_during_operation() {
    let socket_path = "/tmp/test-neural-chaos-restart-c.sock";
    let songbird_socket = "/tmp/test-songbird-chaos-restart-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener1 = UnixListener::bind(socket_path).unwrap();
    let _server_task1 = tokio::spawn(async move {
        for _ in 0..2 {
            if let Ok((mut stream, _)) = listener1.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;

    let config = chaos_config(socket_path, songbird_socket);
    let result1 = register_capabilities_with(&config).await;
    assert!(result1.is_ok(), "Should handle partial registration");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let _ = std::fs::remove_file(socket_path);

    let listener2 = UnixListener::bind(socket_path).unwrap();
    let _server_task2 = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener2.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;
    let result2 = register_capabilities_with(&config).await;
    assert!(result2.is_ok(), "Should self-heal by re-registering after restart");

    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
#[cfg(unix)]
async fn test_fault_malformed_json_response() {
    let socket_path = "/tmp/test-neural-fault-malformed-c.sock";
    let songbird_socket = "/tmp/test-songbird-fault-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let _ = stream.write_all(b"INVALID JSON {{{\n").await;
                }
            }
        }
    });

    tokio::task::yield_now().await;
    let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
    assert!(result.is_ok(), "Should handle malformed JSON gracefully");
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
#[cfg(unix)]
async fn test_fault_neural_api_returns_errors() {
    let socket_path = "/tmp/test-neural-fault-error-c.sock";
    let songbird_socket = "/tmp/test-songbird-fault-error-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;
    let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
    assert!(result.is_ok(), "Should handle JSON-RPC errors gracefully");
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
#[cfg(unix)]
async fn test_fault_connection_drops_mid_request() {
    let socket_path = "/tmp/test-neural-fault-drop-c.sock";
    let songbird_socket = "/tmp/test-songbird-fault-drop-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..6 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    drop(stream);
                }
            }
        }
    });

    tokio::task::yield_now().await;
    let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
    assert!(result.is_ok(), "Should handle dropped connections gracefully");
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_fault_permission_denied_on_socket() {
    let config = chaos_config("/root/nonexistent/test-neural.sock", "/tmp/test-songbird-perm.sock");
    let result = register_capabilities_with(&config).await;
    assert!(result.is_ok(), "Should handle permission denied gracefully");
}

#[tokio::test]
#[cfg(unix)]
async fn test_fault_concurrent_registrations() {
    let socket_path = "/tmp/test-neural-concurrent-c.sock";
    let songbird_socket = "/tmp/test-songbird-concurrent-c.sock";
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let _server_task = tokio::spawn(async move {
        for _ in 0..18 {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });

    tokio::task::yield_now().await;

    let config = chaos_config(socket_path, songbird_socket);
    let mut handles = vec![];
    for _ in 0..3 {
        let c = config.clone();
        let handle = tokio::spawn(async move { register_capabilities_with(&c).await });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent registrations should be safe");
    }

    let _ = std::fs::remove_file(socket_path);
}
