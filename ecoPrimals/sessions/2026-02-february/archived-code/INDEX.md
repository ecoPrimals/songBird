# Archived Code — February 9, 2026

Code removed during deep debt cleanup. Preserved as fossil record.

## tor-debug-examples/
Debug/development examples from `songbird-tor-protocol/examples/`.
Only the declared `fetch_consensus` example was retained.
- `debug_create2.rs` — CREATE2 cell debugging
- `raw_test.rs` — Raw byte-level Tor testing
- `deliberate_test.rs` — Deliberate connection testing
- `multi_relay_test.rs` — Multi-relay circuit testing
- `padding_test.rs` — Cell padding verification
- `test_connection.rs` — Connection lifecycle testing
- `test_create_fast.rs` — CREATE_FAST cell testing
- `connection_health.rs` — Connection health monitoring

## tor-debug-docs/
- `CREATE2_DEBUG_STATUS.md` — Debug session notes from Feb 7, 2026

## gaming-dead-code/
Entire `src/network/gaming/` directory. Never compiled — `src/main.rs`
does not import `mod network`. References missing modules. Dead code.

## rendezvous-standalone/
Standalone `rendezvous/` crate with TODO stubs. Not a workspace member.
Superseded by `songbird-universal-ipc/src/handlers/rendezvous_handler.rs`.

## stale-configs/
- `config.env.example` — Old HTTP-era config (superseded by `.env.example`)
- `.clippy-test.toml` / `clippy-test-config.toml` — Duplicate clippy configs
- `.gitignore.docs` — Unused docs gitignore
- `.github-issue-template.md` — Stale issue template (references `v5.24.0`)
- `.codeauditignore` — Unused audit ignore file
- `start-tower.sh` / `stop-tower.sh` — Old HTTP-era tower scripts (superseded by `songbird server`)
