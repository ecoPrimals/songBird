# Songbird Remaining Work

**Date**: March 24, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: March 24, 2026 (Session 18 — JSON-RPC Enum Dispatch, Coverage Expansion, Stub Evolution)

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 10,687 passed, 0 failed, 271 ignored |
| **Line Coverage** | ~67% (llvm-cov; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 30 crates compile clean (~45s dev) |
| **Clippy Pedantic** | 30/30 crates clean — zero warnings (`clippy::pedantic + nursery`, `--all-targets --all-features`) |
| **Format** | Clean (`cargo fmt --check` passes) |
| **Docs** | Clean (`cargo doc --workspace --all-features --no-deps` passes) |
| **Files >1000 lines** | 0 (max 948 test file; production max 915 `core.rs`) |
| **Unsafe blocks** | 2 (in `songbird-process-env` with `parking_lot::Mutex` guard + `#![deny(unsafe_code)]` + per-fn `#[expect]`) |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 (verified: all remaining are in `#[cfg(test)]` modules, integration tests, or doc examples) |
| **Production `panic!()`** | 0 |
| **Production `unreachable!()`** | 0 (evolved to `Err()` returns in http_server.rs) |
| **Production `eprintln!`** | 0 in library crates (all evolved to `tracing`; CLI binary output remains intentional) |
| **TODO/FIXME/HACK comments** | 0 in Rust source (wateringHole compliant) |
| **`#[allow()]` vs `#[expect()]`** | Fully correct: `#[expect(reason)]` only where lint fires, `#[allow(reason)]` everywhere else |
| **Capability discovery** | `find_primals_with_capability` — real capability filter (env-driven, identity-agnostic) |
| **Hardcoded elimination** | All ports env-driven; `primal_names` constants module; DNS-SD/mDNS/broadcast discovery; capability-first |
| **JSON-RPC handlers** | 14 semantic methods: 10 wrapping REST + `health.liveness` + `health.readiness` + `health.check` + `capabilities.list` (ecosystem standard) |
| **JSON-RPC dispatch** | Enum-based `JsonRpcMethod` routing in `songbird-types`; `FromStr`/`Display` for wire compatibility; sub-enums per domain (Discovery, Network, Stun, Relay, etc.) |
| **Method normalization** | `normalize_json_rpc_method_name()` in `songbird-types`; handles ecosystem naming drift (`capability.list` → `capabilities.list`, `ping` → `health.liveness`, `status`/`check`/`health` → `health.check`) |
| **Lint inheritance** | 30/30 crates inherit workspace lints; 2 crates have justified custom `[lints]` tables |
| **CONTEXT.md** | Present at repo root (wateringHole `PUBLIC_SURFACE_STANDARD` compliant) |
| **BearDog crypto** | All placeholders evolved to explicit `CryptoUnavailable` errors; rendezvous fingerprints use HMAC-SHA256 fallback; XOR mock isolated to `#[cfg(test)]` |
| **C dependencies** | `ring` opt-in only (`ring-crypto` feature); `sysinfo` fully eliminated — replaced by `sys_metrics` pure Rust `/proc` + `/sys` readers |
| **License** | `AGPL-3.0-only` via workspace inheritance (all 30 crates use `license.workspace = true`) + ORC + CC-BY-SA 4.0 |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok) |
| **SPDX headers** | 100% coverage across all `.rs` files |
| **CI quality gate** | Coverage threshold ratcheted to 66%; `Swatinem/rust-cache`; `cargo-deny` job; `rustsec/audit-check` |
| **UniBin** | `songbird server`, `songbird cli` (interactive REPL), `songbird compute-bridge`, `songbird deploy`, `songbird rendezvous` |
| **Nest Atomic** | `health.liveness` + `health.readiness` + `health.check` + `capabilities.list` JSON-RPC methods (14 capability tokens) |
| **Mock isolation** | `MockBearDogProvider` behind `#[cfg(any(test, feature = "test-mocks"))]`; XOR broadcast encryption isolated to test/mock builds; beacon ID uses SHA-256(node_id) fallback |
| **Zero-copy** | `Arc<str>` endpoints, `Arc<[u8]>` TLS keys, move semantics, borrow-through redirects, HKDF buffer reuse, static path labels, `serde_json::to_vec` (no intermediate String) |
| **Concurrent tests** | All tests fully concurrent; injectable `_with` env readers; `tokio::time::pause()` for deterministic timing |
| **Event-driven** | Zero `sleep`-based polling in production |
| **Module docs** | 77 `pub mod` declarations documented across 5 crates |
| **Binary size** | 20MB release |
| **`#[warn(missing_docs)]`** | 30/30 crates (all library crates have the lint enabled) |
| **Dependencies** | ~412 unique (`sysinfo`/`rayon`/`crossbeam` eliminated); duplicates aligned (base32→0.5, base64→0.22, hostname→0.4, thiserror→2.0) |
| **Build time** | ~45s clean dev build, ~68s test suite |
| **Total Rust lines** | ~390,564 (crates + src + tests + examples) |
| **Crates** | 30 workspace members |

---

## Completed (Mar 24, 2026 — JSON-RPC Enum Dispatch, Coverage Expansion, Stub Evolution — Session 18)

### Wave 71: JSON-RPC Enum-Based Dispatch
- [x] Created `songbird_types::json_rpc_method` module with `JsonRpcMethod` enum and domain sub-enums (`PrimalMethod`, `StunMethod`, `DiscoveryMethod`, `NetworkMethod`, `HealthMethod`, `RelayMethod`, `FederationMethod`, `TorMethod`, etc.)
- [x] Implemented `FromStr`, `Display`, `Serialize`/`Deserialize` for wire compatibility (JSON string on wire, not nested objects)
- [x] `from_wire_str()` parses all 50+ semantic method names including aliases (`find_primals` → `DiscoveryMethod::Peers`, etc.)
- [x] `parse_ipc()` normalizes then parses (IPC broker and HTTP gateway path)
- [x] Migrated `IpcServiceHandler::handle` to match on `JsonRpcMethod` enum instead of string patterns
- [x] Migrated HTTP JSON-RPC gateway to use `JsonRpcMethod::from_wire_str` for route dispatch
- [x] Migrated Unix IPC server to use `JsonRpcMethod::from_wire_str` (preserving biomeOS `"health"` routing)
- [x] Moved `normalize_json_rpc_method_name` from `songbird-universal-ipc` to `songbird-types` for reuse
- [x] Updated 3 test files for new error message format (`"unknown JSON-RPC method"`)

### Wave 71: Coverage Expansion — CLI Commands
- [x] `status.rs`: Extracted `overall_status_label` helper, added ~9 tests (health labeling, JSON formatting, memory reporting)
- [x] `tower.rs`: Added 11 tests (CPU/memory/storage boundary conditions for `determine_role`)
- [x] `quick.rs`: Extracted `capabilities_for_contribute_type`, added 12 tests (all variants, config defaults, next-steps generation)
- [x] `network.rs`: Added 10 tests (optimize/test/diagnose with minimal and maximal parameters)
- [x] `federation.rs`: Added 11 tests (init, join, lobby, matchmaking, metrics)

### Wave 71: Coverage Expansion — Config, Bluetooth, Types
- [x] `discoverable_endpoint.rs`: Added HTTPS default port, dev fallback, IPv6 socket addr tests
- [x] `runtime_engine.rs`: Added discover-by-capability sort order, env-prefix stripping, invalid-endpoint fallback tests
- [x] `hosts_evolved.rs`: Added AdvertiseConfig, detect_with, BindConfig, Environment serde round-trip tests
- [x] `config/paths.rs`: Added service path, development config nesting tests
- [x] `zero_touch/infant_config.rs`: Added security level aliases, fallback modes, bind/discovery method tests
- [x] `gatt/services.rs`: Added GATT service, UUID, read-by-group-type parsing, find-service tests
- [x] `gatt/descriptors.rs`: Added notification subscription tests
- [x] `transport/mod.rs`: Added TransportType Display/Eq, TransportConfig builder tests
- [x] `songbird-types/errors.rs`: Added not_implemented, rpc, event, discovery, validation, AutomationHint, Urgency ordering tests
- [x] `traits/canonical_types.rs`: Added DeploymentStatus, PrimalResponse, SystemHealth, SpanContext, ServiceType JSON round-trip tests

### Wave 71: Coverage Expansion — Orchestrator
- [x] `limited_btsp.rs`: Added 10 tests for check_operation_allowed, TrustLevel::Limited defaults, allow/deny list logic
- [x] `full_trust_btsp.rs`: Added 9 tests for TrustLevel::Highest, wildcard allow, deny override
- [x] `federated_btsp.rs`: Added 9 tests for Elevated defaults, custom allow/deny
- [x] `network/mod.rs`: Added 6 tests for binding strategy, connectivity result, sovereign binder
- [x] `core/api.rs`: Added 7 tests for ApiConfig defaults/serde, CoreApi/ApiHandler debug

### Wave 71: Not-Yet-Implemented Stub Evolution
- [x] `runtime_engine.rs`: DNS-SD, Consul, etcd, Kubernetes discovery stubs now return `SongbirdError::not_implemented_with_detail` instead of empty-list no-ops
- [x] `runtime_engine.rs`: mDNS, DNS-SD, Consul, etcd registration stubs now return proper typed errors instead of silent `Ok(())`
- [x] `delegation.rs`: Provider delegation helpers now use `not_implemented_with_detail` with descriptive messages
- [x] BTSP connection files: Fixed clippy `needless_return` in 3 files (`federated_btsp.rs`, `full_trust_btsp.rs`, `limited_btsp.rs`)

### Wave 71: Metrics
- Total: +170 new tests across 20+ modules
- 10,687 tests passing (up from 10,517), 0 failed, 271 ignored
- All 30 crates: `cargo fmt` clean, `cargo clippy -D warnings` clean, `cargo doc` clean

---

## Completed (Mar 24, 2026 — Deep Debt Evolution, Mock Isolation, Smart Refactoring, Coverage Expansion — Session 17)

### Wave 70: Deep Debt Evolution — Test File Evolution
- [x] Deleted 2 garbage scaffolding test files (`discovery_protocol_tests.rs`, `capability_discovery_comprehensive_tests.rs`)
- [x] Enabled 8 formerly-disabled test files (removed `#![cfg(feature = "tests-incomplete")]` gates) — +179 tests
- [x] Removed stale `tests-incomplete` feature from `songbird-universal` and `songbird-discovery` Cargo.toml
- [x] All 8 test files compile and pass without modification (APIs already existed)

### Wave 70: Smart File Refactoring (domain-aligned extraction)
- [x] `tests_discovery_bridge.rs` 959→400 lines: extracted E2E tests to `tests_discovery_bridge_e2e.rs`
- [x] `security.rs` 868→699 lines: extracted `SecurityMetrics`/`SecurityHealth`/`AuthResult` to `security_types.rs` + tests to `security_types_tests.rs`
- [x] `host.rs` 833→560 lines: split into `host/mod.rs` + `host/scan.rs` (BLE scanning domain)
- [x] `config/mod.rs` 824→647 lines: extracted `SecurityConfig` cluster to `config/security.rs`
- [x] `canonical.rs` 888→342 lines: extracted canonical data types to `canonical_types.rs`
- [x] `capability_based_runtime_discovery.rs` 822→478 lines: extracted tests to separate file
- [x] `sovereignty/adapter.rs` 816→355 lines: extracted tests to `adapter_tests.rs`
- [x] `tower_atomic.rs` 810→501 lines: split tests into `tower_atomic_tests.rs` + fuzz-style tests
- [x] Zero API changes — all public types re-exported

### Wave 70: Coverage Expansion
- [x] Added 9 `discover_broadcast_addresses` edge-case tests (env parsing, config fallbacks, dedup)
- [x] Added 14 `AIFirstResponse` / `AIFirstError` unit tests (constructors, serde roundtrips, error helpers)
- [x] Added 6 async `ConnectivityTester` tests (defaults, refused connections, comprehensive checks)
- [x] Added 5 `Interface` / network binding tests (IPv4/IPv6, dual-stack, fallbacks)
- [x] Total: +34 new tests in songbird-orchestrator

### Wave 70: Hardcoding Evolution
- [x] `songbird-universal-ipc` now depends on `songbird-types` (added dep)
- [x] `introspection.rs`: primal identifiers use `primal_names::SELF_NAME` and `primal_names::BEARDOG` constants
- [x] `birdsong_handler.rs`: BearDog socket paths use `primal_names::BEARDOG` in format strings
- [x] `onion_handler.rs` / `birdsong_handler.rs`: error heuristics extracted to `is_expected_crypto_delegate_connectivity_error` helpers using `primal_names` constants

### Wave 70: Production Mock/Placeholder Evolution
- [x] `rendezvous/client.rs`: `"sha256:placeholder"` → HMAC-SHA256 deterministic fingerprint from node_id (domain key `songbird.rendezvous.pkfp.v1`), or `CryptoUnavailable` error if no node_info
- [x] `beardog/birdsong.rs`: XOR "mock encryption" isolated to `#[cfg(any(test, feature = "test-mocks"))]`; production path returns `CryptoUnavailable` error
- [x] `anonymous/broadcaster.rs`: `vec![0u8; 16]` placeholder → SHA-256(node_id) first 16 bytes, or random 16 bytes if no node_id
- [x] `sovereign-onion/service.rs`: misleading "dummy BearDog" comment → accurate documentation of session crypto usage
- [x] Added `hmac = "0.12"` dependency to `songbird-network-federation`

### Wave 70: Dependency Analysis
- [x] Confirmed: zero `openssl`, `ring`, `native-tls`, `sqlite`, `libgit2`, `curl` in production tree
- [x] Confirmed: `blake3` already using `pure` feature (no C compilation at runtime)
- [x] Confirmed: `cc` build-dep artifact from blake3 but not invoked with pure feature
- [x] Only `-sys` crates: `dirs-sys` (libc for XDG paths), `netlink-sys` (interface discovery) — both essential for OS interaction
- [x] `deny.toml` already blocks `openssl` and `native-tls`

### Wave 70: Unsafe Code Analysis
- [x] Confirmed: 2 unsafe blocks in `songbird-process-env` are optimal for Rust 2024 edition on rustc 1.94.0
- [x] `std::env::set_var`/`remove_var` have no safe alternative in `std` — the `parking_lot::Mutex` guard pattern is best practice
- [x] `#![deny(unsafe_code)]` + per-fn `#[expect(unsafe_code, reason = "...")]` — maximum visibility

---

## Completed (Mar 24, 2026 — Cross-Ecosystem Absorption, JSON-RPC Strict, Cast Deny, Constants, Ergonomics — Session 16)

### Wave 69: Strict JSON-RPC 2.0 Compliance
- [x] `JsonRpcRequest.id` → `Option<Value>` across all 3 type definitions (tower_atomic, jsonrpc_api, unix_listener)
- [x] Notification suppression in 5 connection handlers (tower_atomic, HTTP gateway, unix_listener, ipc/unix/server, pure_rust_server, bin_interface×2)
- [x] Serialization-safe fallback in `write_response()` — hard-coded internal-error JSON on serialization failure
- [x] Version validation already present — verified across all handlers

### Wave 69: Cast Lint Discipline
- [x] Added `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` as `"deny"` at workspace level
- [x] Removed per-crate `allow` overrides from `songbird-orchestrator/Cargo.toml` (were masking nothing)
- [x] Fixed `unused_async` lint in `songbird-genesis` coordination bridge with `#[expect(reason)]`
- [x] Zero cast violations across all 30 crates (all existing code already used safe conversions or `#[expect]`)

### Wave 69: Ecosystem Hygiene
- [x] Created `SECURITY.md` (aligned with BearDog/groundSpring/airSpring patterns)
- [x] `rustfmt.toml` already present — verified current

### Wave 69: Primal Name Constants
- [x] Created `songbird_types::primal_names` module (`SELF_NAME`, `APP_DIR`, `BEARDOG`, `SQUIRREL`, `TOADSTOOL`, `NESTGATE`)
- [x] Replaced ~15 raw `"songbird"` literals across env_config, primal_discovery, capability_registration, config modules, unified core, paths, system config
- [x] Replaced `"beardog"` in socket_discovery with `primal_names::BEARDOG`
- [x] Platform endpoint maps left as-is (need deeper architectural change to capability-based)

### Wave 69: `impl Into<String>` Ergonomics
- [x] `ServiceInstance::new` + `with_capability` + `with_health_status` + `with_metadata` (songbird-discovery)
- [x] `ServiceRequest::new` + `with_header` + `with_query_param` (songbird-discovery)
- [x] `ServiceResponse::success` + `error` + `with_header` (songbird-discovery)
- [x] `Provider::new` (songbird-universal-ipc)
- [x] `AnonymousDiscoveryMessage::new_v3` node_id/node_name (songbird-discovery)

### Wave 69: CONTRIBUTING.md
- [x] Updated coverage reference to ~66.59%

---

## Completed (Mar 24, 2026 — sysinfo Elimination, Dead Code Removal, Coverage Push, Doc Cleanup Session 15)

### Wave 68: sysinfo Elimination (ecoBin v3.0)
- [x] Created `songbird_types::sys_metrics` — pure Rust `/proc/meminfo` + `/sys/block/*/size` readers (12 tests)
- [x] Replaced all `sysinfo` usage across 4 crates (orchestrator, cli, registry, observability)
- [x] Eliminated `sysinfo` + `rayon` + `crossbeam-*` from production dependency tree (~6 transitive crates removed)
- [x] Zero unsafe code in `sys_metrics` — all file I/O via `std::fs`

### Wave 68: Dead Code Removal (~48KB)
- [x] Deleted `songbird-observability/src/monitoring/` (4 files, broken syntax, deprecated sysinfo 0.29 API, not in module tree)
- [x] Deleted `songbird-registry/src/health/` (1 file, broken syntax, not in module tree)
- [x] Deleted `songbird-registry/src/scaling/` (1 file, broken syntax, not in module tree)
- [x] Removed empty `songbird-universal-ipc/data/sovereign-onion/blobs/` directory
- [x] Cleaned stale `sysinfo` references in comments (federation.rs, router.rs, checks.rs)

### Wave 68: Coverage Expansion (+121 tests across 8 modules)
- [x] Circuit breaker: 13 new tests (state machine, config validation, error types)
- [x] Connection pool: 17 new tests (lifecycle, concurrency, stale eviction)
- [x] Consent enforcement: 12 new tests (dignity rules, timeout behavior, cost thresholds)
- [x] Primal self-knowledge: 12 new tests (env discovery, error paths, mechanism names)
- [x] Observability metrics: 24 new tests (collection, export, caching, concurrency)
- [x] TLS key schedule: 13 new tests (HKDF extract/expand, derive secret, full schedule flow)
- [x] Beardog birdsong provider: 13 new tests (mock TCP JSON-RPC, encrypt/decrypt roundtrip)
- [x] Lineage beardog relay: 17 new tests (mock JSON-RPC, masking, fail-closed paths)

### Wave 68: Production Mock Audit
- [x] Confirmed all `Mock*` types isolated to `#[cfg(test)]` modules — zero production mocks

### Wave 68: Doc Cleanup
- [x] Updated README.md, CONTEXT.md, REMAINING_WORK.md, CHANGELOG.md with accurate metrics
- [x] Updated CONTRIBUTING.md coverage reference
- [x] Cleaned stale sysinfo references in code comments
- [x] Test count: 10,100 → 10,233 (0 failed, 266 ignored)

---

## Completed (Mar 24, 2026 — Comprehensive Audit, Flaky Test Fix, Smart Refactor, Dependency Analysis Session 14)

### Wave 67: Flaky Test Elimination
- [x] `test_port_allocation_is_cached` (songbird-test-utils): Eliminated race condition — removed `clear_port_registry()` from concurrent tests, unique capability names per test
- [x] `test_bridge_poll_interval` (songbird-orchestrator): Evolved wall-clock timing to `tokio::time::pause()` — deterministic virtual time, zero flake potential
- [x] `test_connectivity_timeout` (songbird-orchestrator): Same `tokio::time::pause()` evolution — exact Duration comparison with 50ms tolerance for runtime overhead
- [x] Full workspace: 10,085 passed, 0 failed (was 1 failed before this session)

### Wave 67: Smart Refactor — `crypto.rs` (1100 → 578 + 454)
- [x] Extracted test module to `crypto_tests.rs` via `#[path]` module (454 lines, 26 tests)
- [x] Deduplicated `call_capability` and `call_jsonrpc` into shared `send_request` method
- [x] Evolved `serde_json::to_string` → `serde_json::to_vec` for zero-copy request serialization
- [x] Production code: 578 lines (was 631 pre-dedup); zero files over 1000 lines

### Wave 67: Profraw Artifact Cleanup
- [x] Cleaned ~48GB of `.profraw` files from `crates/songbird-orchestrator/` (llvm-cov artifacts)
- [x] `.gitignore` already had `*.profraw` pattern; files were local-only

### Wave 67: Dependency Analysis + sysinfo Elimination (ecoBin v3.0)
- [x] `sysinfo` v0.30: **FULLY ELIMINATED** — replaced by `songbird_types::sys_metrics` pure Rust module
- [x] Created `sys_metrics` module: reads `/proc/meminfo` for memory, `/sys/block/*/size` for disk (12 tests)
- [x] Replaced all `sysinfo` usage across 4 crates (orchestrator, cli, registry, observability)
- [x] Eliminated `sysinfo` + `rayon` + `crossbeam-*` from production dependency tree
- [x] `ring` v0.17: Only via songbird-quic → quinn (opt-in `ring-crypto` feature, not default)
- [x] Removed 3 dead code directories: `observability/monitoring/`, `registry/health/`, `registry/scaling/` (~24KB broken code)

### Wave 67: Unsafe Code Audit
- [x] `songbird-process-env`: 2 unsafe blocks are irreducible — Rust 2024 `std::env::set_var`/`remove_var` require unsafe; mutex-guarded, startup-only, `#![deny(unsafe_code)]` at crate level
- [x] No other unsafe in workspace (29/30 crates `#![forbid(unsafe_code)]`, 1 `#![deny(unsafe_code)]` with per-fn `#[expect]`)

---

## Completed (Mar 23, 2026 — Comprehensive Audit, cargo-deny, CI, Coverage, Stub Evolution Session 13)

### Wave 66: cargo-deny Evolution
- [x] Fixed license allowlist: added `MPL-2.0`, `Zlib` for transitive dependencies (`colored`, `option-ext`)
- [x] Evolved wildcards policy from `deny` to `allow` (workspace member deps are inherently wildcarded)
- [x] Added `skip` list for known transitive duplicate crates (windows-sys, syn, parking_lot, etc.)
- [x] Corrected all advisory IDs to actual RUSTSEC identifiers (RUSTSEC-2026-0007, -0009, -2025-0141, etc.)
- [x] cargo-deny now fully passing: `advisories ok, bans ok, licenses ok, sources ok`

### Wave 66: CI Modernization
- [x] Ratcheted coverage threshold from 58% to 66% (current actual; target 90%)
- [x] Replaced `actions/cache@v3` with `Swatinem/rust-cache@v2` across all CI jobs
- [x] Added dedicated `cargo-deny` job to quality-checks pipeline
- [x] Replaced `cargo audit` + `continue-on-error` with `rustsec/audit-check@v2` (gates PRs)
- [x] Added `--all-features` to build/test/coverage/doc CI jobs
- [x] Upgraded `codecov/codecov-action` from v3 to v4

### Wave 66: SPDX Header Compliance
- [x] Added `// SPDX-License-Identifier: AGPL-3.0-only` + copyright to 37 files missing headers
- [x] All `.rs` files under crates/, src/, tests/ now have SPDX headers (100% compliance)

### Wave 66: Lint Evolution
- [x] Migrated `songbird-bluetooth` from `clippy::all = "allow"` to `[lints] workspace = true`
- [x] Fixed 3 clippy errors in bluetooth src (Arc::clone, unfulfilled expect)
- [x] Removed blanket lint suppressions from `songbird-stun/src/lib.rs`
- [x] Fixed production `expect()` in stun client (evolved to `let-else`)
- [x] Added proper `#![allow(reason)]` to all stun test modules
- [x] 30/30 crates now use workspace lints; only 2 justified custom tables remain

### Wave 66: Production Stub Evolution
- [x] Evolved mDNS `query_mdns_services` from empty stub to real multicast UDP implementation
- [x] Evolved compute-bridge no-backend mock response to proper `SERVICE_UNAVAILABLE` error
- [x] Evolved `get_workload_handler` from `NOT_IMPLEMENTED` mock to capability-required error
- [x] Evolved IGD `get_local_ip()` from hardcoded `8.8.8.8:53` to gateway-based local detection
- [x] Updated test to match new compute-bridge behavior

### Wave 66: tarpaulin.toml Cleanup
- [x] Removed references to 8 nonexistent crates in `exclude-files` (songbird-federation, -network, -security, -universal-primals, etc.)

### Wave 66: Coverage Expansion (+65 tests)
- [x] TLS crypto.rs: JSON-RPC loopback tests, chacha20/ed25519/hmac/x25519 helpers, platform connect
- [x] Orchestrator: broadcast address discovery (7), workload classification (14), env config (8)
- [x] Config: providers.rs, capability discovery types/impl, hardcoded_elimination, universal_primals (~22)
- [x] Coverage: 66.20% → 66.96% (10,301 → 10,366 tests, 0 failed)

---

## Completed (Mar 23, 2026 — Doc Cleanup, CI Evolution, Handoffs & Debris Removal Session 12)

### Wave 65: Root Doc Updates
- [x] README.md: updated metrics (10,020 tests, 62.27% coverage, 14 JSON-RPC methods, 30/30 lint inheritance, method normalization)
- [x] CHANGELOG.md: added v0.2.1-wave64 entry for session 11 work
- [x] CONTRIBUTING.md: updated coverage reference (62.27%), added `-D warnings` to doc check command
- [x] CONTEXT.md: updated test count and coverage metrics
- [x] REMAINING_WORK.md: corrected coverage in pending section (62.27%, not 66.02%)
- [x] tests/README.md: rewritten to reflect actual test architecture (was stale with unreachable `cargo test --test e2e` commands)

### Wave 65: CI Workflow Evolution
- [x] Deleted `vendor-hardcoding-check.yml` (referenced nonexistent `tools/vendor_pattern_migrator/`, `agnostic_service_discovery.rs`, `infant_discovery.rs`)
- [x] Deleted `ci-cd.yml` (overlapped with `ci.yml`; referenced nonexistent `scripts/deploy-ecosystem.sh`, `tests/performance/load-test.js`)
- [x] Consolidated `ci.yml` into clean 3-job pipeline (check + test + build-release) using `dtolnay/rust-toolchain` + `Swatinem/rust-cache`
- [x] Fixed `quality-checks.yml`: removed `continue-on-error` on clippy (zero warnings now), added `--workspace` flag, removed `--document-private-items` from doc check, replaced missing `scripts/find_production_unwraps.sh` and `scripts/audit_hardcoding.sh` with inline checks
- [x] Remaining CI: `ci.yml` (primary), `quality-checks.yml` (extended), `coverage.yml`, `production-deploy.yml`

### Wave 65: wateringHole Handoff
- [x] Archived Wave 62 + Wave 63 handoffs to `handoffs/archive/`
- [x] Created `SONGBIRD_V021_WAVE64_NAMING_CONVERGENCE_LINT_UNIFICATION_MAR23_2026.md` with SPDX header, quality gate table, inter-primal notes

### Wave 65: baseCamp Update
- [x] Updated `ecoPrimals/whitePaper/gen3/baseCamp/EXTENSION_PLAN.md` foundation line with Songbird v0.2.1-wave64 status
- [x] Updated cross-institution provenance table entry with latest Songbird capabilities

### Wave 65: Debris Cleanup
- [x] Deleted stale `tests/README_E2E_TESTS.md` (referenced wrong crate paths and test names)
- [x] Removed stale `exclude = ["examples/clients/rust"]` from workspace Cargo.toml (path doesn't exist)
- [x] Removed empty `crates/songbird-orchestrator/sqlite::memory:/blobs` (test artifact from bad path)
- [x] Removed empty `crates/songbird-orchestrator/src/app/modules/` (unused directory)

---

## Completed (Mar 23, 2026 — Cross-Ecosystem Absorption, Naming Convergence & Lint Unification Session 11)

### Wave 64: Ecosystem Method Naming Convergence
- [x] Added `health.readiness` JSON-RPC method (IPC + HTTP gateway) — subsystem status reporting
- [x] Added `health.check` JSON-RPC method (IPC + HTTP gateway) — full health with details; aliases `status`, `check`, `health`
- [x] Added `normalize_method()` in `songbird-universal-ipc/introspection` — canonicalizes ecosystem naming drift
- [x] `capability.list` → `capabilities.list` (biomeOS/Squirrel alias tolerance)
- [x] `ping` → `health.liveness`, `status`/`check`/`health` → `health.check`
- [x] Both IPC service handler and HTTP JSON-RPC gateway dispatch through `normalize_method()`
- [x] Updated `rpc_discover_standard()` to advertise `health.liveness`, `health.readiness`, `health.check`, `capabilities.list`
- [x] 7 new tests for normalization, readiness, and health check functions

### Wave 64: Identity-Based Discovery Elimination
- [x] Evolved `handle_health_standard` — removed hardcoded `BEARDOG_SOCKET` / `beardog.sock` identity-based discovery
- [x] Now uses capability-based 5-tier: `CRYPTO_PROVIDER_SOCKET` → `CRYPTO_SIGN_PROVIDER_SOCKET` → XDG family-scoped socket
- [x] Response field renamed `beardog_connected` → `crypto_provider_available` (capability, not identity)

### Wave 64: Rustdoc Fix
- [x] Fixed private intra-doc link in `health.rs` (`start_health_monitoring` linked to private `run_comprehensive_health_check`)
- [x] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` now passes clean

### Wave 64: Production Safety
- [x] Evolved `unreachable!()` in `http_server.rs:483` → `Err(anyhow!(...))` return (zero production `unreachable!()`)

### Wave 64: Workspace Lint Unification
- [x] Added `[lints] workspace = true` to 15 crates previously missing lint config
- [x] All 30 crates now inherit workspace pedantic+nursery lints (3 with justified custom tables)
- [x] Fixed all clippy errors from lint inheritance (unwrap_used/expect_used scoped to test modules)

### Wave 64: wateringHole Standards Compliance
- [x] Created `CONTEXT.md` at repo root (PUBLIC_SURFACE_STANDARD requirement)
- [x] AI-ingestible context block: role, capabilities, IPC surface, dependencies, metrics

### Wave 64: Cross-Ecosystem Audit
- [x] Reviewed 7 springs: hotSpring, groundSpring, neuralSpring, wetSpring, airSpring, healthSpring, ludoSpring, primalSpring
- [x] Reviewed 13 primals: BearDog, NestGate, Squirrel, ToadStool, biomeOS, petalTongue, rhizoCrypt, LoamSpine, sweetGrass, sourDough, skunkBat, barraCuda, coralReef
- [x] Documented absorption opportunities from primalSpring Phase 12 (bonding, STUN sovereignty, DispatchOutcome)
- [x] Identified and resolved 4 OPEN items from primalSpring capability audit

---

## Completed (Mar 23, 2026 — Comprehensive Clippy Sweep, Smart Refactoring & Metrics Accuracy Session 10)

### Wave 63: Full Workspace Clippy Pedantic+Nursery Sweep (~800+ warnings resolved)
- [x] `songbird-types`: Fixed `significant_drop_tightening` (RwLock guard → Arc clone pattern), 11x `doc_markdown`
- [x] `songbird-crypto-provider`: 6x `doc_markdown` (`BearDog` backticks), `missing_errors_doc`, `must_use_candidate`
- [x] `songbird-tls`: `manual_let_else`, `doc_markdown`, 5x `unreadable_literal`, 4x `redundant_clone`, 4x `uninlined_format_args`, test cast expects
- [x] `songbird-http-client`: 131 errors — TLS cast truncation expects, `# Errors`/`# Panics` docs, `unused_self`, `unnecessary_wraps` removal, `branches_sharing_code`, `map_or_else`, `match_same_arms`, profiler precision loss, f32 test comparisons
- [x] `songbird-tor-protocol`: 24 errors — `map_crypto_err` reference fix, `# Panics` docs, `publish_descriptor` sync evolution, lock scoping
- [x] `songbird-sovereign-onion`: 33 errors — `must_use`/`const fn`, `# Errors`/`# Panics`, `BearDog` backticks, `use_self`, standalone feature gates
- [x] `songbird-config`: 4 errors — `option_if_let_else`, `missing_const_for_fn`
- [x] `songbird-execution-agent`: 2 errors — `doc_markdown`
- [x] `songbird-remote-deploy`: 3 errors — `implicit_hasher`, `missing_errors_doc`
- [x] `songbird-discovery`: 14 errors — `missing_const_for_fn`, `significant_drop_tightening` in health loop, test doc backticks
- [x] `songbird-registry`: 8 errors — `const fn`, lock scope tightening, `uninlined_format_args`
- [x] `songbird-primal-coordination`: 3 errors — `doc_markdown`, `significant_drop_tightening`, `uninlined_format_args`
- [x] `songbird-lineage-relay`: 22 errors — lock scope tightening across 6 files, `const fn`, `map_or_else`, `manual_assert`
- [x] `songbird-onion-relay`: 43 errors — `unix_epoch_millis_u64` helper, lock scoping, `must_use`, `const fn`, `.onion` path extension
- [x] `songbird-universal-ipc`: 30 errors — `significant_drop_tightening` (Arc clone pattern), `option_if_let_else`, `derive_partial_eq_without_eq`, `const fn`
- [x] `songbird-orchestrator`: 638 errors — `# Errors` docs (308), `unused_async` removal (90), lock tightening (61), float comparison expects (60), format args, `map_or_else`, `must_use`, crate-level lint policy alignment
- [x] `songbird-universal`: 5 errors — `missing_errors_doc` on discovery resolvers, `map_or_else`
- [x] `songbird-compute-bridge`: 1 error — `missing_errors_doc`
- [x] Root `songbird`: 7 errors — `must_use`, `# Errors`, `doc_markdown`, `uninlined_format_args`
- [x] Auto-fixed examples and integration tests via `cargo clippy --fix`

### Wave 63: Flaky Test Fix
- [x] `test_port_allocation_is_cached`: Race condition from concurrent `clear_port_registry()` — evolved `test_different_capabilities_get_different_ports` to use unique capability names instead of clearing global state

### Wave 63: Smart Refactoring
- [x] `compute_api.rs` (977 lines) → `compute_api/` directory module (mod.rs 266 + compute_handlers.rs 448 + compute_types.rs 185 + compute_state.rs 117 + compute_routing.rs 31)
- [x] `real_service_discovery.rs` (923 lines) → `real_service_discovery/` directory module (mod.rs 153 + types.rs 76 + health.rs 85 + conversions.rs 78 + service_discovery_impl.rs 172 + tests.rs 412)

### Wave 63: Metrics Accuracy
- [x] Updated REMAINING_WORK.md with llvm-cov measured coverage (66.02%, was incorrectly listed as ~72%)
- [x] Corrected test count to actual measured values (7,304 `#[test]` + 2,719 `#[tokio::test]` = 10,023 total)
- [x] Corrected `#[ignore]` count to 191 (was incorrectly 266)
- [x] Updated max file size (959 test file, 888 production)
- [x] Updated total Rust lines to 405,736
- [x] Updated build times to current measurements

---

## Completed (Mar 23, 2026 — Stub Evolution, Smart Refactoring & Health Probe Modernization Session 9)

### Wave 62: CLI Discovery Stubs → Real Implementations
- [x] `discover_via_subnet_scan` — real TCP probes on local /24 via `tokio::net::TcpStream`
- [x] `discover_via_dns` — DNS-SD SRV lookup via `hickory-resolver::TokioAsyncResolver`
- [x] `discover_via_mdns` — UDP multicast query to mDNS group with JSON response parsing
- [x] `discover_via_broadcast` — UDP broadcast with `SO_BROADCAST` and response collection
- [x] Fixed pre-existing `clap` missing `env` feature in `songbird-cli`

### Wave 62: Smart Refactor — `tls/record.rs` (911 → 454 lines)
- [x] Extracted `record_crypto.rs` (140 lines): `build_nonce()`, `cipher_encrypt()`/`cipher_decrypt()`, `cipher_suite_name()`
- [x] Replaced duplicated inline alert tables with existing `TlsAlert::parse()` from `alert.rs`
- [x] Consolidated verbose diagnostic trace blocks into concise structured logging

### Wave 62: Smart Refactor — `canonical/hardcoded_elimination.rs` (931 → 532 lines)
- [x] Extracted `port_config.rs` (340 lines): `PortConfig`, env-driven loading, validation, capability-registry bridge
- [x] Transparent re-export — zero downstream API changes

### Wave 62: Security Health Stubs → Real Crypto-Provider Probes
- [x] `ServerManager::check_security_integration_health` — orchestrator status + `discover_crypto_provider()`
- [x] `SongbirdOrchestrator::check_security_integration_health` — same real probe pattern

### Wave 62: Debris Cleanup
- [x] Removed stale plan files from crates (4 REFACTOR_PLAN.md files)
- [x] Updated `specs/00_SPECIFICATIONS_INDEX.md` version/date alignment
- [x] Fixed `examples/README.md` — removed references to nonexistent `legacy/` and `clients/rust/`
- [x] Updated root docs (README.md, REMAINING_WORK.md) with current metrics

---

## Completed (Mar 22, 2026 — Deep Coverage, Zero-Copy, Fuzz & Mock Evolution Session 7)

### Wave 54: Deep Orchestrator Coverage (+960 tests)
- [x] JSON-RPC API: Full handler coverage — `compute.route`, `deployment.create`, `task.create`, consent, protocol, services, registry, federation, health, version, identity, beacon
- [x] Axum routes: Invalid `jsonrpc` version → `INVALID_REQUEST`, unknown method → `METHOD_NOT_FOUND`
- [x] `core.rs`: `discover_broadcast_addresses` with env override, config merging, subnet fallbacks
- [x] `discovery_bridge.rs`: Hyphenated family ID tag coverage
- [x] `node_identity.rs`: Serde roundtrip, `new_or_load` stability with temp data dir
- [x] `security_client.rs`: Response parsing — non-2xx, valid JSON, garbage body
- [x] `capability_router.rs`: Multi-provider capability flattening

### Wave 55: Deep Networking Coverage (4 crates)
- [x] `songbird-discovery`: federation-aware discovery (new module wired into lib.rs), real service discovery JSON serde, BearDog birdsong TCP/encrypt/decrypt, dark forest beacon serde/builders, broadcaster v2/v3, primal self-knowledge
- [x] `songbird-network-federation`: multi-federation routing/IPv6/trust, federation config serde, node info roundtrip, state capability merge/endpoint ordering, gaming protocol handlers/sessions
- [x] `songbird-lineage-relay`: relay protocol malformed lengths/JSON, server stats/masking, BearDog lineage chains
- [x] `songbird-tls`: crypto/handshake/key_schedule test modules, socket discovery priority tests

### Wave 56: Federation Mock Evolution → Real State
- [x] `FederationPeersResponse` / `FederationStatusResponse` typed structs (removed inline `serde_json::json!` + debug `comment` fields)
- [x] `IpcServiceHandler::with_federation_state(registry, Arc<FederationState>)` wires live federation data
- [x] `handle_federation_peers_rpc`: queries real `FederationState` — sorted active node IDs, live counts
- [x] `handle_federation_status_rpc`: real `active_nodes` from federation stats
- [x] Orchestrator `http_server.rs` wired to pass federation state to IPC handler
- [x] Backward-compatible JSON shapes (same field names, `comment` removed)

### Wave 57: Large File Refactoring (6 files)
- [x] `environment.rs` (910) → extracted tests to `environment_tests.rs`
- [x] `ai.rs` (908) → extracted tests to `ai_tests.rs`
- [x] `escalation.rs` (867) → extracted tests to `escalation_tests.rs`
- [x] `service_registry.rs` (860) → extracted tests to `service_registry_tests.rs`
- [x] `advanced_cache.rs` (861) → extracted tests to `advanced_cache_tests.rs`
- [x] `federation_aware_discovery.rs` (1097) → extracted tests to `federation_aware_discovery_tests.rs` (730 LOC prod)

### Wave 58: Zero-Copy Evolution
- [x] `songbird-http-client`: Borrow-through redirect loop (no header/body clones per hop)
- [x] `songbird-universal-ipc`: JSON-RPC `id` moved by value (removed `Value::clone` per request), mesh endpoint labels → `&'static str`
- [x] `songbird-tls`: HKDF buffer reuse (eliminated `Vec<u8>` clone per block iteration)
- [x] `songbird-types`: `HashMap::with_capacity` pre-sizing for endpoint maps

### Wave 59: Fuzz-Style Parsing Tests
- [x] TLS record layer: 7 tests — random 1-byte headers, malformed headers, invalid content types, truncated records, max/oversize lengths, empty payloads
- [x] JSON-RPC parsing: 7 tests — malformed JSON, missing fields, various `id` types, deep nesting, 50k method names, unicode
- [x] Lineage relay protocol: 6 tests — truncated allocate, malformed JSON, truncated data packets, zero-length, unknown types, refresh/deallocate roundtrip
- [x] STUN message: 5 tests — short inputs never panic, truncated headers, oversized length claims, invalid types, binding request roundtrip

### Wave 60: Clippy Compliance
- [x] `bool as usize` → `usize::from(bool)` in environment.rs
- [x] `repeat().take()` → `repeat_n()` in TLS record layer tests
- [x] Collapsible `if` statements in federation discovery
- [x] `map_or` → `is_none_or` in pattern confidence
- [x] `&SovereigntyLevel` pass-by-ref, `Ipv4Addr::LOCALHOST`, single-char pattern
- [x] Variable naming disambiguation in IPC federation handlers

---

## Completed (Mar 22, 2026 — Comprehensive Audit & Evolution Session 6)

### Wave 49: Smart File Refactoring (Production/Test Separation)
- [x] `service.rs` (973 lines) → `service.rs` (681) + `service_tests.rs` (295) via `#[path]` module
- [x] `storage.rs` (927 lines) → `storage.rs` (362) + `storage_tests.rs` (565) via `#[path]` module
- [x] `compute_api.rs` (930 lines) → extracted `update_job_status`, `discover_http_client`, `serialize_task` helpers; `submit_compute_task` reduced from 422 to ~250 lines

### Wave 50: TLS Diagnostic Logging Security Fix
- [x] `record.rs`: Evolved `info!`-level diagnostic blocks (key material, nonces, hex dumps, "════" dividers) to `trace!`
- [x] Protocol details (cipher suites, sequence numbers) to `debug!`
- [x] Operational events (connection open/close, write/read) kept at `info!`
- [x] Security fix: key material no longer exposed at default log levels

### Wave 51: Production `eprintln!` → `tracing` Evolution
- [x] `hosts_evolved.rs`: `announce_via_environment` — 4x `eprintln!` → structured `tracing::info!`
- [x] `runtime_engine.rs`: backend registration failure `eprintln!` → `tracing::warn!`
- [x] Zero `eprintln!` remaining in library crates (CLI binary output is intentional)

### Wave 52: Coverage Expansion (+100 new tests across 6 crates)
- [x] `songbird` root: `src/lib.rs` coverage 25% → 83.59% (CLI parsing, rendezvous path, interactive REPL)
- [x] `songbird-universal`: `types/config.rs` and `types/service.rs` from 0% (wired test modules, round-trip tests)
- [x] `songbird-universal`: discovery engine, health checker, unified adapter coverage expanded
- [x] `songbird-orchestrator`: consent enforcement, peer trust, escalation, circuit breaker, cache, process manager
- [x] `songbird-config`: capability discovery, discoverable endpoint, runtime discovery, paths
- [x] `songbird-universal-ipc`: introspection, tower atomic, capability discovery
- [x] `songbird-http-client`: connection pool, client, http config

### Wave 53: Clippy Compliance Sweep
- [x] Fixed 13 unfulfilled `#[expect]` → `#[allow]` (per wateringHole `expect` = fires, `allow` = doesn't fire)
- [x] Fixed 2 redundant `.clone()` calls (advanced_cache, runtime_discovery)
- [x] Fixed `contains()` vs `iter().any()` in introspection tests
- [x] Fixed `if` with identical blocks in discoverable_endpoint
- [x] Zero clippy warnings across all 30 crates with `--all-features --all-targets`

---

## Completed (Mar 22, 2026 — Comprehensive Audit & Deep Debt Session 5)

### Wave 42: Build Restoration & New Crate Compliance
- [x] Fixed compilation error: added `set_user_preferences` / `get_user_preferences` methods to `ConsentManager`
- [x] Fixed `songbird-crypto-provider` Cargo.toml: added `readme = "README.md"` + created crate README
- [x] Fixed 7 clippy errors in `songbird-crypto-provider` (collapsible if, match-same-arms, dead_code → expect)
- [x] Fixed `ref_as_ptr` in `advanced_cache.rs` — evolved to `std::ptr::from_ref`
- [x] Fixed collapsible if in `songbird-universal-ipc/onion_handler.rs`
- [x] Fixed format string interpolation in `songbird-lineage-relay/coordinator.rs`
- [x] Added `EnvReader` type alias in `songbird-universal-ipc/service.rs` (type complexity)
- [x] Fixed `clippy::doc_markdown` — BirdSong → `BirdSong`

### Wave 43: Standards Compliance (Doc Links & SPDX)
- [x] Fixed 5 broken intra-doc links in `songbird-universal`, `songbird-orchestrator`, `songbird-config`
- [x] Fixed 13 redundant explicit doc link targets in `songbird-config` (`[`name`](name)` → `[`name`]`)
- [x] Added SPDX headers to 3 files: `client_impl.rs`, `canonical_tests.rs`, `service_tests.rs`
- [x] 100% SPDX coverage: 1,324/1,324 `.rs` files

### Wave 44: Hardcoded Elimination
- [x] `songbird-lineage-relay/coordinator.rs`: port 42424 → `DEFAULT_BIRDSONG_PORT` const + `SONGBIRD_BIRDSONG_PORT` env var
- [x] `songbird-orchestrator/capability_adapters.rs`: `localhost:8000` → `DEFAULT_ORCHESTRATOR_URL` const + `SONGBIRD_ORCHESTRATOR_URL` env var
- [x] `songbird-orchestrator/ai_workload_classification`: `localhost:8002` → `DEFAULT_AI_ENDPOINT_URL` const + `SONGBIRD_AI_ENDPOINT` env var

### Wave 45: wateringHole Nest Atomic Compliance
- [x] Added `health.liveness` JSON-RPC method (IPC + HTTP gateway) — returns `{"status": "healthy"}`
- [x] Added `capabilities.list` JSON-RPC method (IPC + HTTP gateway) — returns 14 capability tokens
- [x] Added `SONGBIRD_CAPABILITY_STRINGS` const table (single source of truth)
- [x] Both methods work standalone (no IPC handler required in gateway)
- [x] Tests: `health_liveness_is_minimal`, `capabilities_list_matches_const_table` + integration tests

### Wave 46: ecoBin Compliance — QUIC Test Gating
- [x] Gated 5 QUIC test modules behind `#[cfg(all(test, feature = "ring-crypto"))]`
- [x] `cargo test -p songbird-quic` → 4 tests (error module only, no ring needed)
- [x] `cargo test -p songbird-quic --features ring-crypto` → 19 tests (full QUIC)
- [x] Default workspace test suite no longer requires ring

### Wave 47: Test Fix — Validation Alignment
- [x] Fixed `test_validate_port_zero_with_discovery_disabled` — test expected failure but validation correctly allows port 0 when discovery disabled (IPC-only mode)

### Wave 48: Production Unwrap Audit
- [x] Audited top 5 files by unwrap/expect count (196 total hits)
- [x] Confirmed 0 production `.unwrap()` / `.expect()` — all 196 inside `#[cfg(test)] mod tests`
- [x] All test modules have `#![allow(clippy::unwrap_used, reason = "test assertions")]`

---

## Deep Debt Audit (Mar 20, 2026)

| Principle | Status | Evidence |
|-----------|--------|----------|
| Zero `unsafe` | S+ | `#![forbid(unsafe_code)]` across 29/30 crates; 2 justified blocks in `process-env` with Mutex guard, `#![deny(unsafe_code)]` + per-fn `#[allow]` |
| Pure Rust | S+ | SHA3-256, SSDP, SOAP, NAT-PMP, base64, hex from scratch; `ring` opt-in only (not default) |
| Zero production stubs | S+ | All stubs evolved: metrics → `AtomicU64` counters, AI classification → typed workload analysis, relay → lockless `AtomicU64` stats |
| Zero production `panic!()` | S+ | All removed — replaced with `Result`-based error returns |
| Zero `todo!()` in production | S+ | Only in `#[cfg(test)]` functions |
| Zero `.unwrap()` in production | S+ | All remaining are in test modules |
| Zero TODO/FIXME in code | S+ | Tracked in this file instead |
| `#[expect()]` with reasons | S+ | `#[expect(reason)]` where lint fires; `#[allow(reason)]` where unfulfilled; zero unfulfilled expectations |
| Runtime discovery | S+ | All socket paths: env → XDG → fallback; `find_primals_with_capability` capability-based |
| Event-driven architecture | S+ | Zero polling anti-patterns in production code |
| Concurrent-safe testing | S+ | Zero `#[serial_test::serial]`; injectable `_with` env readers across all crates; 10,235 tests fully concurrent |
| Self-knowledge only | S+ | Introspection describes only Songbird |
| AGPL-3.0 license | S+ | `license.workspace = true` (all crates), `AGPL-3.0-only` SPDX headers, cargo-deny configured |
| Capability-based discovery | S+ | `primal_names` constants module; capability-first runtime discovery; env-driven filter |
| Mock isolation | S+ | All mocks behind `#[cfg(test)]` or `feature = "test-mocks"` |
| File size discipline | S+ | 0 files over 1000 lines; 5 near-limit files refactored into domain submodules |

---

## Completed (Mar 21, 2026 — Deep Debt Execution Session 4)

### Wave 28: License Compliance
- [x] 22 crate `Cargo.toml` files changed from `license = "AGPL-3.0"` to `license.workspace = true` (resolves to `AGPL-3.0-only`)
- [x] All crates now inherit license from workspace — single source of truth

### Wave 29: Smart Constants Refactoring
- [x] `canonical/constants.rs` (1,199 lines) → `constants/mod.rs` (752) + `primal_discovery.rs` (352) + `directories.rs` (131)
- [x] Domain-aligned extraction: primal endpoint discovery + capability filtering in one module, platform directories in another
- [x] All 541 config tests pass, zero regressions

### Wave 30: Dead Code → Public API Evolution
- [x] `discover_from_environment` on `CapabilityResolver` → evolved to `pub` (capability-based runtime discovery)
- [x] `introspect_name` / `introspect_capabilities` on `PrimalSelfKnowledge` → evolved to `pub` + `#[must_use]`
- [x] Both align with wateringHole primal self-knowledge requirements

### Wave 31: Lint Expectations Cleanup (5,325 warnings eliminated)
- [x] 299 files: bulk `#[expect(` → `#[allow(` for unfulfilled lint expectations in test code
- [x] 66 remaining real clippy warnings fixed (code quality improvements across all crates)
- [x] Final state: zero clippy warnings (`--all-targets --all-features --workspace`)

### Wave 32: ecoBin Compliance — Ring Crypto
- [x] `songbird-quic/Cargo.toml`: `default = ["ring-crypto"]` → `default = []`
- [x] `ring-crypto` is now opt-in only, not pulled into any default build
- [x] Aligns with ecoBin pure-Rust application code standard

### Wave 33: Dependency Deduplication
- [x] `base32`: 0.4 → 0.5 (aligned in `songbird-tor-protocol`)
- [x] `base64`: 0.21 → 0.22 (aligned in 6 crates)
- [x] `hostname`: 0.3 → 0.4 (aligned in 5 crates)
- [x] `thiserror`: workspace 1.0 → 2.0 (all direct deps aligned; transitive 1.x remains from upstream)
- [x] Remaining duplicates (syn, hashbrown, indexmap, getrandom, parking_lot, socket2, tower) are transitive

### Wave 34: Production Stub Evolution
- [x] `songbird-orchestrator/metrics` → real `ComputeMetrics` with `AtomicU64` counters + `MetricsCapabilityAdapter`
- [x] `songbird-orchestrator/ai_workload_classification` → full typed workload analysis with `AIWorkloadClassificationDelegate`
- [x] `songbird-orchestrator/app/core.rs` → removed deprecated `start_http_server` stub
- [x] `songbird-lineage-relay/relay.rs` → `Arc<Mutex<u64>>` bytes_relayed → `Arc<AtomicU64>` (lockless)

### Wave 35: Coverage Expansion (+169 new tests)
- [x] `songbird-orchestrator`: 95 new tests (consent enforcement, partial success, advanced cache, routing, trust escalation, graph validation)
- [x] `songbird-config`: 36 new tests (capability discovery, config loading, primal endpoint resolution)
- [x] `songbird-universal`: 22 new tests (JSON-RPC client, tarpc client, connection manager)
- [x] `songbird-http-client`: 12 new tests (IPC client, TLS record)
- [x] Test total: 9,730 → 9,899 (0 failed)

### Wave 36: File Size Compliance
- [x] `graph/validator.rs` (1,046 lines after test additions) → extracted to `validator.rs` (363) + `validator_tests.rs` (415)
- [x] Zero files over 1000 lines across entire workspace

### Wave 37: Zero-Copy Clone Audit
- [x] 19 unnecessary `.clone()` calls eliminated across 7 hot-path production files
- [x] `p2p_discovery.rs`: 10 clones eliminated via struct destructuring and field moves
- [x] `task_lifecycle/manager.rs`: 5 clones eliminated via log-then-move pattern
- [x] `trust/escalation.rs`: 2 clones eliminated via deferred insert and identity move
- [x] `discovery_bridge.rs`: 1 clone eliminated via `tags.take()` + conditional move
- [x] `real_service_discovery.rs`: 1 clone eliminated via `Copy` on `ServiceHealthStatus`

### Wave 38: Proactive Large File Refactoring
- [x] `jsonrpc_api.rs` (962) → `jsonrpc_api/` module directory (212 + handlers/* + types + tests)
- [x] `ipc_client/client.rs` (954) → `client/mod.rs` (93) + `client_impl.rs` (677) + `client_tests.rs` (191)
- [x] `capability_discovery.rs` (953) → `capability_discovery/` module directory (257 + types + discover_impl + tests)
- [x] `service.rs` (1,069 after tests) → `service.rs` (665) + `service_tests.rs` (404)
- [x] `canonical.rs` (1,024 after tests) → `canonical.rs` (888) + `canonical_tests.rs` (136)

### Wave 39: Coverage Expansion Wave 2 (+84 new tests)
- [x] `songbird-universal-ipc`: 28 tests (service handler routing, introspection, peer handler)
- [x] `songbird-discovery`: 28 tests (real service discovery, SSDP protocol, primal self-knowledge)
- [x] `songbird-types`: 28 tests (canonical adapter traits, config migration, primal types)
- [x] New SSDP discovery protocol module wired into crate

### Wave 40: Documentation Gap Closure
- [x] 81 `pub mod` declarations across 5 crates documented with `///` doc comments
- [x] Modules retain `#[allow(missing_docs)]` for internal items (incremental coverage)
- [x] Zero `#[allow(missing_docs)]` on any module that lacks a `///` doc comment

### Wave 41: Comprehensive Verification
- [x] `cargo fmt --check` → clean
- [x] `cargo clippy --all-features --all-targets --workspace` → zero warnings
- [x] `cargo test --all-features --workspace` → 9,983 passed, 0 failed
- [x] Zero `.unwrap()` / `.expect()` in production code (all in test modules)
- [x] Primal names centralized in `primal_names` constants module; capability-first discovery
- [x] All mocks test-gated (`#[cfg(test)]` or `feature = "test-mocks"`)
- [x] Zero files over 1000 lines
- [x] 2 unsafe blocks: justified, startup-only, mutex-guarded, documented

---

## Completed (Mar 21, 2026 — Fully Concurrent Architecture Evolution)

### Wave 27: Injectable Environment Readers — Zero Serial Tests
- [x] Evolved `Environment::detect()` → `detect_with(env)` across songbird-config
- [x] Evolved `CanonicalNetworkConfig::from_env()` → `from_env_reader(env)` with `default_from_env_reader`
- [x] Evolved `ZeroTouchConfig::from_environment()` → `from_environment_reader(env)`
- [x] Evolved `PortConfig/HostConfig/EndpointConfig::from_env()` → `from_env_reader(env)` (hardcoded elimination)
- [x] Evolved `ServiceEndpoints::get_by_capability()` → `get_by_capability_with(env, ...)`
- [x] Evolved `LogConfig/ResourceLimits/PerformanceParameters` → `from_env_reader(env)`
- [x] Evolved `RuntimeEndpointResolver::try_env_resolution()` → `try_env_resolution_with(capability, env)`
- [x] Evolved `CapabilityDiscoveryEngine` → `new_with_env_reader(methods, env)`
- [x] Evolved `RuntimeDiscovery::discover_by_capability()` → `from_environment_with(capability, env)`
- [x] Evolved `CapabilityBasedRuntimeDiscovery::discover_from_environment()` → `discover_from_environment_with(capability, env)`
- [x] Evolved `CapabilityDiscovery` → `with_methods_env_reader(methods, env)`
- [x] Evolved `EnvironmentStrategy::discover()` → `discover_with(capability, env)` in songbird-universal-ipc
- [x] Evolved `IpcServiceHandler::handle_identity()` → `with_family_id_env(registry, env)` injection
- [x] Evolved `PrimalSelfKnowledge::discover_self()` → `discover_self_with(env)` in songbird-discovery
- [x] Evolved `DiscoveryConfig` to accept `provider_endpoints: HashMap<String, String>` injection
- [x] Evolved all adapters (compute, ai, storage, security) to `new_from_discovery_with_resolver(resolver)`
- [x] Evolved `CapabilityEndpointResolver` to `with_endpoint_overrides(overrides)` for test injection
- [x] Evolved `canonical/constants.rs` — `get_bind_address_with`, `get_primal_endpoint_with`, `get_log_level_with`, `find_primals_with_capability_in_env`, `get_common_primal_ports_from_env_map`
- [x] Removed ALL `#[serial_test::serial]` from codebase (was 30+ usages)
- [x] Removed `set_var`/`remove_var` from ALL lib-internal `#[cfg(test)]` blocks
- [x] All 9,730 tests pass at `--test-threads=16` with zero races
- [x] `cargo llvm-cov` completes clean: 64.14% line, 63.11% branch, 63.23% region

---

## Completed (Mar 21, 2026 — Comprehensive Audit & Deep Debt Session 3)

### Wave 18: Orphaned Dead Code Removal (41 files, 11.5K lines)
- [x] Deleted `production_benchmarks/` (12 files) — severely malformed code, never compiled
- [x] Deleted `basic_iot/mod.rs` — broken syntax, never compiled
- [x] Deleted 10+ orphaned CLI command stubs (`scale.rs`, `firewall.rs`, `compose.rs`, `node.rs`, `share.rs`, `service.rs`, `basic_iot.rs`, `init.rs`, `join.rs`, `gaming/`)
- [x] Deleted orphaned test files across orchestrator, universal, discovery, CLI
- [x] Cleaned up `#[cfg(test)] #[path = "..."]` references to deleted files
- [x] Total: ~21,800 lines removed from codebase (404K → 383K)

### Wave 19: Unfulfilled Lint Expectations
- [x] Fixed `songbird-bluetooth/transport/mod.rs` — `#[expect(dead_code)]` → `#[allow(dead_code)]` (dual-feature gate)
- [x] Fixed `songbird-genesis/bluetooth_pure.rs` — same pattern
- [x] Fixed `songbird-compute-bridge/service.rs` — `#[expect]` → `#[allow]` for deserialized fields
- [x] Fixed `songbird-cli/resources.rs` — `#[expect]` → `#[allow]` for reserved API
- [x] Fixed `songbird-config/service_registry.rs` — `#[expect]` → `#[allow]` for planned fields
- [x] Zero compiler warnings across entire workspace

### Wave 20: Hardcoding Evolution
- [x] tarpc port: `SONGBIRD_TARPC_PORT` env var with `DEFAULT_TARPC_PORT` const fallback
- [x] All network ports in `CanonicalNetworkConfig` use `SafeEnv::get_port()` pattern
- [x] Issue templates rewritten — removed "Gaming Bridge" branding, reflects network orchestrator identity

### Wave 21: Version & Standards Alignment
- [x] Fixed version discrepancy: REMAINING_WORK.md aligned with Cargo.toml v0.2.1
- [x] All `#[ignore]` tests now have reason strings (14 bare ignores fixed)

### Wave 22: todo!() Test Stubs → Real Implementations
- [x] 7 workflow tests evolved from `todo!()` to real `execute_capability_workflow` assertions
- [x] 4 error handling tests evolved from `todo!()` to real error chain/context assertions
- [x] 2 tests properly marked `#[ignore = "reason"]` where API doesn't exist yet

### Wave 23: dead_code Evolution
- [x] `songbird-remote-deploy` — `#[allow(dead_code)]` → `#[expect(dead_code)]` for Deserialize fields
- [x] `songbird-compute-bridge` — same pattern for request fields

### Wave 24: Module Documentation (77 pub mod docs)
- [x] `songbird-orchestrator/src/lib.rs` — 36 module doc comments
- [x] `songbird-universal-ipc/src/lib.rs` — 10 module doc comments
- [x] `songbird-discovery/src/lib.rs` — 8 module doc comments
- [x] `songbird-http-client/src/lib.rs` — 9 module doc comments
- [x] `songbird-network-federation/src/lib.rs` — 14 module doc comments

### Wave 25: Coverage Expansion (+114 new tests)
- [x] `songbird-orchestrator`: 55 tests (consent, retry, scheduler, quota, admission, graph, registry, identity, self-knowledge, tokens, JWT)
- [x] `songbird-config`: 46 tests (constants, network/core, environment, runtime engine, infant config, capability/primal discovery)
- [x] `songbird-universal`: 5 tests (connection manager, discovery, circuit breaker manager)
- [x] `songbird-http-client`: 4 tests (connection pool, request, response, adaptive TLS)
- [x] `songbird-universal-ipc`: 4 tests (peer handler, discovery handler)
- [x] Test race conditions initially fixed with `#[serial_test::serial]` (superseded by Wave 27 injectable env readers)

### Wave 26: Ring Dependency Analysis
- [x] Documented exact dependency chain: songbird-quic → quinn → quinn-proto → ring; songbird-quic → rcgen → ring
- [x] Confirmed `default-features = false` on quinn eliminates ring (but requires alternative crypto backend)
- [x] Confirmed rcgen supports `default-features = false` with aws-lc-rs alternative
- [x] Pure-Rust QUIC path still blocked upstream (rustls-rustcrypto coverage gaps)
- [x] Documented migration steps and effort estimates in this file

---

## Completed (Mar 20, 2026 — Deep Debt Audit Session 2)

### Wave 7: Broken Tests & Doctests
- [x] Fixed failing doctest in `songbird-sovereign-onion/src/address.rs:147` — `SigningKey::generate()` → `SigningKey::from_bytes()` (ed25519-dalek 2.2.0 API)

### Wave 8: Smart File Refactoring (6 files → domain submodules)
- [x] Refactored `canonical.rs` (1,058 lines → `canonical/mod.rs` + `types.rs` + `adapter.rs` + `routing.rs`, largest 376 lines)
- [x] Refactored `mesh_handler.rs` (977 lines → `mesh_handler/mod.rs` + `json.rs` + `udp_discovery.rs` + `tests.rs`, largest ~406 lines)
- [x] Refactored `availability.rs` (944 lines → `availability/types.rs` + `checker.rs` + `tests.rs`)
- [x] Refactored `core/mod.rs` (933 lines → `consolidated_config.rs` + `orchestrator_health.rs` + `consolidated_engine.rs` + `consolidated_tests.rs`)
- [x] Refactored `capability_registration.rs` (928 lines → `config.rs` + `transport.rs` + `payload.rs` + `lifecycle.rs` + `tests.rs`)
- [x] Zero files now exceed 1000 lines

### Wave 9: `#[allow()]` → `#[expect(reason)]` Bulk Migration
- [x] Migrated all remaining bare `#[allow()]` to `#[expect(reason)]` where lint fires
- [x] Reverted to `#[allow(reason)]` where `expect` would be unfulfilled
- [x] Fixed syntax errors in `basic_iot/mod.rs` (`#[allow()dead_code)]` → valid attribute)
- [x] Normalized crate-level lint attributes across all crates

### Wave 10: Unsafe Code Evolution
- [x] Added `parking_lot::Mutex` serialization guard to `songbird-process-env` for caller-level safety
- [x] Changed crate to `#![deny(unsafe_code)]` with per-fn `#[allow(unsafe_code)]` (tightest scope)
- [x] Added unit test `set_remove_roundtrip` for env facade

### Wave 11: Hardcoded Constants → Capability-Based Discovery
- [x] Evolved `find_primals_with_capability` from stub to real capability filter (`SONGBIRD_CAPABILITY_<CAP>_PROVIDERS` + per-primal `*_CAPABILITIES`)
- [x] Removed hardcoded `staging.internal:8080` — all URLs use env → bind → documented fallback const chain
- [x] All ports configurable via `SONGBIRD_*_PORT` env vars with `FALLBACK_*` constants
- [x] Added `canonical_constants_evolved_tests.rs` integration tests

### Wave 12: Production Stub Evolution
- [x] `health_check_all()` → real TCP reachability probes via protocol router (pre-registered `TcpReachabilityHandler`)
- [x] `songbird cli` → minimal interactive REPL with `help`/`exit`/`quit` and subcommand guidance
- [x] Federation join → parses `FederationStatus`/`nodes`/`peers` from response
- [x] Load balancer `RoundRobin`/`WeightedRoundRobin` → stateful `AtomicU64` counter
- [x] Trust `verify_hardware` → proper error with BearDog capability discovery path
- [x] Protocol negotiation ID → monotonic sequence suffix for collision prevention

### Wave 13: Coverage Expansion (+200 new tests)
- [x] `songbird-orchestrator`: consent rules, preferences, request builder, graph validation, health, config, zero-touch, biome, trust ordering + batch integration (63 tests)
- [x] `songbird-config`: network cors, limits, timeouts, ports, gaming + batch integration (16 tests)
- [x] `songbird-universal`: discovery types, capability errors, traits, communication, adapter errors + batch integration (15 tests)
- [x] Plus ~100+ inline module tests across orchestrator/config/universal

### Wave 14: Documentation (29/29 crates `#[warn(missing_docs)]`)
- [x] Enabled `#![warn(missing_docs)]` on all 17 remaining crates
- [x] Documented public API entry points across all crates
- [x] Added module-level docs for all `pub mod` declarations
- [x] Used scoped `#[allow(missing_docs, reason)]` for deep internal modules in progress

### Wave 15: JSON-RPC Gateway Completion (10 semantic methods)
- [x] `compute.route`, `deployment.create`, `deployment.status`, `task.create`, `task.list`
- [x] `consent.check`, `consent.grant`, `registry.register`, `registry.discover`, `protocol.negotiate`
- [x] All methods share handler logic with REST endpoints (no duplication)
- [x] `JsonRpcState` wired to `ComputeApiState`, `DeploymentState`, `ProtocolApiState`, `TaskLifecycleManager`, `ConsentManager`

### Wave 16: Dependency Pruning
- [x] Removed unused `thiserror` from `songbird-tls`
- [x] Removed unused `tower` from `songbird-http-client`
- [x] Verified `kube`/`k8s-openapi`/`bollard` are feature-gated (not in default builds)
- [x] Documented duplicate version alignment opportunities for future pruning

### Wave 17: Coverage Expansion Wave 2 (+56 tests)
- [x] 21 tests in `songbird-http-client` (beardog RPC, TLS finished, HTTPS connection, connection pool)
- [x] 14 tests in `songbird-universal-ipc` (error display, introspection, capability discovery)
- [x] 8 tests in `songbird-discovery` (factory config, serde, config metadata)
- [x] 13 tests in `songbird-lineage-relay` (error display, types serde, connection stats)

---

## Completed (Mar 20, 2026 — Deep Audit Session 1)

### Wave 1: Standards Compliance
- [x] Migrated 122 `#[allow()]` → `#[expect(reason)]` across all 29 crates (wateringHole standard)
- [x] Reverted 23 to `#[allow(reason)]` where lint doesn't fire (correct behavior for unfulfillable expectations)
- [x] Removed 13 stale lint suppressions discovered by `#[expect()]` (e.g. dead_code that wasn't dead)
- [x] Fixed example crate license: `AGPL-3.0` → `AGPL-3.0-only` (SPDX alignment)
- [x] Isolated `MockBearDogProvider` behind `#[cfg(any(test, feature = "test-mocks"))]`
- [x] Added `test-mocks` feature to `songbird-network-federation`

### Wave 2: Safety & Production Hardening
- [x] Removed 3 production `panic!()`/`unreachable!()` — replaced with `Result`-based error returns
  - `sovereign-onion/service.rs`: standalone mode no longer panics
  - `orchestrator/http_server.rs`: `unreachable!()` → `Err(anyhow!(...))`
  - `lineage-relay/relay_server.rs`: `unreachable!()` removed via match restructure
- [x] Added SAFETY documentation to `songbird-process-env` unsafe blocks
- [x] Enhanced `tower` CLI: `tower info`/`tower config` now honor `SONGBIRD_BIND_ADDRESS`

### Wave 3: Architecture & Refactoring
- [x] Refactored `unified_adapter.rs` (956 lines → 5-module tree, largest 243 lines)
- [x] Refactored `http_handler.rs` (949 lines → 8-module tree, largest 166 lines)
- [x] Extracted `src/lib.rs` from binary-only `songbird` crate (testable CLI types)
- [x] Audited top .clone() hotspots — eliminated 6 unnecessary clones:
  - `discovery_bridge.rs`: `String` clones → moves on `AutoAccept`/`Reject` paths
  - `canonical.rs`: `service.clone()` → move semantics, `protocol` clone → borrowed lookup
  - `real_service_discovery.rs`: `.to_string()` in filter → `.any()` comparison
- [x] Feature-gated `infer_capabilities_from_name` behind `#[cfg(any(feature = "k8s", feature = "docker"))]`

### Wave 4: Coverage Expansion (+150 tests total)
- [x] 16 CLI parsing tests (`tests/cli_parsing_tests.rs`) — `main.rs` from 0% to testable
- [x] 27 tests in `songbird-config` (discovery, endpoints, constants, cache TTL)
- [x] 10 tests in `songbird-orchestrator` (availability, components, compute API, registration)
- [x] 30 tests in `songbird-universal` (tarpc, jsonrpc, connection_manager, query, sovereignty)
- [x] 31 tests in `songbird-http-client` (redirect, IPC client, TLS record, beardog RPC)
- [x] Fixed test race conditions in env-var-dependent tests (primal_discovery, constants, hardcoded_elimination)
- [x] 5 tests in `songbird-tls` (crypto, record layer, cert)
- [x] 8 tests in `songbird-discovery` (real_service_discovery, broadcaster, listener, self_knowledge)
- [x] 8 tests in `songbird-types` (canonical adapters, errors, environment, lineage, memory_optimized)
- [x] 7 tests in `songbird-registry` (production_storage, federation, registry core)
- [x] 3 tests in `songbird-stun` (message decoding, attribute roundtrip)
- [x] 25+ tests in `songbird-orchestrator` (capability_registration, core, graph, compute_api, trust, capability_router, process_manager)

### Wave 5: Analysis & Planning
- [x] Complete `ring` dependency elimination analysis:
  - `rcgen` → removable via BearDog cert delegation or pure-Rust PKIX encoder
  - `quinn` → blocked by upstream (quinn-proto defaults to `rustls-ring`, no C-free alternative)
  - Documented concrete elimination steps and effort estimates

### Wave 6: Root Documentation & Debris Cleanup
- [x] Updated README.md with current metrics (v0.3.3, 63.50% coverage, 6,300+ tests)
- [x] Added CHANGELOG v0.3.3 entry with all session work
- [x] Rewrote CONTRIBUTING.md with `#[expect(reason)]` standard, removed stale patterns
- [x] Deleted broken `config/docker/Dockerfile` (gaming bridge CMD, rust:1.75, wrong layout)
- [x] Deleted broken `config/docker/Dockerfile.federation` (no `songbird-federation` binary)
- [x] Deleted broken `docker/Dockerfile.production` (nonexistent `songbird-lib`)
- [x] Deleted broken `docker/docker-compose.production.yml` (referenced deleted Dockerfile)
- [x] Deleted broken `config/docker/docker-compose.production.yml` (gaming bridge, deleted Dockerfile)
- [x] Deleted broken `config/docker/docker-compose.core.yml` (gaming bridge, deleted Dockerfile)
- [x] Deleted orphaned `docker/entrypoint.sh` (was for deleted Dockerfile.production)
- [x] Deleted `config/scripts/production-deployment-demo.sh` (echo-only marketing script)
- [x] Deleted `config/scripts/deploy.sh` (wrong PROJECT_ROOT, builds `songbird-orchestrator`)
- [x] Corrected `#[warn(missing_docs)]` count: 13/29 crates (was reported as 2)
- [x] Created wateringHole handoff for v0.3.3

---

## Pending: BearDog Crypto Integration

BearDog provides pure Rust crypto via runtime capability discovery.
All stubs currently return `CryptoUnavailable`; wiring requires BearDog running.

### Tor Protocol
- [ ] AES-128-CTR encryption roundtrip via BearDog
- [ ] Running digest (SHA-1/SHA3-256) via BearDog for relay cell integrity
- [ ] HMAC-SHA256 for ESTABLISH_INTRO handshake auth
- [ ] ntor handshake (CREATE2/EXTEND2) via BearDog
- [ ] Heuristic relay selection (Phase 2A: intelligent selection based on consensus weights)

### TLS / Sovereign Onion
- [ ] `ed25519_public_from_secret` via BearDog
- [ ] BearDog-generated lineage-tagged certificates (full X.509 chain validation)
- [ ] CertificateVerify BearDog signing
- [ ] Custom TLS extension building via BearDog

### Ring-Free Workspace
- [ ] `rcgen` removal: replace with BearDog-issued certs or pure-Rust PKIX builder (`x509-cert`)
- [ ] Quinn `default-features = false` + selective feature enablement (avoids `rustls-ring`)
- [ ] Track upstream quinn/rustls for ring-free QUIC (quinn-rs/quinn#2253)

---

## Pending: Coverage Expansion (~66.59% → 90% target)

### High-Impact Targets (by missed lines)
| Module | Missed | Coverage |
|--------|--------|----------|
| songbird-orchestrator | ~7,000 | ~56% |
| songbird-config | ~2,600 | ~68% |
| songbird-universal | ~2,200 | ~72% |
| songbird-http-client | ~1,600 | ~65% |
| songbird-universal-ipc | ~1,200 | ~67% |

### Strategy
- Focus on pure logic modules for unit test ROI
- Add `#[cfg(test)] mod tests` to ~400 files that lack inline tests
- Prioritize: cache logic, graph algorithms, capability routing, TLS record parsing
- Target: 5-10 pp coverage gain per deep session

---

## Completed: Standards Compliance

### `#[warn(missing_docs)]` on library crates
- **30/30 crates** have `#[warn(missing_docs)]` enabled and compile clean
- Large internal modules use scoped `#[allow(missing_docs, reason = "...")]` where bulk documentation is in progress
- All public entry points and re-exports are documented

---

## Pending: Platform & Infrastructure

- [ ] Platform NFC backends (Android JNI, iOS CoreNFC, Linux libnfc)
- [ ] Real hardware IGD test (Tower + Pixel 8a)
- [ ] Genesis physical channels: Bluetooth GATT/L2CAP, QR code, SoloKey (FIDO2)
- [ ] iOS XPC transport (requires platform-specific bindings)
- [ ] WASM primal registry + tokio/mio WASM support
- [ ] Android IPC: configurable fallback bind address
- [ ] USB bulk endpoint streaming (currently uses control transfers)
- [ ] DNS SRV integration for capability discovery

---

## Pending: Architectural Evolution

- [ ] Cluster support for anonymous beacon broadcasting
- [ ] TLS handshake v2 module integration into main handshake flow
- [ ] IPC native endpoint lifecycle management

---

## Pending: Dependency Evolution

- [ ] `ring` elimination: see Ring-Free Workspace section above
- [x] `sysinfo` → pure Rust `sys_metrics` module (ecoBin v3.0): Eliminated `sysinfo` + `rayon` + `crossbeam-*` from dependency tree. Replaced with `songbird_types::sys_metrics` reading `/proc/meminfo` and `/sys/block/*/size` directly.
- [x] Removed dead `songbird-observability/src/monitoring/` directory (broken sysinfo 0.29 API, not wired)
- [x] Removed dead `songbird-registry/src/health/` and `scaling/` directories (broken syntax, not wired)
- [ ] Remaining transitive duplicates (syn, hashbrown, getrandom, parking_lot, socket2) require upstream changes

---

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) — Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration
- [ ] Full NAT type detection (requires multiple STUN requests)
- [ ] Tor consensus microdescriptor parsing (ntor_key, version fields)
- [ ] Tor HSDir descriptor upload

---

## Priority Order

1. **BearDog crypto wiring** — Unblocks circuit build + onion encryption (pure Rust via capability discovery)
2. **Coverage expansion** — Target pure-logic modules first (goal: 90%)
3. **Ring-free workspace** — `rcgen` replacement + quinn feature reconfiguration
4. **Deep documentation** — Fill `#[allow(missing_docs)]` internal modules with full doc coverage
5. **Real hardware tests** (Tower + Pixel) — Validates cross-network
6. **Platform backends** — Mobile pairing, iOS, WASM
7. **Dependency pruning** — Reduce ~412 unique deps where possible
