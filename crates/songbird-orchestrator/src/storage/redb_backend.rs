//! Redb Backend - 100% Pure Rust embedded database
//!
//! This is the production storage backend using redb,
//! a Pure Rust ACID-compliant embedded database.
//!
//! ## Features
//!
//! - 100% Pure Rust (no C dependencies!)
//! - ACID transactions
//! - B-tree based storage
//! - Crash-safe durability
//! - Efficient range queries

use super::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use redb::{Database, ReadableTable, ReadableDatabase, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tracing::{debug, trace};

/// Table definition for generic key-value storage
/// Key: String (table:key composite), Value: bytes
const MAIN_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("songbird_data");

/// Redb-backed storage (100% Pure Rust)
pub struct RedbBackend {
    db: Database,
}

impl RedbBackend {
    /// Create a new redb backend at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db = Database::create(path)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // Initialize the main table
        let write_txn = db.begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        {
            let _ = write_txn.open_table(MAIN_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        debug!("Redb backend initialized");
        Ok(Self { db })
    }
    
    /// Construct composite key from table and key
    fn composite_key(table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }
    
    /// Extract original key from composite key
    fn extract_key(composite: &str, table: &str) -> Option<String> {
        composite.strip_prefix(&format!("{}:", table))
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl StorageBackend for RedbBackend {
    async fn get<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        key: &str
    ) -> StorageResult<Option<T>> {
        let composite = Self::composite_key(table, key);
        trace!("GET {}/{}", table, key);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        match table_handle.get(composite.as_str())
            .map_err(|e| StorageError::Database(e.to_string()))? 
        {
            Some(value) => {
                let bytes = value.value();
                let item: T = serde_json::from_slice(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(item))
            }
            None => Ok(None)
        }
    }
    
    async fn put<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        key: &str, 
        value: &T
    ) -> StorageResult<()> {
        let composite = Self::composite_key(table, key);
        trace!("PUT {}/{}", table, key);
        
        let bytes = serde_json::to_vec(value)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        {
            let mut table_handle = write_txn.open_table(MAIN_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            table_handle.insert(composite.as_str(), bytes.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        write_txn.commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete(&self, table: &str, key: &str) -> StorageResult<bool> {
        let composite = Self::composite_key(table, key);
        trace!("DELETE {}/{}", table, key);
        
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let deleted = {
            let mut table_handle = write_txn.open_table(MAIN_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            let result = table_handle.remove(composite.as_str())
                .map_err(|e| StorageError::Database(e.to_string()))?;
            result.is_some()
        };
        write_txn.commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(deleted)
    }
    
    async fn exists(&self, table: &str, key: &str) -> StorageResult<bool> {
        let composite = Self::composite_key(table, key);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(table_handle.get(composite.as_str())
            .map_err(|e| StorageError::Database(e.to_string()))?
            .is_some())
    }
    
    async fn list_keys(&self, table: &str) -> StorageResult<Vec<String>> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let mut keys = Vec::new();
        for entry in table_handle.iter()
            .map_err(|e| StorageError::Database(e.to_string()))? 
        {
            let (key, _) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let key_str = key.value();
            if let Some(original_key) = Self::extract_key(key_str, table) {
                keys.push(original_key);
            }
        }
        
        Ok(keys)
    }
    
    async fn list_all<T: DeserializeOwned + Send>(&self, table: &str) -> StorageResult<Vec<T>> {
        let prefix = format!("{}:", table);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let mut items = Vec::new();
        for entry in table_handle.iter()
            .map_err(|e| StorageError::Database(e.to_string()))? 
        {
            let (key, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let key_str = key.value();
            if key_str.starts_with(&prefix) {
                let item: T = serde_json::from_slice(value.value())
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                items.push(item);
            }
        }
        
        Ok(items)
    }
    
    async fn get_many<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        keys: &[&str]
    ) -> StorageResult<Vec<Option<T>>> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            let composite = Self::composite_key(table, key);
            match table_handle.get(composite.as_str())
                .map_err(|e| StorageError::Database(e.to_string()))? 
            {
                Some(value) => {
                    let item: T = serde_json::from_slice(value.value())
                        .map_err(|e| StorageError::Serialization(e.to_string()))?;
                    results.push(Some(item));
                }
                None => results.push(None)
            }
        }
        
        Ok(results)
    }
    
    async fn put_many<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        items: &[(&str, &T)]
    ) -> StorageResult<()> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        {
            let mut table_handle = write_txn.open_table(MAIN_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            
            for (key, value) in items {
                let composite = Self::composite_key(table, key);
                let bytes = serde_json::to_vec(value)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                table_handle.insert(composite.as_str(), bytes.as_slice())
                    .map_err(|e| StorageError::Database(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete_many(&self, table: &str, keys: &[&str]) -> StorageResult<usize> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let mut deleted = 0;
        {
            let mut table_handle = write_txn.open_table(MAIN_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            
            for key in keys {
                let composite = Self::composite_key(table, key);
                if table_handle.remove(composite.as_str())
                    .map_err(|e| StorageError::Database(e.to_string()))?
                    .is_some() 
                {
                    deleted += 1;
                }
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(deleted)
    }
    
    async fn count(&self, table: &str) -> StorageResult<usize> {
        let prefix = format!("{}:", table);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let count = table_handle.iter()
            .map_err(|e| StorageError::Database(e.to_string()))?
            .filter_map(|entry| entry.ok())
            .filter(|(key, _)| key.value().starts_with(&prefix))
            .count();
        
        Ok(count)
    }
    
    async fn query_by_prefix<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        prefix: &str
    ) -> StorageResult<Vec<(String, T)>> {
        let full_prefix = format!("{}:{}", table, prefix);
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        let table_handle = read_txn.open_table(MAIN_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let mut results = Vec::new();
        for entry in table_handle.iter()
            .map_err(|e| StorageError::Database(e.to_string()))? 
        {
            let (key, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let key_str = key.value();
            if key_str.starts_with(&full_prefix) {
                if let Some(original_key) = Self::extract_key(key_str, table) {
                    let item: T = serde_json::from_slice(value.value())
                        .map_err(|e| StorageError::Serialization(e.to_string()))?;
                    results.push((original_key, item));
                }
            }
        }
        
        Ok(results)
    }
    
    async fn compact(&self) -> StorageResult<()> {
        // Note: redb handles compaction internally during commits.
        // Manual compaction requires &mut self, but the trait uses &self.
        // For production, consider periodic manual compaction via a separate method.
        debug!("Compact requested (redb handles internally)");
        Ok(())
    }
    
    async fn flush(&self) -> StorageResult<()> {
        // Redb automatically flushes on commit, no-op here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: String,
        name: String,
        value: i32,
    }
    
    #[tokio::test]
    async fn test_basic_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");
        let backend = RedbBackend::new(&db_path).unwrap();
        
        let record = TestRecord {
            id: "test1".to_string(),
            name: "Test Record".to_string(),
            value: 42,
        };
        
        // Put
        backend.put("tests", "test1", &record).await.unwrap();
        
        // Get
        let retrieved: Option<TestRecord> = backend.get("tests", "test1").await.unwrap();
        assert_eq!(retrieved, Some(record.clone()));
        
        // Exists
        assert!(backend.exists("tests", "test1").await.unwrap());
        assert!(!backend.exists("tests", "nonexistent").await.unwrap());
        
        // Delete
        assert!(backend.delete("tests", "test1").await.unwrap());
        assert!(!backend.exists("tests", "test1").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_list_operations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_list.redb");
        let backend = RedbBackend::new(&db_path).unwrap();
        
        // Insert multiple records
        for i in 0..5 {
            let record = TestRecord {
                id: format!("id{}", i),
                name: format!("Record {}", i),
                value: i,
            };
            backend.put("tests", &format!("key{}", i), &record).await.unwrap();
        }
        
        // List keys
        let keys = backend.list_keys("tests").await.unwrap();
        assert_eq!(keys.len(), 5);
        
        // Count
        assert_eq!(backend.count("tests").await.unwrap(), 5);
        
        // List all
        let all: Vec<TestRecord> = backend.list_all("tests").await.unwrap();
        assert_eq!(all.len(), 5);
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_batch.redb");
        let backend = RedbBackend::new(&db_path).unwrap();
        
        let records: Vec<TestRecord> = (0..3).map(|i| TestRecord {
            id: format!("batch{}", i),
            name: format!("Batch Record {}", i),
            value: i * 10,
        }).collect();
        
        // Put many
        let items: Vec<(&str, &TestRecord)> = records.iter()
            .enumerate()
            .map(|(i, r)| {
                let key = Box::leak(format!("bkey{}", i).into_boxed_str());
                (key as &str, r)
            })
            .collect();
        backend.put_many("tests", &items).await.unwrap();
        
        // Get many
        let keys: Vec<&str> = items.iter().map(|(k, _)| *k).collect();
        let results: Vec<Option<TestRecord>> = backend.get_many("tests", &keys).await.unwrap();
        assert_eq!(results.len(), 3);
        for (i, result) in results.into_iter().enumerate() {
            assert_eq!(result, Some(records[i].clone()));
        }
    }
}

