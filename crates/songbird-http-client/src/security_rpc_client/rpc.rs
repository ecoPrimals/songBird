// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider RPC communication
//!
//! JSON-RPC call implementation with dual-mode support (Direct and Neural API).

use super::core::{SecurityRpcClient, SecurityRpcMode};
use super::types::{JsonRpcRequest, JsonRpcResponse};
use crate::crypto::socket_discovery::IpcEndpoint;
use crate::error::{Error, Result};
use serde_json::{Value, json};
use songbird_types::defaults::timeouts::{
    DEFAULT_NEURAL_API_TIMEOUT, DEFAULT_SECURITY_RPC_TIMEOUT,
};
use std::sync::atomic::Ordering;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tracing::{debug, error, trace};

const JSONRPC_VERSION: &str = "2.0";

/// Concrete async stream for security RPC (TCP or Unix socket; enum dispatch, no trait objects).
enum AsyncStreamImpl {
    Tcp(tokio::net::TcpStream),
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
}

impl AsyncRead for AsyncStreamImpl {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self.as_mut() {
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for AsyncStreamImpl {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut *self.as_mut() {
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self.as_mut() {
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self.as_mut() {
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            #[cfg(unix)]
            Self::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl SecurityRpcClient {
    /// Map semantic capability names to actual provider method names
    ///
    /// **DEPRECATED**: This mapping is only used in Direct mode for backward compatibility.
    /// In production (Neural API mode), semantic translation is handled by Neural API's
    /// capability registry, allowing the `security provider` to evolve its API independently.
    ///
    /// (Used only in Direct mode)
    #[deprecated(
        since = "0.2.0",
        note = "Use Neural API's capability.call for semantic routing in production. Direct mode is for testing only."
    )]
    fn semantic_to_actual(capability: &str) -> Result<&'static str> {
        Ok(match capability {
            // Crypto operations - map to the provider's actual method names
            "crypto.generate_keypair" => "crypto.x25519_generate_ephemeral",
            "crypto.ecdh_derive" => "crypto.x25519_derive_secret",
            "crypto.encrypt" => "crypto.chacha20_poly1305_encrypt",
            "crypto.decrypt" => "crypto.chacha20_poly1305_decrypt",
            "crypto.encrypt_aes_128_gcm" => "crypto.aes128_gcm_encrypt",
            "crypto.decrypt_aes_128_gcm" => "crypto.aes128_gcm_decrypt",
            "crypto.encrypt_aes_256_gcm" => "crypto.aes256_gcm_encrypt",
            "crypto.decrypt_aes_256_gcm" => "crypto.aes256_gcm_decrypt",
            "crypto.sha256" => "crypto.sha256",
            "crypto.sha384" => "crypto.sha384",
            "crypto.hkdf_extract" => "crypto.hkdf_extract",
            "crypto.hkdf_expand" => "crypto.hkdf_expand",

            // TLS key derivation
            "tls.derive_handshake_secrets" => "tls.derive_handshake_secrets",
            "tls.derive_application_secrets" => "tls.derive_application_secrets",
            "tls.compute_finished_verify_data" => "tls.compute_finished_verify_data",

            // BTSP session management (identity mapping — BearDog uses these names directly)
            "btsp.session.create" => "btsp.session.create",
            "btsp.session.verify" => "btsp.session.verify",
            "btsp.session.negotiate" => "btsp.session.negotiate",
            "btsp.server.export_keys" => "btsp.server.export_keys",

            _ => {
                return Err(Error::SecurityProviderRpc(format!(
                    "Unknown capability: {capability}. Add mapping to semantic_to_actual()"
                )));
            }
        })
    }

    /// Connect to IPC endpoint (Unix socket or TCP)
    ///
    /// Isomorphic connection helper that works with both Unix sockets and TCP.
    async fn connect_endpoint(endpoint: &IpcEndpoint) -> std::io::Result<AsyncStreamImpl> {
        match endpoint {
            IpcEndpoint::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    use tokio::net::UnixStream;
                    let stream = UnixStream::connect(path).await?;
                    Ok(AsyncStreamImpl::Unix(stream))
                }
                #[cfg(not(unix))]
                {
                    let _ = path;
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Unix sockets not supported on this platform",
                    ))
                }
            }
            IpcEndpoint::TcpLocal(addr) => {
                use tokio::net::TcpStream;
                let stream = TcpStream::connect(addr).await?;
                Ok(AsyncStreamImpl::Tcp(stream))
            }
        }
    }

    /// Make an RPC call to the `security provider`
    ///
    /// In Direct mode: Calls the provider directly using actual method names
    /// In Neural API mode: Routes through Neural API for capability translation
    pub(super) async fn call(&self, capability: &str, args: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        match &self.mode {
            SecurityRpcMode::Direct {
                endpoint,
            } => self.call_direct(endpoint, capability, args, id).await,
            SecurityRpcMode::NeuralApi {
                endpoint,
            } => self.call_neural_api(endpoint, capability, args, id).await,
        }
    }

    /// Direct RPC to the `security provider` (testing, simple deployments)
    async fn call_direct(
        &self,
        endpoint: &IpcEndpoint,
        capability: &str,
        args: Value,
        id: u64,
    ) -> Result<Value> {
        // Note: Direct mode is deprecated for production use
        #[expect(deprecated, reason = "migration to successor API planned")]
        let method = Self::semantic_to_actual(capability)?;

        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.into(),
            method: method.to_string(),
            params: args,
            id,
        };

        trace!("→ Security provider direct RPC: {} (id={}) via {:?}", method, id, endpoint);

        // Connect to security provider (isomorphic: Unix or TCP)
        let mut stream = Self::connect_endpoint(endpoint).await.map_err(|e| {
            Error::SecurityProviderRpc(format!(
                "Failed to connect to security provider at {endpoint:?}: {e}"
            ))
        })?;

        // Send request (newline-terminated; security provider reads line-by-line)
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // JSON-aware chunked read — security provider may keep the socket open (no EOF).
        let buffer = crate::io_util::read_json_response(&mut stream, DEFAULT_SECURITY_RPC_TIMEOUT)
            .await
            .map_err(|e| Error::SecurityProviderRpc(format!("Security provider: {e}")))?;

        let response: JsonRpcResponse = serde_json::from_slice(&buffer)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid JSON response: {e}")))?;

        if let Some(error) = response.error {
            return Err(Error::SecurityProviderRpc(format!(
                "Security provider error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| Error::SecurityProviderRpc(String::from("No result in response")))
    }

    /// TRUE PRIMAL: Route through Neural API for semantic capability resolution
    async fn call_neural_api(
        &self,
        endpoint: &IpcEndpoint,
        capability: &str,
        args: Value,
        id: u64,
    ) -> Result<Value> {
        // Split semantic name into capability + operation
        // e.g. `crypto.generate_keypair` → capability:"crypto", operation:"generate_keypair"
        let parts: Vec<&str> = capability.split('.').collect();
        let (cap, op) = if parts.len() >= 2 {
            (parts[0], parts[1..].join("."))
        } else {
            ("crypto", capability.to_string())
        };

        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.into(),
            method: String::from("capability.call"),
            params: json!({
                "capability": cap,
                "operation": op,
                "args": args
            }),
            id,
        };

        trace!("→ Neural API capability.call: {}.{} (id={}) via {:?}", cap, op, id, endpoint);

        // Connect to Neural API (isomorphic: Unix or TCP)
        let mut stream = Self::connect_endpoint(endpoint).await.map_err(|e| {
            Error::SecurityProviderRpc(format!(
                "Failed to connect to Neural API at {endpoint:?}: {e}"
            ))
        })?;

        // Send request (newline-terminated)
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // JSON-aware chunked read — Neural API keeps socket open (no EOF).
        let buffer = crate::io_util::read_json_response(&mut stream, DEFAULT_NEURAL_API_TIMEOUT)
            .await
            .map_err(|e| Error::SecurityProviderRpc(format!("Neural API: {e}")))?;

        // Log raw response for debugging
        if let Ok(response_str) = std::str::from_utf8(&buffer) {
            trace!(
                "← Raw Neural API response ({} bytes): {}",
                buffer.len(),
                if response_str.len() > 500 {
                    format!("{}... (truncated)", &response_str[..500])
                } else {
                    response_str.to_string()
                }
            );
        }

        let response: JsonRpcResponse = serde_json::from_slice(&buffer).map_err(|e| {
            error!("❌ Failed to parse Neural API response for {}: {}", capability, e);
            if let Ok(response_str) = std::str::from_utf8(&buffer) {
                error!("   Raw response: {}", response_str);
            }
            Error::SecurityProviderRpc(format!("Failed to parse Neural API response: {e}"))
        })?;

        let id_str = response.id.map_or_else(|| String::from("null"), |id| id.to_string());
        trace!("← Neural API result for {} (id={})", capability, id_str);

        // Check for errors
        if let Some(error) = response.error {
            error!(
                "❌ Neural API error for {}: {} (code: {})",
                capability, error.message, error.code
            );
            return Err(Error::SecurityProviderRpc(format!(
                "Neural API error for {}: {} (code: {})",
                capability, error.message, error.code
            )));
        }

        debug!("✅ Neural API call successful: {}", capability);
        response.result.ok_or_else(|| {
            error!("❌ Missing result in Neural API response for {}", capability);
            Error::SecurityProviderRpc(String::from("Missing result in response"))
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_mapping() {
        #[allow(deprecated, reason = "test assertions and harness ergonomics")]
        {
            assert_eq!(
                SecurityRpcClient::semantic_to_actual("crypto.generate_keypair").unwrap(),
                "crypto.x25519_generate_ephemeral"
            );
            assert_eq!(
                SecurityRpcClient::semantic_to_actual("crypto.ecdh_derive").unwrap(),
                "crypto.x25519_derive_secret"
            );
        }
    }

    #[test]
    fn test_semantic_mapping_unknown() {
        #[allow(deprecated, reason = "calling deprecated API until migration completes")]
        let result = SecurityRpcClient::semantic_to_actual("unknown.method");
        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_mapping_all_crypto_and_tls_aliases() {
        #[allow(deprecated, reason = "calling deprecated API until migration completes")]
        {
            let pairs = [
                ("crypto.encrypt", "crypto.chacha20_poly1305_encrypt"),
                ("crypto.decrypt", "crypto.chacha20_poly1305_decrypt"),
                ("crypto.encrypt_aes_128_gcm", "crypto.aes128_gcm_encrypt"),
                ("crypto.decrypt_aes_128_gcm", "crypto.aes128_gcm_decrypt"),
                ("crypto.encrypt_aes_256_gcm", "crypto.aes256_gcm_encrypt"),
                ("crypto.decrypt_aes_256_gcm", "crypto.aes256_gcm_decrypt"),
                ("crypto.sha256", "crypto.sha256"),
                ("crypto.sha384", "crypto.sha384"),
                ("crypto.hkdf_extract", "crypto.hkdf_extract"),
                ("crypto.hkdf_expand", "crypto.hkdf_expand"),
                ("tls.derive_handshake_secrets", "tls.derive_handshake_secrets"),
                ("tls.derive_application_secrets", "tls.derive_application_secrets"),
                ("tls.compute_finished_verify_data", "tls.compute_finished_verify_data"),
            ];
            for (cap, expected) in pairs {
                assert_eq!(SecurityRpcClient::semantic_to_actual(cap).unwrap(), expected);
            }
        }
    }

    #[test]
    fn test_semantic_mapping_unknown_returns_security_rpc_error() {
        #[allow(deprecated, reason = "calling deprecated API until migration completes")]
        let err = SecurityRpcClient::semantic_to_actual("not.mapped.here").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unknown capability"));
    }
}
