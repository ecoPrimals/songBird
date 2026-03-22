# songbird-crypto-provider

Shared crypto provider for Songbird crates — Neural API + Direct BearDog routing.

Provides a unified interface for delegating cryptographic operations to BearDog
via JSON-RPC IPC. Supports two routing modes:

- **Neural API**: Route through the Neural API graph for capability-based dispatch
- **Direct**: Connect directly to a BearDog socket for low-latency operations

## Socket Discovery

Socket paths are resolved at runtime via capability-based discovery:

1. Environment variables (`NEURAL_API_SOCKET`, `CRYPTO_PROVIDER_SOCKET`)
2. XDG runtime directory (`$XDG_RUNTIME_DIR/biomeos/{socket}`)
3. Family-scoped sockets (`beardog-{family_id}.sock`)

No hardcoded paths — primal self-knowledge only.

## License

AGPL-3.0-only
