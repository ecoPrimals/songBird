# songbird-crypto-provider

Shared crypto provider for Songbird crates — Neural API + direct security provider routing.

Provides a unified interface for delegating cryptographic operations to the security provider
via JSON-RPC IPC. Supports two routing modes:

- **Neural API**: Route through the Neural API graph for capability-based dispatch
- **Direct**: Connect directly to a security provider socket for low-latency operations

## Socket discovery

Socket paths are resolved at runtime via capability-based discovery:

1. Environment variables (`SECURITY_PROVIDER_SOCKET`, `CRYPTO_PROVIDER_SOCKET`; legacy: `BEARDOG_SOCKET`)
2. XDG runtime directory: `$XDG_RUNTIME_DIR/biomeos/security.sock`, then `crypto.sock` (and family-scoped names)
3. Temp fallback under `biomeos/` (see `discover_security_socket` in this crate)

No hardcoded paths — primal self-knowledge only.

## License

AGPL-3.0-only
