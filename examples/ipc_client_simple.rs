// SPDX-License-Identifier: AGPL-3.0-only
//! Simple IPC Client Example - NO Songbird Imports!
//!
//! This example demonstrates how other primals can connect to Songbird's
//! IPC service using ONLY standard Rust libraries (tokio).
//!
//! ## TRUE PRIMAL Architecture
//!
//! - ✅ NO `songbird-universal-ipc` import
//! - ✅ NO Songbird code embedded
//! - ✅ Uses standard `tokio::net::UnixStream`
//! - ✅ Uses standard JSON-RPC 2.0 protocol
//! - ✅ Pure service-based communication
//!
//! ## Usage
//!
//! 1. Start Songbird server:
//!    ```bash
//!    cargo run -- server
//!    ```
//!
//! 2. Run this example (in another terminal):
//!    ```bash
//!    cargo run --example ipc_client_simple
//!    ```
#![allow(dead_code)] // Example code — not all fields/functions are used in every path

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// JSON-RPC 2.0 Request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
    id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

/// Simple JSON-RPC client over Unix socket
struct SimpleIpcClient {
    stream: UnixStream,
    request_id: u64,
}

impl SimpleIpcClient {
    /// Connect to Songbird's IPC service
    ///
    /// Uses standard Unix socket path (no hardcoding!)
    async fn connect() -> Result<Self> {
        // Connect to Songbird's IPC endpoint
        // Path is discovered via environment or standard location
        let socket_path = std::env::var("SONGBIRD_IPC_SOCKET")
            .unwrap_or_else(|_| "/tmp/primal-songbird.sock".to_string());

        println!("🔌 Connecting to Songbird IPC: {}", socket_path);

        let stream = UnixStream::connect(&socket_path)
            .await
            .context(format!("Failed to connect to {}", socket_path))?;

        println!("✅ Connected to Songbird IPC service");

        Ok(Self {
            stream,
            request_id: 1,
        })
    }

    /// Call a JSON-RPC method
    async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        // Create JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.request_id,
        };
        self.request_id += 1;

        // Serialize and send request
        let request_json = serde_json::to_string(&request)?;
        self.stream.write_all(request_json.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;

        // Read response
        let (reader, _writer) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&response_line).context("Failed to parse JSON-RPC response")?;

        // Check for errors
        if let Some(error) = response.error {
            anyhow::bail!("JSON-RPC error {}: {}", error.code, error.message);
        }

        // Return result
        response.result.context("No result in response")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌍 Simple IPC Client Example - TRUE PRIMAL Architecture");
    println!("   NO Songbird imports! Pure tokio + JSON-RPC!");
    println!();

    // Connect to Songbird
    let mut client = SimpleIpcClient::connect().await?;

    println!();
    println!("📋 Example 1: Register a service");
    println!("   Calling: ipc.register");

    let register_result = client
        .call(
            "ipc.register",
            json!({
                "primal_id": "example-client",
                "capabilities": ["demo", "example"],
                "endpoint": "/tmp/example-client.sock"
            }),
        )
        .await?;

    println!("   Result: {}", serde_json::to_string_pretty(&register_result)?);

    println!();
    println!("📋 Example 2: Discover services by capability");
    println!("   Calling: ipc.discover");

    let discover_result = client
        .call(
            "ipc.discover",
            json!({
                "capability": "ipc"
            }),
        )
        .await?;

    println!("   Result: {}", serde_json::to_string_pretty(&discover_result)?);

    println!();
    println!("📋 Example 3: List all services");
    println!("   Calling: ipc.list");

    let list_result = client.call("ipc.list", json!({})).await?;

    println!("   Result: {}", serde_json::to_string_pretty(&list_result)?);

    println!();
    println!("✅ All examples completed successfully!");
    println!();
    println!("🎉 TRUE PRIMAL Architecture verified:");
    println!("   - Zero Songbird code embedded");
    println!("   - Standard tokio::net::UnixStream");
    println!("   - Standard JSON-RPC 2.0 protocol");
    println!("   - Pure service-based communication");

    Ok(())
}
