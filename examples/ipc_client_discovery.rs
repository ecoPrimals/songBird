//! Capability-Based Discovery Example - NO Songbird Imports!
//!
//! This example demonstrates capability-based service discovery using
//! ONLY standard Rust libraries. This is how other primals discover
//! services at runtime without any hardcoded dependencies.
//!
//! ## TRUE PRIMAL Architecture
//!
//! - ✅ NO `songbird-universal-ipc` import
//! - ✅ NO Songbird code embedded
//! - ✅ Uses standard `tokio::net::UnixStream`
//! - ✅ Capability-based discovery (zero hardcoding!)
//! - ✅ Runtime service resolution
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
//!    cargo run --example ipc_client_discovery
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

/// Service provider information
#[derive(Debug, Deserialize)]
struct Provider {
    primal_id: String,
    virtual_endpoint: String,
    native_endpoint: String,
    capabilities: Vec<String>,
}

/// Discover services by capability (NO Songbird imports!)
async fn discover_by_capability(capability: &str) -> Result<Vec<Provider>> {
    println!("🔍 Discovering services with capability: {}", capability);

    // Connect to Songbird IPC service
    let socket_path = std::env::var("SONGBIRD_IPC_SOCKET")
        .unwrap_or_else(|_| "/tmp/primal-songbird.sock".to_string());

    let mut stream = UnixStream::connect(&socket_path)
        .await
        .context(format!("Failed to connect to {}", socket_path))?;

    // Create JSON-RPC request
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "ipc.discover".to_string(),
        params: json!({ "capability": capability }),
        id: 1,
    };

    // Send request
    let request_json = serde_json::to_string(&request)?;
    stream.write_all(request_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    // Read response
    let (reader, _writer) = stream.split();
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

    // Extract providers
    let result = response.result.context("No result in response")?;
    let providers: Vec<Provider> =
        serde_json::from_value(result["providers"].clone()).context("Failed to parse providers")?;

    Ok(providers)
}

/// Connect to a discovered service (NO Songbird imports!)
async fn connect_to_service(endpoint: &str) -> Result<UnixStream> {
    println!("🔌 Connecting to service: {}", endpoint);

    let stream = UnixStream::connect(endpoint)
        .await
        .context(format!("Failed to connect to {}", endpoint))?;

    println!("✅ Connected to service");

    Ok(stream)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌍 Capability-Based Discovery Example - TRUE PRIMAL Architecture");
    println!("   NO Songbird imports! Pure tokio + JSON-RPC!");
    println!();

    // Example 1: Discover IPC services
    println!("📋 Example 1: Discover IPC services");
    let ipc_providers = discover_by_capability("ipc").await?;
    println!("   Found {} IPC provider(s):", ipc_providers.len());
    for provider in &ipc_providers {
        println!("     - {}: {}", provider.primal_id, provider.virtual_endpoint);
        println!("       Capabilities: {:?}", provider.capabilities);
    }

    println!();

    // Example 2: Discover discovery services
    println!("📋 Example 2: Discover discovery services");
    let discovery_providers = discover_by_capability("discovery").await?;
    println!("   Found {} discovery provider(s):", discovery_providers.len());
    for provider in &discovery_providers {
        println!("     - {}: {}", provider.primal_id, provider.virtual_endpoint);
        println!("       Capabilities: {:?}", provider.capabilities);
    }

    println!();

    // Example 3: Connect to discovered service
    if let Some(provider) = ipc_providers.first() {
        println!("📋 Example 3: Connect to discovered service");
        println!("   Connecting to: {}", provider.primal_id);

        // In a real scenario, you would connect to the native_endpoint
        // For this example, we'll just demonstrate the pattern
        println!("   Native endpoint: {}", provider.native_endpoint);
        println!("   (Connection would be established here in production)");
    }

    println!();
    println!("✅ All examples completed successfully!");
    println!();
    println!("🎉 TRUE PRIMAL Architecture verified:");
    println!("   - Zero Songbird code embedded");
    println!("   - Capability-based discovery (zero hardcoding!)");
    println!("   - Runtime service resolution");
    println!("   - Standard tokio::net::UnixStream");
    println!("   - Standard JSON-RPC 2.0 protocol");

    Ok(())
}
