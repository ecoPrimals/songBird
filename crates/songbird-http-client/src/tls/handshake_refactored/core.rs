// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core TLS 1.3 handshake struct and constructors

use crate::crypto::SecurityCryptoProvider;
use crate::tls::{config::TlsConfig, profiler::ServerProfiler};
use std::sync::Arc;
use tracing::info;

/// TLS 1.3 handshake
pub struct TlsHandshake {
    pub(super) crypto: Arc<SecurityCryptoProvider>,
    /// Transcript accumulator for RFC 8446 key derivation
    /// Accumulates all handshake messages for transcript hash computation
    pub(super) transcript: Vec<u8>,
    /// Negotiated TLS 1.3 cipher suite from `ServerHello`
    /// 0x1301 = `TLS_AES_128_GCM_SHA256`
    /// 0x1302 = `TLS_AES_256_GCM_SHA384`
    /// 0x1303 = `TLS_CHACHA20_POLY1305_SHA256`
    pub(super) cipher_suite: u16,
    /// Configuration (strategy-based, not hardcoded)
    pub(super) config: TlsConfig,
    /// Optional server profiler for adaptive learning (future feature)
    #[allow(
        dead_code,
        reason = "reserved for adaptive TLS strategy learning from server behavior"
    )]
    pub(super) profiler: Option<Arc<ServerProfiler>>,
}

impl TlsHandshake {
    /// Create a new TLS handshake with default config
    pub fn new(crypto: Arc<SecurityCryptoProvider>) -> Self {
        Self::with_config(crypto, TlsConfig::default(), None)
    }

    /// Create a new TLS handshake with custom config and optional profiler
    pub fn with_config(
        crypto: Arc<SecurityCryptoProvider>,
        config: TlsConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        info!("🎛️  Creating TLS handshake with {:?} strategy", config.extension_strategy);
        if profiler.is_some() {
            info!("🧠 Adaptive learning enabled (profiler provided)");
        }

        Self {
            crypto,
            transcript: Vec::new(),
            cipher_suite: 0, // Will be set after parsing ServerHello
            config,
            profiler,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_creation() {
        let path = tempfile::env::temp_dir()
            .join("songbird-test-security.sock")
            .to_string_lossy()
            .into_owned();
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(path));
        let handshake = TlsHandshake::new(crypto);

        // Verify initial state
        assert_eq!(handshake.transcript.len(), 0);
        assert_eq!(handshake.cipher_suite, 0);
    }

    #[test]
    fn test_handshake_with_config() {
        let path = tempfile::env::temp_dir()
            .join("songbird-test-security.sock")
            .to_string_lossy()
            .into_owned();
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(path));
        let config = TlsConfig::default();
        let handshake = TlsHandshake::with_config(crypto, config, None);

        // Verify initial state
        assert_eq!(handshake.transcript.len(), 0);
        assert_eq!(handshake.cipher_suite, 0);
    }
}
