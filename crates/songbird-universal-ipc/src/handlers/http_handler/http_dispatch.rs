// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Enum dispatch for HTTP client factory, crypto discovery, and HTTP client implementations.

use crate::error::IpcResult;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, instrument};

use super::client::SongbirdHttpClient;
use super::env_discovery::EnvCryptoDiscovery;
#[cfg(test)]
use super::test_support::{QueuedMockClient, RotatingMockClient};
use super::types::HttpResponse;

/// Runtime discovery for crypto / signing endpoints (enum dispatch).
#[derive(Clone, Copy, Debug)]
pub enum CryptoDiscovery {
    Env(EnvCryptoDiscovery),
}

impl CryptoDiscovery {
    pub async fn discover(&self, capability: &str) -> IpcResult<String> {
        match self {
            Self::Env(e) => e.discover(capability).await,
        }
    }
}

/// Default factory using runtime capability discovery
pub struct DefaultHttpClientFactory {
    discovery: CryptoDiscovery,
}

impl DefaultHttpClientFactory {
    #[must_use]
    pub fn new(discovery: CryptoDiscovery) -> Self {
        Self {
            discovery,
        }
    }

    #[instrument(skip(self))]
    pub async fn create_client(&self) -> IpcResult<Arc<HttpClient>> {
        let security_socket = self.discovery.discover("crypto.signing").await?;

        info!("Discovered crypto provider at: {}", security_socket);

        let client = SongbirdHttpClient::new(&security_socket);
        Ok(Arc::new(HttpClient::Songbird(client)))
    }
}

/// HTTP client implementations used by the IPC HTTP handler.
pub enum HttpClient {
    Songbird(SongbirdHttpClient),
    #[cfg(test)]
    Rotating(Arc<RotatingMockClient>),
    #[cfg(test)]
    Queued(Arc<QueuedMockClient>),
    #[cfg(test)]
    AlwaysFailRequest,
}

impl HttpClient {
    /// Execute HTTP request
    #[instrument(skip(self, body), fields(method = %method, url = %url))]
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        match self {
            Self::Songbird(c) => c.request(method, url, headers, body).await,
            #[cfg(test)]
            Self::Rotating(c) => c.request(method, url, headers, body).await,
            #[cfg(test)]
            Self::Queued(c) => c.request(method, url, headers, body).await,
            #[cfg(test)]
            Self::AlwaysFailRequest => {
                Err(crate::error::IpcError::ConnectionFailed("injected request failure".into()))
            }
        }
    }
}

/// Factory for creating [`HttpClient`] instances (enum dispatch).
pub enum HttpClientFactory {
    Default(DefaultHttpClientFactory),
    #[cfg(test)]
    InjectTest {
        client: Arc<HttpClient>,
    },
    #[cfg(test)]
    FailingCreate,
}

impl HttpClientFactory {
    /// Create HTTP client with automatic crypto provider discovery (for default arm).
    pub async fn create_client(&self) -> IpcResult<Arc<HttpClient>> {
        match self {
            Self::Default(f) => f.create_client().await,
            #[cfg(test)]
            Self::InjectTest {
                client,
            } => Ok(Arc::clone(client)),
            #[cfg(test)]
            Self::FailingCreate => {
                Err(crate::error::IpcError::ConnectionFailed("mock factory".into()))
            }
        }
    }

    /// Production helper: environment-based crypto discovery.
    #[must_use]
    pub fn with_default_crypto_discovery() -> Self {
        Self::Default(DefaultHttpClientFactory::new(CryptoDiscovery::Env(EnvCryptoDiscovery)))
    }
}
