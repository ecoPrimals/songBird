// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    fn fixed_ts(secs: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0).unwrap()
    }

    fn sample_task(
        status: crate::task_lifecycle::TaskStatus,
        spec: crate::task_lifecycle::TaskSpec,
        current_tower: Option<crate::task_lifecycle::TowerId>,
        created_at: chrono::DateTime<chrono::Utc>,
        progress: f32,
    ) -> TaskLifecycle {
        TaskLifecycle {
            id: TaskId::from_uuid(uuid::Uuid::from_u128(0xdead_beef_0001)),
            status,
            progress,
            created_at,
            eta_seconds: None,
            current_tower,
            owner: crate::task_lifecycle::UserId::from("alice"),
            spec,
            checkpoint_ids: Vec::new(),
            pausable: true,
            cancellable: true,
            resumable: true,
            last_updated: created_at,
        }
    }

    fn base_spec(
        task_type: &str,
        config: serde_json::Value,
        resources: crate::task_lifecycle::ResourceRequirements,
    ) -> crate::task_lifecycle::TaskSpec {
        crate::task_lifecycle::TaskSpec {
            task_type: task_type.into(),
            config,
            required_capabilities: vec![],
            resources,
            priority: crate::task_lifecycle::Priority::Standard,
        }
    }

    #[test]
    fn task_info_new_has_empty_layers() {
        let id = TaskId::from_uuid(uuid::Uuid::from_u128(0x01));
        let info = TaskInfo::new(id);
        assert_eq!(info.task_id, id);
        assert!(info.public.is_none());
        assert!(info.educational.is_none());
        assert!(info.operational.is_none());
        assert!(info.administrative.is_none());
        assert!(info.infrastructure.is_none());
    }

    #[test]
    fn task_info_add_layers_set_each_field() {
        let id = TaskId::from_uuid(uuid::Uuid::from_u128(0x02));
        let mut info = TaskInfo::new(id);

        info.add_public_layer(PublicInfo {
            status: "Queued".into(),
            completion_time_sec: None,
        });
        assert!(info.public.is_some());

        info.add_educational_layer(EducationalInfo {
            sharding_strategy: None,
            node_topology: AnonymizedTopology {
                nodes: vec![],
            },
            learning_notes: vec![],
        });
        assert!(info.educational.is_some());

        info.add_operational_layer(OperationalInfo {
            node_health: vec![],
            failure_details: None,
        });
        assert!(info.operational.is_some());

        info.add_administrative_layer(AdministrativeInfo {
            node_identities: vec![],
            resource_utilization: UtilizationMetrics {
                gpu_hours_used: 0.0,
                average_queue_time_sec: 0.0,
            },
        });
        assert!(info.administrative.is_some());

        info.add_infrastructure_layer(InfrastructureInfo {
            nodes: vec![],
        });
        assert!(info.infrastructure.is_some());
    }

    #[test]
    fn build_public_status_and_completion_time() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(1_000);
        let completed = fixed_ts(3_700);

        let queued = sample_task(
            crate::task_lifecycle::TaskStatus::Queued,
            base_spec(
                "other",
                serde_json::json!({}),
                crate::task_lifecycle::ResourceRequirements::default(),
            ),
            None,
            created,
            0.0,
        );
        let pub_q = builder.build_public(&queued);
        assert_eq!(pub_q.completion_time_sec, None);
        assert!(pub_q.status.contains("Queued"));

        let done = sample_task(
            crate::task_lifecycle::TaskStatus::Completed {
                completed_at: completed,
            },
            base_spec(
                "other",
                serde_json::json!({}),
                crate::task_lifecycle::ResourceRequirements::default(),
            ),
            None,
            created,
            1.0,
        );
        let pub_c = builder.build_public(&done);
        assert!((pub_c.completion_time_sec.unwrap() - 2700.0).abs() < f64::EPSILON);
        assert!(pub_c.status.contains("Completed"));
    }

    #[test]
    fn build_educational_sharding_inference_and_config_override() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(100);
        let spec_cpu = base_spec(
            "ml_training",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: None,
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let edu = builder.build_educational(&sample_task(
            crate::task_lifecycle::TaskStatus::Queued,
            spec_cpu,
            None,
            t,
            0.0,
        ));
        assert_eq!(edu.sharding_strategy.as_deref(), Some("data_parallel"));

        let spec_inf = base_spec(
            "model_inference",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        assert_eq!(
            builder
                .build_educational(&sample_task(
                    crate::task_lifecycle::TaskStatus::Queued,
                    spec_inf,
                    None,
                    t,
                    0.0,
                ))
                .sharding_strategy
                .as_deref(),
            Some("model_parallel")
        );

        let spec_batch = base_spec(
            "batch_processing",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        assert_eq!(
            builder
                .build_educational(&sample_task(
                    crate::task_lifecycle::TaskStatus::Queued,
                    spec_batch,
                    None,
                    t,
                    0.0,
                ))
                .sharding_strategy
                .as_deref(),
            Some("task_parallel")
        );

        let spec_other = base_spec(
            "custom_job",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        assert_eq!(
            builder
                .build_educational(&sample_task(
                    crate::task_lifecycle::TaskStatus::Queued,
                    spec_other,
                    None,
                    t,
                    0.0,
                ))
                .sharding_strategy
                .as_deref(),
            Some("single_node")
        );

        let spec_override = base_spec(
            "ml_training",
            serde_json::json!({ "sharding_strategy": "pipeline_parallel" }),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        assert_eq!(
            builder
                .build_educational(&sample_task(
                    crate::task_lifecycle::TaskStatus::Queued,
                    spec_override,
                    None,
                    t,
                    0.0,
                ))
                .sharding_strategy
                .as_deref(),
            Some("pipeline_parallel")
        );
    }

    #[test]
    fn build_educational_gpu_class_by_resources() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(200);

        let cpu_only = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(0),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let node_cpu = &builder
            .build_educational(&sample_task(
                crate::task_lifecycle::TaskStatus::Queued,
                cpu_only,
                None,
                t,
                0.0,
            ))
            .node_topology
            .nodes[0];
        assert_eq!(node_cpu.gpu_class, "cpu-only");

        let std_gpu = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(1),
                memory_mb: Some(16 * 1024),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let node_std = &builder
            .build_educational(&sample_task(
                crate::task_lifecycle::TaskStatus::Queued,
                std_gpu,
                None,
                t,
                0.0,
            ))
            .node_topology
            .nodes[0];
        assert_eq!(node_std.gpu_class, "standard-gpu");

        let hi_gpu = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(2),
                memory_mb: Some(65 * 1024),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let node_hi = &builder
            .build_educational(&sample_task(
                crate::task_lifecycle::TaskStatus::Queued,
                hi_gpu,
                None,
                t,
                0.0,
            ))
            .node_topology
            .nodes[0];
        assert_eq!(node_hi.gpu_class, "high-memory-gpu");
    }

    #[test]
    fn build_operational_includes_failure_details_when_failed() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(300);
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );

        let ok = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: t,
            },
            spec.clone(),
            None,
            fixed_ts(299),
            0.5,
        );
        assert!(builder.build_operational(&ok).failure_details.is_none());

        let failed = sample_task(
            crate::task_lifecycle::TaskStatus::Failed {
                failed_at: t,
                error: std::sync::Arc::from("boom"),
                retry_count: 0,
            },
            spec,
            None,
            fixed_ts(298),
            0.0,
        );
        let details = builder.build_operational(&failed).failure_details.unwrap();
        assert_eq!(details.error_type, "ExecutionError");
        assert_eq!(details.node, "compute-node-alpha");
        assert!(!details.suggestions.is_empty());
    }

    #[test]
    fn build_administrative_reads_current_tower_identity() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(400);
        let started = fixed_ts(4600);
        let tower = crate::task_lifecycle::TowerId::from("Eastgate");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(2),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: started,
            },
            spec,
            Some(tower),
            created,
            0.42,
        );
        let admin = builder.build_administrative(&task);
        assert_eq!(admin.node_identities.len(), 1);
        assert_eq!(admin.node_identities[0].node_name, "Eastgate");
        assert_eq!(admin.node_identities[0].gpu, "GPU x2");
        assert!((admin.node_identities[0].utilization - 0.42).abs() < f32::EPSILON);
    }

    #[test]
    fn build_infrastructure_full_node_info() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(500);
        let started = fixed_ts(6800);
        let tower = crate::task_lifecycle::TowerId::from("node-full-1");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(1),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: started,
            },
            spec,
            Some(tower),
            created,
            0.0,
        );
        let infra = builder.build_infrastructure(&task);
        assert_eq!(infra.nodes.len(), 1);
        let n = &infra.nodes[0];
        assert_eq!(n.name, "node-full-1");
        assert_eq!(n.internal_ip, "192.0.2.10:8000");
        let expected_uptime = (started.timestamp() - created.timestamp()) as f64 / 3600.0;
        assert!((n.uptime_hours - expected_uptime).abs() < 1e-9);
        assert_eq!(n.temperature_c, Some(65.0));
    }

    #[test]
    fn build_public_all_status_variants() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(1000);
        let spec = || {
            base_spec(
                "x",
                serde_json::json!({}),
                crate::task_lifecycle::ResourceRequirements::default(),
            )
        };

        let running = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: t,
            },
            spec(),
            None,
            t,
            0.5,
        );
        let p = builder.build_public(&running);
        assert!(p.status.contains("Running"));
        assert_eq!(p.completion_time_sec, None);

        let failed = sample_task(
            crate::task_lifecycle::TaskStatus::Failed {
                failed_at: t,
                error: std::sync::Arc::from("err"),
                retry_count: 1,
            },
            spec(),
            None,
            t,
            0.0,
        );
        let p = builder.build_public(&failed);
        assert!(p.status.contains("Failed"));
        assert_eq!(p.completion_time_sec, None);

        let paused = sample_task(
            crate::task_lifecycle::TaskStatus::Paused {
                paused_at: t,
            },
            spec(),
            None,
            t,
            0.3,
        );
        let p = builder.build_public(&paused);
        assert!(p.status.contains("Paused"));
        assert_eq!(p.completion_time_sec, None);

        let cancelled = sample_task(
            crate::task_lifecycle::TaskStatus::Cancelled {
                cancelled_at: t,
                reason: Some(std::sync::Arc::from("user request")),
            },
            spec(),
            None,
            t,
            0.0,
        );
        let p = builder.build_public(&cancelled);
        assert!(p.status.contains("Cancelled"));
        assert_eq!(p.completion_time_sec, None);
    }

    #[test]
    fn build_educational_learning_notes_content() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(100);

        let spec_gpu_with_cpus = base_spec(
            "ml_training",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(2),
                cpu_cores: Some(8),
                memory_mb: Some(16 * 1024),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let edu = builder.build_educational(&sample_task(
            crate::task_lifecycle::TaskStatus::Queued,
            spec_gpu_with_cpus,
            None,
            t,
            0.0,
        ));

        assert!(edu.learning_notes.iter().any(|n| n.contains("data_parallel")));
        assert!(edu.learning_notes.iter().any(|n| n.contains("GPU acceleration")));
        assert!(edu.learning_notes.iter().any(|n| n.contains("8 CPU cores")));
        assert!(edu.learning_notes.iter().any(|n| n.contains("distributed")));
    }

    #[test]
    fn build_educational_capabilities_include_high_memory() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(100);

        let spec_hi_mem = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(1),
                memory_mb: Some(40 * 1024),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let edu = builder.build_educational(&sample_task(
            crate::task_lifecycle::TaskStatus::Queued,
            spec_hi_mem,
            None,
            t,
            0.0,
        ));
        let caps = &edu.node_topology.nodes[0].capabilities;
        assert!(caps.contains(&"compute".to_string()));
        assert!(caps.contains(&"gpu-compute".to_string()));
        assert!(caps.contains(&"high-memory".to_string()));
    }

    #[test]
    fn build_educational_cpu_only_no_gpu_note() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(100);

        let spec_cpu = base_spec(
            "other",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(0),
                cpu_cores: None,
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let edu = builder.build_educational(&sample_task(
            crate::task_lifecycle::TaskStatus::Queued,
            spec_cpu,
            None,
            t,
            0.0,
        ));
        assert!(!edu.learning_notes.iter().any(|n| n.contains("GPU")));
        assert!(!edu.learning_notes.iter().any(|n| n.contains("CPU cores")));
    }

    #[test]
    fn build_operational_no_failure_for_queued_completed_cancelled() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(300);
        let spec = || {
            base_spec(
                "x",
                serde_json::json!({}),
                crate::task_lifecycle::ResourceRequirements::default(),
            )
        };

        let queued = sample_task(crate::task_lifecycle::TaskStatus::Queued, spec(), None, t, 0.0);
        assert!(builder.build_operational(&queued).failure_details.is_none());

        let done = sample_task(
            crate::task_lifecycle::TaskStatus::Completed {
                completed_at: fixed_ts(400),
            },
            spec(),
            None,
            t,
            1.0,
        );
        assert!(builder.build_operational(&done).failure_details.is_none());

        let cancelled = sample_task(
            crate::task_lifecycle::TaskStatus::Cancelled {
                cancelled_at: fixed_ts(350),
                reason: None,
            },
            spec(),
            None,
            t,
            0.0,
        );
        assert!(builder.build_operational(&cancelled).failure_details.is_none());
    }

    #[test]
    fn build_administrative_no_tower_yields_empty_identities() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(400);
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        let task = sample_task(crate::task_lifecycle::TaskStatus::Queued, spec, None, t, 0.0);
        let admin = builder.build_administrative(&task);
        assert!(admin.node_identities.is_empty());
        assert!((admin.resource_utilization.gpu_hours_used).abs() < f64::EPSILON);
        assert!((admin.resource_utilization.average_queue_time_sec).abs() < f64::EPSILON);
    }

    #[test]
    fn build_administrative_cpu_only_gpu_info() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(400);
        let started = fixed_ts(500);
        let tower = crate::task_lifecycle::TowerId::from("cpu-tower");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(0),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: started,
            },
            spec,
            Some(tower),
            created,
            0.7,
        );
        let admin = builder.build_administrative(&task);
        assert_eq!(admin.node_identities[0].gpu, "CPU-only");
        assert!((admin.resource_utilization.gpu_hours_used).abs() < f64::EPSILON);
    }

    #[test]
    fn build_administrative_gpu_hours_completed_task() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(0);
        let completed = fixed_ts(7200);
        let tower = crate::task_lifecycle::TowerId::from("gpu-tower");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(4),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Completed {
                completed_at: completed,
            },
            spec,
            Some(tower),
            created,
            1.0,
        );
        let admin = builder.build_administrative(&task);
        let expected_hours = 2.0 * 4.0;
        assert!(
            (admin.resource_utilization.gpu_hours_used - expected_hours).abs() < f64::EPSILON,
            "gpu_hours_used: {}",
            admin.resource_utilization.gpu_hours_used
        );
    }

    #[test]
    fn build_administrative_queue_time_for_completed() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(0);
        let completed = fixed_ts(1000);
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Completed {
                completed_at: completed,
            },
            spec,
            None,
            created,
            1.0,
        );
        let admin = builder.build_administrative(&task);
        assert!(
            (admin.resource_utilization.average_queue_time_sec - 100.0).abs() < f64::EPSILON,
            "queue_time: {}",
            admin.resource_utilization.average_queue_time_sec
        );
    }

    #[test]
    fn build_infrastructure_no_tower_yields_empty_nodes() {
        let builder = super::super::InformationLayerBuilder::new();
        let t = fixed_ts(500);
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements::default(),
        );
        let task = sample_task(crate::task_lifecycle::TaskStatus::Queued, spec, None, t, 0.0);
        let infra = builder.build_infrastructure(&task);
        assert!(infra.nodes.is_empty());
    }

    #[test]
    fn build_infrastructure_cpu_only_no_temperature() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(500);
        let started = fixed_ts(600);
        let tower = crate::task_lifecycle::TowerId::from("cpu-node");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(0),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Running {
                started_at: started,
            },
            spec,
            Some(tower),
            created,
            0.0,
        );
        let infra = builder.build_infrastructure(&task);
        assert_eq!(infra.nodes[0].temperature_c, None);
    }

    #[test]
    fn build_infrastructure_completed_uses_completed_at_for_uptime() {
        let builder = super::super::InformationLayerBuilder::new();
        let created = fixed_ts(0);
        let completed = fixed_ts(3600);
        let tower = crate::task_lifecycle::TowerId::from("done-node");
        let spec = base_spec(
            "x",
            serde_json::json!({}),
            crate::task_lifecycle::ResourceRequirements {
                gpu_count: Some(1),
                ..crate::task_lifecycle::ResourceRequirements::default()
            },
        );
        let task = sample_task(
            crate::task_lifecycle::TaskStatus::Completed {
                completed_at: completed,
            },
            spec,
            Some(tower),
            created,
            1.0,
        );
        let infra = builder.build_infrastructure(&task);
        assert!((infra.nodes[0].uptime_hours - 1.0).abs() < 1e-9);
        assert_eq!(infra.nodes[0].temperature_c, Some(65.0));
    }
}
