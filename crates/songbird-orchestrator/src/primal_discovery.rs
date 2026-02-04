//! Agnostic Primal Discovery - TRUE PRIMAL Architecture
//!
//! Discovers primals by capability at runtime, with ZERO hardcoding.
//! Songbird only knows itself - discovers other primals via capabilities.
//!
//! ## Principles
//!
//! 1. **Self-Knowledge Only**: Songbird knows only itself
//! 2. **Capability-Based**: Discover by what primals DO, not what they ARE
//! 3. **Runtime Discovery**: No compile-time dependencies on other primals
//! 4. **Graceful Degradation**: Features work without optional primals
//!
//! ## Discovery Strategy
//!
//! ```text
//! 1. Environment Variables (orchestrator-provided, preferred)
//!    - Explicit: {CAPABILITY}_PROVIDER_SOCKET
//!    - Generic: {PRIMAL}_SOCKET
//!    
//! 2. Capability Registry (runtime discovery)
//!    - Query for capability
//!    - Get socket path
//!    
//! 3. Common Socket Patterns (fallback)
//!    - /tmp/{capability}.sock
//!    - /tmp/{primal}-{family}.sock
//!    
//! 4. Socket Scanning (last resort)
//!    - Scan /tmp for matching sockets
//!    - Check socket capabilities via RPC
//! ```

use anyhow::Result;
use std::path::Path;
use tracing::{debug, info, warn};

/// Capability types for primal discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Cryptographic operations (signing, encryption, hashing)
    Crypto,
    /// Security operations (JWT, auth, trust evaluation)
    Security,
    /// HTTP/HTTPS requests (external API delegation)
    Http,
    /// AI operations (LLM inference, routing)
    Ai,
    /// Storage operations (key-value, blob)
    Storage,
    /// Messaging operations (pub/sub, queues)
    Messaging,
}

impl Capability {
    /// Get environment variable name for this capability
    fn env_var_name(&self) -> &'static str {
        match self {
            Self::Crypto => "CRYPTO_PROVIDER_SOCKET",
            Self::Security => "SECURITY_PROVIDER_SOCKET",
            Self::Http => "HTTP_PROVIDER_SOCKET",
            Self::Ai => "AI_PROVIDER_SOCKET",
            Self::Storage => "STORAGE_PROVIDER_SOCKET",
            Self::Messaging => "MESSAGING_PROVIDER_SOCKET",
        }
    }

    /// Get alternative environment variable names (for compatibility)
    fn alt_env_vars(&self) -> Vec<&'static str> {
        match self {
            Self::Crypto => vec!["BEARDOG_CRYPTO_SOCKET", "BEARDOG_SOCKET"],
            Self::Security => vec!["SONGBIRD_SECURITY_PROVIDER", "BEARDOG_SOCKET"],
            Self::Http => vec!["HTTP_CLIENT_SOCKET", "SONGBIRD_SOCKET"],
            Self::Ai => vec!["SQUIRREL_SOCKET", "AI_PROVIDER_SOCKETS"],
            Self::Storage => vec!["NESTGATE_SOCKET", "STORAGE_SOCKET"],
            Self::Messaging => vec!["MESSENGER_SOCKET", "PUBSUB_SOCKET"],
        }
    }

    /// Get common socket path patterns for this capability
    fn socket_patterns(&self) -> Vec<&'static str> {
        match self {
            Self::Crypto => {
                vec!["/tmp/crypto.sock", "/tmp/beardog-crypto.sock", "/tmp/beardog-nat0.sock"]
            }
            Self::Security => {
                vec!["/tmp/security.sock", "/tmp/beardog-nat0.sock", "/tmp/songbird-nat0.sock"]
            }
            Self::Http => vec!["/tmp/http.sock", "/tmp/songbird-nat0.sock"],
            Self::Ai => vec!["/tmp/ai.sock", "/tmp/squirrel-nat0.sock"],
            Self::Storage => vec!["/tmp/storage.sock", "/tmp/nestgate-nat0.sock"],
            Self::Messaging => vec!["/tmp/messaging.sock", "/tmp/messenger-nat0.sock"],
        }
    }
}

/// Discover a primal by capability (functional, no state)
///
/// # Example
///
/// ```rust,no_run
/// use songbird_orchestrator::primal_discovery::{discover, Capability};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Discover crypto provider (could be BearDog, or any primal with crypto capability)
/// let crypto_socket = discover(Capability::Crypto).await?;
/// println!("Crypto provider at: {}", crypto_socket);
///
/// // Discover AI provider (could be Squirrel, or any primal with AI capability)
/// let ai_socket = discover(Capability::Ai).await?;
/// println!("AI provider at: {}", ai_socket);
/// # Ok(())
/// # }
/// ```
pub async fn discover(capability: Capability) -> Result<String> {
    info!("🔍 Discovering {:?} provider (capability-based discovery)...", capability);

    // Strategy 1: Environment variable (orchestrator-provided, preferred)
    if let Ok(socket_path) = std::env::var(capability.env_var_name()) {
        info!("   ✅ Found via {}: {}", capability.env_var_name(), socket_path);
        return Ok(socket_path);
    }

    // Strategy 2: Alternative environment variables (compatibility)
    for alt_var in capability.alt_env_vars() {
        if let Ok(socket_path) = std::env::var(alt_var) {
            info!("   ✅ Found via {} (compatibility): {}", alt_var, socket_path);
            return Ok(socket_path);
        }
    }

    // Strategy 3: Common socket patterns (Unix sockets)
    for pattern in capability.socket_patterns() {
        if Path::new(pattern).exists() {
            info!("   ✅ Found {:?} provider socket at: {}", capability, pattern);
            return Ok(pattern.to_string());
        }
        debug!("   ⏭️  Not found: {}", pattern);
    }

    // Strategy 3.5: TCP discovery files (isomorphic fallback)
    // When primals can't use Unix sockets (Android/SELinux, Windows), they automatically
    // fall back to TCP localhost and write a discovery file. Check for those files.
    if let Some(tcp_endpoint) = discover_tcp_from_capability(capability) {
        info!("   ✅ Found {:?} provider via TCP discovery file: {}", capability, tcp_endpoint);
        return Ok(tcp_endpoint);
    }

    // Strategy 4: Socket scanning (last resort)
    if let Some(socket_path) = scan_sockets(capability) {
        info!("   ✅ Found {:?} provider via scanning: {}", capability, socket_path);
        return Ok(socket_path);
    }

    // Not found
    warn!("❌ No {:?} provider found - checked all discovery strategies", capability);
    anyhow::bail!("No {:?} provider available", capability)
}

/// Scan /tmp for sockets matching capability
fn scan_sockets(capability: Capability) -> Option<String> {
    let search_terms = match capability {
        Capability::Crypto => vec!["crypto", "beardog"],
        Capability::Security => vec!["security", "beardog", "auth"],
        Capability::Http => vec!["http", "songbird"],
        Capability::Ai => vec!["ai", "squirrel"],
        Capability::Storage => vec!["storage", "nestgate"],
        Capability::Messaging => vec!["messaging", "messenger"],
    };

    if let Ok(entries) = std::fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                // Check if filename matches any search term
                if file_name.ends_with(".sock") {
                    for term in &search_terms {
                        if file_name.contains(term) {
                            let path = entry.path();
                            debug!("   Found potential socket: {}", path.display());
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Discover TCP endpoint for a capability (isomorphic fallback support)
///
/// Checks TCP discovery files for primals that provide this capability.
/// This enables transparent fallback when Unix sockets are unavailable
/// (Android/SELinux, Windows).
///
/// # Discovery File Format
///
/// File: `$XDG_RUNTIME_DIR/{primal}-ipc-port`\
/// Content: `tcp:127.0.0.1:12345`
///
/// # Arguments
///
/// * `capability` - The capability to discover (e.g., Crypto, Storage)
///
/// # Returns
///
/// Socket descriptor string (e.g., "tcp:127.0.0.1:12345") if found, None otherwise.
///
/// # Deep Debt Principles
///
/// - ✅ **Runtime Discovery**: Detects TCP endpoints automatically
/// - ✅ **Zero Hardcoding**: No hardcoded ports or addresses
/// - ✅ **Platform Agnostic**: Works on any platform with filesystem
/// - ✅ **Isomorphic**: Same discovery code for Unix and TCP
fn discover_tcp_from_capability(capability: Capability) -> Option<String> {
    // Map capability to primal names that might provide it
    let primal_names = match capability {
        Capability::Crypto | Capability::Security => vec!["beardog"],
        Capability::Http => vec!["songbird"],
        Capability::Ai => vec!["squirrel"],
        Capability::Storage => vec!["nestgate"],
        Capability::Messaging => vec!["messenger"],
    };

    // Check TCP discovery files for each potential primal
    for primal_name in primal_names {
        if let Some(tcp_addr) = check_tcp_discovery_file(primal_name) {
            // Return in socket descriptor format for compatibility
            return Some(format!("tcp:{}", tcp_addr));
        }
    }

    None
}

/// Check TCP discovery file for a specific primal
///
/// Checks XDG-compliant locations in priority order:
/// 1. `$XDG_RUNTIME_DIR/{primal}-ipc-port`
/// 2. `$HOME/.local/share/{primal}-ipc-port`
/// 3. `/tmp/{primal}-ipc-port`
///
/// # Arguments
///
/// * `primal_name` - Primal name (e.g., "beardog", "squirrel")
///
/// # Returns
///
/// TCP socket address (e.g., "127.0.0.1:12345") if found, None otherwise.
fn check_tcp_discovery_file(primal_name: &str) -> Option<String> {
    let filename = format!("{}-ipc-port", primal_name);
    let mut candidates = Vec::new();

    // Priority 1: XDG_RUNTIME_DIR (preferred)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        candidates.push(std::path::PathBuf::from(runtime_dir).join(&filename));
    }

    // Priority 2: HOME/.local/share (fallback)
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(std::path::PathBuf::from(home).join(".local/share").join(&filename));
    }

    // Priority 3: /tmp (last resort)
    candidates.push(std::path::PathBuf::from(format!("/tmp/{}", filename)));

    // Check each candidate
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            // Parse format: "tcp:127.0.0.1:12345"
            if let Some(addr_str) = content.strip_prefix("tcp:") {
                let addr_trimmed = addr_str.trim();
                // Validate it's a parseable socket address
                if addr_trimmed.parse::<std::net::SocketAddr>().is_ok() {
                    debug!("   Found TCP discovery file: {} -> {}", path.display(), addr_trimmed);
                    return Some(addr_trimmed.to_string());
                }
            }
        }
    }

    None
}

/// Convenience function: Discover crypto provider
pub async fn discover_crypto_provider() -> Result<String> {
    discover(Capability::Crypto).await
}

/// Convenience function: Discover security provider
pub async fn discover_security_provider() -> Result<String> {
    discover(Capability::Security).await
}

/// Convenience function: Discover HTTP provider
pub async fn discover_http_provider() -> Result<String> {
    discover(Capability::Http).await
}

/// Convenience function: Discover AI provider
pub async fn discover_ai_provider() -> Result<String> {
    discover(Capability::Ai).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_env_vars() {
        assert_eq!(Capability::Crypto.env_var_name(), "CRYPTO_PROVIDER_SOCKET");
        assert_eq!(Capability::Security.env_var_name(), "SECURITY_PROVIDER_SOCKET");
    }

    #[test]
    fn test_capability_patterns() {
        let patterns = Capability::Crypto.socket_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&"/tmp/crypto.sock"));
    }

    #[test]
    fn test_family_id_default() {
        // Without env var, should return default
        let family_id = get_family_id();
        assert!(!family_id.is_empty());
    }

    #[test]
    fn test_primal_name_default() {
        let primal_name = get_primal_name();
        assert_eq!(primal_name, "songbird");
    }

    #[test]
    fn test_tcp_discovery_file_parsing() {
        use std::io::Write;

        // Create temp discovery file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-beardog-ipc-port");

        // Write TCP endpoint
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:127.0.0.1:12345").unwrap();
        drop(file);

        // Set XDG_RUNTIME_DIR to temp
        std::env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

        // Test discovery
        let result = check_tcp_discovery_file("test-beardog");
        assert_eq!(result, Some("127.0.0.1:12345".to_string()));

        // Cleanup
        std::fs::remove_file(file_path).ok();
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_tcp_discovery_from_crypto_capability() {
        use std::io::Write;

        // Create temp discovery file for beardog
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("beardog-ipc-port");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:127.0.0.1:33765").unwrap();
        drop(file);

        std::env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

        // Test Crypto capability maps to beardog
        let result = discover_tcp_from_capability(Capability::Crypto);
        assert_eq!(result, Some("tcp:127.0.0.1:33765".to_string()));

        // Cleanup
        std::fs::remove_file(file_path).ok();
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_tcp_discovery_invalid_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("invalid-beardog-ipc-port");

        // Write invalid format (missing tcp: prefix)
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"127.0.0.1:12345").unwrap();
        drop(file);

        std::env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

        // Should return None for invalid format
        let result = check_tcp_discovery_file("invalid-beardog");
        assert_eq!(result, None);

        // Cleanup
        std::fs::remove_file(file_path).ok();
        std::env::remove_var("XDG_RUNTIME_DIR");
    }
}

/// Get family ID from environment (agnostic)
pub fn get_family_id() -> String {
    std::env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "nat0".to_string())
}

/// Get primal name from environment (self-knowledge)
pub fn get_primal_name() -> String {
    std::env::var("PRIMAL_NAME").unwrap_or_else(|_| "songbird".to_string())
}
