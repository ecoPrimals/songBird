// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` encryption provider — enum dispatch over concrete backends (no trait objects).

use super::mocks::{
    ChaoticBirdSongMock, CrossFamilyBirdSongMock, DarkForestTestProvider, FailingBirdSongMock,
    LegacyBirdSongStub, OrchestratorPrefixMock, ProcessorXorMock, ProtocolPassthroughMock,
    UnavailableBirdSongMock,
};
use crate::security_birdsong_provider::SecurityBirdSongProvider;
use anyhow::Result;
use std::sync::Arc;

/// Production and test `BirdSong` encryption backends.
#[derive(Clone)]
pub enum BirdSongEncryption {
    /// Live security-provider RPC adapter.
    Security(Arc<SecurityBirdSongProvider>),
    /// Unit-test XOR mock (`processor` tests).
    ProcessorXor(Arc<ProcessorXorMock>),
    /// Anonymous protocol tests.
    ProtocolPassthrough(Arc<ProtocolPassthroughMock>),
    /// Dark Forest integration harness.
    DarkForestTest(Arc<DarkForestTestProvider>),
    /// Legacy default-behavior stub.
    Legacy(LegacyBirdSongStub),
    /// Chaos / random failure mock.
    Chaotic(Arc<ChaoticBirdSongMock>),
    /// Fails after N operations.
    Failing(Arc<FailingBirdSongMock>),
    /// Always unavailable.
    Unavailable(Arc<UnavailableBirdSongMock>),
    /// Cross-family prefix semantics (fault-injection tests).
    CrossFamily(Arc<CrossFamilyBirdSongMock>),
    /// `ENCRYPTED:` prefix mock (orchestrator tests).
    OrchestratorPrefix(Arc<OrchestratorPrefixMock>),
}

impl BirdSongEncryption {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Security(p) => p.encrypt_discovery(plaintext).await,
            Self::ProcessorXor(p) => p.encrypt_discovery(plaintext).await,
            Self::ProtocolPassthrough(p) => p.encrypt_discovery(plaintext).await,
            Self::DarkForestTest(p) => p.encrypt_discovery(plaintext).await,
            Self::Legacy(p) => p.encrypt_discovery(plaintext).await,
            Self::Chaotic(p) => p.encrypt_discovery(plaintext).await,
            Self::Failing(p) => p.encrypt_discovery(plaintext).await,
            Self::Unavailable(p) => p.encrypt_discovery(plaintext).await,
            Self::CrossFamily(p) => p.encrypt_discovery(plaintext).await,
            Self::OrchestratorPrefix(p) => p.encrypt_discovery(plaintext).await,
        }
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        match self {
            Self::Security(p) => p.decrypt_discovery(ciphertext).await,
            Self::ProcessorXor(p) => p.decrypt_discovery(ciphertext).await,
            Self::ProtocolPassthrough(p) => p.decrypt_discovery(ciphertext).await,
            Self::DarkForestTest(p) => p.decrypt_discovery(ciphertext).await,
            Self::Legacy(p) => p.decrypt_discovery(ciphertext).await,
            Self::Chaotic(p) => p.decrypt_discovery(ciphertext).await,
            Self::Failing(p) => p.decrypt_discovery(ciphertext).await,
            Self::Unavailable(p) => p.decrypt_discovery(ciphertext).await,
            Self::CrossFamily(p) => p.decrypt_discovery(ciphertext).await,
            Self::OrchestratorPrefix(p) => p.decrypt_discovery(ciphertext).await,
        }
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        match self {
            Self::Security(p) => p.is_available(),
            Self::ProcessorXor(p) => p.is_available(),
            Self::ProtocolPassthrough(p) => p.is_available(),
            Self::DarkForestTest(p) => p.is_available(),
            Self::Legacy(p) => p.is_available(),
            Self::Chaotic(p) => p.is_available(),
            Self::Failing(p) => p.is_available(),
            Self::Unavailable(p) => p.is_available(),
            Self::CrossFamily(p) => p.is_available(),
            Self::OrchestratorPrefix(p) => p.is_available(),
        }
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        match self {
            Self::Security(p) => p.family_id(),
            Self::ProcessorXor(p) => p.family_id(),
            Self::ProtocolPassthrough(p) => p.family_id(),
            Self::DarkForestTest(p) => p.family_id(),
            Self::Legacy(p) => p.family_id(),
            Self::Chaotic(p) => p.family_id(),
            Self::Failing(p) => p.family_id(),
            Self::Unavailable(p) => p.family_id(),
            Self::CrossFamily(p) => p.family_id(),
            Self::OrchestratorPrefix(p) => p.family_id(),
        }
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        match self {
            Self::Security(p) => p.provider_name(),
            Self::ProcessorXor(p) => p.provider_name(),
            Self::ProtocolPassthrough(p) => p.provider_name(),
            Self::DarkForestTest(p) => p.provider_name(),
            Self::Legacy(p) => p.provider_name(),
            Self::Chaotic(p) => p.provider_name(),
            Self::Failing(p) => p.provider_name(),
            Self::Unavailable(p) => p.provider_name(),
            Self::CrossFamily(p) => p.provider_name(),
            Self::OrchestratorPrefix(p) => p.provider_name(),
        }
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        match self {
            Self::Security(p) => p.encrypt_beacon(payload).await,
            Self::ProcessorXor(p) => p.encrypt_beacon(payload).await,
            Self::ProtocolPassthrough(p) => p.encrypt_beacon(payload).await,
            Self::DarkForestTest(p) => p.encrypt_beacon(payload).await,
            Self::Legacy(p) => p.encrypt_beacon(payload).await,
            Self::Chaotic(p) => p.encrypt_beacon(payload).await,
            Self::Failing(p) => p.encrypt_beacon(payload).await,
            Self::Unavailable(p) => p.encrypt_beacon(payload).await,
            Self::CrossFamily(p) => p.encrypt_beacon(payload).await,
            Self::OrchestratorPrefix(p) => p.encrypt_beacon(payload).await,
        }
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        match self {
            Self::Security(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::ProcessorXor(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::ProtocolPassthrough(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::DarkForestTest(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::Legacy(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::Chaotic(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::Failing(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::Unavailable(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::CrossFamily(p) => p.try_decrypt_beacon(encrypted, nonce).await,
            Self::OrchestratorPrefix(p) => p.try_decrypt_beacon(encrypted, nonce).await,
        }
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        match self {
            Self::Security(p) => p.get_beacon_id().await,
            Self::ProcessorXor(p) => p.get_beacon_id().await,
            Self::ProtocolPassthrough(p) => p.get_beacon_id().await,
            Self::DarkForestTest(p) => p.get_beacon_id().await,
            Self::Legacy(p) => p.get_beacon_id().await,
            Self::Chaotic(p) => p.get_beacon_id().await,
            Self::Failing(p) => p.get_beacon_id().await,
            Self::Unavailable(p) => p.get_beacon_id().await,
            Self::CrossFamily(p) => p.get_beacon_id().await,
            Self::OrchestratorPrefix(p) => p.get_beacon_id().await,
        }
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        match self {
            Self::Security(p) => p.list_known_beacons().await,
            Self::ProcessorXor(p) => p.list_known_beacons().await,
            Self::ProtocolPassthrough(p) => p.list_known_beacons().await,
            Self::DarkForestTest(p) => p.list_known_beacons().await,
            Self::Legacy(p) => p.list_known_beacons().await,
            Self::Chaotic(p) => p.list_known_beacons().await,
            Self::Failing(p) => p.list_known_beacons().await,
            Self::Unavailable(p) => p.list_known_beacons().await,
            Self::CrossFamily(p) => p.list_known_beacons().await,
            Self::OrchestratorPrefix(p) => p.list_known_beacons().await,
        }
    }

    pub async fn supports_dark_forest(&self) -> bool {
        match self {
            Self::Security(p) => p.supports_dark_forest().await,
            Self::ProcessorXor(p) => p.supports_dark_forest().await,
            Self::ProtocolPassthrough(p) => p.supports_dark_forest().await,
            Self::DarkForestTest(p) => p.supports_dark_forest().await,
            Self::Legacy(p) => p.supports_dark_forest().await,
            Self::Chaotic(p) => p.supports_dark_forest().await,
            Self::Failing(p) => p.supports_dark_forest().await,
            Self::Unavailable(p) => p.supports_dark_forest().await,
            Self::CrossFamily(p) => p.supports_dark_forest().await,
            Self::OrchestratorPrefix(p) => p.supports_dark_forest().await,
        }
    }

    /// Toggle availability flag on [`super::mocks::ChaoticBirdSongMock`] only; no-op for other variants.
    pub fn chaotic_toggle_availability(&self) {
        if let Self::Chaotic(p) = self {
            p.toggle_availability();
        }
    }

    /// Reset chaotic encrypt/decrypt counter (see [`super::mocks::ChaoticBirdSongMock::reset_failure_counter`]).
    pub fn chaotic_reset_failure_counter(&self) {
        if let Self::Chaotic(p) = self {
            p.reset_failure_counter();
        }
    }
}
