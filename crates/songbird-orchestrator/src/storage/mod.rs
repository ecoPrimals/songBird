//! Pure Rust Storage Abstraction Layer
//!
//! This module provides a storage backend abstraction that enables:
//! - 100% Pure Rust (no C dependencies)
//! - ACID-compliant transactions
//! - Easy backend swapping (redb, in-memory for tests)
//!
//! ## Migration from sqlx
//!
//! This replaces sqlx (which requires libsqlite3-sys C library)
//! with redb (100% Pure Rust embedded database).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           StorageBackend Trait          │
//! │  (abstraction for all storage needs)    │
//! └────────────┬───────────────┬────────────┘
//!              │               │
//!      ┌───────┴───────┐ ┌────┴─────────┐
//!      │  RedbBackend  │ │ MemoryBackend│
//!      │  (production) │ │   (testing)  │
//!      └───────────────┘ └──────────────┘
//! ```

pub mod backend;
pub mod redb_backend;
pub mod memory_backend;

pub use backend::{StorageBackend, StorageError, StorageResult};
pub use redb_backend::RedbBackend;
pub use memory_backend::MemoryBackend;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_module_exists() {
        // Simple test to verify module is compiled
        assert!(true);
    }
}

/// Create a storage backend from environment configuration
///
/// Looks for `STORAGE_BACKEND` environment variable:
/// - `redb` (default): Use redb Pure Rust database
/// - `memory`: Use in-memory storage (testing only)
///
/// For `redb`, also uses `DATABASE_PATH` for the database file location.
pub async fn create_backend() -> StorageResult<Box<dyn StorageBackend>> {
    let backend_type = std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "redb".to_string());
    
    match backend_type.as_str() {
        "memory" => {
            tracing::info!("Using in-memory storage backend (testing mode)");
            Ok(Box::new(MemoryBackend::new()))
        }
        "redb" | _ => {
            let db_path = std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "data/songbird.redb".to_string());
            tracing::info!("Using redb storage backend: {}", db_path);
            Ok(Box::new(RedbBackend::new(&db_path)?))
        }
    }
}
