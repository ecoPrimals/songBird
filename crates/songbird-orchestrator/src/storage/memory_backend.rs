//! In-Memory Backend - Fast, ephemeral storage for testing
//!
//! This backend stores all data in memory (HashMap), making it ideal for:
//! - Unit tests
//! - Integration tests
//! - Development environments
//!
//! Data is NOT persisted and will be lost when the process exits.

use super::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::trace;

/// In-memory storage backend (for testing)
pub struct MemoryBackend {
    /// Table name -> Key -> JSON bytes
    data: RwLock<HashMap<String, HashMap<String, Vec<u8>>>>,
}

impl MemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn get<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        key: &str
    ) -> StorageResult<Option<T>> {
        trace!("GET {}/{}", table, key);
        
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        if let Some(table_data) = data.get(table) {
            if let Some(bytes) = table_data.get(key) {
                let item: T = serde_json::from_slice(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                return Ok(Some(item));
            }
        }
        Ok(None)
    }
    
    async fn put<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        key: &str, 
        value: &T
    ) -> StorageResult<()> {
        trace!("PUT {}/{}", table, key);
        
        let bytes = serde_json::to_vec(value)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let mut data = self.data.write()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        data.entry(table.to_string())
            .or_default()
            .insert(key.to_string(), bytes);
        
        Ok(())
    }
    
    async fn delete(&self, table: &str, key: &str) -> StorageResult<bool> {
        trace!("DELETE {}/{}", table, key);
        
        let mut data = self.data.write()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        if let Some(table_data) = data.get_mut(table) {
            return Ok(table_data.remove(key).is_some());
        }
        Ok(false)
    }
    
    async fn exists(&self, table: &str, key: &str) -> StorageResult<bool> {
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        Ok(data.get(table)
            .map(|t| t.contains_key(key))
            .unwrap_or(false))
    }
    
    async fn list_keys(&self, table: &str) -> StorageResult<Vec<String>> {
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        Ok(data.get(table)
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default())
    }
    
    async fn list_all<T: DeserializeOwned + Send>(&self, table: &str) -> StorageResult<Vec<T>> {
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        let mut items = Vec::new();
        if let Some(table_data) = data.get(table) {
            for bytes in table_data.values() {
                let item: T = serde_json::from_slice(bytes)
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
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        let table_data = data.get(table);
        
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(td) = table_data {
                if let Some(bytes) = td.get(*key) {
                    let item: T = serde_json::from_slice(bytes)
                        .map_err(|e| StorageError::Serialization(e.to_string()))?;
                    results.push(Some(item));
                    continue;
                }
            }
            results.push(None);
        }
        Ok(results)
    }
    
    async fn put_many<T: Serialize + Send + Sync>(
        &self, 
        table: &str, 
        items: &[(&str, &T)]
    ) -> StorageResult<()> {
        let mut data = self.data.write()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        let table_data = data.entry(table.to_string()).or_default();
        
        for (key, value) in items {
            let bytes = serde_json::to_vec(value)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            table_data.insert((*key).to_string(), bytes);
        }
        
        Ok(())
    }
    
    async fn delete_many(&self, table: &str, keys: &[&str]) -> StorageResult<usize> {
        let mut data = self.data.write()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        let mut deleted = 0;
        if let Some(table_data) = data.get_mut(table) {
            for key in keys {
                if table_data.remove(*key).is_some() {
                    deleted += 1;
                }
            }
        }
        Ok(deleted)
    }
    
    async fn count(&self, table: &str) -> StorageResult<usize> {
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        Ok(data.get(table).map(|t| t.len()).unwrap_or(0))
    }
    
    async fn query_by_prefix<T: DeserializeOwned + Send>(
        &self, 
        table: &str, 
        prefix: &str
    ) -> StorageResult<Vec<(String, T)>> {
        let data = self.data.read()
            .map_err(|e| StorageError::Database(format!("Lock poisoned: {}", e)))?;
        
        let mut results = Vec::new();
        if let Some(table_data) = data.get(table) {
            for (key, bytes) in table_data {
                if key.starts_with(prefix) {
                    let item: T = serde_json::from_slice(bytes)
                        .map_err(|e| StorageError::Serialization(e.to_string()))?;
                    results.push((key.clone(), item));
                }
            }
        }
        Ok(results)
    }
    
    async fn compact(&self) -> StorageResult<()> {
        // No-op for in-memory backend
        Ok(())
    }
    
    async fn flush(&self) -> StorageResult<()> {
        // No-op for in-memory backend
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: String,
        name: String,
    }
    
    #[tokio::test]
    async fn test_memory_crud() {
        let backend = MemoryBackend::new();
        
        let record = TestRecord {
            id: "1".to_string(),
            name: "Test".to_string(),
        };
        
        // Put
        backend.put("items", "1", &record).await.unwrap();
        
        // Get
        let retrieved: Option<TestRecord> = backend.get("items", "1").await.unwrap();
        assert_eq!(retrieved, Some(record));
        
        // Delete
        assert!(backend.delete("items", "1").await.unwrap());
        
        // Verify deleted
        let retrieved: Option<TestRecord> = backend.get("items", "1").await.unwrap();
        assert_eq!(retrieved, None);
    }
    
    #[tokio::test]
    async fn test_memory_list() {
        let backend = MemoryBackend::new();
        
        for i in 0..3 {
            let record = TestRecord {
                id: format!("{}", i),
                name: format!("Item {}", i),
            };
            backend.put("items", &format!("{}", i), &record).await.unwrap();
        }
        
        // Count
        assert_eq!(backend.count("items").await.unwrap(), 3);
        
        // List keys
        let keys = backend.list_keys("items").await.unwrap();
        assert_eq!(keys.len(), 3);
        
        // List all
        let all: Vec<TestRecord> = backend.list_all("items").await.unwrap();
        assert_eq!(all.len(), 3);
    }
}

