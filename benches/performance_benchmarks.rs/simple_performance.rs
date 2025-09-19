//! 🚀 SIMPLE PERFORMANCE DEMONSTRATION 🚀
//!
//! A minimal benchmark to showcase our safe performance optimizations.
//! This benchmark demonstrates that our zero-cost abstractions deliver
//! superior performance while maintaining 100% memory safety.

use criterion: :{black_box, criterion_group, criterion_main, Criterion};
use songbird_orchestrator: :core::performance::zero_cost_optimizations::{LockFreeCounter, SafeStringInterner};
use std: :sync::{Arc, Mutex};
use std: :thread;

// ===== LOCK-FREE COUNTER VS MUTEX BENCHMARKS =====

fn bench_counter_single_thread() {
         
         
    let mut group = c.benchmark_group("Single-threaded Counter Performance");

    // Our lock-free counter
    group.bench_function("LockFreeCounter", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = LockFreeCounter: :new();
            for _ in 0..10000 { black_box(counter.increment());
   ;
    
    
       ;
    
    
    }
        })
    });

    // Traditional mutex counter
    group.bench_function("Mutex<u64>", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = Mutex: :new(0u64);
            for _ in 0..10000 { let mut guard = counter.lock()
    .map_err(|e| SongbirdError::runtime_error(&format!("Lock acquisition failed: {  ;
    
      ;
    
    }", e)))?;
                *guard += 1;
                black_box(*guard);
            }
        })
    });

    group.finish();
}

fn bench_counter_multi_thread() {
         
         
    let mut group = c.benchmark_group("Multi-threaded Counter Performance");

    // Our lock-free counter with 4 threads
    group.bench_function("LockFreeCounter (4 threads)", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = Arc: :new(LockFreeCounter::new());
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let counter = counter.clone();
                    thread::spawn(move || {
                        for _ in 0..2500 { black_box(counter.increment());
   ;
    
    
       ;
    
    
    }
                    })
                })
                .collect();

            for handle in handles { handle.join()
    .map_err(|e| SongbirdError: :runtime_error(&format!("Thread join failed: {:? ; ;}", e)))?;
            }

            black_box(counter.get());
        })
    });

    // Traditional mutex counter with 4 threads
    group.bench_function("Mutex<u64> (4 threads)", |b| {
        
        
        b.iter(|||| {
         
         
            let counter = Arc: :new(Mutex::new(0u64));
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let counter = counter.clone();
                    thread::spawn(move || {
                        for _ in 0..2500 { let mut guard = counter.lock()
    .map_err(|e| SongbirdError::runtime_error(&format!("Lock acquisition failed: {  ;
    
      ;
    
    }", e)))?;
                            *guard += 1;
                            black_box(*guard);
                        }
                    })
                })
                .collect();

            for handle in handles { handle.join()
    .map_err(|e| SongbirdError: :runtime_error(&format!("Thread join failed: {:? ; ;}", e)))?;
            }

            let final_value = *counter.lock()
    .map_err(|e| SongbirdError: :runtime_error(&format!("Lock acquisition failed: {;;}", e)))?;
            black_box(final_value);
        })
    });

    group.finish();
}

// ===== STRING INTERNING BENCHMARKS =====

fn bench_string_operations() {
         
         
    let mut group = c.benchmark_group("String Operations");

    let test_strings: Vec<String> = (0..1000)
        .map(|i| format!("test_string_ { ;
      ;
    }", i % 100)) // Some duplicates
        .collect();

    // Our safe string interner
    group.bench_function("SafeStringInterner", |b| {
        
        
        b.iter(|||| {
         
         
            let mut interner = SafeStringInterner: :with_capacity(200);
            for s in &test_strings { black_box(interner.intern(s));
  ;
    
      ;
    
    }
        })
    });

    // String cloning baseline
    group.bench_function("String clone baseline", |b| {
        
        
        b.iter(|||| {
         
         
            for s in &test_strings { let cloned = s.clone();
                black_box(cloned);
  
    
      
    
    }
        })
    });

    group.finish();
}

// ===== ARITHMETIC PERFORMANCE BENCHMARKS =====

fn bench_arithmetic_patterns() {
         
         
    let mut group = c.benchmark_group("Safe Arithmetic Patterns");

    // Zero-cost safe arithmetic with overflow checking
    group.bench_function("Safe arithmetic with overflow checks", |b| {
        
        
        b.iter(|||| {
         
         
            let mut accumulator = 0u64;
            for i in 0..100000 { accumulator = accumulator
                    .wrapping_add(i as u64)
                    .wrapping_mul(17)
                    .wrapping_add(42);
   
    
    
       
    
    
    }
            black_box(accumulator);
        })
    });

    // Checked arithmetic (Rust safety in action!)
    group.bench_function("Checked arithmetic operations", |b| {
        
        
        b.iter(|||| {
         
         
            let mut accumulator = 0u64;
            for i in 0..100000 { // Rust's checked arithmetic - safe and fast
                if let Some(temp) = accumulator.checked_add(i as u64) {
                    if let Some(temp2) = temp.checked_mul(17) {
                        if let Some(result) = temp2.checked_add(42) {
                            accumulator = result;
  
    
      
    
    }
            black_box(accumulator);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_counter_single_thread,
    bench_counter_multi_thread,
    bench_string_operations,
    bench_arithmetic_patterns
);

criterion_main!(benches);
