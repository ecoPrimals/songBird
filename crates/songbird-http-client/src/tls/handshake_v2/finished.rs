// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Finished Message Handler
//!
//! This module handles the Finished handshake message as defined in RFC 8446 Section 4.4.4.
//!
//! ## RFC 8446 Compliance
//!
//! From RFC 8446 Section 4.4.4:
//! ```text
//! struct {
//!     opaque verify_data[Hash.length];
//! } Finished;
//! ```
//!
//! The `verify_data` is computed as:
//! ```text
//! verify_data = HMAC(finished_key, Transcript-Hash(Handshake Context, Certificate*, CertificateVerify*))
//!
//! where finished_key is derived from the appropriate traffic secret:
//! finished_key = HKDF-Expand-Label(Base Key, "finished", "", Hash.length)
//! ```
//!
//! ## Design Philosophy
//!
//! - **Agnostic**: Works for both client and server
//! - **Reusable**: Pure logic, no state dependencies
//! - **Defensive**: Validates all inputs
//! - **Informative**: Comprehensive logging
//!
//! ## Reusability
//!
//! This module is designed to be reusable by:
//! - TLS client (sends client Finished)
//! - TLS server (sends server Finished, validates client Finished)

use crate::error::{Error, Result};
use tracing::{debug, error, info};

/// Build a Finished handshake message
///
/// # Arguments
/// * `verify_data` - The computed `verify_data` from `HMAC(finished_key`, `transcript_hash`)
///
/// # Returns
/// * Complete Finished handshake message (type + length + `verify_data`)
///
/// # Example
/// ```rust,ignore
/// let verify_data = crypto.compute_finished_verify_data(base_key, transcript_hash).await?;
/// let finished_msg = build_finished_message(&verify_data)?;
/// ```
///
/// # Errors
///
/// Returns an error if `verify_data` is empty or has an invalid length for TLS 1.3 Finished.
#[expect(clippy::cast_possible_truncation, reason = "TLS wire format: values are masked/bounded")]
pub fn build_finished_message(verify_data: &[u8]) -> Result<Vec<u8>> {
    // Validate verify_data length (should be 32 bytes for SHA-256, 48 for SHA-384)
    if verify_data.is_empty() {
        return Err(Error::TlsHandshake("verify_data is empty".to_string()));
    }

    if verify_data.len() != 32 && verify_data.len() != 48 {
        return Err(Error::TlsHandshake(format!(
            "Invalid verify_data length: {} bytes (expected 32 or 48)",
            verify_data.len()
        )));
    }

    debug!("Building Finished message with {} bytes of verify_data", verify_data.len());

    // Build Finished handshake message
    // Format: HandshakeType (1 byte) + Length (3 bytes) + verify_data
    let mut finished_msg = Vec::new();
    finished_msg.push(0x14); // HandshakeType: Finished

    // Length (3 bytes, big-endian)
    let length = verify_data.len();
    finished_msg.push(((length >> 16) & 0xFF) as u8);
    finished_msg.push(((length >> 8) & 0xFF) as u8);
    finished_msg.push((length & 0xFF) as u8);

    // Verify data
    finished_msg.extend_from_slice(verify_data);

    info!("✅ Built Finished message: {} bytes total", finished_msg.len());
    debug!("   Finished message (hex): {}", hex::encode(&finished_msg));

    Ok(finished_msg)
}

/// Parse a Finished handshake message and extract `verify_data`
///
/// # Arguments
/// * `data` - Complete handshake message (including type + length header)
///
/// # Returns
/// * `Ok(Vec<u8>)` - The extracted `verify_data`
/// * `Err` - If parsing fails or validation fails
///
/// # Example
/// ```rust,ignore
/// let verify_data = parse_finished_message(&handshake_message)?;
/// // Validate verify_data against expected value
/// ```
///
/// # Errors
///
/// Returns an error if the message is malformed or truncated.
pub fn parse_finished_message(data: &[u8]) -> Result<Vec<u8>> {
    debug!("Parsing Finished message: {} bytes", data.len());

    // Validate handshake message type
    if data.is_empty() || data[0] != 0x14 {
        return Err(Error::TlsHandshake(format!(
            "Invalid Finished message: expected type 0x14, got 0x{:02x}",
            data.first().copied().unwrap_or(0xFF)
        )));
    }

    // Parse length (3 bytes, big-endian)
    if data.len() < 4 {
        return Err(Error::TlsHandshake("Finished message too short for header".to_string()));
    }

    let length = ((data[1] as usize) << 16) | ((data[2] as usize) << 8) | (data[3] as usize);
    debug!("Finished message body length: {} bytes", length);

    // Validate length
    if length != 32 && length != 48 {
        return Err(Error::TlsHandshake(format!(
            "Invalid Finished verify_data length: {length} bytes (expected 32 or 48)"
        )));
    }

    // Extract verify_data
    if data.len() < 4 + length {
        return Err(Error::TlsHandshake(format!(
            "Finished message truncated: expected {} bytes, got {}",
            4 + length,
            data.len()
        )));
    }

    let verify_data = data[4..4 + length].to_vec();

    debug!("Extracted verify_data: {} bytes", verify_data.len());
    debug!("  verify_data (hex): {}", hex::encode(&verify_data));

    Ok(verify_data)
}

/// Validate `verify_data` against expected value
///
/// This is used by the receiving party to verify that the sender
/// computed the correct Finished message.
///
/// # Arguments
/// * `received` - The `verify_data` from the received Finished message
/// * `expected` - The `verify_data` computed locally
///
/// # Returns
/// * `Ok(())` - If `verify_data` matches
/// * `Err` - If `verify_data` doesn't match (handshake failure)
///
/// # Errors
///
/// Returns an error if lengths differ or the `verify_data` bytes do not match.
pub fn validate_verify_data(received: &[u8], expected: &[u8]) -> Result<()> {
    if received.len() != expected.len() {
        error!(
            "verify_data length mismatch: received {} bytes, expected {} bytes",
            received.len(),
            expected.len()
        );
        return Err(Error::TlsHandshake(format!(
            "verify_data length mismatch: {} != {}",
            received.len(),
            expected.len()
        )));
    }

    // Constant-time comparison (important for security!)
    let mut differences = 0u8;
    for (a, b) in received.iter().zip(expected.iter()) {
        differences |= a ^ b;
    }

    if differences != 0 {
        error!("❌ verify_data mismatch!");
        error!("   Received: {}", hex::encode(received));
        error!("   Expected: {}", hex::encode(expected));
        return Err(Error::TlsHandshake("verify_data mismatch - handshake failed".to_string()));
    }

    info!("✅ verify_data validated successfully");
    Ok(())
}

/// Add `ContentType` byte for TLS 1.3 encryption
///
/// In TLS 1.3, the `ContentType` (0x16 = Handshake) is encrypted as part of the payload.
/// This function prepares the Finished message for encryption.
///
/// # Arguments
/// * `finished_msg` - The Finished handshake message
///
/// # Returns
/// * Plaintext ready for AEAD encryption (Finished + `ContentType` byte)
pub fn prepare_for_encryption(finished_msg: &[u8]) -> Vec<u8> {
    let mut plaintext = finished_msg.to_vec();
    plaintext.push(0x16); // ContentType: Handshake

    debug!("Prepared Finished for encryption: {} bytes (includes ContentType)", plaintext.len());
    debug!("   Last byte (ContentType): 0x{:02x}", plaintext[plaintext.len() - 1]);

    plaintext
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_finished_message() {
        let verify_data = vec![0x42; 32]; // 32 bytes for SHA-256
        let finished = build_finished_message(&verify_data).unwrap();

        // Check structure: type (1) + length (3) + verify_data (32) = 36 bytes
        assert_eq!(finished.len(), 36);
        assert_eq!(finished[0], 0x14); // HandshakeType: Finished
        assert_eq!(finished[1], 0x00); // Length MSB
        assert_eq!(finished[2], 0x00); // Length
        assert_eq!(finished[3], 0x20); // Length LSB (32)
        assert_eq!(&finished[4..], &verify_data[..]);
    }

    #[test]
    fn test_build_finished_message_sha384() {
        let verify_data = vec![0x42; 48]; // 48 bytes for SHA-384
        let finished = build_finished_message(&verify_data).unwrap();

        // Check structure: type (1) + length (3) + verify_data (48) = 52 bytes
        assert_eq!(finished.len(), 52);
        assert_eq!(finished[0], 0x14);
        assert_eq!(finished[3], 0x30); // Length LSB (48)
    }

    #[test]
    fn test_build_finished_message_empty() {
        let verify_data = vec![];
        let result = build_finished_message(&verify_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_finished_message_invalid_length() {
        let verify_data = vec![0x42; 31]; // Invalid length
        let result = build_finished_message(&verify_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_finished_message() {
        let mut msg = vec![
            0x14, // HandshakeType: Finished
            0x00, 0x00, 0x20, // Length: 32 bytes
        ];
        msg.extend_from_slice(&[0x42; 32]); // verify_data

        let verify_data = parse_finished_message(&msg).unwrap();
        assert_eq!(verify_data.len(), 32);
        assert_eq!(verify_data, vec![0x42; 32]);
    }

    #[test]
    fn test_parse_finished_message_invalid_type() {
        let msg = vec![0x01, 0x00, 0x00, 0x20]; // ClientHello type
        let result = parse_finished_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_finished_message_truncated() {
        let msg = vec![0x14, 0x00, 0x00, 0x20]; // Header only, no body
        let result = parse_finished_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_verify_data_success() {
        let data = vec![0x42; 32];
        let result = validate_verify_data(&data, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_verify_data_mismatch() {
        let received = vec![0x42; 32];
        let expected = vec![0x43; 32];
        let result = validate_verify_data(&received, &expected);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_verify_data_length_mismatch() {
        let received = vec![0x42; 32];
        let expected = vec![0x42; 48];
        let result = validate_verify_data(&received, &expected);
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_for_encryption() {
        let finished = vec![0x14, 0x00, 0x00, 0x20];
        let plaintext = prepare_for_encryption(&finished);

        assert_eq!(plaintext.len(), finished.len() + 1);
        assert_eq!(plaintext[plaintext.len() - 1], 0x16); // ContentType: Handshake
    }
}
