# Contributing to Songbird

**License**: AGPL-3.0-only (scyBorg provenance trio)  
**Edition**: Rust 2024

---

## Code Quality Standards

### Error Handling

**Production code** uses `Result<T, E>` with `?`. No `unwrap()`, `expect()`, or
`panic!()` outside `#[cfg(test)]`.

```rust
pub fn load_config() -> SongbirdResult<Config> {
    let file = std::fs::read_to_string("config.toml")?;
    Ok(toml::from_str(&file)?)
}
```

**Test code** may use `.unwrap()` and `.expect()` — test panics are clear failures.

### Lint Suppression: `#[expect]` and `#[allow]` with reasons

Per wateringHole standards, all lint suppressions require a `reason` string:

- `#[expect(lint, reason = "...")]` — when the lint **fires** and you're suppressing it
- `#[allow(lint, reason = "...")]` — when the lint **does not fire** but may in the future

```rust
#[expect(clippy::too_many_lines, reason = "protocol state machine is inherently sequential")]
fn build_circuit(&self) -> Result<Circuit> { /* ... */ }

#[allow(clippy::cast_sign_loss, reason = "value guaranteed non-negative by prior check")]
fn compute_offset(&self) -> usize { /* ... */ }
```

**Never** use bare `#[allow(lint)]` or `#[expect(lint)]` without a reason string.

### Unsafe Code

`#![forbid(unsafe_code)]` across all 30 crates with zero exceptions.
`songbird-process-env` uses an in-memory overlay (`std::sync::Mutex<HashMap>`)
instead of calling `std::env::set_var`/`remove_var`, eliminating the Rust 2024
`unsafe` requirement entirely.

### SPDX Headers

Every `.rs` file must have:

```rust
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
```

---

## Idiomatic Rust

### Error Types

Use `SongbirdResult<T>` from `songbird-types`:

```rust
use songbird_types::{SongbirdResult, SongbirdError};

pub async fn my_function() -> SongbirdResult<Value> {
    Ok(fallible_operation()?)
}
```

### Zero-Copy

Prefer borrowing over cloning. Use `Arc<str>` for shared strings in hot paths:

```rust
fn process_name(name: &str) { }
process_name(&service.name);
```

### Module Organization

- Files under 1000 lines
- Tests in `tests.rs` submodule or `tests/` directory
- Split by domain concern, not arbitrary line count

---

## Testing

### Coverage Target

**Goal**: 90% line coverage. Current: ~68.48% (llvm-cov measured, Mar 30 2026). Priority: pure-logic modules first.

```bash
cargo llvm-cov --workspace --lib --html
```

### Concurrent-Safe Tests (CRITICAL)

Tests **must not** use `std::env::set_var` (unsafe in Rust 2024). Use
`songbird_process_env::set_var` for overlay mutation, and injectable `_with`
variants where available. Mark tests that mutate the shared overlay with
`#[serial_test::serial]` to prevent interference:

```rust
#[tokio::test]
async fn test_discovery() {
    let env = HashMap::from([
        ("BEARDOG_SOCKET".to_string(), "/tmp/test.sock".to_string()),
    ]);
    let result = discover_with(|k| env.get(k).cloned()).await;
    assert!(result.is_ok());
}
```

### No Polling in Production

Use `tokio::sync::Notify` or channels, not `sleep` loops. Acceptable `sleep` uses:
retry backoff, rate limiting, periodic renewal, chaos tests.

### Test Categories

1. **Unit**: `#[cfg(test)]` modules
2. **Integration**: `tests/` directory
3. **E2E**: `tests/e2e/`
4. **Chaos/Fault**: `tests/chaos/`

---

## Pull Request Checklist

```bash
cargo fmt --all
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

- [ ] Zero errors, zero warnings
- [ ] No `unwrap()` in production paths
- [ ] `#[expect(reason)]` for any lint suppressions
- [ ] Doc comments on public APIs with `# Errors` sections
- [ ] Files under 1000 lines
- [ ] Coverage maintained or improved

---

## Human Dignity & Sovereignty

- No surveillance code
- No tracking without explicit consent
- Privacy-first architecture
- User control over data
- Transparent operations

See `specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md`.

---

## License

By contributing, you agree that your contributions will be licensed under
AGPL-3.0-only (scyBorg provenance trio: AGPL-3.0-only + ORC + CC-BY-SA 4.0).
