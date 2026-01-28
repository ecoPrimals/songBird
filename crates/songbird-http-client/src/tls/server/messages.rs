//! TLS Message Construction
//!
//! Handles building TLS 1.3 server messages (ServerHello, Certificate, etc.)

use crate::error::{Error, Result};
use crate::tls::handshake_v2::keys::CipherSuite;
use crate::tls::{handshake_type, TLS_1_2, TLS_1_3};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

use super::core::TlsServer;

impl TlsServer {
    /// Generate 32-byte cryptographically secure random value
    ///
    /// Uses OS-provided CSPRNG via getrandom for 28 bytes of randomness,
    /// with first 4 bytes as Unix timestamp per RFC 8446 format.
    pub(super) fn generate_random(&self) -> Vec<u8> {
        let mut random = Vec::with_capacity(32);

        // First 4 bytes: Unix time (seconds since epoch)
        // Note: In TLS 1.3, this is optional but helps prevent replay attacks
        let time =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs()
                as u32;
        random.extend_from_slice(&time.to_be_bytes());

        // Remaining 28 bytes: cryptographically secure random from OS
        // Uses getrandom crate which provides CSPRNG from:
        // - Linux: getrandom(2) syscall or /dev/urandom
        // - macOS: SecRandomCopyBytes
        // - Windows: BCryptGenRandom
        let mut random_bytes = [0u8; 28];
        if getrandom::fill(&mut random_bytes).is_ok() {
            random.extend_from_slice(&random_bytes);
        } else {
            // Fallback: use time-seeded fastrand if getrandom fails
            // This is less secure but still better than predictable pattern
            let seed = time as u64 ^ std::process::id() as u64;
            let mut rng = fastrand::Rng::with_seed(seed);
            for _ in 0..28 {
                random.push(rng.u8(..));
            }
            warn!("Using fallback RNG - getrandom unavailable");
        }

        random
    }

    /// Build ServerHello message
    pub(super) fn build_server_hello(
        &self,
        server_random: &[u8],
        server_public_key: &[u8],
        cipher_suite: CipherSuite,
    ) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // Handshake type: ServerHello
        msg.push(handshake_type::SERVER_HELLO);

        // Placeholder for length (3 bytes)
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // Legacy version (TLS 1.2 for compatibility)
        msg.extend_from_slice(&TLS_1_2.to_be_bytes());

        // Server random (32 bytes)
        msg.extend_from_slice(server_random);

        // Legacy session ID (empty)
        msg.push(0);

        // Cipher suite
        msg.extend_from_slice(&cipher_suite.to_u16().to_be_bytes());

        // Compression method (null)
        msg.push(0);

        // Extensions
        let extensions = self.build_server_hello_extensions(server_public_key)?;
        msg.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        msg.extend_from_slice(&extensions);

        // Fill in length
        let body_len = msg.len() - length_pos - 3;
        msg[length_pos] = ((body_len >> 16) & 0xFF) as u8;
        msg[length_pos + 1] = ((body_len >> 8) & 0xFF) as u8;
        msg[length_pos + 2] = (body_len & 0xFF) as u8;

        Ok(msg)
    }

    /// Build ServerHello extensions
    fn build_server_hello_extensions(&self, server_public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. Supported versions (0x002b) - REQUIRED
        ext.extend_from_slice(&[0x00, 0x2b]); // Extension type
        ext.extend_from_slice(&[0x00, 0x02]); // Length
        ext.extend_from_slice(&TLS_1_3.to_be_bytes()); // TLS 1.3

        // 2. Key share (0x0033) - REQUIRED
        ext.extend_from_slice(&[0x00, 0x33]); // Extension type
        let key_share_len = 2 + 2 + server_public_key.len(); // group + length + key
        ext.extend_from_slice(&(key_share_len as u16).to_be_bytes());
        ext.extend_from_slice(&[0x00, 0x1d]); // group: x25519
        ext.extend_from_slice(&(server_public_key.len() as u16).to_be_bytes());
        ext.extend_from_slice(server_public_key);

        Ok(ext)
    }

    /// Build EncryptedExtensions message
    pub(super) fn build_encrypted_extensions(&self) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // Handshake type: EncryptedExtensions
        msg.push(handshake_type::ENCRYPTED_EXTENSIONS);

        // Placeholder for length (3 bytes)
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // Extensions length (empty for now - could add ALPN here)
        msg.extend_from_slice(&[0x00, 0x00]);

        // Fill in length
        let body_len = msg.len() - length_pos - 3;
        msg[length_pos] = ((body_len >> 16) & 0xFF) as u8;
        msg[length_pos + 1] = ((body_len >> 8) & 0xFF) as u8;
        msg[length_pos + 2] = (body_len & 0xFF) as u8;

        Ok(msg)
    }

    /// Build Certificate message
    pub(super) fn build_certificate(&self) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // Handshake type: Certificate
        msg.push(handshake_type::CERTIFICATE);

        // Placeholder for length (3 bytes)
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // Certificate request context (empty for server cert)
        msg.push(0);

        // Certificate list length
        let cert_list_len = 3 + self.cert_chain.len() + 2; // length + cert + extensions
        msg.extend_from_slice(&((cert_list_len as u32).to_be_bytes()[1..4])); // 3 bytes

        // Certificate entry
        msg.extend_from_slice(&((self.cert_chain.len() as u32).to_be_bytes()[1..4])); // 3 bytes
        msg.extend_from_slice(&self.cert_chain);

        // Extensions (empty)
        msg.extend_from_slice(&[0x00, 0x00]);

        // Fill in length
        let body_len = msg.len() - length_pos - 3;
        msg[length_pos] = ((body_len >> 16) & 0xFF) as u8;
        msg[length_pos + 1] = ((body_len >> 8) & 0xFF) as u8;
        msg[length_pos + 2] = (body_len & 0xFF) as u8;

        Ok(msg)
    }

    /// Build CertificateVerify message
    ///
    /// # Current Status (January 2026)
    ///
    /// **BLOCKED**: Requires BearDog signing API integration
    ///
    /// Per RFC 8446 Section 4.4.3, CertificateVerify contains a signature over:
    /// - 64 spaces (0x20)
    /// - Context string ("TLS 1.3, server CertificateVerify")
    /// - 0x00 separator
    /// - Transcript hash up to this point
    ///
    /// ## Required BearDog API
    ///
    /// Need `crypto.sign_ecdsa_p256_sha256` or `crypto.sign_ed25519` method:
    /// ```json
    /// {
    ///   "method": "crypto.sign",
    ///   "params": {
    ///     "algorithm": "ecdsa_secp256r1_sha256",
    ///     "private_key": "<base64>",
    ///     "data": "<base64-transcript-context>"
    ///   }
    /// }
    /// ```
    ///
    /// ## Workaround for Testing
    ///
    /// Currently returns zero-filled placeholder signature. This allows
    /// the handshake flow to complete for protocol testing, but will fail
    /// signature verification by any real TLS client.
    pub(super) async fn build_certificate_verify(&self) -> Result<Vec<u8>> {
        // Build the data to be signed (RFC 8446 Section 4.4.3)
        let mut to_sign = Vec::new();

        // 64 spaces (0x20)
        to_sign.extend_from_slice(&[0x20; 64]);

        // Context string
        to_sign.extend_from_slice(b"TLS 1.3, server CertificateVerify");

        // Single 0x00 separator
        to_sign.push(0x00);

        // Transcript hash
        let transcript_hash = self.transcript.compute_hash();
        to_sign.extend_from_slice(&transcript_hash);

        // TODO(P0): Add BearDog signing integration
        // Once BearDog exposes `crypto.sign` API, implement:
        // ```
        // let signature = self.crypto.sign(
        //     SignatureAlgorithm::EcdsaSecp256r1Sha256,
        //     &self.private_key,
        //     &to_sign,
        // ).await?;
        // ```
        //
        // For now, use placeholder that will fail real verification
        debug!("⚠️ CertificateVerify using placeholder signature (BearDog signing API pending)");
        let signature = vec![0u8; 64]; // Placeholder - will fail verification

        let mut msg = Vec::new();

        // Handshake type: CertificateVerify
        msg.push(handshake_type::CERTIFICATE_VERIFY);

        // Placeholder for length (3 bytes)
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // Signature algorithm: ecdsa_secp256r1_sha256 (0x0403)
        msg.extend_from_slice(&[0x04, 0x03]);

        // Signature
        msg.extend_from_slice(&(signature.len() as u16).to_be_bytes());
        msg.extend_from_slice(&signature);

        // Fill in length
        let body_len = msg.len() - length_pos - 3;
        msg[length_pos] = ((body_len >> 16) & 0xFF) as u8;
        msg[length_pos + 1] = ((body_len >> 8) & 0xFF) as u8;
        msg[length_pos + 2] = (body_len & 0xFF) as u8;

        Ok(msg)
    }

    /// Build Finished message
    pub(super) async fn build_finished(&self, handshake_secret: &[u8]) -> Result<Vec<u8>> {
        // Compute transcript hash
        let transcript_hash = self.transcript.compute_hash();

        // Compute verify_data via BearDog (expects u16 for cipher_suite)
        let verify_data = self
            .crypto
            .tls_compute_finished_verify_data(
                handshake_secret,
                &transcript_hash,
                self.cipher_suite.to_u16(), // Convert to u16
            )
            .await
            .map_err(|e| Error::TlsHandshake(format!("Failed to compute verify_data: {}", e)))?;

        let mut msg = Vec::new();

        // Handshake type: Finished
        msg.push(handshake_type::FINISHED);

        // Length (3 bytes)
        let length = verify_data.len();
        msg.push(((length >> 16) & 0xFF) as u8);
        msg.push(((length >> 8) & 0xFF) as u8);
        msg.push((length & 0xFF) as u8);

        // Verify data
        msg.extend_from_slice(&verify_data);

        Ok(msg)
    }

    /// Wrap data in TLS record (5-byte header + data)
    pub(super) fn wrap_in_tls_record(&self, content_type_byte: u8, data: &[u8]) -> Vec<u8> {
        let mut record = Vec::with_capacity(5 + data.len());

        record.push(content_type_byte);
        record.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        record.extend_from_slice(&(data.len() as u16).to_be_bytes());
        record.extend_from_slice(data);

        record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::BearDogProvider;
    use crate::tls::content_type;
    use std::sync::Arc;

    fn create_test_server() -> TlsServer {
        let crypto = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
        TlsServer::new(crypto, vec![], vec![])
    }

    #[test]
    fn test_generate_random() {
        let server = create_test_server();
        let random = server.generate_random();
        assert_eq!(random.len(), 32);
    }

    #[test]
    fn test_wrap_in_tls_record() {
        let server = create_test_server();
        let data = vec![1, 2, 3, 4];
        let record = server.wrap_in_tls_record(content_type::HANDSHAKE, &data);

        assert_eq!(record[0], content_type::HANDSHAKE);
        assert_eq!(record.len(), 5 + data.len());
        assert_eq!(&record[5..], data.as_slice());
    }
}
