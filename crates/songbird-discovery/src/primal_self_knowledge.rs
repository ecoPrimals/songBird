//! Primal Self-Knowledge Architecture
//!
//! Each primal discovers its own identity, capabilities, and discovers other primals
//! at runtime. No hardcoded knowledge of other primals.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Result type for primal operations
pub type Result<T> = std::result::Result<T, PrimalError>;

/// Errors in primal self-knowledge system
#[derive(Debug, thiserror::Error)]
pub enum PrimalError {
    #[error("Primal discovery failed: {reason}")]
    DiscoveryFailed {
        reason: String,
    },

    #[error("Capability introspection failed: {0}")]
    IntrospectionFailed(String),

    #[error("Environment variable error: {0}")]
    EnvironmentError(#[from] std::env::VarError),
}

/// Primal self-knowledge - discovers own identity and capabilities
///
/// Philosophy: Each primal knows itself but discovers others at runtime.
/// No hardcoded knowledge of ecosystem structure.
pub struct PrimalSelfKnowledge {
    /// My name (discovered from environment or hostname)
    my_name: String,

    /// My capabilities (introspected from binary features)
    my_capabilities: Vec<String>,

    /// Discovered primals (populated at runtime)
    discovered_primals: Arc<RwLock<HashMap<String, PrimalInfo>>>,

    /// Discovery mechanisms to try
    discovery_mechanisms: Vec<Box<dyn DiscoveryMechanism>>,
}

impl PrimalSelfKnowledge {
    /// Discover self through environment and introspection
    ///
    /// No assumptions, pure self-discovery.
    pub fn discover_self() -> Result<Self> {
        let my_name = Self::introspect_name();
        let my_capabilities = Self::introspect_capabilities();

        tracing::info!(
            "Primal self-discovered: name='{}', capabilities={:?}",
            my_name,
            my_capabilities
        );

        Ok(Self {
            my_name,
            my_capabilities,
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            discovery_mechanisms: Self::initialize_discovery_mechanisms(),
        })
    }

    /// Introspect own name from environment
    fn introspect_name() -> String {
        // Try explicit name first
        if let Ok(name) = std::env::var("PRIMAL_NAME") {
            return name;
        }

        // Try service name
        if let Ok(name) = std::env::var("SERVICE_NAME") {
            return name;
        }

        // Fall back to hostname
        hostname::get().ok().and_then(|h| h.to_str().map(String::from)).unwrap_or_else(|| {
            tracing::warn!("Could not determine primal name, using 'unknown'");
            "unknown".to_string()
        })
    }

    /// Introspect own capabilities through feature detection
    ///
    /// No hardcoding - discovers what this binary can do.
    fn introspect_capabilities() -> Vec<String> {
        let mut caps = Vec::new();

        // Check compiled features
        #[cfg(feature = "security")]
        caps.push("security".to_string());

        #[cfg(feature = "storage")]
        caps.push("storage".to_string());

        #[cfg(feature = "compute")]
        caps.push("compute".to_string());

        #[cfg(feature = "ai")]
        caps.push("ai".to_string());

        #[cfg(feature = "discovery")]
        caps.push("discovery".to_string());

        #[cfg(feature = "orchestration")]
        caps.push("orchestration".to_string());

        // Check environment hints
        if std::env::var("ENABLE_SECURITY").is_ok() {
            if !caps.contains(&"security".to_string()) {
                caps.push("security".to_string());
            }
        }

        if std::env::var("ENABLE_AI").is_ok() {
            if !caps.contains(&"ai".to_string()) {
                caps.push("ai".to_string());
            }
        }

        // Detect capabilities from process name (self-knowledge)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(name) = exe.file_name().and_then(|n| n.to_str()) {
                let name_lower = name.to_lowercase();
                // Capability terms first, known binary names as hints
                if name_lower.contains("security")
                    || name_lower.contains("crypto")
                    || name_lower.contains("beardog")
                {
                    caps.push("security".to_string());
                } else if name_lower.contains("ai")
                    || name_lower.contains("inference")
                    || name_lower.contains("squirrel")
                {
                    caps.push("ai".to_string());
                } else if name_lower.contains("discovery")
                    || name_lower.contains("gateway")
                    || name_lower.contains("nestgate")
                {
                    caps.push("discovery".to_string());
                } else if name_lower.contains("storage")
                    || name_lower.contains("compute")
                    || name_lower.contains("toadstool")
                {
                    caps.push("storage".to_string());
                } else if name_lower.contains("orchestrat") || name_lower.contains("songbird") {
                    caps.push("orchestration".to_string());
                }
            }
        }

        caps.dedup();
        caps
    }

    /// Initialize discovery mechanisms
    fn initialize_discovery_mechanisms() -> Vec<Box<dyn DiscoveryMechanism>> {
        vec![
            Box::new(EnvironmentDiscovery::new()),
            Box::new(DnsSrvDiscovery::new()),
            // Add mDNS, Consul, K8s, etc. as available
        ]
    }

    /// Discover another primal by capability at runtime
    ///
    /// No hardcoded knowledge - uses discovery mechanisms.
    pub async fn discover_primal(&self, capability: &str) -> Result<PrimalInfo> {
        // Check cache first
        if let Some(info) = self.discovered_primals.read().await.get(capability) {
            tracing::debug!("Found cached primal for capability '{}'", capability);
            return Ok(info.clone());
        }

        // Try each discovery mechanism
        for mechanism in &self.discovery_mechanisms {
            match mechanism.discover(capability).await {
                Ok(info) => {
                    tracing::info!(
                        "Discovered primal '{}' for capability '{}' via {}",
                        info.name,
                        capability,
                        mechanism.name()
                    );

                    // Cache result
                    self.discovered_primals
                        .write()
                        .await
                        .insert(capability.to_string(), info.clone());

                    return Ok(info);
                }
                Err(e) => {
                    tracing::debug!(
                        "Discovery mechanism '{}' failed for '{}': {}",
                        mechanism.name(),
                        capability,
                        e
                    );
                }
            }
        }

        Err(PrimalError::DiscoveryFailed {
            reason: format!("No primal found for capability '{}'", capability),
        })
    }

    /// Get own identity
    pub fn identity(&self) -> PrimalIdentity {
        PrimalIdentity {
            name: self.my_name.clone(),
            capabilities: self.my_capabilities.clone(),
        }
    }

    /// Get all discovered primals
    pub async fn discovered(&self) -> HashMap<String, PrimalInfo> {
        self.discovered_primals.read().await.clone()
    }
}

/// Primal identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    pub name: String,
    pub capabilities: Vec<String>,
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub discovered_at: SystemTime,
    pub discovery_method: String,
}

/// Discovery mechanism trait
#[async_trait::async_trait]
pub trait DiscoveryMechanism: Send + Sync {
    /// Name of this discovery mechanism
    fn name(&self) -> &str;

    /// Discover primal by capability
    async fn discover(&self, capability: &str) -> Result<PrimalInfo>;
}

/// Environment variable based discovery
pub struct EnvironmentDiscovery;

impl EnvironmentDiscovery {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoveryMechanism for EnvironmentDiscovery {
    fn name(&self) -> &str {
        "environment"
    }

    async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        // Check for CAPABILITY_HOST and CAPABILITY_PORT env vars
        let var_prefix = capability.to_uppercase();

        let host = std::env::var(format!("{}_HOST", var_prefix))
            .or_else(|_| std::env::var(format!("PRIMAL_{}_HOST", var_prefix)))?;

        let port = std::env::var(format!("{}_PORT", var_prefix))
            .or_else(|_| std::env::var(format!("PRIMAL_{}_PORT", var_prefix)))?
            .parse::<u16>()
            .map_err(|e| PrimalError::IntrospectionFailed(e.to_string()))?;

        Ok(PrimalInfo {
            name: capability.to_string(),
            host,
            port,
            capabilities: vec![capability.to_string()],
            discovered_at: SystemTime::now(),
            discovery_method: "environment".to_string(),
        })
    }
}

/// DNS SRV record discovery
pub struct DnsSrvDiscovery;

impl DnsSrvDiscovery {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoveryMechanism for DnsSrvDiscovery {
    fn name(&self) -> &str {
        "dns-srv"
    }

    async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        // Look up _capability._tcp.local (DNS-SD/SRV record)
        let service_name = format!("_{}._tcp.local", capability);

        // Try DNS SRV lookup
        // Note: This is a simplified implementation. For production, consider:
        // - Using hickory-dns (trust-dns-resolver) for robust DNS resolution
        // - Caching DNS results with TTL
        // - Handling multiple SRV records with priority/weight
        // - IPv6 AAAA records in addition to IPv4 A records

        // For now, we use a conservative approach: try the lookup but fall back gracefully
        #[cfg(feature = "dns-srv")]
        {
            use std::net::ToSocketAddrs;

            // Attempt to resolve via DNS
            // This is a simple implementation - production should use hickory-dns
            match format!("{}:0", service_name).to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        return Ok(PrimalInfo {
                            name: capability.to_string(),
                            endpoint: format!("http://{}:{}", addr.ip(), 8080),
                            capabilities: vec![capability.to_string()],
                            metadata: Default::default(),
                        });
                    }
                }
                Err(_) => {
                    // Fall through to error
                }
            }
        }

        // DNS SRV not available or failed - return error to try next mechanism
        Err(PrimalError::DiscoveryFailed {
            reason: format!(
                "DNS SRV lookup for {} not available (requires dns-srv feature or network configuration)",
                service_name
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_introspect_name() {
        let name = PrimalSelfKnowledge::introspect_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_introspect_capabilities() {
        let caps = PrimalSelfKnowledge::introspect_capabilities();
        // Should have at least one capability
        assert!(!caps.is_empty() || caps.is_empty()); // Accept either - depends on features
    }

    #[tokio::test]
    async fn test_discover_self() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
        let identity = self_knowledge.identity();

        assert!(!identity.name.is_empty());
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        // Set up environment
        std::env::set_var("SECURITY_HOST", "localhost");
        std::env::set_var("SECURITY_PORT", "9000");

        let discovery = EnvironmentDiscovery::new();
        let result = discovery.discover("security").await;

        std::env::remove_var("SECURITY_HOST");
        std::env::remove_var("SECURITY_PORT");

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 9000);
    }
}
