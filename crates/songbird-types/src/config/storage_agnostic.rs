//! Vendor-Agnostic Storage Configuration
//!
//! This module provides capability-based storage configuration that works with
//! ANY storage backend without hardcoding vendor names.
//!
//! # Philosophy
//!
//! Instead of hardcoding vendor names like "postgres", "redis", "mongodb",
//! we use capability-based abstractions:
//! - Persistence capabilities (durable, transient, ephemeral)
//! - Access patterns (key-value, document, relational, graph)
//! - Performance characteristics (latency, throughput, consistency)
//!
//! # Examples
//!
//! ```rust
//! use songbird_types::config::storage::{CanonicalStorageConfig, StorageCapabilities};
//!
//! // Vendor-agnostic: Define what you need, not what you use
//! let config = CanonicalStorageConfig {
//!     enabled: true,
//!     capabilities: StorageCapabilities {
//!         persistence: PersistenceLevel::Durable,
//!         access_pattern: AccessPattern::KeyValue,
//!         consistency: ConsistencyLevel::Strong,
//!     },
//!     // Optional: Provide implementation hint if you have preference
//!     implementation_hint: Some("redis".to_string()),
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Persistence durability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistenceLevel {
    /// In-memory only, lost on restart (fastest)
    Ephemeral,
    /// Persisted but may be lost on crash
    Transient,
    /// Fully durable, survives crashes and restarts
    Durable,
}

/// Storage access pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessPattern {
    /// Key-value access (Redis, Memcached, etc.)
    KeyValue,
    /// Document-oriented (MongoDB, CouchDB, etc.)
    Document,
    /// Relational/SQL (Postgres, MySQL, etc.)
    Relational,
    /// Graph-based (Neo4j, etc.)
    Graph,
    /// Time-series (InfluxDB, TimescaleDB, etc.)
    TimeSeries,
    /// Blob/Object storage (S3, MinIO, etc.)
    Blob,
}

/// Consistency level requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Eventually consistent (highest performance)
    Eventual,
    /// Read-your-writes consistency
    ReadYourWrites,
    /// Strong consistency (ACID)
    Strong,
}

/// Storage capability requirements (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapabilities {
    /// Required persistence level
    pub persistence: PersistenceLevel,
    /// Required access pattern
    pub access_pattern: AccessPattern,
    /// Required consistency level
    pub consistency: ConsistencyLevel,
    /// Optional: Maximum acceptable latency (milliseconds)
    pub max_latency_ms: Option<u64>,
    /// Optional: Minimum required throughput (ops/sec)
    pub min_throughput: Option<u64>,
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            persistence: PersistenceLevel::Transient,
            access_pattern: AccessPattern::KeyValue,
            consistency: ConsistencyLevel::ReadYourWrites,
            max_latency_ms: Some(100),
            min_throughput: Some(1000),
        }
    }
}

/// **CANONICAL**: Vendor-Agnostic Storage Configuration
///
/// Defines storage requirements by capabilities, not by vendor names.
/// The system discovers appropriate storage backends at runtime based on
/// capabilities and availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalStorageConfig {
    /// Enable storage features
    pub enabled: bool,

    /// Storage capabilities (vendor-agnostic requirements)
    pub capabilities: StorageCapabilities,

    /// Optional implementation hint (not a requirement)
    ///
    /// If provided, the system will prefer this implementation if available
    /// and it meets the capability requirements. If not available or doesn't
    /// meet requirements, will select another implementation.
    ///
    /// Examples: "memory", "file", "distributed", "cloud"
    /// Note: NOT vendor names like "redis" or "postgres"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_hint: Option<String>,

    /// Connection endpoint (discovered at runtime if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

impl Default for CanonicalStorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities::default(),
            implementation_hint: Some("memory".to_string()),
            endpoint: None,
        }
    }
}

impl CanonicalStorageConfig {
    /// Create configuration for in-memory storage
    ///
    /// Fast, ephemeral storage for caching and session data.
    pub fn memory() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Ephemeral,
                access_pattern: AccessPattern::KeyValue,
                consistency: ConsistencyLevel::Strong,
                max_latency_ms: Some(10),
                min_throughput: Some(100_000),
            },
            implementation_hint: Some("memory".to_string()),
            endpoint: None,
        }
    }

    /// Create configuration for durable key-value storage
    ///
    /// Suitable for caching, session storage, rate limiting, etc.
    /// Will work with Redis, KeyDB, Valkey, or similar.
    pub fn durable_key_value() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Durable,
                access_pattern: AccessPattern::KeyValue,
                consistency: ConsistencyLevel::Strong,
                max_latency_ms: Some(50),
                min_throughput: Some(10_000),
            },
            implementation_hint: Some("distributed".to_string()),
            endpoint: None,
        }
    }

    /// Create configuration for relational database
    ///
    /// Suitable for structured data with ACID requirements.
    /// Will work with Postgres, MySQL, SQLite, or similar.
    pub fn relational() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Durable,
                access_pattern: AccessPattern::Relational,
                consistency: ConsistencyLevel::Strong,
                max_latency_ms: Some(100),
                min_throughput: Some(1_000),
            },
            implementation_hint: Some("distributed".to_string()),
            endpoint: None,
        }
    }

    /// Create configuration for document storage
    ///
    /// Suitable for flexible schemas and JSON data.
    /// Will work with MongoDB, CouchDB, RethinkDB, or similar.
    pub fn document() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Durable,
                access_pattern: AccessPattern::Document,
                consistency: ConsistencyLevel::Eventual,
                max_latency_ms: Some(200),
                min_throughput: Some(5_000),
            },
            implementation_hint: Some("distributed".to_string()),
            endpoint: None,
        }
    }

    /// Create configuration for blob/object storage
    ///
    /// Suitable for large files, images, videos, etc.
    /// Will work with S3, MinIO, SeaweedFS, or similar.
    pub fn blob() -> Self {
        Self {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Durable,
                access_pattern: AccessPattern::Blob,
                consistency: ConsistencyLevel::Eventual,
                max_latency_ms: Some(1000),
                min_throughput: Some(100),
            },
            implementation_hint: Some("cloud".to_string()),
            endpoint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SongbirdError;

    #[test]
    fn test_default_storage_config() {
        let config = CanonicalStorageConfig::default();
        assert!(config.enabled);
        assert_eq!(config.capabilities.persistence, PersistenceLevel::Transient);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::KeyValue);
    }

    #[test]
    fn test_memory_storage_config() {
        let config = CanonicalStorageConfig::memory();
        assert!(config.enabled);
        assert_eq!(config.capabilities.persistence, PersistenceLevel::Ephemeral);
        assert_eq!(config.implementation_hint, Some("memory".to_string()));
    }

    #[test]
    fn test_relational_storage_config() {
        let config = CanonicalStorageConfig::relational();
        assert!(config.enabled);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::Relational);
        assert_eq!(config.capabilities.consistency, ConsistencyLevel::Strong);
    }

    #[test]
    fn test_document_storage_config() {
        let config = CanonicalStorageConfig::document();
        assert!(config.enabled);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::Document);
        assert_eq!(config.capabilities.consistency, ConsistencyLevel::Eventual);
    }

    #[test]
    fn test_blob_storage_config() {
        let config = CanonicalStorageConfig::blob();
        assert!(config.enabled);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::Blob);
    }

    #[test]
    fn test_storage_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalStorageConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        assert!(json.contains("enabled"));
        assert!(json.contains("capabilities"));
        Ok(())
    }

    #[test]
    fn test_storage_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "enabled": true,
            "capabilities": {
                "persistence": "Durable",
                "access_pattern": "KeyValue",
                "consistency": "Strong"
            }
        }"#;

        let config: CanonicalStorageConfig =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;

        assert!(config.enabled);
        assert_eq!(config.capabilities.persistence, PersistenceLevel::Durable);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::KeyValue);
        assert_eq!(config.capabilities.consistency, ConsistencyLevel::Strong);
        Ok(())
    }

    #[test]
    fn test_custom_capabilities() {
        let config = CanonicalStorageConfig {
            enabled: true,
            capabilities: StorageCapabilities {
                persistence: PersistenceLevel::Durable,
                access_pattern: AccessPattern::Graph,
                consistency: ConsistencyLevel::Strong,
                max_latency_ms: Some(50),
                min_throughput: Some(5000),
            },
            implementation_hint: None,
            endpoint: Some("discovered://storage-service".to_string()),
        };

        assert!(config.enabled);
        assert_eq!(config.capabilities.access_pattern, AccessPattern::Graph);
        assert_eq!(config.endpoint, Some("discovered://storage-service".to_string()));
    }

    #[test]
    fn test_storage_config_clone() {
        let config1 = CanonicalStorageConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.capabilities.persistence, config2.capabilities.persistence);
    }

    #[test]
    fn test_storage_config_debug() {
        let config = CanonicalStorageConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalStorageConfig"));
        assert!(debug_str.contains("capabilities"));
    }

    #[test]
    fn test_no_vendor_names_in_config() {
        // Ensure we're not hardcoding vendor names
        let config = CanonicalStorageConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        // These vendor names should NOT appear
        assert!(!json.contains("postgres"));
        assert!(!json.contains("redis"));
        assert!(!json.contains("mongodb"));
        assert!(!json.contains("mysql"));

        // Only generic hints should appear
        assert!(json.contains("memory") || json.contains("capabilities"));
    }
}
