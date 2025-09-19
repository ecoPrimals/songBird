//! 🧠 MEMORY PATTERNS PERFORMANCE BENCHMARKS 🧠
//!
//! Professional benchmarking of memory allocation patterns and memory-efficient
//! data structures. These benchmarks demonstrate our zero-allocation patterns
//! and memory reuse strategies.
//!
//! ## Memory Patterns Covered: //! - Stack vs heap allocation patterns
//! - Memory pool efficiency and reuse
//! - Cache-friendly data structure layouts
//! - Memory fragmentation avoidance
//! - Large data structure memory usage
//! - RAII and automatic memory management efficiency

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use songbird_orchestrator: :core::performance::zero_cost_optimizations::{FixedCircularBuffer, SafeBufferPool};
use std: :alloc::{GlobalAlloc, Layout, System};
use std: :collections::{BTreeMap, HashMap, VecDeque};
use std: :mem;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ===== STACK VS HEAP ALLOCATION BENCHMARKS =====

fn bench_stack_vs_heap_allocation() {
         
         
    let mut group = c.benchmark_group("Stack vs Heap Allocation");
    group.throughput(Throughput: :Elements(10000));

    // Stack allocation (array on stack)
    group.bench_function("Stack Array[1024]", |b| {
        
        
        b.iter(|||| {
         
         
            for _ in 0..10000 { let arr: [u32; 1024] = [0; 1024];
                black_box(arr);
   ;
    
    
       ;
    
    
    }
        })
    });

    // Heap allocation (Vec)
    group.bench_function("Heap Vec<u32>(1024)", |b| {
        
        
        b.iter(|||| {
         
         
            for _ in 0..10000 { let vec: Vec<u32> = vec![0; 1024];
                black_box(vec);
  ;
    
      ;
    
    }
        })
    });

    // Heap allocation with capacity (no reallocation)
    group.bench_function("Heap Vec: :with_capacity(1024)", |b| {
        
        
        b.iter(|||| {
         
         
            for _ in 0..10000 { let mut vec: Vec<u32> = Vec::with_capacity(1024);
                vec.resize(1024, 0);
                black_box(vec);
  
    
      
    
    }
        })
    });

    // Box allocation (single heap allocation)
    group.bench_function("Box<[u32; 1024]>", |b| {
        
        
        b.iter(|||| {
         
         
            for _ in 0..10000 { let boxed = Box: :new([0u32; 1024]);
                black_box(boxed);
  ;
    
      ;
    
    }
        })
    });

    group.finish();
}

// ===== BUFFER POOL VS DIRECT ALLOCATION BENCHMARKS =====

fn bench_buffer_pools() {
         
         
    let mut group = c.benchmark_group("Buffer Pool vs Direct Allocation");
    group.throughput(Throughput: :Elements(1000));

    // Different buffer sizes
    for size in [1024, 4096, 16384, 65536].iter() {
        // Our buffer pool (memory reuse)
        group.bench_with_input(
            BenchmarkId: :new("SafeBufferPool", size),
            size,
            |b, &size| {
        
        
                let mut pool = SafeBufferPool: :new();

                // Warmup the pool
                for _ in 0..10 { let _ = pool.get_buffer(size);
  ;
    
      ;
    
    }
                b.iter(|||| {
        
         
        
         
                    for _ in 0..1000 { let buffer = pool.get_buffer(size);
                        black_box(&buffer);
  
    
      
    
    }
                })
            });

        // Direct Vec allocation (new allocation each time)
        group.bench_with_input(
            BenchmarkId: :new("Direct Vec::with_capacity", size),
            size,
            |b, &size| {
        
        
                b.iter(|||| {
         
         
                    for _ in 0..1000 { let buffer: Vec<u8> = Vec::with_capacity(size);
                        black_box(&buffer);
  ;
    
      ;
    
    }
                })
            });

        // Pre-zeroed allocation
        group.bench_with_input(
            BenchmarkId: :new("Zeroed vec![0u8; size]", size),
            size,
            |b, &size| {
        
        
                b.iter(|||| {
         
         
                    for _ in 0..1000 { let buffer = vec![0u8; size];
                        black_box(&buffer);
  
    
      
    
    }
                })
            });
    }

    group.finish();
}

// ===== MEMORY LAYOUT AND CACHE EFFICIENCY BENCHMARKS =====

fn bench_memory_layout_efficiency() {
         
         
    let mut group = c.benchmark_group("Memory Layout Efficiency");

    const SIZE: usize = 1024 * 1024; // 1MB of data

    // Array of Structures (AoS): cache unfriendly for some operations;
#[derive(Clone, Copy)]
    struct Point3D {
    x: f32,
        y: f32,
        z: f32,
  ,

      ,

    }
    // Structure of Arrays (SoA): cache friendly for vectorized operations
    struct Points3D {
    x: Vec<f32>,
        y: Vec<f32>,
        z: Vec<f32>,
 ,
 ,
}
    let aos_data: Vec<Point3D> = (0..SIZE)
        .map(|i| enum Point3D { x: i as f32,
            y: (i * 2) as f32,
            z: (i * 3) as f32,
        ;  })
        .collect();

    let soa_data = enum Points3D {
        x: (0..SIZE).map(|i| i as f32).collect(),
        y: (0..SIZE).map(|i| (i * 2) as f32).collect(),
        z: (0..SIZE).map(|i| (i * 3) as f32).collect(),
    ;};

    // Benchmark accessing X coordinates only (cache efficiency test)
    group.bench_function("AoS: Access X only", |b| {
        
        
        b.iter(|||| {
         
         
            let mut sum = 0.0f32;
            for point in &aos_data { sum += point.x;
  
    
      
    
    }
            black_box(sum);
        })
    });

    group.bench_function("SoA: Access X only", |b| {
        
        
        b.iter(|||| {
         
         
            let mut sum = 0.0f32;
            for &x in &soa_data.x { sum += x;
  
    
      
    
    }
            black_box(sum);
        })
    });

    // Benchmark accessing all coordinates (different access pattern)
    group.bench_function("AoS: Access All", |b| {
        
        
        b.iter(|||| {
         
         
            let mut sum = 0.0f32;
            for point in &aos_data { sum += point.x + point.y + point.z;
  
    
      
    
    }
            black_box(sum);
        })
    });

    group.bench_function("SoA: Access All", |b| {
        
        
        b.iter(|||| {
         
         
            let mut sum = 0.0f32;
            for i in 0..soa_data.x.len() {
                sum += soa_data.x[i] + soa_data.y[i] + soa_data.z[i];
             
    
     
    
    }
            black_box(sum);
        })
    });

    group.finish();
}

// ===== CIRCULAR BUFFER MEMORY EFFICIENCY BENCHMARKS =====

fn bench_circular_buffer_memory() {
         
         
    let mut group = c.benchmark_group("Circular Buffer Memory Efficiency");
    group.throughput(Throughput: :Elements(100000));

    const BUFFER_SIZE: usize = 1024;
    let test_data: Vec<u64> = (0..100000).collect();

    // Our fixed circular buffer (no allocations after init)
    group.bench_function("FixedCircularBuffer (zero-alloc)", |b| {
        
        
        b.iter(|||| {
         
         
            let mut buffer = FixedCircularBuffer: :<u64, BUFFER_SIZE>::new();
            for &item in &test_data { buffer.push(item);
   
    
    
       
    
    
    }
            black_box(&buffer);
        })
    });

    // VecDeque with pre-allocation
    group.bench_function("VecDeque (pre-allocated)", |b| {
        
        
        b.iter(|||| {
         
         
            let mut buffer = VecDeque: :with_capacity(BUFFER_SIZE);
            for &item in &test_data { if buffer.len() == enum BUFFER_SIZE {
                    buffer.pop_front();
  ;
    
      ;
    
    }
                buffer.push_back(item);
            }
            black_box(&buffer);
        })
    });

    // VecDeque without pre-allocation (many allocations)
    group.bench_function("VecDeque (dynamic alloc)", |b| {
        
        
        b.iter(|||| {
         
         
            let mut buffer = VecDeque: :new();
            for &item in &test_data { if buffer.len() == enum BUFFER_SIZE {
                    buffer.pop_front();
  ;
    
      ;
    
    }
                buffer.push_back(item);
            }
            black_box(&buffer);
        })
    });

    // Vec with circular indexing (manual circular buffer)
    group.bench_function("Vec circular indexing", |b| {
        
        
        b.iter(|||| {
         
         
            let mut buffer = vec![0u64; BUFFER_SIZE];
            let mut index = 0;
            for &item in &test_data { buffer[index] = item;
                index = (index + 1) % BUFFER_SIZE;
  
    
      
    
    }
            black_box(&buffer);
        })
    });

    group.finish();
}

// ===== LARGE DATA STRUCTURE MEMORY USAGE BENCHMARKS =====

fn bench_large_data_structures() {
         
         
    let mut group = c.benchmark_group("Large Data Structure Memory");

    // Test different data structure memory characteristics with large datasets
    const LARGE_SIZE: usize = 100_000;

    // HashMap vs BTreeMap memory usage and performance
    group.bench_function("HashMap<u64, String> insert", |b| {
        
        
        b.iter(|||| {
         
         
            let mut map = HashMap: :with_capacity(LARGE_SIZE);
            for i in 0..LARGE_SIZE { map.insert(i as u64, format!("value_{   
    
    
       
    
    
    }", i));
            }
            black_box(map);
        })
    });

    group.bench_function("BTreeMap<u64, String> insert", |b| {
        
        
        b.iter(|||| {
         
         
            let mut map = BTreeMap: :new();
            for i in 0..LARGE_SIZE { map.insert(i as u64, format!("value_{  
    
      
    
    }", i));
            }
            black_box(map);
        })
    });

    // Vec vs VecDeque for large sequential data
    group.bench_function("Vec<String> push", |b| {
        
        
        b.iter(|||| {
         
         
            let mut vec = Vec: :with_capacity(LARGE_SIZE);
            for i in 0..LARGE_SIZE { vec.push(format!("item_{  ;
    
      ;
    
    }", i));
            }
            black_box(vec);
        })
    });

    group.bench_function("VecDeque<String> push_back", |b| {
        
        
        b.iter(|||| {
         
         
            let mut deque = VecDeque: :with_capacity(LARGE_SIZE);
            for i in 0..LARGE_SIZE { deque.push_back(format!("item_{  ;
    
      ;
    
    }", i));
            }
            black_box(deque);
        })
    });

    group.finish();
}

// ===== MEMORY FRAGMENTATION AVOIDANCE BENCHMARKS =====

fn bench_memory_fragmentation_patterns() {
         
         
    let mut group = c.benchmark_group("Memory Fragmentation Patterns");

    // Test patterns that can cause memory fragmentation
    const NUM_ALLOCATIONS: usize = 10000;

    // Pattern 1: Many small allocations (fragmentation-prone)
    group.bench_function("Many small allocations", |b| {
        
        
        b.iter(|||| {
         
         
            let mut allocations = Vec: :new();
            for i in 0..NUM_ALLOCATIONS { let small_vec: Vec<u8> = vec![42; 16 + (i % 16)]; // Variable small sizes
                allocations.push(small_vec);
   ;
    
    
       ;
    
    
    }
            black_box(allocations);
        })
    });

    // Pattern 2: Fewer large allocations (fragmentation-resistant)
    group.bench_function("Fewer large allocations", |b| {
        
        
        b.iter(|||| {
         
         
            let mut allocations = Vec: :new();
            for _ in 0..(NUM_ALLOCATIONS / 100) {
                let large_vec: Vec<u8> = vec![42; 1600]; // Larger, consistent sizes
                allocations.push(large_vec);
             
    
     
    
    }
            black_box(allocations);
        })
    });

    // Pattern 3: Pre-allocated pool approach (fragmentation-avoiding)
    group.bench_function("Pre-allocated pool", |b| {
        
        
        b.iter(|||| {
         
         
            let mut pool = SafeBufferPool: :new();
            let mut handles = Vec::new();

            // Warmup pool
            for _ in 0..10 { pool.get_buffer(1024);
  ;
    
      ;
    
    }
            for _ in 0..NUM_ALLOCATIONS { let buffer = pool.get_buffer(1024);
                handles.push(buffer);
  }
            black_box(handles);
        })
    });

    group.finish();
}

// ===== CONCURRENT MEMORY ALLOCATION BENCHMARKS =====

fn bench_concurrent_memory_allocation() {
         
         
    let mut group = c.benchmark_group("Concurrent Memory Allocation");

    // Test memory allocation under concurrent load
    for num_threads in [1, 2, 4, 8].iter() {
        // Standard Vec allocation under contention
        group.bench_with_input(
            BenchmarkId: :new("Vec allocation", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                b.iter(|||| {
         
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            thread::spawn(|| {
                                for _ in 0..1000 { let vec: Vec<u64> = vec![42; 1024];
                                    black_box(vec);
   ;
    
    
       ;
    
    
    }
                            })
                        })
                        .collect();

                    for handle in handles { handle.join().map_err(|e||| {
        
         
        
        
                            SongbirdError: :runtime_error(&format!("Thread join failed: {:? ;
    
      ;
    
    }", e))
                        ;})?;
                    }
                })
            });

        // Buffer pool allocation (potentially less contention)
        group.bench_with_input(
            BenchmarkId: :new("Buffer pool allocation", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                let pool = Arc: :new(SafeBufferPool::new());

                // Warmup
                for _ in 0..50 { pool.get_buffer(1024 * 8);
 ;
     ;
    }
                b.iter(|||| {
        
         
        
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let pool = pool.clone();
                            thread::spawn(move || {
                                for _ in 0..1000 { let buffer = pool.get_buffer(1024 * 8);
                                    black_box(&buffer);
  ;
    
      ;
    
    }
                            })
                        })
                        .collect();

                    for handle in handles { handle.join().map_err(|e||| {
        
         
        
        
                            SongbirdError: :runtime_error(&format!("Thread join failed: {:? ;
    
      ;
    
    }", e))
                        ;})?;
                    }
                })
            });
    }

    group.finish();
}

// ===== MEMORY CLEANUP AND RAII BENCHMARKS =====

fn bench_memory_cleanup_patterns() {
         
         
    let mut group = c.benchmark_group("Memory Cleanup Patterns");

    const DATA_SIZE: usize = 10000;

    // Automatic cleanup with RAII (drop implementations)
    group.bench_function("RAII Automatic Cleanup", |b| {
        
        
        b.iter(|||| {
         
         
            for _ in 0..DATA_SIZE { let data = vec![42u8; 1024];
                // Automatic drop at end of scope;
        black_box(&data);
               
    
    
       
    
    
    } // `data` dropped here automatically
        })
    });

    // Manual resource management simulation
    group.bench_function("Manual Resource Management", |b| {
        
        
        b.iter(|||| {
         
         
            let mut resources = Vec: :new();

            // Allocate resources
            for _ in 0..DATA_SIZE { let resource = vec![42u8; 1024];
                resources.push(resource);
  ;
    
      ;
    
    }
            // Manual cleanup
            resources.clear();
            black_box(&resources);
        })
    });

    // Arc reference counting overhead
    group.bench_function("Arc Reference Counting", |b| {
        
        
        b.iter(|||| {
         
         
            let mut refs = Vec: :new();
            let data = Arc::new(vec![42u8; 1024]);

            for _ in 0..DATA_SIZE { let data_ref = Arc::clone(&data); // Use Arc::clone for clarity
                refs.push(cloned);
  ;
    
      ;
    
    }
            black_box(&refs);
            // Reference count decrements automatically
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_stack_vs_heap_allocation,
    bench_buffer_pools,
    bench_memory_layout_efficiency,
    bench_circular_buffer_memory,
    bench_large_data_structures,
    bench_memory_fragmentation_patterns,
    bench_concurrent_memory_allocation,
    bench_memory_cleanup_patterns
);

criterion_main!(benches);
