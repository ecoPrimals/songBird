//! # Storage Adapter Types
//!
//! Common types and structures for storage operations.

use serde::{Deserialize, Serialize};

/// Storage operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    Store { key: String, data: Vec<u8> },
    Retrieve { key: String },
    Delete { key: String },
    List { prefix: Option<String> },
    Exists { key: String },
}

/// Storage operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageResult {
    Stored { key: String, size: u64 },
    Retrieved { key: String, data: Vec<u8> },
    Deleted { key: String },
    Listed { keys: Vec<String> },
    Exists { key: String, exists: bool },
}

/// Storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub key: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: String,
    pub endpoint: Option<String>,
    pub bucket: Option<String>,
    pub prefix: Option<String>,
    pub timeout_ms: Option<u64>,
    pub retry_attempts: Option<u32>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            provider: "universal".to_string(),
            endpoint: None,
            bucket: None,
            prefix: None,
            timeout_ms: Some(30000), // 30 seconds
            retry_attempts: Some(3),
        }
    }
}
