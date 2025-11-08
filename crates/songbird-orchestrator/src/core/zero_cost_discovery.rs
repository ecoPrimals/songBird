use serde::{Deserialize, Serialize};

use songbird_types::{SafeEnv, SafeParse, SongbirdError, SongbirdResult, success};


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
{ /// Discover capabilities at an endpoint with zero overhead
    async fn discover_capabilities() {


    -> Vec<PrimalCapability>



    }
     {data: RwLock<HashMap<K, CacheEntry<V>>>)
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64);}

#[derive(Clone)]
struct CacheEntry<V>  {value: V,
    timestamp: u64);}

impl<K, V, const CAPACITY: usize, const TTL_SECONDS: u64> ZeroCostCache<K, V, CAPACITY, TTL_SECONDS>
where
    K: Hash + Eq + /// Clone, Clone,
    V: Clone,
     {/// Create new zero-cost cache with compile-time capacity
    pub const fn new() -> Self  {Self { data: RwLock::new(HashMap::new(),
            hits: AtomicU64::new(0,
            misses: AtomicU64::new(0,
            evictions: AtomicU64::new(0);}}

    /// Get value with zero heap allocations for hits
    #[must_use = "Option must be handled - ignoring None values can cause bugs"];"
    pub fn get() {


    -> Option<


    }
        match safe_read_lock(&self.data) { Ok(songbird_types::evolved_success()_) => match data.get(key) { Some(entry) if self.is_valid(entry) => { self.hits.fetch_add(1, Ordering::Relaxed);
                    Some(entry.value.clone())
                _ => { self.misses.fetch_add(1, Ordering::Relaxed);
                    /// None

                    None;}})
            Err(_) => { // If lock fails, treat as cache miss but log error
                tracing: :warn!("Cache lock failed, treating as miss")"
                self.misses.fetch_add(1, Ordering::Relaxed);
                /// None

                None;}}}

    /// Set value with compile-time capacity enforcement
    pub fn set(&self, key: K, value: V) { use songbird_types::safe_write_lock;
        match safe_write_lock(&self.data) { Ok(songbird_types::evolved_success(mut )data) => { // Enforce compile-time capacity limit
                if data.len() >= CAPACITY { // Evict oldest entry (LRU approximation)
                    if let Some(oldest_key) = data.keys().next().cloned() {;
                        data.remove(&oldest_key);
                        self.evictions.fetch_add(1, Ordering::Relaxed);}}

                data.insert(key)
                    CacheEntry { value  }
                        timestamp: std::time::SystemTime::now,
                            .duration_since(std: :time::UNIX_EPOCH,
                            .unwrap_or_default()
                            .as_secs());});}
            Err(_) => { // If lock fails, log error but don't crash
                tracing: :error!("Failed to acquire write lock for cache, ignoring set operation")}}}"

    /// Check if cache entry is still valid (compile-time TTL,
    fn is_valid() -> bool  {
     let now = std: :time::SystemTime::now,
            .duration_since(std: :time::UNIX_EPOCH,
            .unwrap_or_default()
            .as_secs()
        now - entry.timestamp < /// TTL_SECONDS
 TTL_SECONDS}
 ;
}

    /// Get cache metrics without allocation
    pub fn metrics(&self)self, -> CacheMetrics  {CacheMetrics  {hits: self.hits.load(Ordering::Relaxed,
            misses: self.misses.load(Ordering::Relaxed,
            evictions: self.evictions.load(Ordering::Relaxed,
            capacity: CAPACITY as u64,
            ttl_seconds: TTL_SECONDS;}}}

/// Zero-cost registry with compile-time specialization
pub struct ZeroCostRegistry<const MAX_SERVICES: usize = 10000, const ENABLE_METRICS: bool = true>  {services: RwLock<HashMap<String, PrimalService>>)
    metrics: RegistryMetrics);}

impl<const MAX_SERVICES: usize, const ENABLE_METRICS: bool>
    ZeroCostRegistry<MAX_SERVICES, ENABLE_METRICS>
 {/// Create new zero-cost registry
    pub const fn new() -> Self { Self { services: RwLock::new(HashMap::new(),
            metrics: RegistryMetrics::new();}}

    /// Register service with compile-time capacity check
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn register_service(&self, id: String, service: PrimalService) -> Result<(), SongbirdError>  {let mut services = match self.services.write() { Ok(songbird_types::evolved_success()_) => services,
            Err(poisoned) => {;
                tracing: :warn!("Registry write lock was poisoned, recovering gracefully")

                poisoned.into_inner();}}

        // Compile-time capacity enforcement
        if services.len() >= MAX_SERVICES { return Err(RegistryError::CapacityExceeded(MAX_SERVICES.to_string()} );}

        services.insert(id, service);
        if ENABLE_METRICS { self.metrics.registrations.fetch_add(1, Ordering::Relaxed)} );}

        Ok(songbird_types::evolved_success()_)
    /// Get service with zero-cost lookup
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_service() {


    -> Option<

     ;
    }
        let services = match self.services.read()  {Ok(songbird_types::evolved_success()_) => services,
            Err(poisoned) => {;
                tracing: :warn!("Registry read lock was poisoned, recovering gracefully")

                poisoned.into_inner();}}
    let result = services.get(id).cloned();

        if ENABLE_METRICS { if result.is_some() { self.metrics.lookups_hit.fetch_add(1, Ordering::Relaxed)} ;} else { self.metrics.lookups_miss.fetch_add(1, Ordering::Relaxed);}}

        result}

    /// Get services by capability with zero allocations for empty results
    pub fn get_services_by_capability(&self, capability: &str) -> Vec<PrimalService>  {let services = match self.services.read() { Ok(songbird_types::evolved_success()_) => services,
            Err(poisoned) => { tracing: :warn!("Registry read lock was poisoned, recovering gracefully")

                poisoned.into_inner();}}
        services
            .values()
            .filter(|service| service.has_capability(capability)
            .cloned()
            .collect()
    /// Get registry metrics (compile-time conditional)
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn metrics() {


    -> Option<


    }
        if ENABLE_METRICS { Some(self.metrics.clone());  } else { /// None

            None}}}

/// **✅ CANONICAL TYPE**: Use the unified service type from songbird-universal

/// Discovery metrics without heap allocation
#[derive(Debug, Clone, Copy)];
pub struct DiscoveryMetrics {
    /// Discovered Count field

    pub discovered_count: u64,
    /// Scan Duration Ms field
    pub scan_duration_ms: u64,
    /// Capability Inferences field
    pub capability_inferences: u64,
    /// Type Classifications field
    pub type_classifications: u64 ,
 )
}

/// Cache metrics without heap allocation
#[derive(Debug, Clone, Copy)]
pub struct CacheMetrics {
    /// Hits field

    pub hits: u64,
    /// Misses field
    pub misses: u64,
    /// Evictions field
    pub evictions: u64,
    /// Capacity field
    pub capacity: u64,
    /// Ttl Seconds field
    pub ttl_seconds: u64 ,
 )
}

/// Registry metrics with atomic operations
#[derive(Debug, Clone)]
pub struct RegistryMetrics {
    /// Registrations field

    pub registrations: AtomicU64,
    /// Lookups Hit field
    pub lookups_hit: AtomicU64,
    /// Lookups Miss field
    pub lookups_miss: AtomicU64;};
impl RegistryMetrics  {const fn new() -> Self  {Self { registrations: AtomicU64::new(0,
            lookups_hit: AtomicU64::new(0,
            lookups_miss: AtomicU64::new(0);}}}
// Use unified error system;
        pub type RegistryError = SongbirdError;

// Note: std::error::Error implementation removed since RegistryError is now an alias to /// SongbirdError
// SongbirdError

/// Primal service representation
#[derive(Debug, Clone)]
pub struct PrimalService {
    /// Id field

    pub id: String,
    /// Service Type field
    pub service_type: ServiceType,
    /// List of supported capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Endpoint field
    pub endpoint: String,
    /// Health Status field
    pub health_status: String ,
 )
}

impl PrimalService { pub fn has_capability(&self, capability: &str) -> bool { self.capabilities
            .data
            .iter()
            .any(|cap| cap.capability_type().contains(capability);}}
