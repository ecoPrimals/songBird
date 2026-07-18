// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Transport Endpoint — Canonical type for structured service resolution.
//!
//! This is the ecosystem-standard wire type returned by `ipc.resolve` and
//! `capability.resolve`. Consumers use it to determine how to connect to a
//! resolved service without parsing URI strings.
//!
//! # Sourdough Standard
//!
//! This type is designed for adoption across all primals. It lives in
//! `songbird-types` (minimal dependencies) so any primal can deserialize
//! resolution responses without depending on the full IPC stack.
//!
//! # Wire Format
//!
//! Tagged enum via `#[serde(tag = "transport")]`:
//!
//! ```json
//! { "transport": "uds", "path": "/run/membrane/beardog.sock" }
//! { "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
//! { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
//! ```
//!
//! # Consumer Pattern
//!
//! ```rust,ignore
//! use songbird_types::TransportEndpoint;
//!
//! let response: serde_json::Value = rpc_call("ipc.resolve", params).await?;
//! let endpoint: TransportEndpoint = serde_json::from_value(response["endpoint"].clone())?;
//!
//! match endpoint {
//!     TransportEndpoint::Uds { ref path } => connect_unix(path).await?,
//!     TransportEndpoint::Tcp { ref host, port } => connect_tcp(host, port).await?,
//!     TransportEndpoint::MeshRelay { ref peer_id, ref capability } => {
//!         route_via_mesh(peer_id, capability).await?
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Structured transport endpoint — the canonical way to describe how to reach a service.
///
/// Returned by `ipc.resolve` and `capability.resolve` as the `endpoint` field.
/// Consumers match on the variant to select the appropriate connection strategy.
///
/// # Variants
///
/// - [`Uds`](Self::Uds): Unix Domain Socket — same-host inter-primal communication (fastest)
/// - [`Tcp`](Self::Tcp): Direct TCP — cross-host or container networking
/// - [`MeshRelay`](Self::MeshRelay): Songbird mesh relay — cross-gate NAT-traversal
///
/// # Ordering
///
/// Variants are ordered by locality (local → network → relay). Consumers should
/// prefer earlier variants when multiple resolution paths are available.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket — local primal on same host.
    ///
    /// Fastest path. Only available when both primals share a filesystem namespace.
    /// Typical in single-host deployments, containers with shared `/run`, and NUCLEUS.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket (e.g. `/run/user/1000/biomeos/beardog.sock`).
        /// May be prefixed with `@` for Linux abstract namespace sockets.
        path: String,
    },

    /// TCP — direct network connection.
    ///
    /// Used for cross-host communication where both endpoints have network
    /// reachability (no NAT in the way). Typical for VPS-to-VPS or LAN peers.
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address (IPv4, IPv6, or hostname).
        host: String,
        /// TCP port number.
        port: u16,
    },

    /// Mesh relay — primal reachable via Songbird's mesh network.
    ///
    /// Used when direct connectivity is unavailable (NAT, firewall, different
    /// physical networks). Traffic routes through Songbird's relay infrastructure.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier (e.g. `"strand-gate"`, `"east-gate"`).
        peer_id: String,
        /// Capability being resolved on the remote peer.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Whether this endpoint is local (same-host, no network hop).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds {
                ..
            } => true,
            Self::Tcp {
                host,
                ..
            } => crate::constants::is_loopback_host(host),
            Self::MeshRelay {
                ..
            } => false,
        }
    }

    /// Whether this endpoint requires network access.
    #[must_use]
    pub fn is_network(&self) -> bool {
        !self.is_local()
    }

    /// Whether this endpoint uses relay infrastructure (higher latency).
    #[must_use]
    pub const fn is_relayed(&self) -> bool {
        matches!(self, Self::MeshRelay { .. })
    }

    /// Returns the transport name as it appears in the wire format.
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            Self::Uds {
                ..
            } => "uds",
            Self::Tcp {
                ..
            } => "tcp",
            Self::MeshRelay {
                ..
            } => "mesh_relay",
        }
    }

    /// Returns a URI-style string for logging/diagnostics.
    ///
    /// Not for parsing — use the structured fields directly.
    #[must_use]
    pub fn display_uri(&self) -> String {
        match self {
            Self::Uds {
                path,
            } => {
                if let Some(abstract_name) = path.strip_prefix('@') {
                    format!("unix-abstract://{abstract_name}")
                } else {
                    format!("unix://{path}")
                }
            }
            Self::Tcp {
                host,
                port,
            } => {
                if host.contains(':') {
                    format!("tcp://[{host}]:{port}")
                } else {
                    format!("tcp://{host}:{port}")
                }
            }
            Self::MeshRelay {
                peer_id,
                capability,
            } => format!("mesh://{peer_id}/{capability}"),
        }
    }

    /// Returns the socket path if this is a UDS endpoint.
    #[must_use]
    pub fn uds_path(&self) -> Option<&str> {
        match self {
            Self::Uds {
                path,
            } => Some(path),
            _ => None,
        }
    }

    /// Returns (host, port) if this is a TCP endpoint.
    #[must_use]
    pub fn tcp_addr(&self) -> Option<(&str, u16)> {
        match self {
            Self::Tcp {
                host,
                port,
            } => Some((host, *port)),
            _ => None,
        }
    }

    /// Returns (`peer_id`, capability) if this is a mesh relay endpoint.
    #[must_use]
    pub fn mesh_peer(&self) -> Option<(&str, &str)> {
        match self {
            Self::MeshRelay {
                peer_id,
                capability,
            } => Some((peer_id, capability)),
            _ => None,
        }
    }

    /// Construct a UDS endpoint from a socket path.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds {
            path: path.into(),
        }
    }

    /// Construct a TCP endpoint from host and port.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Construct a mesh relay endpoint.
    #[must_use]
    pub fn mesh_relay(peer_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self::MeshRelay {
            peer_id: peer_id.into(),
            capability: capability.into(),
        }
    }
}

impl fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_uri())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn uds_serializes_tagged() {
        let ep = TransportEndpoint::uds("/run/membrane/beardog.sock");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "uds");
        assert_eq!(json["path"], "/run/membrane/beardog.sock");
        assert!(json.get("host").is_none());
    }

    #[test]
    fn tcp_serializes_tagged() {
        let ep = TransportEndpoint::tcp("192.168.1.144", 7700);
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "tcp");
        assert_eq!(json["host"], "192.168.1.144");
        assert_eq!(json["port"], 7700);
    }

    #[test]
    fn mesh_relay_serializes_tagged() {
        let ep = TransportEndpoint::mesh_relay("strand-gate", "security");
        let json = serde_json::to_value(&ep).unwrap();
        assert_eq!(json["transport"], "mesh_relay");
        assert_eq!(json["peer_id"], "strand-gate");
        assert_eq!(json["capability"], "security");
    }

    #[test]
    fn uds_round_trips() {
        let ep = TransportEndpoint::uds("/tmp/test.sock");
        let json_str = serde_json::to_string(&ep).unwrap();
        let de: TransportEndpoint = serde_json::from_str(&json_str).unwrap();
        assert_eq!(ep, de);
    }

    #[test]
    fn tcp_round_trips() {
        let ep = TransportEndpoint::tcp("10.0.0.1", 8080);
        let json_str = serde_json::to_string(&ep).unwrap();
        let de: TransportEndpoint = serde_json::from_str(&json_str).unwrap();
        assert_eq!(ep, de);
    }

    #[test]
    fn mesh_relay_round_trips() {
        let ep = TransportEndpoint::mesh_relay("east-gate", "crypto");
        let json_str = serde_json::to_string(&ep).unwrap();
        let de: TransportEndpoint = serde_json::from_str(&json_str).unwrap();
        assert_eq!(ep, de);
    }

    #[test]
    fn deserializes_from_wire_examples() {
        let uds: TransportEndpoint =
            serde_json::from_str(r#"{"transport":"uds","path":"/run/membrane/beardog.sock"}"#)
                .unwrap();
        assert_eq!(uds, TransportEndpoint::uds("/run/membrane/beardog.sock"));

        let tcp: TransportEndpoint =
            serde_json::from_str(r#"{"transport":"tcp","host":"192.168.1.144","port":7700}"#)
                .unwrap();
        assert_eq!(tcp, TransportEndpoint::tcp("192.168.1.144", 7700));

        let relay: TransportEndpoint = serde_json::from_str(
            r#"{"transport":"mesh_relay","peer_id":"strand-gate","capability":"security"}"#,
        )
        .unwrap();
        assert_eq!(relay, TransportEndpoint::mesh_relay("strand-gate", "security"));
    }

    #[test]
    fn is_local_for_uds() {
        assert!(TransportEndpoint::uds("/tmp/test.sock").is_local());
    }

    #[test]
    fn is_local_for_localhost_tcp() {
        assert!(TransportEndpoint::tcp("127.0.0.1", 8080).is_local());
        assert!(TransportEndpoint::tcp("::1", 8080).is_local());
        assert!(TransportEndpoint::tcp("localhost", 8080).is_local());
    }

    #[test]
    fn is_not_local_for_remote_tcp() {
        assert!(!TransportEndpoint::tcp("192.168.1.5", 7700).is_local());
        assert!(!TransportEndpoint::tcp("strand.primals.eco", 7700).is_local());
    }

    #[test]
    fn is_not_local_for_mesh_relay() {
        assert!(!TransportEndpoint::mesh_relay("strand-gate", "crypto").is_local());
    }

    #[test]
    fn is_relayed() {
        assert!(!TransportEndpoint::uds("/tmp/x.sock").is_relayed());
        assert!(!TransportEndpoint::tcp("10.0.0.1", 80).is_relayed());
        assert!(TransportEndpoint::mesh_relay("peer", "cap").is_relayed());
    }

    #[test]
    fn transport_name_matches_wire() {
        assert_eq!(TransportEndpoint::uds("/x").transport_name(), "uds");
        assert_eq!(TransportEndpoint::tcp("h", 1).transport_name(), "tcp");
        assert_eq!(TransportEndpoint::mesh_relay("p", "c").transport_name(), "mesh_relay");
    }

    #[test]
    fn display_uri_formats() {
        assert_eq!(TransportEndpoint::uds("/run/test.sock").display_uri(), "unix:///run/test.sock");
        assert_eq!(
            TransportEndpoint::uds("@abstract-name").display_uri(),
            "unix-abstract://abstract-name"
        );
        assert_eq!(TransportEndpoint::tcp("10.0.0.1", 7700).display_uri(), "tcp://10.0.0.1:7700");
        assert_eq!(TransportEndpoint::tcp("::1", 8080).display_uri(), "tcp://[::1]:8080");
        assert_eq!(
            TransportEndpoint::mesh_relay("east-gate", "crypto").display_uri(),
            "mesh://east-gate/crypto"
        );
    }

    #[test]
    fn accessor_methods() {
        let uds = TransportEndpoint::uds("/tmp/sock");
        assert_eq!(uds.uds_path(), Some("/tmp/sock"));
        assert_eq!(uds.tcp_addr(), None);
        assert_eq!(uds.mesh_peer(), None);

        let tcp = TransportEndpoint::tcp("host", 99);
        assert_eq!(tcp.uds_path(), None);
        assert_eq!(tcp.tcp_addr(), Some(("host", 99)));
        assert_eq!(tcp.mesh_peer(), None);

        let relay = TransportEndpoint::mesh_relay("p", "c");
        assert_eq!(relay.uds_path(), None);
        assert_eq!(relay.tcp_addr(), None);
        assert_eq!(relay.mesh_peer(), Some(("p", "c")));
    }

    #[test]
    fn display_trait_matches_display_uri() {
        let ep = TransportEndpoint::tcp("host.example", 443);
        assert_eq!(format!("{ep}"), ep.display_uri());
    }

    #[test]
    fn hash_impl_works() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TransportEndpoint::uds("/a"));
        set.insert(TransportEndpoint::uds("/a"));
        set.insert(TransportEndpoint::tcp("h", 1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn constructors_are_ergonomic() {
        let _ = TransportEndpoint::uds("/path");
        let _ = TransportEndpoint::tcp("host", 80);
        let _ = TransportEndpoint::mesh_relay("peer", "cap");
        let _ = TransportEndpoint::uds(String::from("/dynamic"));
        let _ = TransportEndpoint::tcp(String::from("dynamic"), 443);
    }
}
