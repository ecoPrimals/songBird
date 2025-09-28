//! Songbird Configuration System
//!
//! This crate provides comprehensive configuration management for Songbird,
//! supporting environment-based configuration, validation, and zero-hardcoded systems.
//!
//! ## Key Features
//! - **Environment-Based Configuration**: Support for dev/staging/production
//! - **Zero-Hardcoded Values**: All values configurable through environment or files
//! - **Hardcoded Value Elimination**: Dynamic configuration without hardcoded values
//! - **Performance Tuning**: Configurable performance parameters
//! - **Universal Configuration**: Support for any deployment scenario

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod canonical_network;
pub mod config;
pub mod environment_config_clean;
pub mod zero_touch;

pub use config::*;

// Re-export environment configuration from config module
pub use config::environment::EnvironmentConfig;

// Re-export environment configuration helper
pub use environment_config_clean::EnvironmentConfig as EnvConfig;

/// Performance configuration for fine-tuning system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Buffer pool size for high-performance operations
    pub buffer_pool_size: Option<usize>,

    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,

    /// Number of worker threads (default: CPU cores)
    pub worker_threads: Option<usize>,

    /// Connection pool size for networking
    pub connection_pool_size: Option<usize>,

    /// Request timeout in milliseconds
    pub request_timeout_ms: Option<u64>,

    /// Enable zero-copy optimizations where possible
    pub enable_zero_copy: Option<bool>,

    /// Batch processing size for bulk operations
    pub batch_size: Option<usize>,

    /// Custom performance parameters
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

impl Default for PerformanceConfig  {fn default() -> Self  {Self {
            buffer_pool_size: None, // Calculated based on environment
            max_memory_mb: None,    // Detected from system
            worker_threads: None,   // Defaults to CPU cores
            connection_pool_size: Some(100),
            request_timeout_ms: Some(30000),
            enable_zero_copy: Some(true),
            batch_size: Some(1000),
            custom_params: None,
        }
    }
}
