//! Cryptographic Operations
//!
//! Handles encryption and decryption of TLS records using handshake/application keys.

use crate::error::{Error, Result};
use crate::tls::content_type;
use crate::tls::handshake_v2::keys::CipherSuite;
use tracing::{debug, error};

use super::core::TlsServer;

impl TlsServer {
    /// Encrypt handshake message with handshake traffic keys
    ///
    /// Reference: RFC 8446 Section 5.2 (Record Payload Protection)
    pub(super) async fn encrypt_handshake_message(
        &self,
        plaintext: &[u8],
        key: &[u8],
        iv: &[u8],
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        // Build nonce (IV XOR sequence_number)
        let mut nonce = iv.to_vec();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        debug!("   Nonce (IV XOR seq {}): {:02x?}", sequence_number, nonce);

        // Calculate ciphertext length (plaintext + 16-byte AEAD tag)
        let ciphertext_length = plaintext.len() + 16;

        // Build AAD (TLS record header)
        let record_type = 0x17; // APPLICATION_DATA (all encrypted records use 0x17 in TLS 1.3)
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let aad = [
            record_type,
            version[0],
            version[1],
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];

        debug!("   AAD (TLS record header): {:02x?}", aad);

        // Encrypt via BearDog (uses correct AEAD algorithm based on cipher suite)
        let ciphertext = match self.cipher_suite {
            CipherSuite::Aes128GcmSha256 => {
                debug!("   → Using AES-128-GCM");
                self.crypto.aes128_gcm_encrypt(key, &nonce, plaintext, &aad).await
            }
            CipherSuite::Aes256GcmSha384 => {
                debug!("   → Using AES-256-GCM");
                self.crypto.aes256_gcm_encrypt(key, &nonce, plaintext, &aad).await
            }
            CipherSuite::ChaCha20Poly1305Sha256 => {
                debug!("   → Using ChaCha20-Poly1305");
                self.crypto.encrypt(key, &nonce, plaintext, &aad).await
            }
        }
        .map_err(|e| {
            error!("❌ Encryption failed: {}", e);
            Error::TlsHandshake(format!("Failed to encrypt: {e}"))
        })?;

        debug!(
            "✅ Encrypted {} bytes → {} bytes (includes 16-byte tag)",
            plaintext.len(),
            ciphertext.len()
        );

        Ok(ciphertext)
    }

    /// Decrypt application data with application traffic keys
    ///
    /// Reference: RFC 8446 Section 5.2 (Record Payload Protection)
    pub(super) async fn decrypt_application_data(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        iv: &[u8],
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        // Build nonce (IV XOR sequence_number)
        let mut nonce = iv.to_vec();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        debug!("   Nonce (IV XOR seq {}): {:02x?}", sequence_number, nonce);

        // Build AAD (TLS record header)
        let ciphertext_length = ciphertext.len();
        let record_type = 0x17; // APPLICATION_DATA
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let aad = [
            record_type,
            version[0],
            version[1],
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];

        debug!("   AAD (TLS record header): {:02x?}", aad);

        // Decrypt via BearDog
        let plaintext = match self.cipher_suite {
            CipherSuite::Aes128GcmSha256 => {
                debug!("   → Using AES-128-GCM");
                self.crypto.aes128_gcm_decrypt(key, &nonce, ciphertext, &aad).await
            }
            CipherSuite::Aes256GcmSha384 => {
                debug!("   → Using AES-256-GCM");
                self.crypto.aes256_gcm_decrypt(key, &nonce, ciphertext, &aad).await
            }
            CipherSuite::ChaCha20Poly1305Sha256 => {
                debug!("   → Using ChaCha20-Poly1305");
                self.crypto.decrypt(key, &nonce, ciphertext, &aad).await
            }
        }
        .map_err(|e| {
            error!("❌ Decryption failed: {}", e);
            error!("   AEAD authentication failure");
            Error::TlsHandshake(format!("Failed to decrypt: {e}"))
        })?;

        debug!("✅ Decrypted {} bytes → {} bytes", ciphertext.len(), plaintext.len());

        // Strip ContentType byte (last byte)
        if plaintext.is_empty() {
            return Err(Error::TlsHandshake("Decrypted plaintext is empty".to_string()));
        }

        let content_type_byte = plaintext[plaintext.len() - 1];
        let content = &plaintext[..plaintext.len() - 1];

        debug!("   ContentType: 0x{:02x}", content_type_byte);

        Ok(content.to_vec())
    }

    /// Send encrypted handshake message
    pub(super) async fn send_encrypted_handshake_message(
        &self,
        stream: &mut tokio::net::TcpStream,
        plaintext: &[u8],
        sequence_number: u64,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let handshake_keys = self
            .handshake_keys
            .as_ref()
            .ok_or_else(|| Error::TlsHandshake("Handshake keys not available".to_string()))?;

        // Add ContentType byte for TLS 1.3
        let mut inner_plaintext = plaintext.to_vec();
        inner_plaintext.push(content_type::HANDSHAKE);

        // Encrypt using helper
        let ciphertext = self
            .encrypt_handshake_message(
                &inner_plaintext,
                &handshake_keys.server_write_key,
                &handshake_keys.server_write_iv,
                sequence_number,
            )
            .await?;

        // Wrap in TLS record
        let record = self.wrap_in_tls_record(content_type::APPLICATION_DATA, &ciphertext);

        // Send
        stream.write_all(&record).await.map_err(Error::Io)?;

        Ok(())
    }
}
