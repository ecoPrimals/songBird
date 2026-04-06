// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

/// Graph validation module for Collaborative Intelligence
///
/// This module provides graph validation, availability checking, and coordination
/// pattern validation for the Collaborative Intelligence system.
///
/// # Architecture
///
/// The graph module is built on top of the service registry (v3.20.0) and follows
/// the same principles:
/// - **Zero hardcoding**: All primal discovery is capability-based
/// - **Modern Rust**: Safe, idiomatic patterns throughout
/// - **Thread-safe**: All operations safe for concurrent use
/// - **Observable**: Comprehensive logging and error messages
///
/// # Modules
///
/// - `types`: Core data structures (Graph, Node, Edge)
/// - `validator`: Graph structure validation
/// - `availability`: Primal availability checking (uses service registry)
/// - `coordination`: Coordination pattern validation
///
/// # Example
///
/// ```rust,ignore
/// use songbird_orchestrator::graph::{Graph, GraphValidator};
///
/// let validator = GraphValidator::new();
/// let graph = Graph::new("my-graph", vec![], vec![]);
/// let result = validator.validate(&graph)?;
///
/// if result.valid {
///     println!("Graph is valid!");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub mod availability;
pub mod coordination;
pub mod types;
pub mod validator;

// Re-export commonly used types
pub use availability::{
    AlternativePrimal, AlternativeRecommendation, AlternativeSuggestions, AvailabilityChecker,
    AvailabilityReport, AvailabilitySummary, NodeAvailability, NodeAvailabilityStatus,
};
pub use coordination::{
    CoordinationIssue, CoordinationPattern, CoordinationValidationResult, CoordinationValidator,
};
pub use types::{Graph, GraphEdge, GraphMetadata, GraphNode, ValidationIssue, ValidationResult};
pub use validator::GraphValidator;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use crate::graph::types::{GraphEdge, GraphMetadata, GraphNode};
    use crate::graph::{Graph, GraphValidator};

    fn sample_node(id: &str) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            primal_name: None,
            capability: "cap".to_string(),
            inputs: vec![],
            outputs: vec![],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        }
    }

    #[test]
    fn graph_new_and_entry_exit_points() {
        let g = Graph::new(
            "g1".to_string(),
            "G".to_string(),
            vec![sample_node("a"), sample_node("b")],
            vec![GraphEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                data_mapping: None,
            }],
            GraphMetadata::default(),
        );
        assert_eq!(g.entry_points().len(), 1);
        assert_eq!(g.exit_points().len(), 1);
        assert!(g.get_node("a").is_some());
    }

    #[test]
    fn graph_validator_new() {
        let v = GraphValidator::new();
        let g = Graph::new(
            "empty".to_string(),
            "E".to_string(),
            vec![],
            vec![],
            GraphMetadata::default(),
        );
        let r = v.validate(&g);
        assert!(r.valid);
    }

    #[test]
    fn empty_graph_metadata_default() {
        let m = GraphMetadata::default();
        assert!(!m.version.is_empty());
    }
}
