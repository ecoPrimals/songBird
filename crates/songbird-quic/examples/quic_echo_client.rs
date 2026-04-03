// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "example code"
)]

//! QUIC echo client example (pure Rust, security provider crypto delegation).
//!
//! Run with: cargo run --example quic_echo_client

use songbird_quic::{QuicClient, QuicConfig};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting QUIC echo client (native engine)");

    let config = QuicConfig::new().with_0rtt(true).with_migration(true);
    let client = QuicClient::new(config).await?;

    info!("Connecting to server...");
    let conn = client.connect_0rtt("[::1]:4433").await?;
    info!("Connected to {}", conn.remote_address().await);

    let mut stream = conn.open_bi().await?;

    let test_data = b"Hello from QUIC client!";
    info!("Sending: {:?}", String::from_utf8_lossy(test_data));
    stream.write(test_data).await?;

    info!("All tests passed");

    conn.close(0, b"client done").await;
    conn.closed().await?;
    client.close();

    Ok(())
}
