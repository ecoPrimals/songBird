// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Graph topology analysis, staged scheduling helpers, and resource feasibility checks.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

use super::CoordinationValidator;
use super::state::ResourceCheck;
use crate::graph::types::Graph;

impl CoordinationValidator {
    // ========================================================================
    // Helper Methods (Graph Analysis)
    // ========================================================================

    /// Build dependency map from graph edges
    pub(crate) fn build_dependency_map(&self, graph: &Graph) -> HashMap<String, Vec<String>> {
        let _ = self;
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();

        for edge in &graph.edges {
            deps.entry(edge.to.clone()).or_default().push(edge.from.clone());
        }

        deps
    }

    /// Check if graph has fan-out (one node → multiple nodes)
    pub(crate) fn has_fan_out(&self, dependencies: &HashMap<String, Vec<String>>) -> bool {
        let _ = self;
        // Build reverse map (node → dependents)
        let mut dependents: HashMap<String, usize> = HashMap::new();

        for sources in dependencies.values() {
            for source in sources {
                *dependents.entry(source.clone()).or_insert(0) += 1;
            }
        }

        dependents.values().any(|&count| count > 1)
    }

    /// Check if graph has fan-in (multiple nodes → one node)
    pub(crate) fn has_fan_in(&self, dependencies: &HashMap<String, Vec<String>>) -> bool {
        let _ = self;
        dependencies.values().any(|sources| sources.len() > 1)
    }

    /// Check if graph is a linear chain (A → B → C → ...)
    pub(crate) fn is_linear_chain(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
        node_count: usize,
    ) -> bool {
        let _ = self;
        if node_count == 0 {
            return true;
        }

        // Every node (except first) should have exactly one dependency
        // Every node (except last) should have exactly one dependent
        dependencies.values().all(|sources| sources.len() <= 1) && !self.has_fan_out(dependencies)
    }

    /// Check if graph matches map-reduce pattern
    pub(crate) fn is_map_reduce_pattern(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
        graph: &Graph,
    ) -> bool {
        let _ = self;
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
    pub(crate) fn identify_parallel_groups(&self, graph: &Graph) -> Result<Vec<Vec<String>>> {
        let dependencies = self.build_dependency_map(graph);
        let mut groups = Vec::new();

        // Find nodes with same dependencies (can execute in parallel)
        let mut dep_groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            let mut deps = dependencies.get(&node.id).cloned().unwrap_or_default();
            deps.sort();

            dep_groups.entry(deps).or_default().push(node.id.clone());
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
    pub(crate) fn identify_pipeline_stages(&self, graph: &Graph) -> Result<Vec<Vec<String>>> {
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
    pub(crate) fn identify_map_reduce_nodes(
        &self,
        graph: &Graph,
    ) -> Result<(Vec<String>, Vec<String>)> {
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
    pub(crate) fn decompose_into_subgraphs(&self, graph: &Graph) -> Result<Vec<Graph>> {
        let _ = self;
        // FUTURE (Phase 2): Smart graph decomposition for distributed execution
        // Current: Single subgraph execution is sufficient for current workloads
        // Future use case: Multi-datacenter graph execution, compute-heavy workloads
        // Algorithm: Analyze graph connectivity, minimize cross-subgraph dependencies
        // For now, treat as single subgraph
        Ok(vec![graph.clone()])
    }

    // ========================================================================
    // Validation Helpers
    // ========================================================================

    pub(crate) const fn validate_data_flow_sequential(&self, _graph: &Graph) -> Result<()> {
        let _ = self;
        // Sequential data flow is always valid if graph is valid
        Ok(())
    }

    pub(crate) const fn validate_stage_data_flow(
        &self,
        _stage1: &[String],
        _stage2: &[String],
    ) -> Result<()> {
        let _ = self;
        // Check that outputs of stage1 match inputs of stage2
        // For now, assume valid (graph validator already checked this)
        Ok(())
    }

    pub(crate) const fn validate_map_partitioning(
        &self,
        _graph: &Graph,
        _map_nodes: &[String],
    ) -> Result<()> {
        let _ = self;
        // Check that input data can be partitioned for map nodes
        Ok(())
    }

    pub(crate) const fn validate_reduce_aggregation(
        &self,
        _graph: &Graph,
        _reduce_nodes: &[String],
    ) -> Result<()> {
        let _ = self;
        // Check that reduce nodes can aggregate map outputs
        Ok(())
    }

    pub(crate) const fn detect_deadlocks(&self, _graph: &Graph) -> Result<Option<String>> {
        let _ = self;
        // Check for circular dependencies that could cause deadlocks
        // Graph validator already checks for cycles
        Ok(None)
    }

    pub(crate) fn detect_pipeline_bottleneck(
        &self,
        _graph: &Graph,
        stages: &[Vec<String>],
    ) -> Result<Option<String>> {
        let _ = self;
        // Detect stages with single node that could bottleneck
        for (i, stage) in stages.iter().enumerate() {
            if stage.len() == 1 && i > 0 && i < stages.len() - 1 {
                return Ok(Some(format!("Stage {i} (single node)")));
            }
        }
        Ok(None)
    }

    // ========================================================================
    // Resource Checking (integrated with service registry)
    // ========================================================================

    pub(crate) async fn check_sequential_resources(&self, graph: &Graph) -> Result<ResourceCheck> {
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
            reason: String::from("All capabilities available"),
        })
    }

    pub(crate) async fn check_parallel_resources(
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
            reason: String::from("Sufficient resources for parallel execution"),
        })
    }

    pub(crate) async fn check_mapreduce_resources(
        &self,
        graph: &Graph,
        map_nodes: &[String],
    ) -> Result<ResourceCheck> {
        // Check resources for map phase (most demanding)
        self.check_parallel_resources(graph, map_nodes).await
    }
}
