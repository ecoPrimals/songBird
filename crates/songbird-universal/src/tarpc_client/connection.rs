// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Lazy TCP connection and bincode-framed tarpc transport setup.

use tracing::{debug, info};

use crate::tarpc_types::SongbirdRpcClient;
use songbird_types::{SongbirdError, SongbirdResult};

use super::TarpcClient;

impl TarpcClient {
    pub(super) async fn get_connection(&self) -> SongbirdResult<SongbirdRpcClient> {
        {
            let conn = self.connection.read().await;
            if let Some(ref client) = *conn {
                return Ok(client.clone());
            }
        }

        let mut conn = self.connection.write().await;

        if let Some(ref client) = *conn {
            return Ok(client.clone());
        }

        info!("🔌 Establishing tarpc connection to {}", self.addr);
        let client = self.connect().await?;
        *conn = Some(client.clone());

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

        debug!("✅ TCP connection established to {}", self.addr);

        let transport = tarpc::serde_transport::new(
            tokio_util::codec::LengthDelimitedCodec::builder()
                .max_frame_length(16 * 1024 * 1024)
                .new_framed(stream),
            tokio_serde::formats::Bincode::default(),
        );

        let client = SongbirdRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        info!("🚀 tarpc client ready for {}", self.endpoint);

        Ok(client)
    }
}
