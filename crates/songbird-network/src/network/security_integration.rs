//! # Universal Security Integration - CANONICAL MODERNIZED
//!
//! This module provides universal security integration that works with ANY primal
//! through the universal adapter pattern. Primals only need to know themselves - no hardcoded
//! dependencies on other primals required.
//!
//! ## Universal Architecture
//!
//! - Zero hardcoded primal names or types
//! - Works with ANY security-capable primal
//! - Universal security context and session management
//! - Extensible authentication and tunnel protocols
//! - Backward compatibility with existing configurations
//!
//! ## Migration from Hardcoded Integration
//!
//! Old: Hardcoded integration with specific primal types
//! New: Universal integration with dynamic primal support

use serde::{Deserialize, Serialize};
use songbird_config::SongbirdConfig;
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Universal security capability enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityCapability {
    Authentication,
    Authorization,
    Encryption,
    KeyManagement,
    AuditLogging,
    AccessControl,
}

/// Universal Security Context (replaces hardcoded security contexts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityContext {
    pub security_level: SecurityLevel,
    pub use_secure_tunnels: bool, // Works with any tunnel protocol
    pub metadata: HashMap<String, String>,
}

/// Universal Security Level (replaces hardcoded security levels)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Basic,
    Standard,
    High,
    Maximum,
}

/// Security health status information
#[derive(Debug, Clone)]
pub struct SecurityHealth {
    pub status: String,
    pub primal_type: String,
    pub enabled: bool,
    pub last_check: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Universal Security Integration Manager
///
/// This manager provides universal security integration without hardcoded
/// knowledge of specific security providers.
pub struct UniversalSecurityIntegration {
    config: SongbirdConfig,
    security_providers: Arc<RwLock<HashMap<String, SecurityProviderInfo>>>,
    context: UniversalSecurityContext,
}

/// Security provider information
#[derive(Debug, Clone)]
pub struct SecurityProviderInfo {
    pub provider_id: String,
    pub capabilities: Vec<SecurityCapability>,
    pub endpoint: String,
    pub health: SecurityHealth,
}

impl UniversalSecurityIntegration {
    /// Create new universal security integration
    pub fn new(config: SongbirdConfig) -> Self {
        let context = UniversalSecurityContext {
            security_level: SecurityLevel::Standard,
            use_secure_tunnels: true,
            metadata: HashMap::new(),
        };

        Self {
            config,
            security_providers: Arc::new(RwLock::new(HashMap::new())),
            context,
        }
    }

    /// Register a security provider
    pub async fn register_security_provider(&self, provider: SecurityProviderInfo) -> Result<()> {
        let mut providers = self.security_providers.write().await;
        info!("Registering security provider: {}", provider.provider_id);
        providers.insert(provider.provider_id.clone(), provider);
        Ok(())
    }

    /// Get available security providers
    pub async fn get_security_providers(&self) -> Result<Vec<SecurityProviderInfo>> {
        let providers = self.security_providers.read().await;
        Ok(providers.values().cloned().collect())
    }

    /// Check security health across all providers
    pub async fn check_security_health(&self) -> Result<SecurityHealth> {
        let providers = self.security_providers.read().await;
        
        if providers.is_empty() {
            warn!("No security providers registered");
            return Ok(SecurityHealth {
                status: "No providers".to_string(),
                primal_type: "universal".to_string(),
                enabled: false,
                last_check: std::time::SystemTime::now(),
                metadata: HashMap::new(),
            });
        }

        // Aggregate health from all providers
        let enabled_count = providers.values().filter(|p| p.health.enabled).count();
        let total_count = providers.len();

        Ok(SecurityHealth {
            status: if enabled_count > 0 { "Healthy" } else { "Degraded" }.to_string(),
            primal_type: "universal".to_string(),
            enabled: enabled_count > 0,
            last_check: std::time::SystemTime::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("enabled_providers".to_string(), enabled_count.to_string());
                meta.insert("total_providers".to_string(), total_count.to_string());
                meta
            },
        })
    }

    /// Get security context
    pub fn get_context(&self) -> &UniversalSecurityContext {
        &self.context
    }

    /// Update security context
    pub fn update_context(&mut self, context: UniversalSecurityContext) {
        self.context = context;
    }

    /// Check if security is enabled
    pub async fn is_security_enabled(&self) -> bool {
        let providers = self.security_providers.read().await;
        providers.values().any(|p| p.health.enabled)
    }

    /// Get providers with specific capability
    pub async fn get_providers_with_capability(&self, capability: SecurityCapability) -> Result<Vec<SecurityProviderInfo>> {
        let providers = self.security_providers.read().await;
        Ok(providers
            .values()
            .filter(|p| p.capabilities.contains(&capability))
            .cloned()
            .collect())
    }
}

impl Default for UniversalSecurityIntegration {
    fn default() -> Self {
        Self::new(SongbirdConfig::default())
    }
}

impl Default for UniversalSecurityContext {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            use_secure_tunnels: true,
            metadata: HashMap::new(),
        }
    }
}
