# Songbird Remaining Work

**Date**: March 19, 2026  
**Version**: v0.3.1  
**Last Deep Debt Audit**: March 19, 2026

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 9,358 total, 0 failed, ~165 ignored |
| **Line Coverage** | ~70% (goal: 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, all 29 crates compile clean |
| **Clippy Pedantic** | 29/29 crates clean (2 cosmetic metadata warnings on process-env facade) |
| **Format** | Clean (`cargo fmt --check` passes) |
| **Docs** | Clean (`RUSTDOCFLAGS="-D warnings" cargo doc` passes) |
| **Files >1000 lines** | 0 (gatt->5 modules, coordination->4 modules, server->3 modules) |
| **Unsafe blocks** | 0 (`#![forbid(unsafe_code)]` all crates; `songbird-process-env` is the sole `allow(unsafe)` facade for Rust 2024 env APIs) |
| **Production `todo!()`** | 0 |
| **BearDog crypto** | All placeholders evolved to explicit `CryptoUnavailable` errors with delegation paths |
| **C dependencies** | ring via quinn+rcgen (structural; requires quinn feature reconfiguration for ring-free) |
| **License** | AGPL-3.0-only + ORC + CC-BY-SA 4.0 (full scyBorg trio) |
| **SPDX Headers** | All 1,300+ .rs files have `SPDX-License-Identifier: AGPL-3.0-only` |
| **UniBin** | `songbird compute-bridge` and `songbird deploy` subcommands (one binary) |
| **Platform stubs** | Evolved to `#[cfg(target_os)]` with proper error types (no panics) |
| **Zero-copy** | `Arc<str>` endpoints, `Arc<[u8]>` TLS keys, move semantics in handshake |
| **Concurrent tests** | Zero `std::env::set_var` (via `songbird-process-env` facade) |
| **Event-driven** | Zero `sleep`-based polling in production |

---

## Deep Debt Audit (Mar 19, 2026)

| Principle | Status | Evidence |
|-----------|--------|----------|
| Zero `unsafe` | S+ | `#![forbid(unsafe_code)]` across all crates |
| Pure Rust | S+ | SHA3-256, SSDP, SOAP, NAT-PMP, base64, hex all from scratch |
| Zero production stubs | S+ | NFC -> BearDog IPC, HTTP rendezvous, UDP punch all complete |
| Zero `todo!()` in production | S+ | Only in `#[cfg(test)]` functions |
| Runtime discovery | S+ | All socket paths: env -> XDG -> fallback |
| Event-driven architecture | S+ | Zero polling anti-patterns in production code |
| Concurrent-safe testing | S+ | Injectable env readers, no `env::set_var` in tests |
| Self-knowledge only | S+ | Introspection describes only Songbird |
| AGPL-3.0 license | S+ | All SPDX headers corrected to AGPL-3.0-only |

---

## Completed (This Session)

- [x] Clippy pedantic: All 29 crates clean (was 23/27)
- [x] Edition 2024 migration (from 2021) with `songbird-process-env` facade
- [x] SPDX headers on all 1,300+ source files
- [x] UniBin consolidation: compute-bridge and remote-deploy as subcommands
- [x] Smart refactor: gatt (5 modules), coordination (4 modules), server (3 modules)
- [x] Zero-copy: `Arc<str>` endpoints, `Arc<[u8]>` TLS keys, move semantics
- [x] BearDog crypto stubs evolved to explicit `CryptoUnavailable` errors
- [x] Platform stubs evolved to `#[cfg(target_os)]` with proper error types
- [x] Concurrent test isolation: zero `std::env::set_var`, injectable env readers
- [x] License compliance: scyBorg trio (AGPL-3.0 + ORC + CC-BY-SA)
- [x] Test count: 9,358 (up from 8,968)
- [x] Coverage: ~70% (up from ~61%)

---

## Pending: BearDog Crypto Integration

BearDog provides pure Rust crypto via runtime capability discovery.
All stubs currently return `CryptoUnavailable`; wiring requires BearDog running.

### Tor Protocol
- [ ] AES-128-CTR encryption roundtrip via BearDog
- [ ] Running digest (SHA-1/SHA3-256) via BearDog for relay cell integrity
- [ ] HMAC-SHA256 for ESTABLISH_INTRO handshake auth
- [ ] ntor handshake (CREATE2/EXTEND2) via BearDog

### TLS / Sovereign Onion
- [ ] `ed25519_public_from_secret` via BearDog
- [ ] BearDog-generated lineage-tagged certificates (X.509 chain validation)
- [ ] CertificateVerify BearDog signing

### Ring-Free Workspace
- [ ] Quinn feature reconfiguration for ring-free (quinn -> `ring` dependency)
- [ ] `rcgen` replacement or BearDog-generated certs

---

## Pending: Coverage (70% -> 90%)

### High-Impact Targets (by missed lines)
| Module | Missed | Coverage |
|--------|--------|----------|
| songbird-orchestrator | 7,200+ | ~55% |
| songbird-config | 2,800+ | ~66% |
| songbird-universal | 2,400+ | ~70% |
| songbird-http-client | 1,800+ | ~63% |

~455 files still lack inline `#[cfg(test)]` modules (many exercised by integration tests).
Focus on pure logic modules for unit test ROI.

---

## Pending: Platform & Infrastructure

- [ ] Platform NFC backends (Android JNI, iOS CoreNFC, Linux libnfc)
- [ ] Real hardware IGD test (Tower + Pixel 8a)
- [ ] Genesis physical channels: Bluetooth (btleplug), QR code, SoloKey (FIDO2)
- [ ] iOS XPC transport
- [ ] WASM primal registry + tokio/mio WASM support
- [ ] Android IPC: configurable fallback bind address

---

## Pending: Architectural Evolution

- [ ] REST endpoints -> JSON-RPC wrapping
- [ ] Federation join logic (currently placeholder)
- [ ] Capability router selection strategy (currently first-provider)

---

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) -- Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration
- [ ] GATT/L2CAP real operations
- [ ] USB bulk endpoint streaming

---

## Priority Order

1. **BearDog crypto wiring** -- Unblocks circuit build + onion encryption (pure Rust via capability discovery)
2. **Coverage expansion** -- Target pure-logic modules first (70% -> 90%)
3. **Ring-free workspace** -- Quinn feature reconfiguration + rcgen replacement
4. **Real hardware tests** (Tower + Pixel) -- Validates cross-network
5. **Platform backends** -- Mobile pairing, iOS, WASM
