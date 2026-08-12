// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Lazy TCP and UDS connection with bincode-framed tarpc transport.

use std::sync::Arc;

use tracing::{debug, info};

use crate::tarpc_types::SongbirdRpcClient;
use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

impl TarpcClient {
    pub(super) async fn get_connection(&self) -> SongbirdResult<Arc<SongbirdRpcClient>> {
        {
            let conn = self.connection.read().await;
            if let Some(ref client) = *conn {
                return Ok(Arc::clone(client));
            }
        }

        let mut conn = self.connection.write().await;

        if let Some(ref client) = *conn {
            return Ok(Arc::clone(client));
        }

        info!("Establishing tarpc connection to {}", self.addr);
        let client = Arc::new(self.connect().await?);
        *conn = Some(Arc::clone(&client));

        Ok(client)
    }

    async fn connect(&self) -> SongbirdResult<SongbirdRpcClient> {
        debug!("Connecting to tarpc server at {}", self.addr);

        let stream = tokio::time::timeout(self.timeout, tokio::net::TcpStream::connect(self.addr))
            .await
            .map_err(|_| SongbirdError::network(format!("Connection timeout to {}", self.addr)))?
            .map_err(|e| {
                SongbirdError::network(format!("Failed to connect to {}: {}", self.addr, e))
            })?;

        debug!("TCP connection established to {}", self.addr);

        let transport = tarpc::serde_transport::new(
            tarpc::tokio_util::codec::LengthDelimitedCodec::builder()
                .max_frame_length(16 * 1024 * 1024)
                .new_framed(stream),
            tarpc::tokio_serde::formats::Bincode::default(),
        );

        let client = SongbirdRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        info!("tarpc client ready for {}", self.endpoint);

        Ok(client)
    }

    /// Connect to a tarpc server via Unix domain socket (G64 dual-socket pattern).
    ///
    /// Prefer this over TCP for intra-gate calls — eliminates TCP overhead.
    #[cfg(unix)]
    pub async fn connect_uds(
        socket_path: &std::path::Path,
        timeout: std::time::Duration,
    ) -> SongbirdResult<SongbirdRpcClient> {
        debug!("Connecting to tarpc UDS at {}", socket_path.display());

        let stream = tokio::time::timeout(timeout, tokio::net::UnixStream::connect(socket_path))
            .await
            .map_err(|_| {
                SongbirdError::network(format!(
                    "UDS connection timeout to {}",
                    socket_path.display()
                ))
            })?
            .map_err(|e| {
                SongbirdError::network(format!(
                    "Failed to connect UDS {}: {}",
                    socket_path.display(),
                    e
                ))
            })?;

        let transport = tarpc::serde_transport::new(
            tarpc::tokio_util::codec::LengthDelimitedCodec::builder()
                .max_frame_length(16 * 1024 * 1024)
                .new_framed(stream),
            tarpc::tokio_serde::formats::Bincode::default(),
        );

        let client = SongbirdRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        info!("tarpc UDS client ready for {}", socket_path.display());

        Ok(client)
    }
}
