// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! E2E Tests for Unix Socket JSON-RPC IPC
//!
//! v3.19.3: Integration tests with real Unix socket connections
//!
//! ## Test Coverage
//!
//! 1. Server lifecycle (start/stop)
//! 2. Client connection
//! 3. `discover_by_family` API
//! 4. `create_genetic_tunnel` API
//! 5. `announce_capabilities` API
//! 6. Error handling

mod common;
use common::event_helpers::wait_for;

use anyhow::Result;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// Simple Unix socket JSON-RPC client for testing
struct UnixSocketClient {
    stream: UnixStream,
}

impl UnixSocketClient {
    /// Connect to Unix socket
    fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self {
            stream,
        })
    }

    /// Send JSON-RPC request and receive response
    fn call(&mut self, method: &str, params: &Value) -> Result<Value> {
        // Build JSON-RPC 2.0 request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Send request
        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stream, "{request_str}")?;
        self.stream.flush()?;

        // Read response
        let mut reader = BufReader::new(&self.stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        // Parse response
        let response: Value = serde_json::from_str(&response_line)?;

        // Check for error
        if let Some(error) = response.get("error") {
            anyhow::bail!("JSON-RPC error: {error}");
        }

        // Return result
        response.get("result").cloned().ok_or_else(|| anyhow::anyhow!("No result in response"))
    }
}

/// Test helper: Wait for socket file to exist
async fn wait_for_socket(socket_path: &str, timeout_secs: u64) -> Result<()> {
    let socket_path_owned = socket_path.to_string();

    // Event-driven: poll for socket file existence
    wait_for(|| Path::new(&socket_path_owned).exists(), Duration::from_secs(timeout_secs)).await?;

    // Yield to let server finish binding (socket file exists → listener is ready)
    tokio::task::yield_now().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_unix_socket_connection() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Wait for socket to exist (server should be started manually)
    println!("Waiting for socket: {socket_path}");
    wait_for_socket(socket_path, 5).await?;

    // Connect to Unix socket
    println!("Connecting to socket...");
    let _client = UnixSocketClient::connect(socket_path)?;
    println!("✅ Connected successfully!");

    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_discover_by_family_api() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Connect
    wait_for_socket(socket_path, 5).await?;
    let mut client = UnixSocketClient::connect(socket_path)?;

    // Call discover_by_family
    println!("Calling discover_by_family...");
    let result = client.call(
        "discover_by_family",
        &json!({
            "family_tags": ["nat0", "lan0"],
            "timeout_ms": 5000
        }),
    )?;

    println!("Response: {}", serde_json::to_string_pretty(&result)?);

    // Verify response structure
    assert!(result.get("nodes").is_some());
    let nodes = result["nodes"].as_array().unwrap();
    println!("✅ Discovered {} nodes", nodes.len());

    // Verify node structure
    if let Some(node) = nodes.first() {
        assert!(node.get("node_id").is_some());
        assert!(node.get("capabilities").is_some());
        assert!(node.get("genetic_families").is_some());
        println!("✅ Node structure valid");
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_create_genetic_tunnel_api() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Connect
    wait_for_socket(socket_path, 5).await?;
    let mut client = UnixSocketClient::connect(socket_path)?;

    // Call create_genetic_tunnel
    println!("Calling create_genetic_tunnel...");
    let result = client.call(
        "create_genetic_tunnel",
        &json!({
            "peer_node_id": "test-peer",
            "peer_endpoint": "https://localhost:8081",
            "genetic_proof": {
                "family_id": "nat0",
                "parent_seed_hash": "test123",
                "relationship": "sibling"
            }
        }),
    )?;

    println!("Response: {}", serde_json::to_string_pretty(&result)?);

    // Verify response structure
    assert!(result.get("tunnel_id").is_some());
    assert!(result.get("status").is_some());
    println!("✅ Tunnel response valid");

    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_announce_capabilities_api() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Connect
    wait_for_socket(socket_path, 5).await?;
    let mut client = UnixSocketClient::connect(socket_path)?;

    // Call announce_capabilities
    println!("Calling announce_capabilities...");
    let result = client.call(
        "announce_capabilities",
        &json!({
            "capabilities": ["storage", "compute"],
            "sub_federations": ["gaming"],
            "genetic_families": ["nat0"]
        }),
    )?;

    println!("Response: {}", serde_json::to_string_pretty(&result)?);

    // Verify response structure
    assert!(result.get("status").is_some());
    assert!(result.get("broadcasting").is_some());
    println!("✅ Announce response valid");

    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_invalid_method() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Connect
    wait_for_socket(socket_path, 5).await?;
    let mut client = UnixSocketClient::connect(socket_path)?;

    // Call invalid method
    println!("Calling invalid method...");
    let result = client.call("nonexistent_method", &json!({}));

    // Should return error
    assert!(result.is_err());
    println!("✅ Error handling works");

    Ok(())
}

#[tokio::test]
#[ignore = "requires Unix domain socket"] // Requires server to be running
async fn test_invalid_params() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Connect
    wait_for_socket(socket_path, 5).await?;
    let mut client = UnixSocketClient::connect(socket_path)?;

    // Call with invalid params
    println!("Calling with invalid params...");
    let result = client.call(
        "discover_by_family",
        &json!({
            "wrong_field": "wrong_value"
        }),
    );

    // Should return error
    assert!(result.is_err());
    println!("✅ Parameter validation works");

    Ok(())
}

#[tokio::test]
#[ignore = "manual concurrent connection test; requires Unix domain socket"] // Manual test
async fn test_concurrent_connections() -> Result<()> {
    let socket_path = "/tmp/songbird-test.sock";

    // Wait for socket
    wait_for_socket(socket_path, 5).await?;

    // Spawn multiple concurrent connections
    let mut handles = vec![];

    for i in 0..5 {
        let socket_path = socket_path.to_string();
        let handle = tokio::spawn(async move {
            let mut client = UnixSocketClient::connect(&socket_path)?;
            let result = client.call(
                "discover_by_family",
                &json!({
                    "family_tags": ["nat0"],
                    "timeout_ms": 5000
                }),
            )?;
            println!("Client {i} completed");
            Ok::<_, anyhow::Error>(result)
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.await??;
    }

    println!("✅ Concurrent connections work");
    Ok(())
}
