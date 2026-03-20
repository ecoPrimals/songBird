// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Minimal pure Rust HTTP GET client for Tor directory fetching
//!
//! Replaces `reqwest` to eliminate the `ring` dependency chain.
//! All Tor directory authorities serve plain HTTP — no TLS needed.
//!
//! **Zero ring | Zero reqwest | Zero C dependencies**

use crate::error::{Error, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Duration;
use tokio::time::timeout;
use tracing::debug;

/// Perform a simple HTTP GET request and return the body as a String.
///
/// Only supports plain HTTP (not HTTPS) — which is correct for Tor
/// directory authority fetches. These authorities serve consensus and
/// relay descriptors over plain HTTP on their `DirPort`.
///
/// # Arguments
///
/// * `url` - Full HTTP URL (e.g., "<http://131.188.40.189:80/tor/status-vote/current/consensus>")
/// * `request_timeout` - Maximum time to wait for the response
///
/// # Errors
///
/// Returns `Error::Network` on connection failure or timeout.
pub async fn get(url: &str, request_timeout: Duration) -> Result<String> {
    debug!("HTTP GET: {}", url);

    let uri: hyper::Uri =
        url.parse().map_err(|e| Error::Network(format!("Invalid URL '{url}': {e}")))?;

    let client = Client::builder(TokioExecutor::new()).build_http::<Empty<Bytes>>();

    let request = Request::builder()
        .method("GET")
        .uri(&uri)
        .header("User-Agent", "Songbird/3.33.0")
        .body(Empty::<Bytes>::new())
        .map_err(|e| Error::Network(format!("Failed to build request: {e}")))?;

    let response = timeout(request_timeout, client.request(request))
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(|e| Error::Network(format!("HTTP request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        return Err(Error::Network(format!("HTTP {status} from {url}")));
    }

    let body_bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|e| Error::Network(format!("Failed to read response body: {e}")))?
        .to_bytes();

    let body = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| Error::Network(format!("Response body is not valid UTF-8: {e}")))?;

    debug!("HTTP GET {} complete: {} bytes", url, body.len());
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_url() {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let result = get("not a url", Duration::from_secs(5)).await;
            assert!(result.is_err());
        });
    }
}
