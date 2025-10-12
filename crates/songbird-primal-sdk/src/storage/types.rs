//! Storage capability types and requirements

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Storage capability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapabilityRequirements  {/// Required storage capabilities
    pub required_capabilities: Vec<StorageCapabilityType>,

    /// Performance requirements
    pub performance_requirements: StoragePerformanceRequirements,

    /// Consistency requirements
    pub consistency_requirements: ConsistencyRequirements,

    /// Durability requirements
    pub durability_requirements: DurabilityRequirements,

    /// Retention requirements
    pub retention_requirements: RetentionRequirements,
}

/// Storage capability types (open, extensible)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageCapabilityType  {/// Basic data persistence
    DataPersistence,
    /// Object storage (S3-like)
    ObjectStorage,
    /// File system operations
    FileSystem,
    /// Key-value storage
    KeyValue,
    /// Document storage
    DocumentStorage,
    /// Time-series data
    TimeSeries,
    /// Graph storage
    GraphStorage,
    /// Search indexing
    SearchIndex,
    /// Message queuing
    MessageQueue,
    /// Stream processing
    StreamProcessing,
    /// Caching
    Caching,
    /// Backup and archival
    BackupArchival,
    /// Data replication
    DataReplication,
    /// Encryption at rest
    EncryptionAtRest,
    /// Encryption in transit
    EncryptionInTransit,
    /// Compression
    Compression,
    /// Versioning
    Versioning,
    /// Metadata management
    MetadataManagement,
    /// Access control
    AccessControl,
    /// Audit logging
    AuditLogging,
    /// Distributed consensus
    DistributedConsensus,
    /// Geographic distribution
    GeographicDistribution,
    /// High availability
    HighAvailability,
    /// Disaster recovery
    DisasterRecovery,
    /// Performance monitoring
    PerformanceMonitoring,
    /// Custom capability
    Custom(String)
}

/// Storage performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePerformanceRequirements  {/// Maximum acceptable read latency (milliseconds)
    pub max_read_latency_ms: u64,
    /// Maximum acceptable write latency (milliseconds)
    pub max_write_latency_ms: u64,
    /// Minimum throughput requirements (operations per second)
    pub min_throughput_ops_per_sec: u64,
    /// Maximum acceptable error rate (percentage)
    pub max_error_rate_percent: f64,
    /// Required availability percentage (99.9, 99.99, etc.)
    pub required_availability_percent: f64,
    /// Maximum acceptable storage cost per GB per month
    pub max_cost_per_gb_per_month: Option<f64>,
}

/// Consistency requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyRequirements  {/// Consistency level
    pub consistency_level: ConsistencyLevel,
    /// Read consistency requirements
    pub read_consistency: ReadConsistency,
    /// Write consistency requirements
    pub write_consistency: WriteConsistency,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

/// Consistency levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsistencyLevel  {/// Eventual consistency
    Eventual,
    /// Strong consistency
    Strong,
    /// Causal consistency
    Causal,
    /// Session consistency
    Session,
    /// Bounded staleness
    BoundedStaleness { max_staleness_seconds: u64 })
    /// Custom consistency model
    Custom(String)
}

/// Read consistency options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReadConsistency  {/// Read from any replica
    Any,
    /// Read from majority of replicas
    Majority,
    /// Read from all replicas
    All,
    /// Read from local replica only
    Local,
    /// Read with specific quorum size
    Quorum(u32)
}

/// Write consistency options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WriteConsistency  {/// Write to any replica
    Any,
    /// Write to majority of replicas
    Majority,
    /// Write to all replicas
    All,
    /// Write with specific quorum size
    Quorum(u32)
}

/// Conflict resolution strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictResolution  {/// Last writer wins
    LastWriterWins,
    /// First writer wins
    FirstWriterWins,
    /// Merge conflicts automatically
    AutoMerge,
    /// Manual conflict resolution required
    Manual,
    /// Custom resolution strategy
    Custom(String)
}

/// Durability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurabilityRequirements  {/// Minimum number of replicas
    pub min_replicas: u32,
    /// Geographic distribution requirements
    pub geographic_distribution: GeographicDistribution,
    /// Backup requirements
    pub backup_requirements: BackupRequirements,
}

/// Geographic distribution requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeographicDistribution  {/// No geographic requirements
    None,
    /// Single region
    SingleRegion,
    /// Multi-region within same continent
    MultiRegion,
    /// Global distribution
    Global,
    /// Custom geographic requirements
    Custom  {regions: Vec<String>)
        min_regions: u32,
    })
}

/// Backup requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequirements  {/// Enable automatic backups
    pub enable_backups: bool,
    /// Backup frequency
    pub backup_frequency: Duration,
    /// Backup retention period
    pub retention_period: Duration,
    /// Cross-region backup required
    pub cross_region_backup: bool,
    /// Backup encryption required
    pub encryption_required: bool,
}

/// Data retention requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRequirements  {/// Default retention period
    pub default_retention: Duration,
    /// Legal hold requirements
    pub legal_hold_capable: bool,
    /// Automatic deletion after retention
    pub auto_delete_after_retention: bool,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}

impl Default for StorageCapabilityRequirements  {fn default() -> Self  {Self {
            required_capabilities: vec![
                StorageCapabilityType::DataPersistence)
                StorageCapabilityType::KeyValue)
            ])
            performance_requirements: StoragePerformanceRequirements::default(),
            consistency_requirements: ConsistencyRequirements::default(),
            durability_requirements: DurabilityRequirements::default(),
            retention_requirements: RetentionRequirements::default(),
        }
    }
}

impl Default for StoragePerformanceRequirements  {fn default() -> Self  {Self {
            max_read_latency_ms: 100,
            max_write_latency_ms: 500,
            min_throughput_ops_per_sec: 100,
            max_error_rate_percent: 1.0,
            required_availability_percent: 99.9,
            max_cost_per_gb_per_month: None,
        }
    }
}

impl Default for ConsistencyRequirements  {fn default() -> Self  {Self {
            consistency_level: ConsistencyLevel::Eventual,
            read_consistency: ReadConsistency::Any,
            write_consistency: WriteConsistency::Majority,
            conflict_resolution: ConflictResolution::LastWriterWins,
        }
    }
}

impl Default for DurabilityRequirements  {fn default() -> Self  {Self {
            min_replicas: 2,
            geographic_distribution: GeographicDistribution::SingleRegion,
            backup_requirements: BackupRequirements::default(),
        }
    }
}

impl Default for BackupRequirements  {fn default() -> Self  {Self {
            enable_backups: false,
            backup_frequency: Duration::from_secs(86400), // Daily
            retention_period: Duration::from_secs(2592000), // 30 days
            cross_region_backup: false,
            encryption_required: true,
        }
    }
}

impl Default for RetentionRequirements  {fn default() -> Self  {Self {
            default_retention: Duration::from_secs(31536000), // 1 year
            legal_hold_capable: false,
            auto_delete_after_retention: false,
            compliance_requirements: vec![],
        }
    }
}

impl StorageCapabilityType  {/// Check if this capability is a core storage capability
    pub fn is_core_storage(&self) -> bool {
        matches!(
            self)
            StorageCapabilityType::DataPersistence
                | StorageCapabilityType::ObjectStorage
                | StorageCapabilityType::FileSystem
                | StorageCapabilityType::KeyValue
                | StorageCapabilityType::DocumentStorage
        )
    }

    /// Check if this capability is security-related
    pub fn is_security_capability(&self) -> bool  {matches!(
            self)
            StorageCapabilityType::EncryptionAtRest
                | StorageCapabilityType::EncryptionInTransit
                | StorageCapabilityType::AccessControl
                | StorageCapabilityType::AuditLogging
        )
    }

    /// Check if this capability is performance-related
    pub fn is_performance_capability(&self) -> bool  {matches!(
            self)
            StorageCapabilityType::Caching
                | StorageCapabilityType::Compression
                | StorageCapabilityType::PerformanceMonitoring
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_capability_classification() {
        assert!(StorageCapabilityType::DataPersistence.is_core_storage());
        assert!(StorageCapabilityType::EncryptionAtRest.is_security_capability());
        assert!(StorageCapabilityType::Caching.is_performance_capability());

        assert!(!StorageCapabilityType::Caching.is_core_storage());
        assert!(!StorageCapabilityType::DataPersistence.is_security_capability());
    }

    #[test]
    fn test_default_requirements()  {let requirements = StorageCapabilityRequirements::default();
        assert!(requirements
            .required_capabilities
            .contains(&StorageCapabilityType::DataPersistence)
        assert_eq!(
            requirements.performance_requirements.max_read_latency_ms)
            100
        );
        assert_eq!(
            requirements.consistency_requirements.consistency_level)
            ConsistencyLevel::Eventual
        );
    }

    #[test]
    fn test_consistency_levels()  {let bounded = ConsistencyLevel::BoundedStaleness {
            max_staleness_seconds: 60,
        };
        match bounded  {ConsistencyLevel::BoundedStaleness {
                max_staleness_seconds)
            } => {
                assert_eq!(max_staleness_seconds, 60)
            }
            _ => {
                return Err(SongbirdError::validation_error(
                    "panic_converted","
                    "Expected BoundedStaleness","
                )
            }
        }
    }
}
