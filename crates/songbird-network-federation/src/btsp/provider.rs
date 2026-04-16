// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Provider Trait and Configuration
//!
//! Defines the interface that all BTSP implementations must provide.

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use std::future::Future;
use std::sync::Arc;
use std::sync::LazyLock;

use super::http_provider::HttpBtspProvider;
#[cfg(feature = "local-btsp")]
use super::local::LocalBtspProvider;
use super::tunnel::{SecurityContext, TunnelHandle, TunnelStatus};
use songbird_types::{SongbirdError, SongbirdResult};

fn default_upa_endpoint_base() -> &'static str {
    static URL: LazyLock<String> = LazyLock::new(|| {
        use songbird_types::constants::LOCALHOST;
        use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;
        songbird_process_env::var("SONGBIRD_UPA_ENDPOINT")
            .unwrap_or_else(|_| format!("http://{LOCALHOST}:{DEFAULT_HTTP_PORT}"))
    });
    URL.as_str()
}

/// Configuration for BTSP provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspConfig {
    /// Enable BTSP encryption
    pub enabled: bool,

    /// `security provider` discovery method (capability-based, not hardcoded)
    pub discovery_method: DiscoveryMethod,

    /// Capability to discover `security provider` service
    pub security_capability: String,

    /// Fallback to local implementation if `security provider` unavailable
    pub local_fallback: bool,

    /// Genetic auth enabled (requires `security provider`)
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
            genetic_auth: false,  // Requires security provider
            key_lineage: false,   // Requires security provider
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
/// The concrete entry point is [`BtspProviderImpl`] (enum dispatch); this trait remains the
/// documented contract for that type.
///
/// Implementations can be:
/// - Local (for testing)
/// - `security provider` (real genetic crypto)
/// - Mock (for unit tests)
pub trait BtspProvider: Send + Sync {
    /// Establish a secure tunnel with peer
    fn establish_tunnel(
        &self,
        peer: &PeerInfo,
    ) -> impl Future<Output = SongbirdResult<TunnelHandle>> + Send;

    /// Encrypt data for transmission through tunnel
    fn encrypt(
        &self,
        data: &[u8],
        context: &SecurityContext,
    ) -> impl Future<Output = SongbirdResult<Vec<u8>>> + Send;

    /// Decrypt data received through tunnel
    fn decrypt(
        &self,
        data: &[u8],
        context: &SecurityContext,
    ) -> impl Future<Output = SongbirdResult<Vec<u8>>> + Send;

    /// Get tunnel status
    fn tunnel_status(
        &self,
        handle: &TunnelHandle,
    ) -> impl Future<Output = SongbirdResult<TunnelStatus>> + Send;

    /// Close tunnel
    fn close_tunnel(
        &self,
        handle: &TunnelHandle,
    ) -> impl Future<Output = SongbirdResult<()>> + Send;

    /// Get provider name (for logging/debugging)
    fn provider_name(&self) -> &str;

    /// Check if provider supports genetic auth
    fn supports_genetic_auth(&self) -> bool;

    /// Check if provider supports key lineage
    fn supports_key_lineage(&self) -> bool;
}

/// Concrete BTSP provider (static enum dispatch; replaces `Arc<dyn BtspProvider>`).
pub enum BtspProviderImpl {
    /// Local AES-GCM test provider ([`LocalBtspProvider`]).
    #[cfg(feature = "local-btsp")]
    Local(LocalBtspProvider),
    /// Remote provider over Unix RPC ([`HttpBtspProvider`]).
    Http(HttpBtspProvider),
}

impl BtspProvider for BtspProviderImpl {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle> {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.establish_tunnel(peer).await,
            Self::Http(p) => p.establish_tunnel(peer).await,
        }
    }

    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.encrypt(data, context).await,
            Self::Http(p) => p.encrypt(data, context).await,
        }
    }

    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.decrypt(data, context).await,
            Self::Http(p) => p.decrypt(data, context).await,
        }
    }

    async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus> {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.tunnel_status(handle).await,
            Self::Http(p) => p.tunnel_status(handle).await,
        }
    }

    async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()> {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.close_tunnel(handle).await,
            Self::Http(p) => p.close_tunnel(handle).await,
        }
    }

    fn provider_name(&self) -> &str {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.provider_name(),
            Self::Http(p) => p.provider_name(),
        }
    }

    fn supports_genetic_auth(&self) -> bool {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.supports_genetic_auth(),
            Self::Http(p) => p.supports_genetic_auth(),
        }
    }

    fn supports_key_lineage(&self) -> bool {
        match self {
            #[cfg(feature = "local-btsp")]
            Self::Local(p) => p.supports_key_lineage(),
            Self::Http(p) => p.supports_key_lineage(),
        }
    }
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
    /// This method discovers `security provider` via capability system at runtime.
    /// If `security provider` is not available and `local_fallback` is enabled, returns
    /// local implementation.
    pub async fn create_provider(&self) -> SongbirdResult<Arc<BtspProviderImpl>> {
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
                    #[cfg(feature = "local-btsp")]
                    {
                        Ok(Arc::new(BtspProviderImpl::Local(
                            crate::btsp::local::LocalBtspProvider::new(),
                        )))
                    }
                    #[cfg(not(feature = "local-btsp"))]
                    {
                        Err(e)
                    }
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
    async fn discover_security_provider(&self) -> SongbirdResult<Arc<BtspProviderImpl>> {
        use tracing::{debug, warn};

        debug!("🔍 Attempting to discover security provider via capability system");

        // Strategy 1: Query local UPA service registry for "security" capability
        if let Some(provider_endpoint) = self.query_local_upa_for_security_provider().await? {
            debug!("✅ Found security provider via local UPA: {}", provider_endpoint);
            return self.connect_to_security_provider(&provider_endpoint).await;
        }

        // Strategy 2: Check environment variable (explicit override)
        if let Ok(endpoint) = songbird_process_env::var("SONGBIRD_SECURITY_PROVIDER_ENDPOINT") {
            debug!("✅ Found security provider via env var: {}", endpoint);
            return self.connect_to_security_provider(&endpoint).await;
        }

        // Strategy 3: Probe security provider port and adjacent ports
        let base_port = songbird_config::defaults::ports::security_provider_port();
        for port in [base_port, base_port + 1, base_port + 2] {
            let endpoint = format!("https://{}:{port}", songbird_types::constants::LOCALHOST);
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
    /// support will be discovered (`security provider`, future alternatives, etc.)
    async fn query_local_upa_for_security_provider(&self) -> SongbirdResult<Option<String>> {
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP client creation failed: {e}")))?;

        let base = default_upa_endpoint_base();
        let url = format!("{base}/api/v1/services/query/security");

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
                                return Ok(Some(format!(
                                    "https://{}:{port}",
                                    songbird_types::constants::LOCALHOST
                                )));
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
    /// not just `security provider`. The provider self-identifies through UPA registration.
    async fn connect_to_security_provider(
        &self,
        endpoint: &str,
    ) -> SongbirdResult<Arc<BtspProviderImpl>> {
        use tracing::info;

        info!("🔗 Connecting to security provider at {}", endpoint);

        // Extract provider name from UPA metadata if available, or default to "security-provider"
        let provider_name = "security-provider".to_string();

        // Create HTTP provider
        let provider = HttpBtspProvider::new(endpoint.to_string(), provider_name)?;

        // Verify connection
        provider.verify_connection().await?;

        info!("✅ Connected to security provider at {}", endpoint);

        Ok(Arc::new(BtspProviderImpl::Http(provider)))
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
