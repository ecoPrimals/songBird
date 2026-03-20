// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP-based Rendezvous Client
//!
//! Production implementation of `RendezvousClient` using HTTP for relay server communication.
//!
//! ## Deep Debt Compliance
//! - Zero hardcoding: Configurable server URLs
//! - Mocks isolated: Real implementation for production
//! - Pure Rust: Uses tokio TCP for HTTP requests (no C deps)
//! - Modern async: Full async/await
//! - Event-driven: No polling, no sleeps

use super::rendezvous_handler::{RendezvousClient, RendezvousPeer, RendezvousRegisterResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// HTTP-based rendezvous client for production use
///
/// Uses pure Rust TCP+HTTP for relay server communication.
/// Supports both registration (announcing presence) and lookup
/// (finding peers) via a rendezvous server.
pub struct HttpRendezvousClient {
    /// HTTP request timeout
    timeout: Duration,
    /// Retry attempts for transient failures
    max_retries: u32,
}

impl Default for HttpRendezvousClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Registration request body
#[derive(Debug, Serialize)]
struct RegisterRequest<'a> {
    node_id: &'a str,
    family_id: &'a str,
    public_address: &'a str,
    capabilities: Vec<&'a str>,
    timestamp: u64,
}

/// Registration response from server
#[derive(Debug, Deserialize)]
struct RegisterResponse {
    #[serde(default)]
    registration_id: String,
    #[serde(default)]
    expires_at: String,
    #[serde(default)]
    rendezvous_token: String,
}

/// Lookup response from server
#[derive(Debug, Deserialize)]
struct LookupResponse {
    #[serde(default)]
    peers: Vec<LookupPeer>,
}

/// Peer entry from lookup
#[derive(Debug, Deserialize)]
struct LookupPeer {
    node_id: String,
    #[serde(default)]
    family_id: String,
    #[serde(default)]
    public_address: String,
    #[serde(default)]
    rendezvous_token: String,
}

impl HttpRendezvousClient {
    pub fn new() -> Self {
        info!("✅ HTTP Rendezvous Client initialized (production)");
        Self {
            timeout: Duration::from_secs(10),
            max_retries: 2,
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub const fn with_config(timeout: Duration, max_retries: u32) -> Self {
        Self {
            timeout,
            max_retries,
        }
    }

    /// Make an HTTP POST request (pure Rust, minimal implementation)
    async fn http_post(&self, url: &str, body: &str) -> Result<String, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let (host, port, path) = Self::parse_url(url)?;

        let addr = format!("{host}:{port}");
        let stream = tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("Connection timeout to {addr}"))?
            .map_err(|e| format!("Connection failed to {addr}: {e}"))?;

        let request = format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            path,
            host,
            body.len(),
            body
        );

        let (mut reader, mut writer) = stream.into_split();
        writer.write_all(request.as_bytes()).await.map_err(|e| format!("Write failed: {e}"))?;
        writer.shutdown().await.map_err(|e| format!("Shutdown failed: {e}"))?;

        let mut response = Vec::new();
        reader.read_to_end(&mut response).await.map_err(|e| format!("Read failed: {e}"))?;

        let response_str = String::from_utf8_lossy(&response);

        // Extract body after \r\n\r\n
        response_str.find("\r\n\r\n").map_or_else(
            || Err("Invalid HTTP response: no body separator".to_string()),
            |body_start| Ok(response_str[body_start + 4..].to_string()),
        )
    }

    /// Make an HTTP GET request (pure Rust)
    async fn http_get(&self, url: &str) -> Result<String, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let (host, port, path) = Self::parse_url(url)?;

        let addr = format!("{host}:{port}");
        let stream = tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| format!("Connection timeout to {addr}"))?
            .map_err(|e| format!("Connection failed to {addr}: {e}"))?;

        let request = format!(
            "GET {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Accept: application/json\r\n\
             Connection: close\r\n\
             \r\n"
        );

        let (mut reader, mut writer) = stream.into_split();
        writer.write_all(request.as_bytes()).await.map_err(|e| format!("Write failed: {e}"))?;
        writer.shutdown().await.map_err(|e| format!("Shutdown failed: {e}"))?;

        let mut response = Vec::new();
        reader.read_to_end(&mut response).await.map_err(|e| format!("Read failed: {e}"))?;

        let response_str = String::from_utf8_lossy(&response);

        response_str.find("\r\n\r\n").map_or_else(
            || Err("Invalid HTTP response: no body separator".to_string()),
            |body_start| Ok(response_str[body_start + 4..].to_string()),
        )
    }

    /// Parse URL into (host, port, path)
    fn parse_url(url: &str) -> Result<(String, u16, String), String> {
        let url = url.trim();

        // Strip scheme
        let (scheme_port, rest) = if let Some(rest) = url.strip_prefix("https://") {
            (443u16, rest)
        } else if let Some(rest) = url.strip_prefix("http://") {
            (80u16, rest)
        } else {
            return Err(format!("Unsupported URL scheme: {url}"));
        };

        // Split host:port and path
        let (host_port, path) = rest
            .find('/')
            .map_or((rest, "/"), |slash_pos| (&rest[..slash_pos], &rest[slash_pos..]));

        // Parse host and optional port
        let (host, port) = host_port.rfind(':').map_or_else(
            || (host_port.to_string(), scheme_port),
            |colon_pos| {
                let port_str = &host_port[colon_pos + 1..];
                port_str.parse::<u16>().map_or_else(
                    |_| (host_port.to_string(), scheme_port),
                    |port| (host_port[..colon_pos].to_string(), port),
                )
            },
        );

        Ok((host, port, path.to_string()))
    }
}

#[async_trait]
impl RendezvousClient for HttpRendezvousClient {
    async fn register(
        &self,
        server: &str,
        node_id: &str,
        family_id: &str,
        public_address: &str,
    ) -> Result<RendezvousRegisterResult, String> {
        info!(
            "🌐 HTTP Rendezvous: Registering with {} (node: {}, family: {})",
            server, node_id, family_id
        );

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let request = RegisterRequest {
            node_id,
            family_id,
            public_address,
            capabilities: vec!["relay", "mesh", "stun"],
            timestamp,
        };

        let body =
            serde_json::to_string(&request).map_err(|e| format!("Serialization error: {e}"))?;

        let url = format!("{}/rendezvous/register", server.trim_end_matches('/'));

        // Retry loop
        let mut last_error = String::new();
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!("Rendezvous register retry {}/{}", attempt, self.max_retries);
            }

            match self.http_post(&url, &body).await {
                Ok(response_body) => {
                    match serde_json::from_str::<RegisterResponse>(&response_body) {
                        Ok(response) => {
                            info!(
                                "✅ Registered with rendezvous (token: {})",
                                &response.rendezvous_token
                            );
                            return Ok(RendezvousRegisterResult {
                                registration_id: response.registration_id,
                                expires_at: response.expires_at,
                                rendezvous_token: response.rendezvous_token,
                            });
                        }
                        Err(e) => {
                            warn!("Invalid rendezvous response: {}", e);
                            last_error = format!("Invalid response: {e}");
                        }
                    }
                }
                Err(e) => {
                    warn!("Rendezvous register failed: {}", e);
                    last_error = e;
                }
            }
        }

        Err(format!(
            "Rendezvous registration failed after {} attempts: {}",
            self.max_retries + 1,
            last_error
        ))
    }

    async fn lookup(&self, server: &str, target: &str) -> Result<Vec<RendezvousPeer>, String> {
        info!("🌐 HTTP Rendezvous: Looking up {} on {}", target, server);

        let url = format!("{}/rendezvous/lookup?target={}", server.trim_end_matches('/'), target);

        match self.http_get(&url).await {
            Ok(response_body) => {
                match serde_json::from_str::<LookupResponse>(&response_body) {
                    Ok(response) => {
                        let peers: Vec<RendezvousPeer> = response
                            .peers
                            .into_iter()
                            .map(|p| RendezvousPeer {
                                node_id: p.node_id,
                                family_id: p.family_id,
                                public_address: p.public_address,
                                rendezvous_token: p.rendezvous_token,
                            })
                            .collect();

                        info!("🌐 Rendezvous lookup found {} peers", peers.len());
                        Ok(peers)
                    }
                    Err(e) => {
                        // Return empty (not error) — graceful degradation
                        warn!("Invalid rendezvous lookup response: {}", e);
                        Ok(Vec::new())
                    }
                }
            }
            Err(e) => {
                // Return empty (not error) — system works without rendezvous
                warn!("Rendezvous lookup failed: {}. Using local discovery.", e);
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_rendezvous_client_creation() {
        let _client = HttpRendezvousClient::new();
    }

    #[tokio::test]
    async fn test_custom_config() {
        let client = HttpRendezvousClient::with_config(Duration::from_secs(5), 3);
        assert_eq!(client.timeout, Duration::from_secs(5));
        assert_eq!(client.max_retries, 3);
    }

    #[tokio::test]
    async fn test_register_unreachable_server() {
        let client = HttpRendezvousClient::new();
        let result = client
            .register("http://127.0.0.1:19999", "node-test", "nat0", "203.0.113.1:5000")
            .await;
        // Should fail gracefully (no panic)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lookup_unreachable_returns_empty() {
        let client = HttpRendezvousClient::new();
        let result = client.lookup("http://127.0.0.1:19999", "target-node").await;
        // Should return empty (graceful degradation), not error
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_url_http() {
        let (host, port, path) =
            HttpRendezvousClient::parse_url("http://relay.example.com:8080/api").unwrap();
        assert_eq!(host, "relay.example.com");
        assert_eq!(port, 8080);
        assert_eq!(path, "/api");
    }

    #[test]
    fn test_parse_url_default_port() {
        let (host, port, path) = HttpRendezvousClient::parse_url("http://relay.local/v1").unwrap();
        assert_eq!(host, "relay.local");
        assert_eq!(port, 80);
        assert_eq!(path, "/v1");
    }

    #[test]
    fn test_parse_url_https() {
        let (host, port, _) = HttpRendezvousClient::parse_url("https://secure.relay.io").unwrap();
        assert_eq!(host, "secure.relay.io");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_url_invalid_scheme() {
        assert!(HttpRendezvousClient::parse_url("ftp://bad.scheme").is_err());
    }
}
