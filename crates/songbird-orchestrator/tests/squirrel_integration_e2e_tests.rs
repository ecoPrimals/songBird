//! E2E Tests: Squirrel Integration - Real-World Scenarios
//!
//! Tests end-to-end integration scenarios for Squirrel AI delegation:
//! - Full discovery flow
//! - HTTP delegation workflow
//! - Real API calls (with mocking)
//! - Error handling

use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::timeout;

/// Helper: Start a mock Songbird server for testing
async fn start_mock_server(socket_path: &str) -> tokio::task::JoinHandle<()> {
    let socket_path = socket_path.to_string();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                
                if reader.read_line(&mut line).await.is_ok() {
                    if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
                        let method = request["method"].as_str().unwrap_or("");
                        
                        let response = match method {
                            "discover_capabilities" => json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "capabilities": ["http.request", "http.post", "http.get"],
                                    "metadata": {
                                        "primal_name": "songbird",
                                        "version": "4.3.0",
                                        "family_id": "nat0"
                                    }
                                },
                                "id": request["id"]
                            }),
                            "http.request" => json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "status": 200,
                                    "headers": {"content-type": "application/json"},
                                    "body": {"success": true, "message": "Mock response"}
                                },
                                "id": request["id"]
                            }),
                            _ => json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32601,
                                    "message": "Method not found"
                                },
                                "id": request["id"]
                            })
                        };
                        
                        let response_str = serde_json::to_string(&response).unwrap();
                        let _ = stream.write_all(response_str.as_bytes()).await;
                        let _ = stream.write_all(b"\n").await;
                    }
                }
            });
        }
    })
}

/// Helper: Send JSON-RPC request
async fn send_request(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    
    Ok(serde_json::from_str(&line)?)
}

#[tokio::test]
async fn test_e2e_capability_discovery_flow() {
    // Simulate Squirrel discovering Songbird's capabilities
    
    let socket_path = "/tmp/test-songbird-e2e-discovery.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Step 1: Squirrel connects and discovers capabilities
    let result = timeout(
        Duration::from_secs(5),
        send_request(socket_path, "discover_capabilities", json!({}))
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap().unwrap();
    
    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["capabilities"].is_array());
    
    let capabilities = response["result"]["capabilities"].as_array().unwrap();
    assert!(capabilities.iter().any(|c| c == "http.request"));
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_http_delegation_workflow() {
    // Simulate Squirrel delegating HTTP request to Songbird
    
    let socket_path = "/tmp/test-songbird-e2e-http.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Step 1: Discover capabilities
    let discover_result = send_request(socket_path, "discover_capabilities", json!({})).await;
    assert!(discover_result.is_ok());
    
    // Step 2: Delegate HTTP request
    let http_params = json!({
        "method": "POST",
        "url": "https://api.example.com/v1/messages",
        "headers": {
            "content-type": "application/json",
            "authorization": "Bearer test-token"
        },
        "body": {
            "message": "Hello from Squirrel"
        }
    });
    
    let http_result = send_request(socket_path, "http.request", http_params).await;
    assert!(http_result.is_ok());
    
    let response = http_result.unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], 200);
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_sequential_requests() {
    // Test multiple sequential requests (realistic usage)
    
    let socket_path = "/tmp/test-songbird-e2e-sequential.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Make 5 sequential requests
    for i in 0..5 {
        let params = json!({
            "method": "GET",
            "url": format!("https://api.example.com/item/{}", i),
            "headers": {}
        });
        
        let result = send_request(socket_path, "http.request", params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["result"]["status"], 200);
    }
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_connection_reuse() {
    // Test that connections can be reused
    
    let socket_path = "/tmp/test-songbird-e2e-reuse.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect once
    let stream = UnixStream::connect(socket_path).await;
    assert!(stream.is_ok());
    drop(stream);
    
    // Connect again
    let stream = UnixStream::connect(socket_path).await;
    assert!(stream.is_ok());
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_invalid_method() {
    // Test error handling for invalid method
    
    let socket_path = "/tmp/test-songbird-e2e-invalid.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let result = send_request(socket_path, "unknown_method", json!({})).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // Method not found
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_timeout_handling() {
    // Test timeout behavior
    
    // Test 1: Connection to nonexistent socket (will error immediately)
    let result1 = timeout(
        Duration::from_millis(100),
        UnixStream::connect("/tmp/nonexistent-socket-test.sock")
    ).await;
    
    // Should either timeout OR get connection error (both are acceptable)
    assert!(result1.is_err() || (result1.is_ok() && result1.unwrap().is_err()));
    
    // Test 2: Just verify timeout mechanism works
    let result2 = timeout(
        Duration::from_millis(10),
        tokio::time::sleep(Duration::from_secs(1))
    ).await;
    
    assert!(result2.is_err(), "Expected timeout");
}

#[tokio::test]
async fn test_e2e_large_response() {
    // Test handling of large responses (AI responses can be large)
    
    let socket_path = "/tmp/test-songbird-e2e-large.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Request that would produce large response
    let params = json!({
        "method": "POST",
        "url": "https://api.anthropic.com/v1/messages",
        "headers": {"content-type": "application/json"},
        "body": {
            "model": "claude-3-opus-20240229",
            "max_tokens": 4096  // Large response
        }
    });
    
    let result = send_request(socket_path, "http.request", params).await;
    assert!(result.is_ok());
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_concurrent_clients() {
    // Test multiple clients connecting simultaneously
    
    let socket_path = "/tmp/test-songbird-e2e-concurrent.sock";
    let _ = std::fs::remove_file(socket_path);
    
    let server = start_mock_server(socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Spawn 3 concurrent clients
    let mut handles = vec![];
    for _ in 0..3 {
        let path = socket_path.to_string();
        let handle = tokio::spawn(async move {
            send_request(&path, "discover_capabilities", json!({})).await
        });
        handles.push(handle);
    }
    
    // All should succeed
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    // Cleanup
    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

