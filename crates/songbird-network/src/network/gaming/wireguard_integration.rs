//! WireGuard Integration Module
//!
//! Provides secure VPN tunneling for gaming sessions using WireGuard protocol.
//! This module creates encrypted tunnels that gaming traffic can pass through securely.

use serde::{Deserialize, Serialize};

use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, info, warn};

/// WireGuard-based security provider (failsafe fallback)
///
/// This provides basic security capabilities when BearDog is not available.
/// It does NOT attempt to recreate BearDog's advanced features.
#[allow(dead_code)]
pub struct WireGuardSecurityProvider {
    config: WireGuardConfig,
    active_tunnels: Arc<tokio::sync::RwLock<HashMap<String, WireGuardTunnel>>>,
    is_beardog_available: bool,
}

impl WireGuardSecurityProvider {
    pub fn new(config: WireGuardConfig) -> Self {
        Self {
            config,
            active_tunnels: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            is_beardog_available: false, // Will be updated by health checks
        }
    }

    /// Check if BearDog security primal is available
    pub async fn check_beardog_availability(&mut self) -> bool {
        // Try to detect BearDog at ../beardog
        let beardog_endpoints = vec!["https://127.0.0.1:8443", "http://127.0.0.1:8080"];

        for endpoint in beardog_endpoints {
            if let Ok(response) = reqwest::Client::new()
                .get(format!("{endpoint}/api/v1/health"))
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                if response.status().is_success() {
                    info!("🐕 BearDog security primal detected at {}", endpoint);
                    self.is_beardog_available = true;
                    return true;
                }
            }
        }

        if self.is_beardog_available {
            warn!("⚠️  BearDog security primal no longer available, falling back to WireGuard");
            self.is_beardog_available = false;
        }
        false
    }

    /// Create a basic WireGuard tunnel (fallback security)
    pub async fn create_wireguard_tunnel(
        &self,
        session_id: &str,
        peer_info: &PeerInfo,
    ) -> Result<WireGuardTunnel> {
        if self.is_beardog_available {
            warn!("Using WireGuard fallback while BearDog is available - check integration");
        }

        info!(
            "🔒 Creating WireGuard tunnel for session: {} (BearDog unavailable)",
            session_id
        );

        let tunnel_config = self.generate_tunnel_config(session_id, peer_info).await?;
        let tunnel = WireGuardTunnel::new(tunnel_config)?;

        Ok(tunnel)
    }

    /// Generate basic WireGuard configuration
    async fn generate_tunnel_config(
        &self,
        session_id: &str,
        peer_info: &PeerInfo,
    ) -> Result<WireGuardTunnelConfig> {
        // Generate basic WireGuard keypair
        let private_key = self.generate_private_key().await?;
        let public_key = self.derive_public_key(&private_key).await?;

        Ok(WireGuardTunnelConfig {
            session_id: session_id.to_string(),
            private_key,
            public_key,
            peer_public_key: peer_info.public_key.clone(),
            endpoint: peer_info.endpoint.clone(),
            allowed_ips: vec!["10.0.0.0/24".to_string()], // Basic IP range
            persistent_keepalive: Some(25),
        })
    }

    /// Generate WireGuard private key
    async fn generate_private_key(&self) -> Result<String> {
        let output = tokio::task::spawn_blocking(|| {
            Command::new("wg")
                .args(["genkey"])
                .stdout(Stdio::piped())
                .output()
        })
        .await
        .map_err(|e| SongbirdError::network_error(format!("Failed to spawn key generation: {e}")))?
        .map_err(|e| {
            SongbirdError::network_error(format!("Failed to generate private key: {e}"))
        })?;

        if !output.status.success() {
            return Err(SongbirdError::network_error(
                "WireGuard key generation failed",
            ));
        }

        let key = String::from_utf8(output.stdout)
            .map_err(|e| SongbirdError::network_error(format!("Invalid key format: {e}")))?
            .trim()
            .to_string();

        Ok(key)
    }

    /// Derive public key from private key
    async fn derive_public_key(&self, private_key: &str) -> Result<String> {
        let private_key = private_key.to_string();
        let output = tokio::task::spawn_blocking(move || {
            let mut child = Command::new("wg")
                .args(["pubkey"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            use std::io::Write;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(private_key.as_bytes())?;
            child.wait_with_output()
        })
        .await
        .map_err(|e| {
            SongbirdError::network_error(format!("Failed to spawn pubkey derivation: {e}"))
        })?
        .map_err(|e| SongbirdError::network_error(format!("Failed to derive public key: {e}")))?;

        if !output.status.success() {
            return Err(SongbirdError::network_error(
                "WireGuard pubkey derivation failed",
            ));
        }

        let key = String::from_utf8(output.stdout)
            .map_err(|e| SongbirdError::network_error(format!("Invalid pubkey format: {e}")))?
            .trim()
            .to_string();

        Ok(key)
    }

    /// Get security mode description
    pub fn get_security_mode(&self) -> &'static str {
        if self.is_beardog_available {
            "BearDog Enhanced Security"
        } else {
            "WireGuard Fallback Security"
        }
    }
}

/// Basic WireGuard tunnel (fallback implementation)
pub struct WireGuardTunnel {
    config: WireGuardTunnelConfig,
    interface_name: String,
    is_active: bool,
}

impl WireGuardTunnel {
    pub fn new(config: WireGuardTunnelConfig) -> Result<Self> {
        let interface_name = format!("wg-{}", &config.session_id[..8]);

        Ok(Self {
            config,
            interface_name,
            is_active: false,
        })
    }

    /// Activate the WireGuard tunnel
    pub async fn activate(&mut self) -> Result<()> {
        info!("🔗 Activating WireGuard tunnel: {}", self.interface_name);

        // Create WireGuard configuration file
        let config_content = self.generate_config_file();
        let config_path = format!("/tmp/{}.conf", self.interface_name);

        tokio::fs::write(&config_path, config_content)
            .await
            .map_err(|e| SongbirdError::network_error(format!("Failed to write config: {e}")))?;

        // Bring up the interface
        let output = Command::new("wg-quick")
            .args(["up", &config_path])
            .output()
            .map_err(|e| {
                SongbirdError::network_error(format!("Failed to execute wg-quick: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SongbirdError::network_error(format!(
                "WireGuard activation failed: {stderr}"
            )));
        }

        self.is_active = true;
        info!("✅ WireGuard tunnel active: {}", self.interface_name);
        Ok(())
    }

    /// Generate WireGuard configuration file content
    fn generate_config_file(&self) -> String {
        format!(
            r#"[Interface]
PrivateKey = {}
Address = 10.0.0.2/24
DNS = 1.1.1.1

[Peer]
PublicKey = {}
Endpoint = {}
AllowedIPs = {}
PersistentKeepalive = {}
"#,
            self.config.private_key,
            self.config.peer_public_key,
            self.config.endpoint,
            self.config.allowed_ips.join(", "),
            self.config.persistent_keepalive.unwrap_or(25)
        )
    }

    /// Deactivate the tunnel
    pub async fn deactivate(&mut self) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        info!("🔌 Deactivating WireGuard tunnel: {}", self.interface_name);

        let config_path = format!("/tmp/{}.conf", self.interface_name);

        let output = Command::new("wg-quick")
            .args(["down", &config_path])
            .output()
            .map_err(|e| {
                SongbirdError::network_error(format!("Failed to execute wg-quick: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("WireGuard deactivation failed: {}", stderr);
        }

        // Clean up config file
        if let Err(e) = tokio::fs::remove_file(&config_path).await {
            warn!("Failed to remove config file: {}", e);
        }

        self.is_active = false;
        info!("✅ WireGuard tunnel deactivated: {}", self.interface_name);
        Ok(())
    }
}

/// WireGuard tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardTunnelConfig {
    pub session_id: String,
    pub private_key: String,
    pub public_key: String,
    pub peer_public_key: String,
    pub endpoint: String,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive: Option<u32>,
}

/// WireGuard provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub enabled: bool,
    pub default_port: u16,
    pub network_range: String,
    pub dns_servers: Vec<String>,
}

impl Default for WireGuardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_port: 51820,
            network_range: "10.0.0.0/24".to_string(),
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        }
    }
}

/// Peer information for WireGuard connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub public_key: String,
    pub endpoint: String,
    pub allowed_ips: Vec<String>,
}

// STUB implementations marked for future enhancement by BearDog integration

/// Gaming tunnel manager with BearDog integration points
///
/// **NOTE**: These are STUB implementations that will be replaced when BearDog
/// integration is complete. They provide basic functionality for standalone operation.
pub struct GamingTunnelManager {
    wireguard_provider: WireGuardSecurityProvider,
    // Future: BearDog integration will replace STUB implementations
}

impl GamingTunnelManager {
    pub fn new() -> Self {
        Self {
            wireguard_provider: WireGuardSecurityProvider::new(WireGuardConfig::default()),
        }
    }

    /// Create tunnel - tries BearDog first, falls back to WireGuard
    pub async fn create_tunnel(&mut self, session_id: String) -> Result<String> {
        // Check if BearDog is available
        if self.wireguard_provider.check_beardog_availability().await {
            info!("🐕 BearDog available - using BSTP tunnels instead of WireGuard");
            // Future: Delegate to BearDog's BSTP implementation
            return Ok(format!("BSTP-{session_id}"));
        }

        // Fallback to WireGuard
        warn!("🔒 BearDog unavailable - using WireGuard fallback");
        let peer_info = PeerInfo {
            public_key: "placeholder-peer-key".to_string(),
            endpoint: "192.168.1.100:51820".to_string(),
            allowed_ips: vec!["10.0.0.0/24".to_string()],
        };

        let _tunnel = self
            .wireguard_provider
            .create_wireguard_tunnel(&session_id, &peer_info)
            .await?;
        Ok(format!("WG-{session_id}"))
    }
}

/// BSTP Tunnel Manager - Integration point for BearDog
///
/// **NOTE**: This is currently a STUB that provides basic WireGuard fallback.
/// When BearDog integration is complete, this will delegate to BearDog's BSTP
/// implementation for advanced security features.
pub struct BSTPTunnelManager {
    inner: super::advanced_tunnel_system::BSTPTunnelManager,
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
        let _advanced_tunnel =
            super::advanced_tunnel_system::BSTPTunnel::new_bstp_tunnel(session_id.clone())?;

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

        debug!(
            "🔐 BSTP encrypted {} bytes to {} bytes",
            packet.len(),
            encrypted.len()
        );
        Ok(encrypted)
    }

    /// STUB: Zero-copy BSTP encryption
    pub fn encrypt_zero_copy(&mut self, packet: &mut [u8]) -> Result<usize> {
        // This would perform in-place encryption with BearDog's zero-copy optimization
        debug!("🚀 BSTP zero-copy encryption for {} bytes", packet.len());

        // Placeholder: XOR with a simple pattern
        for (i, byte) in packet.iter_mut().enumerate() {
            *byte ^= ((i % 256) as u8).wrapping_add(0x5A);
        }

        Ok(packet.len()) // Return the size of encrypted data
    }

    /// STUB: Gaming-optimized BSTP decryption
    pub fn decrypt_gaming_packet_bstp(&mut self, encrypted: &[u8]) -> Result<Vec<u8>> {
        // Validate BSTP header
        if encrypted.len() < 24 || &encrypted[0..4] != b"BSTP" {
            return Err(SongbirdError::network_error("Invalid BSTP packet format"));
        }

        // Extract length
        let length =
            u32::from_le_bytes([encrypted[4], encrypted[5], encrypted[6], encrypted[7]]) as usize;

        if encrypted.len() < 8 + length + 16 {
            return Err(SongbirdError::network_error("BSTP packet too short"));
        }

        // Extract the original data (placeholder - would be decrypted)
        let decrypted = encrypted[8..8 + length].to_vec();

        debug!(
            "🔓 BSTP decrypted {} bytes to {} bytes",
            encrypted.len(),
            decrypted.len()
        );
        Ok(decrypted)
    }
}

/// BSTP tunnel structure - placeholder for BearDog integration
pub struct BSTPTunnel {
    // Placeholder - BearDog integration will provide real tunnel state
}

impl Default for BSTPTunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GamingTunnelManager {
    fn default() -> Self {
        Self::new()
    }
}
