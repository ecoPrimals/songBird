// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Data types for primal availability reports and alternative suggestions.

use serde::{Deserialize, Serialize};

/// Availability report for a graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilityReport {
    /// Node IDs that are available (healthy primals)
    pub available: Vec<String>,

    /// Node IDs that are unavailable (no primal registered)
    pub unavailable: Vec<String>,

    /// Node IDs with unhealthy primals
    pub unhealthy: Vec<String>,

    /// Node IDs with degraded primals
    pub degraded: Vec<String>,

    /// Detailed availability information for each node
    pub details: std::collections::HashMap<String, NodeAvailability>,

    /// Summary statistics
    pub summary: AvailabilitySummary,
}

/// Summary statistics for availability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilitySummary {
    /// Total number of nodes in the graph
    pub total_nodes: usize,

    /// Number of available nodes
    pub available_nodes: usize,

    /// Availability percentage (0-100)
    pub availability_percent: f64,
}

/// Availability status for a single node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeAvailability {
    /// Availability status
    pub status: NodeAvailabilityStatus,

    /// Primal name (if available)
    pub primal: Option<String>,

    /// Service ID (if available)
    pub service_id: Option<String>,

    /// Endpoint (if available)
    pub endpoint: Option<String>,

    /// Protocol (if available)
    pub protocol: Option<String>,

    /// Health status (if available)
    pub health_status: Option<String>,

    /// Last seen timestamp (if available)
    pub last_seen: Option<String>,

    /// Required capability (if unavailable)
    pub required_capability: Option<String>,

    /// Reason for unavailability (if unavailable)
    pub reason: Option<String>,

    /// Suggested action (if unavailable)
    pub suggested_action: Option<String>,
}

/// Node availability status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeAvailabilityStatus {
    /// Primal available and healthy
    Available,
    /// No primal registered with required capability
    Unavailable,
    /// Primal registered but unhealthy (down/unknown)
    Unhealthy,
    /// Primal registered but degraded
    Degraded,
}

/// Alternative primal suggestions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativeSuggestions {
    /// List of alternative primals, ranked by compatibility
    pub alternatives: Vec<AlternativePrimal>,

    /// Recommended alternative (best match)
    pub recommendation: Option<AlternativeRecommendation>,

    /// Reason if no alternatives available
    pub unavailable_reason: Option<String>,
}

/// An alternative primal suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativePrimal {
    /// Rank (1 = best, 2 = second best, etc.)
    pub rank: usize,

    /// Service ID
    pub service_id: String,

    /// Primal name
    pub primal_name: String,

    /// Endpoint
    pub endpoint: String,

    /// Protocol
    pub protocol: String,

    /// Health status
    pub health_status: String,

    /// Last seen timestamp
    pub last_seen: String,

    /// Reason for suggestion
    pub reason: String,

    /// Compatibility score (0-100)
    pub compatibility_score: u32,
}

/// Recommended alternative
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativeRecommendation {
    /// Service ID of recommended primal
    pub service_id: String,

    /// Reason for recommendation
    pub reason: String,
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::{
        AlternativePrimal, AlternativeRecommendation, AlternativeSuggestions, AvailabilityReport,
        AvailabilitySummary, NodeAvailability, NodeAvailabilityStatus,
    };
    use std::collections::HashMap;

    #[test]
    fn node_availability_status_serde_roundtrip() {
        for s in [
            NodeAvailabilityStatus::Available,
            NodeAvailabilityStatus::Unavailable,
            NodeAvailabilityStatus::Unhealthy,
            NodeAvailabilityStatus::Degraded,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let back: NodeAvailabilityStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn availability_summary_roundtrip() {
        let sum = AvailabilitySummary {
            total_nodes: 4,
            available_nodes: 3,
            availability_percent: 75.0,
        };
        let j = serde_json::to_string(&sum).unwrap();
        let back: AvailabilitySummary = serde_json::from_str(&j).unwrap();
        assert_eq!(sum, back);
    }

    #[test]
    fn availability_report_roundtrip() {
        let mut details = HashMap::new();
        details.insert(
            "n1".to_string(),
            NodeAvailability {
                status: NodeAvailabilityStatus::Available,
                primal: Some("p".to_string()),
                service_id: Some("s".to_string()),
                endpoint: None,
                protocol: None,
                health_status: None,
                last_seen: None,
                required_capability: None,
                reason: None,
                suggested_action: None,
            },
        );
        let report = AvailabilityReport {
            available: vec!["n1".to_string()],
            unavailable: vec![],
            unhealthy: vec![],
            degraded: vec![],
            details,
            summary: AvailabilitySummary {
                total_nodes: 1,
                available_nodes: 1,
                availability_percent: 100.0,
            },
        };
        let j = serde_json::to_string(&report).unwrap();
        let back: AvailabilityReport = serde_json::from_str(&j).unwrap();
        assert_eq!(report, back);
    }

    #[test]
    fn alternative_suggestions_roundtrip() {
        let sug = AlternativeSuggestions {
            alternatives: vec![AlternativePrimal {
                rank: 1,
                service_id: "sid".to_string(),
                primal_name: "pn".to_string(),
                endpoint: "e".to_string(),
                protocol: "http".to_string(),
                health_status: "ok".to_string(),
                last_seen: "t".to_string(),
                reason: "nearby".to_string(),
                compatibility_score: 90,
            }],
            recommendation: Some(AlternativeRecommendation {
                service_id: "sid".to_string(),
                reason: "best".to_string(),
            }),
            unavailable_reason: None,
        };
        let j = serde_json::to_string(&sug).unwrap();
        let back: AlternativeSuggestions = serde_json::from_str(&j).unwrap();
        assert_eq!(sug, back);
    }

    #[test]
    fn alternative_suggestions_with_unavailable_reason() {
        let sug = AlternativeSuggestions {
            alternatives: vec![],
            recommendation: None,
            unavailable_reason: Some("none found".to_string()),
        };
        let j = serde_json::to_string(&sug).unwrap();
        let back: AlternativeSuggestions = serde_json::from_str(&j).unwrap();
        assert_eq!(sug, back);
    }
}
