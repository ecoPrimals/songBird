# Songbird Test Suite

## Layout

- **Workspace crates** — `#[cfg(test)]` in `crates/*/src/` and integration tests in `crates/*/tests/` (most tests).
- **Root `songbird` package** — integration tests as one Cargo target per file directly under `tests/`:

| File | Role |
|------|------|
| `local_infrastructure_ci.rs` | Local CI: JSON-RPC-over-TCP against real IPC handler paths; in-file e2e-/chaos-/fault-style cases (see module docs; scenario notes under `tests/e2e/SCENARIO_TEMPLATES.md`). |
| `cli_parsing_tests.rs` | CLI parsing and validation only (no servers). |
| `integration_task_lifecycle.rs` | Orchestrator task lifecycle against isolated temp DBs. |

**Module subtrees** (not separate Cargo targets): `tests/e2e/`, `tests/chaos/`, `tests/fault/`, `tests/integration/`. They are organized as `mod` trees for a would-be harness crate, but **no** current `tests/*.rs` root declares `mod e2e`, `mod chaos`, etc., so they are **not** compiled or run by `cargo test -p songbird` until such a root exists.

**Shared helpers** — `tests/common/`, `tests/helpers/` — pulled in with `#[path = ...]` from sources under those subtrees (e.g. `tests/e2e/`).

## Counts (workspace, `--all-features`)

| | |
|--|--|
| Passed | 13,030 |
| Failed | 0 |
| Ignored | 252 |

## Running

```bash
cargo test --workspace --all-features    # full workspace (feature-gated tests need this)
cargo test --workspace --lib             # library unit tests only
cargo test -p songbird                   # root crate + the three `tests/*.rs` targets above
cargo test -p <crate>                  # single workspace member
./scripts/test-with-security-provider.sh # optional: live security provider
./scripts/coverage.sh                   # llvm-cov HTML report
```
