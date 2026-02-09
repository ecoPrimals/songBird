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
    AlternativePrimal, AlternativeSuggestions, AvailabilityChecker, AvailabilityReport,
    NodeAvailability, NodeAvailabilityStatus,
};
pub use coordination::{
    CoordinationIssue, CoordinationPattern, CoordinationValidationResult, CoordinationValidator,
};
pub use types::{Graph, GraphEdge, GraphMetadata, GraphNode, ValidationIssue, ValidationResult};
pub use validator::GraphValidator;
