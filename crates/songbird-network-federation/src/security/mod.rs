// SPDX-License-Identifier: AGPL-3.0-or-later
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

use songbird_types::SongbirdResult;

use crate::security::noop::NoOpSecurityProvider;
use crate::security::production::ProductionSecurityProvider;

#[cfg(any(test, feature = "test-mocks"))]
use crate::security::mock::MockSecurityProvider;

/// Security provider that combines lineage, `BirdSong`, and relay capabilities
///
/// This is the main interface Songbird uses for optional security services.
pub trait SecurityProvider: LineageProvider + BirdSongCrypto + LineageRelay + Send + Sync {
    /// Check if the provider is available and operational
    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send;

    /// Provider version for compatibility checking
    fn version(&self) -> &str;

    /// Graceful shutdown
    fn shutdown(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

/// Concrete security provider selected at runtime (enum dispatch — no `dyn`).
pub enum SecurityProviderImpl {
    /// Production Unix-socket JSON-RPC provider
    Production(ProductionSecurityProvider),
    /// Graceful degradation: operations fail with clear errors
    NoOp(NoOpSecurityProvider),
    /// Test-only fake provider
    #[cfg(any(test, feature = "test-mocks"))]
    Mock(MockSecurityProvider),
}

impl LineageProvider for SecurityProviderImpl {
    async fn generate_lineage(
        &self,
        node_id: &str,
        parent_id: &str,
    ) -> anyhow::Result<LineageChain> {
        match self {
            Self::Production(p) => p.generate_lineage(node_id, parent_id).await,
            Self::NoOp(p) => p.generate_lineage(node_id, parent_id).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.generate_lineage(node_id, parent_id).await,
        }
    }

    async fn verify_lineage(&self, proof: &LineageProof) -> anyhow::Result<bool> {
        match self {
            Self::Production(p) => p.verify_lineage(proof).await,
            Self::NoOp(p) => p.verify_lineage(proof).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.verify_lineage(proof).await,
        }
    }

    async fn get_descendants(&self, root_id: &str) -> anyhow::Result<Vec<String>> {
        match self {
            Self::Production(p) => p.get_descendants(root_id).await,
            Self::NoOp(p) => p.get_descendants(root_id).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.get_descendants(root_id).await,
        }
    }

    async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> anyhow::Result<Option<usize>> {
        match self {
            Self::Production(p) => p.get_lineage_depth(ancestor_id, descendant_id).await,
            Self::NoOp(p) => p.get_lineage_depth(ancestor_id, descendant_id).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.get_lineage_depth(ancestor_id, descendant_id).await,
        }
    }
}

impl BirdSongCrypto for SecurityProviderImpl {
    async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> anyhow::Result<EncryptedBirdSong> {
        match self {
            Self::Production(p) => p.encrypt_for_lineage(payload, lineage_hint).await,
            Self::NoOp(p) => p.encrypt_for_lineage(payload, lineage_hint).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.encrypt_for_lineage(payload, lineage_hint).await,
        }
    }

    async fn decrypt_birdsong(
        &self,
        encrypted: &EncryptedBirdSong,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        match self {
            Self::Production(p) => p.decrypt_birdsong(encrypted).await,
            Self::NoOp(p) => p.decrypt_birdsong(encrypted).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.decrypt_birdsong(encrypted).await,
        }
    }

    async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> anyhow::Result<BroadcastKey> {
        match self {
            Self::Production(p) => p.request_key(lineage_hint, proof).await,
            Self::NoOp(p) => p.request_key(lineage_hint, proof).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.request_key(lineage_hint, proof).await,
        }
    }

    async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> anyhow::Result<Vec<BroadcastKey>> {
        match self {
            Self::Production(p) => p.request_keys_batch(requests).await,
            Self::NoOp(p) => p.request_keys_batch(requests).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.request_keys_batch(requests).await,
        }
    }
}

impl LineageRelay for SecurityProviderImpl {
    async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> anyhow::Result<RelaySession> {
        match self {
            Self::Production(p) => p.offer_relay(requester, target, lineage_proof).await,
            Self::NoOp(p) => p.offer_relay(requester, target, lineage_proof).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.offer_relay(requester, target, lineage_proof).await,
        }
    }

    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        match self {
            Self::Production(p) => p.get_visibility_level(lineage_depth),
            Self::NoOp(p) => p.get_visibility_level(lineage_depth),
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.get_visibility_level(lineage_depth),
        }
    }

    async fn relay_packet(&self, session: &RelaySession, packet: &[u8]) -> anyhow::Result<()> {
        match self {
            Self::Production(p) => p.relay_packet(session, packet).await,
            Self::NoOp(p) => p.relay_packet(session, packet).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.relay_packet(session, packet).await,
        }
    }

    async fn revoke_relay(&self, session_id: &str) -> anyhow::Result<()> {
        match self {
            Self::Production(p) => p.revoke_relay(session_id).await,
            Self::NoOp(p) => p.revoke_relay(session_id).await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.revoke_relay(session_id).await,
        }
    }
}

impl SecurityProvider for SecurityProviderImpl {
    async fn is_available(&self) -> bool {
        match self {
            Self::Production(p) => p.is_available().await,
            Self::NoOp(p) => p.is_available().await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.is_available().await,
        }
    }

    fn version(&self) -> &str {
        match self {
            Self::Production(p) => p.version(),
            Self::NoOp(p) => p.version(),
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.version(),
        }
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        match self {
            Self::Production(p) => p.shutdown().await,
            Self::NoOp(p) => p.shutdown().await,
            #[cfg(any(test, feature = "test-mocks"))]
            Self::Mock(p) => p.shutdown().await,
        }
    }
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
    pub async fn discover() -> SongbirdResult<Option<SecurityProviderImpl>> {
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
    pub fn create_noop() -> SecurityProviderImpl {
        SecurityProviderImpl::NoOp(NoOpSecurityProvider::new())
    }

    /// Create mock provider for testing
    #[cfg(any(test, feature = "test-mocks"))]
    #[must_use]
    pub fn create_mock() -> SecurityProviderImpl {
        SecurityProviderImpl::Mock(MockSecurityProvider::new())
    }

    async fn discover_via_upa() -> SongbirdResult<Option<SecurityProviderImpl>> {
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
                        return Ok(Some(SecurityProviderImpl::Production(provider)));
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

    async fn discover_via_env() -> SongbirdResult<Option<SecurityProviderImpl>> {
        // Capability-based sockets first (SECURITY_*), then legacy BEARDOG_* with migration warning.
        for (env_key, label, legacy) in [
            ("SECURITY_PROVIDER_SOCKET", "SECURITY_PROVIDER_SOCKET", false),
            ("SECURITY_SOCKET", "SECURITY_SOCKET", false),
            ("BEARDOG_SOCKET", "BEARDOG_SOCKET", true),
        ] {
            if let Ok(socket_path) = songbird_process_env::var(env_key) {
                if legacy {
                    tracing::warn!(
                        "Using legacy env var BEARDOG_SOCKET — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first)"
                    );
                }
                tracing::info!("Using security provider socket from {label}: {socket_path}");
                match crate::security::production::ProductionSecurityProvider::new(&socket_path)
                    .await
                {
                    Ok(provider) => {
                        return Ok(Some(SecurityProviderImpl::Production(provider)));
                    }
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
                tracing::warn!(
                    "Using legacy env var BEARDOG_URL — migrate to SECURITY_URL or CAPABILITY_SECURITY_ENDPOINT (capability-first)"
                );
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
                    Ok(provider) => return Ok(Some(SecurityProviderImpl::Production(provider))),
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

    async fn discover_via_wellknown() -> SongbirdResult<Option<SecurityProviderImpl>> {
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
                    Ok(provider) => return Ok(Some(SecurityProviderImpl::Production(provider))),
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::{SecurityProvider, SecurityProviderFactory, SecurityProviderImpl};
    use crate::security::{
        AccessLevel, BirdSongCrypto, LineageChain, LineageHint, LineageProof, LineageProvider,
        LineageRelay, RelaySession,
    };

    #[test]
    fn create_noop_is_noop_variant() {
        assert!(matches!(SecurityProviderFactory::create_noop(), SecurityProviderImpl::NoOp(_)));
    }

    #[test]
    fn create_mock_is_mock_variant() {
        assert!(matches!(SecurityProviderFactory::create_mock(), SecurityProviderImpl::Mock(_)));
    }

    #[tokio::test]
    async fn noop_impl_security_provider_trait_surface() {
        let p = SecurityProviderFactory::create_noop();
        assert_eq!(SecurityProvider::version(&p), "0.0.0-noop");
        assert!(!SecurityProvider::is_available(&p).await);
        SecurityProvider::shutdown(&p).await.unwrap();
        assert_eq!(LineageRelay::get_visibility_level(&p, 0), AccessLevel::FullLineage);
        assert_eq!(LineageRelay::get_visibility_level(&p, 11), AccessLevel::Transport);
    }

    #[tokio::test]
    async fn noop_lineage_provider_verify_lineage_errors() {
        let p = SecurityProviderFactory::create_noop();
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".into(),
                node_id: "n".into(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(LineageProvider::verify_lineage(&p, &proof).await.is_err());
    }

    #[tokio::test]
    async fn noop_generate_lineage_errors() {
        let p = SecurityProviderFactory::create_noop();
        assert!(p.generate_lineage("node-1", "root-0").await.is_err());
    }

    #[tokio::test]
    async fn noop_get_descendants_errors() {
        let p = SecurityProviderFactory::create_noop();
        assert!(p.get_descendants("root-0").await.is_err());
    }

    #[tokio::test]
    async fn noop_get_lineage_depth_errors() {
        let p = SecurityProviderFactory::create_noop();
        assert!(p.get_lineage_depth("a", "b").await.is_err());
    }

    #[tokio::test]
    async fn noop_encrypt_for_lineage_errors() {
        let p = SecurityProviderFactory::create_noop();
        assert!(p.encrypt_for_lineage(b"hello", LineageHint::Universal).await.is_err());
    }

    #[tokio::test]
    async fn noop_decrypt_birdsong_errors() {
        use crate::security::EncryptedBirdSong;
        let p = SecurityProviderFactory::create_noop();
        let encrypted = EncryptedBirdSong {
            version: 1,
            ciphertext: vec![0u8; 32],
            lineage_hint: LineageHint::DirectDescendants,
            timestamp: chrono::Utc::now(),
            signature: vec![0u8; 64],
            genesis_witness: None,
        };
        assert!(p.decrypt_birdsong(&encrypted).await.is_err());
    }

    #[tokio::test]
    async fn noop_request_key_errors() {
        let p = SecurityProviderFactory::create_noop();
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".into(),
                node_id: "n".into(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(p.request_key(&LineageHint::AllDescendants, proof).await.is_err());
    }

    #[tokio::test]
    async fn noop_offer_relay_errors() {
        let p = SecurityProviderFactory::create_noop();
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".into(),
                node_id: "n".into(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(p.offer_relay("req", "target", proof).await.is_err());
    }

    #[tokio::test]
    async fn noop_relay_packet_errors() {
        let p = SecurityProviderFactory::create_noop();
        let session = RelaySession {
            session_id: "sess-1".into(),
            requester_id: "a".into(),
            target_id: "b".into(),
            relay_id: "relay-0".into(),
            access_level: AccessLevel::FullLineage,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        assert!(p.relay_packet(&session, b"data").await.is_err());
    }

    #[tokio::test]
    async fn noop_revoke_relay_errors() {
        let p = SecurityProviderFactory::create_noop();
        assert!(p.revoke_relay("sess-1").await.is_err());
    }

    #[tokio::test]
    async fn discover_returns_none_when_no_provider() {
        // No UPA, no env, no socket — graceful None
        let result = SecurityProviderFactory::discover().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mock_provider_is_available() {
        let p = SecurityProviderFactory::create_mock();
        assert!(SecurityProvider::is_available(&p).await);
    }

    #[tokio::test]
    async fn mock_provider_version() {
        let p = SecurityProviderFactory::create_mock();
        assert!(!SecurityProvider::version(&p).is_empty());
    }

    #[tokio::test]
    async fn mock_provider_shutdown() {
        let p = SecurityProviderFactory::create_mock();
        assert!(SecurityProvider::shutdown(&p).await.is_ok());
    }

    #[tokio::test]
    async fn mock_generate_lineage_succeeds() {
        let p = SecurityProviderFactory::create_mock();
        let chain = p.generate_lineage("child-1", "root-0").await.unwrap();
        assert_eq!(chain.root_id, "root-0");
        assert_eq!(chain.node_id, "child-1");
    }

    #[tokio::test]
    async fn mock_encrypt_decrypt_roundtrip() {
        let p = SecurityProviderFactory::create_mock();
        let encrypted =
            p.encrypt_for_lineage(b"secret payload", LineageHint::Universal).await.unwrap();
        let decrypted = p.decrypt_birdsong(&encrypted).await.unwrap();
        assert_eq!(decrypted.as_deref(), Some(b"secret payload".as_slice()));
    }
}
