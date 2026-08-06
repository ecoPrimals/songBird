// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared accept loops for tarpc (bincode + length-delimited frames).
//!
//! Provides both TCP and UDS (Unix domain socket) accept loops.
//!
//! Implemented as macros so each call site uses a concrete `SongbirdRpc` impl; rustc can
//! then prove response futures are `Send` for `tokio::spawn` (async methods in traits do not
//! imply `Send` when the service type is generic).

/// Listen on TCP `addr` and serve a concrete `SongbirdRpc` implementation until the process exits.
macro_rules! run_tarpc_accept_loop {
    ($addr:expr, $server:expr, $startup_log:expr) => {
        async move {
            use futures_util::StreamExt;
            use tarpc::server::Channel;
            use tracing::{debug, error, info};

            let addr = $addr;
            let server = $server;
            let startup_log: String = $startup_log;

            info!("{}", startup_log);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            info!("✅ tarpc server listening on {}", addr);

            loop {
                let (stream, peer_addr) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                        continue;
                    }
                };

                debug!("New tarpc connection from {}", peer_addr);

                let server = server.clone();

                tokio::spawn(async move {
                    let transport = tarpc::serde_transport::new(
                        tarpc::tokio_util::codec::LengthDelimitedCodec::builder()
                            .max_frame_length(16 * 1024 * 1024)
                            .new_framed(stream),
                        tarpc::tokio_serde::formats::Bincode::default(),
                    );

                    let channel = tarpc::server::BaseChannel::with_defaults(transport);

                    channel
                        .execute(server.serve())
                        .for_each(|response| async move {
                            tokio::spawn(response);
                        })
                        .await;

                    debug!("tarpc connection from {} closed", peer_addr);
                });
            }
        }
    };
}

/// Listen on a Unix domain socket and serve a concrete `SongbirdRpc` implementation.
///
/// Dual-socket pattern (G64 cephalization): JSON-RPC on `.sock`, tarpc on `.tarpc.sock`.
/// UDS eliminates TCP overhead for intra-gate primal-to-primal calls (sub-ms latency).
#[cfg(unix)]
macro_rules! run_tarpc_uds_accept_loop {
    ($path:expr, $server:expr, $startup_log:expr) => {
        async move {
            use futures_util::StreamExt;
            use tarpc::server::Channel;
            use tracing::{debug, error, info};

            let path: std::path::PathBuf = $path;
            let server = $server;
            let startup_log: String = $startup_log;

            info!("{}", startup_log);

            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let listener = tokio::net::UnixListener::bind(&path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o660));
            }
            info!("✅ tarpc UDS server listening on {}", path.display());

            loop {
                let (stream, _peer) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("tarpc UDS: Failed to accept connection: {}", e);
                        continue;
                    }
                };

                debug!("New tarpc UDS connection");

                let server = server.clone();

                tokio::spawn(async move {
                    let transport = tarpc::serde_transport::new(
                        tarpc::tokio_util::codec::LengthDelimitedCodec::builder()
                            .max_frame_length(16 * 1024 * 1024)
                            .new_framed(stream),
                        tarpc::tokio_serde::formats::Bincode::default(),
                    );

                    let channel = tarpc::server::BaseChannel::with_defaults(transport);

                    channel
                        .execute(server.serve())
                        .for_each(|response| async move {
                            tokio::spawn(response);
                        })
                        .await;

                    debug!("tarpc UDS connection closed");
                });
            }
        }
    };
}
