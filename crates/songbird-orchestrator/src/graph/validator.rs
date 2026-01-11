//! Graph structure validation
//!
//! This module provides validation logic for execution graphs, including:
//! - Cycle detection
//! - Orphan node detection
//! - Dependency validation
//! - Schema validation
//!
//! # Design Principles
//!
//! - **Zero Hardcoding**: No primal names in validation logic
//! - **Clear Error Messages**: Every issue includes actionable information
//! - **Fast**: Optimized algorithms for large graphs
//! - **Safe**: No unsafe code, all operations are memory-safe

use super::types::{Graph, IssueSeverity, ValidationInfo, ValidationIssue, ValidationResult};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

/// Graph validator
///
/// Validates graph structure and schema, detecting issues like cycles,
/// orphan nodes, and broken dependencies.
///
/// # Example
///
/// ```rust
/// use songbird_orchestrator::graph::{Graph, GraphValidator, GraphMetadata};
///
/// let validator = GraphValidator::new();
/// let graph = Graph::new(
///     "test".to_string(),
///     "Test".to_string(),
///     vec![],
///     vec![],
///     GraphMetadata::default(),
/// );
///
/// let result = validator.validate(&graph);
/// assert!(result.valid);
/// ```
pub struct GraphValidator {
    // Configuration options can be added here in the future
}

impl GraphValidator {
    /// Create a new graph validator
    pub fn new() -> Self {
        Self {}
    }

    /// Validate a graph
    ///
    /// Performs comprehensive validation including:
    /// - Structure validation (cycles, orphans)
    /// - Schema validation (required fields)
    /// - Dependency validation (inputs/outputs match)
    ///
    /// Returns a `ValidationResult` with detailed information about any issues found.
    pub fn validate(&self, graph: &Graph) -> ValidationResult {
        debug!("Validating graph: {}", graph.id);

        let mut result = ValidationResult::valid();

        // 1. Validate schema (required fields present)
        if let Err(issues) = self.validate_schema(graph) {
            result = result.with_issues(issues);
        }

        // 2. Validate structure (no cycles, no orphans)
        if let Err(issues) = self.validate_structure(graph) {
            result = result.with_issues(issues);
        }

        // 3. Validate dependencies (inputs match outputs)
        if let Err(issues) = self.validate_dependencies(graph) {
            result = result.with_issues(issues);
        }

        // 4. Add informational data
        let info = self.gather_info(graph);
        result = result.with_info(info);

        // 5. Add warnings for common issues
        result = self.add_warnings(graph, result);

        if result.valid {
            debug!("Graph {} is valid", graph.id);
        } else {
            warn!("Graph {} has {} issues", graph.id, result.issues.len());
        }

        result
    }

    /// Validate graph schema
    fn validate_schema(&self, graph: &Graph) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for duplicate node IDs
        let mut seen_ids = HashSet::new();
        for node in &graph.nodes {
            if !seen_ids.insert(&node.id) {
                issues.push(
                    ValidationIssue::error(
                        "DUPLICATE_NODE_ID",
                        format!("Duplicate node ID: {}", node.id),
                    )
                    .with_nodes(vec![node.id.clone()]),
                );
            }

            // Validate required fields
            if node.capability.is_empty() {
                issues.push(
                    ValidationIssue::error(
                        "MISSING_CAPABILITY",
                        format!("Node '{}' missing required capability field", node.id),
                    )
                    .with_nodes(vec![node.id.clone()]),
                );
            }
        }

        // Check that all edges reference valid nodes
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();
        for edge in &graph.edges {
            if !node_ids.contains(&edge.from) {
                issues.push(
                    ValidationIssue::error(
                        "INVALID_EDGE_SOURCE",
                        format!("Edge references non-existent source node: {}", edge.from),
                    )
                    .with_nodes(vec![edge.from.clone()]),
                );
            }
            if !node_ids.contains(&edge.to) {
                issues.push(
                    ValidationIssue::error(
                        "INVALID_EDGE_TARGET",
                        format!("Edge references non-existent target node: {}", edge.to),
                    )
                    .with_nodes(vec![edge.to.clone()]),
                );
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Validate graph structure (cycles, orphans)
    fn validate_structure(&self, graph: &Graph) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Check for cycles using DFS
        if let Some(cycle) = self.detect_cycle(graph) {
            issues.push(
                ValidationIssue::error(
                    "CYCLE_DETECTED",
                    format!("Cycle detected: {}", cycle.join(" → ")),
                )
                .with_nodes(cycle),
            );
        }

        // Check for orphan nodes (no inputs and no outputs)
        for node in &graph.nodes {
            let has_incoming = graph.edges.iter().any(|e| e.to == node.id);
            let has_outgoing = graph.edges.iter().any(|e| e.from == node.id);
            let is_entry = node.inputs.is_empty();
            let is_exit = node.outputs.is_empty();

            if !has_incoming && !has_outgoing && !is_entry && !is_exit {
                issues.push(
                    ValidationIssue::error(
                        "ORPHAN_NODE",
                        format!(
                            "Node '{}' has no connections and is not an entry/exit point",
                            node.id
                        ),
                    )
                    .with_nodes(vec![node.id.clone()]),
                );
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Detect cycles in the graph using DFS
    fn detect_cycle(&self, graph: &Graph) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &graph.edges {
            adj.entry(edge.from.as_str()).or_insert_with(Vec::new).push(edge.to.as_str());
        }

        // Try DFS from each node
        for node in &graph.nodes {
            if !visited.contains(node.id.as_str()) {
                if let Some(cycle) =
                    self.dfs_cycle(node.id.as_str(), &adj, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    /// DFS helper for cycle detection
    fn dfs_cycle(
        &self,
        node: &str,
        adj: &HashMap<&str, Vec<&str>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.dfs_cycle(neighbor, adj, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle - extract it from the path
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.to_string());
                    return Some(cycle);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// Validate dependencies (inputs match outputs)
    fn validate_dependencies(&self, graph: &Graph) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Build a map of node outputs
        let mut outputs: HashMap<&str, HashSet<&str>> = HashMap::new();
        for node in &graph.nodes {
            outputs.insert(node.id.as_str(), node.outputs.iter().map(|s| s.as_str()).collect());
        }

        // Check that each node's inputs are satisfied
        for node in &graph.nodes {
            for input in &node.inputs {
                // Find edges that provide this input
                let providers: Vec<_> = graph
                    .edges
                    .iter()
                    .filter(|e| e.to == node.id)
                    .filter(|e| {
                        if let Some(ref mapping) = e.data_mapping {
                            mapping.values().any(|v| v == input)
                        } else {
                            outputs
                                .get(e.from.as_str())
                                .map(|outs| outs.contains(input.as_str()))
                                .unwrap_or(false)
                        }
                    })
                    .collect();

                if providers.is_empty() && !graph.entry_points().iter().any(|n| n.id == node.id) {
                    issues.push(
                        ValidationIssue::error(
                            "UNSATISFIED_INPUT",
                            format!("Node '{}' input '{}' has no provider", node.id, input),
                        )
                        .with_nodes(vec![node.id.clone()]),
                    );
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Gather informational data about the graph
    fn gather_info(&self, graph: &Graph) -> ValidationInfo {
        let entry_points = graph.entry_points().iter().map(|n| n.id.clone()).collect();
        let exit_points = graph.exit_points().iter().map(|n| n.id.clone()).collect();
        let has_cycles = self.detect_cycle(graph).is_some();

        ValidationInfo {
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            entry_points,
            exit_points,
            has_cycles,
        }
    }

    /// Add warnings for common issues
    fn add_warnings(&self, graph: &Graph, mut result: ValidationResult) -> ValidationResult {
        // Warn about entry points with inputs
        for node in graph.entry_points() {
            if !node.inputs.is_empty() {
                result = result.with_warning(format!(
                    "Node '{}' is an entry point but has inputs: {} (these will be graph inputs)",
                    node.id,
                    node.inputs.join(", ")
                ));
            }
        }

        // Warn about exit points with no outputs
        for node in graph.exit_points() {
            if node.outputs.is_empty() {
                result = result.with_warning(format!(
                    "Node '{}' is an exit point but produces no outputs",
                    node.id
                ));
            }
        }

        result
    }
}

impl Default for GraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_valid_simple_graph() {
        let validator = GraphValidator::new();
        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![create_test_node("node-1", "encryption")],
            vec![],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(result.valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_detect_cycle() {
        let validator = GraphValidator::new();

        let mut node1 = create_test_node("node-1", "encryption");
        node1.outputs = vec!["data".to_string()];

        let mut node2 = create_test_node("node-2", "storage");
        node2.inputs = vec!["data".to_string()];
        node2.outputs = vec!["result".to_string()];

        let mut node3 = create_test_node("node-3", "compute");
        node3.inputs = vec!["result".to_string()];
        node3.outputs = vec!["data".to_string()]; // Cycles back to node-1

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node1, node2, node3],
            vec![
                GraphEdge {
                    from: "node-1".to_string(),
                    to: "node-2".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "node-2".to_string(),
                    to: "node-3".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "node-3".to_string(),
                    to: "node-1".to_string(),
                    data_mapping: None,
                },
            ],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "CYCLE_DETECTED"));
    }

    #[test]
    fn test_duplicate_node_ids() {
        let validator = GraphValidator::new();
        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![
                create_test_node("node-1", "encryption"),
                create_test_node("node-1", "storage"), // Duplicate ID
            ],
            vec![],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "DUPLICATE_NODE_ID"));
    }

    #[test]
    fn test_invalid_edge_reference() {
        let validator = GraphValidator::new();
        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![create_test_node("node-1", "encryption")],
            vec![GraphEdge {
                from: "node-1".to_string(),
                to: "node-2".to_string(), // Non-existent node
                data_mapping: None,
            }],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "INVALID_EDGE_TARGET"));
    }

    #[test]
    fn test_missing_capability() {
        let validator = GraphValidator::new();
        let mut node = create_test_node("node-1", "");
        node.capability = String::new(); // Empty capability

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node],
            vec![],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "MISSING_CAPABILITY"));
    }

    #[test]
    fn test_orphan_node() {
        let validator = GraphValidator::new();

        let mut node1 = create_test_node("node-1", "encryption");
        node1.outputs = vec!["data".to_string()];

        let mut node2 = create_test_node("node-2", "storage");
        node2.inputs = vec!["data".to_string()];

        // Node 3 is an orphan - has no connections
        let mut node3 = create_test_node("node-3", "compute");
        node3.inputs = vec!["input".to_string()];
        node3.outputs = vec!["output".to_string()];

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node1, node2, node3],
            vec![GraphEdge {
                from: "node-1".to_string(),
                to: "node-2".to_string(),
                data_mapping: None,
            }],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "ORPHAN_NODE"));
        assert!(result.issues.iter().any(|i| i.nodes.contains(&"node-3".to_string())));
    }

    #[test]
    fn test_unsatisfied_input() {
        let validator = GraphValidator::new();

        let mut node1 = create_test_node("node-1", "encryption");
        node1.outputs = vec!["encrypted_data".to_string()];

        let mut node2 = create_test_node("node-2", "storage");
        node2.inputs = vec!["decrypted_data".to_string()]; // Wrong input - not provided by node1

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node1, node2],
            vec![GraphEdge {
                from: "node-1".to_string(),
                to: "node-2".to_string(),
                data_mapping: None,
            }],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.code == "UNSATISFIED_INPUT"));
    }

    #[test]
    fn test_complex_graph_validation() {
        let validator = GraphValidator::new();

        // Create a complex valid graph with 10 nodes
        let mut nodes = Vec::new();
        for i in 1..=10 {
            let mut node = create_test_node(&format!("node-{}", i), "compute");
            if i > 1 {
                node.inputs = vec![format!("data-{}", i - 1)];
            }
            if i < 10 {
                node.outputs = vec![format!("data-{}", i)];
            }
            nodes.push(node);
        }

        let mut edges = Vec::new();
        for i in 1..10 {
            edges.push(GraphEdge {
                from: format!("node-{}", i),
                to: format!("node-{}", i + 1),
                data_mapping: None,
            });
        }

        let graph = Graph::new(
            "complex".to_string(),
            "Complex Graph".to_string(),
            nodes,
            edges,
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(result.valid, "Complex graph should be valid");
        assert!(result.issues.is_empty());
        assert_eq!(result.info.as_ref().unwrap().node_count, 10);
        assert_eq!(result.info.as_ref().unwrap().edge_count, 9);
        assert!(!result.info.as_ref().unwrap().has_cycles);
    }

    #[test]
    fn test_multiple_entry_points() {
        let validator = GraphValidator::new();

        // Two entry points feeding into one exit
        let mut node1 = create_test_node("entry-1", "source");
        node1.outputs = vec!["data-1".to_string()];

        let mut node2 = create_test_node("entry-2", "source");
        node2.outputs = vec!["data-2".to_string()];

        let mut node3 = create_test_node("merge", "compute");
        node3.inputs = vec!["data-1".to_string(), "data-2".to_string()];
        node3.outputs = vec!["merged".to_string()];

        let graph = Graph::new(
            "multi-entry".to_string(),
            "Multiple Entry Points".to_string(),
            vec![node1, node2, node3],
            vec![
                GraphEdge {
                    from: "entry-1".to_string(),
                    to: "merge".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "entry-2".to_string(),
                    to: "merge".to_string(),
                    data_mapping: None,
                },
            ],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(result.valid);
        let info = result.info.unwrap();
        assert_eq!(info.entry_points.len(), 2);
        assert!(info.entry_points.contains(&"entry-1".to_string()));
        assert!(info.entry_points.contains(&"entry-2".to_string()));
    }

    #[test]
    fn test_multiple_exit_points() {
        let validator = GraphValidator::new();

        // One entry splitting into two exits (fan-out)
        let mut node1 = create_test_node("source", "source");
        node1.outputs = vec!["data".to_string()];

        let mut node2 = create_test_node("exit-1", "sink");
        node2.inputs = vec!["data".to_string()];

        let mut node3 = create_test_node("exit-2", "sink");
        node3.inputs = vec!["data".to_string()];

        let graph = Graph::new(
            "multi-exit".to_string(),
            "Multiple Exit Points".to_string(),
            vec![node1, node2, node3],
            vec![
                GraphEdge {
                    from: "source".to_string(),
                    to: "exit-1".to_string(),
                    data_mapping: None,
                },
                GraphEdge {
                    from: "source".to_string(),
                    to: "exit-2".to_string(),
                    data_mapping: None,
                },
            ],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(result.valid);
        let info = result.info.unwrap();
        assert_eq!(info.exit_points.len(), 2);
        assert!(info.exit_points.contains(&"exit-1".to_string()));
        assert!(info.exit_points.contains(&"exit-2".to_string()));
    }

    #[test]
    fn test_data_mapping() {
        let validator = GraphValidator::new();

        let mut node1 = create_test_node("node-1", "encryption");
        node1.outputs = vec!["encrypted_data".to_string()];

        let mut node2 = create_test_node("node-2", "storage");
        node2.inputs = vec!["data_to_store".to_string()];

        // Use data mapping to connect mismatched names
        let mut data_mapping = HashMap::new();
        data_mapping.insert("encrypted_data".to_string(), "data_to_store".to_string());

        let graph = Graph::new(
            "mapping".to_string(),
            "Data Mapping".to_string(),
            vec![node1, node2],
            vec![GraphEdge {
                from: "node-1".to_string(),
                to: "node-2".to_string(),
                data_mapping: Some(data_mapping),
            }],
            GraphMetadata::default(),
        );

        let result = validator.validate(&graph);
        assert!(result.valid, "Graph with proper data mapping should be valid");
    }
}
