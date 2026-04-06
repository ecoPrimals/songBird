// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `SoloKey` / FIDO2 hardware channel — **integration point, not a live CTAP2 implementation**.
//!
//! ## Status
//!
//! The Cargo feature `solokey` exposes [`SoloKeyChannel`] so callers can type-check and route
//! genesis flows toward a hardware-backed path. **There is no USB HID / CTAP2 client in this
//! crate yet**; [`PhysicalChannelProvider::verify_proximity`] and [`PhysicalChannelProvider::secure_exchange`]
//! return [`crate::error::GenesisError::SoloKeyNotIntegrated`] until a real stack is wired in.
//!
//! ## wateringHole / capability alignment
//!
//! - **Runtime crypto** for Songbird is delegated to the **security capability** (discoverable
//!   primal / `SECURITY_PROVIDER_SOCKET`, `BEARDOG_SOCKET`, etc.); see
//!   `songbird-network-federation::security::SecurityProviderFactory` and wateringHole v1.2
//!   capability sockets under `$XDG_RUNTIME_DIR/biomeos/`.
//! - **Local FIDO2** (this module) is orthogonal: it would talk to a token over CTAP2 for
//!   proximity and credential material *before* or *alongside* that delegation, not as a
//!   substitute for the security primal.
//!
//! ## Integration path (future work)
//!
//! 1. **Pure Rust CTAP2** over USB HID (or platform HID abstractions), or a thin FFI to an
//!    audited library, behind a dedicated feature if dependencies are heavy.
//! 2. **Delegate to security primal**: IPC to the capability-discovered security endpoint for
//!    operations that must stay centralized (policy, attestation verification, key handles).
//! 3. Map CTAP2 assertion / credential outputs into [`crate::types::ProximityProof`] and
//!    genesis credential blobs — **never** return static demo bytes in production paths.
//!
//! Until then, [`SoloKeyChannel`] reports [`TrustLevel::Low`] for metadata (the channel kind is
//! still [`PhysicalChannelType::HardwareKey`]) because no hardware-backed attestation is performed.

use crate::error::{GenesisError, Result};
use crate::types::{PhysicalChannelType, ProximityProof, TrustLevel};

use super::PhysicalChannelProvider;

const NOT_INTEGRATED_MSG: &str = "CTAP2/FIDO2 USB HID not integrated; enable a future \
    `solokey-ctap` (or similar) feature and wire CTAP2 or security-provider delegation. \
    See module documentation in physical_channels::solokey.";

/// Hardware security key channel (`SoloKey`, `YubiKey`, etc.) — type shell for the FIDO2 path.
///
/// Construct with [`SoloKeyChannel::new`] when the `solokey` Cargo feature is enabled.
/// Cryptographic operations are not implemented in-tree yet; call sites must handle
/// [`GenesisError::SoloKeyNotIntegrated`].
#[derive(Debug)]
pub struct SoloKeyChannel;

impl SoloKeyChannel {
    /// Create a new hardware-key channel handle (integration pending).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for SoloKeyChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalChannelProvider for SoloKeyChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        Err(GenesisError::SoloKeyNotIntegrated(NOT_INTEGRATED_MSG.into()))
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        Err(GenesisError::SoloKeyNotIntegrated(NOT_INTEGRATED_MSG.into()))
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Low
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::HardwareKey
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::SoloKeyChannel;
    use crate::error::GenesisError;
    use crate::physical_channels::PhysicalChannelProvider;
    use crate::types::{PhysicalChannelType, TrustLevel};

    #[test]
    fn solokey_channel_metadata_honest_until_ctap2() {
        let ch = SoloKeyChannel::new();
        assert_eq!(ch.channel_type(), PhysicalChannelType::HardwareKey);
        assert_eq!(ch.trust_level(), TrustLevel::Low);
    }

    #[tokio::test]
    async fn proximity_and_exchange_return_not_integrated() {
        let ch = SoloKeyChannel::new();
        let prox = ch.verify_proximity().await;
        let ex = ch.secure_exchange().await;
        assert!(matches!(prox, Err(GenesisError::SoloKeyNotIntegrated(_))));
        assert!(matches!(ex, Err(GenesisError::SoloKeyNotIntegrated(_))));
    }
}
