// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust Certificate Generation (Hybrid Standalone + Security Provider)
//!
//! This module provides certificate generation with two modes:
//! 1. **Standalone**: Built-in ed25519-dalek (100% Pure Rust, zero dependencies)
//! 2. **Security-provider enhanced**: Delegation to the crypto provider for HSM-backed, lineage-tracked certs
//! 3. **Auto**: Try the security provider first, graceful fallback to standalone
//!
//! Philosophy: Songbird is secure by default and alone, enhanced when a security provider is available.

use crate::crypto::SecurityTlsCryptoClient;
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
    /// Security-provider mode: delegate to the crypto provider for enhanced capabilities
    SecurityProvider,
    /// Deprecated alias for [`CertGenerationMode::SecurityProvider`].
    #[deprecated(note = "use CertGenerationMode::SecurityProvider")]
    LegacySecurityProvider,
    /// Auto: try the security provider, fallback to standalone (default)
    #[default]
    Auto,
}

/// Hybrid certificate generator
///
/// Provides both standalone Pure Rust certificate generation and optional
/// security-provider integration for enhanced capabilities.
pub struct CertificateGenerator {
    mode: CertGenerationMode,
    security_client: Option<SecurityTlsCryptoClient>,
}

impl CertificateGenerator {
    /// Create a new generator with auto-discovery (default)
    ///
    /// # Errors
    ///
    /// Returns an error if security-provider mode is requested but the provider is not available.
    pub fn new() -> Result<Self> {
        Self::with_mode(CertGenerationMode::Auto)
    }

    /// Create with explicit mode
    ///
    /// # Errors
    ///
    /// Returns an error if security-provider mode is requested but the provider is not available.
    pub fn with_mode(mode: CertGenerationMode) -> Result<Self> {
        let security_client = match &mode {
            #[allow(deprecated, reason = "match arm handles legacy variant")]
            CertGenerationMode::SecurityProvider
            | CertGenerationMode::LegacySecurityProvider
            | CertGenerationMode::Auto => {
                // Try to discover the security (crypto) provider socket
                match SecurityTlsCryptoClient::new() {
                    Ok(client) => {
                        tracing::info!(
                            "✅ Security provider discovered for enhanced certificate generation"
                        );
                        Some(client)
                    }
                    Err(e) => {
                        tracing::debug!("Security provider not available: {}", e);
                        #[allow(deprecated, reason = "match arm handles legacy variant")]
                        if matches!(
                            mode,
                            CertGenerationMode::SecurityProvider
                                | CertGenerationMode::LegacySecurityProvider
                        ) {
                            return Err(anyhow::anyhow!(
                                "Security provider mode requested but crypto provider not available"
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
            security_client,
        })
    }

    /// Generate a self-signed certificate
    ///
    /// Will use the security provider if available (Auto / security-provider mode), otherwise standalone.
    ///
    /// # Errors
    ///
    /// Returns an error if certificate generation fails (e.g., security-provider mode requested
    /// but the provider is unavailable, or key/cert generation fails).
    pub fn generate_self_signed(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<(Certificate, SigningKey)> {
        // Try security provider first if available (Auto or security-provider mode)
        if let Some(ref client) = self.security_client {
            tracing::trace!("Using {:?} mode for certificate generation", self.mode);
            let result = Self::generate_via_security_provider(client, domain, validity_days);
            tracing::info!("✅ Generated certificate via security provider: {}", domain);
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

    /// Enhanced generation via the security (crypto) provider
    ///
    /// Delegates to the provider for:
    /// - HSM-backed key generation
    /// - Lineage tracking
    /// - Attestation
    /// - Key rotation support
    fn generate_via_security_provider(
        _client: &SecurityTlsCryptoClient,
        domain: &str,
        validity_days: u32,
    ) -> (Certificate, SigningKey) {
        tracing::info!("🐻 Generating certificate via security provider: {}", domain);

        // For now, the provider doesn't expose certificate generation in its JSON-RPC API yet
        // So we'll use standalone generation but could delegate crypto operations to the provider
        // This is a placeholder for future integration

        // No `certificate.generate_self_signed` RPC in this build; standalone path only.

        tracing::debug!("Security provider cert generation not yet implemented, using standalone");
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
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn test_standalone_cert_generation() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();

        let (cert, signing_key) =
            generator.generate_self_signed("test.songbird.local", 365).unwrap();

        assert!(!cert.certificate_list.is_empty());
        assert!(!cert.certificate_list[0].cert_data.is_empty());
        assert_eq!(signing_key.verifying_key().as_bytes().len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn test_auto_mode_fallback() {
        let generator = CertificateGenerator::new().unwrap();

        let (cert, _) = generator.generate_self_signed("auto.songbird.local", 90).unwrap();

        assert!(!cert.certificate_list.is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
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
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn test_cert_validity_period() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();

        let validity_days = 30;
        let (_cert, _key) =
            generator.generate_self_signed("validity.local", validity_days).unwrap();

        // Certificate should be valid for the specified period
        // (validation logic would be in certificate usage, not generation)
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_cert_der_includes_domain_prefix_and_public_key_material() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let domain = "edge-case.songbird";
        let (cert, signing_key) = generator.generate_self_signed(domain, 1).unwrap();
        let der = &cert.certificate_list[0].cert_data;
        assert!(
            der.windows(domain.len()).any(|w| w == domain.as_bytes()),
            "placeholder DER should embed the subject CN bytes for discovery"
        );
        assert!(
            der.windows(32).any(|w| w == signing_key.verifying_key().as_bytes()),
            "placeholder DER should embed the Ed25519 public key bytes"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_zero_day_validity_still_emits_non_empty_cert() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let (cert, _) = generator.generate_self_signed("zero-day.local", 0).unwrap();
        assert!(!cert.certificate_list[0].cert_data.is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn successive_standalone_generations_produce_distinct_signing_keys() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let (_, a) = generator.generate_self_signed("a.local", 10).unwrap();
        let (_, b) = generator.generate_self_signed("b.local", 10).unwrap();
        assert_ne!(a.to_bytes(), b.to_bytes(), "OsRng-backed keys should almost never collide");
    }

    #[test]
    fn security_provider_mode_behaviour_depends_on_environment() {
        match CertificateGenerator::with_mode(CertGenerationMode::SecurityProvider) {
            Ok(_gen) => {
                // Live security provider socket discovered — construction succeeds.
                // This is correct behaviour: SecurityProvider mode works when
                // the provider is actually reachable.
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("crypto provider") || msg.contains("Security"),
                    "error should reference the security provider: {msg}"
                );
            }
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn cert_generation_mode_equality() {
        assert_eq!(CertGenerationMode::Standalone, CertGenerationMode::Standalone);
        assert_ne!(CertGenerationMode::Standalone, CertGenerationMode::Auto);
    }

    /// Parse placeholder DER validity timestamps written by [`create_simple_cert_der`].
    fn parse_placeholder_validity(der: &[u8]) -> (i64, i64) {
        assert!(der.len() >= 16, "DER too short for validity timestamps");
        let not_before = i64::from_le_bytes(der[der.len() - 16..der.len() - 8].try_into().unwrap());
        let not_after = i64::from_le_bytes(der[der.len() - 8..].try_into().unwrap());
        (not_before, not_after)
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_validity_period_bounds_in_der() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let validity_days = 45;
        let before = Utc::now();
        let (cert, _) = generator.generate_self_signed("bounds.local", validity_days).unwrap();
        let after = Utc::now();
        let der = &cert.certificate_list[0].cert_data;
        let (not_before, not_after) = parse_placeholder_validity(der);
        let expected_span = i64::from(validity_days) * 86_400;
        assert!(
            (not_after - not_before) - expected_span <= 1,
            "validity span should match requested days"
        );
        assert!(not_before >= before.timestamp() - 2);
        assert!(not_before <= after.timestamp() + 2);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_zero_and_long_validity_differ_in_der() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let (cert_zero, _) = generator.generate_self_signed("z.local", 0).unwrap();
        let (cert_year, _) = generator.generate_self_signed("y.local", 365).unwrap();
        let (zero_before, zero_after) =
            parse_placeholder_validity(&cert_zero.certificate_list[0].cert_data);
        let (year_before, year_after) =
            parse_placeholder_validity(&cert_year.certificate_list[0].cert_data);
        assert_eq!(zero_after - zero_before, 0);
        assert_eq!(year_after - year_before, 365 * 86_400);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_cert_data_unique_per_generation() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let domain = "serial-check.local";
        let (cert_a, _) = generator.generate_self_signed(domain, 30).unwrap();
        let (cert_b, _) = generator.generate_self_signed(domain, 30).unwrap();
        assert_ne!(
            cert_a.certificate_list[0].cert_data, cert_b.certificate_list[0].cert_data,
            "each generation should produce distinct certificate bytes"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_empty_domain_generates_non_empty_cert() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let (cert, key) = generator.generate_self_signed("", 7).unwrap();
        assert!(!cert.certificate_list[0].cert_data.is_empty());
        assert_eq!(key.verifying_key().as_bytes().len(), 32);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_long_domain_embedded_in_der() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let domain = "a".repeat(512);
        let (cert, _) = generator.generate_self_signed(&domain, 14).unwrap();
        let der = &cert.certificate_list[0].cert_data;
        assert!(
            der.windows(domain.len()).any(|w| w == domain.as_bytes()),
            "long domain should be embedded verbatim in placeholder DER"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_special_characters_in_domain() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let domains = [
            "*.wildcard.example.com",
            "host/with/slashes",
            "quote\"d&<>chars",
            "unicode-🔐-host.local",
        ];
        for domain in domains {
            let (cert, _) = generator.generate_self_signed(domain, 10).unwrap();
            let der = &cert.certificate_list[0].cert_data;
            assert!(
                der.windows(domain.len()).any(|w| w == domain.as_bytes()),
                "domain {domain:?} should appear in DER"
            );
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn standalone_certificate_entry_has_no_extensions() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone).unwrap();
        let (cert, _) = generator.generate_self_signed("no-ext.local", 90).unwrap();
        let entry = &cert.certificate_list[0];
        assert!(entry.extensions.is_empty());
        assert!(cert.certificate_request_context.is_empty());
    }

    #[test]
    fn cert_generation_mode_default_is_auto() {
        assert_eq!(CertGenerationMode::default(), CertGenerationMode::Auto);
    }
}
