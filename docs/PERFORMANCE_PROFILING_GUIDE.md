# 🔥 Performance Profiling Guide
## Hot Path Analysis and Optimization for Songbird

**Version**: 1.0  
**Date**: October 27, 2025  
**Purpose**: Guide for profiling and optimizing performance hot paths

---

## 📊 Overview

This guide provides a systematic approach to identifying and optimizing performance hot paths in Songbird. Current estimates suggest ~60 clones in hot paths that could benefit from zero-copy optimization.

---

## 🎯 Goals

1. **Identify true hot paths** using flamegraph profiling
2. **Measure baseline performance** with benchmarks
3. **Optimize clone operations** where impactful
4. **Validate improvements** with before/after metrics
5. **Maintain correctness** throughout optimization

---

## 🛠️ Tool Setup

### Install cargo-flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# On Linux, may need perf
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`

# Verify installation
flamegraph --version
```

### Alternative: cargo-profiler

```bash
# Alternative profiling tool
cargo install cargo-profiler

# Usage
cargo profiler callgrind --bin=songbird
```

### Install cargo-benchcmp

```bash
# For comparing benchmark results
cargo install cargo-benchcmp
```

---

## 📈 Profiling Workflow

### Step 1: Run Baseline Benchmarks

```bash
# Run all benchmarks and save baseline
cd /home/eastgate/Development/ecoPrimals/songbird
cargo bench --all -- --save-baseline before_optimization

# Or run specific benchmark suite
cargo bench --bench critical_path_benchmarks -- --save-baseline before
```

### Step 2: Generate Flamegraph

```bash
# Profile the application under realistic load
sudo flamegraph -o flamegraph.svg -- \
    cargo run --release --bin songbird -- \
    --config config/production.env \
    --duration 60

# For specific benchmark
sudo flamegraph -o bench_flamegraph.svg -- \
    cargo bench --bench critical_path_benchmarks --no-run
```

### Step 3: Analyze Flamegraph

Open `flamegraph.svg` in a browser and look for:
1. **Wide bars** = hot paths (frequently executed)
2. **Tall stacks** = deep call chains
3. **Clone operations** in hot paths
4. **Allocation patterns** (look for `alloc::` calls)

### Step 4: Identify Optimization Targets

Priority order:
1. Clones in hot paths (>1% of total time)
2. Unnecessary allocations
3. Inefficient data structures
4. Redundant computations

---

## 🎯 Known Hot Path Candidates

### High Priority (Estimated)

Based on code analysis, these are likely hot paths:

#### 1. Service Discovery Loop
**Location**: `songbird-discovery/src/discovery/`
**Estimated Impact**: High
```rust
// BEFORE (potential hot path)
let service_name = service.name.clone(); // May be in hot path

// AFTER (zero-copy)
let service_name = &service.name; // Reference instead
```

#### 2. Capability Routing
**Location**: `songbird-universal/src/unified_adapter.rs`
**Estimated Impact**: High
```rust
// BEFORE
let capability = request.capability.clone();

// AFTER
let capability = &request.capability;
```

#### 3. Configuration Loading
**Location**: `songbird-config/src/config/`
**Estimated Impact**: Medium
```rust
// BEFORE
let config_value = config.get(&key).cloned();

// AFTER
let config_value = config.get(key); // Return reference
```

#### 4. Health Check Propagation
**Location**: `songbird-observability/src/health/`
**Estimated Impact**: Medium
```rust
// BEFORE
let health_status = node.health.clone();

// AFTER
let health_status = &node.health;
```

---

## 🔍 Clone Analysis

### Current Clone Statistics
```
Total clones:        855
Estimated hot path:  ~60
Arc usage:           960 (good for shared ownership)
Cow usage:           Present where appropriate
```

### Categories of Clones

#### Category 1: Hot Path Clones (~60)
- **Priority**: P1 (High)
- **Action**: Replace with references or Cow
- **Expected gain**: 10-20% performance improvement

#### Category 2: Warm Path Clones (~200)
- **Priority**: P2 (Medium)
- **Action**: Evaluate case-by-case
- **Expected gain**: 5-10% performance improvement

#### Category 3: Cold Path Clones (~595)
- **Priority**: P3 (Low)
- **Action**: Leave as-is (correctness over micro-optimization)
- **Expected gain**: Minimal

---

## 🎨 Optimization Patterns

### Pattern 1: Clone to Reference

```rust
// BEFORE
fn process_service(service: Service) {
    let name = service.name.clone(); // Unnecessary clone
    // ... use name ...
}

// AFTER
fn process_service(service: &Service) {
    let name = &service.name; // Zero-copy reference
    // ... use name ...
}
```

### Pattern 2: Clone to Cow

```rust
use std::borrow::Cow;

// BEFORE
fn format_message(prefix: String, msg: String) -> String {
    format!("{}: {}", prefix, msg) // Always allocates
}

// AFTER
fn format_message<'a>(prefix: &'a str, msg: Cow<'a, str>) -> Cow<'a, str> {
    if msg.is_empty() {
        Cow::Borrowed(prefix)
    } else {
        Cow::Owned(format!("{}: {}", prefix, msg))
    }
}
```

### Pattern 3: Arc for Shared Ownership

```rust
use std::sync::Arc;

// BEFORE (multiple clones)
let config_copy1 = config.clone();
let config_copy2 = config.clone();
let config_copy3 = config.clone();

// AFTER (Arc - single allocation)
let config = Arc::new(config);
let config_ref1 = Arc::clone(&config);
let config_ref2 = Arc::clone(&config);
let config_ref3 = Arc::clone(&config);
```

### Pattern 4: String to &str

```rust
// BEFORE
fn log_message(msg: String) {
    println!("{}", msg);
}

// AFTER
fn log_message(msg: &str) {
    println!("{}", msg);
}

// Usage
log_message("test"); // No allocation
log_message(&owned_string); // No clone needed
```

---

## 📊 Benchmark Suite

### Critical Path Benchmarks

Create/update: `benches/hot_path_benchmarks.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use songbird_discovery::ServiceDiscovery;
use songbird_universal::UniversalCapabilityAdapter;

fn bench_service_discovery(c: &mut Criterion) {
    let discovery = ServiceDiscovery::new_test();
    
    c.bench_function("discover_services", |b| {
        b.iter(|| {
            discovery.discover_services(black_box("compute"))
        })
    });
}

fn bench_capability_routing(c: &mut Criterion) {
    let adapter = UniversalCapabilityAdapter::new_test();
    
    c.bench_function("route_capability", |b| {
        b.iter(|| {
            adapter.route_capability(black_box("compute"))
        })
    });
}

criterion_group!(benches, bench_service_discovery, bench_capability_routing);
criterion_main!(benches);
```

### Memory Benchmarks

```rust
fn bench_clone_operations(c: &mut Criterion) {
    let service = create_test_service();
    
    c.bench_function("service_clone", |b| {
        b.iter(|| {
            let _cloned = black_box(&service).clone();
        })
    });
    
    c.bench_function("service_reference", |b| {
        b.iter(|| {
            let _referenced = black_box(&service);
        })
    });
}
```

---

## 🧪 Testing Strategy

### Performance Regression Tests

```rust
#[test]
fn test_no_performance_regression() {
    let start = std::time::Instant::now();
    
    // Run critical operation
    perform_service_discovery();
    
    let duration = start.elapsed();
    
    // Assert performance target
    assert!(
        duration < std::time::Duration::from_millis(10),
        "Service discovery took {}ms, expected <10ms",
        duration.as_millis()
    );
}
```

### Memory Usage Tests

```rust
#[test]
fn test_memory_efficiency() {
    let initial = get_heap_usage();
    
    // Perform operations
    for _ in 0..1000 {
        discover_and_route_service();
    }
    
    let final_usage = get_heap_usage();
    let increase = final_usage - initial;
    
    // Assert memory target
    assert!(
        increase < 10 * 1024 * 1024, // 10MB
        "Memory increase: {}MB",
        increase / (1024 * 1024)
    );
}
```

---

## 📝 Optimization Checklist

### Before Optimization
- [ ] Run baseline benchmarks
- [ ] Generate flamegraph
- [ ] Identify hot paths (>1% of time)
- [ ] Document current performance
- [ ] Create optimization branch

### During Optimization
- [ ] Change one hot path at a time
- [ ] Run benchmarks after each change
- [ ] Verify correctness with tests
- [ ] Document changes
- [ ] Commit incrementally

### After Optimization
- [ ] Run full benchmark suite
- [ ] Compare with baseline (`cargo benchcmp`)
- [ ] Verify all tests pass
- [ ] Check for performance regressions
- [ ] Update documentation
- [ ] Create PR with benchmarks

---

## 🎯 Target Metrics

### Performance Targets

| Operation | Current (est.) | Target | Improvement |
|-----------|---------------|--------|-------------|
| Service Discovery | 5ms | 3ms | 40% |
| Capability Routing | 2ms | 1ms | 50% |
| Health Check | 1ms | 0.5ms | 50% |
| Config Load | 10ms | 7ms | 30% |

### Memory Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Hot Path Clones | ~60 | 20-30 | 50-67% |
| Peak Memory | TBD | -10% | 10% reduction |
| Allocations/sec | TBD | -20% | 20% reduction |

---

## 📊 Reporting Template

### Performance Optimization Report

```markdown
## Optimization: [Description]

**Date**: YYYY-MM-DD
**Author**: [Name]
**PR**: #[number]

### Changes Made
- Replaced X clones with references in [module]
- Used Cow for [use case]
- Introduced Arc for [shared data]

### Benchmark Results
\```
Before:
  operation_name: 5.23ms ± 0.15ms

After:
  operation_name: 3.45ms ± 0.10ms
  
Improvement: 34% faster
\```

### Memory Impact
\```
Before: 1,250 allocations/sec
After:  875 allocations/sec

Reduction: 30% fewer allocations
\```

### Test Results
- All tests passing: ✅
- No regressions: ✅
- New perf tests added: ✅

### Flamegraph Comparison
[Attach before/after flamegraphs]
```

---

## 🚀 Quick Start

### 1. Profile Right Now

```bash
# Quick profiling session
sudo flamegraph -o quick_profile.svg -- \
    cargo bench --bench critical_path_benchmarks

# Open flamegraph
firefox quick_profile.svg
```

### 2. Identify Top 5 Hot Paths

Look for the widest bars in the flamegraph.

### 3. Create Optimization Tasks

For each hot path:
1. Create GitHub issue
2. Estimate impact (high/medium/low)
3. Assign priority
4. Document in enhancement roadmap

### 4. Optimize One at a Time

```bash
# Create branch
git checkout -b optimize/service-discovery-clones

# Make changes
# Run benchmarks
cargo bench --bench critical_path_benchmarks -- --save-baseline after

# Compare
cargo benchcmp before after

# Commit if improved
git commit -m "perf: reduce clones in service discovery (34% faster)"
```

---

## 🎓 Resources

### Documentation
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [flamegraph.rs](https://github.com/flamegraph-rs/flamegraph)
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)

### Tools
- **flamegraph**: Visual profiling
- **criterion**: Statistical benchmarking
- **cargo-benchcmp**: Compare benchmark results
- **valgrind/cachegrind**: Cache analysis
- **perf**: Linux performance monitoring

### Songbird-Specific
- `benches/critical_path_benchmarks.rs` - Core benchmarks
- `benches/performance_benchmarks.rs/` - Performance suite
- `docs/PERFORMANCE_GUIDE.md` - General performance guide

---

## 📋 Next Steps

1. **This Week**
   - [ ] Install profiling tools
   - [ ] Run baseline benchmarks
   - [ ] Generate first flamegraph
   - [ ] Identify top 10 hot paths

2. **Next Week**
   - [ ] Optimize top 3 hot paths
   - [ ] Create performance regression tests
   - [ ] Document improvements

3. **Month 1**
   - [ ] Optimize remaining hot paths
   - [ ] Achieve 50% reduction in hot path clones
   - [ ] Validate 10-20% performance improvement

---

## ✅ Success Criteria

- [ ] Flamegraph generated and analyzed
- [ ] Hot paths identified and documented
- [ ] Baseline benchmarks established
- [ ] Top 10 hot paths optimized
- [ ] 50% reduction in hot path clones (60 → 30)
- [ ] 10-20% overall performance improvement
- [ ] All tests passing
- [ ] No regressions introduced

---

**Document Owner**: Performance Team  
**Review Cycle**: Quarterly  
**Last Updated**: October 27, 2025  
**Next Review**: January 2026

