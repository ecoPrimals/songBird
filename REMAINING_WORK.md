# Songbird Remaining Work

**Date**: February 9, 2026  
**Version**: v3.41.0  
**Last Deep Debt Audit**: February 9, 2026

---

## Recent Deep Debt Evolutions (Feb 8-9, 2026)

### Polling Anti-Patterns Eliminated
- ConsentManager: 100ms polling -> `tokio::sync::Notify` (instant wake on approve/deny)
- UnixSocketServer: 10ms polling -> `tokio::sync::Notify` (instant readiness)
- PunchHandler: Simulated punch loop -> real `HolePunchCoordinator::punch_to_peer()`
- BirdSongBroadcaster: Sleep polling -> `tokio::sync::Notify` (instant message wake)
- Coordinator: 1s polling -> event-driven relay request processing

### Environment Variable Pollution Eliminated
- 120+ `env::set_var`/`remove_var` calls removed from tests
- Injectable environment readers (`_with` variants) across 15+ modules
- `CapabilityRegistrationConfig::for_testing()` pattern for test isolation
- `BearDogProvider::with_mode()` for explicit constructor tests

### Dead Code Removed
- 4,130 lines of corrupted dead code in `core/biome/` directory (10 files)
- Unreachable code in `sovereign-onion/keys.rs` fixed

### Stub Implementations Evolved
- `HttpRendezvousClient`: Full HTTP register/lookup with retry logic
- `UdpPeerConnector`: Real UDP hole punching via `tokio::select!`
- `TorHandler`: Full JSON-RPC handler using `CircuitManager` and `Consensus`

---

## Test Coverage Summary

| Crate | Lib Tests | Status |
|-------|-----------|--------|
| songbird-orchestrator | 605 | All pass |
| songbird-universal | 566 | All pass |
| songbird-config | 452 | All pass |
| songbird-types | 264 | All pass |
| songbird-discovery | 235 | All pass |
| songbird-tls | 179 | All pass |
| songbird-universal-ipc | 172 | All pass |
| songbird-http-client | 130 | All pass |
| songbird-primal-coordination | 90 | All pass |
| songbird-tor-protocol | 76 | 75 pass, 1 ignored (BearDog AES) |
| songbird-lineage-relay | 66 | All pass |
| songbird-network-federation | 43 | All pass |
| songbird-observability | 30 | All pass |
| songbird-igd | 28 | All pass |
| songbird-registry | 26 | All pass |
| songbird-nfc | 23 | All pass |
| songbird-bluetooth | 17 | All pass |
| songbird-stun | 12 | All pass |
| songbird-cli | 12 | All pass |
| songbird-onion-relay | 10 | All pass |
| songbird-sovereign-onion | 8 (+ 22 with standalone) | All pass |
| songbird-quic | 6 | All pass |
| **TOTAL** | **3,504+** | **All pass** |

---

## Deep Debt Audit (Feb 9, 2026)

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

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) - Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration

---

## Priority Order

1. **BearDog Tor crypto** - Unblocks circuit build + onion encryption
2. **Real hardware IGD test** (Tower + Pixel) - Validates cross-network
3. **Android IPC bind address** - Configurable for cross-process
4. **Platform NFC backends** - Mobile pairing
