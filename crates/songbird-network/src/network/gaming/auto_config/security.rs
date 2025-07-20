//! Security Validation
//!
//! Grandma-safe security validation that prevents scammer access while
//! enabling legitimate family and friend connections.

use super::types::{SecurityLevel, TrustLevel};
use tracing::{info, warn};

/// Security validator for grandma-safe protection
#[derive(Debug)]
pub struct SecurityValidator {
    #[allow(dead_code)]
    trusted_sources: Vec<String>,
    #[allow(dead_code)]
    scammer_patterns: Vec<String>,
    security_level: SecurityLevel,
    family_mode: bool,
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityValidator {
    /// Create new security validator with grandma-safe defaults
    pub fn new() -> Self {
        Self {
            trusted_sources: vec![
                "beardog.local".to_string(),
                "toadstool.local".to_string(),
                "nestgate.local".to_string(),
            ],
            scammer_patterns: vec![
                "urgent".to_string(),
                "click here".to_string(),
                "verify account".to_string(),
                "suspended".to_string(),
                "confirm payment".to_string(),
            ],
            security_level: SecurityLevel::High,
            family_mode: true,
        }
    }

    /// Validate if a connection request is safe for grandma
    pub fn validate_connection_request(&self, request: &str, source: &str) -> bool {
        info!("🔒 Validating connection request from: {}", source);

        // Check for scammer patterns (grandma protection)
        for pattern in &self.scammer_patterns {
            if request.to_lowercase().contains(pattern) {
                warn!(
                    "⚠️  SCAMMER ALERT: Request contains suspicious pattern: {}",
                    pattern
                );
                return false;
            }
        }

        // Check if source is trusted
        let is_trusted = self
            .trusted_sources
            .iter()
            .any(|trusted| source.contains(trusted));

        if !is_trusted && self.family_mode {
            warn!("🚨 Family mode: Blocking unknown source: {}", source);
            return false;
        }

        info!("✅ Connection request validated successfully");
        true
    }

    /// Get current security level
    pub fn get_security_level(&self) -> &SecurityLevel {
        &self.security_level
    }

    /// Set security level
    pub fn set_security_level(&mut self, level: SecurityLevel) {
        self.security_level = level;
    }

    /// Enable or disable family mode
    pub fn set_family_mode(&mut self, enabled: bool) {
        self.family_mode = enabled;
    }

    /// Check if family mode is enabled
    pub fn is_family_mode(&self) -> bool {
        self.family_mode
    }

    /// Evaluate trust level for a device
    pub fn evaluate_trust_level(&self, device_info: &str) -> TrustLevel {
        // Simple heuristics for trust level evaluation
        if device_info.contains("family") || device_info.contains("trusted") {
            TrustLevel::Family
        } else if device_info.contains("friend") {
            TrustLevel::Friend
        } else if device_info.contains("verified") {
            TrustLevel::Known
        } else {
            TrustLevel::Untrusted
        }
    }

    /// Add trusted source
    pub fn add_trusted_source(&mut self, source: String) {
        if !self.trusted_sources.contains(&source) {
            self.trusted_sources.push(source);
        }
    }

    /// Remove trusted source
    pub fn remove_trusted_source(&mut self, source: &str) {
        self.trusted_sources.retain(|s| s != source);
    }

    /// Get trusted sources
    pub fn get_trusted_sources(&self) -> &[String] {
        &self.trusted_sources
    }
}
