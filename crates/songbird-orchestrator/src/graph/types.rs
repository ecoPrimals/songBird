//! Core data types for graph validation
//!
//! This module defines the data structures used for representing and validating
//! execution graphs in the Collaborative Intelligence system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete execution graph
///
/// A graph consists of nodes (representing primal operations) and edges
/// (representing data dependencies between nodes).
///
/// # Design Principles
///
/// - **Zero Hardcoding**: Nodes reference capabilities, not primal names
/// - **Self-Describing**: All metadata included for debugging and visualization
/// - **Validation-Friendly**: Structure designed for easy cycle detection
///
/// # Example
///
/// ```rust
/// use songbird_orchestrator::graph::{Graph, GraphNode, GraphEdge, GraphMetadata};
///
/// let graph = Graph {
///     id: "pipeline-1".to_string(),
///     name: "Data Processing Pipeline".to_string(),
///     nodes: vec![
///         GraphNode {
///             id: "encrypt".to_string(),
///             primal_name: None, // No hardcoding!
///             capability: "encryption".to_string(),
///             inputs: vec!["raw_data".to_string()],
///             outputs: vec!["encrypted_data".to_string()],
///             config: serde_json::json!({}),
///             preferred_protocol: Some("json-rpc".to_string()),
///             timeout_secs: Some(30),
///         },
///     ],
///     edges: vec![],
///     metadata: GraphMetadata {
///         created_by: "user@example.com".to_string(),
///         created_at: "2026-01-11T10:00:00Z".to_string(),
///         description: Some("Encrypts and stores data".to_string()),
///         tags: vec!["encryption".to_string(), "storage".to_string()],
///         version: "1.0".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Graph {
    /// Unique graph identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// List of nodes in the graph
    pub nodes: Vec<GraphNode>,

    /// List of edges (dependencies) between nodes
    pub edges: Vec<GraphEdge>,

    /// Graph metadata
    pub metadata: GraphMetadata,
}

impl Graph {
    /// Create a new graph
    ///
    /// # Example
    ///
    /// ```rust
    /// use songbird_orchestrator::graph::{Graph, GraphNode, GraphMetadata};
    ///
    /// let graph = Graph::new(
    ///     "my-graph".to_string(),
    ///     "My Graph".to_string(),
    ///     vec![],
    ///     vec![],
    ///     GraphMetadata::default(),
    /// );
    /// ```
    pub fn new(
        id: String,
        name: String,
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
        metadata: GraphMetadata,
    ) -> Self {
        Self {
            id,
            name,
            nodes,
            edges,
            metadata,
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Get all entry points (nodes with no incoming edges)
    pub fn entry_points(&self) -> Vec<&GraphNode> {
        let has_incoming: HashMap<&str, bool> =
            self.edges.iter().map(|e| (e.to.as_str(), true)).collect();

        self.nodes.iter().filter(|n| !has_incoming.contains_key(n.id.as_str())).collect()
    }

    /// Get all exit points (nodes with no outgoing edges)
    pub fn exit_points(&self) -> Vec<&GraphNode> {
        let has_outgoing: HashMap<&str, bool> =
            self.edges.iter().map(|e| (e.from.as_str(), true)).collect();

        self.nodes.iter().filter(|n| !has_outgoing.contains_key(n.id.as_str())).collect()
    }
}

/// Represents a single node in the execution graph
///
/// Each node represents an operation performed by a primal. The node specifies
/// what capability is required (e.g., "encryption") but NOT which specific primal
/// should perform it. This allows dynamic discovery and alternative suggestions.
///
/// # Design: Zero Hardcoding
///
/// Notice that `primal_name` is `Option<String>` and used for informational
/// purposes only. The validator and availability checker use only the `capability`
/// field to discover appropriate primals at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    /// Unique node identifier within the graph
    pub id: String,

    /// Primal name (optional - for informational/debugging purposes only)
    ///
    /// This field is NOT used for primal selection. The `capability` field
    /// is used for runtime discovery via the service registry.
    pub primal_name: Option<String>,

    /// Required capability (e.g., "encryption", "storage", "compute")
    ///
    /// This is the ONLY field used for primal discovery. The service registry
    /// is queried for primals with this capability.
    pub capability: String,

    /// Input data keys this node expects
    ///
    /// These must be satisfied by either:
    /// - Graph inputs (for entry nodes)
    /// - Outputs from predecessor nodes (via edges)
    pub inputs: Vec<String>,

    /// Output data keys this node produces
    ///
    /// These can be consumed by successor nodes or returned as graph outputs.
    pub outputs: Vec<String>,

    /// Node-specific configuration
    ///
    /// Arbitrary JSON configuration passed to the primal when executing.
    pub config: serde_json::Value,

    /// Preferred protocol (e.g., "json-rpc", "tarpc")
    ///
    /// If specified, the availability checker will prefer primals using this
    /// protocol. If not specified, any protocol is acceptable.
    pub preferred_protocol: Option<String>,

    /// Timeout in seconds
    ///
    /// Maximum time to wait for this node's execution. If not specified,
    /// a default timeout is used.
    pub timeout_secs: Option<u64>,
}

/// Represents a dependency between two nodes
///
/// An edge indicates that the target node (`to`) depends on the source node (`from`).
/// The edge can optionally specify how to map output data from the source to
/// input data for the target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Data key mapping (optional)
    ///
    /// Maps source node outputs to target node inputs.
    /// If not specified, outputs and inputs are matched by name.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "encrypted_data": "data_to_store"
    /// }
    /// ```
    ///
    /// This maps the "encrypted_data" output from the source node to the
    /// "data_to_store" input of the target node.
    pub data_mapping: Option<HashMap<String, String>>,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphMetadata {
    /// User or system that created the graph
    pub created_by: String,

    /// ISO 8601 timestamp of creation
    pub created_at: String,

    /// Human-readable description
    pub description: Option<String>,

    /// Tags for categorization and search
    pub tags: Vec<String>,

    /// Semantic version (e.g., "1.0", "2.1")
    pub version: String,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            created_by: "unknown".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            description: None,
            tags: vec![],
            version: "1.0".to_string(),
        }
    }
}

/// Result of graph validation
///
/// Contains detailed information about validation success or failure,
/// including any issues found, warnings, and informational messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    /// Whether the graph is valid
    ///
    /// A graph is valid if it has no errors (issues with severity "error").
    /// Warnings and info messages do not affect validity.
    pub valid: bool,

    /// List of validation issues (errors and warnings)
    pub issues: Vec<ValidationIssue>,

    /// List of warnings (non-blocking)
    pub warnings: Vec<String>,

    /// Informational data about the graph
    pub info: Option<ValidationInfo>,
}

impl ValidationResult {
    /// Create a new validation result indicating success
    pub fn valid() -> Self {
        Self {
            valid: true,
            issues: vec![],
            warnings: vec![],
            info: None,
        }
    }

    /// Create a new validation result indicating failure
    pub fn invalid(issues: Vec<ValidationIssue>) -> Self {
        Self {
            valid: false,
            issues,
            warnings: vec![],
            info: None,
        }
    }

    /// Add a warning to the result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add informational data
    pub fn with_info(mut self, info: ValidationInfo) -> Self {
        self.info = Some(info);
        self
    }

    /// Add multiple issues
    pub fn with_issues(mut self, issues: Vec<ValidationIssue>) -> Self {
        // Check for errors before consuming the vector
        let has_errors = issues.iter().any(|i| i.severity == IssueSeverity::Error);
        self.issues.extend(issues);
        // If any issue is an error, mark as invalid
        if has_errors {
            self.valid = false;
        }
        self
    }
}

/// A single validation issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,

    /// Machine-readable error code
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Node IDs involved in the issue (if applicable)
    pub nodes: Vec<String>,
}

impl ValidationIssue {
    /// Create a new error
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: IssueSeverity::Error,
            code: code.into(),
            message: message.into(),
            nodes: vec![],
        }
    }

    /// Create a new warning
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            code: code.into(),
            message: message.into(),
            nodes: vec![],
        }
    }

    /// Add node IDs to the issue
    pub fn with_nodes(mut self, nodes: Vec<String>) -> Self {
        self.nodes = nodes;
        self
    }
}

/// Severity of a validation issue
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Error (blocks deployment)
    Error,
    /// Warning (doesn't block but should be addressed)
    Warning,
}

/// Informational data about a validated graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationInfo {
    /// Number of nodes in the graph
    pub node_count: usize,

    /// Number of edges in the graph
    pub edge_count: usize,

    /// Entry point node IDs (nodes with no incoming edges)
    pub entry_points: Vec<String>,

    /// Exit point node IDs (nodes with no outgoing edges)
    pub exit_points: Vec<String>,

    /// Whether the graph contains cycles
    pub has_cycles: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new(
            "test-graph".to_string(),
            "Test Graph".to_string(),
            vec![],
            vec![],
            GraphMetadata::default(),
        );

        assert_eq!(graph.id, "test-graph");
        assert_eq!(graph.name, "Test Graph");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_entry_points() {
        let node1 = GraphNode {
            id: "node-1".to_string(),
            primal_name: None,
            capability: "encryption".to_string(),
            inputs: vec![],
            outputs: vec!["data".to_string()],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        };

        let node2 = GraphNode {
            id: "node-2".to_string(),
            primal_name: None,
            capability: "storage".to_string(),
            inputs: vec!["data".to_string()],
            outputs: vec![],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        };

        let edge = GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: None,
        };

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node1.clone(), node2],
            vec![edge],
            GraphMetadata::default(),
        );

        let entries = graph.entry_points();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "node-1");
    }

    #[test]
    fn test_exit_points() {
        let node1 = GraphNode {
            id: "node-1".to_string(),
            primal_name: None,
            capability: "encryption".to_string(),
            inputs: vec![],
            outputs: vec!["data".to_string()],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        };

        let node2 = GraphNode {
            id: "node-2".to_string(),
            primal_name: None,
            capability: "storage".to_string(),
            inputs: vec!["data".to_string()],
            outputs: vec![],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        };

        let edge = GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: None,
        };

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![node1, node2.clone()],
            vec![edge],
            GraphMetadata::default(),
        );

        let exits = graph.exit_points();
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].id, "node-2");
    }

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::valid();
        assert!(result.valid);
        assert!(result.issues.is_empty());

        let issue = ValidationIssue::error("TEST_ERROR", "Test error");
        let result = ValidationResult::invalid(vec![issue]);
        assert!(!result.valid);
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::error("CYCLE", "Cycle detected")
            .with_nodes(vec!["node-1".to_string(), "node-2".to_string()]);

        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.code, "CYCLE");
        assert_eq!(issue.nodes.len(), 2);
    }
}
