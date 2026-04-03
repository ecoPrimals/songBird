# Songbird Security

**Last Updated**: April 3, 2026
**Status**: Active Development (v0.2.1)

---

## Security Posture

| Metric | Status |
|--------|--------|
| **Pure Rust** | ✅ Application code — `quinn`/`rustls`/`ring` fully eliminated from QUIC; `ring-crypto` opt-in feature gate on CLI only |
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
```

### Known C Dependencies

- **`ring`** — Opt-in via `ring-crypto` feature gate on `songbird-cli` only (not enabled by default); `songbird-quic` is fully ring-free (native QUIC engine with security provider crypto delegation)
- **System metrics** — Pure Rust `/proc` and `/sys` readers (replaced `sysinfo`)

---

## Reporting Security Issues

Report security vulnerabilities through secure channels to the ecoPrimals development team.

**Do not** open public issues for security vulnerabilities.

---

## License

AGPL-3.0-only — Ensures full transparency of security implementations.
