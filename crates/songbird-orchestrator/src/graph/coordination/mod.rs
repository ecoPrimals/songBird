// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Coordination pattern validation for graph execution
//!
//! This module validates coordination patterns (sequential, parallel, pipeline, mapreduce)
//! and ensures they're feasible given available primals and resource constraints.
//!
//! # Design Principles
//!
//! - **Zero Hardcoding**: Discovers primal capabilities at runtime
//! - **Deep Debt Solutions**: Proper graph algorithms (not naive checks)
//! - **Modern Rust**: Safe, idiomatic, async throughout
//! - **Observable**: Comprehensive logging at all levels
//!
//! ## Submodules
//!
//! - `state`: Pattern and result types
//! - `events`: Pattern detection and validation orchestration
//! - `scheduler`: Topology analysis and resource scheduling checks

mod events;
mod scheduler;
mod state;

pub use state::{
    CoordinationIssue, CoordinationPattern, CoordinationValidationResult, IssueSeverity,
};

use std::sync::Arc;

use crate::ipc::registry::ServiceRegistry;

/// Coordination pattern validator
///
/// Validates that coordination patterns (sequential, parallel, pipeline, mapreduce)
/// can be executed given available primals and system constraints.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use songbird_orchestrator::graph::{CoordinationValidator, Graph, GraphMetadata};
/// use songbird_orchestrator::ipc::registry::ServiceRegistry;
///
/// # async fn example() -> anyhow::Result<()> {
/// let registry = Arc::new(ServiceRegistry::new());
/// let validator = CoordinationValidator::new(registry);
///
/// let graph = Graph::new(
///     "workflow".to_string(),
///     "Data Pipeline".to_string(),
///     vec![],
///     vec![],
///     GraphMetadata::default(),
/// );
///
/// let result = validator.validate_pattern(&graph).await?;
/// println!("Pattern valid: {}", result.valid);
/// # Ok(())
/// # }
/// ```
pub struct CoordinationValidator {
    pub(crate) service_registry: Arc<ServiceRegistry>,
}

impl CoordinationValidator {
    /// Create a new coordination validator
    #[must_use]
    pub const fn new(service_registry: Arc<ServiceRegistry>) -> Self {
        Self {
            service_registry,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::graph::types::{GraphEdge, GraphMetadata, GraphNode};

    fn create_test_node(id: &str, capability: &str) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            primal_name: None,
            capability: capability.to_string(),
            inputs: vec![],
            outputs: vec![],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        }
    }

    fn create_sequential_graph() -> Graph {
        Graph::new(
            "sequential".to_string(),
            "Sequential Test".to_string(),
            vec![
                create_test_node("node1", "capability1"),
                create_test_node("node2", "capability2"),
                create_test_node("node3", "capability3"),
            ],
            vec![
                GraphEdge {
                    from: "node1".to_string(),
                    to: "node2".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "node2".to_string(),
                    to: "node3".to_string(),
                    data_mapping: None,
                },
            ],
            GraphMetadata::default(),
        )
    }

    fn create_parallel_graph() -> Graph {
        Graph::new(
            "parallel".to_string(),
            "Parallel Test".to_string(),
            vec![
                create_test_node("input", "input"),
                create_test_node("parallel1", "compute"),
                create_test_node("parallel2", "compute"),
                create_test_node("parallel3", "compute"),
                create_test_node("output", "output"),
            ],
            vec![
                GraphEdge {
                    from: "input".to_string(),
                    to: "parallel1".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "input".to_string(),
                    to: "parallel2".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "input".to_string(),
                    to: "parallel3".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "parallel1".to_string(),
                    to: "output".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "parallel2".to_string(),
                    to: "output".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "parallel3".to_string(),
                    to: "output".to_string(),
                    data_mapping: None,
                },
            ],
            GraphMetadata::default(),
        )
    }

    #[test]
    fn test_detect_sequential_pattern() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();

        let pattern = validator.detect_pattern(&graph).unwrap();
        assert_eq!(pattern, CoordinationPattern::Sequential);
    }

    #[test]
    fn test_detect_parallel_pattern() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();

        let pattern = validator.detect_pattern(&graph).unwrap();
        // This graph actually matches MapReduce pattern (single input → parallel → single output)
        assert_eq!(pattern, CoordinationPattern::MapReduce);
    }

    #[test]
    fn test_identify_parallel_groups() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();

        let groups = validator.identify_parallel_groups(&graph).unwrap();
        assert!(!groups.is_empty());
        assert!(groups.iter().any(|g| g.len() == 3)); // The 3 parallel compute nodes
    }

    #[test]
    fn test_identify_pipeline_stages() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();

        let stages = validator.identify_pipeline_stages(&graph).unwrap();
        assert_eq!(stages.len(), 3); // 3 sequential stages
        assert_eq!(stages[0].len(), 1); // Each stage has 1 node
        assert_eq!(stages[1].len(), 1);
        assert_eq!(stages[2].len(), 1);
    }

    #[tokio::test]
    async fn test_validate_sequential_no_primals() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();

        let result = validator.validate_sequential(&graph).await.unwrap();
        // Should fail because no primals registered for capabilities
        // Resource check will detect missing primals and add error
        assert!(!result.valid, "Expected validation to fail with no primals, but got: {result:?}");
    }

    #[tokio::test]
    async fn test_validate_parallel_no_primals() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();

        let result = validator.validate_parallel(&graph).await.unwrap();
        // Should fail because no primals registered
        assert!(!result.valid, "Expected validation to fail with no primals, but got: {result:?}");
    }

    // ─── scheduler.rs pure-logic helpers ─────────────────────────────────

    #[test]
    fn build_dependency_map_empty_graph() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph =
            Graph::new("empty".into(), "Empty".into(), vec![], vec![], GraphMetadata::default());
        let deps = validator.build_dependency_map(&graph);
        assert!(deps.is_empty());
    }

    #[test]
    fn build_dependency_map_sequential() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let deps = validator.build_dependency_map(&graph);
        assert_eq!(deps.get("node2").unwrap(), &vec!["node1".to_string()]);
        assert_eq!(deps.get("node3").unwrap(), &vec!["node2".to_string()]);
        assert!(deps.get("node1").is_none());
    }

    #[test]
    fn has_fan_out_sequential_is_false() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(!validator.has_fan_out(&deps));
    }

    #[test]
    fn has_fan_out_parallel_is_true() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(validator.has_fan_out(&deps));
    }

    #[test]
    fn has_fan_in_sequential_is_false() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(!validator.has_fan_in(&deps));
    }

    #[test]
    fn has_fan_in_parallel_is_true() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(validator.has_fan_in(&deps));
    }

    #[test]
    fn is_linear_chain_sequential() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(validator.is_linear_chain(&deps, graph.nodes.len()));
    }

    #[test]
    fn is_linear_chain_parallel_is_false() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(!validator.is_linear_chain(&deps, graph.nodes.len()));
    }

    #[test]
    fn is_linear_chain_empty_graph() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let deps = std::collections::HashMap::new();
        assert!(validator.is_linear_chain(&deps, 0));
    }

    #[test]
    fn is_map_reduce_pattern_parallel_graph() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(validator.is_map_reduce_pattern(&deps, &graph));
    }

    #[test]
    fn is_map_reduce_pattern_sequential_is_false() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let deps = validator.build_dependency_map(&graph);
        assert!(!validator.is_map_reduce_pattern(&deps, &graph));
    }

    #[test]
    fn identify_parallel_groups_sequential_is_empty() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let groups = validator.identify_parallel_groups(&graph).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn identify_pipeline_stages_parallel_graph() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let stages = validator.identify_pipeline_stages(&graph).unwrap();
        assert!(stages.len() >= 2);
        // Input node is in first stage
        assert!(stages[0].contains(&"input".to_string()));
    }

    #[test]
    fn identify_map_reduce_nodes_parallel() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();
        let (map_nodes, reduce_nodes) = validator.identify_map_reduce_nodes(&graph).unwrap();
        assert!(!map_nodes.is_empty());
        assert!(!reduce_nodes.is_empty());
    }

    #[test]
    fn decompose_into_subgraphs_returns_single_graph() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let subgraphs = validator.decompose_into_subgraphs(&graph).unwrap();
        assert_eq!(subgraphs.len(), 1);
        assert_eq!(subgraphs[0].id, graph.id);
    }

    #[test]
    fn detect_pipeline_bottleneck_sequential_inner_node() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let stages = validator.identify_pipeline_stages(&graph).unwrap();
        let bottleneck = validator.detect_pipeline_bottleneck(&graph, &stages).unwrap();
        // Middle stage has single node → bottleneck
        assert!(bottleneck.is_some());
    }

    #[test]
    fn detect_pipeline_bottleneck_empty_stages() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        let bottleneck = validator.detect_pipeline_bottleneck(&graph, &[]).unwrap();
        assert!(bottleneck.is_none());
    }

    #[test]
    fn validate_data_flow_sequential_always_ok() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        assert!(validator.validate_data_flow_sequential(&graph).is_ok());
    }

    #[test]
    fn validate_stage_data_flow_always_ok() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let s1 = vec!["a".into()];
        let s2 = vec!["b".into()];
        assert!(validator.validate_stage_data_flow(&s1, &s2).is_ok());
    }

    #[test]
    fn detect_deadlocks_returns_none() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_sequential_graph();
        assert_eq!(validator.detect_deadlocks(&graph).unwrap(), None);
    }
}
