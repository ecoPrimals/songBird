// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Data structures for graduated information disclosure layers.

use crate::task_lifecycle::TaskId;
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
    pub internal_ip: String,
    pub uptime_hours: f64,
    pub temperature_c: Option<f32>,
}
