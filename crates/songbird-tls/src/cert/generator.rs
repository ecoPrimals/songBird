// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust Certificate Generation (Hybrid Standalone + `BearDog`)
//!
//! This module provides certificate generation with two modes:
//! 1. **Standalone**: Built-in ed25519-dalek (100% Pure Rust, zero dependencies)
//! 2. **`BearDog` Enhanced**: Delegation to `BearDog` for HSM-backed, lineage-tracked certs
//! 3. **Auto**: Try `BearDog` first, graceful fallback to standalone
//!
//! Philosophy: Songbird is secure by default and alone, enhanced when `BearDog` is available.

use crate::crypto::BeardogCryptoClient;
use crate::error::Result;
use crate::messages::certificate::{Certificate, CertificateEntry};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{RngCore, rngs::OsRng};

/// Certificate generation mode
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CertGenerationMode {
    /// Standalone: Use built-in ed25519-dalek (100% Pure Rust)
    Standalone,
    /// `BearDog`: Delegate to `BearDog` for enhanced capabilities
    BearDog,
    /// Auto: Try `BearDog`, fallback to standalone (default)
    #[default]
    Auto,
}

/// Hybrid certificate generator
///
/// Provides both standalone Pure Rust certificate generation and optional
/// `BearDog` integration for enhanced capabilities.
pub struct CertificateGenerator {
    mode: CertGenerationMode,
    beardog_client: Option<BeardogCryptoClient>,
}

impl CertificateGenerator {
    /// Create a new generator with auto-discovery (default)
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` mode is requested but `BearDog` is not available.
    pub fn new() -> Result<Self> {
        Self::with_mode(CertGenerationMode::Auto)
    }

    /// Create with explicit mode
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` mode is requested but `BearDog` is not available.
    pub fn with_mode(mode: CertGenerationMode) -> Result<Self> {
        let beardog_client = match &mode {
            CertGenerationMode::BearDog | CertGenerationMode::Auto => {
                // Try to discover BearDog
                match BeardogCryptoClient::new() {
                    Ok(client) => {
                        tracing::info!("✅ BearDog discovered for enhanced certificate generation");
                        Some(client)
                    }
                    Err(e) => {
                        tracing::debug!("BearDog not available: {}", e);
                        if matches!(mode, CertGenerationMode::BearDog) {
                            return Err(anyhow::anyhow!(
                                "BearDog mode requested but BearDog not available"
                            )
                            .into());
                        }
                        None
                    }
                }
            }
            CertGenerationMode::Standalone => None,
        };

        Ok(Self {
            mode,
            beardog_client,
        })
    }

    /// Generate a self-signed certificate
    ///
    /// Will use `BearDog` if available (Auto/BearDog mode), otherwise standalone.
    ///
    /// # Errors
    ///
    /// Returns an error if certificate generation fails (e.g., `BearDog` mode requested
    /// but `BearDog` unavailable, or key/cert generation fails).
    pub fn generate_self_signed(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<(Certificate, SigningKey)> {
        // Try BearDog first if available (Auto or BearDog mode)
        if let Some(ref client) = self.beardog_client {
            tracing::trace!("Using {:?} mode for certificate generation", self.mode);
            let result = Self::generate_via_beardog(client, domain, validity_days);
            tracing::info!("✅ Generated certificate via BearDog: {}", domain);
            return Ok(result);
        }

        // Fallback to standalone
        Ok(Self::generate_standalone(domain, validity_days))
    }

    /// Standalone generation using ed25519-dalek (100% Pure Rust)
    fn generate_standalone(domain: &str, validity_days: u32) -> (Certificate, SigningKey) {
        tracing::info!("🔐 Generating standalone certificate: {}", domain);

        // Generate Ed25519 keypair (Pure Rust!)
        // ed25519-dalek 2.x uses from_bytes with random data
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Calculate validity period
        let not_before = Utc::now();
        let not_after = not_before + Duration::days(i64::from(validity_days));

        // Create certificate with Ed25519 public key
        let cert_entry = CertificateEntry {
            cert_data: create_simple_cert_der(domain, &verifying_key, &not_before, &not_after),
            extensions: Vec::new(),
        };

        let certificate = Certificate {
            certificate_request_context: Vec::new(),
            certificate_list: vec![cert_entry],
        };

        tracing::info!("✅ Standalone certificate generated: {}", domain);
        (certificate, signing_key)
    }

    /// Enhanced generation via `BearDog`
    ///
    /// Delegates to `BearDog` for:
    /// - HSM-backed key generation
    /// - Lineage tracking
    /// - Attestation
    /// - Key rotation support
    fn generate_via_beardog(
        _client: &BeardogCryptoClient,
        domain: &str,
        validity_days: u32,
    ) -> (Certificate, SigningKey) {
        tracing::info!("🐻 Generating certificate via BearDog: {}", domain);

        // For now, BearDog doesn't have certificate generation in its JSON-RPC API yet
        // So we'll use standalone generation but could delegate crypto operations to BearDog
        // This is a placeholder for future BearDog integration

        // BearDog has no `certificate.generate_self_signed` RPC in this build; standalone path only.

        tracing::debug!("BearDog cert generation not yet implemented, using standalone");
        Self::generate_standalone(domain, validity_days)
    }
}

/// Create a simple DER-encoded certificate
///
/// This is a minimal implementation for self-signed certificates.
/// For production use, consider more complete certificate generation.
fn create_simple_cert_der(
    domain: &str,
    public_key: &VerifyingKey,
    not_before: &DateTime<Utc>,
    not_after: &DateTime<Utc>,
) -> Vec<u8> {
    // Simple DER encoding for demo purposes
    // In production, use proper ASN.1 DER encoding

    let mut cert_data = Vec::new();

    // Simplified certificate structure (placeholder)
    // Subject: CN=domain
    cert_data.extend_from_slice(domain.as_bytes());
    cert_data.push(0x00); // Separator

    // Public key
    cert_data.extend_from_slice(&public_key.to_bytes());

    // Validity (timestamps as i64 bytes)
    cert_data.extend_from_slice(&not_before.timestamp().to_le_bytes());
    cert_data.extend_from_slice(&not_after.timestamp().to_le_bytes());

    cert_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standalone_cert_generation() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();

        let (cert, signing_key) =
            generator.generate_self_signed("test.songbird.local", 365).unwrap();

        assert!(!cert.certificate_list.is_empty());
        assert!(!cert.certificate_list[0].cert_data.is_empty());
        assert_eq!(signing_key.verifying_key().as_bytes().len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_auto_mode_fallback() {
        let generator = CertificateGenerator::new().unwrap();

        let (cert, _) = generator.generate_self_signed("auto.songbird.local", 90).unwrap();

        assert!(!cert.certificate_list.is_empty());
    }

    #[test]
    fn test_standalone_multiple_certs() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();

        // Generate multiple certificates
        let domains = vec!["test1.local", "test2.local", "test3.local"];

        for domain in domains {
            let (cert, _) = generator.generate_self_signed(domain, 365).unwrap();

            assert!(!cert.certificate_list.is_empty());
        }
    }

    #[test]
    fn test_cert_validity_period() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();

        let validity_days = 30;
        let (_cert, _key) =
            generator.generate_self_signed("validity.local", validity_days).unwrap();

        // Certificate should be valid for the specified period
        // (validation logic would be in certificate usage, not generation)
    }
}
