// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC echo server example
//!
//! Start with: cargo run --example quic_echo_server
//! Test with: cargo run --example quic_echo_client

use songbird_quic::{QuicConfig, QuicServer};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting QUIC echo server");

    // Create configuration with runtime discovery
    let config = QuicConfig::new()
        .with_idle_timeout(std::time::Duration::from_secs(60))
        .with_0rtt(true)
        .with_migration(true);

    info!("Neural API socket: {:?}", config.neural_api_socket);

    // Bind to IPv6 dual-stack (supports both IPv4 and IPv6)
    let server = QuicServer::new("[::]:4433", config).await?;

    info!("📡 QUIC server listening on {}", server.local_addr());
    info!("Features: 0-RTT ✅  Migration ✅  Multiplexing ✅");

    // Accept connections
    let mut incoming = server.accept();

    while let Some(conn) = incoming.recv().await {
        info!("✅ New connection from {}", conn.remote_address());

        tokio::spawn(async move {
            // Handle bidirectional streams
            loop {
                match conn.accept_bi().await {
                    Ok(mut stream) => {
                        info!("📥 New stream from {}", conn.remote_address());

                        // Echo loop
                        match tokio::spawn(async move {
                            let mut buf = vec![0u8; 4096];
                            loop {
                                match stream.read(&mut buf).await {
                                    Ok(n) if n > 0 => {
                                        info!("Received {} bytes", n);
                                        if let Err(e) = stream.write(&buf[..n]).await {
                                            error!("Write error: {}", e);
                                            break;
                                        }
                                    }
                                    Ok(_) => {
                                        info!("Stream closed by client");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("Read error: {}", e);
                                        break;
                                    }
                                }
                            }
                        })
                        .await
                        {
                            Ok(()) => info!("Stream handler completed"),
                            Err(e) => error!("Stream handler panicked: {}", e),
                        }
                    }
                    Err(e) => {
                        error!("Accept stream error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    Ok(())
}
