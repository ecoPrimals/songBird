# Songbird Test Suite

## Layout

- **Workspace crates** — `#[cfg(test)]` in `crates/*/src/` and integration tests in `crates/*/tests/` (most tests).
- **Root `songbird` package** — integration tests as one Cargo target per file directly under `tests/`:

| File | Role |
|------|------|
| `local_infrastructure_ci.rs` | Local CI: JSON-RPC-over-TCP against real IPC handler paths; in-file e2e-/chaos-/fault-style cases (see module docs). |
| `cli_parsing_tests.rs` | CLI parsing and validation only (no servers). |
| `integration_task_lifecycle.rs` | Orchestrator task lifecycle against isolated temp DBs. |

**Note**: Former `tests/e2e/`, `tests/chaos/`, `tests/fault/`, `tests/integration/`, `tests/common/`, `tests/helpers/` subtrees were removed in Wave 157 (Apr 22, 2026) — they were never compiled (no `tests/*.rs` root declared `mod e2e` etc.), referenced removed dependencies (`reqwest`), and contained ~12,400 lines of dead code.

## Counts (workspace lib tests)

| | |
|--|--|
| Passed | 7,683 |
| Failed | 0 |
| Ignored | 22 |

## Running

```bash
cargo test --workspace --lib             # library unit tests only (primary)
cargo test --workspace --all-features    # full workspace (feature-gated tests)
cargo test -p songbird                   # root crate + the three tests/*.rs targets above
cargo test -p <crate>                    # single workspace member
./scripts/test-with-security-provider.sh # optional: live security provider
./scripts/coverage.sh                    # llvm-cov HTML report
```
