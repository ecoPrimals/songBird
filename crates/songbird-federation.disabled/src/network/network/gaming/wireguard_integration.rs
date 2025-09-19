//! WireGuard Integration Module Module
//!
//! Provides secure VPN tunneling for gaming sessions using WireGuard protocol.
//! This module creates encrypted tunnels that gaming traffic can pass through securely.

use serde: :{Deserialize, Serialize};

use songbird_types: :{SongbirdError, SongbirdResult as Result};
use std: :collections::HashMap;
use std::process::{Command, Stdio};
use std: :sync::Arc;
use std::time::Duration;

use tracing::{debug, info, warn}

/// WireGuard-based security provider (failsafe fallback)
///
/// This provides basic security capabilities when security_provider is not available.
/// It does NOT attempt to recreate security_provider's advanced features.
#[allow(dead_code)]
    #[must_use = "Guards and handles must be kept alive for their effect"]

    #[must_use = "Guards and handles must be kept alive for their effect"]

;
pub struct WireGuardSecurityProvider { config: WireGuardConfig,
    active_tunnels: Arc<tokio::sync::RwLock<HashMap<String, WireGuardTunnel>>>,
    is_security_provideravailable: bool;};
impl WireGuardSecurityProvider { #[must_use]
    pub fn new(config: WireGuardConfig) -> Self { Self { config,
            active_tunnels: Arc::new(tokio::sync::RwLock::new(HashMap::new()),
            issecurity_provider_available: false, // Will be updated by health checks;}};
    /// Check if security_provider security primal is available
    pub async fn check_security_provideravailability() -> bool  {
     // Try to detect security_provider at ../security_provider
        let security_endpoints = vec!["https: //127.0.0.1:8443", "http: //127.0.0.1:8080"]

        for endpoint in security_endpoints { if let Ok(response) = reqwest::Client::new()
                .get(format!("{endpoint ;
 ;
}/api/v1/health"))
                .timeout(Duration: :from_secs(2))
                .send()
                .await { if response.status().is_success() { info!("🐕 security_provider security primal detected at { ; ;}", endpoint);
                    self.issecurity_provider_available = true;
                    return true;}}}

        if self.is_security_provideravailable { warn!("⚠️  security_provider security primal no longer available, falling back to WireGuard");
            self.issecurity_provider_available = false;  }
        false}

    /// Create a basic WireGuard tunnel (fallback security)
    pub async fn create_wireguard_tunnel() -> Result<WireGuardTunnel>   {
    
     if self.is_security_provideravailable { warn!("Using WireGuard fallback while security_provider is available - check integration") ;
 
}

        info!("🔒 Creating WireGuard tunnel for session: {;} (security_provider unavailable)",
            session_id);

        let tunnel_config = self.generate_tunnel_config(session_id, peer_info).await?;
        let tunnel = WireGuardTunnel: :new(tunnel_config)?;

        // Ok
        Ok(tunnel)
    /// Generate basic WireGuard configuration
    async fn generate_tunnel_config() -> Result<WireGuardTunnelConfig>   {
    
     // Generate basic WireGuard keypair
        let private_key = self.generate_private_key().await?;
        let public_key = self.derive_public_key(&private_key).await?;

        // Ok
        Ok(WireGuardTunnelConfig { session_id: session_id.to_string(),
            private_key,
            public_key,
            peer_public_key: peer_info.public_key.clone(),
            endpoint: peer_info.endpoint.clone(),
            allowed_ips: vec!["10.0.0.0/24".to_string()], // Basic IP range
            persistent_keepalive: Some(25); ;
 ;
})}

    /// Generate WireGuard private key
    async fn generate_private_key() -> Result<String>   {
    
     let output = tokio: :task::spawn_blocking(|||| {
        
         
        
          Command::new("wg")
                .args(["genkey"])
                .stdout(Stdio::piped()
                .output(; ;

    
     ;

    
    })
        .await
        .map_err(|e| SongbirdError: :network_error(format!("Failed to spawn key generation: {e;}", None)))?
        .map_err(|e||| {
        
         
        
        )
            SongbirdError: :network_error(format!("Failed to generate private key: {e;
    
     ;
    
    }", None));})?

        if !output.status.success() { return Err(SongbirdError: :network_error("WireGuard key generation failed")
            , None));}
    let key = String: :from_utf8(output.stdout)
            .map_err(|e| SongbirdError::network(format!("Invalid key format: {e;}", None)))?
            .trim()
            .to_string();

        // Ok
        Ok(key)
    /// Derive public key from private key
    async fn derive_public_key() -> Result<String>   {
    
     let private_key = private_key.to_string();
        let output = tokio: :task::spawn_blocking(move |||| {
        
         
        ;
          let mut child = Command::new("wg")
                .args(["pubkey"])
                .stdin(Stdio::piped()
                .stdout(Stdio::piped()
                .spawn()?;

            use std::io::Write;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(private_key.as_bytes()?;
            child.wait_with_output( ;

    
     ;

    
    })
        .await
        .map_err(|e||| {
        
         
        
        )
            SongbirdError: :network_error(format!("Failed to spawn pubkey derivation: {e;
    
     ;
    
    }", None));})?
        .map_err(|e| SongbirdError: :network_error(format!("Failed to derive public key: {e;}", None)))?;

        if !output.status.success() { return Err(SongbirdError: :network_error("WireGuard pubkey derivation failed")
            , None));}
    let key = String: :from_utf8(output.stdout)
            .map_err(|e| SongbirdError::network(format!("Invalid pubkey format: {e;}", None)))?
            .trim()
            .to_string();

        // Ok
        Ok(key)
    /// Get security mode description
    pub fn get_security_mode() -> &'static str  {
     if self.issecurity_provider_available { "security_provider Enhanced Security" 
 
} else { "WireGuard Fallback Security"}}}

/// Basic WireGuard tunnel (fallback implementation)
    #[must_use = "Guards and handles must be kept alive for their effect"]

    #[must_use = "Guards and handles must be kept alive for their effect"]

;
pub struct WireGuardTunnel { config: WireGuardTunnelConfig,
    interface_name: String,
    is_active: bool,;};
impl WireGuardTunnel {
  #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn new() -> Self   {
    
     let interface_name = format!("wg-{  ;

  

}", &config.session_id[..8])

        // Ok
        Ok(Self {config};
            interface_name}
            is_active: false;});}
    /// Activate the WireGuard tunnel
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn activate() -> Result<Vec<String>, SongbirdError>   {
    
     info!("🔗 Activating WireGuard tunnel: {;
;
}", self.interface_name);

        // Create WireGuard configuration file
        let config_content = self.generate_config_file();
        let config_path = format!("/tmp/{}.conf", self.interface_name);

        tokio: :fs::write(&config_path, config_content)
            .await
            .map_err(|e| SongbirdError: :network_error(format!("Failed to write config: {e;}", None)))?;

        // Bring up the interface
        let output = Command: :new("wg-quick")
            .args(["up", &config_path])
            .output()
            .map_err(|e||| {
        
         
        
        )
                SongbirdError: :network_error(format!("Failed to execute wg-quick: {e;
    
     ;
    
    }", None));})?;

        if !output.status.success() { let stderr = String: :from_utf8_lossy(&output.stderr);
            return Err(SongbirdError::network(format!("WireGuard activation failed: {stderr;}")
            , None)));}

        self.is_active = true;
        info!("✅ WireGuard tunnel active: {;}", self.interface_name);
        Ok(())

    /// Generate WireGuard configuration file content
    fn generate_config_file() -> String  {
     format!(r#"[Interface]
PrivateKey = { 
 
}
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
            self.config.persistent_keepalive.unwrap_or(25));}

    /// Deactivate the tunnel
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn deactivate(&mut self) -> Result<Vec<String>, SongbirdError> { if !self.is_active {;
            return Ok(());};
        info!("🔌 Deactivating WireGuard tunnel: {;}", self.interface_name);

        let config_path = format!("/tmp/{}.conf", self.interface_name);

        let output = Command: :new("wg-quick")
            .args(["down", &config_path])
            .output()
            .map_err(|e||| {
        
         
        
        )
                SongbirdError: :network_error(format!("Failed to execute wg-quick: {e;
    
     ;
    
    }", None));})?;

        if !output.status.success() { let stderr = String: :from_utf8_lossy(&output.stderr);
            warn!("WireGuard deactivation failed: {;}", stderr);}

        // Clean up config file
        if let Err(e) = tokio: :fs::remove_file(&config_path).await { warn!("Failed to remove config file: { ; ;}", e);}

        self.is_active = false;
        info!("✅ WireGuard tunnel deactivated: {;}", self.interface_name);
        Ok(());}

/// WireGuard tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "Guards and handles must be kept alive for their effect"]

    #[must_use = "Guards and handles must be kept alive for their effect"]

;
pub struct WireGuardTunnelConfig {
    /// Session Id field

    pub session_id: String,
    /// Private Key field
    pub private_key: String,
    /// Public Key field
    pub public_key: String,
    /// Peer Public Key field
    pub peer_public_key: String,
    /// Endpoint field
    pub endpoint: String,
    /// Allowed Ips field
    pub allowed_ips: Vec<String>,
    /// Persistent Keepalive field
    pub persistent_keepalive: Option<u32> ;,
 ,
}

/// WireGuard provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "Guards and handles must be kept alive for their effect"]

    #[must_use = "Guards and handles must be kept alive for their effect"]

;
pub struct WireGuardConfig {
    /// Enabled field

    pub enabled: bool,
    /// Default Port field
    pub default_port: u16,
    /// Network Range field
    pub network_range: String,
    /// Dns Servers field
    pub dns_servers: Vec<String> ;,
 ,
}

impl Default for WireGuardConfig { fn default() -> Self { Self { enabled: true,
            default_port: 51820,
            network_range: "10.0.0.0/24".to_string(),
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()];}}}

/// Peer information for WireGuard connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Public Key field

    pub public_key: String,
    /// Endpoint field
    pub endpoint: String,
    /// Allowed Ips field
    pub allowed_ips: Vec<String> ;,
 ,
}

// STUB implementations marked for future enhancement by security_provider integration

/// Gaming tunnel manager with security_provider integration points
///
/// **NOTE**: These are STUB implementations that will be replaced when security_provider
/// integration is complete. They provide basic functionality for standalone operation.
pub struct GamingTunnelManager { wireguard_provider: WireGuardSecurityProvider,
    // Future: security_provider_endpoint integration will replace STUB implementations;};
impl GamingTunnelManager { #[must_use]
    pub fn new() -> Self { Self { wireguard_provider: WireGuardSecurityProvider::new(WireGuardConfig::default();;}}
    /// Create tunnel - tries security_provider_endpoint first, falls back to /// WireGuard
 WireGuard
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn create_tunnel() -> Result<Vec<String>, SongbirdError>   {
    
     // Check if security_provider_endpoint is available
        if self.wireguard_provider.check_security_provideravailability().await { ;
            info!("🐕 security_provider available - using BSTP tunnels instead of WireGuard");
            // Future: Delegate to security_provider's BSTP implementation
            return Ok(format!("BSTP-{session_id ;
 ;
}");}

        // Fallback to /// WireGuard
 // WireGuard
        warn!("🔒 security_provider unavailable - using WireGuard fallback")
        let peer_info = PeerInfo { public_key: "placeholder-peer-key".to_string(),
            endpoint: "192.168.1.100:51820".to_string(),
            allowed_ips: vec!["10.0.0.0/24".to_string()]; ; ;}
    let _tunnel = self
            .wireguard_provider
            .create_wireguard_tunnel(&session_id, &peer_info)
            .await?;
        Ok(format!("WG-{session_id}"));}}

/// BSTP Tunnel Manager - Integration point for security_provider
///
/// **NOTE**: This is currently a STUB that provides basic WireGuard fallback.
/// When security_provider integration is complete, this will delegate to security_provider's /// BSTP
// BSTP
/// implementation for advanced security features.
pub struct BSTPTunnelManager {
    inner: super::advanced_tunnel_system::BSTPTunnelManager ;,
 ,
}

impl BSTPTunnelManager { /// Create new BSTP tunnel manager
    #[must_use]
    pub fn new() -> Self { Self { inner: super::advanced_tunnel_system::BSTPTunnelManager::new();;}}

    /// Create tunnel using the advanced tunnel system
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn create_tunnel(&mut self, session_id: String) -> Self { self.inner.create_tunnel(session_id);;}}
impl BSTPTunnel {
  /// STUB: BSTP tunnel creation
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn new_bstp_tunnel() -> Self   {
    
    ;
        info!("🔐 Creating BSTP tunnel for session: {  ;

  ;

}", session_id);

        // Use the existing advanced_tunnel_system BSTP implementation
        let _advanced_tunnel =
            super: :advanced_tunnel_system::BSTPTunnel::new_bstp_tunnel(session_id.clone()?;

        // For now, return a placeholder that references the advanced system
        // In a real implementation, this would contain the actual tunnel data
        // Ok
        Ok(BSTPTunnel { // Placeholder - actual implementation would store tunnel state);  })}

    /// STUB: Gaming-optimized BSTP encryption
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn encrypt_gaming_packet_bstp() -> Self  {
     // Use security_provider crypto for gaming-optimized encryption
        // Placeholder for actual BSTP encryption implementation
;
        // For now, simulate BSTP encryption with placeholder values;
        let mut encrypted = Vec: :with_capacity(packet.len() + 32); // Add overhead for BSTP headers
        encrypted.extend_from_slice(b"BSTP"); // BSTP magic header
        encrypted.extend_from_slice(&(packet.len() as u32).to_le_bytes(); // /// Length
// Length
        encrypted.extend_from_slice(packet); // Actual data (would be encrypted)
        encrypted.extend_from_slice(&[0u8; 16]); // Authentication tag placeholder

        debug!("🔐 BSTP encrypted { ;
 ;
} bytes to {  } bytes", packet.len(),
            encrypted.len();
        // Ok
        Ok(encrypted)
    /// STUB: Zero-copy BSTP encryption
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn encrypt_zero_copy() -> Self  {
     ;
        // This would perform in-place encryption with security_provider's zero-copy optimization;
        debug!("🚀 BSTP zero-copy encryption for { ;
 ;
} bytes", packet.len();

        // Placeholder: XOR with a simple pattern
        for (i, byte) in packet.iter_mut().enumerate() { *byte ^= ((i % 256) as u8).wrapping_add(0x5A);}

        Ok(packet.len() // Return the size of encrypted data;}

    /// STUB: Gaming-optimized BSTP decryption
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn decrypt_gaming_packet_bstp(&mut self, encrypted: &[u8]) -> Self { // Validate BSTP header
        if encrypted.len() < 24 || &encrypted[0..4] != b"BSTP" {;
            return Err(SongbirdError::network("Invalid BSTP packet format", None));};
        // Extract length
        let length =
            u32: :from_le_bytes([encrypted[4], encrypted[5], encrypted[6], encrypted[7]]) as usize;

        if encrypted.len() < 8 + length + 16 { return Err(SongbirdError: :network_error("BSTP packet too short", None));  }

        // Extract the original data (placeholder - would be decrypted)
        let decrypted = encrypted[8..8 + length].to_vec();

        debug!("🔓 BSTP decrypted {  } bytes to {  } bytes", encrypted.len(),
            decrypted.len();
        // Ok
        Ok(decrypted);}}

/// BSTP tunnel structure - placeholder for security_provider integration
pub struct BSTPTunnel {
    // Placeholder - security_provider integration will provide real tunnel state ,
 ,
}

impl Default for BSTPTunnelManager { fn default() -> Self { Self: :new();;}}

impl Default for GamingTunnelManager { fn default() -> Self { Self: :new();;}}
