// SPDX-License-Identifier: AGPL-3.0-or-later
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
    discovery_mechanisms: Vec<DiscoveryMechanism>,
}

impl PrimalSelfKnowledge {
    /// Discover self through environment and introspection
    ///
    /// No assumptions, pure self-discovery.
    pub fn discover_self() -> Result<Self> {
        let my_name = Self::introspect_name_with(|k| songbird_process_env::var(k));
        let my_capabilities = Self::introspect_capabilities_with(|k| songbird_process_env::var(k));

        tracing::info!(
            "Primal self-discovered: name='{}', capabilities={:?}",
            my_name,
            my_capabilities
        );

        Ok(Self {
            my_name,
            my_capabilities,
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            discovery_mechanisms: vec![
                DiscoveryMechanism::ProcessEnvironment,
                DiscoveryMechanism::DnsSrv(DnsSrvDiscovery::new()),
            ],
        })
    }

    /// Same as [`discover_self`](Self::discover_self) with a merged env map for tests and tooling.
    ///
    /// Keys are environment variable names; values override `songbird_process_env::var` when present.
    pub fn discover_self_with(env: HashMap<String, String>) -> Result<Self> {
        let my_name = Self::introspect_name_with(|k| lookup_env_map_or_process(&env, k));
        let my_capabilities =
            Self::introspect_capabilities_with(|k| lookup_env_map_or_process(&env, k));

        tracing::info!(
            "Primal self-discovered: name='{}', capabilities={:?}",
            my_name,
            my_capabilities
        );

        let env = Arc::new(env);
        Ok(Self {
            my_name,
            my_capabilities,
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            discovery_mechanisms: vec![
                DiscoveryMechanism::MappedEnvironment(Arc::clone(&env)),
                DiscoveryMechanism::DnsSrv(DnsSrvDiscovery::new()),
            ],
        })
    }

    /// Like [`discover_self_with`](Self::discover_self_with), but environment-based discovery uses
    /// only the provided map (no `songbird_process_env` fallback). Intended for tests that need
    /// deterministic "missing env" behavior.
    pub fn discover_self_with_strict(env: HashMap<String, String>) -> Result<Self> {
        let my_name = Self::introspect_name_with(|k| lookup_env_map_or_process(&env, k));
        let my_capabilities =
            Self::introspect_capabilities_with(|k| lookup_env_map_or_process(&env, k));

        tracing::info!(
            "Primal self-discovered: name='{}', capabilities={:?}",
            my_name,
            my_capabilities
        );

        let env = Arc::new(env);
        Ok(Self {
            my_name,
            my_capabilities,
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            discovery_mechanisms: vec![
                DiscoveryMechanism::MappedEnvironmentStrict(Arc::clone(&env)),
                DiscoveryMechanism::DnsSrv(DnsSrvDiscovery::new()),
            ],
        })
    }

    /// Introspect own name from environment
    #[must_use]
    pub fn introspect_name() -> String {
        Self::introspect_name_with(|k| songbird_process_env::var(k))
    }

    fn introspect_name_with(
        mut env: impl FnMut(&str) -> std::result::Result<String, VarError>,
    ) -> String {
        // Try explicit name first
        if let Ok(name) = env("PRIMAL_NAME") {
            return name;
        }

        // Try service name
        if let Ok(name) = env("SERVICE_NAME") {
            return name;
        }

        // Fall back to hostname
        gethostname::gethostname().into_string().unwrap_or_else(|_| String::from("unknown"))
    }

    /// Introspect own capabilities through feature detection
    ///
    /// No hardcoding - discovers what this binary can do.
    #[must_use]
    pub fn introspect_capabilities() -> Vec<String> {
        Self::introspect_capabilities_with(|k| songbird_process_env::var(k))
    }

    fn introspect_capabilities_with(
        mut env: impl FnMut(&str) -> std::result::Result<String, VarError>,
    ) -> Vec<String> {
        let mut caps = vec![
            #[cfg(feature = "security")]
            String::from("security"),
            #[cfg(feature = "storage")]
            String::from("storage"),
            #[cfg(feature = "compute")]
            String::from("compute"),
            #[cfg(feature = "ai")]
            String::from("ai"),
            #[cfg(feature = "discovery")]
            String::from("discovery"),
            #[cfg(feature = "orchestration")]
            String::from("orchestration"),
        ];

        // Check environment hints
        if env("ENABLE_SECURITY").is_ok() && !caps.contains(&String::from("security")) {
            caps.push(String::from("security"));
        }

        if env("ENABLE_AI").is_ok() && !caps.contains(&String::from("ai")) {
            caps.push(String::from("ai"));
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
                caps.push(String::from("security"));
            } else if name_lower.contains("ai")
                || name_lower.contains("inference")
                || name_lower.contains("ml")
            {
                caps.push(String::from("ai"));
            } else if name_lower.contains("discovery")
                || name_lower.contains("gateway")
                || name_lower.contains("registry")
            {
                caps.push(String::from("discovery"));
            } else if name_lower.contains("storage")
                || name_lower.contains("data")
                || name_lower.contains("persist")
            {
                caps.push(String::from("storage"));
            } else if name_lower.contains("compute")
                || name_lower.contains("worker")
                || name_lower.contains("exec")
            {
                caps.push(String::from("compute"));
            } else if name_lower.contains("orchestrat") || name_lower.contains("coordinat") {
                caps.push(String::from("orchestration"));
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

fn lookup_env_map_or_process(
    map: &HashMap<String, String>,
    key: &str,
) -> std::result::Result<String, VarError> {
    if let Some(v) = map.get(key) {
        return Ok(v.clone());
    }
    songbird_process_env::var(key)
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

/// Concrete discovery mechanism (enum dispatch; no trait objects).
#[derive(Clone)]
pub enum DiscoveryMechanism {
    /// Environment variables via [`songbird_process_env`] (production [`PrimalSelfKnowledge::discover_self`]).
    ProcessEnvironment,
    /// Merged map + process env fallback (see [`PrimalSelfKnowledge::discover_self_with`]).
    MappedEnvironment(Arc<HashMap<String, String>>),
    /// Map only — no process fallback (see [`PrimalSelfKnowledge::discover_self_with_strict`]).
    MappedEnvironmentStrict(Arc<HashMap<String, String>>),
    /// DNS SRV style lookup (optional / feature-gated behavior inside).
    DnsSrv(DnsSrvDiscovery),
}

impl DiscoveryMechanism {
    /// Name of this discovery mechanism
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::ProcessEnvironment
            | Self::MappedEnvironment(_)
            | Self::MappedEnvironmentStrict(_) => "environment",
            Self::DnsSrv(_) => "dns-srv",
        }
    }

    /// Discover primal by capability
    pub async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        match self {
            Self::ProcessEnvironment => {
                EnvironmentDiscovery::discover_with(capability, |k| songbird_process_env::var(k))
                    .await
            }
            Self::MappedEnvironment(map) => {
                let map = Arc::clone(map);
                EnvironmentDiscovery::discover_with(capability, move |k| {
                    if let Some(v) = map.get(k) {
                        Ok(v.clone())
                    } else {
                        songbird_process_env::var(k)
                    }
                })
                .await
            }
            Self::MappedEnvironmentStrict(map) => {
                let map = Arc::clone(map);
                EnvironmentDiscovery::discover_with(capability, move |k| {
                    map.get(k).cloned().ok_or(VarError::NotPresent)
                })
                .await
            }
            Self::DnsSrv(d) => d.discover(capability).await,
        }
    }
}

/// Environment variable based discovery
pub struct EnvironmentDiscovery;

/// Uses injected env lookup for [`PrimalSelfKnowledge::discover_self_with`].
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

    /// Mechanism label (same as [`DiscoveryMechanism`] for the process-backed path).
    #[must_use]
    pub fn name(&self) -> &'static str {
        "environment"
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
            discovery_method: String::from("environment"),
        })
    }

    /// Discover using process environment only.
    pub async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
        Self::discover_with(capability, |k| songbird_process_env::var(k)).await
    }
}

/// DNS SRV record discovery
#[derive(Clone, Copy)]
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

    #[must_use]
    pub fn name(&self) -> &'static str {
        "dns-srv"
    }

    /// Look up `_capability._tcp.local` (simplified).
    pub async fn discover(&self, capability: &str) -> Result<PrimalInfo> {
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
                    discovery_method: String::from("dns-srv"),
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
