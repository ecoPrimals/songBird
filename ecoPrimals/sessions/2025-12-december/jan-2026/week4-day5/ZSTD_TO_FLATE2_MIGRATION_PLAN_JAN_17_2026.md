# 🚀 zstd → flate2 Migration Execution Plan

**Date**: January 17, 2026  
**Goal**: Replace zstd C library with pure Rust flate2  
**Timeline**: 2 weeks (~14 hours)  
**Impact**: ecoBin compliance achieved (minus TLS)!

---

## 🎯 Executive Summary

**What**: Replace `zstd` (C library) with `flate2` (pure Rust)  
**Where**: `crates/songbird-orchestrator/src/task_lifecycle/checkpoint.rs`  
**Why**: Eliminate C dependency, achieve ecoBin  
**Risk**: Low (battle-tested alternative, comprehensive tests)

**Trade-offs**:
- ⬇️ Compression ratio: 3-5x (zstd) → 2-3x (flate2)
- ⬇️ Speed: ~80% of zstd (acceptable for checkpoints)
- ⬆️ 100% Pure Rust (no C dependencies!)
- ⬆️ Ecosystem standard (cargo/rustc use flate2)

---

## 📋 Migration Checklist

### Phase 1: Implementation (4 hours)

- [ ] **Update Cargo.toml**
  ```toml
  # Remove:
  zstd = "0.13"
  
  # Add:
  flate2 = { version = "1.0", default-features = false, features = ["rust_backend"] }
  ```

- [ ] **Update checkpoint.rs imports**
  ```rust
  // Remove:
  // (zstd is not imported directly, used inline)
  
  // Add:
  use flate2::write::GzEncoder;
  use flate2::read::GzDecoder;
  use flate2::Compression;
  use std::io::{Read, Write};
  ```

- [ ] **Update CompressionAlgorithm enum**
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum CompressionAlgorithm {
      None,
      Gzip,  // Changed from Zstd
  }
  ```

- [ ] **Implement compress_state()**
  ```rust
  fn compress_state(data: &[u8]) -> Result<Vec<u8>> {
      let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
      encoder.write_all(data)?;
      encoder.finish()
          .context("Failed to compress checkpoint state")
  }
  ```

- [ ] **Implement decompress_state()**
  ```rust
  fn decompress_state(data: &[u8]) -> Result<Vec<u8>> {
      let mut decoder = GzDecoder::new(data);
      let mut result = Vec::new();
      decoder.read_to_end(&mut result)
          .context("Failed to decompress checkpoint state")?;
      Ok(result)
  }
  ```

- [ ] **Update new_compressed() metadata**
  ```rust
  pub fn new_compressed(task_id: TaskId, progress: f32, state: Vec<u8>) -> Result<Self> {
      let compressed = Self::compress_state(&state)?;
      // ... (size_bytes, checksum)
      Ok(Self {
          // ...
          metadata: CheckpointMetadata {
              compression: Some(CompressionAlgorithm::Gzip),  // Changed
              // ...
          },
      })
  }
  ```

- [ ] **Update get_state() match**
  ```rust
  pub fn get_state(&self) -> Result<Vec<u8>> {
      match self.metadata.compression {
          Some(CompressionAlgorithm::Gzip) => Self::decompress_state(&self.state),  // Changed
          Some(CompressionAlgorithm::None) | None => Ok(self.state.clone()),
      }
  }
  ```

### Phase 2: Testing (3 hours)

- [ ] **Run existing unit tests**
  ```bash
  cd crates/songbird-orchestrator
  cargo test checkpoint
  ```

- [ ] **Verify compression test**
  - Test: `test_checkpoint_compression`
  - Expected: Passes (may need compression ratio adjustment)

- [ ] **Verify integrity test**
  - Test: `test_checkpoint_integrity`
  - Expected: Passes (checksum logic unchanged)

- [ ] **Run full test suite**
  ```bash
  cd /home/eastgate/Development/ecoPrimals/phase1/songbird
  cargo test
  ```

- [ ] **Integration tests**
  ```bash
  cargo test --test '*' -- checkpoint
  ```

### Phase 3: Performance Benchmarks (2 hours)

- [ ] **Create benchmark script**
  ```rust
  // benches/checkpoint_compression.rs
  use criterion::{black_box, criterion_group, criterion_main, Criterion};
  use songbird_orchestrator::task_lifecycle::Checkpoint;
  
  fn benchmark_compression(c: &mut Criterion) {
      let data = vec![1u8; 5_000_000]; // 5MB
      let task_id = TaskId::new();
      
      c.bench_function("compress_5mb", |b| {
          b.iter(|| {
              Checkpoint::new_compressed(task_id, 0.5, black_box(data.clone()))
          });
      });
  }
  
  criterion_group!(benches, benchmark_compression);
  criterion_main!(benches);
  ```

- [ ] **Run benchmarks**
  ```bash
  cargo bench --bench checkpoint_compression
  ```

- [ ] **Document results**
  - Compression time: Expected ~80% of zstd
  - Compression ratio: Expected ~60-70% of zstd
  - Decompression time: Expected ~90% of zstd

### Phase 4: Backward Compatibility (Optional, 3 hours)

**Decision**: Do we need to support old zstd checkpoints?

**Option A: Migration Support** (Recommended for production)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Zstd,  // Legacy (read-only)
    Gzip,  // New default
}

pub fn get_state(&self) -> Result<Vec<u8>> {
    match self.metadata.compression {
        Some(CompressionAlgorithm::Gzip) => Self::decompress_state_gzip(&self.state),
        Some(CompressionAlgorithm::Zstd) => Self::decompress_state_zstd(&self.state),
        Some(CompressionAlgorithm::None) | None => Ok(self.state.clone()),
    }
}

#[cfg(feature = "legacy-zstd")]
fn decompress_state_zstd(data: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::decode_all(data).context("Failed to decompress zstd checkpoint")
}

#[cfg(not(feature = "legacy-zstd"))]
fn decompress_state_zstd(_data: &[u8]) -> Result<Vec<u8>> {
    anyhow::bail!("zstd decompression not available (build with --features legacy-zstd)")
}
```

**Cargo.toml**:
```toml
[features]
legacy-zstd = ["zstd"]  # Optional for migration period
```

**Migration Timeline**:
- Release N: Support both (feature = "legacy-zstd")
- Release N+1: Deprecation warning
- Release N+2: Remove zstd entirely

**Option B: Clean Break** (Simpler)

- Remove zstd entirely
- Old checkpoints become unreadable
- Acceptable if: Checkpoints are ephemeral (7-day max)

**Recommendation**: **Option B (Clean Break)**
- Checkpoints have 7-day TTL (default)
- No critical production data in checkpoints
- Simpler codebase

### Phase 5: Documentation (2 hours)

- [ ] **Update CHANGELOG.md**
  ```markdown
  ## [Unreleased]
  
  ### Changed
  - Replaced zstd compression with pure Rust flate2/gzip
  - Compression ratio: ~2-3x (was 3-5x with zstd)
  - Performance: ~80% of zstd (acceptable for checkpoints)
  
  ### Removed
  - zstd C library dependency
  
  ### Migration
  - Existing checkpoints will be recompressed on next save
  - No action required (checkpoints are ephemeral)
  ```

- [ ] **Update checkpoint.rs documentation**
  ```rust
  //! Task checkpointing with pure Rust compression
  //!
  //! Uses flate2/gzip for 100% pure Rust, zero C dependencies.
  //! Compression ratio: ~2-3x for structured data.
  //! Performance: ~80% of zstd, acceptable for checkpoint use case.
  ```

- [ ] **Update README.md**
  ```markdown
  ## Dependencies
  
  - ✅ 100% Pure Rust (excluding TLS)
  - ✅ Zero C dependencies for compression
  - ✅ musl-static ready
  ```

- [ ] **Update PURE_RUST_EVOLUTION_PLAN**
  - Mark zstd migration as complete
  - Update ecoBin status: 95% (A grade)

---

## 🎯 Expected Results

### Before Migration
```toml
[dependencies]
zstd = "0.13"  # C library dependency
```

**Binary**:
- Size: +1.5MB (libzstd)
- C deps: zstd, zstd-safe, zstd-sys
- ecoBin: ❌ No (has C dependencies)

**Performance**:
- Compress 5MB: ~50ms
- Ratio: 3-5x
- Decompress: ~30ms

### After Migration
```toml
[dependencies]
flate2 = { version = "1.0", default-features = false, features = ["rust_backend"] }
```

**Binary**:
- Size: -1.4MB net (remove zstd, add flate2)
- C deps: **ZERO!** ✅
- ecoBin: ✅ **YES** (minus TLS)

**Performance** (Expected):
- Compress 5MB: ~60-65ms (~80% of zstd)
- Ratio: 2-3x (~70% of zstd)
- Decompress: ~35ms (~90% of zstd)

**Acceptable?**: ✅ **YES**
- Checkpoints are infrequent (not hot path)
- Compression happens async (non-blocking)
- Trade-off worth it for Pure Rust

---

## ⚠️ Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lower compression ratio | High | Low | Acceptable (2-3x still good) |
| Slower compression | High | Low | Not hot path, async |
| Test failures | Low | Medium | Existing tests comprehensive |
| Performance regression | Medium | Low | Benchmark and document |
| Backward compatibility | Low | Low | Checkpoints ephemeral (7-day TTL) |

**Overall Risk**: ✅ **LOW**

---

## 🚀 Execution Timeline

### Day 1-2: Implementation
- Hours 1-2: Update Cargo.toml + imports
- Hours 3-4: Implement compress/decompress
- Hours 5-6: Update enum + metadata
- Hours 7-8: Run initial tests

### Day 3-4: Testing
- Hours 9-10: Unit tests + integration
- Hours 11-12: Create + run benchmarks
- Hours 13-14: Performance analysis

### Day 5: Documentation
- Hours 15-16: Update docs + CHANGELOG
- Hours 17-18: Review + polish

**Total**: 2-3 days (14-18 hours)

---

## ✅ Success Criteria

- [ ] All tests passing
- [ ] Compression works (2-3x ratio)
- [ ] Decompression works
- [ ] Performance acceptable (< 20% slower)
- [ ] Zero C dependencies
- [ ] Documentation updated
- [ ] ecoBin status: 95% (A grade)

---

## 🎊 Completion Checklist

- [ ] Code changes committed
- [ ] Tests passing (161/161)
- [ ] Benchmarks documented
- [ ] CHANGELOG updated
- [ ] Documentation updated
- [ ] Git commit + push
- [ ] WateringHole status updated
- [ ] Celebrate ecoBin achievement! 🎉

---

**Status**: ✅ **READY TO EXECUTE**  
**Risk**: Low  
**Impact**: High (ecoBin achievement!)  
**Timeline**: 2-3 days

🦀 **Let's evolve to Pure Rust!** 🦀

---

**Author**: Songbird Team  
**Date**: January 17, 2026  
**Next**: Execute Phase 1 (Implementation)

