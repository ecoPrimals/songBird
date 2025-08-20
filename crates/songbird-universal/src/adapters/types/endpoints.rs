//! Primal endpoint and provider definitions

use super::performance::{PerformanceMetrics, PrimalPerformanceMetrics};
use serde::{Deserialize, Serialize};
// use songbird_config::canonical::  // TEMPORARILY DISABLED - no canonical modulePrimalType;
use songbird_config::UniversalHealthStatus;
use uuid::Uuid;

/// Represents a primal endpoint that provides capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    pub endpoint_id: Uuid,
    pub primal_type: PrimalType,
    pub capabilities: Vec<String>,
    pub endpoint_url: String,
    pub performance_metrics: PrimalPerformanceMetrics,
}

/// Represents a primal that provides specific capabilities (unified core type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    /// Unique identifier for the provider
    pub provider_id: String,
    /// Alternative identifier field for compatibility
    pub id: String,
    /// Human-readable name for the provider
    pub display_name: String,
    /// List of capabilities this provider offers
    pub capabilities: Vec<String>,
    /// Base URL or connection string for the provider
    pub endpoint: String,
    /// Current health status
    pub health_status: UniversalHealthStatus,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Provider priority (lower numbers = higher priority)
    pub priority: u32,
    /// Type of primal (compute, storage, AI, etc.)
    pub primal_type: PrimalType,
}

impl CapabilityProvider {
    /// Create a new capability provider
    pub fn new(
        provider_id: String,
        display_name: String,
        capabilities: Vec<String>,
        endpoint: String,
        primal_type: PrimalType,
    ) -> Self {
        Self {
            id: provider_id.clone(),
            provider_id,
            display_name,
            capabilities,
            endpoint,
            health_status: UniversalHealthStatus::Unknown,
            performance_metrics: PerformanceMetrics::default(),
            priority: 100,
            primal_type,
        }
    }

    /// Check if provider has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if provider has all required capabilities
    pub fn has_capabilities(&self, required_capabilities: &[String]) -> bool {
        required_capabilities
            .iter()
            .all(|req| self.has_capability(req))
    }

    /// Check if provider is healthy and available
    pub fn is_available(&self) -> bool {
        matches!(
            self.health_status,
            UniversalHealthStatus::Healthy | UniversalHealthStatus::Degraded
        )
    }

    /// Get provider performance score
    pub fn performance_score(&self) -> f64 {
        self.performance_metrics.performance_score()
    }

    /// Update provider health status
    pub fn update_health(&mut self, health_status: UniversalHealthStatus) {
        self.health_status = health_status;
    }

    /// Update provider performance metrics
    pub fn update_performance(&mut self, metrics: PerformanceMetrics) {
        self.performance_metrics = metrics;
    }
}

impl Default for CapabilityProvider {
    fn default() -> Self {
        Self {
            provider_id: Uuid::new_v4().to_string(),
            id: Uuid::new_v4().to_string(),
            display_name: "Default Provider".to_string(),
            capabilities: vec![],
            endpoint: "http://localhost:{}".to_string(),
            health_status: UniversalHealthStatus::Unknown,
            performance_metrics: PerformanceMetrics::default(),
            priority: 100,
            primal_type: PrimalType::Unknown,
        }
    }
}

impl PrimalEndpoint {
    /// Create a new primal endpoint
    pub fn new(primal_type: PrimalType, capabilities: Vec<String>, endpoint_url: String) -> Self {
        Self {
            endpoint_id: Uuid::new_v4(),
            primal_type,
            capabilities,
            endpoint_url,
            performance_metrics: PrimalPerformanceMetrics::new(100, 100.0, 100.0),
        }
    }

    /// Check if endpoint supports a capability
    pub fn supports_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Get endpoint performance score
    pub fn performance_score(&self) -> f64 {
        self.performance_metrics.overall_score()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_provider_creation() {
        let provider = CapabilityProvider::new(
            "test-provider".to_string(),
            "Test Provider".to_string(),
            vec!["compute".to_string(), "storage".to_string()],
            "http://test.example.com".to_string(),
            PrimalType::Compute,
        );

        assert_eq!(provider.provider_id, "test-provider");
        assert_eq!(provider.display_name, "Test Provider");
        assert!(provider.has_capability("compute"));
        assert!(provider.has_capability("storage"));
        assert!(!provider.has_capability("ai"));
    }

    #[test]
    fn test_has_capabilities() {
        let provider = CapabilityProvider::new(
            "test".to_string(),
            "Test".to_string(),
            vec!["compute".to_string(), "storage".to_string()],
            "http://test.com".to_string(),
            PrimalType::Compute,
        );

        let required = vec!["compute".to_string()];
        assert!(provider.has_capabilities(&required));

        let required_both = vec!["compute".to_string(), "storage".to_string()];
        assert!(provider.has_capabilities(&required_both));

        let required_missing = vec!["compute".to_string(), "ai".to_string()];
        assert!(!provider.has_capabilities(&required_missing));
    }

    #[test]
    fn test_provider_availability() {
        let mut provider = CapabilityProvider::default();

        // Initially unknown, should not be available
        assert!(!provider.is_available());

        // Update to healthy
        provider.update_health(Universaltrue);
        assert!(provider.is_available());

        // Update to degraded (still available)
        provider.update_health(UniversalHealthStatus::Degraded);
        assert!(provider.is_available());

        // Update to failed (not available)
        provider.update_health(UniversalHealthStatus::Failed);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_primal_endpoint() {
        let endpoint = PrimalEndpoint::new(
            PrimalType::AI,
            vec!["machine_learning".to_string(), "inference".to_string()],
            "http://ai.example.com".to_string(),
        );

        assert!(endpoint.supports_capability("machine_learning"));
        assert!(endpoint.supports_capability("inference"));
        assert!(!endpoint.supports_capability("storage"));

        let score = endpoint.performance_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
}
