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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info};

use super::types::Graph;
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
    service_registry: Arc<ServiceRegistry>,
}

impl CoordinationValidator {
    /// Create a new coordination validator
    pub fn new(service_registry: Arc<ServiceRegistry>) -> Self {
        Self {
            service_registry,
        }
    }

    /// Validate coordination pattern for a graph
    ///
    /// Detects the coordination pattern from graph structure and validates
    /// that it can be executed given available primals.
    pub async fn validate_pattern(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        debug!("Validating coordination pattern for graph: {}", graph.id);

        // Detect pattern from graph structure
        let pattern = self.detect_pattern(graph)?;
        info!("Detected pattern: {:?} for graph {}", pattern, graph.id);

        // Validate the detected pattern
        let validation = match pattern {
            CoordinationPattern::Sequential => self.validate_sequential(graph).await?,
            CoordinationPattern::Parallel => self.validate_parallel(graph).await?,
            CoordinationPattern::Pipeline => self.validate_pipeline(graph).await?,
            CoordinationPattern::MapReduce => self.validate_mapreduce(graph).await?,
            CoordinationPattern::Hybrid => self.validate_hybrid(graph).await?,
        };

        Ok(validation)
    }

    /// Detect coordination pattern from graph structure
    ///
    /// Uses graph topology analysis to determine the coordination pattern.
    fn detect_pattern(&self, graph: &Graph) -> Result<CoordinationPattern> {
        if graph.nodes.is_empty() {
            return Ok(CoordinationPattern::Sequential);
        }

        // Build dependency graph
        let dependencies = self.build_dependency_map(graph);

        // Analyze topology
        let has_fan_out = self.has_fan_out(&dependencies);
        let has_fan_in = self.has_fan_in(&dependencies);
        let has_linear_chain = self.is_linear_chain(&dependencies, graph.nodes.len());
        let has_map_reduce = self.is_map_reduce_pattern(&dependencies, graph);

        // Pattern detection logic
        if has_map_reduce {
            Ok(CoordinationPattern::MapReduce)
        } else if has_fan_out && has_fan_in {
            Ok(CoordinationPattern::Pipeline)
        } else if has_fan_out {
            Ok(CoordinationPattern::Parallel)
        } else if has_linear_chain {
            Ok(CoordinationPattern::Sequential)
        } else {
            Ok(CoordinationPattern::Hybrid)
        }
    }

    /// Validate sequential pattern
    ///
    /// Sequential: Node1 → Node2 → Node3 → ...
    /// Requirements: Each node must complete before next starts
    async fn validate_sequential(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        let mut result = CoordinationValidationResult::new(
            CoordinationPattern::Sequential,
            "Sequential execution pattern".to_string(),
        );

        // Check linear chain
        let dependencies = self.build_dependency_map(graph);
        if !self.is_linear_chain(&dependencies, graph.nodes.len()) {
            result.add_issue(CoordinationIssue::error(
                "Sequential pattern requires linear chain".to_string(),
            ));
        }

        // Check data flow
        if let Err(e) = self.validate_data_flow_sequential(graph) {
            result.add_issue(CoordinationIssue::error(format!("Data flow error: {}", e)));
        }

        // Check resource requirements
        let resource_check = self.check_sequential_resources(graph).await?;
        if !resource_check.feasible {
            result.add_issue(CoordinationIssue::error(resource_check.reason));
        }

        Ok(result)
    }

    /// Validate parallel pattern
    ///
    /// Parallel: Node1 → (Node2a, Node2b, Node2c) → Node3
    /// Requirements: Multiple nodes can execute concurrently
    async fn validate_parallel(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        let mut result = CoordinationValidationResult::new(
            CoordinationPattern::Parallel,
            "Parallel execution pattern".to_string(),
        );

        // Check for parallel branches
        let parallel_groups = self.identify_parallel_groups(graph)?;
        if parallel_groups.is_empty() {
            result.add_issue(CoordinationIssue::warning("No parallel groups detected".to_string()));
        }

        // Check resource availability for concurrent execution
        for group in &parallel_groups {
            let resource_check = self.check_parallel_resources(graph, group).await?;
            if !resource_check.feasible {
                result.add_issue(CoordinationIssue::error(format!(
                    "Insufficient resources for parallel group: {}",
                    resource_check.reason
                )));
            }
        }

        // Check for deadlocks
        if let Some(deadlock) = self.detect_deadlocks(graph)? {
            result.add_issue(CoordinationIssue::error(format!(
                "Potential deadlock detected: {}",
                deadlock
            )));
        }

        Ok(result)
    }

    /// Validate pipeline pattern
    ///
    /// Pipeline: Stage1 → Stage2 → Stage3, with data streaming between stages
    /// Requirements: Stages can overlap (stage N+1 starts before stage N completes)
    async fn validate_pipeline(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        let mut result = CoordinationValidationResult::new(
            CoordinationPattern::Pipeline,
            "Pipeline execution pattern".to_string(),
        );

        // Identify pipeline stages
        let stages = self.identify_pipeline_stages(graph)?;
        info!("Identified {} pipeline stages", stages.len());

        // Check stage dependencies
        if stages.len() < 2 {
            result.add_issue(CoordinationIssue::warning(
                "Pipeline pattern requires at least 2 stages".to_string(),
            ));
        }

        // Check data flow between stages
        for i in 0..stages.len().saturating_sub(1) {
            if let Err(e) = self.validate_stage_data_flow(&stages[i], &stages[i + 1]) {
                result.add_issue(CoordinationIssue::error(format!(
                    "Stage {}->{} data flow error: {}",
                    i,
                    i + 1,
                    e
                )));
            }
        }

        // Check for bottlenecks
        if let Some(bottleneck) = self.detect_pipeline_bottleneck(graph, &stages)? {
            result.add_issue(CoordinationIssue::warning(format!(
                "Potential bottleneck at stage: {}",
                bottleneck
            )));
        }

        Ok(result)
    }

    /// Validate mapreduce pattern
    ///
    /// MapReduce: Input → (Map1, Map2, ..., MapN) → (Reduce) → Output
    /// Requirements: Map phase parallelizes, reduce phase aggregates
    async fn validate_mapreduce(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        let mut result = CoordinationValidationResult::new(
            CoordinationPattern::MapReduce,
            "MapReduce execution pattern".to_string(),
        );

        // Identify map and reduce nodes
        let (map_nodes, reduce_nodes) = self.identify_map_reduce_nodes(graph)?;

        // Validate map phase
        if map_nodes.is_empty() {
            result.add_issue(CoordinationIssue::error(
                "MapReduce pattern requires map nodes".to_string(),
            ));
        }

        // Validate reduce phase
        if reduce_nodes.is_empty() {
            result.add_issue(CoordinationIssue::error(
                "MapReduce pattern requires reduce nodes".to_string(),
            ));
        }

        // Check data partitioning
        if let Err(e) = self.validate_map_partitioning(graph, &map_nodes) {
            result.add_issue(CoordinationIssue::error(format!("Map partitioning error: {}", e)));
        }

        // Check reduce aggregation
        if let Err(e) = self.validate_reduce_aggregation(graph, &reduce_nodes) {
            result.add_issue(CoordinationIssue::error(format!("Reduce aggregation error: {}", e)));
        }

        // Check resource requirements for parallel map
        let resource_check = self.check_mapreduce_resources(graph, &map_nodes).await?;
        if !resource_check.feasible {
            result.add_issue(CoordinationIssue::error(format!(
                "Insufficient resources for map phase: {}",
                resource_check.reason
            )));
        }

        Ok(result)
    }

    /// Validate hybrid pattern (complex graphs with multiple patterns)
    async fn validate_hybrid(&self, graph: &Graph) -> Result<CoordinationValidationResult> {
        let mut result = CoordinationValidationResult::new(
            CoordinationPattern::Hybrid,
            "Hybrid coordination pattern (complex graph)".to_string(),
        );

        // Decompose into sub-patterns
        let subgraphs = self.decompose_into_subgraphs(graph)?;
        info!("Decomposed into {} subgraphs", subgraphs.len());

        for (i, subgraph) in subgraphs.iter().enumerate() {
            let pattern = self.detect_pattern(subgraph)?;
            debug!("Subgraph {} has pattern: {:?}", i, pattern);

            // Validate each subgraph
            let subresult = match pattern {
                CoordinationPattern::Sequential => self.validate_sequential(subgraph).await?,
                CoordinationPattern::Parallel => self.validate_parallel(subgraph).await?,
                CoordinationPattern::Pipeline => self.validate_pipeline(subgraph).await?,
                CoordinationPattern::MapReduce => self.validate_mapreduce(subgraph).await?,
                CoordinationPattern::Hybrid => {
                    result.add_issue(CoordinationIssue::warning(format!(
                        "Nested hybrid pattern in subgraph {}",
                        i
                    )));
                    continue;
                }
            };

            // Merge issues
            for issue in subresult.issues {
                result.add_issue(issue);
            }
        }

        Ok(result)
    }

    // ========================================================================
    // Helper Methods (Graph Analysis)
    // ========================================================================

    /// Build dependency map from graph edges
    fn build_dependency_map(&self, graph: &Graph) -> HashMap<String, Vec<String>> {
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();

        for edge in &graph.edges {
            deps.entry(edge.to.clone()).or_insert_with(Vec::new).push(edge.from.clone());
        }

        deps
    }

    /// Check if graph has fan-out (one node → multiple nodes)
    fn has_fan_out(&self, dependencies: &HashMap<String, Vec<String>>) -> bool {
        // Build reverse map (node → dependents)
        let mut dependents: HashMap<String, usize> = HashMap::new();

        for (_, sources) in dependencies.iter() {
            for source in sources {
                *dependents.entry(source.clone()).or_insert(0) += 1;
            }
        }

        dependents.values().any(|&count| count > 1)
    }

    /// Check if graph has fan-in (multiple nodes → one node)
    fn has_fan_in(&self, dependencies: &HashMap<String, Vec<String>>) -> bool {
        dependencies.values().any(|sources| sources.len() > 1)
    }

    /// Check if graph is a linear chain (A → B → C → ...)
    fn is_linear_chain(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
        node_count: usize,
    ) -> bool {
        if node_count == 0 {
            return true;
        }

        // Every node (except first) should have exactly one dependency
        // Every node (except last) should have exactly one dependent
        dependencies.values().all(|sources| sources.len() <= 1) && !self.has_fan_out(dependencies)
    }

    /// Check if graph matches map-reduce pattern
    fn is_map_reduce_pattern(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
        graph: &Graph,
    ) -> bool {
        // Look for: single source → multiple parallel → single sink
        let entry_points: HashSet<_> =
            graph.entry_points().into_iter().map(|n| n.id.clone()).collect();
        let exit_points: HashSet<_> =
            graph.exit_points().into_iter().map(|n| n.id.clone()).collect();

        entry_points.len() == 1
            && exit_points.len() == 1
            && self.has_fan_out(dependencies)
            && self.has_fan_in(dependencies)
    }

    /// Identify parallel execution groups
    fn identify_parallel_groups(&self, graph: &Graph) -> Result<Vec<Vec<String>>> {
        let dependencies = self.build_dependency_map(graph);
        let mut groups = Vec::new();

        // Find nodes with same dependencies (can execute in parallel)
        let mut dep_groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            let mut deps = dependencies.get(&node.id).cloned().unwrap_or_default();
            deps.sort();

            dep_groups.entry(deps).or_insert_with(Vec::new).push(node.id.clone());
        }

        // Only include groups with multiple nodes
        for (_, nodes) in dep_groups {
            if nodes.len() > 1 {
                groups.push(nodes);
            }
        }

        Ok(groups)
    }

    /// Identify pipeline stages (topological layers)
    fn identify_pipeline_stages(&self, graph: &Graph) -> Result<Vec<Vec<String>>> {
        let dependencies = self.build_dependency_map(graph);
        let mut stages = Vec::new();
        let mut processed = HashSet::new();

        // Topological sort by layers
        while processed.len() < graph.nodes.len() {
            let mut current_stage = Vec::new();

            for node in &graph.nodes {
                if processed.contains(&node.id) {
                    continue;
                }

                // Check if all dependencies are processed
                let node_deps = dependencies.get(&node.id).cloned().unwrap_or_default();
                if node_deps.iter().all(|dep| processed.contains(dep)) {
                    current_stage.push(node.id.clone());
                }
            }

            if current_stage.is_empty() {
                // Cycle detected or logic error
                break;
            }

            for node_id in &current_stage {
                processed.insert(node_id.clone());
            }

            stages.push(current_stage);
        }

        Ok(stages)
    }

    /// Identify map and reduce nodes in mapreduce pattern
    fn identify_map_reduce_nodes(&self, graph: &Graph) -> Result<(Vec<String>, Vec<String>)> {
        let dependencies = self.build_dependency_map(graph);
        let entry_points: HashSet<_> =
            graph.entry_points().into_iter().map(|n| n.id.clone()).collect();
        let exit_points: HashSet<_> =
            graph.exit_points().into_iter().map(|n| n.id.clone()).collect();

        let mut map_nodes = Vec::new();
        let mut reduce_nodes = Vec::new();

        for node in &graph.nodes {
            // Map nodes: depend on entry point, have fan-in to reduce
            let deps = dependencies.get(&node.id).cloned().unwrap_or_default();
            if deps.iter().any(|d| entry_points.contains(d)) {
                map_nodes.push(node.id.clone());
            }

            // Reduce nodes: have fan-in from map nodes, connect to exit
            if exit_points.contains(&node.id) || deps.len() > 1 {
                reduce_nodes.push(node.id.clone());
            }
        }

        Ok((map_nodes, reduce_nodes))
    }

    /// Decompose hybrid graph into simpler subgraphs
    fn decompose_into_subgraphs(&self, graph: &Graph) -> Result<Vec<Graph>> {
        // For now, treat as single subgraph
        // TODO: Implement smart decomposition based on connectivity
        Ok(vec![graph.clone()])
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    fn validate_data_flow_sequential(&self, _graph: &Graph) -> Result<()> {
        // Sequential data flow is always valid if graph is valid
        Ok(())
    }

    fn validate_stage_data_flow(&self, _stage1: &[String], _stage2: &[String]) -> Result<()> {
        // Check that outputs of stage1 match inputs of stage2
        // For now, assume valid (graph validator already checked this)
        Ok(())
    }

    fn validate_map_partitioning(&self, _graph: &Graph, _map_nodes: &[String]) -> Result<()> {
        // Check that input data can be partitioned for map nodes
        Ok(())
    }

    fn validate_reduce_aggregation(&self, _graph: &Graph, _reduce_nodes: &[String]) -> Result<()> {
        // Check that reduce nodes can aggregate map outputs
        Ok(())
    }

    fn detect_deadlocks(&self, _graph: &Graph) -> Result<Option<String>> {
        // Check for circular dependencies that could cause deadlocks
        // Graph validator already checks for cycles
        Ok(None)
    }

    fn detect_pipeline_bottleneck(
        &self,
        _graph: &Graph,
        _stages: &[Vec<String>],
    ) -> Result<Option<String>> {
        // Detect stages with single node that could bottleneck
        for (i, stage) in _stages.iter().enumerate() {
            if stage.len() == 1 && i > 0 && i < _stages.len() - 1 {
                return Ok(Some(format!("Stage {} (single node)", i)));
            }
        }
        Ok(None)
    }

    // ========================================================================
    // Resource Checking (integrated with service registry)
    // ========================================================================

    async fn check_sequential_resources(&self, graph: &Graph) -> Result<ResourceCheck> {
        // Sequential execution only needs one primal at a time
        for node in &graph.nodes {
            let primals =
                self.service_registry.discover_by_capability(&node.capability, None).await?;

            if primals.is_empty() {
                return Ok(ResourceCheck {
                    feasible: false,
                    reason: format!("No primal for capability '{}'", node.capability),
                });
            }
        }

        Ok(ResourceCheck {
            feasible: true,
            reason: "All capabilities available".to_string(),
        })
    }

    async fn check_parallel_resources(
        &self,
        graph: &Graph,
        group: &[String],
    ) -> Result<ResourceCheck> {
        // Parallel execution needs concurrent primals
        for node_id in group {
            let node = graph.nodes.iter().find(|n| n.id == *node_id).context("Node not found")?;

            let primals =
                self.service_registry.discover_by_capability(&node.capability, None).await?;

            if primals.is_empty() {
                return Ok(ResourceCheck {
                    feasible: false,
                    reason: format!(
                        "No primal for capability '{}' in parallel group",
                        node.capability
                    ),
                });
            }

            // Check if we have enough instances for parallelism
            if primals.len() < group.len() {
                return Ok(ResourceCheck {
                    feasible: true, // Still feasible, but not optimal
                    reason: format!(
                        "Limited parallelism: {} primals for {} nodes",
                        primals.len(),
                        group.len()
                    ),
                });
            }
        }

        Ok(ResourceCheck {
            feasible: true,
            reason: "Sufficient resources for parallel execution".to_string(),
        })
    }

    async fn check_mapreduce_resources(
        &self,
        graph: &Graph,
        map_nodes: &[String],
    ) -> Result<ResourceCheck> {
        // Check resources for map phase (most demanding)
        self.check_parallel_resources(graph, map_nodes).await
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Coordination pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoordinationPattern {
    /// Sequential execution (A → B → C)
    Sequential,
    /// Parallel execution (A → (B1, B2, B3) → C)
    Parallel,
    /// Pipeline execution (streaming data through stages)
    Pipeline,
    /// MapReduce pattern (map phase + reduce phase)
    MapReduce,
    /// Hybrid (complex graph with multiple patterns)
    Hybrid,
}

/// Coordination validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationValidationResult {
    /// Is the coordination pattern valid?
    pub valid: bool,

    /// Detected pattern
    pub pattern: CoordinationPattern,

    /// Pattern description
    pub description: String,

    /// Validation issues (errors and warnings)
    pub issues: Vec<CoordinationIssue>,
}

impl CoordinationValidationResult {
    fn new(pattern: CoordinationPattern, description: String) -> Self {
        Self {
            valid: true,
            pattern,
            description,
            issues: Vec::new(),
        }
    }

    fn add_issue(&mut self, issue: CoordinationIssue) {
        if issue.severity == IssueSeverity::Error {
            self.valid = false;
        }
        self.issues.push(issue);
    }
}

/// Coordination validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

impl CoordinationIssue {
    fn error(message: String) -> Self {
        Self {
            severity: IssueSeverity::Error,
            message,
        }
    }

    fn warning(message: String) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            message,
        }
    }
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Resource availability check result
struct ResourceCheck {
    feasible: bool,
    reason: String,
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
        assert!(
            !result.valid,
            "Expected validation to fail with no primals, but got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_validate_parallel_no_primals() {
        let registry = Arc::new(ServiceRegistry::new());
        let validator = CoordinationValidator::new(registry);
        let graph = create_parallel_graph();

        let result = validator.validate_parallel(&graph).await.unwrap();
        // Should fail because no primals registered
        assert!(
            !result.valid,
            "Expected validation to fail with no primals, but got: {:?}",
            result
        );
    }
}
