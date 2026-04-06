// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC configuration with `security provider` crypto delegation.
//!
//! All cryptographic operations are delegated to `security provider` via JSON-RPC IPC.
//! No quinn, rustls, or ring dependencies.

use crate::crypto::SecurityQuicCrypto;
use crate::tls::transport_params::TransportParams;
use songbird_crypto_provider::socket_discovery;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// QUIC configuration.
///
/// Transport configuration; all crypto is routed via the Neural API / `security provider` socket.
#[derive(Debug, Clone)]
pub struct QuicConfig {
    /// Neural API Unix socket path (same discovery as `songbird_crypto_provider`).
    pub neural_api_socket: PathBuf,

    /// Connection idle timeout.
    pub idle_timeout: Duration,

    /// Keep-alive interval.
    pub keep_alive_interval: Option<Duration>,

    /// Maximum concurrent bidirectional streams.
    pub max_concurrent_bidi_streams: u64,

    /// Maximum concurrent unidirectional streams.
    pub max_concurrent_uni_streams: u64,

    /// Enable 0-RTT (faster reconnection).
    pub enable_0rtt: bool,

    /// Connection migration enabled (mobile roaming).
    pub enable_migration: bool,

    /// Maximum MTU.
    pub max_mtu: u16,

    /// TLS certificate domain (for self-signed inter-primal certs).
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
    /// Create new configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override Neural API socket path.
    #[must_use]
    pub fn with_neural_api_socket(mut self, socket: PathBuf) -> Self {
        self.neural_api_socket = socket;
        self
    }

    /// Set idle timeout.
    #[must_use]
    pub const fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Enable 0-RTT connections.
    #[must_use]
    pub const fn with_0rtt(mut self, enabled: bool) -> Self {
        self.enable_0rtt = enabled;
        self
    }

    /// Enable connection migration.
    #[must_use]
    pub const fn with_migration(mut self, enabled: bool) -> Self {
        self.enable_migration = enabled;
        self
    }

    /// Build transport parameters from this config.
    #[must_use]
    pub fn transport_params(&self) -> TransportParams {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "idle timeout in ms fits u64 for transport params"
        )]
        let max_idle_timeout = self.idle_timeout.as_millis() as u64;
        TransportParams {
            max_idle_timeout,
            max_udp_payload_size: u64::from(self.max_mtu),
            initial_max_streams_bidi: self.max_concurrent_bidi_streams,
            initial_max_streams_uni: self.max_concurrent_uni_streams,
            disable_active_migration: !self.enable_migration,
            ..TransportParams::songbird_defaults()
        }
    }

    /// Create a `security provider` crypto provider from this config's socket path.
    #[must_use]
    pub fn crypto_provider(&self) -> Arc<SecurityQuicCrypto> {
        Arc::new(SecurityQuicCrypto::discover())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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
    fn transport_params_from_config() {
        let config = QuicConfig::new();
        let tp = config.transport_params();
        assert_eq!(tp.initial_max_streams_bidi, 100);
        assert_eq!(tp.initial_max_streams_uni, 100);
        assert_eq!(tp.max_idle_timeout, 30_000);
    }

    #[test]
    fn migration_disabled_in_transport_params() {
        let config = QuicConfig::new().with_migration(false);
        let tp = config.transport_params();
        assert!(tp.disable_active_migration);
    }

    #[test]
    fn transport_params_max_udp_payload_matches_max_mtu() {
        let mtu: u16 = 1452;
        let config = QuicConfig::new();
        let config = QuicConfig {
            max_mtu: mtu,
            ..config
        };
        let tp = config.transport_params();
        assert_eq!(
            tp.max_udp_payload_size,
            u64::from(mtu),
            "transport params should mirror config max_mtu for UDP payload size"
        );
    }

    #[test]
    fn transport_params_zero_idle_timeout() {
        let config = QuicConfig::new().with_idle_timeout(Duration::ZERO);
        let tp = config.transport_params();
        assert_eq!(tp.max_idle_timeout, 0, "zero idle timeout should map to 0 ms");
    }

    #[test]
    fn stream_limits_reflected_in_transport_params() {
        let config = QuicConfig {
            max_concurrent_bidi_streams: 7,
            max_concurrent_uni_streams: 3,
            ..QuicConfig::new()
        };
        let tp = config.transport_params();
        assert_eq!(tp.initial_max_streams_bidi, 7);
        assert_eq!(tp.initial_max_streams_uni, 3);
    }

    #[test]
    fn migration_enabled_sets_disable_active_migration_false() {
        let tp = QuicConfig::new().with_migration(true).transport_params();
        assert!(
            !tp.disable_active_migration,
            "when migration is on, disable_active_migration should be false"
        );
    }
}
