// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI workload classification delegated via capability-based discovery.
//!
//! Requests are forwarded to any primal that exposes AI capabilities; when none
//! are found, [`WorkloadClassification::basic_fallback`] provides a deterministic
//! heuristic result from the local [`WorkloadRequest`].

use songbird_config::capability_endpoints;
use songbird_types::SongbirdResult as Result;
use songbird_universal::capabilities::{DiscoveryConfig, UniversalCapabilityAdapter};
use tracing::{debug, warn};

pub mod types;
pub use types::*;

/// Delegates AI classification calls to a discovered AI-capable primal.
pub struct AIWorkloadClassificationDelegate {
    capability_adapter: UniversalCapabilityAdapter,
}

impl Default for AIWorkloadClassificationDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl AIWorkloadClassificationDelegate {
    /// Creates a delegate with default discovery configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capability_adapter: UniversalCapabilityAdapter::new(DiscoveryConfig::default()),
        }
    }

    async fn get_client(&self) -> Result<songbird_http_client::IpcHttpClient> {
        songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))
    }

    /// Classify a workload by delegating to any AI-capable primal.
    pub async fn classify_workload(
        &self,
        workload: &WorkloadRequest,
    ) -> Result<WorkloadClassification> {
        if let Some((ref provider_name, ref endpoint)) = self.get_ai_provider_endpoint().await {
            debug!("Delegating AI workload classification to: {provider_name}");
            self.send_classification_request(endpoint, workload).await
        } else {
            warn!("No AI providers available — using basic classification fallback");
            Ok(WorkloadClassification::basic_fallback(workload))
        }
    }

    /// Predict resource needs for a workload type.
    pub async fn predict_resources(
        &self,
        workload_type: &WorkloadType,
    ) -> Result<ResourceRequirements> {
        if let Some((ref provider_name, ref endpoint)) = self.get_ai_provider_endpoint().await {
            debug!("Delegating resource prediction to: {provider_name}");
            self.send_resource_prediction_request(endpoint, workload_type).await
        } else {
            warn!("No AI providers available — using basic resource estimation");
            Ok(ResourceRequirements::basic_estimation(workload_type))
        }
    }

    /// Assess risks for a workload given proposed resources.
    pub async fn assess_risks(
        &self,
        workload: &WorkloadRequest,
        resources: &ResourceRequirements,
    ) -> Result<RiskAssessment> {
        if let Some((ref provider_name, ref endpoint)) = self.get_ai_provider_endpoint().await {
            debug!("Delegating risk assessment to: {provider_name}");
            self.send_risk_assessment_request(endpoint, workload, resources).await
        } else {
            warn!("No AI providers available — using basic risk assessment");
            Ok(RiskAssessment::from_pressure(
                resources,
                &["No AI provider; heuristic risk only".to_string()],
            ))
        }
    }

    async fn get_ai_provider_endpoint(&self) -> Option<(String, String)> {
        let mut ai_providers = self.capability_adapter.find_capability_providers("ai").await;
        if ai_providers.is_empty() {
            ai_providers.extend(self.capability_adapter.find_capability_providers("ml").await);
            ai_providers
                .extend(self.capability_adapter.find_capability_providers("intelligence").await);
            ai_providers.extend(self.capability_adapter.find_capability_providers("model").await);
        }

        if ai_providers.is_empty() {
            debug!("No AI/ML capability providers found");
            return None;
        }

        let provider_name = ai_providers.into_iter().next()?;
        let endpoint = capability_endpoints::get_capability_endpoint("ai")
            .await
            .unwrap_or_else(|_| "http://localhost:8002".to_string());
        Some((provider_name, endpoint))
    }

    async fn send_classification_request(
        &self,
        endpoint: &str,
        workload: &WorkloadRequest,
    ) -> Result<WorkloadClassification> {
        let client = self.get_client().await?;
        let response = client
            .post(format!("{endpoint}/api/classify-workload"))
            .await
            .json(workload)
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?
            .send()
            .await
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?;

        if response.is_success() {
            let classification: WorkloadClassification = response
                .json()
                .await
                .map_err(|e| songbird_types::SongbirdError::serialization(e.to_string()))?;
            Ok(classification)
        } else {
            Err(songbird_types::SongbirdError::service(
                "ai_provider",
                format!("Classification failed: {}", response.status()),
            ))
        }
    }

    async fn send_resource_prediction_request(
        &self,
        endpoint: &str,
        workload_type: &WorkloadType,
    ) -> Result<ResourceRequirements> {
        let client = self.get_client().await?;
        let response = client
            .post(format!("{endpoint}/api/predict-resources"))
            .await
            .json(&serde_json::json!({ "workload_type": workload_type }))
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?
            .send()
            .await
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?;

        if response.is_success() {
            let requirements: ResourceRequirements = response
                .json()
                .await
                .map_err(|e| songbird_types::SongbirdError::serialization(e.to_string()))?;
            Ok(requirements)
        } else {
            Err(songbird_types::SongbirdError::service(
                "ai_provider",
                format!("Resource prediction failed: {}", response.status()),
            ))
        }
    }

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

        let client = self.get_client().await?;
        let response = client
            .post(format!("{endpoint}/api/assess-risks"))
            .await
            .json(&payload)
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?
            .send()
            .await
            .map_err(|e| songbird_types::SongbirdError::network(e.to_string()))?;

        if response.is_success() {
            let assessment: RiskAssessment = response
                .json()
                .await
                .map_err(|e| songbird_types::SongbirdError::serialization(e.to_string()))?;
            Ok(assessment)
        } else {
            Err(songbird_types::SongbirdError::service(
                "ai_provider",
                format!("Risk assessment failed: {}", response.status()),
            ))
        }
    }
}
