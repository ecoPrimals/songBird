//! AI Workload Classification - Delegated via Capability-Based Discovery
//!
//! This module provides a delegation layer that forwards AI workload classification
//! requests to any primal that provides AI capabilities, maintaining Songbird's role
//! as an orchestrator rather than implementing AI functionality directly.

use songbird_errors::Result;
use songbird_universal::capabilities::UniversalCapabilityAdapter;
use songbird_universal::DiscoveryConfig;
use std::collections::HashMap;
use tracing::{debug, info, warn};

// Re-export types that are still needed for interface compatibility
pub use types::*;

pub mod types;

/// AI Workload Classification Delegate with Capability-Based Discovery
///
/// This struct delegates AI classification requests to any primal that provides
/// AI capabilities instead of being hardcoded to a specific primal.
pub struct AIWorkloadClassificationDelegate {
    capability_adapter: UniversalCapabilityAdapter,
    http_client: reqwest::Client,
}

impl Default for AIWorkloadClassificationDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl AIWorkloadClassificationDelegate {
    /// Create a new AI workload classification delegate
    pub fn new() -> Self {
        let discovery_config = DiscoveryConfig::default();
        let capability_adapter = UniversalCapabilityAdapter::new(discovery_config);

        Self {
            capability_adapter,
            http_client: reqwest::Client::new(),
        }
    }

    /// Classify a workload by delegating to any AI-capable primal
    pub async fn classify_workload(
        &self,
        workload: &WorkloadRequest,
    ) -> Result<WorkloadClassification> {
        match self.get_ai_provider_endpoint().await {
            Some((provider_name, endpoint)) => {
                debug!(
                    "🤖 Delegating AI workload classification to: {}",
                    provider_name
                );
                self.send_classification_request(&endpoint, workload).await
            }
            None => {
                warn!("🎼 No AI providers available - using basic classification fallback");
                Ok(WorkloadClassification::basic_fallback(workload))
            }
        }
    }

    /// Get resource predictions by delegating to any AI-capable primal
    pub async fn predict_resources(
        &self,
        workload_type: &WorkloadType,
    ) -> Result<ResourceRequirements> {
        match self.get_ai_provider_endpoint().await {
            Some((provider_name, endpoint)) => {
                debug!("🤖 Delegating resource prediction to: {}", provider_name);
                self.send_resource_prediction_request(&endpoint, workload_type)
                    .await
            }
            None => {
                warn!("🎼 No AI providers available - using basic resource estimation");
                Ok(ResourceRequirements::basic_estimation(workload_type))
            }
        }
    }

    /// Assess risks by delegating to any AI-capable primal
    pub async fn assess_risks(
        &self,
        workload: &WorkloadRequest,
        resources: &ResourceRequirements,
    ) -> Result<RiskAssessment> {
        match self.get_ai_provider_endpoint().await {
            Some((provider_name, endpoint)) => {
                debug!("🤖 Delegating risk assessment to: {}", provider_name);
                self.send_risk_assessment_request(&endpoint, workload, resources)
                    .await
            }
            None => {
                warn!("🎼 No AI providers available - using basic risk assessment");
                Ok(RiskAssessment::basic_assessment())
            }
        }
    }

    /// Find the best AI provider using capability-based discovery
    async fn get_ai_provider_endpoint(&self) -> Option<(String, String)> {
        // Find primals that provide AI capabilities (could be squirrel, or any other AI primal)
        let ai_providers = self
            .capability_adapter
            .find_capability_providers("ai")
            .await;

        if ai_providers.is_empty() {
            // Try alternative capability names
            let ml_providers = self
                .capability_adapter
                .find_capability_providers("ml")
                .await;
            let intelligence_providers = self
                .capability_adapter
                .find_capability_providers("intelligence")
                .await;
            let model_providers = self
                .capability_adapter
                .find_capability_providers("model")
                .await;

            let mut all_providers = Vec::new();
            all_providers.extend(ml_providers);
            all_providers.extend(intelligence_providers);
            all_providers.extend(model_providers);

            if all_providers.is_empty() {
                debug!("🔍 No AI/ML capability providers found");
                return None;
            }

            // Use the first available provider
            let provider_name = all_providers.into_iter().next()?;
            let endpoint = self.get_primal_endpoint(&provider_name);
            return Some((provider_name, endpoint));
        }

        // Get the best AI provider (for now, just use the first one)
        // TODO: Implement QoS-based selection using capability_adapter.get_best_primal_for_capability()
        let provider_name = ai_providers.into_iter().next()?;
        let endpoint = self.get_primal_endpoint(&provider_name);

        Some((provider_name, endpoint))
    }

    /// Get endpoint for a primal by name
    fn get_primal_endpoint(&self, primal_name: &str) -> String {
        songbird_config::config::constants::get_primal_endpoint(primal_name)
    }

    /// Send classification request to AI provider
    async fn send_classification_request(
        &self,
        endpoint: &str,
        workload: &WorkloadRequest,
    ) -> Result<WorkloadClassification> {
        let response = self
            .http_client
            .post(&format!("{}/api/classify-workload", endpoint))
            .json(workload)
            .send()
            .await
            .map_err(|e| songbird_errors::SongbirdError::network_error(e.to_string()))?;

        if response.status().is_success() {
            let classification: WorkloadClassification = response
                .json()
                .await
                .map_err(|e| songbird_errors::SongbirdError::io_error(e.to_string()))?;
            Ok(classification)
        } else {
            Err(songbird_errors::SongbirdError::service_error(
                "ai_provider",
                format!("Classification failed: {}", response.status()),
            ))
        }
    }

    /// Send resource prediction request to AI provider
    async fn send_resource_prediction_request(
        &self,
        endpoint: &str,
        workload_type: &WorkloadType,
    ) -> Result<ResourceRequirements> {
        let response = self
            .http_client
            .post(&format!("{}/api/predict-resources", endpoint))
            .json(workload_type)
            .send()
            .await
            .map_err(|e| songbird_errors::SongbirdError::network_error(e.to_string()))?;

        if response.status().is_success() {
            let requirements: ResourceRequirements = response
                .json()
                .await
                .map_err(|e| songbird_errors::SongbirdError::io_error(e.to_string()))?;
            Ok(requirements)
        } else {
            Err(songbird_errors::SongbirdError::service_error(
                "ai_provider",
                format!("Resource prediction failed: {}", response.status()),
            ))
        }
    }

    /// Send risk assessment request to AI provider
    async fn send_risk_assessment_request(
        &self,
        endpoint: &str,
        workload: &WorkloadRequest,
        resources: &ResourceRequirements,
    ) -> Result<RiskAssessment> {
        let payload = serde_json::json!({
            "workload": workload,
            "resources": resources
        });

        let response = self
            .http_client
            .post(&format!("{}/api/assess-risks", endpoint))
            .json(&payload)
            .send()
            .await
            .map_err(|e| songbird_errors::SongbirdError::network_error(e.to_string()))?;

        if response.status().is_success() {
            let assessment: RiskAssessment = response
                .json()
                .await
                .map_err(|e| songbird_errors::SongbirdError::io_error(e.to_string()))?;
            Ok(assessment)
        } else {
            Err(songbird_errors::SongbirdError::service_error(
                "ai_provider",
                format!("Risk assessment failed: {}", response.status()),
            ))
        }
    }
}

/// Helper trait for basic fallback implementations
trait BasicFallback {
    fn basic_fallback(workload: &WorkloadRequest) -> Self;
    fn basic_estimation(workload_type: &WorkloadType) -> Self;
    fn basic_assessment() -> Self;
}

impl BasicFallback for WorkloadClassification {
    fn basic_fallback(_workload: &WorkloadRequest) -> Self {
        Self {
            workload_type: WorkloadType::Generic,
            confidence_score: 0.5,
            resource_requirements: ResourceRequirements::basic_estimation(&WorkloadType::Generic),
            performance_prediction: PerformancePrediction::default(),
            risk_assessment: RiskAssessment::basic_assessment(),
        }
    }

    fn basic_estimation(_workload_type: &WorkloadType) -> Self {
        // This method doesn't make sense for WorkloadClassification
        // but it's required by the trait
        Self {
            workload_type: WorkloadType::Generic,
            confidence_score: 0.5,
            resource_requirements: ResourceRequirements::default(),
            performance_prediction: PerformancePrediction::default(),
            risk_assessment: RiskAssessment::basic_assessment(),
        }
    }

    fn basic_assessment() -> Self {
        Self {
            workload_type: WorkloadType::Generic,
            confidence_score: 0.5,
            resource_requirements: ResourceRequirements::default(),
            performance_prediction: PerformancePrediction::default(),
            risk_assessment: RiskAssessment::basic_assessment(),
        }
    }
}

impl BasicFallback for ResourceRequirements {
    fn basic_fallback(_workload: &WorkloadRequest) -> Self {
        Self::default()
    }

    fn basic_estimation(_workload_type: &WorkloadType) -> Self {
        Self {
            cpu_cores: 2,
            memory_mb: 4096,
            storage_mb: 10240,
            network_bandwidth_mbps: 100,
            priority: ResourcePriority::Medium,
        }
    }

    fn basic_assessment() -> Self {
        Self::default()
    }
}

impl BasicFallback for RiskAssessment {
    fn basic_fallback(_workload: &WorkloadRequest) -> Self {
        Self::basic_assessment()
    }

    fn basic_estimation(_workload_type: &WorkloadType) -> Self {
        Self::basic_assessment()
    }

    fn basic_assessment() -> Self {
        Self {
            overall_risk_score: 0.3, // Low risk for basic operations
            risk_factors: vec![],
            mitigation_strategies: vec!["Use basic resource limits".to_string()],
            confidence: 0.7, // Moderate confidence in basic assessment
        }
    }
}
