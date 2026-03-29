// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Deployment specifications, resources, and rollout status.

use super::service::Endpoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Describe a workload the orchestrator should run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    /// Kubernetes-style deployment name.
    pub name: String,
    /// Container image reference.
    pub image: String,
    /// Desired replica count.
    pub replicas: u32,
    /// CPU and memory bounds.
    pub resources: ResourceRequirements,
    /// Environment variables for the workload.
    pub environment: HashMap<String, String>,
    /// Ports to expose.
    pub ports: Vec<PortSpec>,
}

/// Express CPU and memory requests and limits for schedulers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Hard CPU cap (Kubernetes quantity string).
    pub cpu_limit: Option<String>,
    /// Hard memory cap.
    pub memory_limit: Option<String>,
    /// Guaranteed CPU reservation.
    pub cpu_request: Option<String>,
    /// Guaranteed memory reservation.
    pub memory_request: Option<String>,
}

/// Map a named service port inside a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port name for service discovery.
    pub name: String,
    /// Port exposed on the Service resource.
    pub port: u16,
    /// Port the container listens on.
    pub target_port: u16,
    /// Transport protocol (TCP/UDP).
    pub protocol: String,
}

/// Return orchestration outcome after applying a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Orchestrator-assigned deployment id.
    pub deployment_id: String,
    /// High-level rollout state.
    pub status: DeploymentStatus,
    /// Endpoints that became reachable.
    pub endpoints: Vec<Endpoint>,
    /// Human-readable status or error text.
    pub message: String,
}

/// Track rollout lifecycle for a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Accepted but not yet scheduled.
    Pending,
    /// At least one replica is ready.
    Running,
    /// Rollout failed; inspect message and events.
    Failed,
    /// Workload has been torn down.
    Terminated,
}

/// Snapshot deployment state for dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current rollout status.
    pub status: DeploymentStatus,
    /// Desired replica count.
    pub replicas: u32,
    /// Replicas passing readiness checks.
    pub ready_replicas: u32,
    /// Creation timestamp.
    pub created_at: SystemTime,
    /// Last status transition.
    pub updated_at: SystemTime,
}
