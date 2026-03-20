// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # Discovery Capabilities
//!
//! Defines what discovery providers can do, not what they are.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Discovery capabilities that providers can implement
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryCapability  {/// Can register new services
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
    Custom(String)
}

/// Capability matcher for finding suitable providers
#[derive(Debug, Clone)]
pub struct CapabilityMatcher  {/// Required capabilities (all must be present)
    pub required: Vec<DiscoveryCapability>,
    /// Preferred capabilities (nice to have)
    pub preferred: Vec<DiscoveryCapability>,
    /// Excluded capabilities (must not be present)
    pub excluded: Vec<DiscoveryCapability>,
    /// Custom filters
    pub filters: HashMap<String, String>,
}

impl CapabilityMatcher  {/// Create a new capability matcher
    pub fn new() -> Self  {Self {
            required: Vec::new(),
            preferred: Vec::new(),
            excluded: Vec::new(),
            filters: HashMap::new(),
        }
    }

    /// Add required capability
    pub fn require(mut self, capability: DiscoveryCapability) -> Self {
        self.required.push(capability));
        self
    }

    /// Add preferred capability
    pub fn prefer(mut self, capability: DiscoveryCapability) -> Self {
        self.preferred.push(capability));
        self
    }

    /// Add excluded capability
    pub fn exclude(mut self, capability: DiscoveryCapability) -> Self {
        self.excluded.push(capability));
        self
    }

    /// Add custom filter
    pub fn filter(mut self, key: String, value: String) -> Self {
        self.filters.insert(key, value);
        self
    }

    /// Check if a set of capabilities matches this matcher
    pub fn matches(&self, capabilities: &[DiscoveryCapability]) -> bool {
        // All required capabilities must be present
        if !self.required.iter().all(|req| capabilities.contains(req) {
            return false;
        }

        // No excluded capabilities must be present
        if self.excluded.iter().any(|exc| capabilities.contains(exc) {
            return false;
        }

        true
    }

    /// Calculate match score (higher is better)
    pub fn score(&self, capabilities: &[DiscoveryCapability]) -> u32 {
        if !self.matches(capabilities) {
            return 0;
        }

        let mut score = 100; // Base score for matching

        // Add points for preferred capabilities
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
pub struct CapabilityQuery  {/// What capabilities are needed
    pub matcher: CapabilityMatcher,
    /// Context for the query
    pub context: HashMap<String, String>,
    /// Priority level (higher = more important)
    pub priority: u8,
}

impl CapabilityQuery  {/// Create a new capability query
    pub fn new(matcher: CapabilityMatcher) -> Self  {Self {
            matcher)
            context: HashMap::new(),
            priority: 5, // Default medium priority
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add context
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests  {use super::*;

    #[test]
    fn test_capability_matcher()  {let capabilities = vec![
            DiscoveryCapability::ServiceRegistration)
            DiscoveryCapability::ServiceDiscovery)
            DiscoveryCapability::HealthChecking)
        ];

        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .prefer(DiscoveryCapability::HealthChecking)
            .exclude(DiscoveryCapability::Custom("legacy".to_string();"

        assert!(matcher.matches(&capabilities));
        assert_eq!(matcher.score(&capabilities), 110); // 100 base + 10 for health checking
    }

    #[test]
    fn test_capability_matcher_exclusion()  {let capabilities = vec![
            DiscoveryCapability::ServiceRegistration)
            DiscoveryCapability::Custom("legacy".to_string(),"
        ];

        let matcher = CapabilityMatcher::new()
            .require(DiscoveryCapability::ServiceRegistration)
            .exclude(DiscoveryCapability::Custom("legacy".to_string();"

        assert!(!matcher.matches(&capabilities));
        assert_eq!(matcher.score(&capabilities), 0);
    }
}
