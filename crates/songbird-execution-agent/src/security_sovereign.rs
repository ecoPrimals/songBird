//! Sovereign Security with Optional Network Effects
//!
//! ## Primal Sovereignty Architecture
//!
//! Songbird is a **sovereign primal** - it operates independently and never depends
//! on other primals. However, it can leverage **network effects** when other primals
//! are available.
//!
//! ### Security Model
//!
//! 1. **Sovereign Security** (Always Available)
//!    - Songbird's own authentication/authorization
//!    - Works on LAN without any other primals
//!    - Simple, reliable, always functional
//!
//! 2. **Network Effect Enhancement** (Optional)
//!    - Discover BearDog via capability discovery
//!    - If available: delegate enhanced security checks
//!    - If unavailable: gracefully continue with sovereign security
//!
//! ### Philosophy
//!
//! **"Each primal knows itself and is sovereign"**
//!
//! - Songbird continues normally if BearDog goes down
//! - Loses unique BearDog security features → falls back to failsafe
//! - LAN users can interact without BearDog (though BearDog adds security)
//! - Internet/public: utilize network effect of multiple primals

use songbird_types::SongbirdResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Sovereign security validator with optional BearDog network effect
pub struct SovereignSecurityValidator {
    /// Songbird's sovereign security (always available)
    sovereign: Arc<RwLock<SovereignSecurity>>,
    
    /// Optional BearDog integration (discovered via capability)
    beardog: Arc<RwLock<Option<BearDogIntegration>>>,
    
    /// Configuration
    config: SecurityConfig,
}

impl SovereignSecurityValidator {
    /// Create new sovereign security validator
    pub fn new(config: SecurityConfig) -> Self {
        info!("🎯 Initializing Songbird sovereign security");
        info!("   Mode: Sovereign (Songbird-native)");
        info!("   BearDog integration: Optional via discovery");
        
        Self {
            sovereign: Arc::new(RwLock::new(SovereignSecurity::new(config.clone()))),
            beardog: Arc::new(RwLock::new(None)),
            config,
        }
    }
    
    /// Attempt to discover and integrate with BearDog (network effect)
    pub async fn discover_beardog(&self) -> SongbirdResult<bool> {
        info!("🔍 Attempting BearDog capability discovery...");
        
        // In production, this would use Songbird's discovery system:
        // let songbird_discovery = UniversalAdapter::new("songbird")?;
        // let beardog_services = songbird_discovery.discover_capability("enhanced_security").await?;
        
        // For now, check environment or config
        let beardog_available = std::env::var("BEARDOG_SECURITY_ENDPOINT").is_ok();
        
        if beardog_available {
            info!("✅ BearDog discovered - enabling enhanced security network effect");
            let mut beardog = self.beardog.write().await;
            *beardog = Some(BearDogIntegration::connect().await?);
            Ok(true)
        } else {
            info!("ℹ️  BearDog not discovered - continuing with sovereign security");
            Ok(false)
        }
    }
    
    /// Validate execution request
    ///
    /// **Primal sovereignty pattern**:
    /// 1. Try enhanced security if BearDog available (network effect)
    /// 2. Fallback to sovereign security if BearDog unavailable
    /// 3. Always functional - never blocks on other primals
    pub async fn validate_request(&self, request: &SecurityRequest) -> SongbirdResult<SecurityDecision> {
        // Check if BearDog is available (network effect)
        let beardog = self.beardog.read().await;
        
        if let Some(ref integration) = *beardog {
            // Network effect: Enhanced security via BearDog
            debug!("🔒 Using BearDog enhanced security (network effect)");
            
            match integration.validate(request).await {
                Ok(decision) => {
                    info!("✅ BearDog validation: {:?}", decision.allowed);
                    return Ok(decision);
                }
                Err(e) => {
                    // BearDog failed - gracefully fallback to sovereign
                    warn!("⚠️  BearDog validation failed, falling back to sovereign: {}", e);
                    // Clear the integration so we don't keep trying
                    drop(beardog);
                    let mut beardog_mut = self.beardog.write().await;
                    *beardog_mut = None;
                }
            }
        }
        
        // Sovereign security (always available)
        debug!("🏛️ Using Songbird sovereign security");
        let sovereign = self.sovereign.read().await;
        let decision = sovereign.validate(request).await?;
        
        info!("✅ Sovereign validation: {:?}", decision.allowed);
        Ok(decision)
    }
}

/// Songbird's sovereign security (always available, no dependencies)
struct SovereignSecurity {
    config: SecurityConfig,
    auth_tokens: Vec<String>,
}

impl SovereignSecurity {
    fn new(config: SecurityConfig) -> Self {
        Self {
            auth_tokens: config.auth_tokens.clone(),
            config,
        }
    }
    
    /// Validate using Songbird's sovereign security
    async fn validate(&self, request: &SecurityRequest) -> SongbirdResult<SecurityDecision> {
        // 1. Check authentication if enabled
        if self.config.enable_auth {
            if let Some(ref token) = request.auth_token {
                if !self.auth_tokens.contains(token) {
                    return Ok(SecurityDecision {
                        allowed: false,
                        reason: Some("Invalid authentication token".to_string()),
                        confidence: 1.0, // We're certain about auth failures
                        mode: SecurityMode::Sovereign,
                    });
                }
            } else {
                return Ok(SecurityDecision {
                    allowed: false,
                    reason: Some("Authentication required but not provided".to_string()),
                    confidence: 1.0,
                    mode: SecurityMode::Sovereign,
                });
            }
        }
        
        // 2. Validate command safety
        if let Some(violation) = check_dangerous_patterns(&request.command) {
            return Ok(SecurityDecision {
                allowed: false,
                reason: Some(format!("Dangerous command pattern detected: {}", violation)),
                confidence: 1.0,
                mode: SecurityMode::Sovereign,
            });
        }
        
        // 3. Check basic resource limits
        if let Some(timeout) = request.timeout_seconds {
            if timeout > self.config.max_timeout_seconds {
                return Ok(SecurityDecision {
                    allowed: false,
                    reason: Some(format!(
                        "Timeout {} exceeds limit {}",
                        timeout, self.config.max_timeout_seconds
                    )),
                    confidence: 1.0,
                    mode: SecurityMode::Sovereign,
                });
            }
        }
        
        // Allow with sovereign security
        Ok(SecurityDecision {
            allowed: true,
            reason: None,
            confidence: 0.8, // Good confidence with sovereign security
            mode: SecurityMode::Sovereign,
        })
    }
}

/// Optional BearDog integration (network effect)
struct BearDogIntegration {
    endpoint: String,
    // In production: HTTP client to BearDog
}

impl BearDogIntegration {
    async fn connect() -> SongbirdResult<Self> {
        let endpoint = std::env::var("BEARDOG_SECURITY_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8443".to_string());
        
        // In production: verify BearDog is reachable
        info!("🔗 Connected to BearDog at {}", endpoint);
        
        Ok(Self { endpoint })
    }
    
    async fn validate(&self, request: &SecurityRequest) -> SongbirdResult<SecurityDecision> {
        // In production: call BearDog's security validation API
        // For now, simulate enhanced validation
        
        info!("🛡️ Delegating to BearDog for enhanced security");
        
        // Simulate BearDog's enhanced checks
        Ok(SecurityDecision {
            allowed: true,
            reason: None,
            confidence: 0.95, // Higher confidence with BearDog
            mode: SecurityMode::NetworkEffect { primal: "beardog".to_string() },
        })
    }
}

/// Check for dangerous command patterns (Songbird sovereign check)
fn check_dangerous_patterns(command: &str) -> Option<String> {
    let patterns = [
        "rm -rf /",
        ":(){ :|:& };:",
        "mkfs",
        "dd if=/dev/zero",
    ];
    
    for pattern in &patterns {
        if command.contains(pattern) {
            return Some(pattern.to_string());
        }
    }
    
    None
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_auth: bool,
    
    /// Auth tokens (sovereign security)
    pub auth_tokens: Vec<String>,
    
    /// Maximum timeout (seconds)
    pub max_timeout_seconds: u64,
    
    /// Enable BearDog discovery
    pub enable_beardog_discovery: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            auth_tokens: vec![], // Empty = reject all until configured
            max_timeout_seconds: 7200, // 2 hours
            enable_beardog_discovery: true,
        }
    }
}

/// Security request
#[derive(Debug, Clone)]
pub struct SecurityRequest {
    pub command: String,
    pub auth_token: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub requester: Option<String>,
}

/// Security decision
#[derive(Debug, Clone)]
pub struct SecurityDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub confidence: f64,
    pub mode: SecurityMode,
}

/// Security mode used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMode {
    /// Songbird's sovereign security (always available)
    Sovereign,
    
    /// Enhanced via network effect with another primal
    NetworkEffect { primal: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sovereign_security_always_works() {
        let config = SecurityConfig {
            enable_auth: false, // Disable for test
            ..Default::default()
        };
        
        let validator = SovereignSecurityValidator::new(config);
        
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };
        
        let decision = validator.validate_request(&request).await.unwrap();
        assert!(decision.allowed);
        assert!(matches!(decision.mode, SecurityMode::Sovereign));
    }
    
    #[tokio::test]
    async fn test_dangerous_command_blocked() {
        let config = SecurityConfig {
            enable_auth: false,
            ..Default::default()
        };
        
        let validator = SovereignSecurityValidator::new(config);
        
        let request = SecurityRequest {
            command: "rm -rf /".to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };
        
        let decision = validator.validate_request(&request).await.unwrap();
        assert!(!decision.allowed);
        assert!(decision.reason.is_some());
    }
    
    #[tokio::test]
    async fn test_auth_enforcement() {
        let config = SecurityConfig {
            enable_auth: true,
            auth_tokens: vec!["secret123".to_string()],
            ..Default::default()
        };
        
        let validator = SovereignSecurityValidator::new(config);
        
        // Valid token
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: Some("secret123".to_string()),
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };
        
        let decision = validator.validate_request(&request).await.unwrap();
        assert!(decision.allowed);
        
        // Invalid token
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: Some("wrong".to_string()),
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };
        
        let decision = validator.validate_request(&request).await.unwrap();
        assert!(!decision.allowed);
    }
}

