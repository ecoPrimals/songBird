# Songbird Security

**Last Updated**: April 27, 2026
**Status**: Active Development (v0.2.1)

---

## Security Posture

| Metric | Status |
|--------|--------|
| **Pure Rust** | ✅ Application code — `quinn`/`ring` eliminated from QUIC path (native pure-Rust QUIC engine with security provider crypto delegation); `ring-crypto` feature removed (Wave 135, SB-02 resolved); `ring` in Cargo.lock is uncompiled lockfile artifact (banned in `deny.toml`); `sled` removed (Wave 135, SB-03 resolved); all default-build deps are pure Rust |
| **Unsafe Code** | ✅ `forbid(unsafe_code)` across all 30 crates; zero `unsafe` blocks (`songbird-process-env` evolved to in-memory overlay) |
| **Cast Safety** | ✅ `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` denied workspace-wide |
| **Panic Paths** | ✅ `unwrap_used`/`expect_used` warn workspace-wide |
| **JSON-RPC 2.0** | ✅ Strict version validation, notification suppression, serialization fallbacks |
| **Capability Discovery** | ✅ Runtime discovery, no hardcoded primal names or ports |

---

## Implemented Protections

### Code Safety

- **Zero unsafe code** — `forbid(unsafe_code)` across all 30 crates; `songbird-process-env` evolved to in-memory overlay pattern (zero calls to `std::env::set_var`/`remove_var`)
- **Cast discipline** — All four cast lints denied at workspace level; justified sites use `#[expect(reason)]`
- **Result-based errors** — All fallible operations return `Result<T, E>`
- **Strict linting** — `clippy::pedantic` + `clippy::nursery` workspace-wide

### Network Safety

- **BTSP Phase 2** — Incoming UDS connections require 4-step BTSP handshake when `FAMILY_ID` is set; crypto delegated to security provider via `btsp.session.create/verify/negotiate`; post-handshake framing uses length-prefixed (4-byte BE) frames; development mode (no `FAMILY_ID`) uses newline-delimited JSON-RPC
- **TLS** — Pure Rust TLS 1.3 implementation
- **QUIC** — Pure Rust QUIC transport
- **Sovereign Onion** — Privacy-preserving routing
- **Dark Forest gating** — TCP requests subject to trust verification

### Operational Safety

- **No hardcoded secrets** — Environment-first configuration
- **Capability-based routing** — Runtime service discovery, no hardcoded endpoints
- **Structured logging** — `tracing` framework for observability
- **JSON-RPC strict mode** — Version validation, notification suppression, safe serialization fallbacks

---

## Dependency Auditing

```bash
cargo audit
cargo deny check advisories
cargo deny check                  # Full check: bans, licenses, advisories, sources (enforced in CI since Wave 134)
```

### Known C Dependencies

- **`ring`** — `ring-crypto` feature **removed** (Wave 135); `rustls_rustcrypto` is the sole TLS bootstrap provider; `ring` remains only as unactivated optional dep of `rustls` in lockfile and via optional `k8s` feature; `songbird-quic` is fully ring-free (native QUIC engine with security provider crypto delegation)
- **System metrics** — Pure Rust `/proc` and `/sys` readers (replaced `sysinfo`)

---

## Reporting Security Issues

Report security vulnerabilities through secure channels to the ecoPrimals development team.

**Do not** open public issues for security vulnerabilities.

---

## License

AGPL-3.0-or-later — Ensures full transparency of security implementations.
