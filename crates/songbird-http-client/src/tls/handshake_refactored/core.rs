//! Core TLS 1.3 handshake struct and constructors

use crate::crypto::CryptoCapability;
use crate::tls::{config::TlsConfig, profiler::ServerProfiler};
use std::sync::Arc;
use tracing::info;

/// TLS 1.3 handshake
pub struct TlsHandshake {
    pub(super) crypto: Arc<dyn CryptoCapability>,
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
    #[allow(dead_code)]
    pub(super) profiler: Option<Arc<ServerProfiler>>,
}

impl TlsHandshake {
    /// Create a new TLS handshake with default config
    pub fn new(crypto: Arc<dyn CryptoCapability>) -> Self {
        Self::with_config(crypto, TlsConfig::default(), None)
    }

    /// Create a new TLS handshake with custom config and optional profiler
    pub fn with_config(
        crypto: Arc<dyn CryptoCapability>,
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
    use crate::crypto::CryptoCapability;

    #[test]
    fn test_handshake_creation() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);

        // Verify initial state
        assert_eq!(handshake.transcript.len(), 0);
        assert_eq!(handshake.cipher_suite, 0);
    }

    #[test]
    fn test_handshake_with_config() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig::default();
        let handshake = TlsHandshake::with_config(beardog, config, None);

        // Verify initial state
        assert_eq!(handshake.transcript.len(), 0);
        assert_eq!(handshake.cipher_suite, 0);
    }
}
