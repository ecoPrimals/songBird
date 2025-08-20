//! Load balancing and capability routing types

use songbird_errors::EvolvedResult;
use super::endpoints::CapabilityProvider;
use super::performance::PerformanceRequirements;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capability routing and load balancing
#[derive(Debug, Clone)]
pub struct CapabilityRouter {
    /// Available providers by capability type
    providers: HashMap<String, Vec<CapabilityProvider>>,
    /// Load balancing strategy
    strategy: LoadBalancingStrategy,
    /// Performance requirements
    requirements: PerformanceRequirements,
}

/// Load balancing strategies for capability routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Select provider with lowest current load
    LeastLoad,
    /// Select provider with best performance score
    BestPerformance,
    /// Random selection
    Random,
    /// Weighted selection based on performance
    Weighted,
}

impl CapabilityRouter {
    /// Create a new capability router
    pub fn new(strategy: LoadBalancingStrategy, requirements: PerformanceRequirements) -> Self {
        Self {
            providers: HashMap::new(),
            strategy,
            requirements,
        }
    }

    /// Add a provider to the router
    pub fn add_provider(&mut self, provider: CapabilityProvider) {
        for capability in &provider.capabilities {
            self.providers
                .entry(capability.clone())
                .or_default()
                .push(provider.clone());
        }
    }

    /// Remove a provider from the router
    pub fn remove_provider(&mut self, provider_id: &str) {
        for providers in self.providers.values_mut() {
            providers.retain(|p| p.provider_id != provider_id);
        }
    }

    /// Get providers for a specific capability
    pub fn get_providers_for_capability(&self, capability: &str) -> Vec<&CapabilityProvider> {
        self.providers
            .get(capability)
            .map(|providers| providers.iter().collect())
            .unwrap_or_default()
    }

    /// Select best provider for a capability based on strategy
    pub fn select_provider(&self, capability: &str) -> Option<&CapabilityProvider> {
        let providers = self.get_providers_for_capability(capability);
        if providers.is_empty() {
            return None;
        }

        // Filter available providers that meet requirements
        let available_providers: Vec<_> = providers
            .into_iter()
            .filter(|p| {
                p.is_available() && p.performance_metrics.meets_requirements(&self.requirements)
            })
            .collect();

        if available_providers.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                // Simple implementation - in production would maintain state
                available_providers.first().copied()
            }
            LoadBalancingStrategy::LeastLoad => available_providers.into_iter().min_by(|a, b| {
                a.performance_metrics
                    .current_load
                    .partial_cmp(&b.performance_metrics.current_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            LoadBalancingStrategy::BestPerformance => {
                available_providers.into_iter().max_by(|a, b| {
                    a.performance_score()
                        .partial_cmp(&b.performance_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
            LoadBalancingStrategy::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                // Simple deterministic random based on provider count
                let mut hasher = DefaultHasher::new();
                available_providers.len().hash(&mut hasher);
                let hash = hasher.finish();
                let index = (hash as usize) % available_providers.len();
                available_providers.get(index).copied()
            }
            LoadBalancingStrategy::Weighted => {
                // Weighted by inverse of current load
                available_providers.into_iter().min_by(|a, b| {
                    let weight_a = 1.0 / (a.performance_metrics.current_load + 1.0);
                    let weight_b = 1.0 / (b.performance_metrics.current_load + 1.0);
                    weight_b
                        .partial_cmp(&weight_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
    }

    /// Get all available capabilities
    pub fn get_available_capabilities(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get provider count for a capability
    pub fn get_provider_count(&self, capability: &str) -> usize {
        self.providers
            .get(capability)
            .map(|providers| providers.len())
            .unwrap_or(0)
    }

    /// Get total provider count across all capabilities
    pub fn total_provider_count(&self) -> usize {
        self.providers
            .values()
            .map(|providers| providers.len())
            .sum()
    }

    /// Update load balancing strategy
    pub fn set_strategy(&mut self, strategy: LoadBalancingStrategy) {
        self.strategy = strategy;
    }

    /// Update performance requirements
    pub fn set_requirements(&mut self, requirements: PerformanceRequirements) {
        self.requirements = requirements;
    }

    /// Get current strategy
    pub fn get_strategy(&self) -> &LoadBalancingStrategy {
        &self.strategy
    }

    /// Get current requirements
    pub fn get_requirements(&self) -> &PerformanceRequirements {
        &self.requirements
    }

    /// Get health summary of all providers
    pub fn get_health_summary(&self) -> HashMap<String, (usize, usize)> {
        let mut summary = HashMap::new();

        for (capability, providers) in &self.providers {
            let total = providers.len();
            let healthy = providers.iter().filter(|p| p.is_available()).count();
            summary.insert(capability.clone(), (healthy, total));
        }

        summary
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new(
            LoadBalancingStrategy::BestPerformance,
            PerformanceRequirements::default(),
        )
    }
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::BestPerformance
    }
}

#[cfg(test)]
mod tests {
    use super::super::performance::PerformanceMetrics;
    use super::*;
    // use songbird_config::canonical::  // TEMPORARILY DISABLED - no canonical modulePrimalType;
    use songbird_config::UniversalHealthStatus;

    fn create_test_provider(
        id: &str,
        capabilities: Vec<&str>,
        load: f64,
        healthy: bool,
    ) -> CapabilityProvider {
        let mut provider = CapabilityProvider::new(
            id.to_string(),
            format!("Provider {id}"),
            capabilities.into_iter().map(|s| s.to_string()).collect(),
            format!("http://{id}.example.com"),
            PrimalType::Compute,
        );

        provider.performance_metrics = PerformanceMetrics::new(100.0, 95.0, load);
        provider.health_status = if healthy {
            UniversalHealthStatus::Healthy
        } else {
            UniversalHealthStatus::Failed
        };

        provider
    }

    #[test]
    fn test_router_creation() {
        let router = CapabilityRouter::new(
            LoadBalancingStrategy::RoundRobin,
            PerformanceRequirements::default(),
        );

        assert_eq!(router.total_provider_count(), 0);
        assert!(router.get_available_capabilities().is_empty());
    }

    #[test]
    fn test_add_remove_providers() {
        let mut router = CapabilityRouter::default();

        let provider1 = create_test_provider("p1", vec!["compute", "storage"], 50.0, true);
        let provider2 = create_test_provider("p2", vec!["compute"], 30.0, true);

        router.add_provider(provider1);
        router.add_provider(provider2);

        assert_eq!(router.get_provider_count("compute"), 2);
        assert_eq!(router.get_provider_count("storage"), 1);
        assert_eq!(router.total_provider_count(), 3); // p1 counted twice

        router.remove_provider("p1");
        assert_eq!(router.get_provider_count("compute"), 1);
        assert_eq!(router.get_provider_count("storage"), 0);
    }

    #[test]
    fn test_provider_selection_strategies() -> SongbirdResult<()> {
        let mut router = CapabilityRouter::default();

        let low_load = create_test_provider("low", vec!["compute"], 20.0, true);
        let high_load = create_test_provider("high", vec!["compute"], 80.0, true);
        let unhealthy = create_test_provider("unhealthy", vec!["compute"], 10.0, false);

        router.add_provider(low_load);
        router.add_provider(high_load);
        router.add_provider(unhealthy);

        // Test least load strategy
        router.set_strategy(LoadBalancingStrategy::LeastLoad);
        let selected = router.select_provider("compute");
        assert!(selected.is_some());
        assert_eq!(
            selected.ok_or_else(|| "No provider selected")?.provider_id,
            "low"
        );

        // Test best performance strategy
        router.set_strategy(LoadBalancingStrategy::BestPerformance);
        let selected = router.select_provider("compute");
        assert!(selected.is_some());

        // Should not select unhealthy provider
        let selected_ids: Vec<_> = (0..10)
            .filter_map(|_| router.select_provider("compute"))
            .map(|p| &p.provider_id)
            .collect();
        assert!(!selected_ids.contains(&&"unhealthy".to_string()));

        Ok(SongbirdResponse::success(()))
    }

    #[test]
    fn test_health_summary() {
        let mut router = CapabilityRouter::default();

        router.add_provider(create_test_provider("p1", vec!["compute"], 50.0, true));
        router.add_provider(create_test_provider("p2", vec!["compute"], 60.0, false));
        router.add_provider(create_test_provider("p3", vec!["storage"], 40.0, true));

        let summary = router.get_health_summary();

        // compute: 1 healthy out of 2 total
        assert_eq!(summary.get("compute"), Some(&(1, 2)));
        // storage: 1 healthy out of 1 total
        assert_eq!(summary.get("storage"), Some(&(1, 1)));
    }

    #[test]
    fn test_no_providers_available() -> SongbirdResult<()> {
        let router = CapabilityRouter::default();

        let selected = router.select_provider("nonexistent");
        assert!(selected.is_none());

        assert_eq!(router.get_provider_count("nonexistent"), 0);
        assert!(router
            .get_providers_for_capability("nonexistent")
            .is_empty());

        Ok(SongbirdResponse::success(()))
    }
}
