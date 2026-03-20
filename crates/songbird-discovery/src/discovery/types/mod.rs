// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::service::ServiceInfo;

/// Node ID for identifying unique nodes in the federation
pub type NodeId = String;

/// Federation discovery protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessage {
    /// Node announces itself to the federation
    NodeAnnouncement {
        node: Box<NodeInfo>,
        timestamp: DateTime<Utc>,
    },
    /// Request for node information
    NodeDiscoveryRequest {
        sender_id: NodeId,
    },
    /// Response to discovery request
    NodeDiscoveryResponse {
        nodes: Vec<NodeInfo>,
    },
    /// Periodic heartbeat
    Heartbeat {
        node_id: NodeId,
        resource_usage: ResourceUsage,
    },
    /// Service advertisement
    ServiceAdvertisement {
        services: Vec<ServiceInfo>,
    },
}

/// Local node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub institution: Option<String>,
    pub resources: ComputeResources,
    pub network_location: NetworkLocation,
    pub created_at: DateTime<Utc>,
}

/// Information about a node in the federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Basic identification
    pub id: NodeId,
    pub node_type: NodeType,
    pub institution: Option<String>,
    pub address: String,

    /// Compute capabilities
    pub resources: ComputeResources,
    pub current_load: ResourceUsage,

    /// Data capabilities
    pub available_datasets: Vec<DatasetInfo>,
    pub storage_capacity: StorageInfo,

    /// Federation metadata
    pub trust_level: TrustLevel,
    pub reputation_score: f64,

    /// Network optimization
    pub bandwidth_measurements: HashMap<NodeId, f64>, // Mbps
    pub latency_measurements: HashMap<NodeId, f64>, // ms

    /// Operational
    pub last_seen: DateTime<Utc>,
    pub health_status: ServiceHealthStatus,
    pub services: Vec<String>, // Service IDs hosted on this node
}

/// Type of node in the federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Compute-focused node (CPUs, GPUs,
    Compute,
    /// Storage-focused node (large datasets)
    Storage,
    /// Gateway node (connects institutions)
    Gateway,
    /// Hybrid node (compute + storage)
    Hybrid,
    /// Orchestrator node (management)
    Orchestrator,
}

/// Compute resource description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResources {
    pub cpu_cores: u32,
    pub cpu_architecture: String, // "x86_64", "ARM64", etc."
    pub memory_total_gb: u64,
    pub memory_available_gb: u64,
    pub gpu_info: Vec<GpuInfo>,
    pub storage_devices: Vec<StorageDevice>,
    pub network_bandwidth_mbps: f64,
}

/// GPU information for scientific computing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub model: String,                      // "NVIDIA A100", "AMD MI250X""
    pub memory_gb: u32,                     // GPU memory
    pub compute_capability: Option<String>, // For CUDA compatibility
    pub utilization_percent: f32,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub device_type: String, // "NVMe", "SSD", "HDD""
    pub capacity_gb: u64,
    pub available_gb: u64,
    pub mount_point: String,
    pub performance_tier: StoragePerformanceTier,
}

/// Performance classification for storage
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum StoragePerformanceTier {
    HighPerformance, // NVMe, fast SSDs
    Standard,        // Regular SSDs
    Archive,         // HDDs, bulk storage
}

/// Current resource usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_utilization_percent: f32,
    pub memory_used_gb: u64,
    pub gpu_utilization: Vec<f32>, // Per-GPU utilization
    pub storage_used_gb: u64,
    pub network_utilization_percent: f32,
    pub active_jobs: u32,
}

/// Scientific dataset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub id: String,
    pub name: String,
    pub dataset_type: DatasetType,
    pub size_bytes: u64,
    pub format: String, // "FASTA", "FASTQ", "BAM", "VCF", etc."
    pub checksum: String,
    pub access_level: AccessLevel,
    pub last_updated: DateTime<Utc>,
}

/// Scientific dataset types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatasetType {
    Genomic,
    Proteomic,
    Imaging,
    Metabolomic,
    Clinical,
    Environmental,
    Other(String),
}

/// Data access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,        // Openly available
    Institutional, // Within institution only
    Consortium,    // Multi-institutional consortium
    Private,       // Restricted access
}

/// Storage capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_capacity_gb: u64,
    pub available_capacity_gb: u64,
    pub performance_tier_breakdown: HashMap<StoragePerformanceTier, u64>,
}

/// Trust level in federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq, Default)]
pub enum TrustLevel {
    #[default]
    Unknown,
    Basic,
    Verified,
    Institutional,
    Consortium,
}

/// Network location for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLocation {
    pub region: String,         // "us-east", "eu-west", etc."
    pub subnet: Option<String>, // For local optimization
    pub external_ip: Option<String>,
    pub internal_ip: Option<String>,
}

/// Resource query for finding optimal compute nodes
#[derive(Debug, Clone, Default)]
pub struct ResourceQuery {
    pub min_cpu_cores: Option<u32>,
    pub min_memory_gb: Option<u64>,
    pub required_node_type: Option<NodeType>,
    pub institution_filter: Option<String>,
    pub min_trust_level: TrustLevel,
    pub max_latency_ms: Option<f64>,
    pub required_datasets: Vec<String>,
}

/// Federation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationStats {
    pub total_nodes: u32,
    pub total_services: u32,
    pub compute_nodes: u32,
    pub storage_nodes: u32,
    pub gateway_nodes: u32,
    pub hybrid_nodes: u32,
    pub orchestrator_nodes: u32,
    pub total_cpu_cores: u32,
    pub total_memory_gb: u64,
    pub total_storage_gb: u64,
    pub average_trust_score: f64,
    pub federation_health: f64, // 0.0 to 1.0
}

/// Federation health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationHealth {
    pub healthy_nodes: u32,
    pub degraded_nodes: u32,
    pub unhealthy_nodes: u32,
    pub overall_health_score: f64, // 0-100
    pub average_cpu_utilization: f32,
    pub average_memory_utilization: f64,
    pub network_partition_detected: bool,
}

/// Node interaction result for reputation updates
#[derive(Debug, Clone, Copy)]
pub enum InteractionResult {
    Success,
    SlowResponse,
    Failure,
    Timeout,
    Malicious,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub nodes_by_region: HashMap<String, u32>,
    pub average_latencies: HashMap<NodeId, f64>,
    pub network_partitions: Vec<NetworkPartition>,
    pub bandwidth_measurements: HashMap<NodeId, f64>,
}

/// Network partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPartition {
    pub partition_id: String,
    pub affected_nodes: Vec<NodeId>,
    pub detected_at: DateTime<Utc>,
    pub severity: PartitionSeverity,
}

/// Severity of network partition
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PartitionSeverity {
    Minor,    // Few nodes affected
    Major,    // Significant portion affected
    Critical, // Majority of nodes affected
}

/// Network performance measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeasurement {
    pub target_node_id: NodeId,
    pub latency_ms: f64,
    pub bandwidth_mbps: f64,
    pub packet_loss_percent: f64,
    pub jitter_ms: f64,
    pub measured_at: DateTime<Utc>,
}

/// Resource monitoring update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdate {
    pub node_id: NodeId,
    pub cpu_usage: CpuUsage,
    pub memory_usage: MemoryUsage,
    pub gpu_usage: Vec<GpuUsage>,
    pub network_usage: NetworkUsage,
    pub storage_usage: Vec<StorageUsage>,
    pub timestamp: DateTime<Utc>,
}

/// Detailed CPU usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub overall_percent: f32,
    pub per_core_percent: Vec<f32>,
    pub load_average: [f64; 3], // 1min, 5min, 15min
    pub context_switches_per_sec: u64,
    pub interrupts_per_sec: u64,
}

/// Detailed memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names, reason = "memory/swap field names match host metrics schema")]
pub struct MemoryUsage {
    pub total_gb: u64,
    pub used_gb: u64,
    pub cached_gb: u64,
    pub buffer_gb: u64,
    pub swap_total_gb: u64,
    pub swap_used_gb: u64,
}

/// GPU usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUsage {
    pub gpu_index: u32,
    pub utilization_percent: f32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
    pub temperature_celsius: f32,
    pub power_draw_watts: f32,
}

/// Network interface usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names, reason = "per-sec counters match network interface stats")]
pub struct NetworkUsage {
    pub bytes_sent_per_sec: u64,
    pub bytes_received_per_sec: u64,
    pub packets_sent_per_sec: u64,
    pub packets_received_per_sec: u64,
    pub errors_per_sec: u64,
    pub drops_per_sec: u64,
}

/// Storage device usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub device_name: String,
    pub reads_per_sec: u64,
    pub writes_per_sec: u64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub queue_depth: f32,
}
