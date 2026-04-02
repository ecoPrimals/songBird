// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::sync::Arc;

use songbird_crypto_provider::CryptoProvider;

/// TLS Record Layer
///
/// Handles framing, encryption, and decryption of TLS records.
pub struct RecordLayer {
    pub(crate) crypto_provider: Option<Arc<CryptoProvider>>,

    /// Sequence number for outgoing records (for nonce construction)
    pub(crate) write_sequence: u64,

    /// Sequence number for incoming records (for nonce construction)
    pub(crate) read_sequence: u64,

    /// Are we in encrypted mode? (after handshake)
    pub(crate) encrypted: bool,
}

impl RecordLayer {
    /// Create a new `RecordLayer` in plaintext mode
    #[must_use]
    pub const fn new() -> Self {
        Self {
            crypto_provider: None,
            write_sequence: 0,
            read_sequence: 0,
            encrypted: false,
        }
    }

    /// Construct a record layer that routes AEAD to [`CryptoProvider`] (BearDog / Neural API).
    #[must_use]
    pub fn with_crypto_provider(crypto_provider: Arc<CryptoProvider>) -> Self {
        Self {
            crypto_provider: Some(crypto_provider),
            write_sequence: 0,
            read_sequence: 0,
            encrypted: false,
        }
    }

    /// Enable encryption (called after handshake completion)
    pub const fn enable_encryption(&mut self) {
        self.encrypted = true;
        // Note: Sequence numbers are NOT reset when enabling encryption
        // They continue from handshake phase
    }

    /// Get the current write sequence number
    #[must_use]
    pub const fn write_sequence(&self) -> u64 {
        self.write_sequence
    }

    /// Get the current read sequence number
    #[must_use]
    pub const fn read_sequence(&self) -> u64 {
        self.read_sequence
    }

    /// Increment write sequence number
    pub(crate) const fn increment_write_sequence(&mut self) {
        self.write_sequence = self.write_sequence.wrapping_add(1);
    }

    /// Increment read sequence number
    pub(crate) const fn increment_read_sequence(&mut self) {
        self.read_sequence = self.read_sequence.wrapping_add(1);
    }
}

impl Default for RecordLayer {
    fn default() -> Self {
        Self::new()
    }
}
