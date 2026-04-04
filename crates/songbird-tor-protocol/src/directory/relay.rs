// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Relay information and selection

use std::net::IpAddr;

bitflags::bitflags! {
    /// Relay flags from consensus
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RelayFlags: u16 {
        /// Directory authority
        const AUTHORITY = 1 << 0;
        /// Bad exit relay
        const BAD_EXIT = 1 << 1;
        /// Exit relay
        const EXIT = 1 << 2;
        /// Fast relay (>= 100 KB/s)
        const FAST = 1 << 3;
        /// Guard relay
        const GUARD = 1 << 4;
        /// Hidden service directory
        const HSDIR = 1 << 5;
        /// Currently running
        const RUNNING = 1 << 6;
        /// Stable (high uptime)
        const STABLE = 1 << 7;
        /// Valid (passes basic checks)
        const VALID = 1 << 8;
        /// Version 2 directory
        const V2DIR = 1 << 9;
    }
}

/// Information about a Tor relay
#[derive(Debug, Clone)]
pub struct RelayInfo {
    /// Relay nickname
    pub nickname: String,
    /// Identity fingerprint (SHA1 of Ed25519 public key)
    pub fingerprint: [u8; 20],
    /// IPv4 address
    pub address: IpAddr,
    /// OR port (onion routing)
    pub or_port: u16,
    /// Directory port (optional)
    pub dir_port: Option<u16>,
    /// Relay flags
    pub flags: RelayFlags,
    /// Bandwidth (bytes/second)
    pub bandwidth: u64,
    /// ntor onion key (X25519 public key)
    pub ntor_key: Option<[u8; 32]>,
    /// Tor version
    pub version: Option<String>,
}

impl RelayInfo {
    /// Check if relay can be used as guard
    #[must_use]
    pub const fn is_guard(&self) -> bool {
        self.flags.contains(RelayFlags::GUARD)
            && self.flags.contains(RelayFlags::FAST)
            && self.flags.contains(RelayFlags::STABLE)
            && self.flags.contains(RelayFlags::VALID)
            && self.flags.contains(RelayFlags::RUNNING)
    }

    /// Check if relay can be used as middle
    #[must_use]
    pub const fn is_middle(&self) -> bool {
        self.flags.contains(RelayFlags::FAST)
            && self.flags.contains(RelayFlags::STABLE)
            && self.flags.contains(RelayFlags::VALID)
            && self.flags.contains(RelayFlags::RUNNING)
    }

    /// Check if relay is hidden service directory
    #[must_use]
    pub const fn is_hsdir(&self) -> bool {
        self.flags.contains(RelayFlags::HSDIR)
            && self.flags.contains(RelayFlags::VALID)
            && self.flags.contains(RelayFlags::RUNNING)
    }
}

/// Circuit path (3 hops)
#[derive(Debug, Clone)]
pub struct CircuitPath {
    /// Guard relay (entry)
    pub guard: RelayInfo,
    /// Middle relay
    pub middle: RelayInfo,
    /// Exit/HSDir relay
    pub exit: RelayInfo,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn relay(name: &str, fp_byte: u8, flags: RelayFlags) -> RelayInfo {
        let mut fingerprint = [0u8; 20];
        fingerprint[0] = fp_byte;
        RelayInfo {
            nickname: name.to_string(),
            fingerprint,
            address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, fp_byte)),
            or_port: 443,
            dir_port: None,
            flags,
            bandwidth: 10_000,
            ntor_key: None,
            version: None,
        }
    }

    #[test]
    fn is_guard_requires_full_flag_set() {
        let base = RelayFlags::GUARD
            | RelayFlags::FAST
            | RelayFlags::STABLE
            | RelayFlags::VALID
            | RelayFlags::RUNNING;
        let ok = relay("g", 1, base);
        assert!(ok.is_guard());

        let no_stable = base & !RelayFlags::STABLE;
        assert!(!relay("g2", 2, no_stable).is_guard());
    }

    #[test]
    fn is_middle_rejects_missing_fast() {
        let flags = RelayFlags::STABLE | RelayFlags::VALID | RelayFlags::RUNNING;
        assert!(!relay("m", 3, flags).is_middle());
    }

    #[test]
    fn is_hsdir_requires_hsdir_valid_running() {
        let ok = RelayFlags::HSDIR | RelayFlags::VALID | RelayFlags::RUNNING;
        assert!(relay("h", 4, ok).is_hsdir());

        let no_hsdir = RelayFlags::VALID | RelayFlags::RUNNING;
        assert!(!relay("h2", 5, no_hsdir).is_hsdir());
    }

    #[test]
    fn relay_flags_bit_assignment_distinct() {
        assert_ne!(RelayFlags::EXIT.bits(), RelayFlags::GUARD.bits());
        assert!(RelayFlags::AUTHORITY.contains(RelayFlags::AUTHORITY));
    }

    #[test]
    fn circuit_path_holds_three_distinct_relays() {
        let g = RelayFlags::GUARD
            | RelayFlags::FAST
            | RelayFlags::STABLE
            | RelayFlags::VALID
            | RelayFlags::RUNNING;
        let m = RelayFlags::FAST | RelayFlags::STABLE | RelayFlags::VALID | RelayFlags::RUNNING;
        let e = RelayFlags::HSDIR | RelayFlags::VALID | RelayFlags::RUNNING;
        let path = CircuitPath {
            guard: relay("guard", 1, g),
            middle: relay("mid", 2, m),
            exit: relay("exit", 3, e),
        };
        assert_ne!(path.guard.fingerprint, path.middle.fingerprint);
        assert_ne!(path.exit.fingerprint, path.guard.fingerprint);
    }
}
