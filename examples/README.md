# Songbird Examples

Examples demonstrating the Songbird Universal Orchestrator.

## Current Examples (Pure Rust)

### IPC Clients

| Example | Description |
|---------|-------------|
| `ipc_client_simple.rs` | Basic JSON-RPC IPC client connecting to Songbird |
| `ipc_client_discovery.rs` | Service discovery via IPC |
| `ipc_client_primal.rs` | Primal-to-primal communication patterns |

```bash
cargo run --example ipc_client_simple
cargo run --example ipc_client_discovery
cargo run --example ipc_client_primal
```

### Client Libraries

- `clients/rust/` — Rust tarpc client (standalone crate)
- `clients/python/` — Python JSON-RPC + WebSocket clients
- `clients/javascript/` — JavaScript JSON-RPC + WebSocket clients

### Shell

- `jsonrpc_client.sh` — curl-based JSON-RPC 2.0 examples

### Configuration

- `config/` — Example TOML configurations for capability discovery, ecosystem integration, and BearDog delegation

## Archived

- `legacy/` — Pre-ecoBin v2.0 examples (uses reqwest; historical reference only)
- `future/` — Experimental/proposed API patterns (may not compile against current workspace)
