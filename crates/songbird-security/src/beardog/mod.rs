//! BearDog Security Integration Module
//! 
//! This module provides comprehensive BearDog security integration
//! with modular components for different aspects of the security system.

pub mod client;
pub mod genetics;
pub mod tunnel;
pub mod threat_detection;
pub mod zero_trust;
pub mod encryption;
pub mod audit;
pub mod compliance;
pub mod types;

// Re-export main types for convenience
pub use client::*;
pub use genetics::*;
pub use tunnel::*;
pub use threat_detection::*;
pub use zero_trust::*;
pub use encryption::*;
pub use audit::*;
pub use compliance::*;
pub use types::*;

/// Re-export for convenience
pub use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent::new(SecurityEventType::Authentication, "test-source".to_string());
        assert_eq!(event.event_type, SecurityEventType::Authentication);
        assert_eq!(event.source, "test-source");
        assert!(!event.is_critical());
    }

    #[test]
    fn test_threat_assessment() {
        let assessment = ThreatAssessment {
            assessment_id: "test".to_string(),
            target: "test-target".to_string(),
            risk_score: 0.8,
            threat_level: ThreatLevel::High,
            indicators: Vec::new(),
            recommended_actions: Vec::new(),
            assessment_time: chrono::Utc::now(),
            confidence: 0.9,
        };

        assert!(assessment.is_high_risk());
        assert!(assessment.requires_immediate_action());
    }

    #[test]
    fn test_authentication_context() {
        let context = AuthenticationContext {
            user_id: "test-user".to_string(),
            authentication_method: AuthMethod::MultiFactorAuth,
            session_id: "session-123".to_string(),
            source_ip: "192.168.1.1".to_string(),
            user_agent: Some("test-agent".to_string()),
            timestamp: chrono::Utc::now(),
            mfa_verified: true,
            risk_score: 0.2,
        };

        assert!(context.is_secure());
        assert!(!context.needs_additional_verification());
    }

    #[tokio::test]
    async fn test_beardog_integration() {
        let config = BeardogConfig::default();
        let integration = BeardogIntegration::new(config);
        let stats = integration.get_statistics().await;
        assert_eq!(stats.pending_events, 0);
    }
} 