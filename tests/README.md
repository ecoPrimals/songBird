# Songbird Test Suite

## Test Architecture

Tests live in two locations:

1. **Crate-level tests** — `crates/*/src/` (inline `#[cfg(test)]` modules) and `crates/*/tests/` (integration tests)
2. **Root integration tests** — `tests/` (workspace-level integration tests for the `songbird` binary)

The vast majority of tests are in the crate-level locations. Run them with:

```bash
cargo test --workspace
```

## Root-Level Tests

| File | Purpose |
|------|---------|
| `tests/integration_task_lifecycle.rs` | Task lifecycle integration |
| `tests/cli_parsing_tests.rs` | CLI argument parsing |

## Test Categories (by crate)

- **Unit**: `#[cfg(test)]` modules inside `src/` files
- **Integration**: `crates/*/tests/*.rs` files (e2e, chaos, fault, upstream)
- **Fuzz-style**: Malformed input parsing (TLS records, JSON-RPC, relay protocol, STUN messages)

## Key Crate Test Suites

| Crate | Test count | Notable coverage |
|-------|-----------|------------------|
| `songbird-orchestrator` | ~3,500 | JSON-RPC handlers, startup, health, federation, consent |
| `songbird-universal-ipc` | ~1,200 | Service handler, introspection, tower atomic, mesh |
| `songbird-tor-protocol` | ~800 | Directory, consensus, relay selection, circuit |
| `songbird-http-client` | ~700 | TLS 1.3, redirect, connection pool, BearDog RPC |
| `songbird-config` | ~600 | Discovery, endpoints, constants, environment |
| `songbird-discovery` | ~500 | Federation-aware, mDNS, SSDP, dark forest |

## Running Tests

```bash
cargo test --workspace --all-features            # full suite (11,831 tests)
cargo test --workspace --lib                     # unit tests only
cargo test -p songbird-orchestrator              # single crate
./scripts/test-with-beardog.sh                   # with live BearDog from plasmidBin
./scripts/coverage.sh                            # llvm-cov HTML report
```

## Test Principles

- **Zero serial tests** — all tests run fully concurrent (`--test-threads=16`)
- **Injectable env readers** — `_with` variants replace `std::env::set_var` for isolation
- **No production mocks** — all mocks behind `#[cfg(test)]` or `feature = "test-mocks"`
- **No sleep-based synchronization** — `tokio::sync::Notify` and `oneshot` channels
- **`--all-features`** — many tests are feature-gated; always use `--all-features` for full coverage

## Metrics

| Metric | Value |
|--------|-------|
| Total tests | 11,831 |
| Failed | 0 |
| Coverage | ~68.48% (llvm-cov, target 90%) |
| `#[ignore]` | ~269 (100% with reason strings) |
