//! Resource allocation recommendations and scaling policies

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Resource allocation recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// CPU allocation recommendations
    pub cpu: CPUAllocation,

    /// Memory allocation recommendations
    pub memory: MemoryAllocation,

    /// Storage allocation recommendations
    pub storage: StorageAllocation,

    /// Network allocation recommendations
    pub network: NetworkAllocation,

    /// Scaling recommendations
    pub scaling: ScalingRecommendations,
}

/// CPU allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUAllocation {
    /// Minimum CPU cores required
    pub min_cores: f64,

    /// Optimal CPU cores
    pub optimal_cores: f64,

    /// Maximum CPU cores that can be utilized
    pub max_cores: f64,

    /// CPU architecture preferences
    pub architecture_preferences: Vec<String>,

    /// Special CPU features required
    pub required_features: Vec<String>,
}

/// Memory allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    /// Minimum memory in GB
    pub min_memory_gb: f64,

    /// Optimal memory in GB
    pub optimal_memory_gb: f64,

    /// Maximum memory that can be utilized in GB
    pub max_memory_gb: f64,

    /// Memory type preferences
    pub memory_type_preferences: Vec<String>,

    /// Memory bandwidth requirements
    pub bandwidth_requirements_gbps: f64,
}

/// Storage allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAllocation {
    /// Primary storage requirements
    pub primary_storage_gb: f64,

    /// Secondary storage requirements
    pub secondary_storage_gb: f64,

    /// Storage type preference
    pub storage_type: StorageType,

    /// IOPS requirements
    pub iops_requirements: u64,

    /// Storage durability requirements
    pub durability_level: String,
}

/// Storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    SSD,
    HDD,
    NVMe,
    NetworkStorage,
    ObjectStorage,
    BlockStorage,
    FileSystem,
}

/// Network allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAllocation {
    /// Bandwidth requirements in Mbps
    pub bandwidth_mbps: f64,

    /// Connection pool size
    pub connection_pool_size: u32,

    /// Network latency requirements
    pub max_latency_ms: f64,

    /// Network reliability requirements
    pub reliability_percentage: f64,

    /// Quality of service requirements
    pub qos_class: String,
}

/// Scaling recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendations {
    /// Recommended scaling triggers
    pub triggers: Vec<ScalingTrigger>,

    /// Scaling policies
    pub policies: Vec<ScalingPolicy>,

    /// Auto-scaling suitability
    pub auto_scaling_suitable: bool,
}

/// Scaling trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTrigger {
    /// Metric to monitor for scaling
    pub metric: String,

    /// Threshold value for scaling
    pub threshold: f64,

    /// Scaling direction when threshold is reached
    pub direction: ScalingDirection,

    /// Cooldown period after scaling
    pub cooldown_seconds: u64,
}

/// Scaling directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingDirection {
    Up,
    Down,
}

/// Scaling policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy name
    pub name: String,

    /// Scaling amount
    pub amount: ScalingAmount,

    /// Minimum instances
    pub min_instances: u32,

    /// Maximum instances
    pub max_instances: u32,
}

/// Amount to scale by
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAmount {
    Fixed(u32),
    Percentage(f64),
    Dynamic { factor: f64 },
}
