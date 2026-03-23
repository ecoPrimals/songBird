// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Simple client example using universal IPC

use songbird_universal_ipc::ipc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize universal IPC
    ipc::init()?;

    println!("🚀 Starting simple client...");

    // Give server time to start (in real use, you'd use service discovery)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Connect to server
    println!("🔌 Connecting to /primal/simple-server...");
    let mut stream = ipc::connect("/primal/simple-server").await?;

    println!("✅ Connected!");

    // Send message
    let message = "Hello from client!";
    println!("📤 Sending: {message}");
    stream.write_all(message.as_bytes()).await?;

    // Read response
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;

    let response = String::from_utf8_lossy(&buf[..n]);
    println!("📥 Received: {response}");

    println!("✅ Done!");

    Ok(())
}
