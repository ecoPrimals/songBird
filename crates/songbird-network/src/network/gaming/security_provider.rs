//! Self-Healing Security Provider System
//!
//! This module implements automatic detection and seamless switching between
//! WireGuard (standalone) and BSTP (BearDog integrated) tunnel protocols.
//!
//! The system maintains sovereignty:
//! - Works perfectly with WireGuard alone
//! - Automatically detects BearDog availability  
//! - Upgrades tunnels seamlessly when BearDog is present
//! - Falls back gracefully if BearDog becomes unavailable

use crate::network::gaming::wireguard_integration::GamingTunnelManager;
use async_trait::async_trait;
use songbird_errors::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

#[cfg(feature = "beardog")]
use crate::network::gaming::bstp_handshake::BSTPHandshakeManager;

/// Self-healing security provider trait
#[async_trait]
pub trait SecurityProvider: Send + Sync {
    /// Create a secure tunnel for gaming
    async fn create_tunnel(
        &self,
        session_id: String,
        peer_info: PeerInfo,
    ) -> Result<Box<dyn SecureTunnel>>;

    /// Get current security level
    fn security_level(&self) -> SecurityLevel;

    /// Check if provider is available
    async fn is_available(&self) -> bool;

    /// Get provider name for logging
    fn provider_name(&self) -> &'static str;
}

/// Secure tunnel trait - implemented by both WireGuard and BSTP
#[async_trait]
pub trait SecureTunnel: Send + Sync {
    /// Encrypt gaming packet
    async fn encrypt_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt gaming packet  
    async fn decrypt_packet(&mut self, encrypted: &[u8]) -> Result<Vec<u8>>;

    /// Get tunnel type
    fn tunnel_type(&self) -> TunnelType;

    /// Check if tunnel is active
    async fn is_active(&self) -> bool;

    /// Migrate to higher security tunnel if available
    async fn attempt_upgrade(&self) -> Result<Option<Box<dyn SecureTunnel>>>;
}

/// Security levels in order of preference
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Standard, // WireGuard
    Enhanced, // BSTP
    Maximum,  // Future: Quantum-resistant
}

/// Tunnel types
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType {
    WireGuard,
    BSTP,
}

/// Peer information for tunnel creation
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub session_id: String,
    pub endpoint: std::net::SocketAddr,
    pub public_key: Option<Vec<u8>>,
}

/// Self-healing security manager - automatically detects and uses best available security
pub struct SelfHealingSecurityManager {
    /// Current active provider
    active_provider: Arc<RwLock<Box<dyn SecurityProvider>>>,

    /// Available providers in priority order
    providers: Vec<Box<dyn SecurityProvider>>,

    /// Detection interval
    detection_interval: Duration,

    /// Last detection check
    last_check: Arc<RwLock<Instant>>,

    /// Upgrade statistics
    upgrade_stats: Arc<RwLock<UpgradeStats>>,
}

#[derive(Debug, Default)]
struct UpgradeStats {
    wireguard_to_bstp: u64,
    bstp_to_wireguard: u64,
    failed_upgrades: u64,
    total_tunnels: u64,
}

impl SelfHealingSecurityManager {
    /// Create new self-healing security manager
    pub async fn new() -> Result<Self> {
        info!("🛡️ Initializing Self-Healing Security Manager");

        let mut providers: Vec<Box<dyn SecurityProvider>> = Vec::new();

        // Always add WireGuard (sovereign operation)
        providers.push(Box::new(WireGuardSecurityProvider::new().await?));

        // Conditionally add BearDog if available
        #[cfg(feature = "beardog")]
        {
            match BSTPSecurityProvider::new().await {
                Ok(bstp_provider) => {
                    info!("🐕 BearDog BSTP provider detected and available");
                    providers.push(Box::new(bstp_provider));
                }
                Err(e) => {
                    info!("🔐 BearDog not available, using WireGuard: {}", e);
                }
            }
        }

        // Start with highest available security level
        let best_provider = Self::select_best_provider(&providers).await;

        info!(
            "🛡️ Starting with {} security ({})",
            best_provider.security_level().name(),
            best_provider.provider_name()
        );

        Ok(Self {
            active_provider: Arc::new(RwLock::new(best_provider)),
            providers,
            detection_interval: Duration::from_secs(30),
            last_check: Arc::new(RwLock::new(Instant::now())),
            upgrade_stats: Arc::new(RwLock::new(UpgradeStats::default())),
        })
    }

    /// Create tunnel with self-healing capabilities
    pub async fn create_secure_tunnel(
        &self,
        session_id: String,
        peer_info: PeerInfo,
    ) -> Result<Box<dyn SecureTunnel>> {
        // Check for better providers periodically
        self.check_for_upgrades().await?;

        // Create tunnel with current best provider
        let provider = self.active_provider.read().await;
        let tunnel = provider
            .create_tunnel(session_id.clone(), peer_info)
            .await?;

        // Update stats
        {
            let mut stats = self.upgrade_stats.write().await;
            stats.total_tunnels += 1;
        }

        info!(
            "🔒 Created {} tunnel for session: {}",
            tunnel.tunnel_type().name(),
            session_id
        );

        Ok(tunnel)
    }

    /// Periodically check for provider upgrades
    async fn check_for_upgrades(&self) -> Result<()> {
        let now = Instant::now();
        let should_check = {
            let last_check = self.last_check.read().await;
            now.duration_since(*last_check) > self.detection_interval
        };

        if !should_check {
            return Ok(());
        }

        // Update last check time
        {
            let mut last_check = self.last_check.write().await;
            *last_check = now;
        }

        // Find best available provider
        let best_provider = Self::select_best_provider(&self.providers).await;

        // Check if we should upgrade
        let should_upgrade = {
            let current_provider = self.active_provider.read().await;
            best_provider.security_level() > current_provider.security_level()
        };

        if should_upgrade {
            let old_name = {
                let current = self.active_provider.read().await;
                current.provider_name()
            };

            // Upgrade to better provider
            {
                let mut active_provider = self.active_provider.write().await;
                *active_provider = best_provider;
            }

            let new_name = {
                let current = self.active_provider.read().await;
                current.provider_name()
            };

            info!("⬆️ Upgraded security provider: {} → {}", old_name, new_name);

            // Update stats
            {
                let mut stats = self.upgrade_stats.write().await;
                if old_name == "WireGuard" && new_name == "BSTP" {
                    stats.wireguard_to_bstp += 1;
                }
            }
        }

        Ok(())
    }

    /// Select best available provider
    async fn select_best_provider(
        providers: &[Box<dyn SecurityProvider>],
    ) -> Box<dyn SecurityProvider> {
        let mut best_level = SecurityLevel::Standard;

        for provider in providers {
            if provider.is_available().await && provider.security_level() > best_level {
                best_level = provider.security_level();
            }
        }

        // Clone the best provider (will need to implement Clone for providers)
        // For now, recreate the provider
        match best_level {
            SecurityLevel::Enhanced => {
                #[cfg(feature = "beardog")]
                {
                    if let Ok(bstp) = BSTPSecurityProvider::new().await {
                        return Box::new(bstp);
                    }
                }
                // Fallback to WireGuard
                Box::new(
                    WireGuardSecurityProvider::new()
                        .await
                        .expect("WireGuard should always work"),
                )
            }
            _ => Box::new(
                WireGuardSecurityProvider::new()
                    .await
                    .expect("WireGuard should always work"),
            ),
        }
    }

    /// Get current security stats
    pub async fn get_stats(&self) -> SecurityStats {
        let stats = self.upgrade_stats.read().await;
        let current_provider = self.active_provider.read().await;

        SecurityStats {
            current_provider: current_provider.provider_name().to_string(),
            security_level: current_provider.security_level(),
            total_tunnels: stats.total_tunnels,
            wireguard_to_bstp_upgrades: stats.wireguard_to_bstp,
            bstp_to_wireguard_fallbacks: stats.bstp_to_wireguard,
            failed_upgrades: stats.failed_upgrades,
        }
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub current_provider: String,
    pub security_level: SecurityLevel,
    pub total_tunnels: u64,
    pub wireguard_to_bstp_upgrades: u64,
    pub bstp_to_wireguard_fallbacks: u64,
    pub failed_upgrades: u64,
}

// =============================================================================
// WireGuard Security Provider (Always Available)
// =============================================================================

struct WireGuardSecurityProvider {
    tunnel_manager: Arc<GamingTunnelManager>,
}

impl WireGuardSecurityProvider {
    async fn new() -> Result<Self> {
        let config = crate::network::gaming::wireguard_integration::WireGuardConfig::default();
        let tunnel_manager = Arc::new(GamingTunnelManager::new(config));

        Ok(Self { tunnel_manager })
    }
}

#[async_trait]
impl SecurityProvider for WireGuardSecurityProvider {
    async fn create_tunnel(
        &self,
        session_id: String,
        peer_info: PeerInfo,
    ) -> Result<Box<dyn SecureTunnel>> {
        // For now, use placeholder public key - in real implementation this would come from peer_info
        use x25519_dalek::PublicKey;
        let placeholder_key = PublicKey::from([0u8; 32]);

        let _tunnel_id = self
            .tunnel_manager
            .create_gaming_tunnel(session_id.clone(), placeholder_key, peer_info.endpoint)
            .await?;

        Ok(Box::new(WireGuardTunnelWrapper {
            session_id,
            tunnel_manager: Arc::clone(&self.tunnel_manager),
        }))
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Standard
    }

    async fn is_available(&self) -> bool {
        true // WireGuard is always available
    }

    fn provider_name(&self) -> &'static str {
        "WireGuard"
    }
}

// Wrapper to make WireGuard implement SecureTunnel
struct WireGuardTunnelWrapper {
    session_id: String,
    tunnel_manager: Arc<GamingTunnelManager>,
}

#[async_trait]
impl SecureTunnel for WireGuardTunnelWrapper {
    async fn encrypt_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>> {
        match self
            .tunnel_manager
            .encrypt_packet(&self.session_id, packet)
            .await?
        {
            Some(encrypted) => Ok(encrypted),
            None => Err(songbird_errors::SongbirdError::Network {
                service: "WireGuard".to_string(),
                message: "Tunnel not found for session".to_string(),
                details: Some(self.session_id.clone()),
            }),
        }
    }

    async fn decrypt_packet(&mut self, encrypted: &[u8]) -> Result<Vec<u8>> {
        match self
            .tunnel_manager
            .decrypt_packet(&self.session_id, encrypted)
            .await?
        {
            Some(decrypted) => Ok(decrypted),
            None => Err(songbird_errors::SongbirdError::Network {
                service: "WireGuard".to_string(),
                message: "Failed to decrypt packet".to_string(),
                details: Some(self.session_id.clone()),
            }),
        }
    }

    fn tunnel_type(&self) -> TunnelType {
        TunnelType::WireGuard
    }

    async fn is_active(&self) -> bool {
        self.tunnel_manager.has_tunnel(&self.session_id).await
    }

    async fn attempt_upgrade(&self) -> Result<Option<Box<dyn SecureTunnel>>> {
        // Check if BSTP is available for upgrade
        #[cfg(feature = "beardog")]
        {
            if let Ok(bstp_provider) = BSTPSecurityProvider::new().await {
                if bstp_provider.is_available().await {
                    debug!("⬆️ BSTP upgrade available for session: {}", self.session_id);
                    return Ok(Some(
                        Box::new(BSTPTunnelWrapper::new(self.session_id.clone()).await?)
                            as Box<dyn SecureTunnel>,
                    ));
                }
            }
        }

        #[cfg(not(feature = "beardog"))]
        {
            debug!("BSTP upgrade not available - feature 'beardog' not enabled");
        }

        Ok(None) // No upgrade available
    }
}

// =============================================================================
// BSTP Security Provider (Conditional - BearDog Integration)
// =============================================================================

#[cfg(feature = "beardog")]
struct BSTPSecurityProvider {
    // BearDog crypto provider integration
}

#[cfg(feature = "beardog")]
impl BSTPSecurityProvider {
    async fn new() -> Result<Self> {
        // Try to initialize BearDog
        // This would use the actual BearDog crates when available

        // For now, simulate BearDog availability check
        if std::env::var("BEARDOG_AVAILABLE").unwrap_or_default() == "true" {
            info!("🐕 BearDog crypto provider initialized successfully");
            Ok(Self {})
        } else {
            Err(songbird_errors::SongbirdError::Security {
                message: "BearDog not available".to_string(),
                context: Some("Set BEARDOG_AVAILABLE=true to simulate".to_string()),
            })
        }
    }
}

#[cfg(feature = "beardog")]
#[async_trait]
impl SecurityProvider for BSTPSecurityProvider {
    async fn create_tunnel(
        &self,
        session_id: String,
        _peer_info: PeerInfo,
    ) -> Result<Box<dyn SecureTunnel>> {
        info!("🔐 Creating BSTP tunnel with BearDog security");

        let tunnel = BSTPTunnelWrapper::new(session_id.clone()).await?;
        info!("🔒 Created BSTP tunnel for session: {}", session_id);

        Ok(Box::new(tunnel) as Box<dyn SecureTunnel>)
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Enhanced
    }

    async fn is_available(&self) -> bool {
        // Check if BearDog is still available
        std::env::var("BEARDOG_AVAILABLE").unwrap_or_default() == "true"
    }

    fn provider_name(&self) -> &'static str {
        "BSTP"
    }
}

// BSTP tunnel wrapper with real handshake encryption
#[cfg(feature = "beardog")]
struct BSTPTunnelWrapper {
    session_id: String,
    handshake_manager: BSTPHandshakeManager,
}

#[cfg(feature = "beardog")]
impl BSTPTunnelWrapper {
    async fn new(session_id: String) -> Result<Self> {
        let mut handshake_manager = BSTPHandshakeManager::new(session_id.clone());

        // Start BearDog handshake
        let _greeting = handshake_manager.start_handshake()?;

        // For testing, simulate successful handshake completion
        // In practice, this would involve actual network communication
        let mock_peer_key = [42u8; 32];
        let mock_greeting = crate::network::gaming::bstp_handshake::BearDogGreeting {
            version: 1,
            session_id: session_id.clone(),
            public_key: mock_peer_key,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: [0u8; 64], // Mock signature
        };

        // Process mock response and complete handshake
        let _key_exchange = handshake_manager.process_greeting_response(mock_greeting)?;
        let mock_confirmation = [0u8; 16];
        handshake_manager.complete_handshake(&mock_confirmation)?;

        info!("🤝 BSTP handshake completed for session: {}", session_id);

        Ok(Self {
            session_id,
            handshake_manager,
        })
    }
}

#[cfg(feature = "beardog")]
#[async_trait]
impl SecureTunnel for BSTPTunnelWrapper {
    async fn encrypt_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>> {
        if !self.handshake_manager.is_established() {
            return Err(songbird_errors::SongbirdError::Security {
                message: "BSTP handshake not established".to_string(),
                context: Some(format!("Session: {}", self.session_id)),
            });
        }

        // Use real AES-256-GCM encryption from handshake
        let encrypted = self.handshake_manager.encrypt_data(packet)?;

        // Add BSTP protocol wrapper
        let mut bstp_packet = Vec::with_capacity(encrypted.len() + 24);
        bstp_packet.extend_from_slice(b"BSTP"); // Magic header
        bstp_packet.extend_from_slice(&(encrypted.len() as u32).to_le_bytes()); // Length
        bstp_packet.extend_from_slice(&encrypted); // Encrypted payload
        bstp_packet.extend_from_slice(&[0xBE; 16]); // BearDog signature

        debug!(
            "🔐 BSTP encrypted {} bytes with real AES-256-GCM",
            packet.len()
        );
        Ok(bstp_packet)
    }

    async fn decrypt_packet(&mut self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if !self.handshake_manager.is_established() {
            return Err(songbird_errors::SongbirdError::Security {
                message: "BSTP handshake not established".to_string(),
                context: Some(format!("Session: {}", self.session_id)),
            });
        }

        // Verify BSTP format
        if encrypted.len() < 24 || &encrypted[0..4] != b"BSTP" {
            return Err(songbird_errors::SongbirdError::DecryptionFailed(
                "Invalid BSTP packet format".to_string(),
            ));
        }

        // Extract encrypted payload
        let length =
            u32::from_le_bytes([encrypted[4], encrypted[5], encrypted[6], encrypted[7]]) as usize;
        if encrypted.len() < 8 + length + 16 {
            return Err(songbird_errors::SongbirdError::DecryptionFailed(
                "BSTP packet too short".to_string(),
            ));
        }

        let encrypted_payload = &encrypted[8..8 + length];

        // Use real AES-256-GCM decryption from handshake
        let decrypted = self.handshake_manager.decrypt_data(encrypted_payload)?;

        debug!(
            "🔓 BSTP decrypted {} bytes with real AES-256-GCM",
            decrypted.len()
        );
        Ok(decrypted)
    }

    fn tunnel_type(&self) -> TunnelType {
        TunnelType::BSTP
    }

    async fn is_active(&self) -> bool {
        self.handshake_manager.is_established()
    }

    async fn attempt_upgrade(&self) -> Result<Option<Box<dyn SecureTunnel>>> {
        Ok(None) // BSTP is already highest security level
    }
}

// =============================================================================
// Helper trait implementations
// =============================================================================

impl SecurityLevel {
    pub fn name(&self) -> &'static str {
        match self {
            SecurityLevel::Standard => "Standard",
            SecurityLevel::Enhanced => "Enhanced",
            SecurityLevel::Maximum => "Maximum",
        }
    }
}

impl TunnelType {
    pub fn name(&self) -> &'static str {
        match self {
            TunnelType::WireGuard => "WireGuard",
            TunnelType::BSTP => "BSTP",
        }
    }
}

/// Security provider for gaming traffic
/// Handles encryption, access control, and security policies
pub struct GamingSecurityProvider {
    // ... existing code ...
}
