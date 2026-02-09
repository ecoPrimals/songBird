//! NFC configuration with BearDog integration

use std::path::PathBuf;
use std::time::Duration;

/// NFC protocol configuration
///
/// All crypto operations delegated to BearDog - zero hardcoded secrets
#[derive(Debug, Clone)]
pub struct NfcConfig {
    /// BearDog socket path for crypto operations
    pub beardog_socket: PathBuf,
    
    /// Exchange timeout (including timing protection delays)
    pub exchange_timeout: Duration,
    
    /// Enable timing protection (constant-time operations)
    pub timing_protection: bool,
    
    /// Target exchange duration for timing protection
    pub target_exchange_duration: Duration,
    
    /// Maximum random delay for timing protection
    pub max_random_delay: Duration,
    
    /// Enable connection validation
    pub validate_connection: bool,
}

impl Default for NfcConfig {
    fn default() -> Self {
        Self {
            // BearDog socket discovered at runtime (no hardcoding)
            beardog_socket: Self::discover_beardog_socket(),
            
            exchange_timeout: Duration::from_secs(30),
            timing_protection: true,
            target_exchange_duration: Duration::from_secs(10),
            max_random_delay: Duration::from_millis(500),
            validate_connection: true,
        }
    }
}

impl NfcConfig {
    /// Create new configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set BearDog socket path
    #[must_use]
    pub fn with_beardog_socket(mut self, socket: PathBuf) -> Self {
        self.beardog_socket = socket;
        self
    }
    
    /// Set exchange timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.exchange_timeout = timeout;
        self
    }
    
    /// Enable/disable timing protection
    #[must_use]
    pub fn with_timing_protection(mut self, enabled: bool) -> Self {
        self.timing_protection = enabled;
        self
    }
    
    /// Discover BearDog socket at runtime (primal self-knowledge only)
    ///
    /// Resolution order:
    /// 1. BEARDOG_SOCKET environment variable
    /// 2. SONGBIRD_SECURITY_PROVIDER environment variable
    /// 3. XDG runtime directory + beardog.sock
    /// 4. /tmp/biomeos/beardog.sock (fallback)
    fn discover_beardog_socket() -> PathBuf {
        // 1. Explicit BearDog socket
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            return PathBuf::from(socket);
        }
        
        // 2. Security provider (generic)
        if let Ok(socket) = std::env::var("SONGBIRD_SECURITY_PROVIDER") {
            return PathBuf::from(socket);
        }
        
        // 3. XDG runtime directory
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = PathBuf::from(xdg_runtime).join("biomeos").join("beardog.sock");
            if socket.exists() {
                return socket;
            }
        }
        
        // 4. Fallback (platform-specific)
        #[cfg(unix)]
        {
            PathBuf::from("/tmp/biomeos/beardog.sock")
        }
        
        #[cfg(not(unix))]
        {
            PathBuf::from("beardog.sock")
        }
    }
}
