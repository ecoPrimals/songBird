//! STUN types

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Public endpoint discovered via STUN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicEndpoint {
    /// Public IP address and port
    pub address: SocketAddr,

    /// NAT type detected
    pub nat_type: NatType,
}

/// NAT type classification
///
/// Determines how aggressive NAT traversal needs to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    /// No NAT (direct internet connection)
    None,

    /// Full cone NAT (easiest to traverse)
    ///
    /// Any external host can send to the mapped port.
    FullCone,

    /// Restricted cone NAT (moderate difficulty)
    ///
    /// Only hosts we've sent to can reply.
    RestrictedCone,

    /// Port-restricted cone NAT (harder to traverse)
    ///
    /// Only specific host:port combinations can reply.
    PortRestrictedCone,

    /// Symmetric NAT (hardest to traverse)
    ///
    /// Different mapping for each destination.
    /// Requires relay for most scenarios.
    Symmetric,

    /// Unknown NAT type
    Unknown,
}

impl Default for NatType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for NatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None (no NAT)"),
            Self::FullCone => write!(f, "Full Cone"),
            Self::RestrictedCone => write!(f, "Restricted Cone"),
            Self::PortRestrictedCone => write!(f, "Port-Restricted Cone"),
            Self::Symmetric => write!(f, "Symmetric"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

