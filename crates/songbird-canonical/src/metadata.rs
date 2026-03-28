// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI-first metadata types for the Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI-optimized metadata for responses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIResponseMetadata {
    /// AI decision-making context
    pub decision_context: DecisionContext,

    /// Automation capabilities for this response
    pub automation_capabilities: Vec<AutomationCapability>,

    /// Quality metrics for AI evaluation
    pub quality_metrics: QualityMetrics,

    /// Custom metadata fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl AIResponseMetadata {
    /// Add an automation capability
    #[must_use]
    pub fn with_automation_capability(mut self, capability: AutomationCapability) -> Self {
        self.automation_capabilities.push(capability);
        self
    }

    /// Add a custom field
    #[must_use]
    pub fn with_custom_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let _ = self.custom_fields.insert(key.into(), value);
        self
    }
}

/// Context for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// Factors that influenced this response
    pub influencing_factors: Vec<String>,

    /// Alternative options that were considered
    pub alternatives_considered: Vec<String>,

    /// Reasoning for the chosen approach
    pub reasoning: Option<String>,

    /// Risk assessment
    pub risk_level: RiskLevel,
}

impl Default for DecisionContext {
    fn default() -> Self {
        Self {
            influencing_factors: Vec::new(),
            alternatives_considered: Vec::new(),
            reasoning: None,
            risk_level: RiskLevel::Low,
        }
    }
}

/// Risk levels for AI assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk operation
    Low,
    /// Medium risk operation
    Medium,
    /// High risk operation (requires human oversight)
    High,
    /// Critical risk operation (requires immediate human intervention)
    Critical,
}

/// Automation capabilities for AI agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomationCapability {
    /// Capability identifier
    pub capability: String,

    /// Description of what can be automated
    pub description: String,

    /// Prerequisites for automation
    pub prerequisites: Vec<String>,

    /// Expected automation confidence
    pub confidence_threshold: f64,
}

impl AutomationCapability {
    /// Create a new automation capability
    pub fn new(
        capability: impl Into<String>,
        description: impl Into<String>,
        confidence_threshold: f64,
    ) -> Self {
        Self {
            capability: capability.into(),
            description: description.into(),
            prerequisites: Vec::new(),
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// Add a prerequisite
    #[must_use]
    pub fn with_prerequisite(mut self, prerequisite: impl Into<String>) -> Self {
        self.prerequisites.push(prerequisite.into());
        self
    }
}

/// Quality metrics for AI evaluation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityMetrics {
    /// Accuracy score (0.0-1.0)
    pub accuracy: Option<f64>,

    /// Completeness score (0.0-1.0)
    pub completeness: Option<f64>,

    /// Relevance score (0.0-1.0)
    pub relevance: Option<f64>,

    /// Timeliness score (0.0-1.0)
    pub timeliness: Option<f64>,

    /// Overall quality score (0.0-1.0)
    pub overall_quality: Option<f64>,
}

impl QualityMetrics {
    /// Calculate overall quality from individual metrics
    pub fn calculate_overall(&mut self) {
        let metrics = [self.accuracy, self.completeness, self.relevance, self.timeliness];

        let valid_metrics: Vec<f64> = metrics.into_iter().flatten().collect();

        if !valid_metrics.is_empty() {
            let sum: f64 = valid_metrics.iter().sum();
            // Allow cast precision loss for quality calculation - acceptable trade-off
            #[expect(
                clippy::cast_precision_loss,
                reason = "intentional pattern; clippy false positive for this API"
            )]
            {
                self.overall_quality = Some(sum / valid_metrics.len() as f64);
            }
        }
    }

    /// Set accuracy score
    #[must_use]
    pub fn with_accuracy(mut self, accuracy: f64) -> Self {
        self.accuracy = Some(accuracy.clamp(0.0, 1.0));
        self.calculate_overall();
        self
    }

    /// Set completeness score
    #[must_use]
    pub fn with_completeness(mut self, completeness: f64) -> Self {
        self.completeness = Some(completeness.clamp(0.0, 1.0));
        self.calculate_overall();
        self
    }

    /// Set relevance score
    #[must_use]
    pub fn with_relevance(mut self, relevance: f64) -> Self {
        self.relevance = Some(relevance.clamp(0.0, 1.0));
        self.calculate_overall();
        self
    }

    /// Set timeliness score
    #[must_use]
    pub fn with_timeliness(mut self, timeliness: f64) -> Self {
        self.timeliness = Some(timeliness.clamp(0.0, 1.0));
        self.calculate_overall();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let j = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&j).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), j);
    }

    #[test]
    fn ai_response_metadata_default() {
        let m = AIResponseMetadata::default();
        assert!(m.automation_capabilities.is_empty());
        assert!(m.custom_fields.is_empty());
        assert_eq!(m.decision_context.risk_level, RiskLevel::Low);
    }

    #[test]
    fn ai_response_metadata_builders_and_roundtrip() {
        let cap = AutomationCapability::new("deploy", "roll out", 1.5).with_prerequisite("git");
        assert_eq!(cap.confidence_threshold, 1.0);

        let m = AIResponseMetadata::default()
            .with_automation_capability(cap)
            .with_custom_field("k", json!("v"));
        assert_eq!(m.automation_capabilities.len(), 1);
        assert_eq!(m.custom_fields.get("k"), Some(&json!("v")));
        assert_json_roundtrip(&m);
    }

    #[test]
    fn decision_context_default_and_roundtrip() {
        let d = DecisionContext::default();
        assert!(d.influencing_factors.is_empty());
        assert_json_roundtrip(&d);
    }

    #[test]
    fn risk_level_roundtrip() {
        for level in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical] {
            assert_json_roundtrip(&level);
        }
    }

    #[test]
    fn automation_capability_roundtrip() {
        let a = AutomationCapability::new("x", "y", 0.5);
        assert_json_roundtrip(&a);
    }

    #[test]
    fn quality_metrics_calculate_overall_averages_present_fields() {
        let m = QualityMetrics::default().with_accuracy(0.2).with_completeness(0.4);
        assert!((m.overall_quality.unwrap() - 0.3).abs() < 1e-9);
    }

    #[test]
    fn quality_metrics_calculate_overall_empty_leaves_overall_none() {
        let mut m = QualityMetrics::default();
        m.calculate_overall();
        assert!(m.overall_quality.is_none());
    }

    #[test]
    fn quality_metrics_setters_clamp_and_roundtrip() {
        let m = QualityMetrics::default().with_relevance(2.0).with_timeliness(-1.0);
        assert_eq!(m.relevance, Some(1.0));
        assert_eq!(m.timeliness, Some(0.0));
        assert_json_roundtrip(&m);
    }
}
