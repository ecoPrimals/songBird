// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, instrument};

use super::client::SongbirdHttpClient;
use super::traits::{CryptoCapabilityDiscovery, HttpClientCapability, HttpClientFactory};

/// Default factory using runtime capability discovery
pub struct DefaultHttpClientFactory {
    discovery: Arc<dyn CryptoCapabilityDiscovery>,
}

impl DefaultHttpClientFactory {
    pub fn new(discovery: Arc<dyn CryptoCapabilityDiscovery>) -> Self {
        Self {
            discovery,
        }
    }
}

#[async_trait]
impl HttpClientFactory for DefaultHttpClientFactory {
    #[instrument(skip(self))]
    async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>> {
        // Discover crypto provider at runtime (capability-based!)
        let beardog_socket = self.discovery.discover("crypto.signing").await?;

        info!("Discovered crypto provider at: {}", beardog_socket);

        let client = SongbirdHttpClient::new(&beardog_socket);
        Ok(Arc::new(client))
    }
}
