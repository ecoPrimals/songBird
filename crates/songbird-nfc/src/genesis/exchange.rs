// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::config::NfcConfig;
use crate::error::{NfcError, Result};
use crate::platform::NfcDevice;
use crate::protocol::{NfcMessage, NfcProtocol};
use crate::timing::TimingProtector;
use crate::{MSG_TYPE_GENESIS_REQUEST, MSG_TYPE_GENESIS_RESPONSE, PUBLIC_KEY_SIZE};
use songbird_crypto_provider::CryptoProvider;
use tracing::{debug, info};

use super::types::GenesisCredentials;

/// Genesis exchange protocol
#[derive(Debug)]
pub struct GenesisExchange {
    /// Configuration
    pub(super) config: NfcConfig,

    /// Protocol handler
    #[expect(dead_code, reason = "reserved for future NFC frame operations")]
    pub(super) protocol: NfcProtocol,

    /// Timing protector
    pub(super) timing: TimingProtector,

    /// Shared crypto provider (Neural API by default; `SECURITY_PROVIDER_MODE=direct` or legacy `BEARDOG_MODE=direct` for direct socket routing)
    pub(super) provider: CryptoProvider,
}

impl GenesisExchange {
    /// Create new genesis exchange
    #[must_use]
    pub fn new(config: NfcConfig) -> Self {
        let timing = TimingProtector::new(config.target_exchange_duration, config.max_random_delay);

        let protocol = NfcProtocol::new(config.clone());
        let provider = CryptoProvider::from_env();

        Self {
            config,
            protocol,
            timing,
            provider,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(config: NfcConfig, provider: CryptoProvider) -> Self {
        let timing = TimingProtector::new(config.target_exchange_duration, config.max_random_delay);
        let protocol = NfcProtocol::new(config.clone());
        Self {
            config,
            protocol,
            timing,
            provider,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_with_provider(provider: CryptoProvider) -> Self {
        Self::for_test(NfcConfig::default(), provider)
    }

    /// Initiate genesis exchange (as parent/initiator)
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the exchange fails.
    ///
    /// Steps:
    /// 1. Generate ephemeral X25519 keypair
    /// 2. Send public key to peer
    /// 3. Receive peer's public key
    /// 4. Compute shared secret (X25519 DH)
    /// 5. Encrypt genesis credentials
    /// 6. Send encrypted genesis
    /// 7. Receive confirmation
    /// 8. Destroy ephemeral keys
    pub async fn initiate(
        &mut self,
        device: &mut NfcDevice,
        credentials: &GenesisCredentials,
    ) -> Result<()> {
        info!("🔐 Initiating genesis exchange");

        if self.config.timing_protection {
            self.timing.start();
            self.timing.random_delay().await;
        }

        let ephemeral_pubkey = self.generate_x25519_keypair().await?;

        debug!("Sending ephemeral public key");
        device.send_raw(&ephemeral_pubkey).await?;

        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        let shared_secret = self.x25519_dh(&peer_pubkey).await?;

        let nonce = self.generate_nonce().await?;
        let serialized = serde_json::to_vec(credentials)?;
        let encrypted = self.encrypt(&serialized, &shared_secret, &nonce).await?;

        let signature = self.ed25519_sign(&encrypted).await?;

        let message = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            ephemeral_pubkey,
            nonce,
            encrypted,
            signature,
        );

        device.send_message(&message).await?;

        let response = device.receive_message().await?;

        if response.msg_type != MSG_TYPE_GENESIS_RESPONSE {
            return Err(NfcError::InvalidMessageType(response.msg_type));
        }

        info!("Genesis exchange complete");

        self.destroy_ephemeral_keys().await?;

        if self.config.timing_protection {
            self.timing.pad_to_constant_time().await?;
        }

        Ok(())
    }

    /// Respond to genesis exchange (as child/responder)
    ///
    /// # Errors
    ///
    /// Returns an error if the exchange fails or credentials cannot be decrypted.
    pub async fn respond(&mut self, device: &mut NfcDevice) -> Result<GenesisCredentials> {
        info!("🔓 Responding to genesis exchange");

        if self.config.timing_protection {
            self.timing.start();
            self.timing.random_delay().await;
        }

        let ephemeral_pubkey = self.generate_x25519_keypair().await?;

        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        device.send_raw(&ephemeral_pubkey).await?;
        debug!("Sent ephemeral public key");

        let shared_secret = self.x25519_dh(&peer_pubkey).await?;

        let message = device.receive_message().await?;

        if message.msg_type != MSG_TYPE_GENESIS_REQUEST {
            return Err(NfcError::InvalidMessageType(message.msg_type));
        }

        self.ed25519_verify(&message.encrypted_payload, &message.signature).await?;

        let decrypted =
            self.decrypt(&message.encrypted_payload, &shared_secret, &message.nonce).await?;
        let credentials: GenesisCredentials = serde_json::from_slice(&decrypted)?;

        let conf_nonce = self.generate_nonce().await?;
        let conf_payload = vec![0u8; 16];
        let conf_signature = self.ed25519_sign(&conf_payload).await?;

        let confirmation = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            ephemeral_pubkey,
            conf_nonce,
            conf_payload,
            conf_signature,
        );

        device.send_message(&confirmation).await?;

        info!("Genesis received");

        self.destroy_ephemeral_keys().await?;

        if self.config.timing_protection {
            self.timing.pad_to_constant_time().await?;
        }

        Ok(credentials)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::platform::test_support::ScriptedBackend;
    use crate::{
        MSG_TYPE_GENESIS_REQUEST, MSG_TYPE_GENESIS_RESPONSE, NONCE_SIZE, PUBLIC_KEY_SIZE,
        SIGNATURE_SIZE,
    };
    use songbird_crypto_provider::RoutingMode;

    use super::super::types::GenesisCredentials;

    fn sample_credentials() -> GenesisCredentials {
        GenesisCredentials {
            identity: vec![1, 2, 3],
            family_seed: vec![4, 5, 6],
            lineage: vec!["root".to_string()],
            beacons: vec!["b.onion".to_string()],
            timestamp: 1_707_350_400_000,
        }
    }

    fn offline_provider() -> CryptoProvider {
        CryptoProvider::with_mode("/tmp/nonexistent-security-provider.sock", RoutingMode::Direct)
    }

    #[tokio::test]
    async fn initiate_rejects_non_response_message_type() {
        let peer_pk = [0xabu8; PUBLIC_KEY_SIZE];
        let wrong = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; 16],
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(&wrong.to_bytes().expect("wrong response frame should serialize"));

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(
            NfcConfig::default().with_timing_protection(false),
            offline_provider(),
        );

        let err = ex
            .initiate(&mut device, &sample_credentials())
            .await
            .expect_err("initiator should reject a frame that is not genesis response");
        assert!(
            matches!(err, crate::NfcError::InvalidMessageType(MSG_TYPE_GENESIS_REQUEST)),
            "expected InvalidMessageType for genesis request in response slot, got {err:?}"
        );
    }

    #[tokio::test]
    async fn initiate_completes_when_peer_sends_valid_response() {
        let peer_pk = [0x11u8; PUBLIC_KEY_SIZE];
        let ok_response = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; 16],
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(
            &ok_response.to_bytes().expect("ok response frame should serialize"),
        );

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(
            NfcConfig::default().with_timing_protection(false),
            offline_provider(),
        );

        ex.initiate(&mut device, &sample_credentials())
            .await
            .expect("initiate should succeed with matching response type and offline crypto");
    }

    #[tokio::test]
    async fn respond_rejects_non_request_message_type() {
        let peer_pk = [0xccu8; PUBLIC_KEY_SIZE];
        let wrong = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; 8],
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(&wrong.to_bytes().expect("wrong request frame should serialize"));

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(
            NfcConfig::default().with_timing_protection(false),
            offline_provider(),
        );

        let err = ex
            .respond(&mut device)
            .await
            .expect_err("responder should reject genesis response where request expected");
        assert!(
            matches!(err, crate::NfcError::InvalidMessageType(MSG_TYPE_GENESIS_RESPONSE)),
            "expected InvalidMessageType for response-as-request, got {err:?}"
        );
    }

    #[tokio::test]
    async fn respond_roundtrips_credentials_with_offline_crypto() {
        let creds = sample_credentials();
        let peer_pk = [0xddu8; PUBLIC_KEY_SIZE];
        let payload = serde_json::to_vec(&creds).expect("credentials serialize");
        let request = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0xfeu8; PUBLIC_KEY_SIZE],
            [0xeeu8; NONCE_SIZE],
            payload,
            [0x5u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(
            &request.to_bytes().expect("genesis request frame should serialize"),
        );

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(
            NfcConfig::default().with_timing_protection(false),
            offline_provider(),
        );

        let got = ex
            .respond(&mut device)
            .await
            .expect("respond should decrypt credentials with plaintext crypto fallback");
        assert_eq!(got.identity, creds.identity, "identity should round-trip");
        assert_eq!(got.family_seed, creds.family_seed, "family seed should round-trip");
        assert_eq!(got.lineage, creds.lineage, "lineage should round-trip");
        assert_eq!(got.beacons, creds.beacons, "beacons should round-trip");
        assert_eq!(got.timestamp, creds.timestamp, "timestamp should round-trip");
    }

    #[tokio::test]
    async fn respond_rejects_invalid_json_payload() {
        let peer_pk = [0xeeu8; PUBLIC_KEY_SIZE];
        let request = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            b"not-json".to_vec(),
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(
            &request.to_bytes().expect("genesis request frame should serialize"),
        );

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(
            NfcConfig::default().with_timing_protection(false),
            offline_provider(),
        );

        let err = ex
            .respond(&mut device)
            .await
            .expect_err("invalid JSON in payload should surface as serialization error");
        assert!(
            matches!(err, crate::NfcError::Serialization(_)),
            "expected Serialization error, got {err:?}"
        );
    }
}
