// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    pub const fn predict_next(&self) -> Option<u16> {
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
    pub const fn confidence(&self) -> f64 {
        match self {
            Self::Sequential {
                confidence,
                ..
            } => *confidence,
            Self::Random {
                ..
            }
            | Self::Unknown => 0.0,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use serde_json::{from_value, to_value};
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn public_endpoint_roundtrip_serde() {
        let ep = PublicEndpoint {
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, 3478)),
            nat_type: NatType::FullCone,
        };
        let v = to_value(ep).unwrap();
        let back: PublicEndpoint = from_value(v).unwrap();
        assert_eq!(ep, back);
    }

    #[test]
    fn nat_type_default_is_unknown() {
        assert_eq!(NatType::default(), NatType::Unknown);
    }

    #[test]
    fn nat_type_display() {
        assert_eq!(NatType::None.to_string(), "None (no NAT)");
        assert_eq!(NatType::FullCone.to_string(), "Full Cone");
        assert_eq!(NatType::RestrictedCone.to_string(), "Restricted Cone");
        assert_eq!(NatType::PortRestrictedCone.to_string(), "Port-Restricted Cone");
        assert_eq!(NatType::Symmetric.to_string(), "Symmetric");
        assert_eq!(NatType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn port_pattern_sequential_predict_and_confidence() {
        let p = PortPattern::Sequential {
            step: 1,
            last_port: 1000,
            predicted_next: 1001,
            confidence: 0.9,
        };
        assert_eq!(p.predict_next(), Some(1001));
        assert!((p.confidence() - 0.9).abs() < f64::EPSILON);
        assert!(p.supports_coordinated_punch());
    }

    #[test]
    fn port_pattern_sequential_low_confidence_no_punch() {
        let p = PortPattern::Sequential {
            step: 2,
            last_port: 1000,
            predicted_next: 1002,
            confidence: 0.4,
        };
        assert!(!p.supports_coordinated_punch());
    }

    #[test]
    fn port_pattern_random_unknown_no_predict() {
        let r = PortPattern::Random {
            observed: vec![1, 2, 3],
        };
        assert_eq!(r.predict_next(), None);
        assert!(r.confidence().abs() < f64::EPSILON);
        assert!(!r.supports_coordinated_punch());

        let u = PortPattern::Unknown;
        assert_eq!(u.predict_next(), None);
        assert!(u.confidence().abs() < f64::EPSILON);
        assert!(!u.supports_coordinated_punch());
    }

    #[test]
    fn port_pattern_serde_roundtrip_variants() {
        let seq = PortPattern::Sequential {
            step: 1,
            last_port: 5000,
            predicted_next: 5001,
            confidence: 0.75,
        };
        let v = to_value(&seq).unwrap();
        let back: PortPattern = from_value(v).unwrap();
        assert_eq!(seq, back);

        let rand = PortPattern::Random {
            observed: vec![4000, 4001],
        };
        let v = to_value(&rand).unwrap();
        let back: PortPattern = from_value(v).unwrap();
        assert_eq!(rand, back);

        let unk = PortPattern::Unknown;
        let v = to_value(&unk).unwrap();
        let back: PortPattern = from_value(v).unwrap();
        assert_eq!(unk, back);
    }

    #[test]
    fn port_pattern_sequential_boundary_confidence_half() {
        let p = PortPattern::Sequential {
            step: 1,
            last_port: 1,
            predicted_next: 2,
            confidence: 0.5,
        };
        assert!(!p.supports_coordinated_punch());
    }

    #[test]
    fn port_pattern_sequential_just_above_threshold() {
        let p = PortPattern::Sequential {
            step: 1,
            last_port: 1,
            predicted_next: 2,
            confidence: 0.51,
        };
        assert!(p.supports_coordinated_punch());
    }

    #[test]
    fn public_endpoint_debug_nonempty() {
        let ep = PublicEndpoint {
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, 9)),
            nat_type: NatType::Symmetric,
        };
        assert!(!format!("{ep:?}").is_empty(), "Debug impl should be non-empty");
    }

    #[test]
    fn nat_type_copy_roundtrip() {
        let n = NatType::PortRestrictedCone;
        let n2 = n;
        assert_eq!(n, n2);
    }

    #[test]
    fn port_pattern_predict_next_sequential_only() {
        let p = PortPattern::Sequential {
            step: 3,
            last_port: 10,
            predicted_next: 13,
            confidence: 1.0,
        };
        assert_eq!(p.predict_next(), Some(13));
    }
}
