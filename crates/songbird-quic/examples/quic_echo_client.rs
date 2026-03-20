// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC echo client example
//!
//! Run with: cargo run --example quic_echo_client

use songbird_quic::{QuicClient, QuicConfig};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting QUIC echo client");

    // Create configuration
    let config = QuicConfig::new().with_0rtt(true).with_migration(true);

    // Create client
    let client = QuicClient::new(config).await?;

    info!("📞 Connecting to server...");

    // Connect with 0-RTT if possible
    let conn = client.connect_0rtt("[::1]:4433").await?;

    info!("✅ Connected to {}", conn.remote_address());

    // Open bidirectional stream
    let mut stream = conn.open_bi().await?;

    // Send test data
    let test_data = "Hello from QUIC client!".as_bytes();
    info!("📤 Sending: {:?}", String::from_utf8_lossy(test_data));
    stream.write(test_data).await?;

    // Receive echo
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let response = String::from_utf8_lossy(&buf[..n]);
    info!("📥 Received: {:?}", response);

    // Test multiplexing - open multiple streams simultaneously
    info!("🔀 Testing stream multiplexing...");

    let mut streams = Vec::new();
    for i in 0..5 {
        let mut stream = conn.open_bi().await?;
        let msg = format!("Stream {} message", i);
        stream.write(msg.as_bytes()).await?;
        streams.push(stream);
    }

    for (i, mut stream) in streams.into_iter().enumerate() {
        let n = stream.read(&mut buf).await?;
        info!("Stream {}: {:?}", i, String::from_utf8_lossy(&buf[..n]));
    }

    info!("✅ All tests passed");

    // Close connection gracefully
    conn.close(0, b"client done");
    conn.closed().await?;

    Ok(())
}
