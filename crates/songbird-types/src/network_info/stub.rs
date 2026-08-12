// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Non-Linux stubs for network route detection.

use std::net::Ipv4Addr;

/// A default-route entry (unavailable on non-Linux).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry {
    /// Network interface name.
    pub interface: String,
    /// Gateway IPv4 address.
    pub gateway: Ipv4Addr,
}

/// Returns the default network route (unavailable on non-Linux).
#[must_use]
pub fn default_route() -> Option<RouteEntry> {
    None
}

/// Returns whether a default route exists (always `false` on non-Linux).
#[must_use]
pub fn has_default_route() -> bool {
    false
}

/// Returns local IPv4 addresses from the FIB trie (unavailable on non-Linux).
#[must_use]
pub fn local_ipv4_from_fib_trie() -> Vec<Ipv4Addr> {
    Vec::new()
}

/// Returns whether a public IPv4 interface exists (always `false` on non-Linux).
#[must_use]
pub fn has_public_ipv4_interface() -> bool {
    false
}
