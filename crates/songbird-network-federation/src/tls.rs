//! TLS Configuration and Certificate Management
//!
//! Provides native TLS support using rustls for secure HTTPS communication.
//! Supports both self-signed certificates (development/LAN) and CA-signed certificates (production).

use rcgen::{CertificateParams, DistinguishedName, DnType, Ia5String, SanType};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::io;
use std::path::Path;
use std::sync::{Arc, Once};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

/// Ensure rustls crypto provider is installed (required for rustls 0.23+)
///
/// This must be called before any TLS operations. Uses `Once` to ensure it's only called once.
///
/// # Deep Debt Fix (Dec 18, 2025)
/// Previous implementation was a placeholder that did nothing. This caused TLS initialization
/// failures with "Could not automatically determine CryptoProvider" errors.
/// Now properly installs the ring crypto provider at process startup.
static CRYPTO_PROVIDER_INIT: Once = Once::new();

fn ensure_crypto_provider() {
    CRYPTO_PROVIDER_INIT.call_once(|| {
        // Install ring crypto provider for rustls 0.23+
        // This is required before any TLS operations can be performed
        match rustls::crypto::ring::default_provider().install_default() {
            Ok(()) => {
                debug!("✅ Rustls crypto provider (ring) installed successfully");
            }
            Err(_) => {
                // Already installed (by another crate or earlier call)
                debug!("ℹ️  Rustls crypto provider already installed");
            }
        }
    });
}

/// TLS configuration errors
#[derive(Debug, Error)]
pub enum TlsError {
    /// Certificate generation failed
    #[error("Certificate generation failed: {0}")]
    CertificateGenerationFailed(String),

    /// Certificate loading failed
    #[error("Certificate loading failed: {0}")]
    CertificateLoadFailed(String),

    /// Private key loading failed
    #[error("Private key loading failed: {0}")]
    PrivateKeyLoadFailed(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Rustls error
    #[error("Rustls error: {0}")]
    RustlsError(String),
}

/// TLS certificate configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to certificate file (PEM format)
    pub cert_path: String,
    /// Path to private key file (PEM format)
    pub key_path: String,
    /// Subject Alternative Names (SANs)
    pub sans: Vec<String>,
    /// Organization name
    pub organization: String,
    /// Common name (typically the hostname)
    pub common_name: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_path: "certs/songbird.crt".to_string(),
            key_path: "certs/songbird.key".to_string(),
            sans: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            organization: "ecoPrimals".to_string(),
            common_name: "songbird".to_string(),
        }
    }
}

/// TLS certificate manager
pub struct TlsCertificateManager {
    config: TlsConfig,
}

impl TlsCertificateManager {
    /// Create a new TLS certificate manager
    #[must_use]
    pub fn new(config: TlsConfig) -> Self {
        Self {
            config,
        }
    }

    /// Generate a self-signed certificate
    ///
    /// This is suitable for development and LAN deployments.
    /// For production internet deployments, use CA-signed certificates.
    ///
    /// # Errors
    ///
    /// Returns an error if certificate generation or file I/O fails
    ///
    /// # Panics
    ///
    /// Panics if IP address parsing fails after validation (should not happen in practice)
    pub async fn generate_self_signed_certificate(&self) -> Result<(), TlsError> {
        info!("🔐 Generating self-signed TLS certificate");

        // Create certificate parameters
        let mut params = CertificateParams::default();

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &self.config.common_name);
        dn.push(DnType::OrganizationName, &self.config.organization);
        params.distinguished_name = dn;

        // Add Subject Alternative Names (SANs)
        for san in &self.config.sans {
            if san.parse::<std::net::IpAddr>().is_ok() {
                params
                    .subject_alt_names
                    // IP address already validated by is_ip(), safe to unwrap
                    .push(SanType::IpAddress(san.parse().unwrap_or_else(|_| {
                        unreachable!("IP address validation passed but parsing failed")
                    })));
            } else {
                let ia5_string = Ia5String::try_from(san.to_string()).map_err(|e| {
                    TlsError::CertificateGenerationFailed(format!("Invalid DNS name: {e}"))
                })?;
                params.subject_alt_names.push(SanType::DnsName(ia5_string));
            }
        }

        // Generate certificate and key pair
        let key_pair = rcgen::KeyPair::generate()
            .map_err(|e| TlsError::CertificateGenerationFailed(e.to_string()))?;
        let cert = params
            .self_signed(&key_pair)
            .map_err(|e| TlsError::CertificateGenerationFailed(e.to_string()))?;

        // Get PEM-encoded certificate and key
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // Ensure certificate directory exists
        if let Some(parent) = Path::new(&self.config.cert_path).parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write certificate and key to files
        fs::write(&self.config.cert_path, cert_pem.as_bytes()).await?;
        fs::write(&self.config.key_path, key_pem.as_bytes()).await?;

        info!(
            "✅ Self-signed certificate generated: {} / {}",
            self.config.cert_path, self.config.key_path
        );
        debug!("Certificate SANs: {:?}", self.config.sans);

        Ok(())
    }

    /// Load TLS configuration from certificate and key files
    ///
    /// # Errors
    ///
    /// Returns an error if certificate or key loading fails
    pub async fn load_tls_config(&self) -> Result<rustls::ServerConfig, TlsError> {
        info!("🔐 Loading TLS configuration");

        // Ensure crypto provider is installed (required for rustls 0.23+)
        ensure_crypto_provider();

        // Read certificate chain
        let cert_pem = fs::read(&self.config.cert_path).await?;
        let certs = Self::load_certs_from_pem(&cert_pem)?;

        // Read private key
        let key_pem = fs::read(&self.config.key_path).await?;
        let key = Self::load_private_key_from_pem(&key_pem)?;

        // Build rustls server config
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::RustlsError(e.to_string()))?;

        info!("✅ TLS configuration loaded successfully");

        Ok(config)
    }

    /// Load certificates from PEM data
    fn load_certs_from_pem(pem_data: &[u8]) -> Result<Vec<CertificateDer<'static>>, TlsError> {
        let certs = rustls_pemfile::certs(&mut io::Cursor::new(pem_data))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsError::CertificateLoadFailed(e.to_string()))?;

        if certs.is_empty() {
            return Err(TlsError::CertificateLoadFailed(
                "No certificates found in PEM data".to_string(),
            ));
        }

        Ok(certs)
    }

    /// Load private key from PEM data
    fn load_private_key_from_pem(pem_data: &[u8]) -> Result<PrivateKeyDer<'static>, TlsError> {
        // Try to parse as PKCS8 first, then RSA
        let mut cursor = io::Cursor::new(pem_data);

        // Try PKCS8 private key
        if let Some(Ok(key)) = rustls_pemfile::pkcs8_private_keys(&mut cursor).next() {
            return Ok(PrivateKeyDer::Pkcs8(key));
        }

        // Reset cursor and try RSA private key
        cursor.set_position(0);
        if let Some(Ok(key)) = rustls_pemfile::rsa_private_keys(&mut cursor).next() {
            return Ok(PrivateKeyDer::Pkcs1(key));
        }

        Err(TlsError::PrivateKeyLoadFailed("No valid private key found in PEM data".to_string()))
    }

    /// Check if certificates exist
    pub async fn certificates_exist(&self) -> bool {
        tokio::try_join!(fs::metadata(&self.config.cert_path), fs::metadata(&self.config.key_path))
            .is_ok()
    }

    /// Ensure certificates exist, generating self-signed if needed
    ///
    /// # Errors
    ///
    /// Returns an error if certificate generation fails
    pub async fn ensure_certificates(&self) -> Result<(), TlsError> {
        if self.certificates_exist().await {
            info!("📋 TLS certificates already exist");
            Ok(())
        } else {
            warn!("⚠️  TLS certificates not found, generating self-signed certificates");
            self.generate_self_signed_certificate().await
        }
    }
}

/// Helper to create a TLS acceptor from configuration
pub async fn create_tls_acceptor(config: TlsConfig) -> Result<tokio_rustls::TlsAcceptor, TlsError> {
    let manager = TlsCertificateManager::new(config);

    // Ensure certificates exist
    manager.ensure_certificates().await?;

    // Load TLS configuration
    let server_config = manager.load_tls_config().await?;

    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(server_config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert_eq!(config.cert_path, "certs/songbird.crt");
        assert_eq!(config.key_path, "certs/songbird.key");
        assert!(config.sans.contains(&"localhost".to_string()));
    }

    #[tokio::test]
    async fn test_certificate_manager_creation() {
        let config = TlsConfig::default();
        let _manager = TlsCertificateManager::new(config);
    }

    #[tokio::test]
    async fn test_generate_self_signed_certificate() {
        let config = TlsConfig {
            cert_path: "/tmp/test_songbird.crt".to_string(),
            key_path: "/tmp/test_songbird.key".to_string(),
            sans: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            organization: "Test".to_string(),
            common_name: "test-songbird".to_string(),
        };

        let manager = TlsCertificateManager::new(config.clone());
        let result = manager.generate_self_signed_certificate().await;

        assert!(result.is_ok(), "Certificate generation should succeed");

        // Verify files were created
        assert!(
            tokio::fs::metadata(&config.cert_path).await.is_ok(),
            "Certificate file should exist"
        );
        assert!(tokio::fs::metadata(&config.key_path).await.is_ok(), "Key file should exist");

        // Clean up
        let _ = tokio::fs::remove_file(&config.cert_path).await;
        let _ = tokio::fs::remove_file(&config.key_path).await;
    }
}
