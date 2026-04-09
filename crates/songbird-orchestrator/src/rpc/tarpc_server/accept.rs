// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared TCP accept loop for tarpc (bincode + length-delimited frames).
//!
//! Implemented as a macro so each call site uses a concrete `SongbirdRpc` impl; rustc can
//! then prove response futures are `Send` for `tokio::spawn` (async methods in traits do not
//! imply `Send` when the service type is generic).

/// Listen on `addr` and serve a concrete `SongbirdRpc` implementation until the process exits.
macro_rules! run_tarpc_accept_loop {
    ($addr:expr, $server:expr, $startup_log:expr) => {
        async move {
            use futures::StreamExt;
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
                        tokio_util::codec::LengthDelimitedCodec::builder()
                            .max_frame_length(16 * 1024 * 1024)
                            .new_framed(stream),
                        tokio_serde::formats::Bincode::default(),
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
