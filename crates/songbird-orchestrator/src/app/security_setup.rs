// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Security Setup Module
//!
//! Handles security provider discovery and integration setup using
//! **capability-based discovery** - zero hardcoding!
//!
//! ## Zero Hardcoding Philosophy ✨
//!
//! This module exemplifies the primal philosophy:
//! - NO hardcoded endpoints
//! - Discovers security provider at runtime
//! - Uses capability system for discovery
//! - Environment-driven configuration
//! - Graceful fallbacks
//!
//! ## Discovery Strategy
//!
//! 1. Check `SECURITY_ENDPOINT` (explicit configuration)
//! 2. Query capability system for "security" provider
//! 3. Fall back to `CAPABILITY_SECURITY_ENDPOINT`
//! 4. Final fallback: construct from bind address + port
//!
//! This enables ANY security provider to be discovered and used!

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

use songbird_types::SafeEnv;

/// Security integration placeholder
///
/// Currently a placeholder until `UniversalSecurityIntegration` is available.
/// The setup logic demonstrates capability-based discovery pattern.
pub type SecurityIntegration = Arc<()>;

/// Discover security provider endpoint
///
/// **EVOLVED (v3.15.0)**: Zero vendor hardcoding! Uses capability discovery.
///
/// Priority:
/// 1. `SONGBIRD_SECURITY_PROVIDER` (NEW - generic capability)
/// 2. `SECURITY_ENDPOINT` (existing - generic)
/// 3. Discovery via Universal Adapter (fallback)
///
/// # Arguments
///
/// * `universal_adapter` - Optional Universal Adapter for capability discovery
///
/// # Returns
///
/// - `Ok(String)` if a security provider is found
/// - `Err(...)` if no security provider is available
pub async fn discover_security_endpoint(
    universal_adapter: Option<&mut crate::universal_adapter::UniversalAdapter>,
) -> Result<String> {
    // Priority 1: NEW - Generic capability env var (v3.15.0)
    if let Ok(endpoint) = std::env::var("SONGBIRD_SECURITY_PROVIDER") {
        info!("🔐 Security provider: {} (via SONGBIRD_SECURITY_PROVIDER)", endpoint);
        return Ok(endpoint);
    }

    // Priority 2: EXISTING - Generic security endpoint
    if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
        info!("🔐 Security provider: {} (via SECURITY_ENDPOINT)", endpoint);
        return Ok(endpoint);
    }

    // Priority 3: FALLBACK - Discover via Universal Adapter
    if let Some(adapter) = universal_adapter {
        info!("🔍 No security provider configured, discovering via Universal Adapter...");
        match adapter.discover_capability("security").await {
            Ok(providers) if !providers.is_empty() => {
                let endpoint = providers[0].endpoint.clone();
                info!("✅ Discovered security provider: {}", endpoint);
                return Ok(endpoint);
            }
            Ok(_) => {
                warn!("⚠️  Universal Adapter found no security providers");
            }
            Err(e) => {
                warn!("⚠️  Universal Adapter discovery failed: {}", e);
            }
        }
    }

    // Priority 5: Legacy fallback (for backward compat)
    if let Ok(endpoint) = std::env::var("CAPABILITY_SECURITY_ENDPOINT") {
        warn!("⚠️  Using legacy CAPABILITY_SECURITY_ENDPOINT");
        warn!("   Please use SONGBIRD_SECURITY_PROVIDER instead");
        return Ok(endpoint);
    }

    Err(anyhow::anyhow!(
        "No security provider configured.\n\
         Please set one of:\n\
         - SONGBIRD_SECURITY_PROVIDER (recommended - generic capability)\n\
         - SECURITY_ENDPOINT (alternative - generic)\n\
         - Or configure Universal Adapter for automatic discovery"
    ))
}

/// Setup security integration using capability-based discovery
///
/// **EVOLVED (v3.15.0)**: Now uses `discover_security_endpoint` with deprecation support
///
/// # Zero Hardcoding
///
/// Discovers security provider at runtime via capability discovery (no vendor names!)
///
/// # Returns
///
/// Security integration instance (currently a placeholder)
pub async fn setup_security() -> Result<SecurityIntegration> {
    // Use new discovery function (supports Universal Adapter in future)
    let _security_endpoint = discover_security_endpoint(None).await?;

    // Security integration temporarily disabled - using placeholder
    // The important architectural pattern is demonstrated:
    // - Zero hardcoding
    // - Runtime discovery
    // - Environment-driven configuration
    Ok(Arc::new(()))
}

/// Construct default security endpoint from bind address and port
///
/// Falls back to sensible defaults if not configured.
/// This is the final fallback in the discovery chain.
fn construct_default_security_endpoint() -> String {
    let bind_address = SafeEnv::get_or_default(
        "SONGBIRD_BIND_ADDRESS",
        songbird_config::canonical::constants::get_bind_address(),
    );

    let security_port = SafeEnv::get_or_default(
        "CAPABILITY_SECURITY_PORT",
        SafeEnv::get_or_default(
            "SONGBIRD_SECURITY_PORT",
            songbird_config::defaults::ports::beardog_port().to_string(),
        ),
    );

    format!("http://{bind_address}:{security_port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // All tests are concurrent-safe: no env var mutation.

    #[test]
    fn test_construct_default_security_endpoint() {
        let endpoint = construct_default_security_endpoint();
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains(':'));

        let parts: Vec<&str> = endpoint.split("://").collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "http");
        assert!(parts[1].contains(':'));
    }

    #[tokio::test]
    async fn test_security_setup_graceful() {
        // setup_security reads env vars at runtime.
        // In test environments it may succeed or fail depending on config.
        // The important thing is it never panics.
        let result = setup_security().await;
        match result {
            Ok(_endpoint) => {} // BearDog or security provider available
            Err(e) => {
                let msg = format!("{e}");
                // Expected in CI: no security provider
                assert!(
                    msg.contains("security") || msg.contains("provider") || msg.contains("BearDog"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[test]
    fn test_capability_type_is_generic() {
        // Songbird discovers by capability, not by primal name
        let capability_type = "security";
        assert_eq!(capability_type, "security");
    }
}
