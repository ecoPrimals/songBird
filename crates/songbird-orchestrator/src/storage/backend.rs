//! Storage Backend Trait - Abstraction for Pure Rust storage
//!
//! This trait provides a unified interface for all storage operations,
//! enabling easy backend swapping between redb (production) and
//! in-memory (testing).

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;
use thiserror::Error;

/// Storage operation result type
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found: {table}/{key}")]
    NotFound { table: String, key: String },
    
    #[error("Transaction failed: {0}")]
    Transaction(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Storage backend trait - unified interface for all storage operations
///
/// ## Design Principles
///
/// 1. **Simple K/V Operations**: All operations are key-value based
/// 2. **Table Namespaces**: Logical separation via table names
/// 3. **Serde Integration**: Values are serialized/deserialized automatically
/// 4. **Async Ready**: All operations are async for compatibility
///
/// ## Tables Used
///
/// - `tasks` - Task lifecycle records
/// - `checkpoints` - Task checkpoint data
/// - `consent_records` - Consent management
/// - `auth_tokens` - Authentication tokens
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Get a value by key from a table
    async fn get<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        key: &str
    ) -> StorageResult<Option<T>>;
    
    /// Put a value with key into a table
    async fn put<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        key: &str, 
        value: &T
    ) -> StorageResult<()>;
    
    /// Delete a value by key from a table
    async fn delete(&self, table: &str, key: &str) -> StorageResult<bool>;
    
    /// Check if a key exists in a table
    async fn exists(&self, table: &str, key: &str) -> StorageResult<bool>;
    
    /// List all keys in a table
    async fn list_keys(&self, table: &str) -> StorageResult<Vec<String>>;
    
    /// List all values in a table
    async fn list_all<T: DeserializeOwned + Send>(&self, table: &str) -> StorageResult<Vec<T>>;
    
    /// Get multiple values by keys
    async fn get_many<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        keys: &[&str]
    ) -> StorageResult<Vec<Option<T>>>;
    
    /// Put multiple key-value pairs atomically
    async fn put_many<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        items: &[(&str, &T)]
    ) -> StorageResult<()>;
    
    /// Delete multiple keys atomically
    async fn delete_many(&self, table: &str, keys: &[&str]) -> StorageResult<usize>;
    
    /// Count entries in a table
    async fn count(&self, table: &str) -> StorageResult<usize>;
    
    /// Query entries by prefix (for range scans)
    async fn query_by_prefix<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        prefix: &str
    ) -> StorageResult<Vec<(String, T)>>;
    
    /// Compact/optimize the database (no-op for some backends)
    async fn compact(&self) -> StorageResult<()>;
    
    /// Flush pending writes to disk
    async fn flush(&self) -> StorageResult<()>;
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Database: {}", msg),
            Self::Serialization(msg) => write!(f, "Serialization: {}", msg),
            Self::NotFound { table, key } => write!(f, "Not found: {}/{}", table, key),
            Self::Transaction(msg) => write!(f, "Transaction: {}", msg),
            Self::Io(e) => write!(f, "IO: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_error_display() {
        let err = StorageError::NotFound { 
            table: "tasks".to_string(), 
            key: "task123".to_string() 
        };
        assert_eq!(err.to_string(), "Not found: tasks/task123");
    }
}

