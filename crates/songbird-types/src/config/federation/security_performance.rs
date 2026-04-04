// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation transport tuning: TLS, performance knobs, and buffer sizes.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// **CANONICAL**: Federation security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationSecurityConfig {
    /// Enable TLS encryption
    pub enable_tls: bool,
    /// Certificate path
    pub cert_path: Option<String>,
    /// Private key path
    pub key_path: Option<String>,
    /// Trusted certificate authorities
    pub trusted_cas: Vec<String>,
    /// Enable mutual authentication
    pub mutual_auth: bool,
}

impl Default for CanonicalFederationSecurityConfig {
    fn default() -> Self {
        Self {
            enable_tls: true,
            cert_path: None,
            key_path: None,
            trusted_cas: vec![],
            mutual_auth: false,
        }
    }
}

/// **CANONICAL**: Federation performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationPerformanceConfig {
    /// Maximum concurrent connections
    /// Max Connections field
    pub max_connections: usize,
    /// Connection timeout
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive interval
    /// Keep Alive Interval field
    pub keep_alive_interval: Duration,
    /// Buffer sizes
    pub buffer_sizes: CanonicalBufferSizes,
}

impl Default for CanonicalFederationPerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            keep_alive_interval: Duration::from_secs(30),
            buffer_sizes: CanonicalBufferSizes::default(),
        }
    }
}

/// **CANONICAL**: Buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBufferSizes {
    /// Send buffer size in bytes
    pub send_buffer: usize,
    /// Receive buffer size in bytes
    pub recv_buffer: usize,
    /// Message queue size
    pub message_queue: usize,
}

impl Default for CanonicalBufferSizes {
    fn default() -> Self {
        Self {
            send_buffer: 64 * 1024, // 64KB
            recv_buffer: 64 * 1024, // 64KB
            message_queue: 1000,    // 1000 messages
        }
    }
}
