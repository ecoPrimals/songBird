//! Core types and enums for AI workload classification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of workloads that can be classified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Real-time interactive requests requiring immediate response
    RealTimeInteractive {
        /// Expected response time in milliseconds
        expected_response_ms: f64,
        /// User interaction pattern
        interaction_pattern: String,
    },

    /// Batch processing workloads that can be queued
    BatchProcessing {
        /// Batch size estimate
        batch_size: u32,
        /// Processing priority level
        priority_level: BatchPriority,
    },

    /// AI/ML computation workloads
    AIComputation {
        /// Type of AI computation
        computation_type: AIComputationType,
        /// Model complexity level
        complexity_level: ComplexityLevel,
    },

    /// Stream processing workloads
    StreamProcessing {
        /// Expected throughput (events per second)
        expected_throughput_eps: f64,
        /// Stream processing pattern
        processing_pattern: String,
    },

    /// CRUD operations (Create, Read, Update, Delete)
    CrudOperation {
        /// Operation type
        operation_type: CrudOperationType,
        /// Data size estimate
        data_size_bytes: u64,
    },

    /// Analytics and reporting workloads
    Analytics {
        /// Analytics type
        analytics_type: AnalyticsType,
        /// Data processing scope
        scope: AnalyticsScope,
    },

    /// File operation workloads
    FileOperation {
        /// Operation type
        operation_type: FileOperationType,
        /// File information
        file_info: FileInfo,
    },

    /// Security operation workloads
    SecurityOperation {
        /// Operation type
        operation_type: SecurityOperationType,
        /// Security level required
        security_level: SecurityLevel,
    },

    /// Network operation workloads
    NetworkOperation {
        /// Operation type
        operation_type: NetworkOperationType,
        /// Network requirements
        network_requirements: NetworkRequirements,
    },

    /// Unknown or unclassified workloads
    Unknown {
        /// Classification hints
        hints: Vec<String>,
        /// Confidence in unknown classification
        unknown_confidence: f64,
    },
}

/// Batch processing priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Types of AI computations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIComputationType {
    Inference,
    Training,
    FineTuning,
    FeatureExtraction,
    Embedding,
    Classification,
    Regression,
    Clustering,
}

/// Complexity levels for AI computations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    ExtraHigh,
}

/// CRUD operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrudOperationType {
    Create,
    Read,
    ReadMany,
    Update,
    UpdateMany,
    Delete,
    DeleteMany,
    BulkInsert,
    Search,
    Aggregate,
}

/// Analytics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsType {
    RealTimeAnalytics,
    BatchAnalytics,
    Reporting,
    DataMining,
    MachineLearning,
}

/// Analytics processing scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsScope {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// File operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperationType {
    Read,
    Write,
    Append,
    Delete,
    Copy,
    Move,
    Compress,
    Decompress,
    Index,
}

/// File information for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub size_bytes: u64,
    pub file_type: String,
    pub compression_ratio: Option<f64>,
    pub is_binary: bool,
    pub encoding: Option<String>,
}

/// Security operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperationType {
    Authentication,
    Authorization,
    Encryption,
    Decryption,
    Signing,
    Verification,
    AuditLogging,
    ThreatDetection,
    Compliance,
    KeyGeneration,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Basic,
    Standard,
    High,
    Critical,
}

/// Network operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperationType {
    HttpRequest,
    HttpsRequest,
    WebSocketMessage,
    TcpConnection,
    UdpMessage,
    FileTransfer,
    Streaming,
    P2PMessage,
    DatabaseConnection,
    CacheAccess,
}

/// Network requirements for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirements {
    pub bandwidth_mbps: f64,
    pub latency_ms: f64,
    pub reliability: f64,
    pub connection_count: u32,
    pub data_transfer_gb: f64,
    pub protocol: String,
    pub encryption_required: bool,
    pub qos_class: String,
}
