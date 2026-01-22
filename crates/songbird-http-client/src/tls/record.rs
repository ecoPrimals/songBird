//! TLS 1.3 record layer

use crate::beardog_client::BearDogClient;
use crate::error::{Error, Result};
use crate::tls::content_type;
use crate::tls::session::SessionKeys;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::trace;

/// TLS record layer
pub struct TlsRecordLayer {
    beardog: Arc<BearDogClient>,
    keys: SessionKeys,
    write_sequence_number: u64,
    read_sequence_number: u64,
}

impl TlsRecordLayer {
    /// Create a new TLS record layer
    pub fn new(beardog: Arc<BearDogClient>, keys: SessionKeys) -> Self {
        Self {
            beardog,
            keys,
            write_sequence_number: 0,
            read_sequence_number: 0,
        }
    }

    /// Write application data
    pub async fn write_application_data(
        &mut self,
        stream: &mut TcpStream,
        data: &[u8],
    ) -> Result<()> {
        trace!("📤 Writing {} bytes of application data (write_seq={})", data.len(), self.write_sequence_number);

        // Calculate encrypted length (plaintext + 16-byte AEAD tag)
        let encrypted_length = data.len() + 16;
        
        // Build AAD (TLS record header)
        let aad = [
            content_type::APPLICATION_DATA,
            0x03, 0x03,  // TLS 1.2 (compatibility)
            (encrypted_length >> 8) as u8,
            (encrypted_length & 0xFF) as u8,
        ];
        
        trace!("AAD (write): {:02x?}", aad);

        // Build nonce: IV XOR write_sequence_number (RFC 8446 Section 5.3)
        let nonce = self.build_write_nonce();
        trace!("Nonce (write): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);
        
        // Encrypt data
        let encrypted = self.beardog
            .encrypt(&self.keys.client_write_key, &nonce, data, &aad)
            .await?;

        trace!("Encrypted {} bytes → {} bytes", data.len(), encrypted.len());

        // Build complete TLS record
        let mut record = Vec::new();
        record.extend_from_slice(&aad);  // Header (5 bytes)
        record.extend_from_slice(&encrypted);  // Ciphertext + tag

        // Write to stream
        stream.write_all(&record).await?;
        stream.flush().await?;

        self.write_sequence_number += 1;

        Ok(())
    }

    /// Read application data
    pub async fn read_application_data(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<Vec<u8>> {
        trace!("📥 Reading application data (read_seq={})", self.read_sequence_number);

        // Read record header (5 bytes)
        let mut header = [0u8; 5];
        stream.read_exact(&mut header).await?;

        let content_type = header[0];
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        trace!("Record header: type={:#04x}, length={}", content_type, length);

        if content_type != content_type::APPLICATION_DATA {
            return Err(Error::TlsRecord(format!(
                "Expected APPLICATION_DATA (0x17), got {:#04x}",
                content_type
            )));
        }

        // Read encrypted data (includes 16-byte AEAD tag)
        let mut encrypted = vec![0u8; length];
        stream.read_exact(&mut encrypted).await?;

        trace!("Read {} bytes of encrypted data", encrypted.len());

        // AAD = TLS record header
        let aad = &header;
        trace!("AAD (read): {:02x?}", aad);

        // Build nonce: IV XOR read_sequence_number (RFC 8446 Section 5.3)
        let nonce = self.build_read_nonce();
        trace!("Nonce (read): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);

        // Decrypt data (BearDog will validate AEAD tag)
        let decrypted = self.beardog
            .decrypt(&self.keys.server_write_key, &nonce, &encrypted, aad)
            .await?;

        trace!("Decrypted {} bytes → {} bytes (AEAD authentication succeeded)", encrypted.len(), decrypted.len());

        self.read_sequence_number += 1;

        Ok(decrypted)
    }

    /// Build nonce for writing (encryption)
    /// RFC 8446 Section 5.3: nonce = IV XOR sequence_number (right-aligned)
    fn build_write_nonce(&self) -> Vec<u8> {
        let mut nonce = self.keys.client_write_iv.clone();
        let seq_bytes = self.write_sequence_number.to_be_bytes();
        
        // XOR sequence number with IV (right-aligned)
        // For 12-byte IV and 8-byte sequence: XOR last 8 bytes
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        
        nonce
    }

    /// Build nonce for reading (decryption)
    /// RFC 8446 Section 5.3: nonce = IV XOR sequence_number (right-aligned)
    fn build_read_nonce(&self) -> Vec<u8> {
        let mut nonce = self.keys.server_write_iv.clone();
        let seq_bytes = self.read_sequence_number.to_be_bytes();
        
        // XOR sequence number with IV (right-aligned)
        // For 12-byte IV and 8-byte sequence: XOR last 8 bytes
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        
        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_write_nonce() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![0; 12],
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        let nonce = layer.build_write_nonce();
        assert_eq!(nonce.len(), 12);
        
        // Sequence number should affect nonce
        layer.write_sequence_number = 1;
        let nonce2 = layer.build_write_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_build_read_nonce() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![0; 12],
            server_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        let nonce = layer.build_read_nonce();
        assert_eq!(nonce.len(), 12);
        
        // Sequence number should affect nonce
        layer.read_sequence_number = 1;
        let nonce2 = layer.build_read_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_separate_sequence_numbers() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        
        // Write and read should use different nonces due to different IVs
        let write_nonce = layer.build_write_nonce();
        let read_nonce = layer.build_read_nonce();
        assert_ne!(write_nonce, read_nonce, "Write and read nonces should differ");
        
        // Increment sequence numbers independently
        layer.write_sequence_number = 5;
        layer.read_sequence_number = 3;
        
        let write_nonce2 = layer.build_write_nonce();
        let read_nonce2 = layer.build_read_nonce();
        
        // Nonces should change
        assert_ne!(write_nonce, write_nonce2);
        assert_ne!(read_nonce, read_nonce2);
        assert_ne!(write_nonce2, read_nonce2);
    }
}


