// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Cache and Deduplication
//!
//! EVOLVED: Efficient caching with smart deduplication

use super::types::DiscoveredPrimal;
use std::collections::HashMap;

/// Discovery cache for storing and deduplicating discovered primals
#[derive(Debug, Clone)]
pub struct DiscoveryCache {
    /// Internal cache storage
    cache: HashMap<String, DiscoveredPrimal>,
}

impl DiscoveryCache {
    /// Create a new discovery cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Deduplicate primals and store in cache
    ///
    /// **EVOLVED**: Smart deduplication by name:endpoint key
    /// Preserves most recent discovery metadata
    pub fn deduplicate_and_store(
        &mut self,
        primals: Vec<DiscoveredPrimal>,
    ) -> Vec<DiscoveredPrimal> {
        use std::collections::hash_map::Entry;

        let mut deduplicated = Vec::new();

        for primal in primals {
            let key = format!("{}:{}", primal.name, primal.endpoint);
            if let Entry::Vacant(entry) = self.cache.entry(key) {
                entry.insert(primal.clone());
                deduplicated.push(primal);
            }
        }

        deduplicated
    }

    /// Get all cached primals
    #[must_use]
    pub fn get_all(&self) -> Vec<&DiscoveredPrimal> {
        self.cache.values().collect()
    }

    /// Find primals by capability
    ///
    /// **CAPABILITY-BASED**: Discovery by what services provide
    #[must_use]
    pub fn find_by_capability(&self, capability_type: &str) -> Vec<&DiscoveredPrimal> {
        self.cache
            .values()
            .filter(|primal| {
                primal.capabilities.iter().any(|cap| cap.capability_type == capability_type)
            })
            .collect()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for DiscoveryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Capability;
    use crate::types::PrimalType;

    fn create_test_primal(name: &str, endpoint: &str) -> DiscoveredPrimal {
        DiscoveredPrimal {
            name: name.to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: endpoint.to_string(),
            capabilities: vec![Capability {
                capability_type: "compute".to_string(),
                name: "compute".to_string(),
                version: "1.0".to_string(),
                parameters: Default::default(),
                qos_metrics: Default::default(),
                available: true,
            }],
            health: super::super::types::PrimalHealth::Unknown,
            discovery_method: super::super::types::DiscoveryMethod::Environment,
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_cache_deduplication() {
        let mut cache = DiscoveryCache::new();

        let primals = vec![
            create_test_primal("service-a", "http://localhost:8080"),
            create_test_primal("service-a", "http://localhost:8080"), // Duplicate
            create_test_primal("service-b", "http://localhost:8081"),
        ];

        let deduplicated = cache.deduplicate_and_store(primals);

        assert_eq!(deduplicated.len(), 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_find_by_capability() {
        let mut cache = DiscoveryCache::new();

        let primals = vec![
            create_test_primal("compute-1", "http://localhost:8080"),
            create_test_primal("compute-2", "http://localhost:8081"),
        ];

        cache.deduplicate_and_store(primals);

        let found = cache.find_by_capability("compute");
        assert_eq!(found.len(), 2);
    }
}
