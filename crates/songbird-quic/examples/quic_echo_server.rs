// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "example code"
)]

//! QUIC echo server example (pure Rust, security provider crypto delegation).
//!
//! Start with: cargo run --example `quic_echo_server`
//! Test with:  cargo run --example `quic_echo_client`

use songbird_quic::{QuicConfig, QuicServer};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting QUIC echo server (native engine)");

    let config = QuicConfig::new()
        .with_idle_timeout(std::time::Duration::from_secs(60))
        .with_0rtt(true)
        .with_migration(true);

    info!("Neural API socket: {:?}", config.neural_api_socket);

    let server = QuicServer::new("[::]:4433", config).await?;

    info!("QUIC server listening on {}", server.local_addr());
    info!("Features: 0-RTT, Migration, Multiplexing, security provider crypto");

    let mut incoming = server.accept();

    while let Some(conn) = incoming.recv().await {
        info!("New connection from {}", conn.remote_address().await);

        tokio::spawn(async move {
            loop {
                match conn.accept_bi() {
                    Ok(mut stream) => {
                        info!("New stream from {}", conn.remote_address().await);
                        tokio::spawn(async move {
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
                                        info!("Stream closed");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("Read error: {}", e);
                                        break;
                                    }
                                }
                            }
                        });
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
