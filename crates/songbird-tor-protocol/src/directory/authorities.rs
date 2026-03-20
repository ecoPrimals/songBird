// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tor directory authorities
//!
//! Hardcoded list of Tor directory authorities for consensus fetching.
//! These are the trusted nodes that vote on network consensus.

use std::net::{IpAddr, Ipv4Addr};

/// Helper to create IPv4 address at compile time
const fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

/// Tor directory authority
#[derive(Debug, Clone)]
pub struct DirectoryAuthority {
    /// Authority nickname
    pub nickname: &'static str,
    /// IPv4 address
    pub address: IpAddr,
    /// Directory port
    pub dir_port: u16,
    /// OR port (onion routing)
    pub or_port: u16,
    /// Identity fingerprint (SHA1 of Ed25519 public key)
    pub fingerprint: [u8; 20],
}

/// List of Tor directory authorities (as of February 2026)
///
/// Source: <https://consensus-health.torproject.org/>
/// Note: Order matters - put most reliable/fastest first
pub const DIRECTORY_AUTHORITIES: &[DirectoryAuthority] = &[
    // gabelmoo (gabelmoo.torproject.org) - FIRST: known to be reliable
    DirectoryAuthority {
        nickname: "gabelmoo",
        address: ipv4(131, 188, 40, 189),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0xF2, 0x04, 0x4D, 0x13, 0xDA, 0x1D, 0xB6, 0x8F, 0x24, 0x17, 0x4D, 0x98, 0xC3, 0x95,
            0xF8, 0x4A, 0x8C, 0x45, 0x7A, 0x4F,
        ],
    },
    // dannenberg (dannenberg.torproject.org)
    DirectoryAuthority {
        nickname: "dannenberg",
        address: ipv4(193, 23, 244, 244),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0x74, 0x85, 0x85, 0x75, 0xC2, 0x88, 0x6B, 0x76, 0xF6, 0xBE, 0x7E, 0x5C, 0xD0, 0x8D,
            0x4F, 0x8B, 0x6D, 0xE1, 0x0F, 0x24,
        ],
    },
    // bastet (bastet.torproject.org)
    DirectoryAuthority {
        nickname: "bastet",
        address: ipv4(204, 13, 164, 118),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0x27, 0x10, 0x2B, 0xC1, 0x23, 0xE7, 0xDB, 0x31, 0x9C, 0x30, 0x97, 0xBA, 0xB8, 0x8D,
            0xB8, 0xF9, 0x23, 0x44, 0x64, 0x42,
        ],
    },
    // moria1 (moria.csail.mit.edu) - may have connectivity issues
    DirectoryAuthority {
        nickname: "moria1",
        address: ipv4(128, 31, 0, 34),
        dir_port: 9131,
        or_port: 9101,
        fingerprint: [
            0x96, 0x95, 0xDF, 0xC3, 0x5F, 0xFE, 0xB8, 0x61, 0x32, 0x9B, 0x9F, 0x1A, 0xB0, 0x4C,
            0x46, 0x39, 0x70, 0x20, 0xCE, 0x31,
        ],
    },
    // tor26 (tor26.torproject.org)
    DirectoryAuthority {
        nickname: "tor26",
        address: ipv4(86, 59, 21, 38),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0x84, 0x7B, 0x1F, 0x85, 0x03, 0x44, 0xD7, 0x87, 0x64, 0x91, 0xA5, 0x48, 0x92, 0xF9,
            0x04, 0x93, 0x4E, 0x4E, 0xB8, 0x5D,
        ],
    },
    // maatuska (maatuska.torproject.org)
    DirectoryAuthority {
        nickname: "maatuska",
        address: ipv4(171, 25, 193, 9),
        dir_port: 443,
        or_port: 80,
        fingerprint: [
            0xBD, 0x6A, 0x26, 0xDF, 0xC8, 0x26, 0x8E, 0x0C, 0x3C, 0x52, 0xFC, 0xF9, 0x9C, 0x78,
            0x5B, 0xF9, 0xD1, 0x4B, 0x83, 0xEE,
        ],
    },
    // longclaw (longclaw.torproject.org)
    DirectoryAuthority {
        nickname: "longclaw",
        address: ipv4(199, 58, 81, 140),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0x23, 0xD5, 0xAB, 0x88, 0x19, 0x23, 0x95, 0x72, 0xF7, 0x25, 0x68, 0x68, 0x14, 0x2E,
            0x71, 0x5D, 0x6A, 0x2B, 0xB5, 0xB1,
        ],
    },
    // faravahar (faravahar.torproject.org)
    DirectoryAuthority {
        nickname: "faravahar",
        address: ipv4(154, 35, 175, 225),
        dir_port: 80,
        or_port: 443,
        fingerprint: [
            0xCF, 0x6D, 0x0A, 0xAF, 0xB3, 0x85, 0xBE, 0x71, 0xB8, 0xE1, 0x11, 0xFC, 0x5C, 0xDD,
            0x4B, 0x81, 0x9C, 0x36, 0x08, 0xA3,
        ],
    },
];

impl DirectoryAuthority {
    /// Get consensus URL for this authority
    #[must_use]
    pub fn consensus_url(&self) -> String {
        format!("http://{}:{}/tor/status-vote/current/consensus", self.address, self.dir_port)
    }

    /// Get descriptor URL for a relay
    #[must_use]
    pub fn descriptor_url(&self, fingerprint: &str) -> String {
        format!("http://{}:{}/tor/server/fp/{}", self.address, self.dir_port, fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorities_count() {
        assert_eq!(DIRECTORY_AUTHORITIES.len(), 8);
    }

    #[test]
    fn test_consensus_url() {
        // gabelmoo is first (most reliable)
        let auth = &DIRECTORY_AUTHORITIES[0];
        assert_eq!(
            auth.consensus_url(),
            "http://131.188.40.189:80/tor/status-vote/current/consensus"
        );
    }

    #[test]
    fn test_all_authorities_valid() {
        for auth in DIRECTORY_AUTHORITIES {
            assert!(!auth.nickname.is_empty());
            assert!(auth.dir_port > 0);
            assert!(auth.or_port > 0);
            assert_eq!(auth.fingerprint.len(), 20);
        }
    }
}
