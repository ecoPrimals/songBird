//! Tower Atomic Server Example
//!
//! This example demonstrates a simple JSON-RPC server using Tower Atomic
//! over Universal IPC. The server provides basic math operations.
//!
//! Run this example:
//! ```bash
//! cargo run --example tower_atomic_server
//! ```
//!
//! Then in another terminal:
//! ```bash
//! cargo run --example tower_atomic_client
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use songbird_universal_ipc::tower_atomic::{JsonRpcHandler, TowerAtomicServer};
use songbird_universal_ipc::{error::IpcResult, ipc};
use tracing::{info, Level};
use tracing_subscriber;

/// Math service that provides basic arithmetic operations
struct MathService;

#[async_trait]
impl JsonRpcHandler for MathService {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        info!("Handling method: {}", method);

        match method {
            "add" => {
                let a = params["a"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'a'".to_string())?;
                let b = params["b"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'b'".to_string())?;
                let result = a + b;
                info!("add({}, {}) = {}", a, b, result);
                Ok(json!(result))
            }
            "subtract" => {
                let a = params["a"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'a'".to_string())?;
                let b = params["b"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'b'".to_string())?;
                let result = a - b;
                info!("subtract({}, {}) = {}", a, b, result);
                Ok(json!(result))
            }
            "multiply" => {
                let a = params["a"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'a'".to_string())?;
                let b = params["b"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'b'".to_string())?;
                let result = a * b;
                info!("multiply({}, {}) = {}", a, b, result);
                Ok(json!(result))
            }
            "divide" => {
                let a = params["a"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'a'".to_string())?;
                let b = params["b"]
                    .as_i64()
                    .ok_or_else(|| "Missing or invalid parameter 'b'".to_string())?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                let result = a / b;
                info!("divide({}, {}) = {}", a, b, result);
                Ok(json!(result))
            }
            "info" => {
                info!("Returning service info");
                Ok(json!({
                    "name": "MathService",
                    "version": "1.0.0",
                    "methods": ["add", "subtract", "multiply", "divide", "info"]
                }))
            }
            _ => {
                info!("Unknown method: {}", method);
                Err(format!("Unknown method: {}", method))
            }
        }
    }
}

#[tokio::main]
async fn main() -> IpcResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting Tower Atomic Math Service");

    // Initialize Universal IPC
    ipc::init()?;

    // Register service with capabilities
    let endpoint = ipc::register(
        "math-service",
        vec!["math".to_string(), "arithmetic".to_string()],
    )
    .await?;

    info!("✅ Registered at: {}", endpoint.path);
    info!("📡 Listening for JSON-RPC requests...");
    info!("   Available methods: add, subtract, multiply, divide, info");

    // Create and start server
    let server = TowerAtomicServer::new(MathService);
    server.serve(endpoint).await?;

    Ok(())
}

