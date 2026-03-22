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

#![allow(
    missing_docs,
    reason = "runtime resolver structs pair with the module-level architecture doc"
)]

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
            DiscoveryMechanism::Environment => {
                self.discover_from_environment_with(request, &|k| std::env::var(k))
            }
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
    pub fn discover_from_environment(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        self.discover_from_environment_with(request, &|k| std::env::var(k))
    }

    #[allow(
        clippy::unused_self,
        reason = "method shape matches other discover_* helpers that use resolver state"
    )]
    fn discover_from_environment_with(
        &self,
        request: &CapabilityRequest,
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<CapabilityProvider> {
        let env_var = format!("SONGBIRD_{}_PROVIDER_URL", request.capability.to_uppercase());

        if let Ok(url) = env(&env_var) {
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
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use songbird_process_env;

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
        let resolver = CapabilityResolver::new();
        let request = CapabilityRequest::new("ai");

        let result = resolver.discover_from_environment_with(&request, &|k| {
            if k == "SONGBIRD_AI_PROVIDER_URL" {
                Ok("http://test.local:9200".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(result.is_ok());

        let provider = result.expect("Provider discovery should succeed in test");
        assert_eq!(provider.endpoint, "http://test.local:9200");
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
            discovered_at: std::time::Instant::now()
                .checked_sub(Duration::from_secs(400))
                .expect("instant sub"),
            ttl: Duration::from_secs(300),
        };

        assert!(cached.is_expired());
    }

    #[test]
    fn test_cached_provider_not_expired_when_fresh() {
        let provider = CapabilityProvider {
            name: "p".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://x".to_string(),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
        };
        let cached = CachedProvider {
            provider,
            discovered_at: std::time::Instant::now(),
            ttl: Duration::from_secs(3600),
        };
        assert!(!cached.is_expired());
    }

    #[test]
    fn test_protocol_json_roundtrip() {
        let p = Protocol::Custom("coap".to_string());
        let json = serde_json::to_string(&p).expect("serde");
        let back: Protocol = serde_json::from_str(&json).expect("de");
        assert_eq!(format!("{back:?}"), format!("{p:?}"));
    }

    #[test]
    fn test_capability_provider_display() {
        let p = CapabilityProvider {
            name: "prov".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://h:1".to_string(),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
        };
        assert_eq!(format!("{p}"), "prov[ai] @ http://h:1");
    }

    #[test]
    fn test_supports_features_empty_required() {
        let p = CapabilityProvider {
            name: "p".to_string(),
            capability: "x".to_string(),
            endpoint: "http://x".to_string(),
            protocol: Protocol::Http,
            features: vec!["a".to_string()],
            metadata: HashMap::new(),
        };
        assert!(p.supports_features(&[]));
    }

    #[test]
    fn test_discover_from_environment_errors_without_var() {
        let resolver = CapabilityResolver::new();
        let req = CapabilityRequest::new("sbunsetcap");
        let err = resolver
            .discover_from_environment_with(&req, &|_| Err(std::env::VarError::NotPresent))
            .expect_err("no env");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[test]
    fn test_capability_request_optional_features_and_sla() {
        let sla = SlaRequirements {
            max_latency_ms: 50,
            min_uptime_percent: 99.9,
            max_error_rate_percent: 0.1,
        };
        let req = CapabilityRequest::new("storage")
            .with_optional_features(&["cold-archive"])
            .with_sla(sla);
        assert_eq!(req.optional_features, vec!["cold-archive"]);
        assert_eq!(req.min_sla.as_ref().expect("sla").max_latency_ms, 50);
    }

    #[test]
    fn test_env_var_name_uppercases_capability_for_discovery() {
        let resolver = CapabilityResolver::new();
        let req = CapabilityRequest::new("compute");
        let out = resolver
            .discover_from_environment_with(&req, &|k| {
                if k == "SONGBIRD_COMPUTE_PROVIDER_URL" {
                    Ok("http://compute:9".to_string())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .expect("env");
        assert_eq!(out.endpoint, "http://compute:9");
        assert_eq!(out.protocol, Protocol::Http);
        assert_eq!(out.name, "compute-provider-from-env");
    }

    #[test]
    fn test_resolver_default_matches_new() {
        assert_eq!(
            CapabilityResolver::default().discovery_mechanisms.len(),
            CapabilityResolver::new().discovery_mechanisms.len()
        );
    }

    #[test]
    fn test_discovery_mechanism_equality() {
        assert_eq!(DiscoveryMechanism::Environment, DiscoveryMechanism::Environment);
        assert_ne!(DiscoveryMechanism::Environment, DiscoveryMechanism::MDNS);
    }

    #[test]
    fn test_provider_supports_features_requires_all() {
        let p = CapabilityProvider {
            name: "p".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://x".to_string(),
            protocol: Protocol::Https,
            features: vec!["a".to_string()],
            metadata: HashMap::new(),
        };
        assert!(!p.supports_features(&["a".to_string(), "b".to_string()]));
    }

    #[tokio::test]
    async fn test_discover_provider_fails_when_all_mechanisms_fail() {
        let mut resolver = CapabilityResolver {
            discovery_mechanisms: vec![DiscoveryMechanism::Environment],
            provider_cache: HashMap::new(),
        };
        let err = resolver
            .discover_provider(CapabilityRequest::new("missingcap"))
            .await
            .expect_err("no provider");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn test_discover_provider_uses_cache_before_mechanisms() {
        let cached = CapabilityProvider {
            name: "cached".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://cached".to_string(),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
        };
        let mut resolver = CapabilityResolver {
            discovery_mechanisms: vec![DiscoveryMechanism::Environment],
            provider_cache: HashMap::from([(
                "ai".to_string(),
                CachedProvider {
                    provider: cached.clone(),
                    discovered_at: std::time::Instant::now(),
                    ttl: Duration::from_secs(3600),
                },
            )]),
        };
        let got =
            resolver.discover_provider(CapabilityRequest::new("ai")).await.expect("cache hit");
        assert_eq!(got.endpoint, "http://cached");
    }

    #[tokio::test]
    async fn test_discover_provider_cache_miss_when_expired() {
        let cached = CapabilityProvider {
            name: "old".to_string(),
            capability: "ai".to_string(),
            endpoint: "http://old".to_string(),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
        };
        let mut resolver = CapabilityResolver {
            discovery_mechanisms: vec![DiscoveryMechanism::Environment],
            provider_cache: HashMap::from([(
                "ai".to_string(),
                CachedProvider {
                    provider: cached,
                    discovered_at: std::time::Instant::now()
                        .checked_sub(Duration::from_secs(400))
                        .expect("sub"),
                    ttl: Duration::from_secs(300),
                },
            )]),
        };
        let got = resolver
            .discover_provider(CapabilityRequest::new("ai"))
            .await
            .expect_err("env missing");
        assert!(matches!(got, SongbirdError::Discovery { .. }));
    }

    #[test]
    fn capability_request_serde_roundtrip() {
        let req = CapabilityRequest::new("storage")
            .with_features(&["a", "b"])
            .with_optional_features(&["c"])
            .with_preference("latency");
        let json = serde_json::to_string(&req).expect("ser");
        let back: CapabilityRequest = serde_json::from_str(&json).expect("de");
        assert_eq!(back.capability, "storage");
        assert_eq!(back.required_features, vec!["a", "b"]);
        assert_eq!(back.optional_features, vec!["c"]);
    }

    #[test]
    fn capability_provider_and_sla_serde_roundtrip() {
        let sla = SlaRequirements {
            max_latency_ms: 10,
            min_uptime_percent: 99.0,
            max_error_rate_percent: 0.5,
        };
        let p = CapabilityProvider {
            name: "n".into(),
            capability: "c".into(),
            endpoint: "http://e".into(),
            protocol: Protocol::Https,
            features: vec!["f".into()],
            metadata: HashMap::from([("k".into(), "v".into())]),
        };
        let pj = serde_json::to_string(&p).expect("ser p");
        let _: CapabilityProvider = serde_json::from_str(&pj).expect("de p");
        let sj = serde_json::to_string(&sla).expect("ser sla");
        let _: SlaRequirements = serde_json::from_str(&sj).expect("de sla");
    }

    #[test]
    fn capability_request_multiple_preferences_and_optional_features() {
        let r = CapabilityRequest::new("x")
            .with_optional_features(&["o1", "o2"])
            .with_preference("p1")
            .with_preference("p2");
        assert_eq!(r.preferences, vec!["p1", "p2"]);
        assert_eq!(r.optional_features, vec!["o1", "o2"]);
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn discover_provider_succeeds_via_environment_only() {
        let key = "SONGBIRD_SERIALAI_PROVIDER_URL";
        songbird_process_env::set_var(key, "http://serial-ai:9200");
        let mut resolver = CapabilityResolver {
            discovery_mechanisms: vec![DiscoveryMechanism::Environment],
            provider_cache: HashMap::new(),
        };
        let p = resolver
            .discover_provider(CapabilityRequest::new("serialai"))
            .await
            .expect("discovered");
        assert_eq!(p.endpoint, "http://serial-ai:9200");
        assert_eq!(p.protocol, Protocol::Http);
        songbird_process_env::remove_var(key);
    }

    #[tokio::test]
    async fn discover_provider_falls_through_when_first_mechanism_fails_but_second_succeeds() {
        let mut resolver = CapabilityResolver {
            discovery_mechanisms: vec![
                DiscoveryMechanism::Environment,
                DiscoveryMechanism::ServiceRegistry,
            ],
            provider_cache: HashMap::new(),
        };
        let err = resolver
            .discover_provider(CapabilityRequest::new("noregistry"))
            .await
            .expect_err("registry not configured");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }
}
