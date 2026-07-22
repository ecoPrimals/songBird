// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider `BirdSong` integration - Production & Test Implementations
//!
//! Production implementation connects to the discovered security provider via Unix socket JSON-RPC.
//! Test mocks allow testing lineage relay without a live provider.
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Modern async Rust (enum dispatch, async/await)
//! - ✅ Zero unsafe code
//! - ✅ Runtime discovery (no hardcoded paths)
//! - ✅ Mocks isolated to `#[cfg(any(test, feature = "test-mocks"))]`
//! - ✅ Pure Rust (Unix sockets, not HTTP)

mod birdsong_provider;
mod relay_authority;

#[cfg(any(test, feature = "test-mocks"))]
mod mock;

pub use birdsong_provider::SecurityBirdSongProvider;
pub use relay_authority::SecurityRelayAuthority;

#[cfg(any(test, feature = "test-mocks"))]
pub use mock::{MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority};

use crate::error::Result;
use crate::types::{LineageHint, NodeId};

/// Lineage `BirdSong` encryption dispatch (production + test harnesses).
#[derive(Clone, Debug)]
pub enum BirdSongCrypto {
    /// Production client via the security provider (Unix socket JSON-RPC).
    Security(SecurityBirdSongProvider),
    /// Mock keyed by lineage graph (`test-utils` / unit tests).
    #[cfg(any(test, feature = "test-mocks"))]
    Mock(MockBirdSongCrypto),
    /// Pass-through: no crypto transform (unit / integration harnesses).
    #[cfg(any(test, feature = "test-mocks"))]
    StubPassthrough,
    /// Prepends `ENCRYPTED:` for unit tests (paired stub strips prefix on decrypt).
    #[cfg(any(test, feature = "test-mocks"))]
    StubMockEncrypted,
}

impl BirdSongCrypto {
    /// Encrypt message for lineage.
    pub async fn encrypt_for_lineage(&self, message: &[u8], hint: LineageHint) -> Result<Vec<u8>> {
        match self {
            Self::Security(p) => p.encrypt_for_lineage(message, hint).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(m) => m.encrypt_for_lineage(message, hint).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::StubPassthrough => Ok(message.to_vec()),
            #[cfg(any(test, feature = "test-mocks"))]
            Self::StubMockEncrypted => {
                let mut encrypted = b"ENCRYPTED:".to_vec();
                encrypted.extend_from_slice(message);
                Ok(encrypted)
            }
        }
    }

    /// Decrypt `BirdSong` payload (returns `None` if not in lineage / noise).
    pub async fn decrypt_birdsong(
        &self,
        encrypted: &[u8],
        sender: &NodeId,
    ) -> Result<Option<Vec<u8>>> {
        match self {
            Self::Security(p) => p.decrypt_birdsong(encrypted, sender).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(m) => m.decrypt_birdsong(encrypted, sender).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::StubPassthrough => Ok(Some(encrypted.to_vec())),
            #[cfg(any(test, feature = "test-mocks"))]
            Self::StubMockEncrypted => {
                if encrypted.starts_with(b"ENCRYPTED:") {
                    Ok(Some(encrypted[10..].to_vec()))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl From<SecurityBirdSongProvider> for BirdSongCrypto {
    fn from(value: SecurityBirdSongProvider) -> Self {
        Self::Security(value)
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl From<MockBirdSongCrypto> for BirdSongCrypto {
    fn from(value: MockBirdSongCrypto) -> Self {
        Self::Mock(value)
    }
}
