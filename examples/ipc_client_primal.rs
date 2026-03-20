// SPDX-License-Identifier: AGPL-3.0-only
//! Complete Primal IPC Example - NO Songbird Imports!
//!
//! This example demonstrates a complete primal implementation that:
//! 1. Registers itself with Songbird
//! 2. Discovers other services by capability
//! 3. Connects to discovered services
//! 4. Provides its own services
//!
//! ## TRUE PRIMAL Architecture
//!
//! - ✅ NO `songbird-universal-ipc` import
//! - ✅ NO Songbird code embedded
//! - ✅ Uses standard `tokio::net::UnixStream`
//! - ✅ Self-knowledge only (no hardcoded dependencies)
//! - ✅ Runtime discovery and capability-based communication
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
//!    cargo run --example ipc_client_primal
//!    ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// JSON-RPC 2.0 Request
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Example Primal - Demonstrates TRUE PRIMAL architecture
struct ExamplePrimal {
    primal_id: String,
    capabilities: Vec<String>,
    socket_path: PathBuf,
    songbird_socket: String,
}

impl ExamplePrimal {
    /// Create a new example primal
    fn new(primal_id: &str) -> Self {
        Self {
            primal_id: primal_id.to_string(),
            capabilities: vec!["example".to_string(), "demo".to_string()],
            socket_path: PathBuf::from(format!("/tmp/primal-{}.sock", primal_id)),
            songbird_socket: std::env::var("SONGBIRD_IPC_SOCKET")
                .unwrap_or_else(|_| "/tmp/primal-songbird.sock".to_string()),
        }
    }

    /// Register with Songbird IPC service
    async fn register(&self) -> Result<()> {
        println!("📝 Registering with Songbird IPC service");
        println!("   Primal ID: {}", self.primal_id);
        println!("   Capabilities: {:?}", self.capabilities);
        println!("   Endpoint: {}", self.socket_path.display());

        // Connect to Songbird
        let mut stream = UnixStream::connect(&self.songbird_socket)
            .await
            .context("Failed to connect to Songbird")?;

        // Create registration request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ipc.register".to_string(),
            params: json!({
                "primal_id": self.primal_id,
                "capabilities": self.capabilities,
                "endpoint": self.socket_path.to_string_lossy()
            }),
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
        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

        if let Some(error) = response.error {
            anyhow::bail!("Registration failed: {}", error.message);
        }

        println!("✅ Registered successfully");
        Ok(())
    }

    /// Start listening for connections
    fn start_server(&self) -> Result<()> {
        println!("🚀 Starting primal server");
        println!("   Listening on: {}", self.socket_path.display());

        // Remove old socket if exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Create Unix socket listener
        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        println!("✅ Server started, waiting for connections...");

        // Accept connections (in real implementation, this would run in background)
        // For this example, we'll just demonstrate the setup
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream).await {
                                eprintln!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle incoming connection
    async fn handle_connection(mut stream: UnixStream) -> Result<()> {
        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Client disconnected
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Parse request
                    let request: JsonRpcRequest = match serde_json::from_str(&line) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("Failed to parse request: {}", e);
                            continue;
                        }
                    };

                    // Handle request (simple echo example)
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(json!({
                            "method": request.method,
                            "message": "Example primal received your request"
                        })),
                        error: None,
                        id: request.id.into(),
                    };

                    // Send response
                    let response_json = serde_json::to_string(&response)?;
                    writer.write_all(response_json.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Discover other services by capability
    async fn discover(&self, capability: &str) -> Result<Vec<String>> {
        println!("🔍 Discovering services with capability: {}", capability);

        // Connect to Songbird
        let mut stream = UnixStream::connect(&self.songbird_socket)
            .await
            .context("Failed to connect to Songbird")?;

        // Create discovery request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ipc.discover".to_string(),
            params: json!({ "capability": capability }),
            id: 2,
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
        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

        if let Some(error) = response.error {
            anyhow::bail!("Discovery failed: {}", error.message);
        }

        let result = response.result.context("No result")?;
        let providers = result["providers"].as_array().context("No providers")?;

        let primal_ids: Vec<String> =
            providers.iter().filter_map(|p| p["primal_id"].as_str().map(String::from)).collect();

        println!("   Found {} provider(s): {:?}", primal_ids.len(), primal_ids);

        Ok(primal_ids)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌍 Complete Primal IPC Example - TRUE PRIMAL Architecture");
    println!("   NO Songbird imports! Pure tokio + JSON-RPC!");
    println!();

    // Create example primal
    let primal = ExamplePrimal::new("example-primal");

    // Step 1: Start server (listen for connections)
    primal.start_server()?;

    // Step 2: Register with Songbird
    primal.register().await?;

    println!();

    // Step 3: Discover other services
    println!("📋 Discovering other services...");
    let ipc_services = primal.discover("ipc").await?;
    println!("   IPC services: {:?}", ipc_services);

    println!();

    // Step 4: Keep running (in real implementation)
    println!("✅ Primal is now running!");
    println!("   - Registered with Songbird");
    println!("   - Listening for connections");
    println!("   - Can discover other services");
    println!();
    println!("🎉 TRUE PRIMAL Architecture verified:");
    println!("   - Zero Songbird code embedded");
    println!("   - Self-knowledge only (knows own ID and capabilities)");
    println!("   - Runtime discovery (finds others by capability)");
    println!("   - Standard tokio::net::UnixStream");
    println!("   - Standard JSON-RPC 2.0 protocol");

    // Keep running for a bit to demonstrate
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Cleanup
    if primal.socket_path.exists() {
        std::fs::remove_file(&primal.socket_path)?;
    }

    Ok(())
}
