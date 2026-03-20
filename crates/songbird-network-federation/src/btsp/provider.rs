// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Provider Trait and Configuration
//!
//! Defines the interface that all BTSP implementations must provide.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use std::sync::Arc;

use super::tunnel::{SecurityContext, TunnelHandle, TunnelStatus};
use songbird_types::{SongbirdError, SongbirdResult};

/// Configuration for BTSP provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspConfig {
    /// Enable BTSP encryption
    pub enabled: bool,

    /// `BearDog` discovery method (capability-based, not hardcoded)
    pub discovery_method: DiscoveryMethod,

    /// Capability to discover `BearDog` service
    pub security_capability: String,

    /// Fallback to local implementation if `BearDog` unavailable
    pub local_fallback: bool,

    /// Genetic auth enabled (requires `BearDog`)
    pub genetic_auth: bool,

    /// Key lineage tracking
    pub key_lineage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Discover via capability system (sovereign)
    Capability,
    /// Discover via mDNS on LAN
    Mdns,
    /// Discover via registry
    Registry,
    /// Environment variable (for explicit config)
    Environment,
}

impl Default for BtspConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, enable via env
            discovery_method: DiscoveryMethod::Capability,
            security_capability: "enterprise-security".to_string(),
            local_fallback: true, // Graceful degradation
            genetic_auth: false,  // Requires BearDog
            key_lineage: false,   // Requires BearDog
        }
    }
}

/// Peer information for BTSP tunnel establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer tower ID
    pub id: String,

    /// Peer endpoint
    pub endpoint: String,

    /// Peer public key (if available)
    pub public_key: Option<Vec<u8>>,

    /// Supported protocols
    pub protocols: Vec<String>,
}

/// BTSP Provider trait
///
/// This trait defines the interface for all BTSP implementations.
/// Implementations can be:
/// - Local (for testing)
/// - `BearDog` (real genetic crypto)
/// - Mock (for unit tests)
#[async_trait]
pub trait BtspProvider: Send + Sync {
    /// Establish a secure tunnel with peer
    async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle>;

    /// Encrypt data for transmission through tunnel
    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>>;

    /// Decrypt data received through tunnel
    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>>;

    /// Get tunnel status
    async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus>;

    /// Close tunnel
    async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()>;

    /// Get provider name (for logging/debugging)
    fn provider_name(&self) -> &str;

    /// Check if provider supports genetic auth
    fn supports_genetic_auth(&self) -> bool;

    /// Check if provider supports key lineage
    fn supports_key_lineage(&self) -> bool;
}

/// Factory for creating BTSP providers based on runtime discovery
pub struct BtspProviderFactory {
    config: BtspConfig,
}

impl BtspProviderFactory {
    /// Create a new factory with configuration
    #[must_use]
    pub const fn new(config: BtspConfig) -> Self {
        Self {
            config,
        }
    }

    /// Create BTSP provider based on runtime discovery
    ///
    /// This method discovers `BearDog` via capability system at runtime.
    /// If `BearDog` is not available and `local_fallback` is enabled, returns
    /// local implementation.
    pub async fn create_provider(&self) -> SongbirdResult<Arc<dyn BtspProvider>> {
        if !self.config.enabled {
            return Err(SongbirdError::configuration("BTSP is not enabled"));
        }

        // Try to discover security provider via capability system
        match self.discover_security_provider().await {
            Ok(provider) => {
                tracing::info!("✅ Security provider with BTSP support discovered and connected");
                Ok(provider)
            }
            Err(e) => {
                if self.config.local_fallback {
                    tracing::warn!(
                        "⚠️ Security provider not available ({}), falling back to local BTSP implementation",
                        e
                    );
                    Ok(Arc::new(crate::btsp::local::LocalBtspProvider::new()))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Discover security provider via capability system
    ///
    /// **Modern Idiomatic Rust Pattern**:
    /// - Uses capability-based discovery (no hardcoded primal names)
    /// - Works with ANY primal providing "security" + "btsp" capability
    /// - Graceful degradation if no provider available
    /// - Follows "Each Primal Knows Only Itself" principle
    async fn discover_security_provider(&self) -> SongbirdResult<Arc<dyn BtspProvider>> {
        use tracing::{debug, warn};

        debug!("🔍 Attempting to discover security provider via capability system");

        // Strategy 1: Query local UPA service registry for "security" capability
        if let Some(provider_endpoint) = self.query_local_upa_for_security_provider().await? {
            debug!("✅ Found security provider via local UPA: {}", provider_endpoint);
            return self.connect_to_security_provider(&provider_endpoint).await;
        }

        // Strategy 2: Check environment variable (explicit override)
        if let Ok(endpoint) = std::env::var("SONGBIRD_SECURITY_PROVIDER_ENDPOINT") {
            debug!("✅ Found security provider via env var: {}", endpoint);
            return self.connect_to_security_provider(&endpoint).await;
        }

        // Strategy 3: Try well-known local ports (localhost only for security)
        for port in [9000, 9001, 9002] {
            let endpoint = format!("https://localhost:{port}");
            if self.probe_security_provider_endpoint(&endpoint).await.is_ok() {
                debug!("✅ Found security provider via probe: {}", endpoint);
                return self.connect_to_security_provider(&endpoint).await;
            }
        }

        // No security provider found
        warn!("⚠️ No security provider discovered via any method");
        Err(SongbirdError::service(
            "security",
            "Security provider not available (checked UPA, env, localhost probes)",
        ))
    }

    /// Query local UPA service registry for security provider
    ///
    /// **Capability-Based Discovery**: Queries for "security" capability,
    /// not hardcoded primal name. Any primal providing security with BTSP
    /// support will be discovered (`BearDog`, future alternatives, etc.)
    async fn query_local_upa_for_security_provider(&self) -> SongbirdResult<Option<String>> {
        // Query localhost:8080 (local Songbird UPA)
        // Note: Using HTTP (not HTTPS) for localhost discovery
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP client creation failed: {e}")))?;

        let url = "https://localhost:8080/api/v1/services/query/security";

        match client.get(url).await {
            Ok(response) if response.is_success() => {
                if let Ok(services) = response.json::<Vec<serde_json::Value>>().await {
                    // Find ANY primal with security capability and BTSP support
                    for service in services {
                        // Check capabilities array for "btsp" or "lineage" capability
                        if let Some(capabilities) =
                            service.get("capabilities").and_then(|c| c.as_array())
                        {
                            let has_btsp = capabilities.iter().any(|cap| {
                                cap.get("name").and_then(|n| n.as_str()).is_some_and(|name| {
                                    name == "btsp" || name == "lineage" || name == "birdsong"
                                })
                            });

                            if has_btsp
                                && let Some(port) =
                                    service.get("port").and_then(serde_json::Value::as_u64)
                            {
                                let primal_name = service
                                    .get("primal_name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown");
                                tracing::info!(
                                    "🔍 Discovered security provider '{}' with BTSP support",
                                    primal_name
                                );
                                return Ok(Some(format!("https://localhost:{port}")));
                            }
                        }
                    }
                }
            }
            _ => {
                // UPA not available or no services registered
                tracing::debug!("UPA query failed or no security providers registered");
            }
        }

        Ok(None)
    }

    /// Probe a security provider endpoint to verify it's responsive
    async fn probe_security_provider_endpoint(&self, endpoint: &str) -> SongbirdResult<()> {
        // Note: Using HTTP (not HTTPS) for service discovery
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP client creation failed: {e}")))?;

        // Try to hit a health endpoint
        let url = format!("{endpoint}/health");

        match client.get(&url).await {
            Ok(response) if response.is_success() => Ok(()),
            _ => Err(SongbirdError::service("security", "Endpoint probe failed")),
        }
    }

    /// Connect to security provider at discovered endpoint
    ///
    /// **Capability-Based**: Works with ANY primal implementing the BTSP API,
    /// not just `BearDog`. The provider self-identifies through UPA registration.
    async fn connect_to_security_provider(
        &self,
        endpoint: &str,
    ) -> SongbirdResult<Arc<dyn BtspProvider>> {
        use crate::btsp::http_provider::HttpBtspProvider;
        use tracing::info;

        info!("🔗 Connecting to security provider at {}", endpoint);

        // Extract provider name from UPA metadata if available, or default to "security-provider"
        let provider_name = "security-provider".to_string();

        // Create HTTP provider
        let provider = HttpBtspProvider::new(endpoint.to_string(), provider_name)?;

        // Verify connection
        provider.verify_connection().await?;

        info!("✅ Connected to security provider at {}", endpoint);

        Ok(Arc::new(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_btsp_config_default() {
        let config = BtspConfig::default();
        assert!(!config.enabled);
        assert!(config.local_fallback);
        assert!(!config.genetic_auth);
    }

    #[tokio::test]
    async fn test_factory_creates_local_fallback() {
        let config = BtspConfig {
            enabled: true,
            local_fallback: true,
            ..Default::default()
        };

        let factory = BtspProviderFactory::new(config);
        let provider = factory.create_provider().await.unwrap();

        assert_eq!(provider.provider_name(), "Local");
        assert!(!provider.supports_genetic_auth());
    }
}
