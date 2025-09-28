//! # 🏪 UNIVERSAL STORAGE SYSTEM - CAPABILITY-BASED ARCHITECTURE
//!
//! This module implements a completely primal-agnostic storage system that
//! operates purely on storage capabilities without any hardcoded knowledge
//! of specific storage providers (NestGate, etc.).
//!
//! ## Core Principle
//! Storage operations are routed based on capabilities like "data_persistence","
//! "object_storage", "file_system" rather than provider names."
//!
//! ## Modular Organization
//! - `client`: Core storage client and main interface
//! - `types`: Storage capability types and requirements
//! - `cache`: Local caching system for performance
//! - `stats`: Statistics and performance monitoring
//! - `events`: Event system for storage operations
//! - `config`: Configuration types and defaults

pub mod cache;
pub mod client;
pub mod config;
pub mod events;
pub mod stats;
pub mod types;

// Re-export main types for convenience
pub use cache::{CacheConfig, CacheEvictionStrategy, CacheStats, StorageCache};
pub use client::UniversalStorageClient;
pub use config::UniversalStorageConfig;
pub use events::UniversalStorageEvent;
pub use stats::{EcosystemHealthMetrics, ProviderPerformanceStats, StorageStats};
pub use types::*;
