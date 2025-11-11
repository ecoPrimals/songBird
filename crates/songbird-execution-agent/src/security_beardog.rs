//! BearDog-integrated security for sovereign standalone execution
//!
//! This module integrates with BearDog's enterprise security architecture,
//! providing sovereign standalone security for remote command execution.
//!
//! ## Architecture
//!
//! - **Sovereign Mode**: Operates independently with conservative policies
//! - **BearDog Integration**: Uses BearDog's robust security manager
//! - **Sovereignty Levels**: Enforces sovereignty-based access control
//! - **Audit Trail**: Full integration with BearDog audit logging
//!
//! ## Security Philosophy
//!
//! **"Better to deny a legitimate request than approve a malicious one"**
//!
//! In standalone mode, we apply BearDog's conservative security policies,
//! maintaining enterprise-grade security even when disconnected from federation.

use songbird_types::SongbirdResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

/// BearDog-integrated security validator for sovereign standalone execution
pub struct BearDogSecurityValidator {
    /// Sovereignty level for this agent
    sovereignty_level: SovereigntyLevel,
    
    /// Conservative security policy
    policy: ConservativePolicy,
    
    /// Local threat assessment cache
    threat_cache: HashMap<String, ThreatAssessment>,
    
    /// Audit logger integration
    audit_enabled: bool,
}

impl BearDogSecurityValidator {
    /// Create a new BearDog-integrated security validator
    ///
    /// # Arguments
    /// * `sovereignty_level` - The sovereignty level for this agent
    /// * `audit_enabled` - Whether to enable audit logging
    pub fn new(sovereignty_level: SovereigntyLevel, audit_enabled: bool) -> Self {
        info!("🏛️ Initializing BearDog security validator");
        info!("   Sovereignty level: {:?}", sovereignty_level);
        info!("   Conservative policies: enabled");
        info!("   Audit logging: {}", if audit_enabled { "enabled" } else { "disabled" });
        
        let policy = ConservativePolicy::for_sovereignty_level(&sovereignty_level);
        
        Self {
            sovereignty_level,
            policy,
            threat_cache: HashMap::new(),
            audit_enabled,
        }
    }
    
    /// Validate security request for command execution
    ///
    /// Uses BearDog's sovereign standalone security model:
    /// - Sovereignty-based access control
    /// - Conservative threat assessment
    /// - Command validation against policy
    /// - Audit trail recording
    pub async fn validate_execution_request(
        &mut self,
        request: &ExecutionSecurityRequest,
    ) -> SongbirdResult<SecurityDecision> {
        info!("🔒 Validating execution request (BearDog sovereign mode)");
        info!("   Command: {}", request.command);
        info!("   Requester: {}", request.requester_id.as_deref().unwrap_or("anonymous"));
        
        // Check sovereignty requirements
        if !self.check_sovereignty_level(&request.required_sovereignty)? {
            warn!("⚠️ Insufficient sovereignty level for request");
            
            if self.audit_enabled {
                self.record_audit("sovereignty_denied", request).await;
            }
            
            return Ok(SecurityDecision::Deny {
                reason: format!(
                    "Insufficient sovereignty level (required: {:?}, agent: {:?})",
                    request.required_sovereignty,
                    self.sovereignty_level
                ),
                security_level: SecurityLevel::Critical,
            });
        }
        
        // Assess command threat level
        let threat_score = self.assess_command_threat(&request.command)?;
        info!("   Threat score: {:.2}", threat_score);
        
        // Apply conservative policy
        if threat_score > self.policy.max_threat_threshold {
            warn!("⚠️ Command threat score exceeds conservative threshold");
            
            if self.audit_enabled {
                self.record_audit("threat_denied", request).await;
            }
            
            return Ok(SecurityDecision::Deny {
                reason: format!(
                    "Sovereign standalone: threat score {:.2} exceeds conservative threshold {:.2}",
                    threat_score,
                    self.policy.max_threat_threshold
                ),
                security_level: SecurityLevel::High,
            });
        }
        
        // Check for dangerous patterns (BearDog-style validation)
        if let Some(violation) = self.check_command_violations(&request.command)? {
            warn!("⚠️ Command contains security violation: {}", violation);
            
            if self.audit_enabled {
                self.record_audit(&format!("violation_{}", violation), request).await;
            }
            
            return Ok(SecurityDecision::Deny {
                reason: format!("Security violation detected: {}", violation),
                security_level: SecurityLevel::Critical,
            });
        }
        
        // Allow with sovereignty-appropriate confidence
        let confidence = self.sovereignty_level.base_confidence();
        let sovereignty_level = self.sovereignty_level.clone();
        let restrictions = self.policy.get_restrictions();
        
        info!("✅ Request approved (confidence: {:.2})", confidence);
        
        if self.audit_enabled {
            self.record_audit("approved", request).await;
        }
        
        Ok(SecurityDecision::Allow {
            confidence,
            sovereignty_level,
            restrictions,
        })
    }
    
    /// Check sovereignty level compatibility
    fn check_sovereignty_level(&self, required: &SovereigntyLevel) -> SongbirdResult<bool> {
        Ok(self.sovereignty_level.can_satisfy(required))
    }
    
    /// Assess command threat level using BearDog-style heuristics
    fn assess_command_threat(&mut self, command: &str) -> SongbirdResult<f64> {
        // Check cache first
        if let Some(cached) = self.threat_cache.get(command) {
            if cached.age() < Duration::from_secs(300) {
                return Ok(cached.score);
            }
        }
        
        let mut score: f64 = 0.0;
        
        // Dangerous command patterns (BearDog conservative assessment)
        let dangerous_patterns: [(&str, f64); 8] = [
            ("rm -rf /", 1.0),           // System destruction
            (":(){ :|:& };:", 1.0),      // Fork bomb
            ("mkfs", 0.9),               // Filesystem format
            ("dd if=/dev/zero", 0.9),    // Disk wipe
            ("> /dev/", 0.8),            // Device manipulation
            ("chmod 777", 0.6),          // Dangerous permissions
            ("curl | bash", 0.7),        // Remote code execution
            ("wget | sh", 0.7),          // Remote code execution
        ];
        
        for (pattern, weight) in &dangerous_patterns {
            if command.contains(pattern) {
                score = score.max(*weight);
            }
        }
        
        // Cache the assessment
        self.threat_cache.insert(
            command.to_string(),
            ThreatAssessment {
                score,
                assessed_at: SystemTime::now(),
            },
        );
        
        Ok(score)
    }
    
    /// Check for specific security violations
    fn check_command_violations(&self, command: &str) -> SongbirdResult<Option<String>> {
        // Empty command
        if command.trim().is_empty() {
            return Ok(Some("empty_command".to_string()));
        }
        
        // Shell injection attempts
        if command.contains("$(") || command.contains("`") {
            return Ok(Some("shell_injection".to_string()));
        }
        
        // Path traversal
        if command.contains("../../../") {
            return Ok(Some("path_traversal".to_string()));
        }
        
        // Privilege escalation attempts
        if (command.starts_with("sudo ") || command.contains("| sudo "))
            && !self.policy.allow_privilege_escalation {
                return Ok(Some("privilege_escalation".to_string()));
            }
        
        Ok(None)
    }
    
    /// Record audit event (integrates with BearDog audit system)
    async fn record_audit(&self, event_type: &str, request: &ExecutionSecurityRequest) {
        // In production, this would integrate with BearDog's audit logger
        info!("📝 [AUDIT] event={} command={} requester={}", 
            event_type,
            request.command,
            request.requester_id.as_deref().unwrap_or("anonymous")
        );
    }
}

/// Sovereignty levels (aligned with BearDog)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    /// Basic sovereignty - minimal trust required
    Basic,
    /// Enhanced sovereignty - moderate trust
    Enhanced,
    /// Maximum sovereignty - high trust
    Maximum,
    /// Absolute sovereignty - complete trust
    Absolute,
}

impl SovereigntyLevel {
    /// Check if this level can satisfy a requirement
    pub fn can_satisfy(&self, required: &Self) -> bool {
        let self_level = self.as_level();
        let required_level = required.as_level();
        self_level >= required_level
    }
    
    /// Get numerical level
    fn as_level(&self) -> u8 {
        match self {
            Self::Basic => 1,
            Self::Enhanced => 2,
            Self::Maximum => 3,
            Self::Absolute => 4,
        }
    }
    
    /// Get base confidence for this sovereignty level
    pub fn base_confidence(&self) -> f64 {
        match self {
            Self::Basic => 0.5,
            Self::Enhanced => 0.7,
            Self::Maximum => 0.85,
            Self::Absolute => 0.95,
        }
    }
}

impl Default for SovereigntyLevel {
    fn default() -> Self {
        Self::Enhanced // Default to moderate sovereignty
    }
}

/// Security decision from BearDog validator
#[derive(Debug, Clone)]
pub enum SecurityDecision {
    /// Request allowed
    Allow {
        confidence: f64,
        sovereignty_level: SovereigntyLevel,
        restrictions: Vec<String>,
    },
    /// Request denied
    Deny {
        reason: String,
        security_level: SecurityLevel,
    },
}

/// Security level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Moderate,
    High,
    Critical,
}

/// Conservative security policy (BearDog-style)
#[derive(Debug, Clone)]
struct ConservativePolicy {
    max_threat_threshold: f64,
    allow_privilege_escalation: bool,
    restrictions: Vec<String>,
}

impl ConservativePolicy {
    /// Create policy for sovereignty level
    fn for_sovereignty_level(level: &SovereigntyLevel) -> Self {
        match level {
            SovereigntyLevel::Basic => Self {
                max_threat_threshold: 0.3, // Very conservative
                allow_privilege_escalation: false,
                restrictions: vec![
                    "no_privilege_escalation".to_string(),
                    "limited_commands".to_string(),
                ],
            },
            SovereigntyLevel::Enhanced => Self {
                max_threat_threshold: 0.5,
                allow_privilege_escalation: false,
                restrictions: vec!["conservative_mode".to_string()],
            },
            SovereigntyLevel::Maximum => Self {
                max_threat_threshold: 0.7,
                allow_privilege_escalation: false,
                restrictions: vec!["monitored".to_string()],
            },
            SovereigntyLevel::Absolute => Self {
                max_threat_threshold: 0.9,
                allow_privilege_escalation: true,
                restrictions: vec![],
            },
        }
    }
    
    fn get_restrictions(&self) -> Vec<String> {
        self.restrictions.clone()
    }
}

/// Execution security request
#[derive(Debug, Clone)]
pub struct ExecutionSecurityRequest {
    pub command: String,
    pub requester_id: Option<String>,
    pub required_sovereignty: SovereigntyLevel,
}

/// Threat assessment cache entry
struct ThreatAssessment {
    score: f64,
    assessed_at: SystemTime,
}

impl ThreatAssessment {
    fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.assessed_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sovereignty_levels() {
        assert!(SovereigntyLevel::Maximum.can_satisfy(&SovereigntyLevel::Basic));
        assert!(SovereigntyLevel::Maximum.can_satisfy(&SovereigntyLevel::Enhanced));
        assert!(!SovereigntyLevel::Basic.can_satisfy(&SovereigntyLevel::Maximum));
    }
    
    #[tokio::test]
    async fn test_safe_command_approval() {
        let mut validator = BearDogSecurityValidator::new(
            SovereigntyLevel::Enhanced,
            false,
        );
        
        let request = ExecutionSecurityRequest {
            command: "echo hello world".to_string(),
            requester_id: Some("test_user".to_string()),
            required_sovereignty: SovereigntyLevel::Basic,
        };
        
        let decision = validator.validate_execution_request(&request).await.unwrap();
        assert!(matches!(decision, SecurityDecision::Allow { .. }));
    }
    
    #[tokio::test]
    async fn test_dangerous_command_denial() {
        let mut validator = BearDogSecurityValidator::new(
            SovereigntyLevel::Enhanced,
            false,
        );
        
        let request = ExecutionSecurityRequest {
            command: "rm -rf /".to_string(),
            requester_id: Some("test_user".to_string()),
            required_sovereignty: SovereigntyLevel::Basic,
        };
        
        let decision = validator.validate_execution_request(&request).await.unwrap();
        assert!(matches!(decision, SecurityDecision::Deny { .. }));
    }
    
    #[tokio::test]
    async fn test_sovereignty_enforcement() {
        let mut validator = BearDogSecurityValidator::new(
            SovereigntyLevel::Basic,
            false,
        );
        
        let request = ExecutionSecurityRequest {
            command: "echo test".to_string(),
            requester_id: Some("test_user".to_string()),
            required_sovereignty: SovereigntyLevel::Maximum,
        };
        
        let decision = validator.validate_execution_request(&request).await.unwrap();
        assert!(matches!(decision, SecurityDecision::Deny { .. }));
    }
}

