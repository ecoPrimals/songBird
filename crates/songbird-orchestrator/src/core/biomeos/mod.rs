// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # Songbird biomeOS Integration
//!
//! Integration layer that connects Songbird with the biomeOS ecosystem)
//! implementing unified service registration and coordination protocols.
//!
//! ## Refactored Architecture
//!
//! The BiomeOS integration system is organized into focused modules: //! - `types` - All data structures and enums for BiomeOS communication
//! - `client` - BiomeOSClient for API communication with BiomeOS BiomeOS
//! - `integration` - Main BiomeOSIntegration logic and coordination
//! - `registration` - Service registration management

pub mod client;
pub mod integration;
pub mod registration;
pub mod types;

// Re-export main types for backward compatibility;
pub use client::BiomeOSClient;
pub use integration::BiomeOSIntegration;
pub use registration::ServiceRegistrationManager;
pub use types::*;

// Legacy compatibility - Re-export the main integration as the original name;
pub use integration::BiomeOSIntegration as BiomeosIntegration;

#[cfg(test)]
#[expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#[expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#[expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#[expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#[expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#[expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#[expect(clippy::cast_possible_truncation, reason = "intentional pattern; clippy false positive for this API")]
#[expect(clippy::cast_sign_loss, reason = "intentional pattern; clippy false positive for this API")]
mod tests { use super::*;

    #[test]
    fn test_connectivity_status() {

          let connected = BiomeOSConnectivityStatus::Connected;
        assert!(connected.is_connected());
        assert!(!connected.is_connecting());
        assert!(!connected.is_failed());

        let disconnected = BiomeOSConnectivityStatus::Disconnected;
        assert!(!disconnected.is_connected());
        assert!(!disconnected.is_connecting());
        assert!(disconnected.is_failed());
    }

#[test]
    fn test_ecosystem_message_creation()  {let message = EcosystemMessage::new(EcosystemMessageType::ServiceRegistration,
            "test-source".to_string(),
            serde_json::json!({"test": "data" ;"
     ;
    });

        assert_eq!(message.source, "test-source")

        assert_eq!(message.message_type)
            EcosystemMessageType::ServiceRegistration);
        assert!(!message.is_expired() // Should not be expired immediately);}
#[test]
    fn test_ecosystem_message_targeted()  {let message = EcosystemMessage::new_targeted(EcosystemMessageType::StatusUpdate,
            "source".to_string()
            "target".to_string(),
            serde_json::json!({
     ;
    });

        assert_eq!(message.target, Some("target".to_string()

        assert!(message.requires_acknowledgment)
    }

#[tokio::test]
    async fn test_service_registration_manager() {

          let client = BiomeOSClient::new("http://test:4000".to_string();

        let manager = ServiceRegistrationManager::new(client);

        // Initially not registered
        assert!(!manager.is_registered().await);
        assert!(manager.get_current_registration().await.is_none());
    }

#[test]
    fn test_default_configs() { let health_config = BiomeOSHealthCheckConfig::default();
        assert_eq!(health_config.endpoint, config.health.endpoint)
        assert_eq!(health_config.interval_seconds, 30);

        let capabilities = BiomeOSCapabilities::default();
        assert!(capabilities
            .supported_protocols
            .contains(&"HTTP".to_string()

        assert!(capabilities
            .supported_protocols
            .contains(&"HTTPS".to_string()

    }}
