// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider integration traits
//!
//! Songbird defines these traits; a capability-discovered security provider implements them at runtime.
//!
//! **Pattern**: Dependency inversion — no hardcoded primal identity in module names.

pub mod birdsong;
pub mod genesis;
pub mod lineage;
pub mod noop;
pub use noop::NoOpSecurityError;
pub mod production;
pub mod relay;

#[cfg(any(test, feature = "test-mocks"))]
pub mod mock;

pub use birdsong::{BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageHint};
pub use lineage::{LineageChain, LineageLink, LineageProof, LineageProvider};
pub use relay::{AccessLevel, LineageRelay, RelaySession};

/// Security provider that combines lineage, `BirdSong`, and relay capabilities
///
/// This is the main interface Songbird uses for optional security services.
#[async_trait::async_trait]
pub trait SecurityProvider: LineageProvider + BirdSongCrypto + LineageRelay + Send + Sync {
    /// Check if the provider is available and operational
    async fn is_available(&self) -> bool;

    /// Provider version for compatibility checking
    fn version(&self) -> &str;

    /// Graceful shutdown
    async fn shutdown(&self) -> anyhow::Result<()>;
}

/// Factory for discovering security providers
///
/// Supports multiple discovery strategies:
/// 1. UPA — query for "security" capability
/// 2. Environment — `SECURITY_PROVIDER_SOCKET` / legacy `BEARDOG_SOCKET`
/// 3. Development fallback socket (debug builds)
/// 4. Mock provider — testing
pub struct SecurityProviderFactory;

impl SecurityProviderFactory {
    /// Discover a security provider via multiple strategies
    ///
    /// Returns None if no provider is available (graceful degradation)
    pub async fn discover() -> anyhow::Result<Option<Box<dyn SecurityProvider>>> {
        // Strategy 1: Check UPA for "security" capability
        if let Ok(Some(provider)) = Self::discover_via_upa().await {
            tracing::info!("🐻 Security provider discovered via UPA");
            return Ok(Some(provider));
        }

        // Strategy 2: Check environment variable
        if let Ok(Some(provider)) = Self::discover_via_env().await {
            tracing::info!("🐻 Security provider discovered via environment");
            return Ok(Some(provider));
        }

        // Strategy 3: Check well-known port
        if let Ok(Some(provider)) = Self::discover_via_wellknown().await {
            tracing::info!("🐻 Security provider discovered via well-known / fallback socket");
            return Ok(Some(provider));
        }

        tracing::warn!("🐻 Security provider not available, running without encryption");
        Ok(None)
    }

    /// Create no-op provider when no security provider is available
    ///
    /// This is NOT a mock - it returns clear errors for all operations.
    /// Use in production for graceful degradation when security features are optional.
    #[must_use]
    pub fn create_noop() -> Box<dyn SecurityProvider> {
        use crate::security::noop::NoOpSecurityProvider;
        Box::new(NoOpSecurityProvider::new())
    }

    /// Create mock provider for testing
    #[cfg(any(test, feature = "test-mocks"))]
    #[must_use]
    pub fn create_mock() -> Box<dyn SecurityProvider> {
        use crate::security::mock::MockSecurityProvider;
        Box::new(MockSecurityProvider::new())
    }

    async fn discover_via_upa() -> anyhow::Result<Option<Box<dyn SecurityProvider>>> {
        use songbird_config::discovery_helpers::discover_primal;
        use songbird_types::CanonicalPrimalType;

        // Query capability registry for "security" capability
        if let Ok(endpoint) = discover_primal(CanonicalPrimalType::Security).await {
            tracing::info!(
                "Discovered security provider via capability discovery at: {}",
                endpoint.url
            );

            // Extract Unix socket path from URL
            if let Some(socket_path) = endpoint.url.strip_prefix("unix://") {
                match crate::security::production::ProductionSecurityProvider::new(socket_path)
                    .await
                {
                    Ok(provider) => {
                        tracing::info!(
                            "✅ Connected to security provider via Unix socket: {}",
                            socket_path
                        );
                        return Ok(Some(Box::new(provider)));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to discovered security provider: {}", e);
                    }
                }
            } else {
                tracing::warn!("Security endpoint is not a Unix socket URL: {}", endpoint.url);
            }
        }

        Ok(None)
    }

    async fn discover_via_env() -> anyhow::Result<Option<Box<dyn SecurityProvider>>> {
        // Capability-based sockets first (SECURITY_*), then legacy BEARDOG_* with migration warning.
        for (env_key, label, legacy) in [
            ("SECURITY_PROVIDER_SOCKET", "SECURITY_PROVIDER_SOCKET", false),
            ("SECURITY_SOCKET", "SECURITY_SOCKET", false),
            ("BEARDOG_SOCKET", "BEARDOG_SOCKET", true),
        ] {
            if let Ok(socket_path) = songbird_process_env::var(env_key) {
                if legacy {
                    tracing::warn!(
                        "Using legacy env var BEARDOG_SOCKET — migrate to SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET"
                    );
                }
                tracing::info!("Using security provider socket from {label}: {socket_path}");
                match crate::security::production::ProductionSecurityProvider::new(&socket_path)
                    .await
                {
                    Ok(provider) => return Ok(Some(Box::new(provider))),
                    Err(e) => tracing::warn!("Failed to connect to {label}: {e}"),
                }
            }
        }

        // URL-based env vars: SECURITY_URL first, then legacy BEARDOG_URL
        let url_result = match (
            songbird_process_env::var("SECURITY_URL"),
            songbird_process_env::var("BEARDOG_URL"),
        ) {
            (Ok(url), _) => Some(url),
            (Err(_), Ok(url)) => {
                tracing::warn!("Using legacy env var BEARDOG_URL — migrate to SECURITY_URL");
                Some(url)
            }
            (Err(_), Err(_)) => None,
        };

        if let Some(url) = url_result {
            tracing::info!("Found security provider via environment at: {}", url);

            // Try to extract Unix socket path from URL
            if let Some(socket_path) = url.strip_prefix("unix://") {
                match crate::security::production::ProductionSecurityProvider::new(socket_path)
                    .await
                {
                    Ok(provider) => return Ok(Some(Box::new(provider))),
                    Err(e) => tracing::warn!("Failed to connect via URL: {}", e),
                }
            } else {
                tracing::warn!("Security URL is not a Unix socket URL: {}", url);
                tracing::warn!(
                    "Set SECURITY_PROVIDER_SOCKET (or legacy BEARDOG_SOCKET), SECURITY_SOCKET, or use unix:// URLs"
                );
            }
        }

        Ok(None)
    }

    async fn discover_via_wellknown() -> anyhow::Result<Option<Box<dyn SecurityProvider>>> {
        // Development fallback: common legacy socket name (only in debug builds)
        #[cfg(debug_assertions)]
        {
            let default_socket = std::env::temp_dir().join("security.sock");
            if default_socket.exists() {
                tracing::warn!(
                    "Using development fallback socket for security provider: {}",
                    default_socket.display()
                );
                tracing::warn!(
                    "Set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy BEARDOG_SOCKET) for production"
                );
                match crate::security::production::ProductionSecurityProvider::new(default_socket)
                    .await
                {
                    Ok(provider) => return Ok(Some(Box::new(provider))),
                    Err(e) => tracing::warn!("Failed to connect to default socket: {}", e),
                }
            }
        }

        #[cfg(not(debug_assertions))]
        {
            tracing::error!(
                "Security provider not found. Set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy: BEARDOG_SOCKET)"
            );
        }

        Ok(None)
    }
}
