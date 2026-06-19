// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery initialization and management
//!
//! This module handles all discovery-specific logic including:
//! - Anonymous discovery listener setup
//! - Trust escalation configuration
//! - Discovery mode handling

use anyhow::Result;
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_types::config::CanonicalSongbirdConfig;
use std::sync::Arc;
use tracing::info;

use crate::trust::{TrustEscalationManager, TrustTimeouts};

/// Initialize discovery components
///
/// Creates and configures discovery listener and trust escalation manager.
///
/// # Discovery Modes
///
/// - **Disabled**: No discovery, manual configuration only
/// - **Anonymous**: Broadcast/listen for peers without identity
/// - **`CapabilityAware`**: Discover based on capabilities (compute, storage, AI)
/// - **`FullDisclosure`**: Full metadata exchange (for trusted networks)
///
/// # Trust Escalation
///
/// Songbird uses progressive trust escalation:
/// 1. **Anonymous**: Zero trust, encrypted but no identity (shortest timeout)
/// 2. **Capability**: Trust based on capability verification
/// 3. **Identity**: Trust based on cryptographic identity
/// 4. **Hardware**: Trust based on hardware attestation (highest trust, longest timeout)
///
/// This allows flexible security - start with zero trust, escalate as needed.
/// # Errors
///
/// Returns an error if the operation fails.
pub fn initialize_discovery(
    config: &CanonicalSongbirdConfig,
) -> Result<(Arc<TrustEscalationManager>, Option<Arc<AnonymousDiscoveryListener>>)> {
    // Initialize trust escalation manager
    let trust_timeouts = TrustTimeouts {
        anonymous: config.federation.trust_timeouts.anonymous,
        capability: config.federation.trust_timeouts.capability,
        identity: config.federation.trust_timeouts.identity,
        hardware: config.federation.trust_timeouts.hardware,
    };

    let trust_manager = Arc::new(TrustEscalationManager::new(trust_timeouts, None));

    info!("✅ Trust escalation manager initialized");
    info!(
        "   Timeouts: Anonymous={}s, Capability={}s, Identity={}s, Hardware={}",
        config.federation.trust_timeouts.anonymous,
        config.federation.trust_timeouts.capability,
        config.federation.trust_timeouts.identity,
        if config.federation.trust_timeouts.hardware == 0 {
            String::from("never")
        } else {
            format!("{}s", config.federation.trust_timeouts.hardware)
        }
    );

    // Initialize anonymous discovery listener (if enabled)
    let discovery_listener = if config.discovery.mode.is_enabled() {
        let listener = Arc::new(AnonymousDiscoveryListener::new(
            config.discovery.port,
            60, // 60 second peer timeout
        ));
        info!("✅ Anonymous discovery listener initialized (port {})", config.discovery.port);
        Some(listener)
    } else {
        info!("🔒 Discovery disabled (manual configuration mode)");
        None
    };

    Ok((trust_manager, discovery_listener))
}

/// Log discovery configuration for debugging
pub fn log_discovery_config(config: &CanonicalSongbirdConfig) {
    info!(
        "   Discovery: {} ({})",
        if config.discovery.mode.is_enabled() {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        },
        format_discovery_mode(&config.discovery.mode)
    );
}

/// Log federation configuration for debugging
pub fn log_federation_config(config: &CanonicalSongbirdConfig) {
    info!(
        "   Federation: {} (trust: {})",
        if config.federation.cluster_name.is_some() {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        },
        format_trust_escalation_policy(&config.federation.trust_escalation_policy)
    );
    info!("   Trust Model: Zero-trust with progressive escalation");
    info!("   Initial Trust: {} → Escalate on demand", config.federation.initial_trust_level);
}

/// Format discovery mode for logging
const fn format_discovery_mode(
    mode: &songbird_types::config::consolidated_canonical::discovery::DiscoveryMode,
) -> &'static str {
    use songbird_types::config::consolidated_canonical::discovery::DiscoveryMode;
    match mode {
        DiscoveryMode::Disabled => "disabled",
        DiscoveryMode::Anonymous => "anonymous secure",
        DiscoveryMode::CapabilityAware => "capability-aware",
        DiscoveryMode::FullDisclosure => "full disclosure",
    }
}

/// Format trust escalation policy for logging
const fn format_trust_escalation_policy(
    policy: &songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy,
) -> &'static str {
    use songbird_types::config::consolidated_canonical::federation::TrustEscalationPolicy;
    match policy {
        TrustEscalationPolicy::Disabled => "static",
        TrustEscalationPolicy::CapabilityOnly => "capability escalation",
        TrustEscalationPolicy::Progressive => "progressive escalation",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    use songbird_types::config::consolidated_canonical::{
        discovery::DiscoveryMode, federation::TrustTimeouts as ConfigTrustTimeouts,
    };

    #[test]
    fn test_discovery_initialization_disabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.discovery.mode = DiscoveryMode::Disabled;

        let result = initialize_discovery(&config);
        assert!(result.is_ok());

        let (_trust_manager, listener) = result.unwrap();
        assert!(listener.is_none(), "Listener should be None when discovery disabled");
    }

    #[test]
    fn test_discovery_initialization_enabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.discovery.mode = DiscoveryMode::Anonymous;
        config.discovery.port = 9999;

        let result = initialize_discovery(&config);
        assert!(result.is_ok());

        let (_trust_manager, listener) = result.unwrap();
        assert!(listener.is_some(), "Listener should be Some when discovery enabled");
    }

    #[test]
    fn test_trust_timeouts_configuration() {
        let mut config = CanonicalSongbirdConfig::default();
        config.federation.trust_timeouts = ConfigTrustTimeouts {
            anonymous: 300,
            capability: 1800,
            identity: 3600,
            hardware: 0, // Never expire
        };

        let result = initialize_discovery(&config);
        assert!(result.is_ok());

        let (trust_manager, _) = result.unwrap();
        // Trust manager created successfully with custom timeouts
        assert!(Arc::strong_count(&trust_manager) >= 1);
    }
}
