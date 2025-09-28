//! 🔥 ZERO-COST ABSTRACTIONS PERFORMANCE BENCHMARKS 🔥
//!
//! Professional benchmarking of our safe performance optimizations using Criterion.
//! These benchmarks demonstrate Rust's superpower: FAST AND SAFE, NEVER FAST OR SAFE.
//!
//! ## Benchmarks Included:
//! - Safe String Interning vs HashMap<String, usize>
//! - Lock-Free Counters vs Mutex<u64>
//! - Fixed Circular Buffers vs VecDeque
//! - Buffer Pool Reuse vs Vec::new() allocations
//! - Zero-allocation patterns vs standard allocation patterns
//! - Memory access patterns and cache efficiency

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use songbird_core::performance::zero_cost_optimizations::{
    FixedCircularBuffer, LockFreeCounter, PerformanceMeasurement, SafeBufferPool,
    SafeStringInterner,
};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ===== SAFE STRING INTERNING BENCHMARKS =====

fn bench_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Interning");
    group.throughput(Throughput::Elements(1000));

    let test_strings: Vec<String> = (0..1000)
        .map(|i| format!("test_string_{}", i % 100)) // Some duplicates for realistic interning
        .collect();

    // Our safe string interner
    group.bench_function("SafeStringInterner", |b| {
        b.iter(|| {
            let mut interner = SafeStringInterner::new();
            for s in &test_strings {
                black_box(interner.intern(s));
            }
        })
    });

    // Naive HashMap approach
    group.bench_function("HashMap<String, usize>", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            let mut next_id = 0;
            for s in &test_strings {
                if !map.contains_key(s) {
                    map.insert(s.clone(), next_id);
                    next_id += 1;
                }
                black_box(map.get(s));
            }
        })
    });

    // String cloning baseline (worst case)
    group.bench_function("String::clone() baseline", |b| {
        b.iter(|| {
            for s in &test_strings {
                black_box(s.clone());
            }
        })
    });

    group.finish();
}

// ===== LOCK-FREE COUNTER BENCHMARKS =====

fn bench_lock_free_counters(c: &mut Criterion) {
    let mut group = c.benchmark_group("Concurrent Counters");
    group.throughput(Throughput::Elements(10000));

    // Single-threaded comparison
    group.bench_function("LockFreeCounter (single-thread)", |b| {
        b.iter(|| {
            let counter = LockFreeCounter::new();
            for _ in 0..10000 {
                black_box(counter.increment());
            }
        })
    });

    group.bench_function("Mutex<u64> (single-thread)", |b| {
        b.iter(|| {
            let counter = Mutex::new(0u64);
            for _ in 0..10000 {
                let mut guard = counter.lock()
    .map_err(|e| SongbirdError::runtime_error(&format!("Lock acquisition failed: {}", e)))?;
                *guard += 1;
                black_box(*guard);
            }
        })
    });

    // Multi-threaded comparison
    for threads in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("LockFreeCounter (multi-thread)", threads),
            threads,
            |b, &num_threads| {
                b.iter(|| {
                    let counter = Arc::new(LockFreeCounter::new());
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = counter.clone();
                            thread::spawn(move || {
                                for _ in 0..(10000 / num_threads) {
                                    black_box(counter.increment());
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join()
    .map_err(|e| SongbirdError::runtime_error(&format!("Thread join failed: {:?}", e)))?;
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Mutex<u64> (multi-thread)", threads),
            threads,
            |b, &num_threads| {
                b.iter(|| {
                    let counter = Arc::new(Mutex::new(0u64));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = counter.clone();
                            thread::spawn(move || {
                                for _ in 0..(10000 / num_threads) {
                                    let mut guard = counter.lock()
    .map_err(|e| SongbirdError::runtime_error(&format!("Lock acquisition failed: {}", e)))?;
                                    *guard += 1;
                                    black_box(*guard);
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join()
    .map_err(|e| SongbirdError::runtime_error(&format!("Thread join failed: {:?}", e)))?;
                    }
                })
            },
        );
    }

    group.finish();
}

// ===== CIRCULAR BUFFER BENCHMARKS =====

fn bench_circular_buffers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Circular Buffers");
    group.throughput(Throughput::Elements(10000));

    const BUFFER_SIZE: usize = 1024;
    let test_data: Vec<u32> = (0..10000).collect();

    // Our fixed circular buffer (zero heap allocation after init)
    group.bench_function("FixedCircularBuffer<u32, 1024>", |b| {
        b.iter(|| {
            let mut buffer = FixedCircularBuffer::<u32, BUFFER_SIZE>::new();
            for &item in &test_data {
                black_box(buffer.push(item));
            }
        })
    });

    // VecDeque with capacity pre-allocation
    group.bench_function("VecDeque<u32> (pre-allocated)", |b| {
        b.iter(|| {
            let mut buffer = VecDeque::with_capacity(BUFFER_SIZE);
            for &item in &test_data {
                if buffer.len() == BUFFER_SIZE {
                    buffer.pop_front();
                }
                buffer.push_back(item);
                black_box(&buffer);
            }
        })
    });

    // VecDeque without pre-allocation (worst case)
    group.bench_function("VecDeque<u32> (dynamic)", |b| {
        b.iter(|| {
            let mut buffer = VecDeque::new();
            for &item in &test_data {
                if buffer.len() == BUFFER_SIZE {
                    buffer.pop_front();
                }
                buffer.push_back(item);
                black_box(&buffer);
            }
        })
    });

    group.finish();
}

// ===== BUFFER POOL BENCHMARKS =====

fn bench_buffer_pools(c: &mut Criterion) {
    let mut group = c.benchmark_group("Buffer Allocation");
    group.throughput(Throughput::Elements(1000));

    const BUFFER_SIZE: usize = 4096;

    // Our safe buffer pool (zero allocation after warmup)
    group.bench_function("SafeBufferPool<u8> reuse", |b| {
        let mut pool = SafeBufferPool::new();
        // Warmup the pool
        for _ in 0..10 {
            let _ = pool.get_buffer(BUFFER_SIZE);
        }

        b.iter(|| {
            for _ in 0..1000 {
                let buffer = pool.get_buffer(BUFFER_SIZE);
                // Simulate some work
                black_box(&buffer);
            }
        })
    });

    // Direct Vec allocation (baseline)
    group.bench_function("Vec::with_capacity() new", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let buffer: Vec<u8> = Vec::with_capacity(BUFFER_SIZE);
                black_box(&buffer);
            }
        })
    });

    // Zeroed allocation
    group.bench_function("vec![0u8; 4096] zeroed", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let buffer = vec![0u8; BUFFER_SIZE];
                black_box(&buffer);
            }
        })
    });

    group.finish();
}

// ===== MEMORY ACCESS PATTERN BENCHMARKS =====

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Access Patterns");

    const SIZE: usize = 1024 * 1024; // 1MB of u32s
    let data: Vec<u32> = (0..SIZE as u32).collect();

    // Sequential access (cache-friendly)
    group.bench_function("Sequential Access", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for &value in &data {
                sum += value as u64;
            }
            black_box(sum);
        })
    });

    // Stride access patterns
    for stride in [2, 4, 8, 16, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("Stride Access", stride),
            stride,
            |b, &stride| {
                b.iter(|| {
                    let mut sum = 0u64;
                    let mut i = 0;
                    while i < data.len() {
                        sum += data[i] as u64;
                        i += stride;
                    }
                    black_box(sum);
                })
            },
        );
    }

    // Random access (cache-unfriendly, but realistic for some workloads)
    group.bench_function("Random Access", |b| {
        // Pre-compute random indices to avoid RNG overhead in benchmark
        let indices: Vec<usize> = (0..10000)
            .map(|i| (i * 17 + 42) % data.len()) // Simple pseudo-random
            .collect();

        b.iter(|| {
            let mut sum = 0u64;
            for &idx in &indices {
                sum += data[idx] as u64;
            }
            black_box(sum);
        })
    });

    group.finish();
}

// ===== ZERO-ALLOCATION PATTERNS BENCHMARK =====

fn bench_zero_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Zero-Allocation Patterns");
    group.throughput(Throughput::Elements(10000));

    // Configuration processing with stack allocation
    group.bench_function("Stack-allocated config processing", |b| {
        b.iter(|| {
            let mut measurement = PerformanceMeasurement::new("benchmark");
            for i in 0..10000 {
                // Simulate config processing with no heap allocation
                let config_value = i * 2 + 1;
                let processed = config_value.wrapping_mul(17).wrapping_add(42);
                black_box(processed);
            }
            black_box(measurement.ops_per_second());
        })
    });

    // String formatting (unavoidably allocates)
    group.bench_function("String formatting (allocating)", |b| {
        b.iter(|| {
            for i in 0..10000 {
                let formatted = format!("config_value_{}", i);
                black_box(formatted);
            }
        })
    });

    // Arithmetic operations only
    group.bench_function("Pure arithmetic (zero-allocation)", |b| {
        b.iter(|| {
            let mut accumulator = 0u64;
            for i in 0..10000 {
                accumulator = accumulator
                    .wrapping_add(i as u64)
                    .wrapping_mul(17)
                    .wrapping_add(42);
            }
            black_box(accumulator);
        })
    });

    group.finish();
}

// ===== REALISTIC WORKLOAD BENCHMARKS =====

fn bench_realistic_workloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("Realistic Workloads");

    // Simulate a configuration update cycle
    group.bench_function("Config Update Cycle", |b| {
        let mut interner = SafeStringInterner::new();
        let mut counter = LockFreeCounter::new();
        let mut buffer = FixedCircularBuffer::<u32, 256>::new();
        let mut pool = SafeBufferPool::new();

        // Warmup
        for i in 0..100 {
            interner.intern(&format!("config_key_{}", i % 20));
            pool.get_buffer(1024);
        }

        b.iter(|| {
            // Simulate receiving 100 configuration updates
            for i in 0..100 {
                // Intern configuration keys
                let key_id = interner.intern(&format!("config_key_{}", i % 20));

                // Update metrics
                counter.increment();

                // Buffer the change
                buffer.push(i as u32);

                // Process with temporary buffer
                let temp_buffer = pool.get_buffer(512);

                black_box((key_id, counter.get(), temp_buffer.len()));
            }
        })
    });

    // Simulate a high-throughput message processing scenario
    group.bench_function("Message Processing Pipeline", |b| {
        let counter = Arc::new(LockFreeCounter::new());
        let mut buffers: Vec<FixedCircularBuffer<u64, 128>> =
            (0..4).map(|_| FixedCircularBuffer::new()).collect();

        b.iter(|| {
            // Process 1000 messages across 4 pipelines
            for i in 0..1000u64 {
                let pipeline = i % 4;

                // Route message to appropriate pipeline buffer
                buffers[pipeline as usize].push(i);

                // Update global counter
                counter.increment();

                // Simulate some processing
                let processed = i.wrapping_mul(17).wrapping_add(42);
                black_box(processed);
            }

            black_box(counter.get());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_string_interning,
    bench_lock_free_counters,
    bench_circular_buffers,
    bench_buffer_pools,
    bench_memory_patterns,
    bench_zero_allocation_patterns,
    bench_realistic_workloads
);

criterion_main!(benches);
