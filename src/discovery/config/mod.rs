use serde::{Deserialize, Serialize};
use crate::discovery::types::NodeType;

/// Configuration for Songbird Discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdDiscoveryConfig {
    pub node_id: Option<String>,
    pub node_type: NodeType,
    pub institution: Option<String>,
    pub federation_enabled: bool,
    pub health_check_interval_secs: u64,
    pub node_discovery_interval_secs: u64,
    pub trust_verification_enabled: bool,
    pub max_federation_nodes: usize,
    // Network configuration
    pub network: NetworkConfig,
    // Resource monitoring configuration
    pub monitoring: MonitoringConfig,
    // Trust calculation configuration
    pub trust: TrustConfig,
}

/// Network configuration for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub multicast_address: String,
    pub federation_port: u16,
    pub service_port: u16,
    pub bind_address: String,
    pub announcement_interval_secs: u64,
    pub response_timeout_secs: u64,
    pub ping_timeout_secs: u64,
    pub max_packet_size: usize,
    pub default_bandwidth_mbps: f64,
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub resource_update_interval_secs: u64,
    pub network_stats_window_secs: u64,
    pub storage_stats_window_secs: u64,
    pub process_scan_enabled: bool,
    pub gpu_monitoring_enabled: bool,
    pub detailed_cpu_monitoring: bool,
}

/// Trust calculation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    pub institutional_base_score: u32,
    pub edu_domain_bonus: u32,
    pub gov_domain_bonus: u32,
    pub reputation_weight: f64,
    pub uptime_weight: u32,
    pub service_diversity_weight: u32,
    pub trust_thresholds: TrustThresholds,
    pub interaction_penalties: InteractionPenalties,
}

/// Trust level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustThresholds {
    pub basic: u32,
    pub verified: u32,
    pub institutional: u32,
    pub consortium: u32,
}

/// Interaction result penalties/bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPenalties {
    pub success_bonus: f64,
    pub slow_response_penalty: f64,
    pub failure_penalty: f64,
    pub timeout_penalty: f64,
    pub malicious_penalty: f64,
}

/// Network timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimingConfig {
    pub same_subnet_latency_ms: f64,
    pub same_region_latency_ms: f64,
    pub cross_region_latency_ms: f64,
    pub cross_continental_latency_ms: f64,
    pub health_timeout_multiplier: i64,
    pub partition_detection_timeout_secs: i64,
}

// Default implementations
impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            multicast_address: "224.0.0.251".to_string(),
            federation_port: 8765,
            service_port: 8080,
            bind_address: "0.0.0.0".to_string(),
            announcement_interval_secs: 60,
            response_timeout_secs: 2,
            ping_timeout_secs: 5,
            max_packet_size: 65536,
            default_bandwidth_mbps: 1000.0,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            resource_update_interval_secs: 15,
            network_stats_window_secs: 3600,
            storage_stats_window_secs: 3600,
            process_scan_enabled: true,
            gpu_monitoring_enabled: true,
            detailed_cpu_monitoring: true,
        }
    }
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            institutional_base_score: 40,
            edu_domain_bonus: 30,
            gov_domain_bonus: 35,
            reputation_weight: 20.0,
            uptime_weight: 20,
            service_diversity_weight: 20,
            trust_thresholds: TrustThresholds::default(),
            interaction_penalties: InteractionPenalties::default(),
        }
    }
}

impl Default for TrustThresholds {
    fn default() -> Self {
        Self {
            basic: 20,
            verified: 40,
            institutional: 60,
            consortium: 80,
        }
    }
}

impl Default for InteractionPenalties {
    fn default() -> Self {
        Self {
            success_bonus: 0.01,
            slow_response_penalty: -0.005,
            failure_penalty: -0.02,
            timeout_penalty: -0.03,
            malicious_penalty: -0.1,
        }
    }
}

impl Default for NetworkTimingConfig {
    fn default() -> Self {
        Self {
            same_subnet_latency_ms: 5.0,
            same_region_latency_ms: 15.0,
            cross_region_latency_ms: 30.0,
            cross_continental_latency_ms: 100.0,
            health_timeout_multiplier: 3,
            partition_detection_timeout_secs: 300,
        }
    }
}

impl Default for SongbirdDiscoveryConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            node_type: NodeType::Orchestrator,
            institution: None,
            federation_enabled: false,
            health_check_interval_secs: 30,
            node_discovery_interval_secs: 60,
            trust_verification_enabled: true,
            max_federation_nodes: 1000,
            network: NetworkConfig::default(),
            monitoring: MonitoringConfig::default(),
            trust: TrustConfig::default(),
        }
    }
} 