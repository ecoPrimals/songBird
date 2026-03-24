// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal Self-Knowledge Architecture
//!
//! Each primal discovers its own identity, capabilities, and discovers other primals
//! at runtime. No hardcoded knowledge of other primals.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::VarError;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Result type for primal operations
pub type Result<T> = std::result::Result<T, PrimalError>;

/// Injected process environment lookup, shared across discovery tasks.
type EnvVarFn = Arc<dyn Fn(&str) -> std::result::Result<String, VarError> + Send + Sync>;

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
        Self::discover_self_with(|k| std::env::var(k))
    }

    /// Same as [`discover_self`](Self::discover_self) with an injectable env reader (tests).
    pub fn discover_self_with(
        env: impl Fn(&str) -> std::result::Result<String, VarError> + Send + Sync + 'static,
    ) -> Result<Self> {
        let env = Arc::new(env);
        let my_name = Self::introspect_name_with(|k| env(k));
        let my_capabilities = Self::introspect_capabilities_with(|k| env(k));

        tracing::info!(
            "Primal self-discovered: name='{}', capabilities={:?}",
            my_name,
            my_capabilities
        );

        let env_mech = Arc::clone(&env);
        Ok(Self {
            my_name,
            my_capabilities,
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            discovery_mechanisms: vec![
                Box::new(EnvInjectedDiscovery {
                    get_var: env_mech,
                }),
                Box::new(DnsSrvDiscovery::new()),
            ],
        })
    }

    /// Introspect own name from environment
    #[must_use]
    pub fn introspect_name() -> String {
        Self::introspect_name_with(|k| std::env::var(k))
    }

    fn introspect_name_with(env: impl Fn(&str) -> std::result::Result<String, VarError>) -> String {
        // Try explicit name first
        if let Ok(name) = env("PRIMAL_NAME") {
            return name;
        }

        // Try service name
        if let Ok(name) = env("SERVICE_NAME") {
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
    #[must_use]
    pub fn introspect_capabilities() -> Vec<String> {
        Self::introspect_capabilities_with(|k| std::env::var(k))
    }

    fn introspect_capabilities_with(
        env: impl Fn(&str) -> std::result::Result<String, VarError>,
    ) -> Vec<String> {
        let mut caps = vec![
            #[cfg(feature = "security")]
            "security".to_string(),
            #[cfg(feature = "storage")]
            "storage".to_string(),
            #[cfg(feature = "compute")]
            "compute".to_string(),
            #[cfg(feature = "ai")]
            "ai".to_string(),
            #[cfg(feature = "discovery")]
            "discovery".to_string(),
            #[cfg(feature = "orchestration")]
            "orchestration".to_string(),
        ];

        // Check environment hints
        if env("ENABLE_SECURITY").is_ok() && !caps.contains(&"security".to_string()) {
            caps.push("security".to_string());
        }

        if env("ENABLE_AI").is_ok() && !caps.contains(&"ai".to_string()) {
            caps.push("ai".to_string());
        }

        // Detect capabilities from process name (self-knowledge)
        if let Ok(exe) = std::env::current_exe()
            && let Some(name) = exe.file_name().and_then(|n| n.to_str())
        {
            let name_lower = name.to_lowercase();
            if name_lower.contains("security")
                || name_lower.contains("crypto")
                || name_lower.contains("auth")
            {
                caps.push("security".to_string());
            } else if name_lower.contains("ai")
                || name_lower.contains("inference")
                || name_lower.contains("ml")
            {
                caps.push("ai".to_string());
            } else if name_lower.contains("discovery")
                || name_lower.contains("gateway")
                || name_lower.contains("registry")
            {
                caps.push("discovery".to_string());
            } else if name_lower.contains("storage")
                || name_lower.contains("data")
                || name_lower.contains("persist")
            {
                caps.push("storage".to_string());
            } else if name_lower.contains("compute")
                || name_lower.contains("worker")
                || name_lower.contains("exec")
            {
                caps.push("compute".to_string());
            } else if name_lower.contains("orchestrat") || name_lower.contains("coordinat") {
                caps.push("orchestration".to_string());
            }
        }

        caps.dedup();
        caps
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
            reason: format!("No primal found for capability '{capability}'"),
        })
    }

    /// Get own identity
    #[must_use]
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

/// Uses injected env lookup for [`PrimalSelfKnowledge::discover_self_with`].
struct EnvInjectedDiscovery {
    get_var: EnvVarFn,
}

#[async_trait::async_trait]
impl DiscoveryMechanism for EnvInjectedDiscovery {
    fn name(&self) -> &'static str {
        "environment"
    }

    async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        EnvironmentDiscovery::discover_with(capability, |k| (self.get_var)(k)).await
    }
}

impl Default for EnvironmentDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentDiscovery {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Discover `host` / `port` from env using an injectable reader (tests).
    pub async fn discover_with<F>(capability: &str, get_var: F) -> Result<PrimalInfo>
    where
        F: Fn(&str) -> std::result::Result<String, VarError> + Send,
    {
        let var_prefix = capability.to_uppercase();

        let host = get_var(&format!("{var_prefix}_HOST"))
            .or_else(|_| get_var(&format!("PRIMAL_{var_prefix}_HOST")))?;

        let port = get_var(&format!("{var_prefix}_PORT"))
            .or_else(|_| get_var(&format!("PRIMAL_{var_prefix}_PORT")))?
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

#[async_trait::async_trait]
impl DiscoveryMechanism for EnvironmentDiscovery {
    fn name(&self) -> &'static str {
        "environment"
    }

    async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        Self::discover_with(capability, |k| std::env::var(k)).await
    }
}

/// DNS SRV record discovery
pub struct DnsSrvDiscovery;

impl Default for DnsSrvDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsSrvDiscovery {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DiscoveryMechanism for DnsSrvDiscovery {
    fn name(&self) -> &'static str {
        "dns-srv"
    }

    async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        // Look up _capability._tcp.local (DNS-SD/SRV record)
        let service_name = format!("_{capability}._tcp.local");

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
            if let Ok(mut addrs) = format!("{service_name}:0").to_socket_addrs()
                && let Some(addr) = addrs.next()
            {
                return Ok(PrimalInfo {
                    name: capability.to_string(),
                    host: addr.ip().to_string(),
                    port: 8080,
                    capabilities: vec![capability.to_string()],
                    discovered_at: SystemTime::now(),
                    discovery_method: "dns-srv".to_string(),
                });
            }
        }

        // DNS SRV not available or failed - return error to try next mechanism
        Err(PrimalError::DiscoveryFailed {
            reason: format!(
                "DNS SRV lookup for {service_name} not available (requires dns-srv feature or network configuration)"
            ),
        })
    }
}

#[cfg(test)]
#[path = "primal_self_knowledge_tests.rs"]
mod tests;
