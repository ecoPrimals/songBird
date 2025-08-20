//! Service and performance requirement types

use super::enums::CapabilityType;
use serde::{Deserialize, Serialize};

/// Service requirements for capability matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirements {
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Optional capabilities (nice to have)
    pub optional_capabilities: Vec<String>,
    /// Preferred capabilities (for optimization)
    pub preferred_capabilities: Vec<String>,
    /// Capability-specific requirements
    pub capability_requirements: Vec<CapabilityType>,
    /// Minimum performance requirements
    pub performance_requirements: super::performance::PerformanceRequirements,
    /// Minimum performance threshold
    pub min_performance: f64,
    /// Maximum latency in milliseconds
    pub max_latency_ms: u64,
    /// Minimum success rate (0.0 to 1.0)
    pub min_success_rate: f64,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Security requirements
    pub security_level: SecurityRequirement,
    /// Geographic requirements
    pub geographic_requirements: Option<GeographicRequirement>,
    /// Compliance requirements
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

/// Security requirement levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityRequirement {
    /// No special security requirements
    None,
    /// Basic encryption in transit
    Basic,
    /// Encryption in transit and at rest
    Standard,
    /// High security with additional controls
    High,
    /// Maximum security with hardware requirements
    Maximum,
}

/// Geographic requirements for service placement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRequirement {
    /// Preferred regions
    pub preferred_regions: Vec<String>,
    /// Excluded regions
    pub excluded_regions: Vec<String>,
    /// Data residency requirements
    pub data_residency: Option<String>,
    /// Maximum latency tolerance in milliseconds
    pub max_latency_ms: Option<u64>,
}

/// Compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceRequirement {
    /// General Data Protection Regulation
    GDPR,
    /// Health Insurance Portability and Accountability Act
    HIPAA,
    /// Payment Card Industry Data Security Standard
    PCIDSS,
    /// SOC 2 Type II
    SOC2,
    /// ISO 27001
    ISO27001,
    /// Federal Risk and Authorization Management Program
    FedRAMP,
    /// Custom compliance requirement
    Custom(String),
}

impl ServiceRequirements {
    /// Create new service requirements
    pub fn new(required_capabilities: Vec<String>) -> Self {
        Self {
            required_capabilities,
            optional_capabilities: Vec::new(),
            preferred_capabilities: Vec::new(),
            capability_requirements: Vec::new(),
            performance_requirements: super::performance::PerformanceRequirements::default(),
            min_performance: 0.8,
            max_latency_ms: 1000,
            min_success_rate: 0.95,
            max_response_time_ms: 5000,
            security_level: SecurityRequirement::Standard,
            geographic_requirements: None,
            compliance_requirements: Vec::new(),
        }
    }

    /// Add optional capabilities
    pub fn with_optional_capabilities(mut self, optional: Vec<String>) -> Self {
        self.optional_capabilities = optional;
        self
    }

    /// Add capability-specific requirements
    pub fn with_capability_requirements(mut self, requirements: Vec<CapabilityType>) -> Self {
        self.capability_requirements = requirements;
        self
    }

    /// Set performance requirements
    pub fn with_performance_requirements(
        mut self,
        requirements: super::performance::PerformanceRequirements,
    ) -> Self {
        self.performance_requirements = requirements;
        self
    }

    /// Set security level
    pub fn with_security_level(mut self, level: SecurityRequirement) -> Self {
        self.security_level = level;
        self
    }

    /// Add geographic requirements
    pub fn with_geographic_requirements(mut self, requirements: GeographicRequirement) -> Self {
        self.geographic_requirements = Some(requirements);
        self
    }

    /// Add compliance requirements
    pub fn with_compliance_requirements(
        mut self,
        requirements: Vec<ComplianceRequirement>,
    ) -> Self {
        self.compliance_requirements = requirements;
        self
    }

    /// Check if requirements are satisfied by a capability list
    pub fn are_satisfied_by(&self, available_capabilities: &[String]) -> bool {
        // Check required capabilities
        for required in &self.required_capabilities {
            if !available_capabilities.contains(required) {
                return false;
            }
        }
        true
    }

    /// Calculate satisfaction score (0.0 to 1.0)
    pub fn satisfaction_score(&self, available_capabilities: &[String]) -> f64 {
        if !self.are_satisfied_by(available_capabilities) {
            return 0.0;
        }

        let required_count = self.required_capabilities.len() as f64;
        let optional_count = self.optional_capabilities.len() as f64;

        if required_count == 0.0 && optional_count == 0.0 {
            return 1.0;
        }

        // Count satisfied optional capabilities
        let satisfied_optional = self
            .optional_capabilities
            .iter()
            .filter(|opt| available_capabilities.contains(opt))
            .count() as f64;

        // Base score for required capabilities (0.7)
        // Additional score for optional capabilities (0.3)
        let base_score = 0.7;
        let optional_score = if optional_count > 0.0 {
            0.3 * (satisfied_optional / optional_count)
        } else {
            0.3 // No optional requirements = full optional score
        };

        base_score + optional_score
    }

    /// Get all capabilities (required + optional)
    pub fn all_capabilities(&self) -> Vec<&String> {
        self.required_capabilities
            .iter()
            .chain(self.optional_capabilities.iter())
            .collect()
    }

    /// Check if security level is met
    pub fn security_level_met(&self, provided_level: &SecurityRequirement) -> bool {
        use SecurityRequirement::*;
        matches!(
            (&self.security_level, provided_level),
            (None, _)
                | (Basic, Basic | Standard | High | Maximum)
                | (Standard, Standard | High | Maximum)
                | (High, High | Maximum)
                | (Maximum, Maximum)
        )
    }
}

impl GeographicRequirement {
    /// Create new geographic requirement
    pub fn new() -> Self {
        Self {
            preferred_regions: Vec::new(),
            excluded_regions: Vec::new(),
            data_residency: None,
            max_latency_ms: None,
        }
    }

    /// Add preferred regions
    pub fn with_preferred_regions(mut self, regions: Vec<String>) -> Self {
        self.preferred_regions = regions;
        self
    }

    /// Add excluded regions
    pub fn with_excluded_regions(mut self, regions: Vec<String>) -> Self {
        self.excluded_regions = regions;
        self
    }

    /// Set data residency requirement
    pub fn with_data_residency(mut self, residency: String) -> Self {
        self.data_residency = Some(residency);
        self
    }

    /// Set maximum latency tolerance
    pub fn with_max_latency(mut self, latency_ms: u64) -> Self {
        self.max_latency_ms = Some(latency_ms);
        self
    }

    /// Check if region is acceptable
    pub fn is_region_acceptable(&self, region: &str) -> bool {
        if self.excluded_regions.contains(&region.to_string()) {
            return false;
        }

        if self.preferred_regions.is_empty() {
            return true;
        }

        self.preferred_regions.contains(&region.to_string())
    }

    /// Calculate region preference score (0.0 to 1.0)
    pub fn region_preference_score(&self, region: &str) -> f64 {
        if !self.is_region_acceptable(region) {
            return 0.0;
        }

        if self.preferred_regions.is_empty() {
            return 1.0;
        }

        if self.preferred_regions.contains(&region.to_string()) {
            1.0
        } else {
            0.5 // Acceptable but not preferred
        }
    }
}

impl Default for ServiceRequirements {
    fn default() -> Self {
        Self {
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
            preferred_capabilities: Vec::new(),
            capability_requirements: Vec::new(),
            performance_requirements: super::performance::PerformanceRequirements::default(),
            min_performance: 0.8,
            max_latency_ms: 1000,
            min_success_rate: 0.95,
            max_response_time_ms: 5000,
            security_level: SecurityRequirement::Standard,
            geographic_requirements: None,
            compliance_requirements: Vec::new(),
        }
    }
}

impl Default for GeographicRequirement {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityRequirement {
    /// Get security level as numeric value (higher = more secure)
    pub fn level_value(&self) -> u8 {
        match self {
            SecurityRequirement::None => 0,
            SecurityRequirement::Basic => 1,
            SecurityRequirement::Standard => 2,
            SecurityRequirement::High => 3,
            SecurityRequirement::Maximum => 4,
        }
    }

    /// Check if this level is at least as secure as the required level
    pub fn meets_or_exceeds(&self, required: &SecurityRequirement) -> bool {
        self.level_value() >= required.level_value()
    }
}

impl ComplianceRequirement {
    /// Get compliance requirement name
    pub fn name(&self) -> &str {
        match self {
            ComplianceRequirement::GDPR => "GDPR",
            ComplianceRequirement::HIPAA => "HIPAA",
            ComplianceRequirement::PCIDSS => "PCI DSS",
            ComplianceRequirement::SOC2 => "SOC 2",
            ComplianceRequirement::ISO27001 => "ISO 27001",
            ComplianceRequirement::FedRAMP => "FedRAMP",
            ComplianceRequirement::Custom(name) => name,
        }
    }

    /// Check if compliance requirement is region-specific
    pub fn is_region_specific(&self) -> bool {
        matches!(
            self,
            ComplianceRequirement::GDPR | ComplianceRequirement::FedRAMP
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_requirements_creation() {
        let requirements =
            ServiceRequirements::new(vec!["compute".to_string(), "storage".to_string()]);

        assert_eq!(requirements.required_capabilities.len(), 2);
        assert!(requirements
            .required_capabilities
            .contains(&"compute".to_string()));
        assert!(requirements
            .required_capabilities
            .contains(&"storage".to_string()));
        assert_eq!(requirements.security_level, SecurityRequirement::Standard);
    }

    #[test]
    fn test_requirements_satisfaction() {
        let requirements = ServiceRequirements::new(vec!["compute".to_string()])
            .with_optional_capabilities(vec!["storage".to_string(), "ai".to_string()]);

        let available1 = vec!["compute".to_string()];
        assert!(requirements.are_satisfied_by(&available1));

        let available2 = vec!["compute".to_string(), "storage".to_string()];
        assert!(requirements.are_satisfied_by(&available2));

        let available3 = vec!["storage".to_string()];
        assert!(!requirements.are_satisfied_by(&available3)); // Missing required

        // Test satisfaction scores
        let score1 = requirements.satisfaction_score(&available1);
        let score2 = requirements.satisfaction_score(&available2);

        assert!(score1 > 0.0);
        assert!(score2 > score1); // More optional capabilities = higher score
    }

    #[test]
    fn test_security_requirements() {
        assert!(SecurityRequirement::High.meets_or_exceeds(&SecurityRequirement::Standard));
        assert!(SecurityRequirement::Maximum.meets_or_exceeds(&SecurityRequirement::High));
        assert!(!SecurityRequirement::Basic.meets_or_exceeds(&SecurityRequirement::High));

        let requirements = ServiceRequirements::default();
        assert!(requirements.security_level_met(&SecurityRequirement::High));
        assert!(!requirements.security_level_met(&SecurityRequirement::Basic));
    }

    #[test]
    fn test_geographic_requirements() {
        let geo_req = GeographicRequirement::new()
            .with_preferred_regions(vec!["us-west".to_string(), "us-east".to_string()])
            .with_excluded_regions(vec!["restricted-region".to_string()])
            .with_max_latency(100);

        assert!(geo_req.is_region_acceptable("us-west"));
        assert!(geo_req.is_region_acceptable("us-east"));
        assert!(!geo_req.is_region_acceptable("restricted-region"));

        assert_eq!(geo_req.region_preference_score("us-west"), 1.0);
        assert_eq!(geo_req.region_preference_score("restricted-region"), 0.0);
        assert_eq!(geo_req.max_latency_ms, Some(100));
    }

    #[test]
    fn test_compliance_requirements() {
        let gdpr = ComplianceRequirement::GDPR;
        let custom = ComplianceRequirement::Custom("MyCompliance".to_string());

        assert_eq!(gdpr.name(), "GDPR");
        assert_eq!(custom.name(), "MyCompliance");

        assert!(gdpr.is_region_specific());
        assert!(!ComplianceRequirement::SOC2.is_region_specific());
    }

    #[test]
    fn test_builder_pattern() {
        let requirements = ServiceRequirements::new(vec!["compute".to_string()])
            .with_optional_capabilities(vec!["storage".to_string()])
            .with_security_level(SecurityRequirement::High)
            .with_geographic_requirements(
                GeographicRequirement::new().with_preferred_regions(vec!["us-west".to_string()]),
            )
            .with_compliance_requirements(vec![ComplianceRequirement::SOC2]);

        assert_eq!(requirements.required_capabilities.len(), 1);
        assert_eq!(requirements.optional_capabilities.len(), 1);
        assert_eq!(requirements.security_level, SecurityRequirement::High);
        assert!(requirements.geographic_requirements.is_some());
        assert_eq!(requirements.compliance_requirements.len(), 1);
    }
}
