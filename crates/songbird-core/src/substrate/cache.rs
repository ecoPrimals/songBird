//! Optimized substrate cache with TTL and LRU eviction

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::types::SystemInfo;

/// Optimized substrate cache with TTL and size limits
#[derive(Debug)]
pub struct OptimizedSubstrateCache {
    pub paths: HashMap<String, CacheEntry<PathBuf>>,
    pub capabilities: HashMap<String, CacheEntry<Vec<String>>>,
    pub system_info: Option<CacheEntry<SystemInfo>>,
    pub cache_size: usize,
    pub max_size: usize,
    pub ttl: Duration,
}

/// Cache entry with TTL and access tracking
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    /// Create new cache entry
    pub fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }

    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Access the cached value and update statistics
    pub fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }

    /// Get the cached value without updating access statistics
    pub fn peek(&self) -> &T {
        &self.value
    }

    /// Get cache entry age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last access
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

impl OptimizedSubstrateCache {
    /// Create new optimized cache
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            paths: HashMap::new(),
            capabilities: HashMap::new(),
            system_info: None,
            cache_size: 0,
            max_size,
            ttl,
        }
    }

    /// Get system info from cache
    pub fn get_system_info(&mut self) -> Option<SystemInfo> {
        if let Some(entry) = &mut self.system_info {
            if !entry.is_expired() {
                return Some(entry.access().clone());
            } else {
                self.system_info = None;
                self.cache_size = self.cache_size.saturating_sub(1);
            }
        }
        None
    }

    /// Cache system info
    pub fn cache_system_info(&mut self, info: SystemInfo) {
        self.evict_if_needed();
        self.system_info = Some(CacheEntry::new(info, self.ttl));
        self.cache_size += 1;
    }

    /// Get path from cache
    pub fn get_path(&mut self, key: &str) -> Option<PathBuf> {
        if let Some(entry) = self.paths.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.access().clone());
            } else {
                self.paths.remove(key);
                self.cache_size = self.cache_size.saturating_sub(1);
            }
        }
        None
    }

    /// Cache path
    pub fn cache_path(&mut self, key: String, path: PathBuf) {
        self.evict_if_needed();
        self.paths.insert(key, CacheEntry::new(path, self.ttl));
        self.cache_size += 1;
    }

    /// Get capabilities from cache
    pub fn get_capabilities(&mut self, key: &str) -> Option<Vec<String>> {
        if let Some(entry) = self.capabilities.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.access().clone());
            } else {
                self.capabilities.remove(key);
                self.cache_size = self.cache_size.saturating_sub(1);
            }
        }
        None
    }

    /// Cache capabilities
    pub fn cache_capabilities(&mut self, key: String, capabilities: Vec<String>) {
        self.evict_if_needed();
        self.capabilities
            .insert(key, CacheEntry::new(capabilities, self.ttl));
        self.cache_size += 1;
    }

    /// Evict least recently used entries if cache is full
    fn evict_if_needed(&mut self) {
        while self.cache_size >= self.max_size {
            self.evict_lru();
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        // Check paths
        for (key, entry) in &self.paths {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(key.clone());
            }
        }

        // Check capabilities
        for (key, entry) in &self.capabilities {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(format!("cap_{key}"));
            }
        }

        // Check system info
        if let Some(entry) = &self.system_info {
            if entry.last_accessed < oldest_time {
                oldest_key = Some("system_info".to_string());
            }
        }

        // Evict the oldest entry
        if let Some(key) = oldest_key {
            if key == "system_info" {
                self.system_info = None;
            } else if key.starts_with("cap_") {
                let cap_key = key.strip_prefix("cap_").unwrap();
                self.capabilities.remove(cap_key);
            } else {
                self.paths.remove(&key);
            }
            self.cache_size = self.cache_size.saturating_sub(1);
        }
    }

    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.paths.clear();
        self.capabilities.clear();
        self.system_info = None;
        self.cache_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache_size,
            max_size: self.max_size,
            paths_cached: self.paths.len(),
            capabilities_cached: self.capabilities.len(),
            has_system_info: self.system_info.is_some(),
        }
    }

    /// Get cache statistics (alias for compatibility)
    pub fn get_stats(&self) -> CacheStats {
        self.stats()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub paths_cached: usize,
    pub capabilities_cached: usize,
    pub has_system_info: bool,
}
