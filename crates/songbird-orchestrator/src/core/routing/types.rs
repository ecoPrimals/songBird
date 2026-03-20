// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Common types for intelligent routing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A task to be executed, either locally, on a peer, or via a capability
///
/// **ZERO-COPY OPTIMIZATION**: Uses `Arc<str>` for `task_type` to enable
/// cheap sharing across async boundaries without cloning large strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Type of task (e.g., "`ml_training`", "`data_processing`", "`health_check`")
    ///
    /// **OPTIMIZED**: `Arc<str>` enables zero-copy sharing in async contexts
    #[serde(with = "arc_str_serde")]
    pub task_type: std::sync::Arc<str>,

    /// Task payload (arbitrary JSON)
    #[serde(default)]
    pub payload: serde_json::Value,

    /// Resource requirements for this task
    #[serde(default)]
    pub resource_requirements: Option<ResourceRequirements>,

    /// Estimated duration in seconds (for routing optimization)
    #[serde(default)]
    pub estimated_duration_secs: Option<u64>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Serde support for `Arc<str>`
mod arc_str_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(arc: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(arc)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Arc::from(s.as_str()))
    }
}

/// Resource requirements for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required (e.g., 2.0 for 2 cores)
    pub cpu_cores: Option<f64>,

    /// Memory required in MB
    pub memory_mb: Option<u64>,

    /// Whether GPU is required
    pub gpu_required: bool,

    /// Storage space required in MB
    pub storage_mb: Option<u64>,

    /// Network bandwidth required in Mbps
    pub network_mbps: Option<f64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            gpu_required: false,
            storage_mb: Some(100),
            network_mbps: Some(10.0),
        }
    }
}

/// Builder for creating tasks
pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new(task_type: impl Into<String>) -> Self {
        Self {
            task: Task {
                task_type: std::sync::Arc::from(task_type.into().as_str()),
                payload: serde_json::Value::Null,
                resource_requirements: None,
                estimated_duration_secs: None,
                metadata: HashMap::new(),
            },
        }
    }

    /// Set task payload
    #[must_use = "Builder methods should be chained or assigned"]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.task.payload = payload;
        self
    }

    /// Set resource requirements
    #[must_use = "Builder methods should be chained or assigned"]
    pub const fn with_resources(mut self, requirements: ResourceRequirements) -> Self {
        self.task.resource_requirements = Some(requirements);
        self
    }

    /// Require GPU for this task
    #[must_use = "Builder methods should be chained or assigned"]
    pub fn with_gpu(mut self) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.gpu_required = true;
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set CPU requirements
    #[must_use = "Builder methods should be chained or assigned"]
    pub fn with_cpu(mut self, cores: f64) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.cpu_cores = Some(cores);
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set memory requirements
    #[must_use = "Builder methods should be chained or assigned"]
    pub fn with_memory(mut self, memory_mb: u64) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.memory_mb = Some(memory_mb);
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set estimated duration
    #[must_use = "Builder methods should be chained or assigned"]
    pub const fn with_duration(mut self, duration_secs: u64) -> Self {
        self.task.estimated_duration_secs = Some(duration_secs);
        self
    }

    /// Add metadata
    #[must_use = "Builder methods should be chained or assigned"]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.task.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the task
    #[must_use]
    pub fn build(self) -> Task {
        self.task
    }
}

impl Task {
    /// Create a new task builder
    pub fn builder(task_type: impl Into<String>) -> TaskBuilder {
        TaskBuilder::new(task_type)
    }

    /// Quick constructor for simple tasks
    pub fn new(task_type: impl Into<String>) -> Self {
        Self {
            task_type: std::sync::Arc::from(task_type.into().as_str()),
            payload: serde_json::Value::Null,
            resource_requirements: None,
            estimated_duration_secs: None,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_builder() {
        let task = Task::builder("ml_training")
            .with_gpu()
            .with_cpu(4.0)
            .with_memory(8192)
            .with_duration(600)
            .with_metadata("model", "resnet50")
            .build();

        assert_eq!(task.task_type.as_ref(), "ml_training");
        // Validate GPU task routing - use expect in tests with clear messages
        let requirements = task
            .resource_requirements
            .as_ref()
            .expect("GPU task should have resource requirements - test invariant");
        assert!(requirements.gpu_required);
        assert_eq!(requirements.cpu_cores, Some(4.0));
        assert_eq!(task.estimated_duration_secs, Some(600));
        assert_eq!(task.metadata.get("model"), Some(&"resnet50".to_string()));
    }

    #[test]
    fn test_simple_task() {
        let task = Task::new("health_check");
        assert_eq!(task.task_type.as_ref(), "health_check");
        assert!(task.resource_requirements.is_none());
    }
}
