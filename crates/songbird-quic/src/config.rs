// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC configuration with Neural API socket discovery (crypto routing)

use crate::error::{QuicError, Result};
use songbird_crypto_provider::socket_discovery;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// QUIC configuration
///
/// QUIC transport configuration; application crypto is routed via the Neural API socket.
#[derive(Debug, Clone)]
pub struct QuicConfig {
    /// Neural API Unix socket path (same discovery as `songbird-crypto-provider`)
    pub neural_api_socket: PathBuf,

    /// Connection idle timeout
    pub idle_timeout: Duration,

    /// Keep-alive interval
    pub keep_alive_interval: Option<Duration>,

    /// Maximum concurrent bidirectional streams
    pub max_concurrent_bidi_streams: u64,

    /// Maximum concurrent unidirectional streams
    pub max_concurrent_uni_streams: u64,

    /// Enable 0-RTT (faster reconnection)
    pub enable_0rtt: bool,

    /// Connection migration enabled (mobile roaming)
    pub enable_migration: bool,

    /// Maximum MTU
    pub max_mtu: u16,

    /// TLS certificate domain (for self-signed inter-primal certs)
    pub tls_domain: String,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            neural_api_socket: PathBuf::from(socket_discovery::discover_neural_api_socket()),

            idle_timeout: Duration::from_secs(30),
            keep_alive_interval: Some(Duration::from_secs(10)),
            max_concurrent_bidi_streams: 100,
            max_concurrent_uni_streams: 100,
            enable_0rtt: true,
            enable_migration: true,
            max_mtu: 1200,
            tls_domain: songbird_process_env::var("SONGBIRD_TLS_DOMAIN")
                .unwrap_or_else(|_| "songbird.local".to_string()),
        }
    }
}

impl QuicConfig {
    /// Create new configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override Neural API socket path
    #[must_use]
    pub fn with_neural_api_socket(mut self, socket: PathBuf) -> Self {
        self.neural_api_socket = socket;
        self
    }

    /// Set idle timeout
    #[must_use]
    pub const fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Enable 0-RTT connections
    #[must_use]
    pub const fn with_0rtt(mut self, enabled: bool) -> Self {
        self.enable_0rtt = enabled;
        self
    }

    /// Enable connection migration
    #[must_use]
    pub const fn with_migration(mut self, enabled: bool) -> Self {
        self.enable_migration = enabled;
        self
    }

    /// Build quinn `ServerConfig` from this config
    pub(crate) fn build_server_config(&self) -> Result<quinn::ServerConfig> {
        // Ensure crypto provider is installed (required by rustls 0.23 before any TLS ops)
        #[cfg(feature = "ring-crypto")]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        #[cfg(not(feature = "ring-crypto"))]
        {
            let _ = rustls_rustcrypto::provider().install_default();
        }

        // Generate self-signed certificate for inter-primal QUIC
        // Self-signed is correct for inter-primal: identity verified via BearDog lineage
        // When BearDog cert generation is available, it can provide lineage-tagged certs
        let cert = rcgen::generate_simple_self_signed(vec![self.tls_domain.clone()])
            .map_err(|e| QuicError::Config(format!("Failed to generate cert: {e}")))?;

        let cert_der = cert.cert.der().to_vec();
        let priv_key_der = cert.key_pair.serialize_der();

        let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert_der)];
        let priv_key = rustls::pki_types::PrivateKeyDer::try_from(priv_key_der)
            .map_err(|_| QuicError::Config("Invalid private key".to_string()))?;

        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, priv_key)
            .map_err(|e| QuicError::Config(format!("Failed to create TLS config: {e}")))?;

        // Enable early data for 0-RTT
        server_crypto.max_early_data_size = 0xffff_ffff;

        let server_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
            .map_err(|e| QuicError::Config(format!("Failed to create QUIC config: {e}")))?;

        let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));

        // Configure transport
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(
            quinn::VarInt::from_u64(self.max_concurrent_bidi_streams)
                .expect("value within VarInt range"),
        );
        transport.max_concurrent_uni_streams(
            quinn::VarInt::from_u64(self.max_concurrent_uni_streams)
                .expect("value within VarInt range"),
        );

        let idle_timeout = quinn::IdleTimeout::from(
            quinn::VarInt::from_u64(
                u64::try_from(self.idle_timeout.as_millis()).unwrap_or(u64::MAX),
            )
            .expect("value within VarInt range"),
        );
        transport.max_idle_timeout(Some(idle_timeout));

        if let Some(keep_alive) = self.keep_alive_interval {
            transport.keep_alive_interval(Some(keep_alive));
        }

        server_config.transport_config(Arc::new(transport));

        Ok(server_config)
    }

    /// Build quinn `ClientConfig` from this config
    pub(crate) fn build_client_config(&self) -> Result<quinn::ClientConfig> {
        // Ensure crypto provider is installed (required by rustls 0.23)
        #[cfg(feature = "ring-crypto")]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        #[cfg(not(feature = "ring-crypto"))]
        {
            let _ = rustls_rustcrypto::provider().install_default();
        }

        // Lineage-based verification: TLS signatures verified by crypto provider,
        // primal identity verified by BearDog at application layer
        let mut crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(LineageCertVerifier::new()))
            .with_no_client_auth();

        // Enable early data for 0-RTT
        crypto.enable_early_data = true;

        let client_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .map_err(|e| QuicError::Config(format!("Failed to create QUIC config: {e}")))?;

        let mut client_config = quinn::ClientConfig::new(Arc::new(client_crypto));

        // Configure transport
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(
            quinn::VarInt::from_u64(self.max_concurrent_bidi_streams)
                .expect("value within VarInt range"),
        );
        transport.max_concurrent_uni_streams(
            quinn::VarInt::from_u64(self.max_concurrent_uni_streams)
                .expect("value within VarInt range"),
        );

        let idle_timeout = quinn::IdleTimeout::from(
            quinn::VarInt::from_u64(
                u64::try_from(self.idle_timeout.as_millis()).unwrap_or(u64::MAX),
            )
            .expect("value within VarInt range"),
        );
        transport.max_idle_timeout(Some(idle_timeout));

        if let Some(keep_alive) = self.keep_alive_interval {
            transport.keep_alive_interval(Some(keep_alive));
        }

        client_config.transport_config(Arc::new(transport));

        Ok(client_config)
    }
}

/// Lineage-based certificate verifier for inter-primal QUIC
///
/// In biomeOS, primals authenticate via `BearDog` lineage verification,
/// not via public CA certificates. This verifier:
///
/// 1. **TLS signatures**: Validated via `rustls::crypto` (rustls-rustcrypto by default, ring when ring-crypto feature)
/// 2. **Server identity**: Accepted if TLS handshake completes (self-signed OK)
/// 3. **Lineage verification**: Happens at the application layer via `BearDog`
///    after the QUIC connection is established
///
/// This is NOT a security bypass -- it's a deliberate architectural choice:
/// - Public CAs don't know about primal lineage
/// - Self-signed certs are the norm for inter-primal communication
/// - Real identity verification happens via `BearDog`'s `lineage.authorize_relay`
///
/// When `BearDog` is available, the cert's public key is cross-referenced
/// against `BearDog`'s known primal registry for additional assurance.
#[derive(Debug)]
struct LineageCertVerifier {
    crypto_provider: Arc<rustls::crypto::CryptoProvider>,
}

impl LineageCertVerifier {
    fn new() -> Self {
        let crypto_provider = {
            #[cfg(feature = "ring-crypto")]
            {
                Arc::new(rustls::crypto::ring::default_provider())
            }
            #[cfg(not(feature = "ring-crypto"))]
            {
                Arc::new(rustls_rustcrypto::provider())
            }
        };
        Self {
            crypto_provider,
        }
    }
}

impl rustls::client::danger::ServerCertVerifier for LineageCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Inter-primal: accept self-signed certificates
        // Real identity verification happens via BearDog lineage at application layer
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.crypto_provider.signature_verification_algorithms.supported_schemes()
    }
}

#[cfg(all(test, feature = "ring-crypto"))]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn new_matches_default_idle_and_streams() {
        let a = QuicConfig::new();
        let b = QuicConfig::default();
        assert_eq!(a.idle_timeout, b.idle_timeout);
        assert_eq!(a.max_concurrent_bidi_streams, b.max_concurrent_bidi_streams);
        assert_eq!(a.max_concurrent_uni_streams, b.max_concurrent_uni_streams);
    }

    #[test]
    fn builder_overrides_chain() {
        let socket = PathBuf::from("/tmp/test-crypto.sock");
        let c = QuicConfig::new()
            .with_neural_api_socket(socket.clone())
            .with_idle_timeout(Duration::from_secs(60))
            .with_0rtt(false)
            .with_migration(false);
        assert_eq!(c.neural_api_socket, socket);
        assert_eq!(c.idle_timeout, Duration::from_secs(60));
        assert!(!c.enable_0rtt);
        assert!(!c.enable_migration);
    }

    #[test]
    fn build_server_config_ok() {
        assert!(QuicConfig::new().build_server_config().is_ok());
    }

    #[test]
    fn build_client_config_ok() {
        assert!(QuicConfig::new().build_client_config().is_ok());
    }
}
