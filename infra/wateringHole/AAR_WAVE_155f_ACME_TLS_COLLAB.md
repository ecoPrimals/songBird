# AAR — Wave 155f ACME + rustls-rustcrypto Elimination

**Primal**: songBird  
**Date**: July 28, 2026  
**Gate**: eastGate → origin/main (Forgejo)  
**Wave**: 155f  
**Upstream dependency**: bearDog (ACME client completion)

---

## Delivered (1 commit)

| Commit | Type | Summary |
|--------|------|---------|
| `305f5be` | feat | ACME HTTP-01 challenge responder — Phase 1 of `rustls-rustcrypto` elimination |

---

## Context

`rustls-rustcrypto 0.0.2-alpha` is the sole TLS crypto provider for songBird's outbound HTTPS (drawbridge external proxy) and CLI bootstrap. The crate is explicitly marked "DO NOT USE IN PRODUCTION" by its maintainers, has no stable release, and pulls transitive RustCrypto dupes that conflict with workspace stable versions.

bearDog provides crypto delegation via Tower Atomic (`crypto.*` JSON-RPC over UDS). If bearDog can provision ACME certs (Let's Encrypt), it can also provide TLS cert signing — making `rustls-rustcrypto` redundant.

**Blocker**: bearDog's ACME client needs songBird's HTTP listener to serve HTTP-01 challenge responses at `/.well-known/acme-challenge/{token}`.

---

## What Shipped

### ACME HTTP-01 Challenge Responder

| Component | File | Change |
|-----------|------|--------|
| Challenge store | `drawbridge.rs` | `LazyLock<RwLock<HashMap<String, String>>>` for token→authorization pairs |
| Challenge serving | `drawbridge.rs` | `/.well-known/acme-challenge/{token}` path handler (pre-auth, no credentials required) |
| Registration API | `tower.rs` | `handle_acme_challenge_ready(params)` — validates token+authorization, registers |
| Cleanup API | `tower.rs` | `handle_acme_challenge_cleanup(params)` — removes token after validation |
| Type system | `domain_methods.rs` | `AcmeMethod::ChallengeReady`, `AcmeMethod::ChallengeCleanup` |
| Wire format | `mod.rs` | `acme.challenge_ready`, `acme.challenge_cleanup` ↔ enum roundtrip |
| Dispatch | `dispatch/mod.rs` | Routed through `IpcServiceDispatch` |
| Tests | `drawbridge_tests.rs`, `tests.rs` | 3 unit tests (register+serve, 404, cleanup) + roundtrip |

### Flow (bearDog perspective)

```
bearDog → songBird: acme.challenge_ready {token: "abc", authorization: "abc.thumbprint"}
ACME CA → songBird drawbridge: GET /.well-known/acme-challenge/abc → 200 "abc.thumbprint"
bearDog → songBird: acme.challenge_cleanup {token: "abc"}
```

---

## Elimination Roadmap

| Phase | Owner | Status | Action |
|-------|-------|--------|--------|
| 1 | songBird | **DONE** | ACME challenge responder on drawbridge |
| 2 | bearDog | BLOCKED | ACME client uses songBird's challenge responder to provision certs |
| 3 | songBird | After P2 | `songbird-tls` client mode replaces `outbound_tls_connector()` — crypto delegated to bearDog |
| 4 | songBird | After P3 | Remove `rustls-rustcrypto` from `songbird-universal-ipc/Cargo.toml` + `songbird-cli/Cargo.toml` |

### What bearDog Needs to Ship (Phase 2)

1. ACME client that calls `acme.challenge_ready` on songBird via UDS JSON-RPC
2. Let's Encrypt account registration + HTTP-01 order flow
3. Certificate storage + renewal lifecycle
4. `crypto.tls.connect` delegation endpoint (songBird calls this instead of local rustls)
5. Call `acme.challenge_cleanup` after validation success/failure

### What Gets Cut (Phase 4)

From `songbird-universal-ipc/Cargo.toml`:
- `rustls-rustcrypto = "0.0.2-alpha"`
- Potentially `rustls`, `tokio-rustls`, `rustls-native-certs` (if all TLS delegated)

From `songbird-cli/Cargo.toml`:
- `rustls-rustcrypto = "0.0.2-alpha"`

From CLI tower command:
- `rustls_rustcrypto::provider().install_default().ok();`

**Net dep reduction**: alpha crate + transitive `rustls-webpki 0.102` ghost lock + RustCrypto dupes

---

## Key Decisions

### Challenge Store: In-Memory vs Persistent

**Chose**: In-memory `LazyLock<RwLock<HashMap>>`.

**Rationale**: ACME challenges are ephemeral (60s–5min lifetime). Persistence adds complexity with zero benefit. If songBird restarts mid-challenge, bearDog retries — standard ACME client behavior.

### Challenge Path: Pre-Auth

**Chose**: ACME challenge path is served before auth gate evaluation.

**Rationale**: ACME CAs (Let's Encrypt) validate without credentials. The path namespace (`/.well-known/acme-challenge/`) is globally standardized (RFC 8555 §8.3) and collision-free with drawbridge route prefixes.

### Handlers in tower.rs (not separate acme.rs)

**Chose**: ACME handlers live in `tower.rs` alongside Tower Atomic health facade.

**Rationale**: ACME is a Tower Atomic collaboration feature (bearDog ↔ songBird). Keeping them co-located with tower handlers reinforces the architectural boundary. If ACME grows beyond 2 methods, extract to `acme.rs`.

---

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| bearDog doesn't ship ACME client | Medium | songBird continues functioning with `rustls-rustcrypto`; no regression |
| Challenge store memory leak (tokens never cleaned) | Low | bearDog calls `challenge_cleanup`; could add TTL sweep if needed |
| Concurrent challenge races | Low | RwLock provides safe concurrent access; ACME tokens are unique per order |

---

## Validation

- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: PASS (14,835+ tests, 0 failures)
- `cargo fmt --check`: PASS
- New tests: 6 (3 drawbridge ACME + 1 roundtrip + 2 path derivation assertions)

---

## For Overwatch

This is a **coordination checkpoint**. songBird's Phase 1 is complete. The ball is now with bearDog to ship Phase 2 (ACME client). Once bearDog confirms ACME cert provisioning works end-to-end on a live gate, songBird can proceed with Phase 3 (songbird-tls client mode) and Phase 4 (dep removal).

**Live validation target**: Any gate with both songBird + bearDog running (house1, northGate, blueGate). bearDog needs:
- songBird UDS reachable (`songbird.sock`)
- Public HTTP reachable for ACME CA (port 80 or Caddy forwarding `/.well-known/`)
- A domain pointing at the gate for cert provisioning
