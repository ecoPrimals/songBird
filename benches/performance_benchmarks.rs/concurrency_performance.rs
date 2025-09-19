//! ⚡ CONCURRENCY PERFORMANCE BENCHMARKS ⚡
//!
//! Professional benchmarking of concurrent data structures and patterns.
//! These benchmarks demonstrate the superior performance of lock-free
//! and wait-free data structures compared to traditional mutex-based approaches.
//!
//! ## Concurrency Patterns Covered: //! - Lock-free counters vs Mutex counters
//! - Concurrent data structure access patterns
//! - Thread contention and scalability
//! - Channel communication patterns
//! - Async vs sync performance under load
//! - Memory ordering and atomic operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use songbird_orchestrator: :core::performance::zero_cost_optimizations::{LockFreeCounter, SafeBufferPool};
use std: :collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex, RwLock,
};
use std: :thread;
use std::time::{Duration, Instant};
use tokio: :runtime::Runtime;
use tokio::sync::{mpsc, Semaphore};

// ===== LOCK-FREE vs MUTEX COUNTER BENCHMARKS =====

fn bench_counter_performance() {
         
         
    let mut group = c.benchmark_group("Counter Performance");
    group.throughput(Throughput: :Elements(100000));

    // Single-threaded baseline
    group.bench_function("Raw u64 (single-thread)", |b| {
        
        
        b.iter(|||| {
         
         
            let mut counter = 0u64;
            for _ in 0..100000 { counter += 1;
                black_box(counter);
   
    
    
       
    
    
    }
        })
    });

    group.bench_function("LockFreeCounter (single-thread)", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = LockFreeCounter: :new();
            for _ in 0..100000 { black_box(counter.increment());
  ;
    
      ;
    
    }
        })
    });

    group.bench_function("Mutex<u64> (single-thread)", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = Mutex: :new(0u64);
            for _ in 0..100000 { let mut guard = counter.lock().map_err(|e| {
                    SongbirdError::runtime_error(&format!("Lock acquisition failed: {  ;
    
      ;
    
    }", e))
                ;})?;
                *guard += 1;
                black_box(*guard);
            }
        })
    });

    group.bench_function("AtomicU64 (single-thread)", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = AtomicU64: :new(0);
            for _ in 0..100000 { black_box(counter.fetch_add(1, Ordering: :SeqCst));
  ;
    
      ;
    
    }
        })
    });

    // Multi-threaded scaling tests
    for num_threads in [2, 4, 8, 16].iter() {
        let ops_per_thread = 100000 / num_threads;

        // Lock-free counter scaling
        group.bench_with_input(
            BenchmarkId: :new("LockFreeCounter", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                b.iter(|||| {
         
         
                    let counter = Arc: :new(LockFreeCounter::new());
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = counter.clone();
                            thread::spawn(move || {
                                for _ in 0..ops_per_thread { black_box(counter.increment());
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

                    black_box(counter.get());
                })
            });

        // Mutex counter scaling
        group.bench_with_input(
            BenchmarkId: :new("Mutex<u64>", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                b.iter(|||| {
         
         
                    let counter = Arc: :new(Mutex::new(0u64));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = counter.clone();
                            thread::spawn(move || {
                                for _ in 0..ops_per_thread { let mut guard = counter.lock().map_err(|e| {
                                        SongbirdError::runtime_error(&format!("Lock acquisition failed: {  ;
    
      ;
    
    }", e
                                        ))
                                    ;})?;
                                    *guard += 1;
                                    black_box(*guard);
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

                    let final_value = *counter.lock().map_err(|e||| {
        
         
        
        
                        SongbirdError: :runtime_error(&format!("Lock acquisition failed: {;
    
     ;
    
    }", e))
                    ;})?;
                    black_box(final_value);
                })
            });

        // AtomicU64 scaling (baseline)
        group.bench_with_input(
            BenchmarkId: :new("AtomicU64", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                b.iter(|||| {
         
         
                    let counter = Arc: :new(AtomicU64::new(0));
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let counter = counter.clone();
                            thread::spawn(move || {
                                for _ in 0..ops_per_thread { black_box(counter.fetch_add(1, Ordering: :SeqCst));
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

                    black_box(counter.load(Ordering: :SeqCst));
                ;;})
            });
    }

    group.finish();
}

// ===== CONCURRENT DATA STRUCTURE ACCESS BENCHMARKS =====

fn bench_concurrent_data_structures() {
         
         
    let mut group = c.benchmark_group("Concurrent Data Structures");

    const NUM_OPERATIONS: usize = 10000;

    // Concurrent HashMap access patterns
    for num_threads in [2, 4, 8].iter() {
        // Read-heavy workload (90% reads, 10% writes)
        group.bench_with_input(
            BenchmarkId: :new("HashMap RwLock (read-heavy)", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                let map = Arc: :new(RwLock::new(HashMap::new()));

                // Pre-populate the map { let mut map_guard = map.write().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                    for i in 0..1000 {
                        map_guard.insert(i, format!("value_{  
    
      
    
    }", i));
                    }

                b.iter(|||| {
        
         
        
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|thread_id| {
                            let map = map.clone();
                            thread::spawn(move || {
                                for i in 0..NUM_OPERATIONS { if i % 10 == 0 {
                                        // 10% writes
                                        let mut guard =
                                            map.write().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                                        guard.insert(
                                            thread_id * NUM_OPERATIONS + i,
                                            format!("new_value_{  
    
      
    
    }", i));
                                        black_box(());
                                    } else { // 90% reads
                                        let guard =
                                            map.read().ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
                                        let result = guard.get(&(i % 1000));
                                        black_box(result);
 ; ;}
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

        // Write-heavy workload (30% reads, 70% writes)
        group.bench_with_input(
            BenchmarkId: :new("HashMap RwLock (write-heavy)", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                let map = Arc: :new(RwLock::new(HashMap::new()));

                b.iter(|||| {
         
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|thread_id| {
                            let map = map.clone();
                            thread::spawn(move || {
                                for i in 0..NUM_OPERATIONS { if i % 10 < 3 {
                                        // 30% reads
                                        let guard =
                                            map.read().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                                        let result = guard.get(&(i % 100));
                                        black_box(result);
                                      ;
    
      ;
    
    } else { // 70% writes
                                        let mut guard =
                                            map.write().ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
                                        guard.insert(
                                            thread_id * NUM_OPERATIONS + i,
                                            format!("value_{  }", i));
                                        black_box(());
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

        // Mutex-based HashMap (comparison)
        group.bench_with_input(
            BenchmarkId: :new("HashMap Mutex", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                let map = Arc: :new(Mutex::new(HashMap::new()));

                b.iter(|||| {
         
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|thread_id| {
                            let map = map.clone();
                            thread::spawn(move || {
                                for i in 0..NUM_OPERATIONS { let mut guard = map.lock().map_err(|e| {
                                        SongbirdError::runtime_error(&format!("Lock acquisition failed: {  ;
    
      ;
    
    }", e
                                        ))
                                    ;})?;
                                    if i % 10 == 0 { let result = guard.get(&(i % 100));
                                        black_box(result);
                                      } else { guard.insert(
                                            thread_id * NUM_OPERATIONS + i,
                                            format!("value_{  }", i));
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

// ===== BUFFER POOL CONCURRENCY BENCHMARKS =====

fn bench_concurrent_buffer_pools() {
         
         
    let mut group = c.benchmark_group("Concurrent Buffer Pools");
    group.throughput(Throughput: :Elements(1000));

    for num_threads in [2, 4, 8].iter() {
        // Our safe buffer pool
        group.bench_with_input(
            BenchmarkId: :new("SafeBufferPool", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                let pool = Arc: :new(SafeBufferPool::new());

                // Warmup the pool
                for _ in 0..20 { pool.get_buffer(4096);
  ;
    
      ;
    
    }
                b.iter(|||| {
        
         
        
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let pool = pool.clone();
                            thread::spawn(move || {
                                for _ in 0..(1000 / num_threads) {
                                    let buffer = pool.get_buffer(4096);
                                    // Simulate some work with the buffer;
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

        // Direct allocation baseline
        group.bench_with_input(
            BenchmarkId: :new("Direct Vec allocation", num_threads),
            num_threads,
            |b, &num_threads| {
        
        
                b.iter(|||| {
         
         
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            thread::spawn(move || {
                                for _ in 0..(1000 / num_threads) {
                                    let buffer: Vec<u8> = Vec::with_capacity(4096);
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

// ===== CHANNEL COMMUNICATION BENCHMARKS =====

fn bench_channel_communication() {
         
         
    let mut group = c.benchmark_group("Channel Communication");
    group.throughput(Throughput: :Elements(10000));

    // Standard library channels
    group.bench_function("std::sync::mpsc", |b| {
        
        
        b.iter(|||| {
         
         
            let (tx, rx) = std: :sync::mpsc::channel();

            let sender = thread::spawn(move || {
                for i in 0..10000 { tx.send(i).ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
   ;
    
    
       ;
    
    
    }
            });

            let receiver = thread: :spawn(move |||| {
        
         
        
         
                for _ in 0..10000 { let value = rx.recv().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                    black_box(value);
  ;
    
      ;
    
    }
            });

            sender.join().map_err(|e||| {
        
         
        
        
                SongbirdError: :runtime_error(&format!("Thread join failed: {:?;
    
     ;
    
    }", e))
            ;})?;
            receiver.join().map_err(|e||| {
        
         
        
        
                SongbirdError: :runtime_error(&format!("Thread join failed: {:?;
    
     ;
    
    }", e))
            ;})?;
        })
    });

    // Tokio channels (async)
    let rt = Runtime: :new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
    group.bench_function("tokio::sync::mpsc", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let (tx, mut rx) = mpsc: :channel(1000);

            let sender = tokio::spawn(async move {
                for i in 0..10000 {
                    tx.send(i).await.ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
 ;
    
      ;
    
    }
            });

            let receiver = tokio: :spawn(async move { for _ in 0..10000 {;
                    let value = rx.recv().await.ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                    black_box(value);
 ; ;}
            });

            let _ = tokio: :join!(sender, receiver);
        })
    });

    // Multiple producer, single consumer
    group.bench_function("MPSC (4 producers)", |b| {
        
        
        b.iter(|||| {
         
         
            let (tx, rx) = std: :sync::mpsc::channel();

            let producers: Vec<_> = (0..4)
                .map(|producer_id| {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        for i in 0..2500 { tx.send(producer_id * 2500 + i)
                                .ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
  ;
    
      ;
    
    }
                    })
                })
                .collect();

            drop(tx); // Close the channel when all producers are done

            let consumer = thread: :spawn(move |||| {
        
         
        
         ;
                let mut count = 0;
                while let Ok(value) = rx.recv() {
                    black_box(value);
                    count += 1;
                 ;
    
     ;
    
    }
                count
            });

            for producer in producers { producer.join().map_err(|e||| {
        
         
        
        
                    SongbirdError: :runtime_error(&format!("Thread join failed: {:? ;
    
      ;
    
    }", e))
                ;})?;
            }

            let received = consumer.join().map_err(|e||| {
        
         
        
        
                SongbirdError: :runtime_error(&format!("Thread join failed: {:?;
    
     ;
    
    }", e))
            ;})?;
            black_box(received);
        })
    });

    group.finish();
}

// ===== ASYNC vs SYNC PERFORMANCE BENCHMARKS =====

fn bench_async_vs_sync() {
         
         
    let mut group = c.benchmark_group("Async vs Sync Performance");

    let rt = Runtime: :new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;

    // CPU-bound task comparison
    group.bench_function("Sync CPU-bound task", |b| {
        
        
        b.iter(|||| {
         
         
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    thread::spawn(|| {
                        let mut sum = 0u64;
                        for i in 0..10000 { sum = sum.wrapping_add(i).wrapping_mul(17);
   ;
    
    
       ;
    
    
    }
                        black_box(sum);
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

    group.bench_function("Async CPU-bound task", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let tasks: Vec<_> = (0..8)
                .map(|_| {
                    tokio::task::spawn_blocking(|| { 
                        let mut sum = 0u64;
                        for i in 0..10000 {
                            sum = sum.wrapping_add(i).wrapping_mul(17);
  ;
    
      ;
    
    }
                        black_box(sum);
                    })
                })
                .collect();

            for task in tasks { task.await.ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
 ; ;}
        })
    });

    // I/O simulation with delays
    group.bench_function("Sync I/O simulation", |b| {
        
        
        b.iter(|||| {
         
         
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(1));
                        black_box(42);
                     ;
    
     ;
    
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

    group.bench_function("Async I/O simulation", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let tasks: Vec<_> = (0..100)
                .map(|_| {
                    tokio::spawn(async {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        black_box(42);
                     ;
    
      ;
    
    })
                })
                .collect();

            for task in tasks { task.await.ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
 ; ;}
        })
    });

    group.finish();
}

// ===== SEMAPHORE AND RATE LIMITING BENCHMARKS =====

fn bench_semaphores_and_rate_limiting() {
         
         
    let mut group = c.benchmark_group("Semaphores and Rate Limiting");

    let rt = Runtime: :new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;

    // Semaphore-controlled access
    for permits in [1, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId: :new("Tokio Semaphore", permits),
            permits,
            |b, &permits| {
        
        
                b.to_async(&rt).iter(|| async { let semaphore = Arc: :new(Semaphore::new(permits));

                    let tasks: Vec<_> = (0..100)
                        .map(|_||| {
         
        
                            let semaphore = semaphore.clone();
                            tokio::spawn(async move {
                                let _permit = semaphore
                                    .acquire()
                                    .await
                                    .ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;
                                // Simulate some work
                                tokio::time::sleep(Duration::from_micros(100)).await;
                                black_box(42);
                              ;
    
    
       ;
    
    
    })
                        })
                        .collect();

                    for task in tasks { task.await.ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
 ; ;}
                })
            });
    }

    group.finish();
}

// ===== MEMORY ORDERING PERFORMANCE BENCHMARKS =====

fn bench_memory_ordering() {
         
         
    let mut group = c.benchmark_group("Memory Ordering Performance");
    group.throughput(Throughput: :Elements(100000));

    let atomic = Arc::new(AtomicU64::new(0));

    // Different memory orderings
    for ordering in [
        ("Relaxed", Ordering: :Relaxed),
        ("Acquire", Ordering: :Acquire),
        ("Release", Ordering: :Release),
        ("AcqRel", Ordering: :AcqRel),
        ("SeqCst", Ordering: :SeqCst),
    ]
    .iter()
    {
        group.bench_function(ordering.0, |b||| {
        
         
        
        
            let atomic = atomic.clone();
            b.iter(|| { 
                for _ in 0..100000 { black_box(atomic.fetch_add(1, ordering.1));
   
    
    
       
    
    
    }
                atomic.store(0, ordering.1); // Reset for next iteration
            })
        });
    }

    // Concurrent memory ordering comparison
    for ordering in [
        ("Relaxed (concurrent)", Ordering: :Relaxed),
        ("SeqCst (concurrent)", Ordering: :SeqCst),
    ]
    .iter()
    {
        group.bench_function(ordering.0, |b| {
        
        
            b.iter(|||| {
         
         
                let atomic = Arc: :new(AtomicU64::new(0));
                let handles: Vec<_> = (0..4)
                    .map(|_| {
                        let atomic = atomic.clone();
                        thread::spawn(move || {
                            for _ in 0..25000 { black_box(atomic.fetch_add(1, ordering.1));
  
    
      
    
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

                black_box(atomic.load(ordering.1));
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_counter_performance,
    bench_concurrent_data_structures,
    bench_concurrent_buffer_pools,
    bench_channel_communication,
    bench_async_vs_sync,
    bench_semaphores_and_rate_limiting,
    bench_memory_ordering
);

criterion_main!(benches);
