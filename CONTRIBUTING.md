# Contributing to Songbird

**License**: AGPL-3.0-or-later (scyBorg provenance trio)  
**Edition**: Rust 2024

---

## Code Quality Standards

### Error Handling

**Production code** uses `Result<T, E>` with `?`. No `unwrap()` or `panic!()` outside
`#[cfg(test)]`. `expect()` only on provably infallible parses, documented with `#[expect(clippy::expect_used, reason = "...")]`.

```rust
pub fn load_config() -> SongbirdResult<Config> {
    let file = std::fs::read_to_string("config.toml")?;
    Ok(toml::from_str(&file)?)
}
```

**Test code** may use `.unwrap()` and `.expect()` — test panics are clear failures.

### Lint Suppression: `#[expect]` and `#[allow]` with reasons

Per wateringHole standards, all lint suppressions require a `reason` string:

- `#[expect(lint, reason = "...")]` — when the lint **fires** in all build configurations and you're suppressing it
- `#[allow(lint, reason = "...")]` — when the lint may not fire in all configurations (e.g., `dead_code` on items used only from `#[cfg(test)]`), or in `#[cfg(test)]` modules

```rust
#[expect(clippy::too_many_lines, reason = "protocol state machine is inherently sequential")]
fn build_circuit(&self) -> Result<Circuit> { /* ... */ }

#[allow(clippy::cast_sign_loss, reason = "value guaranteed non-negative by prior check")]
fn compute_offset(&self) -> usize { /* ... */ }
```

**Never** use bare `#[allow(lint)]` or `#[expect(lint)]` without a reason string.

**Note**: Wave 134 completed the full `#[expect(dead_code)]` → `#[allow(dead_code)]` migration across all 30 crates. `dead_code` lints are inherently cfg-dependent (items used from `#[cfg(test)]` appear dead in non-test builds), so `#[allow]` is the correct suppression. Do not convert these to `#[expect]` — CI will break with `unfulfilled-lint-expectations`.

### Unsafe Code

`#![forbid(unsafe_code)]` across all 30 crates with zero exceptions.
`songbird-process-env` uses an in-memory overlay (`std::sync::Mutex<HashMap>`)
instead of calling `std::env::set_var`/`remove_var`, eliminating the Rust 2024
`unsafe` requirement entirely.

### SPDX Headers

Every `.rs` file must have:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
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

- Files under 800 lines
- Tests in `tests.rs` submodule or `tests/` directory
- Split by domain concern, not arbitrary line count

---

## Testing

### Coverage Target

**Goal**: 90% line coverage. Current: **72.29%** (llvm-cov measured, Apr 8 2026; 7,380 lib tests / 13,170+ workspace `--all-features`). Priority: pure-logic modules first.

```bash
cargo llvm-cov --workspace --lib --html
```

### Concurrent-Safe Tests (CRITICAL)

Tests **must not** use `std::env::set_var` (unsafe in Rust 2024). Use injectable
`_with` closures for environment isolation — tests inject mock env readers and
run fully concurrently with zero global state mutation:

```rust
#[tokio::test]
async fn test_discovery() {
    let env = HashMap::from([
        ("SECURITY_PROVIDER_SOCKET".into(), "/run/user/1000/biomeos/security.sock".into()),
    ]);
    let result = discover_with(|k| env.get(k).cloned()).await;
    assert!(result.is_ok());
}
```

Zero `#[serial_test]` in the workspace. All tests run concurrently.

### Deterministic Time in Tests

Use `#[tokio::test(start_paused = true)]` for tests involving `tokio::time::sleep` —
virtual time advances instantly. Never use `tokio::task::yield_now()` in poll loops
(causes infinite loops under paused time); use `tokio::time::sleep(Duration::from_millis(1))`
instead.

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
- [ ] Files under 800 lines
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
AGPL-3.0-or-later (scyBorg provenance trio: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0).
