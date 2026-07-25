# songBird Crypto Composition Analysis

**Date**: July 24, 2026  
**Wave**: 150x  
**Finding**: Crypto Composition Divergence (P1)

## Problem

songBird embeds crypto directly in 10+ crates (`sha2`, `hmac`, `ed25519-dalek`,
`chacha20poly1305`, `blake3`, `aes-gcm`, `hkdf`) instead of routing through
bearDog's `crypto.*` capabilities via UDS. This bypasses the composition model
and prevents chimera validation.

## Existing Delegation Infrastructure

songBird already has TWO delegation paths to bearDog:

1. **`SecurityRpcClient`** (`songbird-http-client/src/security_rpc_client/`):
   - `crypto.sha256`, `crypto.sha384` — hash delegation
   - `crypto.hkdf_expand_label` — key derivation delegation
   - `crypto.hmac_sha256` — HMAC delegation
   - `crypto.sign.ed25519`, `crypto.verify.ed25519` — signing delegation
   - `btsp.server.export_keys` — session key export
   - AEAD encrypt/decrypt — `aes128_gcm_encrypt`, `aes256_gcm_encrypt`

2. **`CryptoProvider`** (`songbird-crypto-provider`):
   - Generic `call(method, params)` JSON-RPC to bearDog socket
   - Used in `pure_rust_jwt.rs` (dual-path: local fallback + provider)

## Classification

### MUST STAY LOCAL (transport hot path — chimera candidate)

These run per-frame/per-packet at wire speed. IPC would be 100-1000x too slow.
Mark as **chimera candidates** for future shared-library extraction.

| Crate | File | Crypto | Reason |
|-------|------|--------|--------|
| `songbird-orchestrator` | `ipc/btsp_phase3.rs` | ChaCha20-Poly1305 + HKDF | Per-frame AEAD (16 MiB frames at wire speed) |
| `songbird-http-client` | `tls/handshake_*/transcript.rs` | SHA-256 | TLS transcript hash (per-handshake-message) |
| `songbird-stun` | `message/attributes.rs` | HMAC-SHA1 | STUN MESSAGE-INTEGRITY (RFC 5389 wire format) |
| `songbird-sovereign-onion` | `crypto.rs` | ChaCha20-Poly1305 | Onion layer encryption (per-packet) |
| `songbird-sovereign-onion` | `keys.rs`, `address.rs` | Ed25519 | .onion address derivation + key management |

### SHOULD DELEGATE (one-time / session-setup / not hot-path)

These run infrequently. IPC cost (~0.5ms per call) is negligible vs operation frequency.

| Crate | File | Crypto | Delegation Path |
|-------|------|--------|----------------|
| `songbird-orchestrator` | `access_control/pure_rust_jwt.rs` | HMAC-SHA256 | `CryptoProvider` (ALREADY dual-path) |
| `songbird-orchestrator` | `task_lifecycle/checkpoint.rs` | SHA-256 | `SecurityRpcClient::sha256` |
| `songbird-discovery` | `crypto_helpers.rs` | SHA-256 | `SecurityRpcClient::sha256` |
| `songbird-discovery` | `dark_forest_beacon.rs` | blake3 | `crypto.hash.blake3` — **DONE** (Wave 150x: `new_with_crypto` + `hash_capabilities_async`) |
| `songbird-network-federation` | `crypto_helpers.rs` | HMAC-SHA256 | `SecurityRpcClient::hmac_sha256` |
| `songbird-genesis` | `security_capability_client.rs` | SHA-256 | Already named for delegation! |

### TEST-ONLY (no production impact)

| Crate | File | Crypto | Status |
|-------|------|--------|--------|
| `songbird-tls` | `cert/test_cert_gen.rs` | Ed25519 | Test cert gen — acceptable |
| `songbird-quic` | `cert_gen.rs` | Ed25519 | Behind `local-certs` feature — acceptable |
| `songbird-tls` | `key_schedule/key_schedule_tests.rs` | HMAC + SHA-256 | Test-only verification |
| `songbird-network-federation` | `btsp/local.rs` | AES-GCM | Behind `local-btsp` feature — test-only |
| `songbird-discovery` | `birdsong/mocks.rs` | blake3 | Test mock — cfg-gated |

### ALREADY CORRECTLY DELEGATING

| Crate | Pattern |
|-------|---------|
| `songbird-orchestrator` | `btsp_phase3.rs` → `SecurityRpcClient::btsp_export_keys` |
| `songbird-http-client` | TLS record layer → `SecurityRpcClient` AEAD |
| `songbird-universal-ipc` | `ipc_registry.rs` → `crypto.sign.ed25519` for registration |

## Migration Path

### Phase 1: Composition (current wave)
- Document seams (this file) ✅
- Route `pure_rust_jwt.rs` through bearDog exclusively (remove local fallback when provider available)
- Route `checkpoint.rs` SHA-256 through `SecurityRpcClient`
- Route `network-federation/crypto_helpers.rs` HMAC through delegation
- Route `discovery/crypto_helpers.rs` SHA-256 through delegation
- Route `discovery/dark_forest_beacon.rs` blake3 through bearDog ✅ (`new_with_crypto` + `hash_capabilities_async`)
- `blake3` dep made optional behind `local-crypto-fallback` feature ✅
- Remove direct `sha2`/`hmac` deps from crates that now delegate

### Phase 2: Measure IPC Cost
- Benchmark each delegation seam (target: <1ms per call)
- Identify seams where IPC cost is unacceptable
- Document findings for chimera extraction

### Phase 3: Chimera (future)
- Extract transport hot-path crypto into shared library (`.so`)
- bearDog + songBird link same chimera lib (no IPC for hot path)
- Composition model preserved for session setup / one-time operations

## Dependency Removal Target

After Phase 1 delegation, these deps become removable from production crates:

| Dep | Remove From | Keep In |
|-----|-------------|---------|
| `sha2` | `orchestrator`, `discovery`, `genesis`, `federation` | `http-client` (TLS transcript), `sovereign-onion` |
| `hmac` | `orchestrator`, `federation` | `stun` (wire format), `tls` |
| `blake3` | `discovery` | ~~if bearDog adds `crypto.hash.blake3`~~ **DONE** — delegating via `blake3_hash()` | — |
| `chacha20poly1305` | — | `orchestrator` (BTSP hot path), `sovereign-onion` |
| `hkdf` | — | `orchestrator` (BTSP session derivation) |
| `ed25519-dalek` | — | `sovereign-onion`, `tls` (test), `quic` (test feature) |
