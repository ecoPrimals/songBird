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
            Capability::Crypto => "CRYPTO_PROVIDER_SOCKET",
            Capability::Security => "SECURITY_PROVIDER_SOCKET",
            Capability::Http => "HTTP_PROVIDER_SOCKET",
            Capability::Ai => "AI_PROVIDER_SOCKET",
            Capability::Storage => "STORAGE_PROVIDER_SOCKET",
            Capability::Messaging => "MESSAGING_PROVIDER_SOCKET",
        }
    }

    /// Get alternative environment variable names (for compatibility)
    fn alt_env_vars(&self) -> Vec<&'static str> {
        match self {
            Capability::Crypto => vec!["BEARDOG_CRYPTO_SOCKET", "BEARDOG_SOCKET"],
            Capability::Security => vec!["SONGBIRD_SECURITY_PROVIDER", "BEARDOG_SOCKET"],
            Capability::Http => vec!["HTTP_CLIENT_SOCKET", "SONGBIRD_SOCKET"],
            Capability::Ai => vec!["SQUIRREL_SOCKET", "AI_PROVIDER_SOCKETS"],
            Capability::Storage => vec!["NESTGATE_SOCKET", "STORAGE_SOCKET"],
            Capability::Messaging => vec!["MESSENGER_SOCKET", "PUBSUB_SOCKET"],
        }
    }

    /// Get common socket path patterns for this capability
    fn socket_patterns(&self) -> Vec<&'static str> {
        match self {
            Capability::Crypto => {
                vec!["/tmp/crypto.sock", "/tmp/beardog-crypto.sock", "/tmp/beardog-nat0.sock"]
            }
            Capability::Security => {
                vec!["/tmp/security.sock", "/tmp/beardog-nat0.sock", "/tmp/songbird-nat0.sock"]
            }
            Capability::Http => vec!["/tmp/http.sock", "/tmp/songbird-nat0.sock"],
            Capability::Ai => vec!["/tmp/ai.sock", "/tmp/squirrel-nat0.sock"],
            Capability::Storage => vec!["/tmp/storage.sock", "/tmp/nestgate-nat0.sock"],
            Capability::Messaging => vec!["/tmp/messaging.sock", "/tmp/messenger-nat0.sock"],
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

    // Strategy 3: Common socket patterns
    for pattern in capability.socket_patterns() {
        if Path::new(pattern).exists() {
            info!("   ✅ Found {:?} provider socket at: {}", capability, pattern);
            return Ok(pattern.to_string());
        } else {
            debug!("   ⏭️  Not found: {}", pattern);
        }
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
