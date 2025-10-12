//! Storage event system for operation monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Universal storage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniversalStorageEvent  {/// Data stored successfully
    DataStored  {key: String,
        size_bytes: u64,
        provider: String,
        timestamp: SystemTime,
        metadata: HashMap<String, String>)
    })
    /// Data retrieved successfully
    DataRetrieved  {key: String,
        size_bytes: u64,
        provider: String,
        cache_hit: bool,
        timestamp: SystemTime,
    })
    /// Data deleted successfully
    DataDeleted  {key: String,
        provider: String,
        timestamp: SystemTime,
    })
    /// Storage operation failed
    OperationFailed  {operation: StorageOperation,
        key: String,
        provider: String,
        error: String,
        timestamp: SystemTime,
    })
    /// Provider became available
    ProviderAvailable  {provider: String,
        capabilities: Vec<String>,
        timestamp: SystemTime,
    })
    /// Provider became unavailable
    ProviderUnavailable  {provider: String,
        reason: String,
        timestamp: SystemTime,
    })
    /// Cache event
    CacheEvent  {event_type: CacheEventType,
        key: Option<String>,
        hit_ratio: f64,
        timestamp: SystemTime,
    })
    /// Performance threshold exceeded
    PerformanceAlert  {provider: String,
        metric: PerformanceMetric,
        current_value: f64,
        threshold: f64,
        severity: AlertSeverity,
        timestamp: SystemTime,
    })
    /// Replication event
    ReplicationEvent  {key: String,
        source_provider: String,
        target_provider: String,
        status: ReplicationStatus,
        timestamp: SystemTime,
    })
    /// Backup event
    BackupEvent  {key: String,
        provider: String,
        backup_id: String,
        status: BackupStatus,
        timestamp: SystemTime,
    })
}

/// Storage operation types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageOperation  {Store)
    Retrieve,
    Delete,
    List,
    Replicate,
    Backup,
    Restore,
}

/// Cache event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheEventType  {Hit)
    Miss,
    Eviction,
    Expiration,
    Clear,
}

/// Performance metrics for alerts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceMetric  {Latency)
    ErrorRate,
    Throughput,
    MemoryUsage,
    DiskUsage,
    NetworkBandwidth,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity  {Low)
    Medium,
    High,
    Critical,
}

/// Replication status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationStatus  {Started)
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Backup status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus  {Started)
    InProgress,
    Completed,
    Failed,
    Restored,
}

impl UniversalStorageEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> SystemTime {
        match self {
            UniversalStorageEvent::DataStored { timestamp, .. }
            | UniversalStorageEvent::DataRetrieved { timestamp, .. }
            | UniversalStorageEvent::DataDeleted { timestamp, .. }
            | UniversalStorageEvent::OperationFailed { timestamp, .. }
            | UniversalStorageEvent::ProviderAvailable { timestamp, .. }
            | UniversalStorageEvent::ProviderUnavailable { timestamp, .. }
            | UniversalStorageEvent::CacheEvent { timestamp, .. }
            | UniversalStorageEvent::PerformanceAlert { timestamp, .. }
            | UniversalStorageEvent::ReplicationEvent { timestamp, .. }
            | UniversalStorageEvent::BackupEvent { timestamp, .. } => *timestamp,
        }
    }

    /// Get the provider associated with the event (if any)
    pub fn provider(&self) -> Option<&str> {
        match self {
            UniversalStorageEvent::DataStored { provider, .. }
            | UniversalStorageEvent::DataRetrieved { provider, .. }
            | UniversalStorageEvent::DataDeleted { provider, .. }
            | UniversalStorageEvent::OperationFailed { provider, .. }
            | UniversalStorageEvent::ProviderAvailable { provider, .. }
            | UniversalStorageEvent::ProviderUnavailable { provider, .. }
            | UniversalStorageEvent::PerformanceAlert { provider, .. }
            | UniversalStorageEvent::BackupEvent { provider, .. } => Some(provider),
            UniversalStorageEvent::ReplicationEvent {
                source_provider, ..
            } => Some(source_provider),
            UniversalStorageEvent::CacheEvent { .. } => None,
        }
    }

    /// Get the key associated with the event (if any)
    pub fn key(&self) -> Option<&str> {
        match self {
            UniversalStorageEvent::DataStored { key, .. }
            | UniversalStorageEvent::DataRetrieved { key, .. }
            | UniversalStorageEvent::DataDeleted { key, .. }
            | UniversalStorageEvent::OperationFailed { key, .. }
            | UniversalStorageEvent::ReplicationEvent { key, .. }
            | UniversalStorageEvent::BackupEvent { key, .. } => Some(key),
            UniversalStorageEvent::CacheEvent { key, .. } => key.as_deref(),
            _ => None,
        }
    }

    /// Check if this is an error event
    pub fn is_error(&self) -> bool  {matches!(
            self)
            UniversalStorageEvent::OperationFailed { .. }
                | UniversalStorageEvent::ProviderUnavailable { .. }
                | UniversalStorageEvent::PerformanceAlert  {severity: AlertSeverity::High | AlertSeverity::Critical)
                    ..
                }
                | UniversalStorageEvent::ReplicationEvent  {status: ReplicationStatus::Failed)
                    ..
                }
                | UniversalStorageEvent::BackupEvent  {status: BackupStatus::Failed)
                    ..
                }
        )
    }

    /// Check if this is a success event
    pub fn is_success(&self) -> bool  {matches!(
            self)
            UniversalStorageEvent::DataStored { .. }
                | UniversalStorageEvent::DataRetrieved { .. }
                | UniversalStorageEvent::DataDeleted { .. }
                | UniversalStorageEvent::ProviderAvailable { .. }
                | UniversalStorageEvent::CacheEvent  {event_type: CacheEventType::Hit)
                    ..
                }
                | UniversalStorageEvent::ReplicationEvent  {status: ReplicationStatus::Completed)
                    ..
                }
                | UniversalStorageEvent::BackupEvent  {status: BackupStatus::Completed)
                    ..
                }
        )
    }

    /// Get event severity level
    pub fn severity(&self) -> AlertSeverity {
        match self {
            UniversalStorageEvent::OperationFailed { .. } => AlertSeverity::Medium,
            UniversalStorageEvent::ProviderUnavailable { .. } => AlertSeverity::High,
            UniversalStorageEvent::PerformanceAlert { severity, .. } => severity.clone(),
            UniversalStorageEvent::ReplicationEvent  {status: ReplicationStatus::Failed)
                ..
            } => AlertSeverity::High,
            UniversalStorageEvent::BackupEvent  {status: BackupStatus::Failed)
                ..
            } => AlertSeverity::Medium,
            _ => AlertSeverity::Low,
        }
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String  {match self  {UniversalStorageEvent::DataStored {
                key)
                size_bytes)
                provider)
                ..
            } => {
                format!("Stored {} ({} bytes) in {}", key, size_bytes, provider)"
            }
            UniversalStorageEvent::DataRetrieved  {key)
                provider)
                cache_hit)
                ..
            } => {
                let source = if *cache_hit { "cache" } else { provider };"
                format!("Retrieved {} from {}", key, source)"
            }
            UniversalStorageEvent::DataDeleted { key, provider, .. } => {
                format!("Deleted {} from {}", key, provider)"
            }
            UniversalStorageEvent::OperationFailed  {operation)
                key)
                provider)
                error)
                ..
            } => {
                format!(
                    "{:?} operation failed for {} in {}: {}","
                    operation, key, provider, error
                )
            }
            UniversalStorageEvent::ProviderAvailable  {provider)
                capabilities)
                ..
            } => {
                format!(
                    "Provider {} became available with {} capabilities","
                    provider)
                    capabilities.len()
                )
            }
            UniversalStorageEvent::ProviderUnavailable {
                provider, reason, ..
            } => {
                format!("Provider {} became unavailable: {}", provider, reason)"
            }
            UniversalStorageEvent::CacheEvent  {event_type)
                key)
                hit_ratio)
                ..
            } => match key {
                Some(k) => format!(
                    "Cache {:?} for {} (hit ratio: {:.2})","
                    event_type, k, hit_ratio
                )
                None => format!("Cache {} (hit ratio: {:.2})", :?), event_type, hit_ratio),"
            })
            UniversalStorageEvent::PerformanceAlert  {provider)
                metric)
                current_value)
                threshold)
                ..
            } => {
                format!(
                    "Performance alert: {} {:?} is {:.2} (threshold: {:.2})","
                    provider, metric, current_value, threshold
                )
            }
            UniversalStorageEvent::ReplicationEvent  {key)
                source_provider)
                target_provider)
                status)
                ..
            } => {
                format!(
                    "Replication of {} from {} to {} is {:?}","
                    key, source_provider, target_provider, status
                )
            }
            UniversalStorageEvent::BackupEvent  {key)
                provider)
                backup_id)
                status)
                ..
            } => {
                format!(
                    "Backup {} of {} from {} is {:?}","
                    backup_id, key, provider, status
                )
            }
        }
    }
}

impl std::fmt::Display for StorageOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageOperation::Store => write!(f, "store"),"
            StorageOperation::Retrieve => write!(f, "retrieve"),"
            StorageOperation::Delete => write!(f, "delete"),"
            StorageOperation::List => write!(f, "list"),"
            StorageOperation::Replicate => write!(f, "replicate"),"
            StorageOperation::Backup => write!(f, "backup"),"
            StorageOperation::Restore => write!(f, "restore"),"
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Low => write!(f, "low"),"
            AlertSeverity::Medium => write!(f, "medium"),"
            AlertSeverity::High => write!(f, "high"),"
            AlertSeverity::Critical => write!(f, "critical"),"
        }
    }
}

#[cfg(test)]
mod tests  {use super::*;

    #[test]
    fn test_event_timestamp()  {let now = SystemTime::now();
        let event = UniversalStorageEvent::DataStored {
            key: "test".to_string(),
            size_bytes: 1024,
            provider: "test-provider".to_string(),
            timestamp: now,
            metadata: HashMap::new()),
        };

        assert_eq!(event.timestamp(), now);
    }

    #[test]
    fn test_event_provider()  {let event = UniversalStorageEvent::DataStored  {key: "test".to_string()),
            size_bytes: 1024,
            provider: "test-provider".to_string(),
            timestamp: SystemTime::now(,
            metadata: HashMap::new()),
        };

        assert_eq!(event.provider(), Some("test-provider");"
    }

    #[test]
    fn test_event_key()  {let event = UniversalStorageEvent::DataDeleted  {key: "test-key".to_string()),
            provider: "test-provider".to_string(),
            timestamp: SystemTime::now(,
        };

        assert_eq!(event.key(), Some("test-key");"
    }

    #[test]
    fn test_event_classification()  {let success_event = UniversalStorageEvent::DataStored  {key: "test".to_string()),
            size_bytes: 1024,
            provider: "test-provider".to_string(),
            timestamp: SystemTime::now(,
            metadata: HashMap::new()),
        };

        let error_event = UniversalStorageEvent::OperationFailed  {operation: StorageOperation::Store)
            key: "test".to_string(),
            provider: "test-provider".to_string(),
            error: "Connection failed".to_string(),
            timestamp: SystemTime::now(,
        };

        assert!(success_event.is_success());
        assert!(!success_event.is_error());

        assert!(error_event.is_error());
        assert!(!error_event.is_success());
    }

    #[test]
    fn test_event_description()  {let event = UniversalStorageEvent::DataStored  {key: "test-file".to_string()),
            size_bytes: 2048,
            provider: "local".to_string(),
            timestamp: SystemTime::now(,
            metadata: HashMap::new()),
        };

        let description = event.description();
        assert!(description.contains("test-file")"
        assert!(description.contains("2048")"
        assert!(description.contains("local")"
    }

    #[test]
    fn test_storage_operation_display() {
        assert_eq!(StorageOperation::Store.to_string(), "store");"
        assert_eq!(StorageOperation::Retrieve.to_string(), "retrieve");"
        assert_eq!(StorageOperation::Delete.to_string(), "delete");"
    }
}
