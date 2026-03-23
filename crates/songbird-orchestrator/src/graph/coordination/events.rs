// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pattern detection and per-pattern validation orchestration.

use anyhow::Result;
use tracing::{debug, info};

use super::CoordinationValidator;
use super::state::{CoordinationIssue, CoordinationPattern, CoordinationValidationResult};
use crate::graph::types::Graph;

impl CoordinationValidator {
    /// Validate coordination pattern for a graph
    ///
    /// Detects the coordination pattern from graph structure and validates
    /// that it can be executed given available primals.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
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
    pub(crate) fn detect_pattern(&self, graph: &Graph) -> Result<CoordinationPattern> {
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
    pub(crate) async fn validate_sequential(
        &self,
        graph: &Graph,
    ) -> Result<CoordinationValidationResult> {
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
            result.add_issue(CoordinationIssue::error(format!("Data flow error: {e}")));
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
    pub(crate) async fn validate_parallel(
        &self,
        graph: &Graph,
    ) -> Result<CoordinationValidationResult> {
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
                "Potential deadlock detected: {deadlock}"
            )));
        }

        Ok(result)
    }

    /// Validate pipeline pattern
    ///
    /// Pipeline: Stage1 → Stage2 → Stage3, with data streaming between stages
    /// Requirements: Stages can overlap (stage N+1 starts before stage N completes)
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
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
                "Potential bottleneck at stage: {bottleneck}"
            )));
        }

        Ok(result)
    }

    /// Validate mapreduce pattern
    ///
    /// `MapReduce`: Input → (Map1, Map2, ..., `MapN`) → (Reduce) → Output
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
            result.add_issue(CoordinationIssue::error(format!("Map partitioning error: {e}")));
        }

        // Check reduce aggregation
        if let Err(e) = self.validate_reduce_aggregation(graph, &reduce_nodes) {
            result.add_issue(CoordinationIssue::error(format!("Reduce aggregation error: {e}")));
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
                        "Nested hybrid pattern in subgraph {i}"
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
}
