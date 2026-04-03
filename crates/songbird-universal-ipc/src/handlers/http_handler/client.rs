// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, instrument};

use super::traits::HttpClientCapability;
use super::types::HttpResponse;

/// Production HTTP client using songbird-http-client
pub struct SongbirdHttpClient {
    inner: Arc<songbird_http_client::SongbirdHttpClient>,
}

impl SongbirdHttpClient {
    /// Create new client with `security provider` crypto provider at socket path
    #[must_use]
    pub fn new(security_socket: &str) -> Self {
        let inner = songbird_http_client::SongbirdHttpClient::new(security_socket);

        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl HttpClientCapability for SongbirdHttpClient {
    #[instrument(skip(self, body), fields(method = %method, url = %url))]
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        debug!("Making HTTP request: {} {} with {} headers", method, url, headers.len());

        // FIX: Parse body once, then call request() with headers (Issue #2 - Jan 28, 2026)
        let body_json = body
            .and_then(|b| std::str::from_utf8(b).ok())
            .and_then(|s| serde_json::from_str(s).ok());

        // Use Pure Rust TLS 1.3 via Tower Atomic pattern
        // FIX: Call request() directly (NOT convenience methods like post()) to preserve headers
        let response =
            self.inner.request(method, url, headers.clone(), body_json).await.map_err(|e| {
                error!("HTTP request failed: {}", e);
                crate::error::IpcError::Internal(format!("HTTP request failed: {e}"))
            })?;

        Ok(HttpResponse {
            status_code: response.status,
            headers: response.headers,
            body: response.body,
        })
    }
}
