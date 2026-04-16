// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Concrete test doubles for [`super::BirdSongEncryption`] (enum dispatch; no trait objects).

use anyhow::Result;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// XOR-based mock used by `BirdSongProcessor` unit tests in `processor.rs`.
pub struct ProcessorXorMock {
    pub family_id: String,
    pub available: bool,
}

impl ProcessorXorMock {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.iter().map(|b| b ^ 0x42).collect())
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        if ciphertext.first() == Some(&0xFF) {
            Ok(None)
        } else {
            Ok(Some(ciphertext.iter().map(|b| b ^ 0x42).collect()))
        }
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        self.available
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        Some(self.family_id.clone())
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "MockEncryption".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Minimal mock for `anonymous/protocol` tests.
pub struct ProtocolPassthroughMock {
    pub family_id: String,
}

impl ProtocolPassthroughMock {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec())
    }

    pub async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        Some(self.family_id.clone())
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "mock-protocol-test".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Dark Forest beacon test double (XOR “encryption”) for integration tests.
pub struct DarkForestTestProvider {
    beacon_id: Vec<u8>,
    beacon_seed: [u8; 32],
    known_beacons: Vec<Vec<u8>>,
    available: bool,
}

impl DarkForestTestProvider {
    #[must_use]
    pub fn new(seed: [u8; 32]) -> Self {
        let beacon_id = Self::derive_beacon_id(&seed);
        Self {
            beacon_id,
            beacon_seed: seed,
            known_beacons: Vec::new(),
            available: true,
        }
    }

    #[must_use]
    pub fn with_known_beacons(mut self, beacons: Vec<Vec<u8>>) -> Self {
        self.known_beacons = beacons;
        self
    }

    /// Same derivation as production beacon id (tests / harness).
    #[must_use]
    pub fn derive_beacon_id(seed: &[u8; 32]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(seed);
        hasher.update(b"beacon-id-v1");
        let hash = hasher.finalize();
        hash.as_bytes()[..16].to_vec()
    }

    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec())
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(Some(ciphertext.to_vec()))
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        self.available
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        Some("test-family".to_string())
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "MockDarkForestProvider".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted: Vec<u8> =
            payload.iter().enumerate().map(|(i, &b)| b ^ self.beacon_seed[i % 32]).collect();
        let nonce = [0u8; 12];
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        let decrypted: Vec<u8> =
            encrypted.iter().enumerate().map(|(i, &b)| b ^ self.beacon_seed[i % 32]).collect();
        Ok(Some(decrypted))
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(Some(self.beacon_id.clone()))
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(self.known_beacons.clone())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        true
    }
}

/// Legacy-only stub: matches former default `BirdSongEncryption` trait behavior.
#[derive(Clone, Copy, Debug, Default)]
pub struct LegacyBirdSongStub;

impl LegacyBirdSongStub {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec())
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(Some(ciphertext.to_vec()))
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        Some("legacy".to_string())
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "LegacyBirdSongStub".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Randomized failure injection for chaos tests.
pub struct ChaoticBirdSongMock {
    family_id: Option<String>,
    failure_rate: f64,
    call_count: AtomicUsize,
    is_available: AtomicBool,
}

impl ChaoticBirdSongMock {
    #[must_use]
    pub fn new(family_id: Option<String>, failure_rate: f64) -> Self {
        Self {
            family_id,
            failure_rate,
            call_count: AtomicUsize::new(0),
            is_available: AtomicBool::new(true),
        }
    }

    fn should_fail(&self) -> bool {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        (count % 100) < (self.failure_rate * 100.0) as usize
    }

    pub fn toggle_availability(&self) {
        let current = self.is_available.load(Ordering::SeqCst);
        self.is_available.store(!current, Ordering::SeqCst);
    }

    /// Reset encrypt/decrypt attempt counter (chaos harness).
    pub fn reset_failure_counter(&self) {
        self.call_count.store(0, Ordering::SeqCst);
    }

    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if self.should_fail() {
            return Err(anyhow::anyhow!("Chaotic failure: encryption"));
        }
        Ok(plaintext.iter().map(|&b| b.wrapping_add(1)).collect())
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        if self.should_fail() {
            return Err(anyhow::anyhow!("Chaotic failure: decryption"));
        }
        Ok(Some(ciphertext.iter().map(|&b| b.wrapping_sub(1)).collect()))
    }

    pub fn is_available(&self) -> bool {
        self.is_available.load(Ordering::SeqCst)
    }

    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    pub fn provider_name(&self) -> String {
        "ChaoticProvider".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Fails encrypt/decrypt after N operations.
pub struct FailingBirdSongMock {
    pub family_id: Option<String>,
    call_count: AtomicUsize,
    fail_after: usize,
}

impl FailingBirdSongMock {
    #[must_use]
    pub fn new(family_id: Option<String>, fail_after: usize) -> Self {
        Self {
            family_id,
            call_count: AtomicUsize::new(0),
            fail_after,
        }
    }

    pub async fn encrypt_discovery(&self, _plaintext: &[u8]) -> Result<Vec<u8>> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.fail_after {
            Err(anyhow::anyhow!("Simulated encryption failure after {count} calls"))
        } else {
            Ok(b"ENCRYPTED".to_vec())
        }
    }

    pub async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.fail_after {
            Err(anyhow::anyhow!("Simulated decryption failure after {count} calls"))
        } else {
            Ok(Some(b"DECRYPTED".to_vec()))
        }
    }

    pub fn is_available(&self) -> bool {
        self.call_count.load(Ordering::SeqCst) < self.fail_after
    }

    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    pub fn provider_name(&self) -> String {
        "FailingProvider".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Always-unavailable provider for fault-injection tests.
pub struct UnavailableBirdSongMock {
    pub family_id: Option<String>,
}

impl UnavailableBirdSongMock {
    pub async fn encrypt_discovery(&self, _plaintext: &[u8]) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    pub async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        false
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "UnavailableProvider".to_string()
    }

    pub async fn encrypt_beacon(&self, _payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    pub async fn try_decrypt_beacon(
        &self,
        _encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Err(anyhow::anyhow!("Provider unavailable"))
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// Cross-family prefix mock (fault-injection integration tests).
pub struct CrossFamilyBirdSongMock {
    pub family_id: Option<String>,
}

impl CrossFamilyBirdSongMock {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted = self.family_id.clone().unwrap_or_default().into_bytes();
        encrypted.push(b':');
        encrypted.extend_from_slice(plaintext);
        Ok(encrypted)
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        let family_bytes = self.family_id.clone().unwrap_or_default().into_bytes();
        if ciphertext.starts_with(&family_bytes) && ciphertext.len() > family_bytes.len() + 1 {
            Ok(Some(ciphertext[family_bytes.len() + 1..].to_vec()))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "MockProvider".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}

/// `ENCRYPTED:` prefix mock for orchestrator integration tests.
pub struct OrchestratorPrefixMock {
    pub family_id: Option<String>,
}

impl OrchestratorPrefixMock {
    pub async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted = b"ENCRYPTED:".to_vec();
        encrypted.extend_from_slice(plaintext);
        Ok(encrypted)
    }

    pub async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
        if ciphertext.starts_with(b"ENCRYPTED:") {
            Ok(Some(ciphertext[10..].to_vec()))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }

    #[must_use]
    pub fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    #[must_use]
    pub fn provider_name(&self) -> String {
        "OrchestratorPrefixMock".to_string()
    }

    pub async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let encrypted = self.encrypt_discovery(payload).await?;
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).map_err(|e| anyhow::anyhow!("nonce: {e}"))?;
        Ok((encrypted, nonce))
    }

    pub async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        self.decrypt_discovery(encrypted).await
    }

    pub async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    pub async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        Ok(Vec::new())
    }

    pub async fn supports_dark_forest(&self) -> bool {
        false
    }
}
