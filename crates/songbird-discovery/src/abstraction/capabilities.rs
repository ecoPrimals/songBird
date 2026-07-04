// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Discovery Capabilities
//!
//! Defines what discovery providers can do, not what they are.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Discovery capabilities that providers can implement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryCapability {
    /// Can register new services
    ServiceRegistration,
    /// Can unregister services
    ServiceUnregistration,
    /// Can discover services by query
    ServiceDiscovery,
    /// Can watch for service changes
    ServiceWatching,
    /// Can perform health checks
    HealthChecking,
    /// Can update service metadata
    MetadataUpdating,
    /// Can list all services
    ServiceListing,
    /// Can check service existence
    ServiceExistence,
    /// Can provide service metrics
    ServiceMetrics,
    /// Can handle service dependencies
    DependencyResolution,
    /// Can provide load balancing hints
    LoadBalancingHints,
    /// Can manage service versions
    VersionManagement,
    /// Custom capability (for extensibility)
    Custom(String),
}

/// Capability matcher for finding suitable providers
#[derive(Debug, Clone)]
pub struct CapabilityMatcher {
    /// Required capabilities (all must be present)
    pub required: Vec<DiscoveryCapability>,
    /// Preferred capabilities (nice to have)
    pub preferred: Vec<DiscoveryCapability>,
    /// Excluded capabilities (must not be present)
    pub excluded: Vec<DiscoveryCapability>,
    /// Custom filters
    pub filters: HashMap<String, String>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher
    #[must_use]
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            preferred: Vec::new(),
            excluded: Vec::new(),
            filters: HashMap::new(),
        }
    }

    /// Add required capability
    #[must_use]
    pub fn require(mut self, capability: DiscoveryCapability) -> Self {
        self.required.push(capability);
        self
    }

    /// Add preferred capability
    #[must_use]
    pub fn prefer(mut self, capability: DiscoveryCapability) -> Self {
        self.preferred.push(capability);
        self
    }

    /// Add excluded capability
    #[must_use]
    pub fn exclude(mut self, capability: DiscoveryCapability) -> Self {
        self.excluded.push(capability);
        self
    }

    /// Add custom filter
    #[must_use]
    pub fn filter(mut self, key: String, value: String) -> Self {
        self.filters.insert(key, value);
        self
    }

    /// Check if a set of capabilities matches this matcher
    #[must_use]
    pub fn matches(&self, capabilities: &[DiscoveryCapability]) -> bool {
        if !self.required.iter().all(|req| capabilities.contains(req)) {
            return false;
        }
        if self.excluded.iter().any(|exc| capabilities.contains(exc)) {
            return false;
        }
        true
    }

    /// Calculate match score (higher is better)
    #[must_use]
    pub fn score(&self, capabilities: &[DiscoveryCapability]) -> u32 {
        if !self.matches(capabilities) {
            return 0;
        }

        let mut score = 100_u32;

        for preferred in &self.preferred {
            if capabilities.contains(preferred) {
                score += 10;
            }
        }

        score
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability query for finding providers
#[derive(Debug, Clone)]
pub struct CapabilityQuery {
    /// What capabilities are needed
    pub matcher: CapabilityMatcher,
    /// Context for the query
    pub context: HashMap<String, String>,
    /// Priority level (higher = more important)
    pub priority: u8,
}

impl CapabilityQuery {
    /// Create a new capability query
    #[must_use]
    pub fn new(matcher: CapabilityMatcher) -> Self {
        Self {
            matcher,
            context: HashMap::new(),
            priority: 5,
        }
    }

    /// Set priority
    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add context
    #[must_use]
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_matcher() {
        let capabilities = vec![
            DiscoveryCapability::ServiceRegistration,
            DiscoveryCapability::ServiceDiscovery,
            DiscoveryCapability::HealthChecking,
        ];

        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .prefer(DiscoveryCapability::HealthChecking)
            .exclude(DiscoveryCapability::Custom(String::from("legacy")));

        assert!(matcher.matches(&capabilities));
        assert_eq!(matcher.score(&capabilities), 110);
    }

    #[test]
    fn test_capability_matcher_exclusion() {
        let capabilities = vec![
            DiscoveryCapability::ServiceRegistration,
            DiscoveryCapability::Custom(String::from("legacy")),
        ];

        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .exclude(DiscoveryCapability::Custom(String::from("legacy")));

        assert!(!matcher.matches(&capabilities));
        assert_eq!(matcher.score(&capabilities), 0);
    }

    #[test]
    fn capability_query_default_priority_is_five() {
        let query = CapabilityQuery::new(CapabilityMatcher::new());
        assert_eq!(query.priority, 5);
        assert!(query.context.is_empty());
    }

    #[test]
    fn capability_query_with_priority_and_context() {
        let query = CapabilityQuery::new(CapabilityMatcher::new())
            .with_priority(9)
            .with_context("region".into(), "us-west".into());

        assert_eq!(query.priority, 9);
        assert_eq!(query.context.get("region").map(String::as_str), Some("us-west"));
    }

    #[test]
    fn matcher_requires_all_required_capabilities() {
        let caps = vec![DiscoveryCapability::ServiceRegistration];
        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .require(DiscoveryCapability::ServiceDiscovery);

        assert!(!matcher.matches(&caps));
        assert_eq!(matcher.score(&caps), 0);
    }

    #[test]
    fn matcher_multiple_preferred_boosts_score() {
        let caps = vec![
            DiscoveryCapability::ServiceRegistration,
            DiscoveryCapability::HealthChecking,
            DiscoveryCapability::ServiceMetrics,
        ];
        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .prefer(DiscoveryCapability::HealthChecking)
            .prefer(DiscoveryCapability::ServiceMetrics);

        assert!(matcher.matches(&caps));
        assert_eq!(matcher.score(&caps), 120);
    }

    #[test]
    fn matcher_filter_stores_custom_key_value() {
        let matcher = CapabilityMatcher::new()
            .filter("vendor".into(), "hashicorp".into())
            .require(DiscoveryCapability::ServiceDiscovery);

        assert_eq!(matcher.filters.get("vendor").map(String::as_str), Some("hashicorp"));
    }

    #[test]
    fn capability_serde_roundtrip() {
        let cap = DiscoveryCapability::Custom("federation".into());
        let json = serde_json::to_string(&cap).expect("serialize");
        let back: DiscoveryCapability = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, cap);
    }

    #[test]
    fn empty_capabilities_do_not_match_required() {
        let matcher = CapabilityMatcher::new().require(DiscoveryCapability::ServiceListing);
        assert!(!matcher.matches(&[]));
        assert_eq!(matcher.score(&[]), 0);
    }

    #[test]
    fn excluded_capability_blocks_even_when_required_met() {
        let caps =
            vec![DiscoveryCapability::ServiceDiscovery, DiscoveryCapability::VersionManagement];
        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceDiscovery)
            .exclude(DiscoveryCapability::VersionManagement);

        assert!(!matcher.matches(&caps));
    }
}
