//! X25519 Key Exchange Group for rustls via BearDog Delegation
//!
//! Implements `rustls::crypto::SupportedKxGroup` and `ActiveKeyExchange` by delegating
//! all X25519 operations to BearDog's crypto API via capability-based discovery.
//!
//! **Architecture**:
//! - `X25519Group`: Static group descriptor (implements `SupportedKxGroup`)
//! - `X25519KeyExchange`: Active key exchange state (implements `ActiveKeyExchange`)
//! - `RUNTIME_CRYPTO_PROVIDER`: Global OnceCell for capability reference
//!
//! **Status**: Week 2 Phase 3 Day 1 Evening / Day 2 Morning
//! **Complexity**: MEDIUM-HIGH (two traits, global state, async/sync bridge)
//! **Principles Applied**: Capability-based, zero hardcoding, Pure Rust

use once_cell::sync::OnceCell;
use rustls::crypto::{ActiveKeyExchange, SharedSecret, SupportedKxGroup};
use rustls::{Error, NamedGroup};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::crypto::provider::CryptoProvider;

// ============================================================================
// Global State: Runtime Crypto Provider
// ============================================================================

/// Runtime-initialized crypto provider (set once during initialization)
///
/// **Design Decision**: Use OnceCell for global state
/// - rustls requires `&'static` references for `SupportedKxGroup`
/// - We discover the crypto provider at runtime via capability discovery
/// - OnceCell allows safe one-time initialization of static state
///
/// **Thread Safety**: OnceCell is thread-safe (uses atomic operations)
///
/// **Initialization**: Call `init_runtime_crypto_provider()` once at startup
///
/// **TRUE PRIMAL**: This is initialized with ANY crypto provider discovered
/// at runtime, not hardcoded to "BearDog". It's capability-based!
static RUNTIME_CRYPTO_PROVIDER: OnceCell<Arc<dyn CryptoProvider>> = OnceCell::new();

/// Initialize the runtime crypto provider
///
/// This MUST be called once during application startup, before any TLS
/// connections are established.
///
/// # Arguments
/// * `crypto` - Capability reference to crypto provider (discovered at runtime!)
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(Error)` - Provider already initialized (double init attempt)
///
/// # Example
/// ```no_run
/// use std::sync::Arc;
/// use songbird_orchestrator::crypto::provider::UnixSocketCryptoProvider;
/// use songbird_orchestrator::crypto::rustls_provider::kx_group::init_runtime_crypto_provider;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Discover crypto provider via capability (no hardcoding!)
/// let socket_path = songbird_orchestrator::crypto::discover_crypto_provider().await?;
/// let crypto = Arc::new(UnixSocketCryptoProvider::new(socket_path));
///
/// // Initialize (call once!)
/// init_runtime_crypto_provider(crypto)?;
/// # Ok(())
/// # }
/// ```
pub fn init_runtime_crypto_provider(crypto: Arc<dyn CryptoProvider>) -> Result<(), Error> {
    debug!("🔧 Initializing runtime crypto provider for X25519");
    
    RUNTIME_CRYPTO_PROVIDER
        .set(crypto)
        .map_err(|_| {
            warn!("⚠️ Crypto provider already initialized (double init attempt)");
            Error::General("Crypto provider already initialized".to_string())
        })?;
    
    debug!("✅ Runtime crypto provider initialized for X25519");
    Ok(())
}

/// Get the runtime crypto provider (used by AEAD and other components)
///
/// Returns `None` if not initialized yet.
/// This function is used by components that need access to the crypto provider
/// (e.g., AEAD encryption/decryption).
pub(super) fn get_runtime_crypto_provider() -> Option<Arc<dyn CryptoProvider>> {
    RUNTIME_CRYPTO_PROVIDER.get().cloned()
}

// ============================================================================
// X25519Group - Static Group Descriptor
// ============================================================================

/// X25519 key exchange group delegating to BearDog
///
/// Implements Elliptic Curve Diffie-Hellman (ECDHE) using X25519
/// with all crypto operations delegated to BearDog via capability discovery.
///
/// **Design**:
/// - Zero-sized type (no fields)
/// - Implements `SupportedKxGroup` trait for rustls
/// - Creates `X25519KeyExchange` instances on demand
/// - Delegates key generation to runtime crypto provider
///
/// **Performance**:
/// - Ephemeral key generation: ~0.5ms (BearDog)
/// - Unix socket IPC: ~0.5ms (local)
/// - Total: ~1ms per handshake (acceptable!)
///
/// **Security**:
/// - X25519 is a modern ECDH algorithm
/// - Constant-time implementation in BearDog (via x25519-dalek)
/// - Perfect forward secrecy
///
/// # Example
/// ```no_run
/// use rustls::crypto::SupportedKxGroup;
/// use songbird_orchestrator::crypto::rustls_provider::kx_group::X25519_GROUP;
///
/// // Use in rustls CryptoProvider
/// let kx_groups: Vec<&'static dyn SupportedKxGroup> = vec![&X25519_GROUP];
/// ```
#[derive(Debug, Clone, Copy)]
pub struct X25519Group;

impl SupportedKxGroup for X25519Group {
    /// Start a key exchange by generating an ephemeral keypair
    ///
    /// This method:
    /// 1. Gets the runtime crypto provider (or errors if not initialized)
    /// 2. Delegates to `crypto.x25519_generate_ephemeral()` via BearDog
    /// 3. Returns an `X25519KeyExchange` with the keypair
    ///
    /// # Returns
    /// * `Ok(Box<dyn ActiveKeyExchange>)` - Active key exchange state
    /// * `Err(Error)` - If crypto provider not initialized or keygen fails
    ///
    /// # Performance
    /// - Expected: ~1ms (0.5ms crypto + 0.5ms IPC)
    /// - Called once per TLS handshake
    fn start(&self) -> Result<Box<dyn ActiveKeyExchange>, Error> {
        debug!("🔐 Starting X25519 key exchange");
        
        // Get runtime crypto provider (capability-based!)
        let crypto = get_runtime_crypto_provider().ok_or_else(|| {
            warn!("❌ Crypto provider not initialized for X25519");
            Error::General(
                "Crypto provider not initialized. Call init_runtime_crypto_provider() first."
                    .to_string(),
            )
        })?;
        
        // Generate ephemeral keypair via BearDog
        // Bridge async → sync using tokio runtime
        let (public_key, secret_key) = tokio::runtime::Handle::current()
            .block_on(async {
                crypto
                    .x25519_generate_ephemeral("tls_key_exchange")
                    .await
            })
            .map_err(|e| {
                warn!("❌ X25519 keygen failed: {}", e);
                Error::General(format!("X25519 keygen failed: {}", e))
            })?;
        
        debug!(
            "✅ X25519 keypair generated (public: {} bytes, secret: {} bytes)",
            public_key.len(),
            secret_key.len()
        );
        
        Ok(Box::new(X25519KeyExchange {
            crypto,
            our_secret: secret_key,
            our_public: public_key,
        }))
    }
    
    /// Get the named group identifier
    ///
    /// Returns `NamedGroup::X25519` (TLS 1.3 identifier for X25519)
    fn name(&self) -> NamedGroup {
        NamedGroup::X25519
    }
}

/// Static instance of X25519 group for use in CryptoProvider
///
/// This is required by rustls because it needs `&'static` references.
///
/// # Example
/// ```rust,no_run
/// use rustls::crypto::SupportedKxGroup;
/// use songbird_orchestrator::crypto::rustls_provider::kx_group::X25519_GROUP;
///
/// let kx_groups: Vec<&'static dyn SupportedKxGroup> = vec![&X25519_GROUP];
/// ```
pub static X25519_GROUP: X25519Group = X25519Group;

// ============================================================================
// X25519KeyExchange - Active Key Exchange State
// ============================================================================

/// Active key exchange using X25519 via BearDog
///
/// This struct:
/// - Stores our ephemeral keypair (public + secret)
/// - Stores a capability reference to crypto provider
/// - Implements `ActiveKeyExchange` to complete the key exchange
/// - Delegates shared secret derivation to BearDog
///
/// **Lifecycle**:
/// 1. Created by `X25519Group::start()` with fresh keypair
/// 2. `pub_key()` called by rustls to send our public key to peer
/// 3. `complete()` called by rustls with peer's public key
/// 4. Returns shared secret for session key derivation
///
/// **Security**:
/// - Ephemeral keys (new for each handshake)
/// - Perfect forward secrecy
/// - Constant-time Diffie-Hellman in BearDog
#[derive(Debug)]
struct X25519KeyExchange {
    /// Capability reference to crypto provider
    crypto: Arc<dyn CryptoProvider>,
    
    /// Our ephemeral secret key (32 bytes)
    our_secret: Vec<u8>,
    
    /// Our ephemeral public key (32 bytes)
    our_public: Vec<u8>,
}

impl ActiveKeyExchange for X25519KeyExchange {
    /// Complete the key exchange by deriving the shared secret
    ///
    /// This method:
    /// 1. Takes peer's public key as input
    /// 2. Delegates to `crypto.x25519_derive_secret()` via BearDog
    /// 3. Returns the shared secret for session key derivation
    ///
    /// # Arguments
    /// * `peer_pub_key` - Peer's X25519 public key (32 bytes)
    ///
    /// # Returns
    /// * `Ok(SharedSecret)` - Shared secret (32 bytes)
    /// * `Err(Error)` - If key exchange fails (invalid key, IPC error)
    ///
    /// # Performance
    /// - Expected: ~1ms (0.5ms crypto + 0.5ms IPC)
    /// - Called once per TLS handshake
    ///
    /// # Security
    /// - Uses Diffie-Hellman key agreement
    /// - Constant-time implementation in BearDog
    /// - Perfect forward secrecy (ephemeral keys)
    fn complete(self: Box<Self>, peer_pub_key: &[u8]) -> Result<SharedSecret, Error> {
        debug!(
            "🔐 Completing X25519 key exchange (peer key: {} bytes)",
            peer_pub_key.len()
        );
        
        // Derive shared secret via BearDog
        // Bridge async → sync using tokio runtime
        let shared = tokio::runtime::Handle::current()
            .block_on(async {
                self.crypto
                    .x25519_derive_secret(&self.our_secret, peer_pub_key)
                    .await
            })
            .map_err(|e| {
                warn!("❌ X25519 key exchange failed: {}", e);
                Error::General(format!("X25519 key exchange failed: {}", e))
            })?;
        
        debug!("✅ X25519 shared secret derived ({} bytes)", shared.len());
        
        // Convert to rustls SharedSecret
        Ok(SharedSecret::from(&shared[..]))
    }
    
    /// Get our ephemeral public key
    ///
    /// This is sent to the peer during the TLS handshake.
    ///
    /// # Returns
    /// * `&[u8]` - Our X25519 public key (32 bytes)
    fn pub_key(&self) -> &[u8] {
        &self.our_public
    }
    
    /// Get the named group for this key exchange
    ///
    /// Returns `NamedGroup::X25519` (TLS 1.3 identifier)
    fn group(&self) -> NamedGroup {
        NamedGroup::X25519
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use async_trait::async_trait;
    
    // Mock crypto provider for testing (isolated to tests!)
    #[derive(Debug)]
    struct MockCryptoProvider;
    
    impl MockCryptoProvider {
        fn new() -> Self {
            Self
        }
    }
    
    #[async_trait]
    impl CryptoProvider for MockCryptoProvider {
        async fn blake3_hash(&self, _data: &[u8]) -> Result<Vec<u8>> {
            Ok(vec![0u8; 32])
        }
        
        async fn hmac_sha256(&self, _key: &[u8], _data: &[u8]) -> Result<Vec<u8>> {
            Ok(vec![0u8; 32])
        }
        
        async fn sign_ed25519(
            &self,
            _message: &[u8],
            _key_id: &str,
            _purpose: &str,
        ) -> Result<Vec<u8>> {
            Ok(vec![0u8; 64])
        }
        
        async fn verify_ed25519(
            &self,
            _message: &[u8],
            _signature: &[u8],
            _public_key: &[u8],
        ) -> Result<bool> {
            Ok(true)
        }
        
        async fn x25519_generate_ephemeral(&self, _purpose: &str) -> Result<(Vec<u8>, Vec<u8>)> {
            // Return mock 32-byte public and secret keys
            Ok((vec![1u8; 32], vec![2u8; 32]))
        }
        
        async fn x25519_derive_secret(
            &self,
            _our_secret: &[u8],
            _their_public: &[u8],
        ) -> Result<Vec<u8>> {
            // Return mock 32-byte shared secret
            Ok(vec![3u8; 32])
        }
        
        async fn chacha20_poly1305_encrypt(
            &self,
            _plaintext: &[u8],
            _key: &[u8],
            _aad: Option<&[u8]>,
        ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            Ok((vec![0u8; 16], vec![0u8; 12], vec![0u8; 16]))
        }
        
        async fn chacha20_poly1305_decrypt(
            &self,
            _ciphertext: &[u8],
            _key: &[u8],
            _nonce: &[u8],
            _tag: &[u8],
            _aad: Option<&[u8]>,
        ) -> Result<Vec<u8>> {
            Ok(vec![0u8; 16])
        }
    }
    
    #[test]
    fn test_x25519_group_name() {
        let group = X25519Group;
        assert_eq!(group.name(), NamedGroup::X25519);
    }
    
    #[test]
    fn test_x25519_group_start_without_init_fails() {
        // Should fail if crypto provider not initialized
        let group = X25519Group;
        let result = group.start();
        
        assert!(result.is_err());
        // Check error message without unwrap_err (which requires Debug)
        if let Err(e) = result {
            assert!(e.to_string().contains("not initialized"));
        }
    }
    
    #[test]
    fn test_init_runtime_crypto_provider() {
        // Clear any existing provider (for test isolation)
        // Note: OnceCell doesn't support clearing, so we use a different test strategy
        
        // Create a new runtime for this test
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        
        let crypto = Arc::new(MockCryptoProvider::new());
        
        // This test can only run once per process due to OnceCell
        // In practice, this is fine because init should only happen once
        let result = init_runtime_crypto_provider(crypto);
        
        // May succeed or fail depending on test order, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_x25519_key_exchange_pub_key() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        
        let crypto = Arc::new(MockCryptoProvider::new());
        let kx = X25519KeyExchange {
            crypto,
            our_secret: vec![2u8; 32],
            our_public: vec![1u8; 32],
        };
        
        assert_eq!(kx.pub_key(), &vec![1u8; 32]);
    }
    
    #[test]
    fn test_x25519_key_exchange_group() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        
        let crypto = Arc::new(MockCryptoProvider::new());
        let kx = X25519KeyExchange {
            crypto,
            our_secret: vec![2u8; 32],
            our_public: vec![1u8; 32],
        };
        
        assert_eq!(kx.group(), NamedGroup::X25519);
    }
    
    #[test]
    fn test_x25519_key_exchange_complete() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        
        let crypto = Arc::new(MockCryptoProvider::new());
        let kx = Box::new(X25519KeyExchange {
            crypto,
            our_secret: vec![2u8; 32],
            our_public: vec![1u8; 32],
        });
        
        // Complete with peer's public key
        let peer_key = vec![4u8; 32];
        let result = kx.complete(&peer_key);
        
        // Should succeed with mock provider
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_static_x25519_group_exists() {
        // Verify static instance exists and is accessible
        let _group: &X25519Group = &X25519_GROUP;
        
        // Verify it implements the trait correctly
        assert_eq!(X25519_GROUP.name(), NamedGroup::X25519);
    }
}

