// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎯 Capability-Based Runtime Discovery
//!
//! **PHILOSOPHY**: Zero hardcoding through pure capability-based discovery
//!
//! ## Core Principles
//! 1. **Self-Knowledge Only**: Each service knows only itself
//! 2. **Runtime Discovery**: All external services discovered at runtime
//! 3. **Capability-Based**: Request by capability, not by name
//! 4. **Zero Hardcoding**: No primal names, no vendor names, no endpoints in code
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   Application   │
//! │  "I need AI"    │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────────────┐
//! │ Capability Resolver     │
//! │  discovers providers... │
//! └────────┬────────────────┘
//!          │
//!          ▼
//! ┌─────────────────────────┐
//! │  Discovery Mechanisms   │
//! │  • mDNS                 │
//! │  • DNS-SD               │
//! │  • Service Registry     │
//! │  • Environment Hints    │
//! └────────┬────────────────┘
//!          │
//!          ▼
//! ┌─────────────────────────┐
//! │  Provider Found!        │
//! │  (could be any impl)    │
//! └─────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### Instead of Hardcoding:
//! ```rust,ignore
//! // ❌ OLD WAY: Hardcoded primal name and endpoint
//! let client = SquirrelClient::connect("http://localhost:9200")?;
//! let response = client.process_ai(request).await?;
//! ```
//!
//! ### Use Capability-Based Discovery:
//! ```rust,ignore
//! use songbird_config::capability_based_runtime_discovery::{CapabilityResolver, CapabilityRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // ✅ NEW WAY: Discover by capability at runtime
//! let mut resolver = CapabilityResolver::new();
//! let provider = resolver.discover_provider(
//!     CapabilityRequest::new("ai")
//!         .with_features(&["text-generation", "embeddings"])
//!         .with_preference("performance")
//! ).await?;
//!
//! // Use whatever provider was discovered (Squirrel, OpenAI, Anthropic, etc.)
//! // Connect to the provider endpoint and make requests
//! println!("Found provider: {} at {}", provider.name, provider.endpoint);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use tracing::{debug, info};

// Discovery backend modules
pub mod dnssd;
pub mod mdns;
pub mod service_registry;

// Re-export backend types
pub use dnssd::DnsSDDiscovery;
pub use mdns::MdnsDiscovery;
pub use service_registry::ServiceRegistryDiscovery;

// ============================================================================
// CAPABILITY RESOLVER - Core Discovery Engine
// ============================================================================

/// Resolves capabilities to providers at runtime
///
/// This is the central discovery mechanism that replaces all hardcoded
/// service references with dynamic, capability-based discovery.
#[derive(Debug, Clone)]
pub struct CapabilityResolver {
    /// Discovery mechanisms to try (in order of preference)
    discovery_mechanisms: Vec<DiscoveryMechanism>,
    /// Cache of discovered providers
    provider_cache: HashMap<String, CachedProvider>,
    /// Configuration hints from environment (for future expansion)
    #[allow(dead_code)]
    environment_hints: EnvironmentHints,
}

impl CapabilityResolver {
    /// Create a new capability resolver with default discovery mechanisms
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery_mechanisms: vec![
                DiscoveryMechanism::Environment,
                DiscoveryMechanism::ServiceRegistry,
                DiscoveryMechanism::MDNS,
                DiscoveryMechanism::DNSSD,
            ],
            provider_cache: HashMap::new(),
            environment_hints: EnvironmentHints::from_env(),
        }
    }

    /// Discover a provider for the requested capability
    ///
    /// # Philosophy
    /// - Tries multiple discovery mechanisms
    /// - Returns first working provider
    /// - Caches successful discoveries
    /// - No assumptions about provider identity
    ///
    /// # Errors
    /// Returns error if no provider is found after trying all discovery mechanisms
    pub async fn discover_provider(
        &mut self,
        request: CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        let capability = &request.capability;

        // Check cache first
        if let Some(cached) = self.provider_cache.get(capability)
            && !cached.is_expired()
        {
            debug!("Using cached provider for capability: {}", capability);
            return Ok(cached.provider.clone());
        }

        // Try each discovery mechanism in order
        for mechanism in &self.discovery_mechanisms {
            match self.try_discover_with(mechanism, &request).await {
                Ok(provider) => {
                    info!(
                        "Discovered provider for '{capability}' via {mechanism:?}: {}",
                        provider.name
                    );

                    // Cache successful discovery
                    self.provider_cache.insert(
                        capability.clone(),
                        CachedProvider {
                            provider: provider.clone(),
                            discovered_at: std::time::Instant::now(),
                            ttl: Duration::from_secs(300), // 5 minute cache
                        },
                    );

                    return Ok(provider);
                }
                Err(e) => {
                    debug!("Discovery mechanism {mechanism:?} failed for '{capability}': {e}");
                    // Try next mechanism
                }
            }
        }

        Err(SongbirdError::discovery(format!(
            "No provider found for capability '{capability}' after trying all discovery mechanisms"
        )))
    }

    /// Try to discover using a specific mechanism
    async fn try_discover_with(
        &self,
        mechanism: &DiscoveryMechanism,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        match mechanism {
            DiscoveryMechanism::Environment => self.discover_from_environment(request),
            DiscoveryMechanism::ServiceRegistry => self.discover_from_registry(request).await,
            DiscoveryMechanism::MDNS => self.discover_from_mdns(request).await,
            DiscoveryMechanism::DNSSD => self.discover_from_dnssd(request).await,
        }
    }

    /// Discover from environment variables (highest priority)
    ///
    /// Environment variables provide explicit configuration:
    /// - `SONGBIRD_AI_PROVIDER_URL` -> AI capability provider
    /// - `SONGBIRD_SECURITY_PROVIDER_URL` -> Security capability provider
    /// - `SONGBIRD_STORAGE_PROVIDER_URL` -> Storage capability provider
    /// - etc.
    ///
    /// Pure function kept as method for trait implementation consistency and future extensibility.
    #[allow(clippy::unused_self)]
    fn discover_from_environment(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        let env_var = format!("SONGBIRD_{}_PROVIDER_URL", request.capability.to_uppercase());

        if let Ok(url) = std::env::var(&env_var) {
            return Ok(CapabilityProvider {
                name: format!("{}-provider-from-env", request.capability),
                capability: request.capability.clone(),
                endpoint: url,
                protocol: Protocol::Http,
                features: request.required_features.clone(),
                metadata: HashMap::new(),
            });
        }

        Err(SongbirdError::discovery(format!("No environment variable {env_var} found")))
    }

    /// Discover from service registry
    ///
    /// # Errors
    /// Returns error if registry is unavailable or capability not found
    ///
    /// Note: Currently uses environment for registry endpoint.
    /// Future: Will query actual service registry infrastructure.
    #[allow(clippy::unused_self)]
    async fn discover_from_registry(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Try to create service registry discovery from environment
        match ServiceRegistryDiscovery::from_env() {
            Ok(discovery) => discovery.discover(request).await,
            Err(e) => {
                debug!("Service registry not configured: {e}");
                Err(SongbirdError::discovery("Service registry discovery not configured"))
            }
        }
    }

    /// Discover from mDNS (local network discovery)
    ///
    /// # Errors
    /// Returns error if mDNS is unavailable or no services found
    async fn discover_from_mdns(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Create mDNS discovery with default service type
        let discovery = MdnsDiscovery::new(None);
        discovery.discover(request).await
    }

    /// Discover from DNS-SD (DNS Service Discovery)
    ///
    /// # Errors
    /// Returns error if DNS-SD queries fail or no services found
    async fn discover_from_dnssd(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Create DNS-SD discovery from environment or use default
        let discovery =
            DnsSDDiscovery::from_env().unwrap_or_else(|_| DnsSDDiscovery::new("songbird.local"));
        discovery.discover(request).await
    }
}

impl Default for CapabilityResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CAPABILITY REQUEST
// ============================================================================

/// A request for a specific capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    /// The capability being requested (e.g., "ai", "storage", "compute")
    pub capability: String,
    /// Required features (e.g., `["text-generation", "embeddings"]`)
    pub required_features: Vec<String>,
    /// Optional features (nice to have)
    pub optional_features: Vec<String>,
    /// Performance preferences ("latency", "throughput", "cost")
    pub preferences: Vec<String>,
    /// Minimum required SLA
    pub min_sla: Option<SlaRequirements>,
}

impl CapabilityRequest {
    /// Create a new capability request
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            required_features: Vec::new(),
            optional_features: Vec::new(),
            preferences: Vec::new(),
            min_sla: None,
        }
    }

    /// Add required features
    #[must_use]
    pub fn with_features(mut self, features: &[impl AsRef<str>]) -> Self {
        self.required_features = features.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    /// Add optional features
    #[must_use]
    pub fn with_optional_features(mut self, features: &[impl AsRef<str>]) -> Self {
        self.optional_features = features.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    /// Add performance preference
    #[must_use]
    pub fn with_preference(mut self, preference: impl Into<String>) -> Self {
        self.preferences.push(preference.into());
        self
    }

    /// Add SLA requirements
    #[must_use]
    pub const fn with_sla(mut self, sla: SlaRequirements) -> Self {
        self.min_sla = Some(sla);
        self
    }
}

// ============================================================================
// CAPABILITY PROVIDER
// ============================================================================

/// A discovered provider for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    /// Provider name (for logging/debugging only, not used in application logic)
    pub name: String,
    /// Capability provided
    pub capability: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Protocol used
    pub protocol: Protocol,
    /// Supported features
    pub features: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CapabilityProvider {
    /// Check if provider supports required features
    #[must_use]
    pub fn supports_features(&self, required: &[String]) -> bool {
        required.iter().all(|req| self.features.contains(req))
    }
}

impl fmt::Display for CapabilityProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}] @ {}", self.name, self.capability, self.endpoint)
    }
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

/// Discovery mechanism
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMechanism {
    /// Environment variables (highest priority)
    Environment,
    /// Service registry
    ServiceRegistry,
    /// Multicast DNS (mDNS) for local discovery
    MDNS,
    /// DNS Service Discovery (DNS-SD)
    DNSSD,
}

/// Communication protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Http,
    Https,
    Tarpc,
    WebSocket,
    Custom(String),
}

/// SLA requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaRequirements {
    /// Maximum acceptable latency (milliseconds)
    pub max_latency_ms: u64,
    /// Minimum uptime percentage
    pub min_uptime_percent: f64,
    /// Maximum error rate percentage
    pub max_error_rate_percent: f64,
}

/// Cached provider entry
#[derive(Debug, Clone)]
struct CachedProvider {
    provider: CapabilityProvider,
    discovered_at: std::time::Instant,
    ttl: Duration,
}

impl CachedProvider {
    fn is_expired(&self) -> bool {
        self.discovered_at.elapsed() > self.ttl
    }
}

/// Environment hints for discovery
#[derive(Debug, Clone, Default)]
struct EnvironmentHints {
    hints: HashMap<String, String>,
}

impl EnvironmentHints {
    fn from_env() -> Self {
        let mut hints = HashMap::new();

        // Collect all SONGBIRD_*_HINT environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("SONGBIRD_") && key.ends_with("_HINT") {
                hints.insert(key, value);
            }
        }

        Self {
            hints,
        }
    }

    #[allow(dead_code)]
    fn get_hint(&self, key: &str) -> Option<&String> {
        self.hints.get(key)
    }
}

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/// Helper to migrate from hardcoded endpoints
///
/// This macro helps replace hardcoded service references with capability-based discovery
///
/// # Example
/// ```rust,ignore
/// // OLD:
/// let client = create_client("http://localhost:9200")?;
///
/// // NEW:
/// let provider = discover_capability!("ai").await?;
/// let client = create_client(&provider.endpoint)?;
/// ```
#[macro_export]
macro_rules! discover_capability {
    ($capability:expr) => {
        $crate::capability_based_runtime_discovery::CapabilityResolver::new()
            .discover_provider(
                $crate::capability_based_runtime_discovery::CapabilityRequest::new($capability),
            )
    };
    ($capability:expr, $($feature:expr),+) => {
        $crate::capability_based_runtime_discovery::CapabilityResolver::new()
            .discover_provider(
                $crate::capability_based_runtime_discovery::CapabilityRequest::new($capability)
                    .with_features(&[$($feature),+]),
            )
    };
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_request_builder() {
        let request = CapabilityRequest::new("ai")
            .with_features(&["text-generation", "embeddings"])
            .with_preference("performance");

        assert_eq!(request.capability, "ai");
        assert_eq!(request.required_features.len(), 2);
        assert_eq!(request.preferences.len(), 1);
    }

    #[test]
    fn test_provider_feature_support() {
        let provider = CapabilityProvider {
            name: "test-provider".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://localhost:9200".to_string(),
            protocol: Protocol::Http,
            features: vec!["text-generation".to_string(), "embeddings".to_string()],
            metadata: HashMap::new(),
        };

        assert!(provider.supports_features(&["text-generation".to_string()]));
        assert!(provider.supports_features(&["embeddings".to_string()]));
        assert!(!provider.supports_features(&["image-generation".to_string()]));
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        songbird_process_env::set_var("SONGBIRD_AI_PROVIDER_URL", "http://test.local:9200");

        let mut resolver = CapabilityResolver::new();
        let request = CapabilityRequest::new("ai");

        let result = resolver.discover_provider(request).await;
        assert!(result.is_ok());

        let provider = result.expect("Provider discovery should succeed in test");
        assert_eq!(provider.endpoint, "http://test.local:9200");

        songbird_process_env::remove_var("SONGBIRD_AI_PROVIDER_URL");
    }

    #[test]
    fn test_cached_provider_expiry() {
        let provider = CapabilityProvider {
            name: "test".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://test".to_string(),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
        };

        let cached = CachedProvider {
            provider,
            discovered_at: std::time::Instant::now().checked_sub(Duration::from_secs(400)).unwrap(),
            ttl: Duration::from_secs(300),
        };

        assert!(cached.is_expired());
    }
}
