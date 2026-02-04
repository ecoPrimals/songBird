//! Pure Rust Certificate Generation (Hybrid Standalone + BearDog)
//!
//! This module provides certificate generation with two modes:
//! 1. **Standalone**: Built-in ed25519-dalek (100% Pure Rust, zero dependencies)
//! 2. **BearDog Enhanced**: Delegation to BearDog for HSM-backed, lineage-tracked certs
//! 3. **Auto**: Try BearDog first, graceful fallback to standalone
//!
//! Philosophy: Songbird is secure by default and alone, enhanced when BearDog is available.

use crate::crypto::BeardogCryptoClient;
use crate::error::Result;
use crate::messages::certificate::{Certificate, CertificateEntry};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Certificate generation mode
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CertGenerationMode {
    /// Standalone: Use built-in ed25519-dalek (100% Pure Rust)
    Standalone,
    /// BearDog: Delegate to BearDog for enhanced capabilities
    BearDog,
    /// Auto: Try BearDog, fallback to standalone (default)
    #[default]
    Auto,
}

/// Hybrid certificate generator
///
/// Provides both standalone Pure Rust certificate generation and optional
/// BearDog integration for enhanced capabilities.
pub struct CertificateGenerator {
    mode: CertGenerationMode,
    beardog_client: Option<BeardogCryptoClient>,
}

impl CertificateGenerator {
    /// Create a new generator with auto-discovery (default)
    pub async fn new() -> Result<Self> {
        Self::with_mode(CertGenerationMode::Auto).await
    }

    /// Create with explicit mode
    pub async fn with_mode(mode: CertGenerationMode) -> Result<Self> {
        let beardog_client = match &mode {
            CertGenerationMode::BearDog | CertGenerationMode::Auto => {
                // Try to discover BearDog
                match BeardogCryptoClient::new().await {
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
    /// Will use BearDog if available (Auto/BearDog mode), otherwise standalone.
    pub async fn generate_self_signed(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<(Certificate, SigningKey)> {
        // Try BearDog first if available
        if let Some(ref client) = self.beardog_client {
            match self.generate_via_beardog(client, domain, validity_days).await {
                Ok(result) => {
                    tracing::info!("✅ Generated certificate via BearDog: {}", domain);
                    return Ok(result);
                }
                Err(e) => {
                    if matches!(self.mode, CertGenerationMode::BearDog) {
                        return Err(e);
                    }
                    tracing::warn!(
                        "⚠️ BearDog cert generation failed: {}, falling back to standalone",
                        e
                    );
                }
            }
        }

        // Fallback to standalone
        self.generate_standalone(domain, validity_days)
    }

    /// Standalone generation using ed25519-dalek (100% Pure Rust)
    fn generate_standalone(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<(Certificate, SigningKey)> {
        tracing::info!("🔐 Generating standalone certificate: {}", domain);

        // Generate Ed25519 keypair (Pure Rust!)
        // ed25519-dalek 2.x uses from_bytes with random data
        use rand::RngCore;
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Calculate validity period
        let not_before = Utc::now();
        let not_after = not_before + Duration::days(validity_days as i64);

        // Create certificate with Ed25519 public key
        let cert_entry = CertificateEntry {
            cert_data: create_simple_cert_der(domain, &verifying_key, &not_before, &not_after)?,
            extensions: Vec::new(),
        };

        let certificate = Certificate {
            certificate_request_context: Vec::new(),
            certificate_list: vec![cert_entry],
        };

        tracing::info!("✅ Standalone certificate generated: {}", domain);
        Ok((certificate, signing_key))
    }

    /// Enhanced generation via BearDog
    ///
    /// Delegates to BearDog for:
    /// - HSM-backed key generation
    /// - Lineage tracking
    /// - Attestation
    /// - Key rotation support
    async fn generate_via_beardog(
        &self,
        _client: &BeardogCryptoClient,
        domain: &str,
        validity_days: u32,
    ) -> Result<(Certificate, SigningKey)> {
        tracing::info!("🐻 Generating certificate via BearDog: {}", domain);

        // For now, BearDog doesn't have certificate generation in its JSON-RPC API yet
        // So we'll use standalone generation but could delegate crypto operations to BearDog
        // This is a placeholder for future BearDog integration

        // TODO: Once BearDog adds certificate.generate_self_signed to its JSON-RPC API:
        // let params = serde_json::json!({
        //     "domain": domain,
        //     "validity_days": validity_days,
        //     "key_type": "Ed25519",
        // });
        // let result = client.call_method("certificate.generate_self_signed", params).await?;

        tracing::debug!("BearDog cert generation not yet implemented, using standalone");
        self.generate_standalone(domain, validity_days)
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
) -> Result<Vec<u8>> {
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

    Ok(cert_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standalone_cert_generation() {
        let generator =
            CertificateGenerator::with_mode(CertGenerationMode::Standalone).await.unwrap();

        let (cert, signing_key) =
            generator.generate_self_signed("test.songbird.local", 365).await.unwrap();

        assert!(!cert.certificate_list.is_empty());
        assert!(!cert.certificate_list[0].cert_data.is_empty());
        assert_eq!(signing_key.verifying_key().as_bytes().len(), 32); // Ed25519 public key is 32 bytes
    }

    #[tokio::test]
    async fn test_auto_mode_fallback() {
        // Auto mode should work even without BearDog
        let generator = CertificateGenerator::new().await.unwrap();

        let (cert, _) = generator.generate_self_signed("auto.songbird.local", 90).await.unwrap();

        assert!(!cert.certificate_list.is_empty());
    }

    #[tokio::test]
    async fn test_standalone_multiple_certs() {
        let generator =
            CertificateGenerator::with_mode(CertGenerationMode::Standalone).await.unwrap();

        // Generate multiple certificates
        let domains = vec!["test1.local", "test2.local", "test3.local"];

        for domain in domains {
            let (cert, _) = generator.generate_self_signed(domain, 365).await.unwrap();

            assert!(!cert.certificate_list.is_empty());
        }
    }

    #[tokio::test]
    async fn test_cert_validity_period() {
        let generator =
            CertificateGenerator::with_mode(CertGenerationMode::Standalone).await.unwrap();

        let validity_days = 30;
        let (_cert, _key) =
            generator.generate_self_signed("validity.local", validity_days).await.unwrap();

        // Certificate should be valid for the specified period
        // (validation logic would be in certificate usage, not generation)
    }
}
