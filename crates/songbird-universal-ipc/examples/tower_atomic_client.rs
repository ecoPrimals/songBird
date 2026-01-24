//! Tower Atomic Client Example
//!
//! This example demonstrates a JSON-RPC client using Tower Atomic
//! over Universal IPC. The client calls methods on the math service.
//!
//! First, start the server:
//! ```bash
//! cargo run --example tower_atomic_server
//! ```
//!
//! Then run this client:
//! ```bash
//! cargo run --example tower_atomic_client
//! ```

use serde_json::json;
use songbird_universal_ipc::capability::discovery;
use songbird_universal_ipc::error::IpcResult;
use songbird_universal_ipc::ipc;
use songbird_universal_ipc::tower_atomic::TowerAtomicClient;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> IpcResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).with_target(false).init();

    info!("🔍 Tower Atomic Client Example");

    // Initialize Universal IPC
    ipc::init()?;

    // Discover math service by capability
    info!("🔍 Discovering math service...");
    let provider = discovery::discover("math").await?;
    info!("✅ Found service: {} at {}", provider.id, provider.virtual_endpoint);

    // Connect via Tower Atomic
    info!("🔌 Connecting to service...");
    let client = TowerAtomicClient::connect(&provider.virtual_endpoint).await?;
    info!("✅ Connected!");

    // Get service info
    info!("\n📋 Getting service info...");
    let info_result = client.call_no_params("info").await?;
    info!("Service info: {}", serde_json::to_string_pretty(&info_result).unwrap_or_else(|_| "Error formatting".to_string()));

    // Test addition
    info!("\n➕ Testing addition: 15 + 27");
    let add_result = client.call("add", json!({"a": 15, "b": 27})).await?;
    info!("Result: {}", add_result);

    // Test subtraction
    info!("\n➖ Testing subtraction: 100 - 42");
    let sub_result = client.call("subtract", json!({"a": 100, "b": 42})).await?;
    info!("Result: {}", sub_result);

    // Test multiplication
    info!("\n✖️  Testing multiplication: 7 * 8");
    let mul_result = client.call("multiply", json!({"a": 7, "b": 8})).await?;
    info!("Result: {}", mul_result);

    // Test division
    info!("\n➗ Testing division: 144 / 12");
    let div_result = client.call("divide", json!({"a": 144, "b": 12})).await?;
    info!("Result: {}", div_result);

    // Test error handling (division by zero)
    info!("\n⚠️  Testing error handling: 10 / 0");
    match client.call("divide", json!({"a": 10, "b": 0})).await {
        Ok(result) => info!("Unexpected success: {}", result),
        Err(e) => info!("Expected error: {}", e),
    }

    // Test unknown method
    info!("\n⚠️  Testing unknown method: 'power'");
    match client.call("power", json!({"a": 2, "b": 8})).await {
        Ok(result) => info!("Unexpected success: {}", result),
        Err(e) => info!("Expected error: {}", e),
    }

    info!("\n✅ All tests complete!");

    Ok(())
}
