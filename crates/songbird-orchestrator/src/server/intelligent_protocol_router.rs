//! Intelligent Protocol Router
//!
//! Routes requests to optimal protocols based on workload characteristics:
//! - Data type (binary, JSON, text)
//! - Payload size (small, medium, large)
//! - Latency requirements (real-time, interactive, batch)
//! - Client capabilities (Rust-native, universal)
//! - Network conditions (LAN, WAN, internet)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Workload characteristics for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    /// Type of data being transferred
    pub data_type: DataType,

    /// Estimated payload size in bytes
    pub payload_size: PayloadSize,

    /// Latency requirements
    pub latency_requirement: LatencyRequirement,

    /// Operation type
    pub operation: OperationType,

    /// Client capabilities
    pub client_capabilities: ClientCapabilities,

    /// Network context
    pub network_context: Option<NetworkContext>,
}

/// Type of data being transferred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Binary data (models, datasets, images)
    Binary,

    /// JSON data (API calls, configuration)
    Json,

    /// Plain text (logs, messages)
    Text,

    /// Mixed or unknown
    Mixed,
}

/// Payload size category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadSize {
    /// < 1KB - Small API calls, status updates
    Tiny,

    /// 1KB - 100KB - Typical API responses
    Small,

    /// 100KB - 10MB - Medium files, batch data
    Medium,

    /// 10MB - 1GB - Large files, model weights
    Large,

    /// > 1GB - Very large datasets, full models
    Huge,
}

impl PayloadSize {
    pub fn from_bytes(bytes: u64) -> Self {
        match bytes {
            0..=1_024 => PayloadSize::Tiny,
            1_025..=102_400 => PayloadSize::Small,
            102_401..=10_485_760 => PayloadSize::Medium,
            10_485_761..=1_073_741_824 => PayloadSize::Large,
            _ => PayloadSize::Huge,
        }
    }

    pub fn to_bytes(&self) -> u64 {
        match self {
            PayloadSize::Tiny => 512,
            PayloadSize::Small => 50_000,
            PayloadSize::Medium => 5_000_000,
            PayloadSize::Large => 500_000_000,
            PayloadSize::Huge => 50_000_000_000,
        }
    }
}

/// Latency requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyRequirement {
    /// < 10ms - Real-time gaming, trading
    RealTime,

    /// < 100ms - Interactive UI, user-facing
    Interactive,

    /// < 1s - Typical API calls
    Standard,

    /// < 10s - Batch operations
    Batch,

    /// > 10s - Background processing
    Background,
}

/// Operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Read data
    Read,

    /// Write/upload data
    Write,

    /// Bidirectional streaming
    Stream,

    /// RPC/function call
    Rpc,

    /// Status/monitoring
    Status,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Is client Rust-native? (can use tarpc)
    pub rust_native: bool,

    /// Supports TLS/HTTPS?
    pub supports_tls: bool,

    /// Maximum concurrent connections
    pub max_connections: u32,

    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Network context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    /// Network type
    pub network_type: NetworkType,

    /// Estimated bandwidth (Mbps)
    pub bandwidth_mbps: Option<u32>,

    /// Estimated latency (ms)
    pub latency_ms: Option<u32>,
}

/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Local (same machine)
    Local,

    /// LAN (< 1ms latency)
    Lan,

    /// WAN (1-50ms latency)
    Wan,

    /// Internet (> 50ms latency)
    Internet,
}

/// Recommended protocol for a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRecommendation {
    /// Recommended protocol
    pub protocol: String,

    /// Confidence score (0-100)
    pub confidence: u8,

    /// Reason for recommendation
    pub reason: String,

    /// Alternative protocols (ranked)
    pub alternatives: Vec<String>,

    /// Expected performance characteristics
    pub expected_performance: ExpectedPerformance,
}

/// Expected performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    /// Expected latency (ms)
    pub latency_ms: f64,

    /// Expected throughput (MB/s)
    pub throughput_mbps: f64,

    /// Estimated completion time (seconds)
    pub completion_time_seconds: f64,
}

/// Intelligent protocol router
pub struct IntelligentProtocolRouter {
    /// Available protocols
    available_protocols: Vec<String>,

    /// Performance profiles (cached)
    performance_profiles: HashMap<String, ProtocolPerformance>,
}

/// Protocol performance characteristics (from benchmarks)
#[derive(Debug, Clone)]
struct ProtocolPerformance {
    /// Requests per second (over network)
    req_per_sec: u32,

    /// Throughput in MB/s
    throughput_mbps: f64,

    /// Base latency (ms)
    latency_ms: f64,

    /// Good for binary data?
    binary_efficient: bool,

    /// Good for JSON?
    json_efficient: bool,

    /// Requires Rust client?
    requires_rust: bool,
}

impl Default for IntelligentProtocolRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl IntelligentProtocolRouter {
    pub fn new() -> Self {
        let mut performance_profiles = HashMap::new();

        // HTTP - Universal, good for JSON
        performance_profiles.insert(
            "http".to_string(),
            ProtocolPerformance {
                req_per_sec: 4_665,     // Measured cross-tower
                throughput_mbps: 100.0, // Limited by protocol overhead
                latency_ms: 0.215,      // Measured
                binary_efficient: false,
                json_efficient: true,
                requires_rust: false,
            },
        );

        // JSON-RPC - Universal RPC
        performance_profiles.insert(
            "json-rpc".to_string(),
            ProtocolPerformance {
                req_per_sec: 3_585, // Measured cross-tower
                throughput_mbps: 80.0,
                latency_ms: 0.278, // Measured
                binary_efficient: false,
                json_efficient: true,
                requires_rust: false,
            },
        );

        // tarpc - High-performance binary
        performance_profiles.insert(
            "tarpc".to_string(),
            ProtocolPerformance {
                req_per_sec: 4_955,      // Measured cross-tower (will be much higher with 10Gb)
                throughput_mbps: 1000.0, // With 10Gb NIC
                latency_ms: 0.200,       // Network latency
                binary_efficient: true,
                json_efficient: false,
                requires_rust: true,
            },
        );

        Self {
            available_protocols: vec![
                "http".to_string(),
                "json-rpc".to_string(),
                "tarpc".to_string(),
            ],
            performance_profiles,
        }
    }

    /// Select optimal protocol for workload
    pub fn select_protocol(&self, workload: &WorkloadCharacteristics) -> ProtocolRecommendation {
        info!(
            "Selecting protocol for workload: data_type={:?}, size={:?}, latency={:?}, op={:?}",
            workload.data_type,
            workload.payload_size,
            workload.latency_requirement,
            workload.operation
        );

        // Score each available protocol
        let mut scores: Vec<(String, u8, String)> = self
            .available_protocols
            .iter()
            .filter(|p| {
                // Filter out protocols client doesn't support
                workload.client_capabilities.protocols.contains(p)
            })
            .map(|protocol| {
                let (score, reason) = self.score_protocol(protocol, workload);
                (protocol.clone(), score, reason)
            })
            .collect();

        // Sort by score (descending)
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        // Get best protocol
        let (best_protocol, confidence, reason) = scores
            .first()
            .cloned()
            .unwrap_or_else(|| ("http".to_string(), 50, "Default fallback".to_string()));

        // Get alternatives
        let alternatives: Vec<String> = scores.iter().skip(1).map(|(p, _, _)| p.clone()).collect();

        // Calculate expected performance
        let expected_performance = self.calculate_expected_performance(&best_protocol, workload);

        debug!(
            "Selected protocol: {} (confidence: {}, reason: {})",
            best_protocol, confidence, reason
        );

        ProtocolRecommendation {
            protocol: best_protocol,
            confidence,
            reason,
            alternatives,
            expected_performance,
        }
    }

    /// Score a protocol for a given workload (0-100)
    fn score_protocol(&self, protocol: &str, workload: &WorkloadCharacteristics) -> (u8, String) {
        let perf = match self.performance_profiles.get(protocol) {
            Some(p) => p,
            None => return (50, "Unknown protocol".to_string()),
        };

        let mut score: i32 = 50; // Base score
        let mut reasons = Vec::new();

        // Rule 1: Rust-native check
        if perf.requires_rust && !workload.client_capabilities.rust_native {
            return (0, "Client is not Rust-native".to_string());
        }

        // Rule 2: Binary data → prefer tarpc
        if workload.data_type == DataType::Binary {
            if perf.binary_efficient {
                score += 30;
                reasons.push("excellent for binary data");
            } else {
                score -= 20;
                reasons.push("not optimal for binary");
            }
        }

        // Rule 3: JSON data → prefer HTTP/JSON-RPC
        if workload.data_type == DataType::Json {
            if perf.json_efficient {
                score += 20;
                reasons.push("native JSON support");
            }
        }

        // Rule 4: Large payloads → prefer high throughput
        match workload.payload_size {
            PayloadSize::Huge | PayloadSize::Large => {
                if perf.throughput_mbps > 500.0 {
                    score += 30;
                    reasons.push("high throughput for large data");
                }
            }
            PayloadSize::Tiny | PayloadSize::Small => {
                if perf.req_per_sec > 4000 {
                    score += 15;
                    reasons.push("high req/s for small payloads");
                }
            }
            _ => {}
        }

        // Rule 5: Real-time latency → prefer tarpc
        match workload.latency_requirement {
            LatencyRequirement::RealTime | LatencyRequirement::Interactive => {
                if perf.latency_ms < 0.25 {
                    score += 20;
                    reasons.push("low latency");
                }
            }
            _ => {}
        }

        // Rule 6: Status/monitoring → prefer HTTP (universal)
        if workload.operation == OperationType::Status {
            if protocol == "http" {
                score += 25;
                reasons.push("universal access for monitoring");
            }
        }

        // Rule 7: RPC operations → prefer tarpc or JSON-RPC
        if workload.operation == OperationType::Rpc {
            if protocol == "tarpc" || protocol == "json-rpc" {
                score += 20;
                reasons.push("native RPC protocol");
            }
        }

        // Rule 8: Network type consideration
        if let Some(ref network) = workload.network_context {
            match network.network_type {
                NetworkType::Local | NetworkType::Lan => {
                    // On LAN, prefer high-performance protocols
                    if protocol == "tarpc" {
                        score += 15;
                        reasons.push("optimal for LAN");
                    }
                }
                NetworkType::Internet => {
                    // On internet, prefer universal protocols
                    if protocol == "http" {
                        score += 10;
                        reasons.push("universal for internet");
                    }
                }
                _ => {}
            }
        }

        // Clamp score to 0-100
        let final_score = score.clamp(0, 100) as u8;
        let reason = reasons.join(", ");

        (final_score, reason)
    }

    /// Calculate expected performance for a protocol + workload
    fn calculate_expected_performance(
        &self,
        protocol: &str,
        workload: &WorkloadCharacteristics,
    ) -> ExpectedPerformance {
        let perf =
            self.performance_profiles.get(protocol).cloned().unwrap_or(ProtocolPerformance {
                req_per_sec: 1000,
                throughput_mbps: 10.0,
                latency_ms: 10.0,
                binary_efficient: false,
                json_efficient: true,
                requires_rust: false,
            });

        let payload_bytes = workload.payload_size.to_bytes();
        let payload_mb = payload_bytes as f64 / 1_048_576.0;

        // Calculate expected latency
        let base_latency = perf.latency_ms;
        let transfer_time_ms = if payload_mb > 0.1 {
            (payload_mb / perf.throughput_mbps) * 1000.0
        } else {
            0.0
        };
        let latency_ms = base_latency + transfer_time_ms;

        // Calculate expected throughput
        let throughput_mbps = if payload_mb > 10.0 {
            perf.throughput_mbps * 0.8 // Account for overhead
        } else {
            perf.throughput_mbps * 0.5 // Small payloads have more overhead
        };

        // Calculate completion time
        let completion_time_seconds = latency_ms / 1000.0;

        ExpectedPerformance {
            latency_ms,
            throughput_mbps,
            completion_time_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_large_data_selects_tarpc() {
        let router = IntelligentProtocolRouter::new();

        let workload = WorkloadCharacteristics {
            data_type: DataType::Binary,
            payload_size: PayloadSize::Huge,
            latency_requirement: LatencyRequirement::Batch,
            operation: OperationType::Write,
            client_capabilities: ClientCapabilities {
                rust_native: true,
                supports_tls: true,
                max_connections: 10,
                protocols: vec!["http".to_string(), "tarpc".to_string()],
            },
            network_context: Some(NetworkContext {
                network_type: NetworkType::Lan,
                bandwidth_mbps: Some(1000),
                latency_ms: Some(1),
            }),
        };

        let recommendation = router.select_protocol(&workload);
        assert_eq!(recommendation.protocol, "tarpc");
        assert!(recommendation.confidence > 70);
    }

    #[test]
    fn test_json_status_selects_http() {
        let router = IntelligentProtocolRouter::new();

        let workload = WorkloadCharacteristics {
            data_type: DataType::Json,
            payload_size: PayloadSize::Small,
            latency_requirement: LatencyRequirement::Standard,
            operation: OperationType::Status,
            client_capabilities: ClientCapabilities {
                rust_native: false,
                supports_tls: true,
                max_connections: 1,
                protocols: vec!["http".to_string()],
            },
            network_context: Some(NetworkContext {
                network_type: NetworkType::Internet,
                bandwidth_mbps: Some(100),
                latency_ms: Some(50),
            }),
        };

        let recommendation = router.select_protocol(&workload);
        assert_eq!(recommendation.protocol, "http");
    }
}
