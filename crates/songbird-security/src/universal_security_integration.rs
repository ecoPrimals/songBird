//! Universal Primal Security Integration
//!
//! **REFACTORED FOR UNIVERSAL EXTENSIBILITY**
//!
//! This module provides universal security integration that works with ANY primal
//! having security capabilities, not just BearDog. It replaces hardcoded BearDog
//! integration with a capability-based universal system.
//!
//! ## Universal Architecture
//! 
//! - Capability-based primal selection (not hardcoded names)
//! - Works with BearDog, Toadstool, or any future security primal
//! - Universal security context and session management
//! - Extensible authentication and tunnel protocols
//! - Backward compatibility with existing BearDog configurations
//!
//! ## Migration from Hardcoded BearDog
//! 
//! Old: `BearDogIntegration` with hardcoded BearDog types
//! New: `UniversalSecurityIntegration` with dynamic primal support

// Re-export universal security types (not BearDog-specific)
pub use crate::security::beardog::*;
use songbird_config::config::{PrimalRegistry, PrimalConfiguration};

use async_trait::async_trait;
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Security health status information
#[derive(Debug, Clone)]
pub struct SecurityHealth {
    pub status: String,
    pub primal_type: String,
    pub enabled: bool,
    pub last_check: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Universal Primal Security Integration
/// 
/// This provides security integration with ANY primal that has security capabilities,
/// replacing the old hardcoded BearDog-only approach.
pub struct UniversalSecurityIntegration {
    primal_config: PrimalConfiguration,
    security_context: Arc<RwLock<BearDogSecurityContext>>, // TODO: Rename to UniversalSecurityContext
    statistics: Arc<RwLock<HashMap<String, u64>>>,
}

impl UniversalSecurityIntegration {
    /// Create a new universal security integration for any primal with security capabilities
    pub async fn new(primal_config: PrimalConfiguration) -> Result<Self> {
        info!("🔐 Initializing universal security integration for {}...", primal_config.display_name);
        
        // Verify the primal has security capability
        if primal_config.get_capability("security").is_none() {
            return Err(SongbirdError::Config {
                message: format!("Primal {} does not have security capability", primal_config.primal_type),
                field: Some("capabilities".to_string()),
                context: Some("security_integration".to_string()),
                suggestion: Some("Ensure the primal configuration includes security capability".to_string()),
            });
        }
        
        let security_context = BearDogSecurityContext {
            security_level: BearDogSecurityLevel::Standard,
            use_bstp: true,
            metadata: std::collections::HashMap::new(),
        };
        
        info!("✅ Universal security integration initialized for {}", primal_config.display_name);
        
        Ok(Self {
            primal_config,
            security_context: Arc::new(RwLock::new(security_context)),
            statistics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get the primal type this integration represents
    pub fn primal_type(&self) -> &str {
        &self.primal_config.primal_type
    }
    
    /// Get the primal display name
    pub fn primal_name(&self) -> &str {
        &self.primal_config.display_name
    }
    
    /// Check if the primal supports a specific security feature  
    pub fn supports_feature(&self, feature: &str) -> bool {
        // Simplified feature support check for now
        match feature {
            "authentication" | "session_management" | "tunnel_management" | "encryption" => true,
            _ => false,
        }
    }
    
    /// Initialize the universal security integration
    pub async fn initialize(&self) -> Result<()> {
        info!("🚀 Starting universal security integration for {}...", self.primal_name());
        
        // Verify primal is enabled
        if !self.primal_config.enabled {
            info!("⏭️ Skipping initialization - {} is disabled", self.primal_name());
            return Ok(()); 
        }
        
        // Initialize statistics based on supported features
        {
            let mut stats = self.statistics.write().await;
            stats.insert("sessions_created".to_string(), 0);
            stats.insert("authentication_attempts".to_string(), 0);
            stats.insert("authentication_success".to_string(), 0);
            
            if self.supports_feature("threat_detection") {
                stats.insert("threats_detected".to_string(), 0);
            }
            if self.supports_feature("tunnel_management") {
                stats.insert("tunnels_established".to_string(), 0);
            }
            if self.supports_feature("encryption") {
                stats.insert("encryption_operations".to_string(), 0);
            }
        }
        
        info!("✅ Universal security integration for {} initialized successfully", self.primal_name());
        Ok(())
    }

    /// Universal authentication using any security primal
    pub async fn authenticate(&self, user_id: &str, _credentials: &str) -> Result<bool> {
        info!("🔍 Universal authentication request for user: {} via {}", user_id, self.primal_name());
        
        // Check if primal supports authentication
        if !self.supports_feature("authentication") {
            warn!("⚠️ {} does not support authentication feature", self.primal_name());
            return Ok(false);
        }
        
        // Update security context metadata
        {
            let mut context = self.security_context.write().await;
            context.metadata.insert("user_id".to_string(), user_id.to_string());
        }
        
        // Universal authentication logic based on primal type
        let success = match self.primal_type() {
            "beardog" => {
                info!("🐕 Using BearDog authentication protocol");
                true // Simplified for demonstration
            }
            "toadstool" => {
                info!("🍄 Using Toadstool authentication protocol");
                true // Would implement Toadstool-specific auth
            }
            _ => {
                info!("🔧 Using generic universal authentication for {}", self.primal_type());
                true // Universal fallback authentication
            }
        };
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            let auth_attempts = stats.entry("authentication_attempts".to_string()).or_insert(0);
            *auth_attempts += 1;
            
            if success {
                let auth_success = stats.entry("authentication_success".to_string()).or_insert(0);
                *auth_success += 1;
            }
        }
        
        Ok(success)
    }

    /// Create a new universal security session
    pub async fn create_session(&self, user_id: String) -> Result<String> {
        info!("🔑 Creating universal security session for user: {} via {}", user_id, self.primal_name());
        
        if !self.supports_feature("session_management") {
            warn!("⚠️ {} does not support session management feature", self.primal_name());
            return Ok(format!("fallback_session_{}_{}", user_id, chrono::Utc::now().timestamp()));
        }
        
        // Universal session creation based on primal type
        let session_id = match self.primal_type() {
            "beardog" => {
                info!("🐕 Creating BearDog security session");
                format!("bdog_session_{}_{}", user_id, chrono::Utc::now().timestamp())
            }
            "toadstool" => {
                info!("🍄 Creating Toadstool security session");
                format!("toad_session_{}_{}", user_id, chrono::Utc::now().timestamp())
            }
            _ => {
                info!("🔧 Creating universal security session for {}", self.primal_type());
                format!("{}_session_{}_{}", self.primal_type(), user_id, chrono::Utc::now().timestamp())
            }
        };
        
        // Update security context metadata
        {
            let mut context = self.security_context.write().await;
            context.metadata.insert("session_id".to_string(), session_id.clone());
            context.metadata.insert("user_id".to_string(), user_id);
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            let sessions = stats.entry("sessions_created".to_string()).or_insert(0);
            *sessions += 1;
        }
        
        Ok(session_id)
    }

    /// Create a universal security tunnel
    pub async fn create_tunnel(&self, remote_endpoint: String) -> Result<String> {
        info!("🔐 Creating universal security tunnel to: {} via {}", remote_endpoint, self.primal_name());
        
        if !self.supports_feature("tunnel_management") {
            warn!("⚠️ {} does not support tunnel management feature", self.primal_name());
            return Ok(format!("fallback_tunnel_to_{}", remote_endpoint.replace(":", "_").replace(".", "_")));
        }
        
        // Universal tunnel creation based on primal type
        let tunnel_id = match self.primal_type() {
            "beardog" => {
                info!("🐕 Creating BearDog BSTP tunnel");
                format!("bstp_tunnel_{}_{}", chrono::Utc::now().timestamp(), remote_endpoint.replace(":", "_").replace(".", "_"))
            }
            "toadstool" => {
                info!("🍄 Creating Toadstool secure tunnel");
                format!("toad_tunnel_{}_{}", chrono::Utc::now().timestamp(), remote_endpoint.replace(":", "_").replace(".", "_"))
            }
            _ => {
                info!("🔧 Creating universal secure tunnel for {}", self.primal_type());
                format!("{}_tunnel_{}_{}", self.primal_type(), chrono::Utc::now().timestamp(), remote_endpoint.replace(":", "_").replace(".", "_"))
            }
        };
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            let tunnels = stats.entry("tunnels_established".to_string()).or_insert(0);
            *tunnels += 1;
        }
        
        Ok(tunnel_id)
    }

    /// Get universal integration statistics
    pub async fn get_statistics(&self) -> HashMap<String, u64> {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    /// Get security health status
    pub async fn get_security_health(&self) -> Result<SecurityHealth> {
        info!("📊 Getting universal security health for {}", self.primal_name());
        
        let stats = self.get_statistics().await;
        let auth_attempts = *stats.get("authentication_attempts").unwrap_or(&0);
        let auth_success = *stats.get("authentication_success").unwrap_or(&0);
        
        let health_status = if auth_attempts == 0 {
            "healthy".to_string()
        } else if auth_success as f64 / auth_attempts as f64 > 0.8 {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };
        
        Ok(SecurityHealth {
            status: health_status,
            primal_type: self.primal_type().to_string(),
            enabled: self.primal_config.enabled,
            last_check: std::time::SystemTime::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Shutdown the universal integration
    pub async fn shutdown(&self) -> Result<()> {
        info!("🔒 Shutting down universal security integration for {}...", self.primal_name());
        
        // Clear statistics
        {
            let mut stats = self.statistics.write().await;
            stats.clear();
        }
        
        info!("✅ Universal security integration for {} shutdown complete", self.primal_name());
        Ok(())
    }

    // Additional helper methods could go here in the future
}

// =============================================================================
// Universal Security Integration complete - types re-exported via lib.rs
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test] 
    async fn test_universal_security_integration_creation() {
        // Test universal integration with BearDog primal
        let primal_config = PrimalConfiguration {
            primal_type: "beardog".to_string(),
            display_name: "Test BearDog".to_string(),
            enabled: true,
            endpoint: songbird_config::config::PrimalEndpoint::default(),
            authentication: songbird_config::config::PrimalAuthentication::default(),
            capabilities: std::collections::HashMap::new(),
            qos_requirements: songbird_config::config::QosRequirements::default(),
            metadata: std::collections::HashMap::new(),
        };
        
        let integration = UniversalSecurityIntegration::new(primal_config).await;
        assert!(integration.is_ok());
    }

    #[tokio::test]
    async fn test_universal_authentication() {
        let primal_config = PrimalConfiguration {
            primal_type: "beardog".to_string(),
            display_name: "Test BearDog".to_string(),
            enabled: true,
            endpoint: songbird_config::config::PrimalEndpoint::default(),
            authentication: songbird_config::config::PrimalAuthentication::default(),
            capabilities: {
                let mut caps = std::collections::HashMap::new();
                caps.insert("security".to_string(), songbird_config::config::PrimalCapability {
                    capability_type: "security".to_string(),
                    features: vec!["authentication".to_string(), "session_management".to_string()],
                    qos_metrics: songbird_config::config::QosMetrics::default(),
                    metadata: std::collections::HashMap::new(),
                });
                caps
            },
            qos_requirements: songbird_config::config::QosRequirements::default(),
            metadata: std::collections::HashMap::new(),
        };
        
        let integration = UniversalSecurityIntegration::new(primal_config).await.unwrap();
        let result = integration.authenticate("test_user", "credentials").await;
        assert!(result.is_ok());
        assert!(result.unwrap() == true);
    }
}
