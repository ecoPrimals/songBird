//! SSL/TLS termination and certificate management

use songbird_errors::SongbirdError;
use std::path::Path;

use super::config::NetworkConfig;

/// SSL certificate manager
pub struct SslManager {
    config: NetworkConfig,
}

impl SslManager {
    /// Create new SSL manager
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }

    /// Initialize SSL configuration
    pub async fn initialize(&self) -> Result<(), SongbirdError> {
        if !self.config.ssl_termination_enabled {
            return Ok(());
        }

        // Validate SSL certificate directory
        self.validate_cert_directory()?;

        // Check for existing certificates
        if self.certificates_exist()? {
            tracing::info!("SSL certificates found");
            self.validate_certificates().await?;
        } else if self.config.auto_ssl_enabled {
            tracing::info!("Auto-generating SSL certificates");
            self.generate_certificates().await?;
        } else {
            return Err(SongbirdError::config_field(
                "ssl_certificates",
                "SSL certificates not found and auto-generation is disabled",
            ));
        }

        Ok(())
    }

    /// Validate SSL certificate directory
    fn validate_cert_directory(&self) -> Result<(), SongbirdError> {
        let cert_dir = Path::new(&self.config.ssl_cert_dir);

        if !cert_dir.exists() {
            return Err(SongbirdError::config_field(
                "ssl_cert_dir",
                &format!(
                    "SSL certificate directory does not exist: {}",
                    self.config.ssl_cert_dir
                ),
            ));
        }

        if !cert_dir.is_dir() {
            return Err(SongbirdError::config_field(
                "ssl_cert_dir",
                &format!(
                    "SSL certificate path is not a directory: {}",
                    self.config.ssl_cert_dir
                ),
            ));
        }

        Ok(())
    }

    /// Check if SSL certificates exist
    fn certificates_exist(&self) -> Result<bool, SongbirdError> {
        let cert_path = Path::new(&self.config.ssl_cert_dir).join("cert.pem");
        let key_path = Path::new(&self.config.ssl_cert_dir).join("key.pem");

        Ok(cert_path.exists() && key_path.exists())
    }

    /// Validate existing SSL certificates
    async fn validate_certificates(&self) -> Result<(), SongbirdError> {
        let cert_path = Path::new(&self.config.ssl_cert_dir).join("cert.pem");
        let key_path = Path::new(&self.config.ssl_cert_dir).join("key.pem");

        // Read certificate file
        let cert_content = std::fs::read_to_string(&cert_path).map_err(|e| {
            SongbirdError::io_error(&format!("Failed to read certificate file: {}", e))
        })?;

        // Read private key file
        let key_content = std::fs::read_to_string(&key_path).map_err(|e| {
            SongbirdError::io_error(&format!("Failed to read private key file: {}", e))
        })?;

        // Basic validation - check if files contain PEM data
        if !cert_content.contains("-----BEGIN CERTIFICATE-----") {
            return Err(SongbirdError::config_field(
                "ssl_certificate",
                "Invalid certificate file format",
            ));
        }

        if !key_content.contains("-----BEGIN PRIVATE KEY-----")
            && !key_content.contains("-----BEGIN RSA PRIVATE KEY-----")
        {
            return Err(SongbirdError::config_field(
                "ssl_private_key",
                "Invalid private key file format",
            ));
        }

        tracing::info!("SSL certificates validated successfully");
        Ok(())
    }

    /// Generate self-signed SSL certificates
    async fn generate_certificates(&self) -> Result<(), SongbirdError> {
        use std::process::Command;

        let cert_path = Path::new(&self.config.ssl_cert_dir).join("cert.pem");
        let key_path = Path::new(&self.config.ssl_cert_dir).join("key.pem");

        // Generate self-signed certificate using OpenSSL
        let output = Command::new("openssl")
            .args(&[
                "req",
                "-x509",
                "-newkey",
                "rsa:4096",
                "-keyout",
                key_path.to_str().unwrap(),
                "-out",
                cert_path.to_str().unwrap(),
                "-days",
                "365",
                "-nodes",
                "-subj",
                &format!("/CN={}", self.config.default_domain),
            ])
            .output()
            .map_err(|e| {
                SongbirdError::execution_error(&format!("Failed to execute OpenSSL: {}", e))
            })?;

        if !output.status.success() {
            return Err(SongbirdError::execution_error(&format!(
                "OpenSSL command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        tracing::info!("SSL certificates generated successfully");
        Ok(())
    }

    /// Get SSL certificate information
    pub async fn get_certificate_info(&self) -> Result<CertificateInfo, SongbirdError> {
        if !self.config.ssl_termination_enabled {
            return Err(SongbirdError::config_field(
                "ssl_termination",
                "SSL termination is not enabled",
            ));
        }

        let cert_path = Path::new(&self.config.ssl_cert_dir).join("cert.pem");

        // Use OpenSSL to get certificate information
        let output = std::process::Command::new("openssl")
            .args(&[
                "x509",
                "-in",
                cert_path.to_str().unwrap(),
                "-text",
                "-noout",
            ])
            .output()
            .map_err(|e| {
                SongbirdError::execution_error(&format!("Failed to read certificate info: {}", e))
            })?;

        if !output.status.success() {
            return Err(SongbirdError::execution_error(&format!(
                "Failed to parse certificate: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let cert_text = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_certificate_info(&cert_text))
    }

    /// Parse certificate information from OpenSSL output
    fn parse_certificate_info(&self, cert_text: &str) -> CertificateInfo {
        let mut info = CertificateInfo::default();

        for line in cert_text.lines() {
            let line = line.trim();

            if line.starts_with("Subject:") {
                info.subject = line.replace("Subject:", "").trim().to_string();
            } else if line.starts_with("Issuer:") {
                info.issuer = line.replace("Issuer:", "").trim().to_string();
            } else if line.starts_with("Not Before:") {
                info.valid_from = line.replace("Not Before:", "").trim().to_string();
            } else if line.starts_with("Not After:") {
                info.valid_until = line.replace("Not After:", "").trim().to_string();
            }
        }

        info
    }

    /// Renew SSL certificates
    pub async fn renew_certificates(&self) -> Result<(), SongbirdError> {
        if !self.config.ssl_termination_enabled {
            return Ok(());
        }

        tracing::info!("Renewing SSL certificates");

        // Backup existing certificates
        self.backup_certificates().await?;

        // Generate new certificates
        self.generate_certificates().await?;

        tracing::info!("SSL certificates renewed successfully");
        Ok(())
    }

    /// Backup existing certificates
    async fn backup_certificates(&self) -> Result<(), SongbirdError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cert_path = Path::new(&self.config.ssl_cert_dir).join("cert.pem");
        let key_path = Path::new(&self.config.ssl_cert_dir).join("key.pem");

        let backup_cert_path =
            Path::new(&self.config.ssl_cert_dir).join(format!("cert.{}.bak", timestamp));
        let backup_key_path =
            Path::new(&self.config.ssl_cert_dir).join(format!("key.{}.bak", timestamp));

        if cert_path.exists() {
            std::fs::copy(&cert_path, &backup_cert_path).map_err(|e| {
                SongbirdError::io_error(&format!("Failed to backup certificate: {}", e))
            })?;
        }

        if key_path.exists() {
            std::fs::copy(&key_path, &backup_key_path).map_err(|e| {
                SongbirdError::io_error(&format!("Failed to backup private key: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get SSL configuration summary
    pub fn get_ssl_summary(&self) -> std::collections::HashMap<String, String> {
        let mut summary = std::collections::HashMap::new();

        summary.insert(
            "ssl_termination_enabled".to_string(),
            self.config.ssl_termination_enabled.to_string(),
        );
        summary.insert("ssl_cert_dir".to_string(), self.config.ssl_cert_dir.clone());
        summary.insert(
            "auto_ssl_enabled".to_string(),
            self.config.auto_ssl_enabled.to_string(),
        );
        summary.insert(
            "default_domain".to_string(),
            self.config.default_domain.clone(),
        );

        summary
    }
}

/// SSL certificate information
#[derive(Debug, Clone, Default)]
pub struct CertificateInfo {
    /// Certificate subject
    pub subject: String,
    /// Certificate issuer
    pub issuer: String,
    /// Valid from date
    pub valid_from: String,
    /// Valid until date
    pub valid_until: String,
}

impl CertificateInfo {
    /// Check if certificate is self-signed
    pub fn is_self_signed(&self) -> bool {
        self.subject == self.issuer
    }

    /// Get common name from subject
    pub fn get_common_name(&self) -> Option<String> {
        for part in self.subject.split(',') {
            let part = part.trim();
            if let Some(cn) = part.strip_prefix("CN=") {
                return Some(cn.to_string());
            }
        }
        None
    }
}
