//! # Songbird Network Layer
//!
//! Provides secure network transport for Songbird federation:
//! - WireGuard VPN tunnels
//! - TLS/HTTPS endpoints
//! - Certificate management
//! - Secure peer authentication
//!
//! ## Security Layering
//!
//! Layer 1: LAN (Sovereign security)
//! Layer 2: Internet (WireGuard + TLS) ← This crate
//! Layer 3: Anywhere (BearDog enhanced)

pub mod tls;
pub mod wireguard;
pub mod error;

pub use error::{NetworkError, NetworkResult};
pub use tls::{TlsCertificateManager, TlsConfig, TlsError, create_tls_acceptor};
pub use wireguard::{WireGuardConfig, WireGuardManager, TunnelInfo};

/// Network security mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// LAN-only (sovereign security)
    Lan,
    /// Internet-safe (WireGuard + TLS)
    Internet,
    /// Anywhere-safe (BearDog enhanced)
    Anywhere,
}

impl Default for SecurityMode {
    fn default() -> Self {
        Self::Lan
    }
}

