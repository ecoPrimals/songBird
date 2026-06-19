// SPDX-License-Identifier: AGPL-3.0-or-later
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
    #[allow(dead_code, reason = "reserved for future NFC frame operations")]
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    fn fast_timing_config(enabled: bool) -> NfcConfig {
        let mut config = NfcConfig::default().with_timing_protection(enabled);
        config.target_exchange_duration = std::time::Duration::from_millis(1);
        config.max_random_delay = std::time::Duration::from_millis(1);
        config
    }

    fn unique_socket_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "songbird_nfc_test_{}.sock",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock before epoch")
                .as_nanos()
        ))
    }

    fn mock_crypto_result(method: &str, verify_valid: bool) -> serde_json::Value {
        if method.contains("ed25519_verify") {
            return serde_json::json!({ "valid": verify_valid });
        }
        if method.contains("generate_x25519_keypair") {
            return serde_json::json!({ "public_key": "11".repeat(PUBLIC_KEY_SIZE) });
        }
        if method.contains("x25519_dh") {
            return serde_json::json!({ "shared_secret": "00".repeat(32) });
        }
        if method.contains("generate_random") {
            return serde_json::json!({ "bytes": "aa".repeat(NONCE_SIZE) });
        }
        if method.contains("chacha20poly1305_encrypt")
            || method.contains("chacha20_poly1305_encrypt")
        {
            return serde_json::json!({ "ciphertext": "00" });
        }
        if method.contains("chacha20poly1305_decrypt")
            || method.contains("chacha20_poly1305_decrypt")
        {
            return serde_json::json!({ "plaintext": "00" });
        }
        if method.contains("ed25519_sign") {
            return serde_json::json!({ "signature": "bb".repeat(SIGNATURE_SIZE) });
        }
        if method.contains("destroy_ephemeral_keys") {
            return serde_json::json!({ "destroyed": true });
        }
        serde_json::json!({})
    }

    async fn spawn_crypto_rpc_server(
        listener: tokio::net::UnixListener,
        verify_valid: bool,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = Vec::new();
                let mut tmp = [0u8; 4096];
                loop {
                    let Ok(n) = stream.read(&mut tmp).await else {
                        break;
                    };
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&tmp[..n]);
                }
                let request: serde_json::Value =
                    serde_json::from_slice(&buf).unwrap_or(serde_json::json!({}));
                let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
                let id = request.get("id").and_then(serde_json::Value::as_u64).unwrap_or(1);
                let result = mock_crypto_result(method, verify_valid);
                let body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                });
                let out = serde_json::to_vec(&body).expect("serialize mock rpc response");
                let _ = stream.write_all(&out).await;
                let _ = stream.shutdown().await;
            }
        })
    }

    fn build_responder_recv(initiator_frames: &[Vec<u8>]) -> Vec<u8> {
        let mut recv = initiator_frames[0].clone();
        recv.extend_from_slice(&initiator_frames[1]);
        recv
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

    #[test]
    fn new_constructs_with_default_config_fields() {
        let config = NfcConfig::default();
        let ex = GenesisExchange::new(config.clone());
        assert_eq!(ex.config.exchange_timeout, config.exchange_timeout);
        assert_eq!(ex.config.timing_protection, config.timing_protection);
        assert_eq!(
            ex.config.target_exchange_duration, config.target_exchange_duration,
            "new() should wire target exchange duration from config"
        );
        assert!(ex.config.validate_connection);
    }

    #[test]
    fn new_constructs_with_timing_protection_disabled() {
        let config = NfcConfig::default().with_timing_protection(false);
        let ex = GenesisExchange::new(config);
        assert!(!ex.config.timing_protection);
    }

    #[test]
    fn new_constructs_with_custom_timeout() {
        let config = NfcConfig::default()
            .with_timeout(std::time::Duration::from_secs(7))
            .with_timing_protection(true);
        let ex = GenesisExchange::new(config);
        assert_eq!(ex.config.exchange_timeout, std::time::Duration::from_secs(7));
        assert!(ex.config.timing_protection);
    }

    #[test]
    fn for_test_matches_new_protocol_and_timing_wiring() {
        let config = fast_timing_config(true);
        let via_new = GenesisExchange::new(config.clone());
        let via_for_test = GenesisExchange::for_test(config, offline_provider());
        assert_eq!(via_new.config.timing_protection, via_for_test.config.timing_protection);
        assert_eq!(
            via_new.config.target_exchange_duration,
            via_for_test.config.target_exchange_duration
        );
        assert_eq!(
            via_new.protocol.security_provider_socket(),
            via_for_test.protocol.security_provider_socket()
        );
    }

    #[tokio::test]
    async fn full_initiator_responder_flow_via_scripted_backends() {
        let creds = sample_credentials();
        let config = fast_timing_config(false);
        let placeholder_pk = [0x22u8; PUBLIC_KEY_SIZE];

        let backend = ScriptedBackend::new(placeholder_pk.to_vec());
        let initiator_partial_sent = backend.sent_frames_handle();
        let mut partial_device = backend.into_device(std::time::Duration::from_secs(5));
        let mut partial_ex = GenesisExchange::for_test(config.clone(), offline_provider());

        partial_ex
            .initiate(&mut partial_device, &creds)
            .await
            .expect_err("partial initiator should stop once response frame is missing");
        let partial_frames = initiator_partial_sent.lock().expect("partial sent lock");
        assert_eq!(partial_frames.len(), 2, "initiator should emit pubkey then genesis request");
        let responder_recv = build_responder_recv(&partial_frames);
        drop(partial_frames);

        let responder_backend = ScriptedBackend::new(responder_recv);
        let responder_sent = responder_backend.sent_frames_handle();
        let mut responder_device = responder_backend.into_device(std::time::Duration::from_secs(5));
        let mut responder_ex = GenesisExchange::for_test(config.clone(), offline_provider());
        let got = responder_ex
            .respond(&mut responder_device)
            .await
            .expect("responder should decrypt credentials from scripted initiator frames");
        assert_eq!(got.identity, creds.identity);
        assert_eq!(got.family_seed, creds.family_seed);

        let response_frames = responder_sent.lock().expect("responder sent lock");
        assert_eq!(response_frames.len(), 2, "responder should emit pubkey then confirmation");
        let mut initiator_final_recv = response_frames[0].clone();
        initiator_final_recv.extend_from_slice(&response_frames[1]);
        drop(response_frames);

        let mut final_device = ScriptedBackend::new(initiator_final_recv)
            .into_device(std::time::Duration::from_secs(5));
        let mut final_ex = GenesisExchange::for_test(config, offline_provider());
        final_ex
            .initiate(&mut final_device, &creds)
            .await
            .expect("initiator should finish once responder frames are scripted");
    }

    #[tokio::test]
    async fn initiate_with_timing_protection_completes() {
        let peer_pk = [0x33u8; PUBLIC_KEY_SIZE];
        let ok_response = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; 16],
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(&ok_response.to_bytes().expect("response frame"));

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(fast_timing_config(true), offline_provider());

        ex.initiate(&mut device, &sample_credentials())
            .await
            .expect("timing-protected initiate should succeed with fast timing config");
    }

    #[tokio::test]
    async fn respond_with_timing_protection_completes() {
        let creds = sample_credentials();
        let peer_pk = [0x44u8; PUBLIC_KEY_SIZE];
        let request = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            serde_json::to_vec(&creds).expect("serialize credentials"),
            [0u8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(&request.to_bytes().expect("request frame"));

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(fast_timing_config(true), offline_provider());

        let got = ex
            .respond(&mut device)
            .await
            .expect("timing-protected respond should succeed with fast timing config");
        assert_eq!(got.timestamp, creds.timestamp);
    }

    #[tokio::test]
    async fn initiate_fails_when_peer_pubkey_missing() {
        let mut device =
            ScriptedBackend::new(Vec::new()).into_device(std::time::Duration::from_secs(1));
        let mut ex = GenesisExchange::for_test(fast_timing_config(false), offline_provider());

        let err = ex
            .initiate(&mut device, &sample_credentials())
            .await
            .expect_err("initiator should fail when peer pubkey never arrives");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost waiting for peer pubkey, got {err:?}"
        );
    }

    #[tokio::test]
    async fn respond_fails_when_peer_pubkey_missing() {
        let mut device =
            ScriptedBackend::new(Vec::new()).into_device(std::time::Duration::from_secs(1));
        let mut ex = GenesisExchange::for_test(fast_timing_config(false), offline_provider());

        let err =
            ex.respond(&mut device).await.expect_err("responder should fail without peer pubkey");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost waiting for peer pubkey, got {err:?}"
        );
    }

    #[tokio::test]
    async fn respond_rejects_failed_signature_from_crypto_provider() {
        let sock = unique_socket_path();
        let _ = std::fs::remove_file(&sock);
        let listener = tokio::net::UnixListener::bind(&sock).expect("bind mock crypto socket");
        let server = spawn_crypto_rpc_server(listener, false).await;
        let provider = CryptoProvider::with_mode(
            sock.to_str().expect("temp socket path should be utf-8"),
            RoutingMode::Direct,
        );

        let creds = sample_credentials();
        let peer_pk = [0x55u8; PUBLIC_KEY_SIZE];
        let request = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            serde_json::to_vec(&creds).expect("serialize credentials"),
            [0xabu8; SIGNATURE_SIZE],
        );
        let mut recv = Vec::new();
        recv.extend_from_slice(&peer_pk);
        recv.extend_from_slice(&request.to_bytes().expect("request frame"));

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(5));
        let mut ex = GenesisExchange::for_test(fast_timing_config(false), provider);

        let err = ex
            .respond(&mut device)
            .await
            .expect_err("responder should reject signatures the provider marks invalid");
        assert!(
            matches!(err, crate::NfcError::Crypto(ref msg) if msg.contains("Signature verification failed")),
            "expected signature verification crypto error, got {err:?}"
        );

        server.abort();
        let _ = std::fs::remove_file(sock);
    }

    #[tokio::test]
    async fn initiate_fails_when_response_frame_truncated() {
        let peer_pk = [0x66u8; PUBLIC_KEY_SIZE];
        let mut recv = peer_pk.to_vec();
        recv.extend_from_slice(&[0u8; crate::HEADER_SIZE - 1]);

        let mut device = ScriptedBackend::new(recv).into_device(std::time::Duration::from_secs(1));
        let mut ex = GenesisExchange::for_test(fast_timing_config(false), offline_provider());

        let err = ex
            .initiate(&mut device, &sample_credentials())
            .await
            .expect_err("initiator should fail when response header is truncated");
        assert!(
            matches!(err, crate::NfcError::ConnectionLost),
            "expected ConnectionLost on truncated response header, got {err:?}"
        );
    }
}
