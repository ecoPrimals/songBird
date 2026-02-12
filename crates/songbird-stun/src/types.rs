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

/// Port allocation pattern detected from multiple STUN probes
///
/// Used to predict NAT port assignments for coordinated hole punching.
/// Sequential patterns (step=1 or small step) enable port prediction
/// which dramatically improves symmetric NAT punch success rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortPattern {
    /// NAT allocates ports sequentially (e.g., 41200, 41201, 41202)
    ///
    /// This is the best case for prediction — next port is highly predictable.
    Sequential {
        /// Step between consecutive allocations (e.g., 1 for sequential, 5 for skip-5)
        step: i32,
        /// Last observed port
        last_port: u16,
        /// Predicted next port based on pattern
        predicted_next: u16,
        /// Confidence level (0.0–1.0) based on pattern consistency
        confidence: f64,
    },

    /// NAT allocates ports randomly — no prediction possible
    ///
    /// Relay-only mode; coordinated punch won't improve success rate.
    Random {
        /// All observed ports from probes
        observed: Vec<u16>,
    },

    /// Not enough data to determine pattern
    Unknown,
}

impl PortPattern {
    /// Predict the next port allocation, if possible
    ///
    /// Returns `None` for `Random` or `Unknown` patterns.
    #[must_use]
    pub fn predict_next(&self) -> Option<u16> {
        match self {
            Self::Sequential {
                predicted_next,
                ..
            } => Some(*predicted_next),
            Self::Random {
                ..
            }
            | Self::Unknown => None,
        }
    }

    /// Get confidence level for the prediction
    #[must_use]
    pub fn confidence(&self) -> f64 {
        match self {
            Self::Sequential {
                confidence,
                ..
            } => *confidence,
            Self::Random {
                ..
            } => 0.0,
            Self::Unknown => 0.0,
        }
    }

    /// Whether this pattern supports coordinated punch
    #[must_use]
    pub fn supports_coordinated_punch(&self) -> bool {
        matches!(self, Self::Sequential { confidence, .. } if *confidence > 0.5)
    }
}

/// NAT type classification
///
/// Determines how aggressive NAT traversal needs to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
    #[default]
    Unknown,
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
