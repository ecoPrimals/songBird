// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Task Complexity Analysis
//!
//! Analyzes tasks to determine their complexity level, which informs routing decisions.

use super::types::Task;
use tracing::debug;

/// Task complexity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    /// Lightweight tasks: < 1 CPU, < 512MB, < 10s
    ///
    /// Examples: Health checks, status queries, simple API calls
    ///
    /// Routing strategy: Execute locally or route to peer Songbird
    Lightweight,

    /// Moderate tasks: < 4 CPU, < 4GB, < 5 min
    ///
    /// Examples: Data processing, batch jobs, CSV transformations
    ///
    /// Routing strategy: Prefer peer Songbird, fallback to capability
    Moderate,

    /// Heavy tasks: GPU required, > 4GB, or > 5 min
    ///
    /// Examples: ML training, video processing, large computations
    ///
    /// Routing strategy: Always route to specialized capability
    Heavy,
}

/// Analyzer for determining task complexity
pub struct TaskComplexityAnalyzer;

impl TaskComplexityAnalyzer {
    /// Analyze a task and determine its complexity
    ///
    /// # Classification Rules
    ///
    /// ## Heavy (requires specialized capability)
    /// - GPU required
    /// - > 4 CPU cores
    /// - > 4GB memory
    /// - > 5 minutes estimated duration
    ///
    /// ## Moderate (prefer federation)
    /// - 1-4 CPU cores
    /// - 512MB - 4GB memory
    /// - 10s - 5 min duration
    ///
    /// ## Lightweight (local or peer)
    /// - < 1 CPU core
    /// - < 512MB memory
    /// - < 10 seconds duration
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let task = Task::builder("ml_training").with_gpu().build();
    /// let complexity = TaskComplexityAnalyzer::analyze(&task);
    /// assert_eq!(complexity, TaskComplexity::Heavy);
    /// ```
    pub fn analyze(task: &Task) -> TaskComplexity {
        debug!("Analyzing task complexity for: {}", task.task_type);

        // Check if GPU is required → Heavy
        if let Some(reqs) = &task.resource_requirements {
            if reqs.gpu_required {
                debug!("Task requires GPU → Heavy");
                return TaskComplexity::Heavy;
            }

            // Check CPU threshold → Heavy if > 4 cores
            if let Some(cpu_cores) = reqs.cpu_cores
                && cpu_cores > 4.0
            {
                debug!("Task requires {} CPU cores → Heavy", cpu_cores);
                return TaskComplexity::Heavy;
            }

            // Check memory threshold → Heavy if > 4GB
            if let Some(memory_mb) = reqs.memory_mb
                && memory_mb > 4096
            {
                debug!("Task requires {}MB memory → Heavy", memory_mb);
                return TaskComplexity::Heavy;
            }
        }

        // Check estimated duration → Heavy if > 5 minutes
        if let Some(duration_secs) = task.estimated_duration_secs {
            if duration_secs > 300 {
                debug!("Task estimated duration: {}s → Heavy", duration_secs);
                return TaskComplexity::Heavy;
            }

            // Moderate if 10s - 5 min
            if duration_secs > 10 {
                debug!("Task estimated duration: {}s → Moderate", duration_secs);
                return TaskComplexity::Moderate;
            }
        }

        // Check resource requirements for Moderate threshold
        if let Some(reqs) = &task.resource_requirements {
            // Moderate if 1-4 CPU cores
            if let Some(cpu_cores) = reqs.cpu_cores
                && (1.0..=4.0).contains(&cpu_cores)
            {
                debug!("Task requires {} CPU cores → Moderate", cpu_cores);
                return TaskComplexity::Moderate;
            }

            // Moderate if 512MB - 4GB memory
            if let Some(memory_mb) = reqs.memory_mb
                && (512..=4096).contains(&memory_mb)
            {
                debug!("Task requires {}MB memory → Moderate", memory_mb);
                return TaskComplexity::Moderate;
            }
        }

        // Default to Lightweight
        debug!("Task classified as Lightweight");
        TaskComplexity::Lightweight
    }

    /// Check if a task is heavy (requires specialized capability)
    #[must_use]
    pub fn is_heavy(task: &Task) -> bool {
        matches!(Self::analyze(task), TaskComplexity::Heavy)
    }

    /// Check if a task is lightweight (can be handled locally)
    #[must_use]
    pub fn is_lightweight(task: &Task) -> bool {
        matches!(Self::analyze(task), TaskComplexity::Lightweight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_task_is_heavy() {
        let task = Task::builder("ml_training").with_gpu().build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Heavy);
        assert!(TaskComplexityAnalyzer::is_heavy(&task));
    }

    #[test]
    fn test_high_cpu_is_heavy() {
        let task = Task::builder("batch_processing").with_cpu(8.0).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Heavy);
    }

    #[test]
    fn test_high_memory_is_heavy() {
        let task = Task::builder("data_processing").with_memory(8192).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Heavy);
    }

    #[test]
    fn test_long_duration_is_heavy() {
        let task = Task::builder("long_job").with_duration(600).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Heavy);
    }

    #[test]
    fn test_moderate_cpu_is_moderate() {
        let task = Task::builder("data_transform").with_cpu(2.0).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Moderate);
    }

    #[test]
    fn test_moderate_memory_is_moderate() {
        let task = Task::builder("csv_processing").with_memory(2048).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Moderate);
    }

    #[test]
    fn test_moderate_duration_is_moderate() {
        let task = Task::builder("batch_api_calls").with_duration(60).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Moderate);
    }

    #[test]
    fn test_simple_task_is_lightweight() {
        let task = Task::new("health_check");
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Lightweight);
        assert!(TaskComplexityAnalyzer::is_lightweight(&task));
    }

    #[test]
    fn test_low_resource_task_is_lightweight() {
        let task =
            Task::builder("status_query").with_cpu(0.5).with_memory(256).with_duration(5).build();
        assert_eq!(TaskComplexityAnalyzer::analyze(&task), TaskComplexity::Lightweight);
    }
}
