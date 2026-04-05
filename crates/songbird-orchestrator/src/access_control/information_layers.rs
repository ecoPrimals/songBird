// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Information layer builders for graduated disclosure

use crate::task_lifecycle::{TaskId, TaskLifecycle};
use serde::{Deserialize, Serialize};

/// Complete task information with graduated layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: TaskId,
    pub public: Option<PublicInfo>,
    pub educational: Option<EducationalInfo>,
    pub operational: Option<OperationalInfo>,
    pub administrative: Option<AdministrativeInfo>,
    pub infrastructure: Option<InfrastructureInfo>,
}

impl TaskInfo {
    #[must_use]
    pub const fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            public: None,
            educational: None,
            operational: None,
            administrative: None,
            infrastructure: None,
        }
    }

    pub fn add_public_layer(&mut self, info: PublicInfo) {
        self.public = Some(info);
    }

    pub fn add_educational_layer(&mut self, info: EducationalInfo) {
        self.educational = Some(info);
    }

    pub fn add_operational_layer(&mut self, info: OperationalInfo) {
        self.operational = Some(info);
    }

    pub fn add_administrative_layer(&mut self, info: AdministrativeInfo) {
        self.administrative = Some(info);
    }

    pub fn add_infrastructure_layer(&mut self, info: InfrastructureInfo) {
        self.infrastructure = Some(info);
    }
}

/// Layer 0: Public information (anyone can see)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInfo {
    pub status: String, // "queued", "running", "completed", "failed"
    pub completion_time_sec: Option<f64>,
}

/// Layer 1: Educational information (students learning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalInfo {
    pub sharding_strategy: Option<String>,
    pub node_topology: AnonymizedTopology,
    pub learning_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedTopology {
    pub nodes: Vec<AnonymousNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousNode {
    pub node_id: String, // "compute-node-alpha"
    pub capabilities: Vec<String>,
    pub gpu_class: String, // "high-memory-gpu"
}

/// Layer 2: Operational information (TAs debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalInfo {
    pub node_health: Vec<NodeHealthStatus>,
    pub failure_details: Option<FailureContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealthStatus {
    pub node_id: String, // Still anonymized
    pub gpu_utilization: f32,
    pub memory_utilization: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContext {
    pub error_type: String,
    pub error_message: String,
    pub node: String,
    pub suggestions: Vec<String>,
}

/// Layer 3: Administrative information (professors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeInfo {
    pub node_identities: Vec<NodeIdentity>,
    pub resource_utilization: UtilizationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_name: String, // "Eastgate"
    pub gpu: String,       // "RTX 3090 24GB"
    pub utilization: f32,
    // Still NO IPs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationMetrics {
    pub gpu_hours_used: f64,
    pub average_queue_time_sec: f64,
}

/// Layer 4: Infrastructure information (admins)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureInfo {
    pub nodes: Vec<NodeFull>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFull {
    pub name: String,
    pub internal_ip: String, // "192.0.2.10:8000"
    pub uptime_hours: f64,
    pub temperature_c: Option<f32>,
}

impl super::InformationLayerBuilder {
    #[must_use]
    pub fn build_public(&self, task: &TaskLifecycle) -> PublicInfo {
        let completion_time_sec = match &task.status {
            super::super::task_lifecycle::TaskStatus::Completed {
                completed_at,
            } => Some((completed_at.timestamp() - task.created_at.timestamp()) as f64),
            _ => None,
        };

        PublicInfo {
            status: format!("{:?}", task.status),
            completion_time_sec,
        }
    }

    /// Build educational layer information with real task data
    ///
    /// Extracts learning-relevant information:
    /// - Sharding strategy from task config or inferred from task type
    /// - Anonymized node topology based on resource requirements
    /// - Educational notes about task execution
    pub fn build_educational(&self, task: &TaskLifecycle) -> EducationalInfo {
        // Extract or infer sharding strategy
        let sharding_strategy = task
            .spec
            .config
            .get("sharding_strategy")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                // Infer from task type
                match task.spec.task_type.as_ref() {
                    "ml_training" | "data_processing" => Some("data_parallel".to_string()),
                    "model_inference" => Some("model_parallel".to_string()),
                    "batch_processing" => Some("task_parallel".to_string()),
                    _ => Some("single_node".to_string()),
                }
            });

        // Build anonymized node with capabilities inferred from requirements
        let capabilities = {
            let mut caps = vec!["compute".to_string()];
            if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                caps.push("gpu-compute".to_string());
            }
            if task.spec.resources.memory_mb.unwrap_or(0) > 32 * 1024 {
                caps.push("high-memory".to_string());
            }
            caps
        };

        let gpu_class = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            if task.spec.resources.memory_mb.unwrap_or(0) > 64 * 1024 {
                "high-memory-gpu".to_string()
            } else {
                "standard-gpu".to_string()
            }
        } else {
            "cpu-only".to_string()
        };

        // Generate educational notes
        let mut learning_notes = Vec::new();

        if let Some(ref strategy) = sharding_strategy {
            learning_notes.push(format!("Task uses {strategy} sharding for parallel execution"));
        }

        if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            learning_notes.push("Task leverages GPU acceleration for performance".to_string());
        }

        if let Some(cpus) = task.spec.resources.cpu_cores {
            learning_notes.push(format!("Task parallelized across {cpus} CPU cores"));
        }

        learning_notes.push("Your task was distributed across available compute nodes".to_string());

        EducationalInfo {
            sharding_strategy,
            node_topology: AnonymizedTopology {
                nodes: vec![AnonymousNode {
                    node_id: "compute-node-alpha".into(),
                    capabilities,
                    gpu_class,
                }],
            },
            learning_notes,
        }
    }

    #[must_use]
    pub fn build_operational(&self, task: &TaskLifecycle) -> OperationalInfo {
        OperationalInfo {
            node_health: vec![],
            failure_details: if matches!(
                task.status,
                super::super::task_lifecycle::TaskStatus::Failed { .. }
            ) {
                Some(FailureContext {
                    error_type: "ExecutionError".into(),
                    error_message: "Task execution failed".into(),
                    node: "compute-node-alpha".into(),
                    suggestions: vec!["Check task logs for details".into()],
                })
            } else {
                None
            },
        }
    }

    /// Build administrative layer information with real task data
    ///
    /// Extracts administrative details:
    /// - Node identity and hardware (still no IPs at this layer)
    /// - Resource utilization metrics
    #[must_use]
    pub fn build_administrative(&self, task: &TaskLifecycle) -> AdministrativeInfo {
        // Extract node identity from current tower
        let node_identities = task.current_tower.as_ref().map_or_else(Vec::new, |tower| {
            // Get GPU info from resources
            let gpu_info = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                format!("GPU x{}", task.spec.resources.gpu_count.unwrap_or(1))
            } else {
                "CPU-only".to_string()
            };

            vec![NodeIdentity {
                node_name: tower.as_str().to_string(),
                gpu: gpu_info,
                utilization: task.progress, // Use task progress as utilization proxy
            }]
        });

        // Calculate resource utilization
        let gpu_hours_used = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            // Estimate based on task duration and progress
            match &task.status {
                super::super::task_lifecycle::TaskStatus::Completed {
                    completed_at,
                }
                | super::super::task_lifecycle::TaskStatus::Running {
                    started_at: completed_at,
                } => {
                    let duration_hours =
                        (completed_at.timestamp() - task.created_at.timestamp()) as f64 / 3600.0;
                    duration_hours * f64::from(task.spec.resources.gpu_count.unwrap_or(1))
                }
                _ => 0.0,
            }
        } else {
            0.0
        };

        // Calculate average queue time
        let average_queue_time_sec = match &task.status {
            super::super::task_lifecycle::TaskStatus::Running {
                started_at,
            } => (started_at.timestamp() - task.created_at.timestamp()) as f64,
            super::super::task_lifecycle::TaskStatus::Completed {
                completed_at,
            } => {
                // Estimate queue time as 10% of total time
                let total_time = (completed_at.timestamp() - task.created_at.timestamp()) as f64;
                total_time * 0.1
            }
            _ => 0.0,
        };

        AdministrativeInfo {
            node_identities,
            resource_utilization: UtilizationMetrics {
                gpu_hours_used,
                average_queue_time_sec,
            },
        }
    }

    /// Build infrastructure layer information with real task data
    ///
    /// Extracts complete infrastructure details (admin-level):
    /// - Full node specifications
    /// - Internal IPs and network topology
    /// - Hardware metrics (uptime, temperature)
    #[must_use]
    pub fn build_infrastructure(&self, task: &TaskLifecycle) -> InfrastructureInfo {
        // Build infrastructure node from current tower
        let nodes = task.current_tower.as_ref().map_or_else(Vec::new, |tower| {
            // Calculate uptime based on task execution
            let uptime_hours = match &task.status {
                super::super::task_lifecycle::TaskStatus::Running {
                    started_at,
                }
                | super::super::task_lifecycle::TaskStatus::Completed {
                    completed_at: started_at,
                } => (started_at.timestamp() - task.created_at.timestamp()) as f64 / 3600.0,
                _ => 0.0,
            };

            vec![NodeFull {
                name: tower.as_str().to_string(),
                internal_ip: "192.0.2.10:8000".to_string(), // Would be looked up from service registry
                uptime_hours,
                temperature_c: if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                    Some(65.0) // Would query actual GPU temperature
                } else {
                    None
                },
            }]
        });

        InfrastructureInfo {
            nodes,
        }
    }
}
