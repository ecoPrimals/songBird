// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay endpoint descriptors and transport kinds for the beacon mesh.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// A relay endpoint (could be direct, family relay, or Tor)
#[derive(Debug, Clone)]
pub struct RelayEndpoint {
    /// Node ID of the relay
    pub node_id: String,
    /// How to reach this relay
    pub endpoint_type: EndpointType,
    /// Last measured latency
    pub latency: Option<Duration>,
    /// Last successful contact
    pub last_seen: Instant,
    /// Is this relay currently reachable?
    pub reachable: bool,
}

/// Type of relay endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointType {
    /// Direct UDP connection (hole punch succeeded)
    Direct {
        /// Peer's reachable UDP address.
        addr: SocketAddr,
    },
    /// Relay through another family member
    FamilyRelay {
        /// Node id of the relay participant.
        relay_node_id: String,
    },
    /// Tor onion service (bootstrap/fallback)
    TorOnion {
        /// `.onion` hostname or full rendezvous URL.
        onion_addr: String,
    },
    /// Local network (same LAN)
    Local {
        /// LAN peer address.
        addr: SocketAddr,
    },
    /// Encrypted overlay network (`WireGuard`, etc.)
    Overlay {
        /// Peer address on the overlay subnet.
        addr: SocketAddr,
        /// Which overlay this belongs to (e.g. "wireguard").
        overlay_name: String,
    },
}

impl EndpointType {
    /// Priority for selection (lower = better)
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Local {
                ..
            }
            | Self::Overlay {
                ..
            } => 0,
            Self::Direct {
                ..
            } => 1,
            Self::FamilyRelay {
                ..
            } => 2,
            Self::TorOnion {
                ..
            } => 3,
        }
    }

    /// Extract the IP address if this endpoint type has one.
    #[must_use]
    pub const fn address(&self) -> Option<std::net::IpAddr> {
        match self {
            Self::Direct {
                addr,
            }
            | Self::Local {
                addr,
            }
            | Self::Overlay {
                addr,
                ..
            } => Some(addr.ip()),
            Self::FamilyRelay {
                ..
            }
            | Self::TorOnion {
                ..
            } => None,
        }
    }

    /// Extract the full socket address (IP + port) if available.
    ///
    /// Prefer this over [`Self::address`] when connecting — it preserves the
    /// port from the peer's advertised endpoint rather than falling back to
    /// a hardcoded default.
    #[must_use]
    pub const fn socket_addr(&self) -> Option<SocketAddr> {
        match self {
            Self::Direct {
                addr,
            }
            | Self::Local {
                addr,
            }
            | Self::Overlay {
                addr,
                ..
            } => Some(*addr),
            Self::FamilyRelay {
                ..
            }
            | Self::TorOnion {
                ..
            } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{EndpointType, RelayEndpoint};
    use std::net::SocketAddr;
    use std::time::Instant;

    #[test]
    fn endpoint_type_priority_local_is_best() {
        let local = EndpointType::Local {
            addr: "192.168.1.1:9000".parse().unwrap(),
        };
        assert_eq!(local.priority(), 0);
    }

    #[test]
    fn endpoint_type_priority_ordering_strictly_increases() {
        let local = EndpointType::Local {
            addr: "10.0.0.2:1".parse().unwrap(),
        };
        let direct = EndpointType::Direct {
            addr: "198.51.100.1:2".parse().unwrap(),
        };
        let family = EndpointType::FamilyRelay {
            relay_node_id: "relay-a".into(),
        };
        let tor = EndpointType::TorOnion {
            onion_addr: "abcdefghijklmnop.onion".into(),
        };
        assert!(local.priority() < direct.priority());
        assert!(direct.priority() < family.priority());
        assert!(family.priority() < tor.priority());
    }

    #[test]
    fn endpoint_type_direct_equality_matches_addr() {
        let a: SocketAddr = "203.0.113.5:4444".parse().unwrap();
        let b: SocketAddr = "203.0.113.5:4444".parse().unwrap();
        assert_eq!(
            EndpointType::Direct {
                addr: a,
            },
            EndpointType::Direct {
                addr: b,
            }
        );
    }

    #[test]
    fn endpoint_type_direct_inequality_when_addr_differs() {
        let x = EndpointType::Direct {
            addr: "198.18.0.1:1".parse().unwrap(),
        };
        let y = EndpointType::Direct {
            addr: "198.18.0.2:1".parse().unwrap(),
        };
        assert_ne!(x, y);
    }

    #[test]
    fn endpoint_type_family_relay_clone_and_eq() {
        let a = EndpointType::FamilyRelay {
            relay_node_id: "node-7".into(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn endpoint_type_tor_onion_preserves_hostname() {
        let t = EndpointType::TorOnion {
            onion_addr: "service.onion".into(),
        };
        match t {
            EndpointType::TorOnion {
                onion_addr,
            } => assert_eq!(onion_addr, "service.onion"),
            _ => panic!("expected TorOnion"),
        }
    }

    #[test]
    fn relay_endpoint_clone_preserves_node_id() {
        let ep = RelayEndpoint {
            node_id: "beacon-1".into(),
            endpoint_type: EndpointType::Direct {
                addr: "127.0.0.1:9".parse().unwrap(),
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: true,
        };
        let cloned = ep.clone();
        assert_eq!(cloned.node_id, ep.node_id);
        assert_eq!(cloned.node_id.as_str(), "beacon-1");
        assert!(cloned.reachable);
    }

    #[test]
    fn relay_endpoint_latency_some_duration() {
        let ep = RelayEndpoint {
            node_id: "n".into(),
            endpoint_type: EndpointType::Local {
                addr: "[::1]:1234".parse().unwrap(),
            },
            latency: Some(std::time::Duration::from_millis(12)),
            last_seen: Instant::now(),
            reachable: false,
        };
        assert_eq!(ep.latency, Some(std::time::Duration::from_millis(12)));
        assert!(!ep.reachable);
    }

    #[test]
    fn endpoint_type_debug_contains_variant_name() {
        let d = format!(
            "{:?}",
            EndpointType::Direct {
                addr: "0.0.0.0:0".parse().unwrap(),
            }
        );
        assert!(d.contains("Direct"), "{d}");
    }

    #[test]
    fn endpoint_type_priority_tor_is_worst_among_documented_classes() {
        let tor = EndpointType::TorOnion {
            onion_addr: "x.onion".into(),
        };
        assert_eq!(tor.priority(), 3);
    }

    #[test]
    fn endpoint_type_local_priority_zero_even_with_ipv6() {
        let local = EndpointType::Local {
            addr: "[fd00::1]:8080".parse().unwrap(),
        };
        assert_eq!(local.priority(), 0);
    }

    #[test]
    fn endpoint_type_overlay_priority_zero_same_as_local() {
        let overlay = EndpointType::Overlay {
            addr: "10.13.37.5:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        assert_eq!(overlay.priority(), 0);
    }

    #[test]
    fn endpoint_type_overlay_preferred_over_direct() {
        let overlay = EndpointType::Overlay {
            addr: "10.13.37.5:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        let direct = EndpointType::Direct {
            addr: "203.0.113.5:7700".parse().unwrap(),
        };
        assert!(overlay.priority() < direct.priority());
    }

    #[test]
    fn endpoint_type_overlay_address_extraction() {
        let overlay = EndpointType::Overlay {
            addr: "10.13.37.2:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        assert_eq!(overlay.address(), Some("10.13.37.2".parse().unwrap()));
        assert_eq!(overlay.socket_addr(), Some("10.13.37.2:7700".parse().unwrap()));
    }

    #[test]
    fn endpoint_type_overlay_equality() {
        let a = EndpointType::Overlay {
            addr: "10.13.37.5:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        let b = EndpointType::Overlay {
            addr: "10.13.37.5:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn endpoint_type_overlay_inequality_different_addr() {
        let a = EndpointType::Overlay {
            addr: "10.13.37.5:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        let b = EndpointType::Overlay {
            addr: "10.13.37.6:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn relay_endpoint_unreachable_still_constructible() {
        let ep = RelayEndpoint {
            node_id: "offline".into(),
            endpoint_type: EndpointType::FamilyRelay {
                relay_node_id: "other".into(),
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: false,
        };
        assert!(!ep.reachable);
        match ep.endpoint_type {
            EndpointType::FamilyRelay {
                relay_node_id,
            } => assert_eq!(relay_node_id, "other"),
            _ => panic!("expected FamilyRelay"),
        }
    }
}
