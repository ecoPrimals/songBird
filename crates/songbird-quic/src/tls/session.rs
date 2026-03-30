// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Encryption level management for QUIC connections.
//!
//! QUIC uses four encryption levels (RFC 9001 Section 4.1):
//! - Initial: bootstraps the handshake
//! - Handshake: protects handshake completion
//! - 0-RTT: early data (client only)
//! - 1-RTT (Application): post-handshake data

use crate::crypto::initial_keys::DirectionalKeys;
use crate::crypto::provider::QuicCipherSuite;

/// QUIC encryption levels (RFC 9001 Section 4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EncryptionLevel {
    /// Initial encryption level (derived from DCID).
    Initial,
    /// 0-RTT encryption level (early data, client to server only).
    ZeroRtt,
    /// Handshake encryption level (derived from TLS handshake).
    Handshake,
    /// 1-RTT / Application encryption level (derived from TLS application secrets).
    OneRtt,
}

impl EncryptionLevel {
    /// Returns `true` if this is the initial level.
    #[must_use]
    pub const fn is_initial(self) -> bool {
        matches!(self, Self::Initial)
    }

    /// Returns `true` if this level carries application data.
    #[must_use]
    pub const fn is_application(self) -> bool {
        matches!(self, Self::OneRtt)
    }
}

/// Keying material for one encryption level.
#[derive(Debug, Clone)]
pub struct LevelKeys {
    /// Keys for the local (sending) direction.
    pub local: DirectionalKeys,
    /// Keys for the remote (receiving) direction.
    pub remote: DirectionalKeys,
    /// Cipher suite for this level.
    pub suite: QuicCipherSuite,
}

/// Manages encryption keys across all QUIC encryption levels.
///
/// As the TLS handshake progresses, keys for each level are installed
/// and old levels are eventually discarded.
#[derive(Debug, Default)]
pub struct CryptoSession {
    initial: Option<LevelKeys>,
    zero_rtt: Option<LevelKeys>,
    handshake: Option<LevelKeys>,
    one_rtt: Option<LevelKeys>,
    /// Current key phase for 1-RTT key updates.
    key_phase: bool,
}

impl CryptoSession {
    /// Create a new empty session.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Install keys for a given encryption level.
    pub fn install_keys(&mut self, level: EncryptionLevel, keys: LevelKeys) {
        match level {
            EncryptionLevel::Initial => self.initial = Some(keys),
            EncryptionLevel::ZeroRtt => self.zero_rtt = Some(keys),
            EncryptionLevel::Handshake => self.handshake = Some(keys),
            EncryptionLevel::OneRtt => self.one_rtt = Some(keys),
        }
    }

    /// Discard keys for a given encryption level (RFC 9001 Section 4.9).
    ///
    /// Keys MUST be discarded when they are no longer needed:
    /// - Initial keys: discarded when Handshake keys are available
    /// - Handshake keys: discarded when the handshake is confirmed
    pub fn discard_keys(&mut self, level: EncryptionLevel) {
        match level {
            EncryptionLevel::Initial => self.initial = None,
            EncryptionLevel::ZeroRtt => self.zero_rtt = None,
            EncryptionLevel::Handshake => self.handshake = None,
            EncryptionLevel::OneRtt => self.one_rtt = None,
        }
    }

    /// Get keys for a given encryption level.
    #[must_use]
    pub fn get_keys(&self, level: EncryptionLevel) -> Option<&LevelKeys> {
        match level {
            EncryptionLevel::Initial => self.initial.as_ref(),
            EncryptionLevel::ZeroRtt => self.zero_rtt.as_ref(),
            EncryptionLevel::Handshake => self.handshake.as_ref(),
            EncryptionLevel::OneRtt => self.one_rtt.as_ref(),
        }
    }

    /// Check if keys are available for a given level.
    #[must_use]
    pub fn has_keys(&self, level: EncryptionLevel) -> bool {
        self.get_keys(level).is_some()
    }

    /// Current key phase for 1-RTT packets.
    #[must_use]
    pub const fn key_phase(&self) -> bool {
        self.key_phase
    }

    /// Toggle key phase (used during key updates).
    pub fn toggle_key_phase(&mut self) {
        self.key_phase = !self.key_phase;
    }

    /// The highest encryption level with installed keys.
    #[must_use]
    pub fn highest_level(&self) -> Option<EncryptionLevel> {
        if self.one_rtt.is_some() {
            Some(EncryptionLevel::OneRtt)
        } else if self.handshake.is_some() {
            Some(EncryptionLevel::Handshake)
        } else if self.zero_rtt.is_some() {
            Some(EncryptionLevel::ZeroRtt)
        } else if self.initial.is_some() {
            Some(EncryptionLevel::Initial)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keys() -> LevelKeys {
        LevelKeys {
            local: DirectionalKeys {
                key: vec![0u8; 16],
                iv: vec![0u8; 12],
                hp_key: vec![0u8; 16],
            },
            remote: DirectionalKeys {
                key: vec![1u8; 16],
                iv: vec![1u8; 12],
                hp_key: vec![1u8; 16],
            },
            suite: QuicCipherSuite::Aes128Gcm,
        }
    }

    #[test]
    fn new_session_empty() {
        let session = CryptoSession::new();
        assert!(!session.has_keys(EncryptionLevel::Initial));
        assert!(!session.has_keys(EncryptionLevel::Handshake));
        assert!(!session.has_keys(EncryptionLevel::OneRtt));
        assert_eq!(session.highest_level(), None);
    }

    #[test]
    fn install_and_get_keys() {
        let mut session = CryptoSession::new();
        session.install_keys(EncryptionLevel::Initial, test_keys());
        assert!(session.has_keys(EncryptionLevel::Initial));
        assert!(!session.has_keys(EncryptionLevel::Handshake));
        assert_eq!(session.highest_level(), Some(EncryptionLevel::Initial));
    }

    #[test]
    fn discard_keys() {
        let mut session = CryptoSession::new();
        session.install_keys(EncryptionLevel::Initial, test_keys());
        session.install_keys(EncryptionLevel::Handshake, test_keys());
        session.discard_keys(EncryptionLevel::Initial);
        assert!(!session.has_keys(EncryptionLevel::Initial));
        assert!(session.has_keys(EncryptionLevel::Handshake));
    }

    #[test]
    fn highest_level_progression() {
        let mut session = CryptoSession::new();
        session.install_keys(EncryptionLevel::Initial, test_keys());
        assert_eq!(session.highest_level(), Some(EncryptionLevel::Initial));

        session.install_keys(EncryptionLevel::Handshake, test_keys());
        assert_eq!(session.highest_level(), Some(EncryptionLevel::Handshake));

        session.install_keys(EncryptionLevel::OneRtt, test_keys());
        assert_eq!(session.highest_level(), Some(EncryptionLevel::OneRtt));
    }

    #[test]
    fn key_phase_toggle() {
        let mut session = CryptoSession::new();
        assert!(!session.key_phase());
        session.toggle_key_phase();
        assert!(session.key_phase());
        session.toggle_key_phase();
        assert!(!session.key_phase());
    }

    #[test]
    fn encryption_level_properties() {
        assert!(EncryptionLevel::Initial.is_initial());
        assert!(!EncryptionLevel::Initial.is_application());
        assert!(EncryptionLevel::OneRtt.is_application());
        assert!(!EncryptionLevel::Handshake.is_initial());
    }
}
