//! Common types for intelligent routing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A task to be executed, either locally, on a peer, or via a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Type of task (e.g., "ml_training", "data_processing", "health_check")
    pub task_type: String,

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
                task_type: task_type.into(),
                payload: serde_json::Value::Null,
                resource_requirements: None,
                estimated_duration_secs: None,
                metadata: HashMap::new(),
            },
        }
    }

    /// Set task payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.task.payload = payload;
        self
    }

    /// Set resource requirements
    pub fn with_resources(mut self, requirements: ResourceRequirements) -> Self {
        self.task.resource_requirements = Some(requirements);
        self
    }

    /// Require GPU for this task
    pub fn with_gpu(mut self) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.gpu_required = true;
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set CPU requirements
    pub fn with_cpu(mut self, cores: f64) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.cpu_cores = Some(cores);
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set memory requirements
    pub fn with_memory(mut self, memory_mb: u64) -> Self {
        let mut reqs = self.task.resource_requirements.unwrap_or_default();
        reqs.memory_mb = Some(memory_mb);
        self.task.resource_requirements = Some(reqs);
        self
    }

    /// Set estimated duration
    pub fn with_duration(mut self, duration_secs: u64) -> Self {
        self.task.estimated_duration_secs = Some(duration_secs);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.task.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the task
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
            task_type: task_type.into(),
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

        assert_eq!(task.task_type, "ml_training");
        assert!(task.resource_requirements.as_ref().unwrap().gpu_required);
        assert_eq!(task.resource_requirements.as_ref().unwrap().cpu_cores, Some(4.0));
        assert_eq!(task.estimated_duration_secs, Some(600));
        assert_eq!(task.metadata.get("model"), Some(&"resnet50".to_string()));
    }

    #[test]
    fn test_simple_task() {
        let task = Task::new("health_check");
        assert_eq!(task.task_type, "health_check");
        assert!(task.resource_requirements.is_none());
    }
}
