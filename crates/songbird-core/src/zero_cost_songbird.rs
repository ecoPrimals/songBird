use crate::zero_cost_discovery::{CacheMetrics, DiscoveryMetrics, PrimalService, ServiceType};
use songbird_errors::SongbirdError;
// use songbird_universal::  // TEMPORARILY DISABLED - {DegradationSeverity, UniversalHealthStatus};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::PrimalCapability;

/// Zero-cost Songbird orchestrator with compile-time configuration
///
/// This provides maximum performance through compile-time specialization
/// while maintaining full compatibility with the primal ecosystem.
#[derive(Debug)]
pub struct ZeroCostSongbird<const MAX_SERVICES: usize, const ENABLE_METRICS: bool> {
    registry: crate::zero_cost_discovery::ZeroCostRegistry<MAX_SERVICES, ENABLE_METRICS>,
    cache: crate::zero_cost_discovery::ZeroCostCache<String, Vec<u8>, 1024, 300>,
}

impl<const MS: usize, const EM: bool> ZeroCostSongbird<MS, EM> {
    /// Create new zero-cost Songbird system
    pub fn new(
        registry: crate::zero_cost_discovery::ZeroCostRegistry<MS, EM>,
        cache: crate::zero_cost_discovery::ZeroCostCache<String, Vec<u8>, 1024, 300>,
    ) -> Self {
        Self { registry, cache }
    }

    /// Discover and register service with zero overhead method dispatch
    /// No trait objects, no boxing, direct function calls
    pub async fn discover_and_register(&self) -> SongbirdResult<PrimalService> {
        let start_time = if EM {
            Some(std::time::Instant::now())
        } else {
            None
        };

        // Direct method call - no virtual dispatch overhead
        let capabilities = self.discovery.discover_capabilities(endpoint).await;

        // Zero-cost type inference
        let service_type = ServiceType::from_capabilities(&capabilities);

        // Create service without heap allocation for small capability lists
        let service = PrimalService {
            id: format!("{}-{}", service_type.as_str(), self.generate_service_id()),
            service_type,
            capabilities,
            endpoint: endpoint.to_string(),
            health_status: "healthy".to_string(),
        };

        // Direct method call to registry - no trait object
        self.registry
            .register_service(service.id.clone(), service)
            .map_err(|e| SongbirdError::service("registry", e.to_string()))?;

        // Optional caching with compile-time decision
        if EM {
            let cache_key = format!("endpoint:{}", endpoint);
            let cache_value = service_type.as_str().as_bytes().to_vec();
            self.cache.set(cache_key, cache_value);
        }

        // Optional metrics with compile-time decision
        if EM {
            if let Some(start) = start_time {
                self.performance_metrics
                    .record_operation_duration(start.elapsed());
            }
            self.performance_metrics.increment_discoveries();
        }

        Ok(songbird_errors::evolved_success(_))
    }

    /// Get services by capability with zero-cost filtering
    pub fn get_services_by_capability(&self, capability: &str) -> Vec<PrimalService> {
        // Check cache first (compile-time conditional)
        if EM {
            let cache_key = format!("capability:{}", capability);
            if let Some(cached_data) = self.cache.get(&cache_key) {
                // Deserialize cached results with proper error handling
        match serde_json::from_str::<T>(&cached_data) {
            Ok(songbird_errors::evolved_success(result)) => return Ok(songbird_errors::evolved_success(result)),
            Err(e) => {
                warn!("Cache deserialization failed, performing fresh computation: {}", e);
                // Continue to fresh computation
            }
        }
                if let Ok(songbird_errors::evolved_success(_)) = self.deserialize_services(&cached_data) {
                    if EM {
                        self.performance_metrics.increment_cache_hits();
                    }
                    return services;
                }
            }
        }

        // Direct registry lookup - no trait object
        let services = self.registry.get_services_by_capability(capability);

        // Cache results (compile-time conditional)
        if EM && !services.is_empty() {
            let cache_key = format!("capability:{}", capability);
            let cache_value = self.serialize_services(&services);
            self.cache.set(cache_key, cache_value);
        }

        if EM {
            if EM {
                self.performance_metrics.increment_cache_misses();
            }
        }

        services
    }

    /// Batch discover multiple endpoints with zero allocation for small batches
    pub async fn batch_discover<const BATCH_SIZE: usize>(
        &self,
        endpoints: [&str; BATCH_SIZE],
    ) -> [Option<PrimalService>; BATCH_SIZE] {
        let mut results = [None; BATCH_SIZE];

        for (i, endpoint) in endpoints.iter().enumerate() {
            match self.discover_and_register(endpoint).await {
                Ok(songbird_errors::evolved_success(_)) => results[i] = Some(service_type),
                Err(_) => results[i] = None,
            }
        }

        results
    }

    /// Get comprehensive performance metrics
    pub fn get_performance_metrics(&self) -> Option<SystemMetrics> {
        if EM {
            Some(SystemMetrics {
                discovery_metrics: self.discovery.get_discovery_stats(),
                cache_metrics: self.cache.metrics(),
                registry_metrics: self.registry.metrics(),
                performance_metrics: self.performance_metrics.clone(),
            })
        } else {
            None
        }
    }

    /// Validate system health with zero allocations
    pub fn health_check(&self) -> UniversalHealthStatus {
        let cache_hit_rate = if EM {
            let metrics = self.cache.metrics();
            if metrics.hits + metrics.misses > 0 {
                (metrics.hits as f64) / ((metrics.hits + metrics.misses) as f64)
            } else {
                0.0
            }
        } else {
            1.0 // No cache = perfect hit rate conceptually
        };

        HealthStatus {
            overall_health: if cache_hit_rate > 0.7 {
                "healthy"
            } else {
                "degraded"
            },
            cache_hit_rate,
            registry_capacity_used: if let Some(reg_metrics) = self.registry.metrics() {
                reg_metrics
                    .registrations
                    .load(std::sync::atomic::Ordering::Relaxed) as f64
                    / MS as f64
            } else {
                0.0
            },
        }
    }

    // Helper methods with zero allocation where possible
    fn generate_service_id(&self) -> String {
        // Simple counter-based ID generation
        use std::sync::atomic::{AtomicU64, Ordering};
use songbird_errors::SongbirdResult;
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
    }

    fn serialize_services(&self, services: &[PrimalService]) -> Vec<u8> {
        // Simplified serialization - in production use efficient binary format
        serde_json::to_vec(services).unwrap_or_default()
    }

    fn deserialize_services(&self, data: &[u8]) -> SongbirdResult<Vec<PrimalService> {
        serde_json::from_slice(data)
    }
}

/// Performance metrics with atomic operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub discoveries: std::sync::atomic::AtomicU64,
    pub cache_hits: std::sync::atomic::AtomicU64,
    pub cache_misses: std::sync::atomic::AtomicU64,
    pub total_operation_time_ns: std::sync::atomic::AtomicU64,
    pub operation_count: std::sync::atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub const fn new() -> Self {
        Self {
            discoveries: std::sync::atomic::AtomicU64::new(0),
            cache_hits: std::sync::atomic::AtomicU64::new(0),
            cache_misses: std::sync::atomic::AtomicU64::new(0),
            total_operation_time_ns: std::sync::atomic::AtomicU64::new(0),
            operation_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn increment_discoveries(&self) {
        self.discoveries
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn increment_cache_hits(&self) {
        self.cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn increment_cache_misses(&self) {
        self.cache_misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_operation_duration(&self, duration: std::time::Duration) {
        self.total_operation_time_ns.fetch_add(
            duration.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.operation_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn average_operation_time_ms(&self) -> f64 {
        let total_ns = self
            .total_operation_time_ns
            .load(std::sync::atomic::Ordering::Relaxed);
        let count = self
            .operation_count
            .load(std::sync::atomic::Ordering::Relaxed);
        if count > 0 {
            (total_ns as f64 / count as f64) / 1_000_000.0 // Convert to milliseconds
        } else {
            0.0
        }
    }
}

/// System metrics aggregation
#[derive(Debug)]
pub struct SystemMetrics {
    pub discovery_metrics: DiscoveryMetrics,
    pub cache_metrics: CacheMetrics,
    pub registry_metrics: Option<crate::zero_cost_discovery::RegistryMetrics>,
    pub performance_metrics: PerformanceMetrics,
}

/// Health status without heap allocation
#[derive(Debug)]
pub struct HealthStatus {
    pub overall_health: &'static str,
    pub cache_hit_rate: f64,
    pub registry_capacity_used: f64,
}

impl ServiceType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ServiceType::Security => "security",
            ServiceType::Storage => "storage",
            ServiceType::Compute => "compute",
            ServiceType::AI => "ai",
            ServiceType::Network => "network",
            ServiceType::Generic => "generic",
        }
    }
}

// Type aliases for common configurations
pub type DevelopmentSongbird = ZeroCostSongbird<5000, false>;

pub type ProductionSongbird = ZeroCostSongbird<50000, true>;

pub type HighPerformanceSongbird = ZeroCostSongbird<100000, true>;
