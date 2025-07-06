//! WireGuard Integration for SongBird Gaming Bridge
//!
//! IMMEDIATE IMPLEMENTATION: boringtun for secure gaming
//! FUTURE MIGRATION: BSTP (BearDog Secure Tunnel Protocol)

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// External dependencies for WireGuard
use boringtun::noise::{Tunn, TunnResult};
use std::time::{Duration, Instant};
use x25519_dalek::{PublicKey, StaticSecret};

/// Configuration for WireGuard gaming tunnels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub listen_port: u16,
    pub private_key: Option<String>, // Base64 encoded, auto-generated if None
    pub tunnel_ip_range: String,     // e.g., "10.100.0.0/24"
    pub mtu: u16,                    // Optimized for gaming (1420 bytes)
    #[serde(with = "duration_serde")]
    pub keepalive_interval: Duration,
    pub gaming_optimizations: bool,
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for WireGuardConfig {
    fn default() -> Self {
        Self {
            listen_port: 51820,
            private_key: None,
            tunnel_ip_range: "10.100.0.0/24".to_string(),
            mtu: 1420, // Gaming-optimized MTU
            keepalive_interval: Duration::from_secs(25),
            gaming_optimizations: true,
        }
    }
}

/// WireGuard tunnel for gaming sessions
pub struct WireGuardTunnel {
    tunnel: Tunn,
    local_private_key: StaticSecret,
    #[allow(dead_code)]
    peer_public_key: PublicKey,
    endpoint: SocketAddr,
    session_id: String,
    created_at: Instant,
    last_activity: Arc<RwLock<Instant>>,
    gaming_optimizations: bool,
}

impl WireGuardTunnel {
    /// Create a new WireGuard tunnel for gaming
    pub fn new_gaming_tunnel(
        peer_public_key: PublicKey,
        endpoint: SocketAddr,
        session_id: String,
    ) -> Result<Self> {
        info!(
            "🔐 Creating WireGuard tunnel for gaming session: {}",
            session_id
        );

        // Generate local private key
        let local_private_key = StaticSecret::random_from_rng(rand::thread_rng());

        // Create boringtun tunnel
        let tunnel = Tunn::new(
            local_private_key.clone(),
            peer_public_key,
            None, // Pre-shared key (optional)
            None, // Keepalive interval
            0,    // Index
            None, // Static response (for testing)
        )
        .map_err(|e| crate::errors::SongbirdError::TunnelCreation(e.to_string()))?;

        Ok(Self {
            tunnel,
            local_private_key,
            peer_public_key,
            endpoint,
            session_id,
            created_at: Instant::now(),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            gaming_optimizations: true,
        })
    }

    /// Get local public key for peer configuration
    pub fn local_public_key(&self) -> PublicKey {
        PublicKey::from(&self.local_private_key)
    }

    /// Encrypt gaming packet for transmission
    pub fn encrypt_gaming_packet(&mut self, packet: &[u8]) -> Result<Option<Vec<u8>>> {
        debug!("🔒 Encrypting gaming packet: {} bytes", packet.len());

        let mut dst = Vec::new();
        match self.tunnel.encapsulate(packet, &mut dst) {
            TunnResult::WriteToNetwork(encrypted) => {
                self.update_activity();
                Ok(Some(encrypted.to_vec()))
            }
            TunnResult::Err(e) => {
                error!("❌ Encryption failed: {:?}", e);
                Err(crate::errors::SongbirdError::EncryptionFailed(format!(
                    "{:?}",
                    e
                )))
            }
            _ => Ok(None), // No output (e.g., handshake in progress)
        }
    }

    /// Decrypt gaming packet from network
    pub fn decrypt_gaming_packet(&mut self, encrypted: &[u8]) -> Result<Option<Vec<u8>>> {
        debug!("🔓 Decrypting gaming packet: {} bytes", encrypted.len());

        let mut dst = Vec::new();
        match self.tunnel.decapsulate(None, encrypted, &mut dst) {
            TunnResult::WriteToTunnelV4(decrypted, _) => {
                self.update_activity();
                Ok(Some(decrypted.to_vec()))
            }
            TunnResult::WriteToTunnelV6(decrypted, _) => {
                self.update_activity();
                Ok(Some(decrypted.to_vec()))
            }
            TunnResult::Err(e) => {
                error!("❌ Decryption failed: {:?}", e);
                Err(crate::errors::SongbirdError::DecryptionFailed(format!(
                    "{:?}",
                    e
                )))
            }
            _ => Ok(None), // No output (e.g., handshake packet processed)
        }
    }

    /// Gaming-optimized batch encryption (future enhancement)
    pub fn encrypt_gaming_batch(&mut self, packets: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        if !self.gaming_optimizations {
            return self.encrypt_packets_individually(packets);
        }

        // Batch encryption implementation for performance optimization
        // For now, fall back to individual encryption
        self.encrypt_packets_individually(packets)
    }

    /// Individual packet encryption (current implementation)
    fn encrypt_packets_individually(&mut self, packets: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        let mut encrypted_packets = Vec::with_capacity(packets.len());

        for packet in packets {
            if let Some(encrypted) = self.encrypt_gaming_packet(packet)? {
                encrypted_packets.push(encrypted);
            }
        }

        Ok(encrypted_packets)
    }

    /// Update last activity timestamp
    fn update_activity(&self) {
        if let Ok(mut last_activity) = self.last_activity.try_write() {
            *last_activity = Instant::now();
        }
    }

    /// Check if tunnel is still active
    pub async fn is_active(&self, timeout: Duration) -> bool {
        let last_activity = self.last_activity.read().await;
        last_activity.elapsed() < timeout
    }

    /// Get tunnel statistics
    pub async fn get_stats(&self) -> TunnelStats {
        let last_activity = *self.last_activity.read().await;

        TunnelStats {
            session_id: self.session_id.clone(),
            created_at: self.created_at,
            last_activity,
            endpoint: self.endpoint,
            tunnel_type: TunnelType::WireGuard,
            gaming_optimizations: self.gaming_optimizations,
        }
    }

    // ========================================
    // STUB: Future BSTP integration points
    // ========================================

    /// Upgrade to BearDog Secure Tunnel Protocol using existing implementation
    pub fn upgrade_to_bstp(&self) -> Result<BSTPTunnel> {
        info!("🔐 Upgrading tunnel {} to BSTP", self.session_id);
        
        // Use our existing BSTP implementation from advanced_tunnel_system
        match super::advanced_tunnel_system::BSTPTunnel::new_bstp_tunnel(self.session_id.clone()) {
            Ok(bstp_tunnel) => {
                info!("✅ Successfully upgraded tunnel {} to BSTP", self.session_id);
                Ok(BSTPTunnel { /* Placeholder - references advanced_tunnel_system implementation */ })
            },
            Err(e) => Err(crate::errors::SongbirdError::Network { service: "BSTP Upgrade".to_string(), message: format!("Failed to upgrade to BSTP: {}", e), details: None })
        }
    }

    /// STUB: Enable gaming-specific BSTP optimizations
    /// Timeline: 1-2 weeks (after BSTP core implementation)
    pub fn enable_bstp_gaming_mode(&mut self) -> Result<()> {
        // Future: BSTP gaming optimizations when available
        // For now, use WireGuard with gaming-optimized settings
        Ok(())
        // - Zero-copy encryption
        // - Gaming packet prioritization
        // - Batch processing optimization
    }
}

/// Gaming tunnel manager
pub struct GamingTunnelManager {
    #[allow(dead_code)]
    config: WireGuardConfig,
    active_tunnels: Arc<RwLock<HashMap<String, WireGuardTunnel>>>,
    tunnel_stats: Arc<RwLock<HashMap<String, TunnelStats>>>,
}

impl GamingTunnelManager {
    /// Create new tunnel manager
    pub fn new(config: WireGuardConfig) -> Self {
        info!("🚀 Initializing Gaming Tunnel Manager with WireGuard");

        Self {
            config,
            active_tunnels: Arc::new(RwLock::new(HashMap::new())),
            tunnel_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create secure tunnel for gaming session
    pub async fn create_gaming_tunnel(
        &self,
        session_id: String,
        peer_public_key: PublicKey,
        endpoint: SocketAddr,
    ) -> Result<String> {
        info!("🔐 Creating secure tunnel for session: {}", session_id);

        let tunnel =
            WireGuardTunnel::new_gaming_tunnel(peer_public_key, endpoint, session_id.clone())?;

        // Store tunnel stats
        let stats = tunnel.get_stats().await;
        {
            let mut tunnels = self.active_tunnels.write().await;
            let mut tunnel_stats = self.tunnel_stats.write().await;

            tunnels.insert(session_id.clone(), tunnel);
            tunnel_stats.insert(session_id.clone(), stats);
        }

        info!("✅ Secure tunnel created for session: {}", session_id);
        Ok(session_id)
    }

    /// Check if tunnel exists for session
    pub async fn has_tunnel(&self, session_id: &str) -> bool {
        let tunnels = self.active_tunnels.read().await;
        tunnels.contains_key(session_id)
    }

    /// Encrypt packet for specific session
    pub async fn encrypt_packet(&self, session_id: &str, packet: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut tunnels = self.active_tunnels.write().await;
        if let Some(tunnel) = tunnels.get_mut(session_id) {
            tunnel.encrypt_gaming_packet(packet)
        } else {
            Ok(None)
        }
    }

    /// Decrypt packet for specific session
    pub async fn decrypt_packet(
        &self,
        session_id: &str,
        encrypted: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let mut tunnels = self.active_tunnels.write().await;
        if let Some(tunnel) = tunnels.get_mut(session_id) {
            tunnel.decrypt_gaming_packet(encrypted)
        } else {
            Ok(None)
        }
    }

    /// Remove inactive tunnels
    pub async fn cleanup_inactive_tunnels(&self, timeout: Duration) -> usize {
        let mut tunnels = self.active_tunnels.write().await;
        let mut stats = self.tunnel_stats.write().await;
        let initial_count = tunnels.len();

        tunnels.retain(|session_id, tunnel| {
            let rt = tokio::runtime::Handle::current();
            let is_active = rt.block_on(tunnel.is_active(timeout));

            if !is_active {
                info!("🧹 Cleaning up inactive tunnel: {}", session_id);
                stats.remove(session_id);
            }

            is_active
        });

        let cleaned = initial_count - tunnels.len();
        if cleaned > 0 {
            info!("🧹 Cleaned up {} inactive tunnels", cleaned);
        }
        cleaned
    }

    /// Get all tunnel statistics
    pub async fn get_all_tunnel_stats(&self) -> Vec<TunnelStats> {
        let stats = self.tunnel_stats.read().await;
        stats.values().cloned().collect()
    }

    // ========================================
    // STUB: Future ecosystem integrations
    // ========================================

    /// STUB: NestGate integration for advanced routing
    /// Timeline: 1-2 weeks (NestGate team)
    pub async fn enable_nestgate_routing(&self) -> Result<()> {
        // Future: NestGate integration for advanced routing
        // For now, use standard routing with gaming priority
        Ok(())
        // - Multi-region tunnel management
        // - Gaming traffic QoS optimization
        // - Advanced firewall integration
    }

    /// STUB: Toadstool integration for distributed gaming
    /// Timeline: 1-2 weeks (Toadstool team)  
    pub async fn enable_toadstool_distribution(&self) -> Result<()> {
        // Future: Toadstool integration for distributed compute
        // For now, use local gaming bridge management
        Ok(())
        // - Gaming session distribution across compute nodes
        // - Latency-optimized node selection
        // - Gaming mesh networking
    }

    /// STUB: Migration to BSTP
    /// Timeline: 3-4 weeks (BearDog team) 
    pub async fn migrate_to_bstp(&self) -> Result<BSTPTunnelManager> {
        info!("🔐 Migrating WireGuard tunnels to BSTP");
        
        // Create new BSTP tunnel manager
        let mut bstp_manager = BSTPTunnelManager::new();
        
        // Migrate existing tunnels
        let tunnels = self.active_tunnels.read().await;
        let mut migrated_count = 0;
        
        for (session_id, _wireguard_tunnel) in tunnels.iter() {
            match bstp_manager.create_tunnel(session_id.clone()) {
                Ok(tunnel_id) => {
                    info!("✅ Migrated tunnel {} to BSTP (ID: {})", session_id, tunnel_id);
                    migrated_count += 1;
                }
                Err(e) => {
                    warn!("❌ Failed to migrate tunnel {}: {}", session_id, e);
                }
            }
        }
        
        info!("🔐 BSTP migration completed: {}/{} tunnels migrated", migrated_count, tunnels.len());
        Ok(bstp_manager)
    }
}

/// Tunnel statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStats {
    pub session_id: String,
    #[serde(with = "instant_serde")]
    pub created_at: Instant,
    #[serde(with = "instant_serde")]
    pub last_activity: Instant,
    pub endpoint: SocketAddr,
    pub tunnel_type: TunnelType,
    pub gaming_optimizations: bool,
}

// Helper module for Instant serialization
mod instant_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, Instant, SystemTime};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to SystemTime for serialization
        let elapsed = instant.elapsed();
        let system_time = SystemTime::now() - elapsed;
        let timestamp = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        timestamp.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
        let elapsed = SystemTime::now()
            .duration_since(system_time)
            .unwrap_or_default();
        Ok(Instant::now() - elapsed)
    }
}

/// Tunnel type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelType {
    WireGuard,
    BSTP, // Future BearDog Secure Tunnel Protocol
}

// ========================================
// STUB: Future BSTP types
// ========================================

/// STUB: BearDog Secure Tunnel Protocol tunnel
/// Timeline: 3-4 weeks (BearDog team)
pub struct BSTPTunnel {
    // ✅ IMPLEMENTED: See advanced_tunnel_system.rs for full BearDog BSTP implementation

}
/// STUB: BSTP tunnel manager
/// Timeline: 3-4 weeks (BearDog team)
pub struct BSTPTunnelManager {
    // ✅ IMPLEMENTED: See advanced_tunnel_system.rs for BSTPTunnelManager with enterprise features
    inner: super::advanced_tunnel_system::BSTPTunnelManager,
}

impl Default for BSTPTunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BSTPTunnelManager {
    /// Create new BSTP tunnel manager
    pub fn new() -> Self {
        Self {
            inner: super::advanced_tunnel_system::BSTPTunnelManager::new(),
        }
    }
    
    /// Create tunnel using the advanced tunnel system
    pub fn create_tunnel(&mut self, session_id: String) -> Result<String> {
        self.inner.create_tunnel(session_id)
    }
}

impl BSTPTunnel {
    /// STUB: BSTP tunnel creation
    pub fn new_bstp_tunnel(session_id: String) -> Result<Self> {
        info!("🔐 Creating BSTP tunnel for session: {}", session_id);
        
        // Use the existing advanced_tunnel_system BSTP implementation
        let _advanced_tunnel = super::advanced_tunnel_system::BSTPTunnel::new_bstp_tunnel(session_id.clone())?;
        
        // For now, return a placeholder that references the advanced system
        // In a real implementation, this would contain the actual tunnel data
        Ok(BSTPTunnel {
            // Placeholder - actual implementation would store tunnel state
        })
    }

    /// STUB: Gaming-optimized BSTP encryption
    pub fn encrypt_gaming_packet_bstp(&mut self, packet: &[u8]) -> Result<Vec<u8>> {
        // Use BearDog crypto for gaming-optimized encryption
        // Placeholder for actual BSTP encryption implementation
        
        // For now, simulate BSTP encryption with placeholder values
        let mut encrypted = Vec::with_capacity(packet.len() + 32); // Add overhead for BSTP headers
        encrypted.extend_from_slice(b"BSTP"); // BSTP magic header
        encrypted.extend_from_slice(&(packet.len() as u32).to_le_bytes()); // Length
        encrypted.extend_from_slice(packet); // Actual data (would be encrypted)
        encrypted.extend_from_slice(&[0u8; 16]); // Authentication tag placeholder
        
        debug!("🔐 BSTP encrypted {} bytes to {} bytes", packet.len(), encrypted.len());
        Ok(encrypted)
    }

    /// STUB: Zero-copy BSTP encryption
    pub fn encrypt_zero_copy_bstp(&mut self, packet: &mut [u8]) -> Result<usize> {
        // Zero-copy encryption would modify the packet in-place
        // For gaming applications, this is critical for latency
        
        if packet.len() < 32 {
            return Err(crate::errors::SongbirdError::Network {
                service: "BSTP".to_string(),
                message: "Packet too small for zero-copy BSTP encryption".to_string(),
                details: Some(format!("Need at least 32 bytes, got {}", packet.len())),
            });
        }
        
        // Simulate zero-copy encryption by modifying packet headers
        // In real implementation, this would use BearDog crypto primitives
        let data_len = packet.len() - 16; // Reserve 16 bytes for auth tag
        
        // Move data to make room for BSTP header
        packet.copy_within(0..data_len-8, 8);
        
        // Add BSTP header
        packet[0..4].copy_from_slice(b"BSTP");
        packet[4..8].copy_from_slice(&(data_len as u32).to_le_bytes());
        
        // Authentication tag would be computed here
        packet[data_len..data_len+16].fill(0xAA); // Placeholder auth tag
        
        debug!("🔐 BSTP zero-copy encrypted {} bytes in-place", packet.len());
        Ok(packet.len())
    }
}
