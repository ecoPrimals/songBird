# Songbird Remaining Work

**Date**: February 11, 2026  
**Version**: v0.2.2  
**Last Deep Debt Audit**: February 11, 2026

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 8,515 passing, 0 failed |
| **Line Coverage** | 60.62% (goal: 90%) |
| **Build** | Zero errors |
| **Clippy** | Zero errors |
| **Format** | Clean |
| **Docs** | Clean |
| **Files >1000 lines** | 0 |
| **Unsafe blocks** | 0 |
| **Production `todo!()`** | 0 |
| **C dependencies** | 0 |

---

## Deep Debt Audit (Feb 11, 2026)

| Principle | Status | Evidence |
|-----------|--------|----------|
| Zero `unsafe` | S+ | `#![forbid(unsafe_code)]` across all crates |
| Pure Rust | S+ | SHA3-256, SSDP, SOAP, NAT-PMP, base64, hex all from scratch |
| Zero production stubs | S+ | NFC → BearDog IPC, HTTP rendezvous, UDP punch all complete |
| Zero `todo!()` in production | S+ | Only in `#[cfg(test)]` functions |
| Runtime discovery | S+ | All socket paths: env → XDG → fallback |
| Event-driven architecture | S+ | Zero polling anti-patterns in production code |
| Concurrent-safe testing | S+ | Injectable env readers, no `env::set_var` in tests |
| Self-knowledge only | S+ | Introspection describes only Songbird |

---

## Recent Evolutions (Feb 2026)

### Capability-First Socket Discovery (Feb 11, 2026) ✅
All socket discovery evolved from primal-specific to capability-first:
- `songbird-lineage-relay/src/beardog.rs` — `security.sock` before `beardog.sock`
- `songbird-quic/src/config.rs` — `crypto.sock` before `beardog.sock`
- `songbird-nfc/src/config.rs` — `security.sock` before `beardog.sock`
- `songbird-nfc/src/genesis.rs` — Full capability-first refactor
- `songbird-tls/src/socket_discovery.rs` — `CRYPTO_PROVIDER_SOCKET`, `SECURITY_PROVIDER_SOCKET` first
- `songbird-config/src/discovery/mdns.rs` — Fixed mdns-sd API compatibility
- `songbird-universal/src/discovery/backends/network.rs` — hickory-resolver migration

### Relay-Assisted Coordinated Punch
- `stun.probe_port_pattern` — Port pattern probing for NAT characterization
- `punch.coordinate` — Relay-assisted coordinated hole punching
- `HolePunchCoordinator` wired to punch handler
- `nat0` hardcoding → `env_config::family_id()` (default: "default")

### Capability-First Discovery
- `PrimalChecks` → `HashMap<String, PrimalCheck>` (dynamic, not hardcoded fields)
- Socket patterns: capability terms first, primal names as hints
- `discover_crypto_socket()` / `discover_security_socket()` public APIs

### Production Mocks Isolated
- `songbird-lineage-relay/src/beardog.rs` gated with `#[cfg(any(test, feature = "test-utils"))]`
- `test-utils` feature for integration test access

### Large File Refactoring
- `main.rs`: 886 → 141 lines (doctor/server/config extracted to `commands/`)
- `service.rs`: 946 → 825 lines (builder pattern, inlined wrappers)
- `beardog_crypto_client.rs`: 906 → 554 lines (generic RPC helper)

### Coverage Tests Added
- `canonical_adapter_coverage_tests.rs` (32 tests)
- `tower_atomic_coverage_tests.rs` (23 tests)
- `config_types_coverage_tests.rs` (28 tests)

---

## Pending: BearDog Crypto Integration

### Tor Protocol (blocked on BearDog session)
- [ ] AES-128-CTR encryption roundtrip via BearDog
- [ ] Running digest (SHA-1/SHA3-256) via BearDog for relay cell integrity
- [ ] HMAC-SHA256 for ESTABLISH_INTRO handshake auth
- [ ] ntor handshake (CREATE2/EXTEND2) via BearDog

### Other
- [ ] Sovereign Onion: `ed25519_public_from_secret` via BearDog
- [ ] TLS: BearDog-generated lineage-tagged certificates

---

## Pending: Platform & Infrastructure

- [ ] Android IPC: configurable fallback bind address
- [ ] Platform NFC backends (Android JNI, iOS CoreNFC, Linux libnfc)
- [ ] Real hardware IGD test (Tower + Pixel 8a)

---

## Pending: Coverage

Current: **60.62%** | Goal: **90%**

### High-Impact Targets (by missed lines)
| Module | Missed | Coverage |
|--------|--------|----------|
| songbird-orchestrator | 7,200+ | 55% |
| songbird-config | 2,800+ | 66% |
| songbird-universal | 2,400+ | 70% |
| songbird-http-client | 1,800+ | 63% |

Many low-coverage modules require integration setup (running servers, crypto providers).
Focus on pure logic modules for unit test ROI.

---

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) — Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration

---

## Priority Order

1. **BearDog Tor crypto** — Unblocks circuit build + onion encryption
2. **Coverage expansion** — Target pure-logic modules first
3. **Real hardware IGD test** (Tower + Pixel) — Validates cross-network
4. **Android IPC bind address** — Configurable for cross-process
5. **Platform NFC backends** — Mobile pairing
