use crate::security::providers::AuthenticationProvider;
use crate::security::types::{SecurityConfig, SubjectType};
use crate::security::universal_security_provider::{
    FallbackSecurityProvider, UniversalSecurityProvider,
};
use crate::test_types::*;
use songbird_universal_primals::{
    registry::UniversalPrimalRegistry,
    traits::{PrimalCapability, PrimalContext},
};
use std::collections::HashMap;
use std::sync::Arc;

/// Example of proper security implementation using Universal Security Provider
///
/// This replaces the previous mock implementations with a real example showing how to:
/// 1. Set up the universal primal registry
/// 2. Configure security providers to use capability-based discovery
/// 3. Provide fallback implementations for standalone operation
///
/// ## Usage Pattern for Integration
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use songbird_universal_primals::registry::UniversalPrimalRegistry;
/// # use songbird_security::{UniversalSecurityProvider, SecurityConfig};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // 1. Create universal primal registry (shared across Songbird)
/// let primal_registry = Arc::new(UniversalPrimalRegistry::new());
///
/// // 2. Configure security settings
/// let security_config = SecurityConfig::default();
///
/// // 3. Create universal security provider
/// let security_provider = UniversalSecurityProvider::new(
///     primal_registry.clone(),
///     security_config
/// );
///
/// // 4. The provider will automatically:
/// //    - Discover BearDog if available at ../beardog
/// //    - Use any other security primals with appropriate capabilities
/// //    - Fall back to standalone security when no primals available
///
/// // 5. Use standard authentication interface
/// let auth_result = security_provider.authenticate("user", "password").await?;
/// println!("Authentication successful: {:?}", auth_result);
/// # Ok(())
/// # }
/// ```
pub async fn setup_universal_security_example() -> Arc<UniversalSecurityProvider> {
    // Create the universal primal registry
    let primal_registry = Arc::new(UniversalPrimalRegistry::new());

    // Configure security with reasonable defaults
    let security_config = SecurityConfig::default();

    // Create universal security provider
    let universal_provider = Arc::new(UniversalSecurityProvider::new(
        primal_registry,
        security_config,
    ));

    // Add some fallback users for standalone testing
    // Note: In real usage, fallback users would be configured via environment/config
    // This is just for test setup demonstration
    tracing::info!(
        "Universal security provider configured with BearDog integration and fallback support"
    );

    universal_provider
}

/// Example of how BearDog (or any security primal) integrates
///
/// This shows the pattern that BearDog follows to advertise its security capabilities
/// through the universal primal system.
///
/// Note: This is just an example - the real BearDog integration happens automatically
/// when BearDog registers itself with the universal primal registry.
pub fn example_security_primal_capabilities() -> Vec<PrimalCapability> {
    vec![
        // BearDog's advanced authentication capabilities
        PrimalCapability::Authentication {
            methods: vec![
                "password".to_string(),
                "oauth2".to_string(),
                "jwt".to_string(),
                "mfa".to_string(),
                "biometric".to_string(),
            ],
        },
        // BearDog's encryption capabilities
        PrimalCapability::Encryption {
            algorithms: vec![
                "aes256gcm".to_string(),
                "chacha20poly1305".to_string(),
                "rsa4096".to_string(),
                "ed25519".to_string(),
            ],
        },
        // BearDog's threat detection
        PrimalCapability::ThreatDetection { ml_enabled: true },
        // BearDog's authorization features
        PrimalCapability::Authorization { rbac_support: true },
        // BearDog's audit logging
        PrimalCapability::Security {
            protocols: vec!["audit_logging".to_string()],
        },
        // BearDog's key management
        PrimalCapability::KeyManagement { hsm_support: true },
    ]
}
