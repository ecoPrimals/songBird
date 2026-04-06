// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

use super::types::{Graph, ValidationInfo, ValidationIssue, ValidationResult};
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
    #[must_use]
    pub const fn new() -> Self {
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
            adj.entry(edge.from.as_str()).or_default().push(edge.to.as_str());
        }

        // Try DFS from each node
        for node in &graph.nodes {
            if !visited.contains(node.id.as_str())
                && let Some(cycle) =
                    self.dfs_cycle(node.id.as_str(), &adj, &mut visited, &mut rec_stack, &mut path)
            {
                return Some(cycle);
            }
        }

        None
    }

    /// DFS helper for cycle detection
    #[allow(
        clippy::self_only_used_in_recursion,
        reason = "&self needed for method dispatch context"
    )]
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
                    if let Some(cycle_start) = path.iter().position(|n| n == neighbor) {
                        let mut cycle = path[cycle_start..].to_vec();
                        cycle.push(neighbor.to_string());
                        return Some(cycle);
                    }
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
            outputs.insert(
                node.id.as_str(),
                node.outputs.iter().map(std::string::String::as_str).collect(),
            );
        }

        // Check that each node's inputs are satisfied
        for node in &graph.nodes {
            for input in &node.inputs {
                // Find edges that provide this input
                let has_provider = graph.edges.iter().filter(|e| e.to == node.id).any(|e| {
                    if let Some(ref mapping) = e.data_mapping {
                        mapping.values().any(|v| v == input)
                    } else {
                        outputs
                            .get(e.from.as_str())
                            .is_some_and(|outs| outs.contains(input.as_str()))
                    }
                });

                if !has_provider && !graph.entry_points().iter().any(|n| n.id == node.id) {
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
#[path = "validator_tests.rs"]
mod tests;
