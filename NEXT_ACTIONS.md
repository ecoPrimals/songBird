# 🚀 Songbird - Next Actions Guide
**Date**: January 24, 2026  
**Status**: ✅ AUDIT COMPLETE - Ready for Development

---

## Current State ✅

```bash
Build:        ✅ CLEAN (cargo build --workspace)
Format:       ✅ PASSING (cargo fmt --all)
Tests:        ✅ 549/555 (98.9%)
Coverage:     ⏳ Pending (after test restoration)
Codebase:     1,306 files, 378,019 lines
```

---

## Immediate Actions (Pick One)

### Option 1: Restore Integration Tests (High Impact)
**Goal**: Get 42 archived tests working with current APIs

```bash
# Start here
cd archive/corrupted-tests-jan-2026/

# Pick a test category:
# - e2e_*.rs          - End-to-end workflows
# - integration_*.rs  - Component integration  
# - chaos_*.rs        - Fault injection

# Strategy:
1. Read archived test
2. Update imports for current API
3. Fix function signatures
4. Move back to tests/
5. Run: cargo test --test <name>
```

**Time**: 1-2 hours per test category  
**Value**: Enables full coverage reporting

---

### Option 2: Run Coverage Analysis (Quick Win)
**Goal**: Get baseline coverage metrics

```bash
# Install if needed
cargo install cargo-llvm-cov

# Run coverage on passing tests
cargo llvm-cov --workspace --html --output-dir target/coverage

# View report
firefox target/coverage/index.html  # or your browser

# Generate summary
cargo llvm-cov --workspace --summary-only
```

**Time**: 5-10 minutes  
**Value**: Know exactly what needs testing

---

### Option 3: Address Clippy Warnings (Code Quality)
**Goal**: Clean up ~100 pedantic warnings

```bash
# See all warnings
cargo clippy --all-targets --all-features

# Fix automatically where possible
cargo clippy --fix --all-targets --allow-dirty

# Manual review needed for:
- unused_variables (may be intentional)
- needless_borrow (performance tradeoff)
- module_name_repetitions (API design)
```

**Time**: 30-60 minutes  
**Value**: Cleaner codebase, catches potential bugs

---

### Option 4: Recreate Benchmarks (Performance)
**Goal**: Restore performance baselines

```bash
cd archive/corrupted-benches-jan-2026/

# Benchmarks to restore:
1. comprehensive_performance_benchmarks.rs
2. fractal_federation_performance.rs
3. ultra_pedantic_performance.rs
4. unified_types_benchmarks.rs

# Modern benchmark template:
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_example(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| {
            // Operation to benchmark
            black_box(function_call());
        });
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
```

**Time**: 1-2 hours  
**Value**: Performance regression detection

---

### Option 5: Update Examples (Documentation)
**Goal**: Working code examples for users

```bash
cd archive/corrupted-examples-jan-2026/

# Priority examples:
1. capability_discovery_example.rs - Show capability-based discovery
2. ipc_client_*.rs - IPC communication patterns
3. zero_cost_*.rs - Zero-copy optimization demos

# Test examples:
cargo run --example <name>
```

**Time**: 15-30 minutes per example  
**Value**: Better developer onboarding

---

## Quick Commands Reference

```bash
# Build everything
cargo build --workspace

# Run all tests (including failures)
cargo test --workspace --no-fail-fast

# Run specific test
cargo test --test <test-name>

# Format code
cargo fmt --all

# Check lints
cargo clippy --all-targets --all-features

# Generate docs
cargo doc --workspace --no-deps --open

# Run benchmarks (when restored)
cargo bench

# Coverage (when tests pass)
cargo llvm-cov --workspace --html --output-dir target/coverage
```

---

## Architecture Quick Reference

### UniBin Structure
```bash
songbird server     # Start orchestrator
songbird discover   # Find services
songbird doctor     # Health check
songbird status     # System info
songbird config     # Configuration
```

### Capability-Based Discovery
```rust
// ❌ OLD: Hardcoded
let client = SquirrelClient::connect("localhost:9200")?;

// ✅ NEW: Discovered
let resolver = CapabilityResolver::new();
let provider = resolver.discover_provider("ai").await?;
let client = connect_to_provider(&provider)?;
```

### IPC Pattern
```rust
// Register capability
let endpoint = ipc::register("my-primal", vec!["compute"]).await?;

// Listen for requests
let mut listener = ipc::listen(endpoint).await?;

// Accept connections
while let Ok(stream) = listener.accept().await {
    // Handle request
}
```

---

## File Size Guidelines

**Target**: 1,000 lines per file  
**Current exceptions** (smart refactoring in progress):
- `handshake_legacy.rs` - 2,770 lines (modular extraction underway)
- `beardog_client.rs` - 1,662 lines (well-organized, low priority)

**Strategy**: Extract by cohesion, not arbitrary line count

---

## Troubleshooting

### Build Issues
```bash
# Clean build
cargo clean
cargo build --workspace

# Check for orphaned files
fd '\.rs$' --exclude target --exclude archive
```

### Test Failures
```bash
# Run with output
cargo test --workspace -- --nocapture

# Run single test with debug
RUST_LOG=debug cargo test <test-name> -- --nocapture
```

### Coverage Not Running
```bash
# Ensure llvm-tools installed
rustup component add llvm-tools-preview

# Use tarpaulin as alternative
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

---

## Standards Compliance Checklist

- [x] UniBin: Single binary with subcommands
- [x] EcoBin: Proper primal structure
- [x] JSON-RPC/tarpc: Primary communication protocols
- [x] Zero hardcoding: Capability-based discovery
- [x] Safe Rust: Minimal unsafe (1 justified impl)
- [x] Self-knowledge: Primals discover others at runtime
- [x] Human dignity: Privacy-first, consent-based

---

## Resources

### Documentation
- `AUDIT_REPORT_JAN_2026.md` - Full audit results
- `STATUS.md` - Project status
- `EVOLUTION_HARDENING_PLAN.md` - Technical evolution plan
- `specs/` - All specifications (100 files)

### Ecosystem Standards
- `/ecoPrimals/wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- `/ecoPrimals/wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- `/ecoPrimals/wateringHole/PRIMAL_IPC_PROTOCOL.md`
- `/ecoPrimals/wateringHole/INTER_PRIMAL_INTERACTIONS.md`

### Testing
- `tarpaulin.toml` - Coverage configuration
- `scripts/coverage.sh` - Coverage script
- `crates/songbird-test-utils/` - Test utilities

---

## Decision Matrix

| Goal | Option | Time | Difficulty | Impact |
|------|--------|------|------------|--------|
| Ship faster | Option 2: Coverage | 10 min | Easy | Medium |
| Quality code | Option 3: Clippy | 1 hour | Easy | High |
| Better tests | Option 1: Integration | 2-4 hours | Medium | High |
| Performance | Option 4: Benchmarks | 2 hours | Medium | Medium |
| Docs | Option 5: Examples | 1 hour | Easy | Medium |

---

## Contact & Support

- **Audit Report**: `AUDIT_REPORT_JAN_2026.md`
- **Issue Tracking**: Create GitHub issues for bugs
- **Architecture Questions**: Reference specs/ directory
- **Ecosystem Integration**: Check wateringHole/ standards

---

**Remember**: This is evolutionary development. Prioritize working features over perfect code. Iterate and improve continuously.

**Status**: ✅ Codebase is healthy and ready for your next focus area.

