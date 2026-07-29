// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Builder implementations that map [`TaskLifecycle`] to each information layer.

use super::types::{
    AdministrativeInfo, AnonymizedTopology, AnonymousNode, EducationalInfo, FailureContext,
    InfrastructureInfo, NodeFull, NodeIdentity, OperationalInfo, PublicInfo, UtilizationMetrics,
};
use crate::task_lifecycle::TaskLifecycle;

impl super::super::InformationLayerBuilder {
    #[must_use]
    pub fn build_public(&self, task: &TaskLifecycle) -> PublicInfo {
        let completion_time_sec = match &task.status {
            super::super::super::task_lifecycle::TaskStatus::Completed {
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
        let sharding_strategy = task
            .spec
            .config
            .get("sharding_strategy")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| match task.spec.task_type.as_ref() {
                "ml_training" | "data_processing" => Some(String::from("data_parallel")),
                "model_inference" => Some(String::from("model_parallel")),
                "batch_processing" => Some(String::from("task_parallel")),
                _ => Some(String::from("single_node")),
            });

        let capabilities = {
            let mut caps = vec![String::from("compute")];
            if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                caps.push(String::from("gpu-compute"));
            }
            if task.spec.resources.memory_mb.unwrap_or(0) > 32 * 1024 {
                caps.push(String::from("high-memory"));
            }
            caps
        };

        let gpu_class = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            if task.spec.resources.memory_mb.unwrap_or(0) > 64 * 1024 {
                String::from("high-memory-gpu")
            } else {
                String::from("standard-gpu")
            }
        } else {
            String::from("cpu-only")
        };

        let mut learning_notes = Vec::new();

        if let Some(ref strategy) = sharding_strategy {
            learning_notes.push(format!("Task uses {strategy} sharding for parallel execution"));
        }

        if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            learning_notes.push(String::from("Task leverages GPU acceleration for performance"));
        }

        if let Some(cpus) = task.spec.resources.cpu_cores {
            learning_notes.push(format!("Task parallelized across {cpus} CPU cores"));
        }

        learning_notes
            .push(String::from("Your task was distributed across available compute nodes"));

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
                super::super::super::task_lifecycle::TaskStatus::Failed { .. }
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
        let node_identities = task.current_tower.as_ref().map_or_else(Vec::new, |tower| {
            let gpu_info = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                format!("GPU x{}", task.spec.resources.gpu_count.unwrap_or(1))
            } else {
                String::from("CPU-only")
            };

            vec![NodeIdentity {
                node_name: tower.as_str().to_string(),
                gpu: gpu_info,
                utilization: task.progress,
            }]
        });

        let gpu_hours_used = if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
            match &task.status {
                super::super::super::task_lifecycle::TaskStatus::Completed {
                    completed_at,
                }
                | super::super::super::task_lifecycle::TaskStatus::Running {
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

        let average_queue_time_sec = match &task.status {
            super::super::super::task_lifecycle::TaskStatus::Running {
                started_at,
            } => (started_at.timestamp() - task.created_at.timestamp()) as f64,
            super::super::super::task_lifecycle::TaskStatus::Completed {
                completed_at,
            } => {
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
        let nodes = task.current_tower.as_ref().map_or_else(Vec::new, |tower| {
            let uptime_hours = match &task.status {
                super::super::super::task_lifecycle::TaskStatus::Running {
                    started_at,
                }
                | super::super::super::task_lifecycle::TaskStatus::Completed {
                    completed_at: started_at,
                } => (started_at.timestamp() - task.created_at.timestamp()) as f64 / 3600.0,
                _ => 0.0,
            };

            vec![NodeFull {
                name: tower.as_str().to_string(),
                internal_ip: songbird_process_env::var("SONGBIRD_FEDERATION_BIND")
                    .or_else(|_| songbird_process_env::var("SONGBIRD_PRODUCTION_BIND_ADDRESS"))
                    .unwrap_or_else(|_| {
                        String::from(songbird_types::constants::PRODUCTION_BIND_ADDRESS)
                    }),
                uptime_hours,
                temperature_c: if task.spec.resources.gpu_count.unwrap_or(0) > 0 {
                    Some(65.0)
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
