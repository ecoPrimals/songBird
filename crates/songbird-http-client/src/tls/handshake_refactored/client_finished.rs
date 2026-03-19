//! Client Finished message construction and sending
//!
//! RFC 8446 Section 4.4.4: The client Finished message provides key confirmation,
//! binds the client's identity to the exchanged keys, and in PSK mode, authenticates
//! the handshake.

use super::core::TlsHandshake;
use crate::crypto::TlsHandshakeSecrets as TlsSecrets;
use crate::error::{Error, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{debug, error, info, trace};

impl TlsHandshake {
    /// Send client Finished message (RFC 8446 Section 4.4.4)
    ///
    /// Computes the transcript hash, delegates `verify_data` computation to `BearDog`,
    /// builds and encrypts the Finished handshake message, then sends it.
    pub(crate) async fn send_client_finished(
        &self,
        stream: &mut TcpStream,
        handshake_keys: &TlsSecrets,
    ) -> Result<()> {
        info!("🔐 Building client Finished message (RFC 8446 Section 4.4.4)");

        // 1. Compute transcript hash of all handshake messages
        let transcript_hash = self.compute_transcript_hash_for_cipher(self.cipher_suite).await?;
        debug!(
            "Transcript hash for Finished: {} bytes ({})",
            transcript_hash.len(),
            if transcript_hash.len() == 32 {
                "SHA-256"
            } else {
                "SHA-384"
            }
        );

        // 2. Compute verify_data via BearDog (RFC 8446 Section 4.4.4)
        let verify_data = self
            .crypto
            .tls_compute_finished_verify_data(
                &handshake_keys.client_handshake_secret,
                &transcript_hash,
                self.cipher_suite,
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to compute Finished verify_data: {}", e);
                e
            })?;

        debug!("Finished verify_data: {} bytes", verify_data.len());

        // 3. Build Finished handshake message
        let finished_msg = Self::build_finished_message(&verify_data);

        // 4. Add ContentType for TLS 1.3 encryption (RFC 8446 Section 5.2)
        let mut plaintext = finished_msg;
        plaintext.push(0x16); // ContentType: Handshake

        // 5. Encrypt with handshake traffic keys
        let ciphertext = self.encrypt_with_handshake_keys(&plaintext, handshake_keys, 0).await?;

        // 6. Build and send TLS record
        let tls_record = Self::build_tls_record(&ciphertext);
        info!("📤 Sending client Finished: {} bytes", tls_record.len());

        stream.write_all(&tls_record).await.map_err(|e| {
            error!("❌ Failed to write client Finished: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await.map_err(|e| {
            error!("❌ Failed to flush client Finished: {}", e);
            Error::Io(e)
        })?;

        info!("✅ Client Finished sent successfully");
        Ok(())
    }

    /// Build a Finished handshake message from `verify_data`
    fn build_finished_message(verify_data: &[u8]) -> Vec<u8> {
        let mut msg = Vec::with_capacity(4 + verify_data.len());
        msg.push(0x14); // HandshakeType: Finished

        let length = verify_data.len();
        msg.push(((length >> 16) & 0xFF) as u8);
        msg.push(((length >> 8) & 0xFF) as u8);
        msg.push((length & 0xFF) as u8);

        msg.extend_from_slice(verify_data);
        msg
    }

    /// Encrypt plaintext using handshake traffic keys with the given sequence number
    async fn encrypt_with_handshake_keys(
        &self,
        plaintext: &[u8],
        handshake_keys: &TlsSecrets,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        // Build nonce: client_write_iv XOR sequence_number (RFC 8446 Section 5.3)
        let mut nonce = handshake_keys.client_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        let ciphertext_length = plaintext.len() + 16;
        let aad = [
            0x17, // APPLICATION_DATA
            0x03,
            0x03, // TLS 1.2 compatibility
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];

        trace!(
            "Encrypting {} bytes (cipher: 0x{:04x}, seq: {})",
            plaintext.len(),
            self.cipher_suite,
            sequence_number
        );

        let encryption_key = &handshake_keys.client_write_key;

        match self.cipher_suite {
            0x1301 => self.crypto.aes128_gcm_encrypt(encryption_key, &nonce, plaintext, &aad).await,
            0x1302 => self.crypto.aes256_gcm_encrypt(encryption_key, &nonce, plaintext, &aad).await,
            0x1303 => self.crypto.encrypt(encryption_key, &nonce, plaintext, &aad).await,
            _ => Err(Error::TlsHandshake(format!(
                "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                self.cipher_suite
            ))),
        }
        .map_err(|e| {
            error!("❌ Failed to encrypt with handshake keys: {}", e);
            e
        })
    }

    /// Build a complete TLS record (header + ciphertext)
    fn build_tls_record(ciphertext: &[u8]) -> Vec<u8> {
        let mut record = Vec::with_capacity(5 + ciphertext.len());
        record.push(0x17); // APPLICATION_DATA
        record.push(0x03);
        record.push(0x03); // TLS 1.2 compatibility
        record.push(((ciphertext.len() >> 8) & 0xFF) as u8);
        record.push((ciphertext.len() & 0xFF) as u8);
        record.extend_from_slice(ciphertext);
        record
    }

    /// Check if decrypted handshake record contains a Finished message (0x14)
    ///
    /// RFC 8446 Section 5.1: Multiple handshake messages MAY be coalesced into
    /// a single TLS record. This method parses the framing to locate Finished
    /// at any offset.
    pub(crate) fn contains_finished_message(&self, plaintext: &[u8]) -> bool {
        let mut offset = 0;
        // Skip ContentType byte at end (added during encryption)
        let data_len = plaintext.len().saturating_sub(1);

        debug!("🔍 Scanning {} byte plaintext for Finished message", plaintext.len());

        while offset < data_len {
            if plaintext[offset] == 0x14 {
                info!("🎯 SERVER FINISHED DETECTED at offset {}", offset);
                return true;
            }

            // Parse handshake header: type (1) + length (3)
            if offset + 4 > data_len {
                break;
            }

            let msg_type = plaintext[offset];
            let msg_len = u32::from_be_bytes([
                0,
                plaintext[offset + 1],
                plaintext[offset + 2],
                plaintext[offset + 3],
            ]) as usize;

            let msg_name = match msg_type {
                0x08 => "EncryptedExtensions",
                0x0B => "Certificate",
                0x0F => "CertificateVerify",
                0x14 => "Finished",
                _ => "Unknown",
            };
            debug!("  offset {}: type=0x{:02x} ({}) len={}", offset, msg_type, msg_name, msg_len);

            offset += 4 + msg_len;

            if msg_len > 65536 {
                break;
            }
        }

        false
    }
}
