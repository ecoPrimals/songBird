// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Service Registry Discovery Backend
//!
//! Complete production implementation for discovering capabilities via service registry.
//! Integrates with songbird-registry for dynamic service discovery.

use super::{CapabilityProvider, CapabilityRequest, Protocol};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

/// Service registry discovery backend
///
/// Discovers capabilities by querying the central service registry.
/// Supports filtering by capability, features, and SLA requirements.
#[derive(Debug, Clone)]
pub struct ServiceRegistryDiscovery {
    /// Registry endpoint
    registry_endpoint: String,
    /// Query timeout (reserved for async timeout implementation)
    #[allow(dead_code, reason = "reserved for registry HTTP client timeout wiring")]
    timeout: Duration,
    /// Cache TTL for future caching implementation
    #[allow(dead_code, reason = "wired when distributed discovery cache is added")]
    cache_ttl: Duration,
}

impl ServiceRegistryDiscovery {
    /// Create a new service registry discovery backend
    ///
    /// # Arguments
    /// * `registry_endpoint` - URL of the service registry
    ///
    /// # Examples
    /// ```no_run
    /// use songbird_config::capability_based_runtime_discovery::service_registry::ServiceRegistryDiscovery;
    ///
    /// let _discovery = ServiceRegistryDiscovery::new("http://registry.local:8500");
    /// ```
    #[must_use]
    pub fn new(registry_endpoint: impl Into<String>) -> Self {
        Self {
            registry_endpoint: registry_endpoint.into(),
            timeout: Duration::from_secs(5),
            cache_ttl: Duration::from_secs(300),
        }
    }

    /// Create from environment variables
    ///
    /// Reads `SONGBIRD_REGISTRY_ENDPOINT` for registry location
    ///
    /// # Errors
    /// Returns error if environment variable is not set
    pub fn from_env() -> SongbirdResult<Self> {
        let endpoint = songbird_process_env::var("SONGBIRD_REGISTRY_ENDPOINT")
            .map_err(|_| SongbirdError::configuration("SONGBIRD_REGISTRY_ENDPOINT not set"))?;

        Ok(Self::new(endpoint))
    }

    /// Discover a capability provider from the registry
    ///
    /// # Errors
    /// Returns error if registry is unreachable or capability not found
    pub async fn discover(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        debug!(
            "Querying service registry at {} for capability: {}",
            self.registry_endpoint, request.capability
        );

        // Query the registry for services matching the capability
        let services = self.query_registry(request).await?;

        if services.is_empty() {
            return Err(SongbirdError::discovery(format!(
                "No services found in registry for capability: {}",
                request.capability
            )));
        }

        // Select the best matching service
        let best_match = self.select_best_match(&services, request)?;

        info!(
            "Discovered provider '{}' for capability '{}' from registry",
            best_match.name, request.capability
        );

        Ok(best_match)
    }

    /// Query the service registry for matching services
    async fn query_registry(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<Vec<RegistryService>> {
        // Build query URL
        let query_url = format!(
            "{}/v1/catalog/service?capability={}",
            self.registry_endpoint, request.capability
        );

        // Use IpcHttpClient for HTTP queries (100% Pure Rust!)
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        let response = client
            .get(&query_url)
            .await
            .map_err(|e| SongbirdError::network(format!("Registry query failed: {e}")))?;

        if !response.is_success() {
            return Err(SongbirdError::discovery(format!(
                "Registry returned error: {}",
                response.status()
            )));
        }

        let services: Vec<RegistryService> = response.json().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to parse registry response: {e}"))
        })?;

        Ok(services)
    }

    /// Select the best matching service based on requirements
    fn select_best_match(
        &self,
        services: &[RegistryService],
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Filter by required features
        let mut candidates: Vec<_> = services
            .iter()
            .filter(|s| Self::supports_required_features(s, &request.required_features))
            .collect();

        if candidates.is_empty() {
            return Err(SongbirdError::discovery(format!(
                "No services support required features: {:?}",
                request.required_features
            )));
        }

        // Filter by SLA if specified
        if let Some(sla) = &request.min_sla {
            candidates.retain(|s| self.meets_sla_requirements(s, sla));

            if candidates.is_empty() {
                return Err(SongbirdError::discovery(String::from(
                    "No services meet SLA requirements",
                )));
            }
        }

        // Apply preference-based scoring
        candidates.sort_by(|a, b| {
            let score_a = Self::calculate_preference_score(a, &request.preferences);
            let score_b = Self::calculate_preference_score(b, &request.preferences);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select the highest scoring service
        let selected = candidates.first().ok_or_else(|| {
            SongbirdError::discovery(String::from("No suitable service found after scoring"))
        })?;

        Ok(CapabilityProvider {
            name: selected.name.clone(),
            capability: request.capability.clone(),
            endpoint: selected.endpoint.clone(),
            protocol: selected.protocol.clone(),
            features: selected.features.clone(),
            metadata: selected.metadata.clone(),
        })
    }

    /// Check if service supports required features
    fn supports_required_features(service: &RegistryService, required: &[String]) -> bool {
        required.iter().all(|req| service.features.contains(req))
    }

    /// Check if service meets SLA requirements
    ///
    /// Pure function kept as method for potential future use of instance state.
    #[expect(clippy::unused_self, reason = "unused bindings/imports in this compilation unit")]
    fn meets_sla_requirements(
        &self,
        service: &RegistryService,
        sla: &super::SlaRequirements,
    ) -> bool {
        service.health_metrics.as_ref().is_some_and(|health| {
            health.average_latency_ms <= sla.max_latency_ms
                && health.uptime_percent >= sla.min_uptime_percent
                && health.error_rate_percent <= sla.max_error_rate_percent
        })
    }

    /// Calculate preference score for service selection
    ///
    /// Uses f64 for scoring as precision loss is acceptable for ranking.
    /// Millisecond-level latency doesn't require u64 precision.
    #[expect(
        clippy::cast_precision_loss,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    fn calculate_preference_score(service: &RegistryService, preferences: &[String]) -> f64 {
        let mut score = 0.0;

        for preference in preferences {
            match preference.as_str() {
                "performance" | "latency" => {
                    if let Some(health) = &service.health_metrics {
                        // Lower latency is better
                        score += 1000.0 / (health.average_latency_ms as f64 + 1.0);
                    }
                }
                "throughput" => {
                    if let Some(health) = &service.health_metrics {
                        score += health.throughput_rps;
                    }
                }
                "reliability" | "uptime" => {
                    if let Some(health) = &service.health_metrics {
                        score += health.uptime_percent * 10.0;
                    }
                }
                "cost" => {
                    // Lower load is preferred (more available capacity)
                    if let Some(health) = &service.health_metrics {
                        score += 100.0 - health.load_percent;
                    }
                }
                _ => {}
            }
        }

        // Default scoring if no preferences
        if score == 0.0 {
            score = service.health_metrics.as_ref().map_or(1.0, |health| {
                health.uptime_percent * health.throughput_rps
                    / (health.average_latency_ms as f64 + 1.0)
            });
        }

        score
    }
}

/// Service information from registry
#[derive(Debug, Clone, serde::Deserialize)]
struct RegistryService {
    /// Service name
    name: String,
    /// Service endpoint
    endpoint: String,
    /// Protocol
    #[serde(default = "default_protocol")]
    protocol: Protocol,
    /// Supported features
    #[serde(default)]
    features: Vec<String>,
    /// Service metadata
    #[serde(default)]
    metadata: HashMap<String, String>,
    /// Health metrics
    health_metrics: Option<HealthMetrics>,
}

/// Health metrics from registry
#[derive(Debug, Clone, serde::Deserialize)]
struct HealthMetrics {
    /// Average latency in milliseconds
    average_latency_ms: u64,
    /// Uptime percentage
    uptime_percent: f64,
    /// Error rate percentage
    error_rate_percent: f64,
    /// Throughput in requests per second
    throughput_rps: f64,
    /// Current load percentage
    load_percent: f64,
}

const fn default_protocol() -> Protocol {
    Protocol::Http
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::capability_based_runtime_discovery::{CapabilityRequest, SlaRequirements};

    #[test]
    fn test_service_registry_discovery_creation() {
        let discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        assert_eq!(discovery.registry_endpoint, "http://localhost:8500");
        assert_eq!(discovery.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_feature_support_check() {
        let _discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        let _ = _discovery; // Suppress unused warning

        let service = RegistryService {
            name: String::from("test-service"),
            endpoint: String::from("http://test:8080"),
            protocol: Protocol::Http,
            features: vec![String::from("feature1"), String::from("feature2")],
            metadata: HashMap::new(),
            health_metrics: None,
        };

        assert!(ServiceRegistryDiscovery::supports_required_features(
            &service,
            &[String::from("feature1")]
        ));
        assert!(ServiceRegistryDiscovery::supports_required_features(
            &service,
            &[String::from("feature1"), String::from("feature2")]
        ));
        assert!(!ServiceRegistryDiscovery::supports_required_features(
            &service,
            &[String::from("feature3")]
        ));
    }

    #[test]
    fn test_preference_scoring() {
        let _discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        let _ = _discovery; // Suppress unused warning

        let service = RegistryService {
            name: String::from("test-service"),
            endpoint: String::from("http://test:8080"),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
            health_metrics: Some(HealthMetrics {
                average_latency_ms: 50,
                uptime_percent: 99.9,
                error_rate_percent: 0.1,
                throughput_rps: 1000.0,
                load_percent: 60.0,
            }),
        };

        let performance_score = ServiceRegistryDiscovery::calculate_preference_score(
            &service,
            &[String::from("performance")],
        );
        let throughput_score = ServiceRegistryDiscovery::calculate_preference_score(
            &service,
            &[String::from("throughput")],
        );

        assert!(performance_score > 0.0);
        assert!(throughput_score > 0.0);
    }

    static REGISTRY_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_from_env_errors_when_registry_endpoint_missing() {
        let _lock = REGISTRY_ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = songbird_process_env::var("SONGBIRD_REGISTRY_ENDPOINT").ok();
        songbird_process_env::remove_var("SONGBIRD_REGISTRY_ENDPOINT");
        let err = ServiceRegistryDiscovery::from_env().expect_err("missing env");
        assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
        match prev {
            Some(v) => songbird_process_env::set_var("SONGBIRD_REGISTRY_ENDPOINT", &v),
            None => songbird_process_env::remove_var("SONGBIRD_REGISTRY_ENDPOINT"),
        }
    }

    #[test]
    fn test_from_env_ok_when_registry_endpoint_set() {
        let _lock = REGISTRY_ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let _g = EnvRestore::set("SONGBIRD_REGISTRY_ENDPOINT", "http://registry.local:8500");
        let d = ServiceRegistryDiscovery::from_env().expect("from_env");
        assert_eq!(d.registry_endpoint, "http://registry.local:8500");
    }

    struct EnvRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = songbird_process_env::var(key).ok();
            songbird_process_env::set_var(key, value);
            Self {
                key,
                previous,
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => songbird_process_env::set_var(self.key, v),
                None => songbird_process_env::remove_var(self.key),
            }
        }
    }

    fn sample_service(name: &str, endpoint: &str, features: Vec<String>) -> RegistryService {
        RegistryService {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            protocol: Protocol::Http,
            features,
            metadata: HashMap::new(),
            health_metrics: Some(HealthMetrics {
                average_latency_ms: 20,
                uptime_percent: 99.95,
                error_rate_percent: 0.05,
                throughput_rps: 500.0,
                load_percent: 40.0,
            }),
        }
    }

    #[test]
    fn test_select_best_match_errors_when_required_features_unmet() {
        let discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        let services = vec![sample_service("a", "http://a:1", vec![String::from("x")])];
        let req = CapabilityRequest::new("compute").with_features(&["missing-feature"]);
        let err = discovery.select_best_match(&services, &req).expect_err("no feature match");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[test]
    fn test_select_best_match_errors_when_sla_not_met() {
        let discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        let mut svc = sample_service("slow", "http://slow:1", vec![]);
        svc.health_metrics = Some(HealthMetrics {
            average_latency_ms: 500,
            uptime_percent: 99.0,
            error_rate_percent: 0.1,
            throughput_rps: 100.0,
            load_percent: 50.0,
        });
        let req = CapabilityRequest::new("compute").with_sla(SlaRequirements {
            max_latency_ms: 50,
            min_uptime_percent: 99.9,
            max_error_rate_percent: 0.05,
        });
        let err = discovery.select_best_match(&[svc], &req).expect_err("sla filter");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[test]
    fn test_select_best_match_prefers_higher_preference_score() {
        let discovery = ServiceRegistryDiscovery::new("http://localhost:8500");
        let mut fast = sample_service("fast", "http://fast:1", vec![]);
        fast.health_metrics = Some(HealthMetrics {
            average_latency_ms: 10,
            uptime_percent: 99.0,
            error_rate_percent: 0.1,
            throughput_rps: 100.0,
            load_percent: 10.0,
        });
        let mut slow = sample_service("slow", "http://slow:2", vec![]);
        slow.health_metrics = Some(HealthMetrics {
            average_latency_ms: 200,
            uptime_percent: 99.0,
            error_rate_percent: 0.1,
            throughput_rps: 100.0,
            load_percent: 90.0,
        });
        let req = CapabilityRequest::new("compute").with_preference("performance");
        let best = discovery.select_best_match(&[slow, fast], &req).expect("best match");
        assert_eq!(best.endpoint, "http://fast:1");
    }

    #[test]
    fn test_preference_scoring_reliability_and_cost_branches() {
        let svc = RegistryService {
            name: String::from("svc"),
            endpoint: String::from("http://svc:1"),
            protocol: Protocol::Http,
            features: vec![],
            metadata: HashMap::new(),
            health_metrics: Some(HealthMetrics {
                average_latency_ms: 30,
                uptime_percent: 99.5,
                error_rate_percent: 0.02,
                throughput_rps: 200.0,
                load_percent: 25.0,
            }),
        };
        let rel = ServiceRegistryDiscovery::calculate_preference_score(
            &svc,
            &[String::from("reliability")],
        );
        let cost =
            ServiceRegistryDiscovery::calculate_preference_score(&svc, &[String::from("cost")]);
        assert!(rel > 0.0);
        assert!(cost > 0.0);
    }
}
