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
    pub fn is_guard(&self) -> bool {
        self.flags.contains(RelayFlags::GUARD)
            && self.flags.contains(RelayFlags::FAST)
            && self.flags.contains(RelayFlags::STABLE)
            && self.flags.contains(RelayFlags::VALID)
            && self.flags.contains(RelayFlags::RUNNING)
    }

    /// Check if relay can be used as middle
    pub fn is_middle(&self) -> bool {
        self.flags.contains(RelayFlags::FAST)
            && self.flags.contains(RelayFlags::STABLE)
            && self.flags.contains(RelayFlags::VALID)
            && self.flags.contains(RelayFlags::RUNNING)
    }

    /// Check if relay is hidden service directory
    pub fn is_hsdir(&self) -> bool {
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
