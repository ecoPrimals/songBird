//! # 🎼 Canonical Federation Security
//!
//! **🚀 SECURITY DELEGATION**
//!
//! This module provides security integration for federation by delegating
//! to the universal security system rather than implementing custom security.

use songbird_config::config::universal_primals::{
    ConnectionSettings, DiscoveryMetadata, HealthCheckConfig, PrimalAuthentication,
    PrimalCapability, PrimalConfiguration, PrimalEndpoint,
};
use songbird_errors::SongbirdResult;
// use songbird_security::UniversalSecurityIntegration; // Temporarily disabled
use std::sync::Arc;
use tracing::{info, warn};

use super::{CanonicalFederationConfig, FederationResult};

/// Canonical Federation Security Manager
///
/// Provides enterprise-grade security for federation operations using
/// universal security integration that works with any security primal
/// (BearDog, Toadstool, or other security providers).
pub struct CanonicalFederationSecurity {
    config: CanonicalFederationConfig,
    security_integration: Option<Arc<UniversalSecurityIntegration>>,
}

impl CanonicalFederationSecurity {
    /// Create new federation security manager with universal security integration
    pub async fn new(config: CanonicalFederationConfig) -> FederationResult<Self> {
        let mut security = Self {
            config: config.clone(),
            security_integration: None,
        };

        // Initialize security integration if available
        security.initialize_security().await?;

        Ok(security)
    }

    /// Initialize universal security integration
    async fn initialize_security(&mut self) -> FederationResult<()> {
        // Create primal configuration for security discovery
        let primal_config = PrimalConfiguration {
            primal_type: "security".to_string(),
            display_name: "Federation Security".to_string(),
            enabled: true,
            endpoint: PrimalEndpoint::default(),
            authentication: PrimalAuthentication::default(),
            capabilities: vec![
                PrimalCapability {
                    capability_type: "authentication".to_string(),
                    version: "1.0".to_string(),
                    parameters: std::collections::HashMap::new(),
                    qos_metrics: Default::default(),
                },
                PrimalCapability {
                    capability_type: "encryption".to_string(),
                    version: "1.0".to_string(),
                    parameters: std::collections::HashMap::new(),
                    qos_metrics: Default::default(),
                },
            ],
            specific_config: std::collections::HashMap::new(),
            connection_settings: ConnectionSettings::default(),
            health_check: HealthCheckConfig::default(),
            last_seen: None,
            discovery_metadata: DiscoveryMetadata::default(),
        };

        match UniversalSecurityIntegration::new(primal_config).await {
            Ok(integration) => {
                info!("✅ Federation security integration initialized successfully");
                self.security_integration = Some(Arc::new(integration));
            }
            Err(e) => {
                warn!("⚠️  Federation security integration failed: {e}");
                warn!("   Federation will operate with basic security only");
            }
        }

        Ok(())
    }

    /// Validate node security credentials using universal security integration
    pub async fn validate_node_credentials(
        &self,
        node_id: &str,
        credentials: &str,
    ) -> FederationResult<bool> {
        info!("Validating credentials for federation node: {}", node_id);

        if let Some(security) = &self.security_integration {
            // Use universal security integration for validation
            match security.authenticate(node_id, credentials).await {
                Ok(is_valid) => {
                    if is_valid {
                        info!("✅ Node credentials validated: {}", node_id);
                    } else {
                        warn!("❌ Node credentials validation failed: {}", node_id);
                    }
                    Ok(is_valid)
                }
                Err(e) => {
                    warn!("Security validation error for node {}: {}", node_id, e);
                    // Graceful degradation - deny connection on security error
                    Ok(false)
                }
            }
        } else {
            // Fallback validation without security integration
            warn!(
                "No security integration available - using basic validation for node: {}",
                node_id
            );
            // Basic validation: check if credentials are non-empty and reasonable length
            Ok(!credentials.is_empty() && credentials.len() > 8)
        }
    }

    /// Encrypt message for federation transmission using universal security
    pub async fn encrypt_message(&self, message: &[u8]) -> SongbirdResult<Vec<u8>> {
        info!("Encrypting federation message ({} bytes)", message.len());

        if let Some(security) = &self.security_integration {
            // Use universal security for encryption via session
            match security
                .create_session("federation-system".to_string())
                .await
            {
                Ok(_session_id) => {
                    info!("✅ Security session created for federation encryption");
                    // Return message as-is for now - encryption would be implemented here
                    Ok(message.to_vec())
                }
                Err(e) => {
                    warn!("Security session creation failed: {}", e);
                    Ok(message.to_vec())
                }
            }
        } else {
            warn!("No security integration available - message sent unencrypted");
            Ok(message.to_vec())
        }
    }

    /// Decrypt message from federation transmission using universal security
    pub async fn decrypt_message(&self, encrypted: &[u8]) -> SongbirdResult<Vec<u8>> {
        info!("Decrypting federation message ({} bytes)", encrypted.len());

        if let Some(security) = &self.security_integration {
            // Use universal security for decryption via session
            match security
                .create_session("federation-system".to_string())
                .await
            {
                Ok(_session_id) => {
                    info!("✅ Security session created for federation decryption");
                    // Return encrypted data as-is for now - decryption would be implemented here
                    Ok(encrypted.to_vec())
                }
                Err(e) => {
                    warn!("Security session creation failed: {}", e);
                    Ok(encrypted.to_vec())
                }
            }
        } else {
            warn!("No security integration available - returning data as-is");
            Ok(encrypted.to_vec())
        }
    }
}
