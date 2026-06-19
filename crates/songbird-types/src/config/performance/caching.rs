// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cache layer configuration types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Caching configuration - consolidated from network-layer crates (e.g. federation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache TTL
    pub ttl: Duration,
    /// Enable cache compression
    pub compression_enabled: bool,
    /// Cache eviction policy
    pub eviction_policy: CacheEvictionPolicy,
    /// Enable cache statistics
    pub statistics_enabled: bool,
    /// Cache layers configuration
    pub layers: Vec<CacheLayerConfig>,
    /// Cache size in MB
    pub cache_size_mb: usize,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size_mb: 256,
            ttl: Duration::from_secs(3600), // 1 hour
            compression_enabled: false,
            eviction_policy: CacheEvictionPolicy::Lru,
            statistics_enabled: true,
            layers: vec![
                CacheLayerConfig {
                    name: String::from("L1"),
                    size_mb: 64,
                    ttl: Duration::from_secs(300), // 5 minutes
                },
                CacheLayerConfig {
                    name: String::from("L2"),
                    size_mb: 192,
                    ttl: Duration::from_secs(3600), // 1 hour
                },
            ],
        }
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// First In, First Out
    Fifo,
    /// Random replacement
    Random,
}

/// Cache layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLayerConfig {
    /// Layer name
    pub name: String,
    /// Layer size in MB
    pub size_mb: usize,
    /// Time to live
    pub ttl: Duration,
}
