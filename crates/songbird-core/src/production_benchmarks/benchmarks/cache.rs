//! Cache Performance Benchmarking
//!
//! Comprehensive cache performance benchmarking and analysis

use crate::performance::*;
use crate::production_benchmarks::types::*;
use songbird_errors::SongbirdResult;
use std::time::{Duration, Instant};

/// Cache benchmark implementation
pub struct CacheBenchmarker<'a> {
    config: &'a BenchmarkConfig,
    performance_optimizer: &'a ProductionPerformanceOptimizer,
}

impl<'a> CacheBenchmarker<'a> {
    pub fn new(
        config: &'a BenchmarkConfig,
        performance_optimizer: &'a ProductionPerformanceOptimizer,
    ) -> Self {
        Self {
            config,
            performance_optimizer,
        }
    }

    /// Benchmark cache performance with PUT and GET operations
    pub async fn benchmark_cache(&self) -> SongbirdResult<CacheBenchmark> {
        println!("🧠 Benchmarking Cache Performance...");

        // Create cache configuration
        let _cache_config = CacheConfig {
            max_size: self.config.cache_test_data_size,
            max_memory_mb: 64,
            ttl: Duration::from_secs(60),
            frequency_window: Duration::from_secs(5),
            adaptive_threshold: 0.8,
        };

        let cache = self.performance_optimizer.create_cache::<String, String>();

        // Benchmark PUT operations
        let put_ops_per_second = self.benchmark_put_operations(&cache).await?;

        // Benchmark GET operations with realistic access patterns
        let (get_ops_per_second, hit_rate_percentage, average_access_time_ns) =
            self.benchmark_get_operations(&cache).await?;

        // Calculate memory efficiency and adaptive performance
        let metrics = cache.get_metrics().await;
        let memory_efficiency_mb_per_1k_items = (metrics.total_size_bytes as f64 / 1024.0 / 1024.0)
            / (self.config.cache_test_data_size as f64 / 1000.0);

        let adaptive_performance_gain = hit_rate_percentage / 70.0; // Compare to baseline 70%

        self.print_results(get_ops_per_second, put_ops_per_second, hit_rate_percentage);

        Ok(CacheBenchmark {
            get_ops_per_second,
            put_ops_per_second,
            hit_rate_percentage,
            average_access_time_ns,
            memory_efficiency_mb_per_1k_items,
            adaptive_performance_gain,
        })
    }

    /// Benchmark cache PUT operations
    async fn benchmark_put_operations(
        &self,
        cache: &AdaptiveCache<String, String>,
    ) -> SongbirdResult<f64> {
        let put_start = Instant::now();
        for i in 0..self.config.cache_test_data_size {
            let key = format!("key-{i}");
            let value = format!("value-{}-{}", i, "x".repeat(100)); // ~100 byte values
            cache.put(key, value, Some(120)).await;
        }
        let put_duration = put_start.elapsed();
        let put_ops_per_second =
            self.config.cache_test_data_size as f64 / put_duration.as_secs_f64();

        Ok(put_ops_per_second)
    }

    /// Benchmark cache GET operations with realistic access patterns
    async fn benchmark_get_operations(
        &self,
        cache: &AdaptiveCache<String, String>,
    ) -> SongbirdResult<(f64, f64, u64)> {
        // Pre-generate access keys for performance (avoid allocations during benchmark)
        let access_keys = self.generate_realistic_access_pattern();

        let get_start = Instant::now();
        let mut hits = 0;
        let mut access_times = Vec::with_capacity(access_keys.len());

        for key in &access_keys {
            let access_start = Instant::now();

            if cache.get(key).await.is_some() {
                hits += 1;
            }
            access_times.push(access_start.elapsed().as_nanos() as u64);
        }

        let get_duration = get_start.elapsed();
        let get_ops_per_second = access_keys.len() as f64 / get_duration.as_secs_f64();

        // Calculate statistics
        let hit_rate_percentage = (hits as f64 / access_keys.len() as f64) * 100.0;
        let average_access_time_ns = if !access_times.is_empty() {
            access_times.iter().sum::<u64>() / access_times.len() as u64
        } else {
            0
        };

        Ok((
            get_ops_per_second,
            hit_rate_percentage,
            average_access_time_ns,
        ))
    }

    /// Generate realistic access pattern following 80/20 rule
    fn generate_realistic_access_pattern(&self) -> Vec<String> {
        let access_count = self.config.cache_test_data_size * 2;
        let mut access_keys = Vec::with_capacity(access_count);

        for i in 0..access_count {
            // 80/20 rule: 80% of accesses to 20% of data
            let key = if i % 5 == 0 {
                format!("key-{}", i % (self.config.cache_test_data_size / 5))
            } else {
                format!("key-{}", i % self.config.cache_test_data_size)
            };
            access_keys.push(key);
        }

        access_keys
    }

    /// Print benchmark results
    fn print_results(
        &self,
        get_ops_per_second: f64,
        put_ops_per_second: f64,
        hit_rate_percentage: f64,
    ) {
        println!("  GET: {get_ops_per_second:.0} ops/sec");
        println!("  PUT: {put_ops_per_second:.0} ops/sec");
        println!("  Hit Rate: {hit_rate_percentage:.1}%");
    }

    /// Run cache eviction policy benchmark
    pub async fn benchmark_eviction_policies(&self) -> SongbirdResult<CacheEvictionBenchmark> {
        println!("🔄 Benchmarking Cache Eviction Policies...");

        // Test LRU eviction
        let lru_results = self.test_lru_eviction().await?;

        // Test LFU eviction (simulated)
        let lfu_results = self.test_lfu_eviction().await?;

        // Test adaptive eviction
        let adaptive_results = self.test_adaptive_eviction().await?;

        Ok(CacheEvictionBenchmark {
            lru_hit_rate: lru_results,
            lfu_hit_rate: lfu_results,
            adaptive_hit_rate: adaptive_results,
        })
    }

    /// Test LRU eviction performance
    async fn test_lru_eviction(&self) -> SongbirdResult<f64> {
        let cache = self.performance_optimizer.create_cache::<String, String>();

        // Fill cache to capacity
        let cache_size = self.config.cache_test_data_size / 4; // Smaller for eviction testing
        for i in 0..cache_size * 2 {
            // Overfill to trigger eviction
            cache
                .put(format!("lru-key-{i}"), format!("value-{i}"), None)
                .await;
        }

        // Access pattern that should work well with LRU
        let mut hits = 0;
        for i in 0..cache_size {
            if cache
                .get(&format!("lru-key-{}", cache_size + i))
                .await
                .is_some()
            {
                hits += 1;
            }
        }

        Ok(hits as f64 / cache_size as f64 * 100.0)
    }

    /// Test LFU eviction performance (simulated)
    async fn test_lfu_eviction(&self) -> SongbirdResult<f64> {
        // Simulate LFU behavior - in practice would use different cache
        let cache = self.performance_optimizer.create_cache::<String, String>();

        // Access pattern that favors frequency
        let cache_size = self.config.cache_test_data_size / 4;

        // Insert and access some keys multiple times
        for i in 0..cache_size / 2 {
            cache
                .put(format!("lfu-key-{i}"), format!("value-{i}"), None)
                .await;
            // Access frequently used items multiple times
            for _ in 0..3 {
                let _ = cache.get(&format!("lfu-key-{i}")).await;
            }
        }

        // Fill remaining space
        for i in cache_size / 2..cache_size * 2 {
            cache
                .put(format!("lfu-key-{i}"), format!("value-{i}"), None)
                .await;
        }

        // Check if frequently accessed items are still there
        let mut hits = 0;
        for i in 0..cache_size / 2 {
            if cache.get(&format!("lfu-key-{i}")).await.is_some() {
                hits += 1;
            }
        }

        Ok(hits as f64 / (cache_size / 2) as f64 * 100.0)
    }

    /// Test adaptive eviction performance
    async fn test_adaptive_eviction(&self) -> SongbirdResult<f64> {
        let cache = self.performance_optimizer.create_cache::<String, String>();

        // Mixed access pattern
        let cache_size = self.config.cache_test_data_size / 4;

        // Add items with different access patterns
        for i in 0..cache_size * 2 {
            cache
                .put(format!("adaptive-key-{i}"), format!("value-{i}"), None)
                .await;

            // Some items accessed immediately (recency)
            if i % 3 == 0 {
                let _ = cache.get(&format!("adaptive-key-{i}")).await;
            }

            // Some items accessed multiple times (frequency)
            if i % 7 == 0 {
                for _ in 0..2 {
                    let _ = cache.get(&format!("adaptive-key-{i}")).await;
                }
            }
        }

        // Test both recent and frequent items
        let mut hits = 0;
        let test_count = cache_size / 2;

        for i in 0..test_count {
            let key = format!("adaptive-key-{}", cache_size + i);
            if cache.get(&key).await.is_some() {
                hits += 1;
            }
        }

        Ok(hits as f64 / test_count as f64 * 100.0)
    }
}

/// Cache eviction policy benchmark results
#[derive(Debug, Clone)]
pub struct CacheEvictionBenchmark {
    pub lru_hit_rate: f64,
    pub lfu_hit_rate: f64,
    pub adaptive_hit_rate: f64,
}
