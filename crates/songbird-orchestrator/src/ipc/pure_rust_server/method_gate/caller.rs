// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Caller identity and connection context for JSON-RPC authorization.

use super::token::TokenClaims;

/// Peer credentials extracted from `SO_PEERCRED` on Unix sockets.
#[derive(Debug, Clone)]
pub struct PeerCredentials {
    /// Process ID of the caller (if available).
    pub pid: Option<u32>,
    /// User ID of the caller.
    pub uid: u32,
}

/// Identity and authorization context for an incoming RPC call.
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Optional bearer / capability token sent in the request.
    pub bearer_token: Option<String>,
    /// Verified claims from the token (populated after async verification).
    pub verified_claims: Option<TokenClaims>,
    /// Peer credentials from `SO_PEERCRED` (Unix socket only).
    pub peer: Option<PeerCredentials>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

/// How the caller connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// Local Unix domain socket.
    Unix,
    /// TCP loopback (`127.0.0.1` / `::1`).
    Loopback,
    /// Remote TCP connection.
    Remote,
}

impl CallerContext {
    /// Create a caller context for a Unix domain socket connection (no credentials).
    #[must_use]
    pub const fn from_unix() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Extract peer credentials from a `UnixStream` via `SO_PEERCRED`.
    ///
    /// Falls back to credential-less context if extraction fails.
    #[cfg(unix)]
    #[must_use]
    pub fn from_unix_stream(stream: &tokio::net::UnixStream) -> Self {
        match stream.peer_cred() {
            Ok(cred) => {
                let peer = PeerCredentials {
                    pid: cred.pid().map(|p| p as u32),
                    uid: cred.uid(),
                };
                tracing::trace!(uid = peer.uid, pid = ?peer.pid, "UDS peer credentials extracted");
                Self {
                    bearer_token: None,
                    verified_claims: None,
                    peer: Some(peer),
                    origin: ConnectionOrigin::Unix,
                }
            }
            Err(e) => {
                tracing::debug!(error = %e, "SO_PEERCRED extraction failed — proceeding without");
                Self::from_unix()
            }
        }
    }

    /// Build a caller context for loopback TCP with no peer credentials.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Build a caller context for a remote TCP connection.
    #[must_use]
    pub const fn remote() -> Self {
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin: ConnectionOrigin::Remote,
        }
    }

    /// Build a caller context from a TCP peer address.
    ///
    /// Checks whether the peer IP is a loopback address (`127.0.0.1`, `::1`)
    /// and sets `ConnectionOrigin` accordingly.
    #[must_use]
    pub const fn from_tcp(addr: std::net::SocketAddr) -> Self {
        let origin = if addr.ip().is_loopback() {
            ConnectionOrigin::Loopback
        } else {
            ConnectionOrigin::Remote
        };
        Self {
            bearer_token: None,
            verified_claims: None,
            peer: None,
            origin,
        }
    }

    /// Attach a bearer token (extracted from `_bearer_token` in params) to
    /// this context. Returns a new context with the token set.
    #[must_use]
    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.bearer_token = Some(token);
        self
    }

    /// Attach verified claims to this context (after async token verification).
    #[must_use]
    pub fn with_verified_claims(mut self, claims: TokenClaims) -> Self {
        self.verified_claims = Some(claims);
        self
    }
}
