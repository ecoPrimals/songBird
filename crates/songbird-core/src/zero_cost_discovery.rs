use serde::{Deserialize, Serialize};

use songbird_errors::{SafeEnv, SafeParse, SongbirdError, SongbirdResult, success};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::discovery::DiscoveredPrimal;

// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::PrimalCapability;

use std::collections::HashMap;

use std::hash::Hash;

use std::sync::atomic::{AtomicU64, Ordering};

use std::sync::{Arc, RwLock};

/// Zero-cost discovery trait with compile-time specialization
/// Replaces async_trait overhead with direct method dispatch
pub trait ZeroCostDiscovery<
    const MAX_PRIMALS: usize = 1000,
    const DISCOVERY_TIMEOUT_MS: u64 = 5000,
    const ENABLE_CACHING: bool = true,
>
{
    /// Discover capabilities at an endpoint with zero overhead
    async fn discover_capabilities(&self, endpoint: &str) -> Vec<PrimalCapability>;

    /// Scan network range with compile-time bounded capacity
    async fn scan_network_range(&self, range: &str) -> [Option<DiscoveredPrimal>; MAX_PRIMALS];

    /// Get discovery metrics without heap allocation
    fn get_discovery_stats(&self) -> DiscoveryMetrics;

    /// Probe service type with zero-cost inference
    fn infer_service_type(&self, capabilities: &[PrimalCapability]) -> ServiceType;
}

/// Zero-cost cache with compile-time specialization
pub struct ZeroCostCache<K, V, const CAPACITY: usize, const TTL_SECONDS: u64>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    data: RwLock<HashMap<K, CacheEntry<V>>>,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    timestamp: u64,
}

impl<K, V, const CAPACITY: usize, const TTL_SECONDS: u64> ZeroCostCache<K, V, CAPACITY, TTL_SECONDS>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create new zero-cost cache with compile-time capacity
    pub const fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Get value with zero heap allocations for hits
    pub fn get(&self, key: &K) -> Option<V> {
        use songbird_errors::safe_read_lock;
        match safe_read_lock(&self.data) {
            Ok(songbird_errors::evolved_success(_)) => match data.get(key) {
                Some(entry) if self.is_valid(entry) => {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    Some(entry.value.clone())
                }
                _ => {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    None
                }
            },
            Err(_) => {
                // If lock fails, treat as cache miss but log error
                tracing::warn!("Cache lock failed, treating as miss");
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Set value with compile-time capacity enforcement
    pub fn set(&self, key: K, value: V) {
        use songbird_errors::safe_write_lock;
use songbird_errors::SongbirdResult;
        match safe_write_lock(&self.data) {
            Ok(songbird_errors::evolved_success(mut data)) => {
                // Enforce compile-time capacity limit
                if data.len() >= CAPACITY {
                    // Evict oldest entry (LRU approximation)
                    if let Some(oldest_key) = data.keys().next().cloned() {
                        data.remove(&oldest_key);
                        self.evictions.fetch_add(1, Ordering::Relaxed);
                    }
                }

                data.insert(
                    key,
                    CacheEntry {
                        value,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    },
                );
            }
            Err(_) => {
                // If lock fails, log error but don't crash
                tracing::error!("Failed to acquire write lock for cache, ignoring set operation");
            }
        }
    }

    /// Check if cache entry is still valid (compile-time TTL)
    fn is_valid(&self, entry: &CacheEntry<V>) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - entry.timestamp < TTL_SECONDS
    }

    /// Get cache metrics without allocation
    pub fn metrics(&self) -> CacheMetrics {
        CacheMetrics {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            capacity: CAPACITY as u64,
            ttl_seconds: TTL_SECONDS,
        }
    }
}

/// Zero-cost registry with compile-time specialization
pub struct ZeroCostRegistry<const MAX_SERVICES: usize = 10000, const ENABLE_METRICS: bool = true> {
    services: RwLock<HashMap<String, PrimalService>>,
    metrics: RegistryMetrics,
}

impl<const MAX_SERVICES: usize, const ENABLE_METRICS: bool>
    ZeroCostRegistry<MAX_SERVICES, ENABLE_METRICS>
{
    /// Create new zero-cost registry
    pub const fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            metrics: RegistryMetrics::new(),
        }
    }

    /// Register service with compile-time capacity check
    pub async fn register_service(&self, id: String, service: PrimalService) -> SongbirdResult<()> {
        let mut services = match self.services.write() {
            Ok(songbird_errors::evolved_success(_)) => services,
            Err(poisoned) => {
                tracing::warn!("Registry write lock was poisoned, recovering gracefully");
                poisoned.into_inner()
            }
        };

        // Compile-time capacity enforcement
        if services.len() >= MAX_SERVICES {
            return Err(RegistryError::CapacityExceeded(MAX_SERVICES.to_string()));
        }

        services.insert(id, service);

        if ENABLE_METRICS {
            self.metrics.registrations.fetch_add(1, Ordering::Relaxed);
        }

        Ok(songbird_errors::evolved_success(_))
    }

    /// Get service with zero-cost lookup
    pub fn get_service(&self, id: &str) -> Option<PrimalService> {
        let services = match self.services.read() {
            Ok(songbird_errors::evolved_success(_)) => services,
            Err(poisoned) => {
                tracing::warn!("Registry read lock was poisoned, recovering gracefully");
                poisoned.into_inner()
            }
        };
        let result = services.get(id).cloned();

        if ENABLE_METRICS {
            if result.is_some() {
                self.metrics.lookups_hit.fetch_add(1, Ordering::Relaxed);
            } else {
                self.metrics.lookups_miss.fetch_add(1, Ordering::Relaxed);
            }
        }

        result
    }

    /// Get services by capability with zero allocations for empty results
    pub fn get_services_by_capability(&self, capability: &str) -> Vec<PrimalService> {
        let services = match self.services.read() {
            Ok(songbird_errors::evolved_success(_)) => services,
            Err(poisoned) => {
                tracing::warn!("Registry read lock was poisoned, recovering gracefully");
                poisoned.into_inner()
            }
        };
        services
            .values()
            .filter(|service| service.has_capability(capability))
            .cloned()
            .collect()
    }

    /// Get registry metrics (compile-time conditional)
    pub fn metrics(&self) -> Option<RegistryMetrics> {
        if ENABLE_METRICS {
            Some(self.metrics.clone())
        } else {
            None
        }
    }
}

/// **✅ CANONICAL TYPE**: Use the unified service type from songbird-universal
pub // use songbird_universal::  // TEMPORARILY DISABLED - adapters::types::UniversalServiceType as ServiceType;

/// Discovery metrics without heap allocation
#[derive(Debug, Clone, Copy)]
pub struct DiscoveryMetrics {
    pub discovered_count: u64,
    pub scan_duration_ms: u64,
    pub capability_inferences: u64,
    pub type_classifications: u64,
}

/// Cache metrics without heap allocation
#[derive(Debug, Clone, Copy)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub capacity: u64,
    pub ttl_seconds: u64,
}

/// Registry metrics with atomic operations
#[derive(Debug, Clone)]
pub struct RegistryMetrics {
    pub registrations: AtomicU64,
    pub lookups_hit: AtomicU64,
    pub lookups_miss: AtomicU64,
}

impl RegistryMetrics {
    const fn new() -> Self {
        Self {
            registrations: AtomicU64::new(0),
            lookups_hit: AtomicU64::new(0),
            lookups_miss: AtomicU64::new(0),
        }
    }
}

// Use unified error system

/// Registry error types
pub type RegistryError = SongbirdError;

// Note: std::error::Error implementation removed since RegistryError is now an alias to SongbirdError

/// Primal service representation
#[derive(Debug, Clone)]
pub struct PrimalService {
    pub id: String,
    pub service_type: ServiceType,
    pub capabilities: Vec<PrimalCapability>,
    pub endpoint: String,
    pub health_status: String,
}

impl PrimalService {
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .data
            .iter()
            .any(|cap| cap.capability_type().contains(capability))
    }
}
