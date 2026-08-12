# Transport Endpoint Standard

**Status**: Sourdough Candidate (Wave 91)  
**Origin**: songBird (southGate)  
**Wire Version**: 1.0  
**Canonical Type**: `songbird_types::TransportEndpoint`  

---

## Purpose

`TransportEndpoint` is the ecosystem-standard wire type for describing how to
reach a resolved service. It eliminates URI string parsing from consumer code
and enables transport-agnostic capability routing.

Every `ipc.resolve` and `capability.resolve` response includes an `endpoint`
field containing this type. Consumers match on the variant to select connection
strategy without parsing or string manipulation.

---

## Wire Format

Tagged JSON enum via `"transport"` discriminator:

```json
{ "transport": "uds", "path": "/run/user/1000/biomeos/beardog.sock" }
{ "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
{ "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
```

### Variants

| Transport | Fields | Use Case |
|-----------|--------|----------|
| `uds` | `path: String` | Same-host inter-primal (fastest) |
| `tcp` | `host: String, port: u16` | Cross-host or container networking |
| `mesh_relay` | `peer_id: String, capability: String` | Cross-gate via Songbird mesh |

### Ordering Semantics

Variants are ordered by locality (local → network → relay). When multiple
resolution paths exist, consumers SHOULD prefer earlier variants for lower
latency.

---

## Consumer Adoption Pattern

### Minimal (deserialize only — any primal)

Add `songbird-types` (or equivalent) with serde support:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    #[serde(rename = "uds")]
    Uds { path: String },
    #[serde(rename = "tcp")]
    Tcp { host: String, port: u16 },
    #[serde(rename = "mesh_relay")]
    MeshRelay { peer_id: String, capability: String },
}
```

Or copy the enum definition — it's 15 lines with no dependencies beyond serde.

### Full (with helpers — Songbird ecosystem)

```rust
use songbird_types::TransportEndpoint;

let endpoint: TransportEndpoint = serde_json::from_value(response["endpoint"].clone())?;

// Query methods
endpoint.is_local()       // true for UDS and localhost TCP
endpoint.is_network()     // true for remote TCP and mesh
endpoint.is_relayed()     // true only for mesh_relay
endpoint.transport_name() // "uds", "tcp", or "mesh_relay"

// Accessors
endpoint.uds_path()       // Option<&str>
endpoint.tcp_addr()       // Option<(&str, u16)>
endpoint.mesh_peer()      // Option<(&str, &str)>

// Display
endpoint.display_uri()    // "unix:///path", "tcp://host:port", "mesh://peer/cap"
format!("{endpoint}")     // same as display_uri()

// Constructors
TransportEndpoint::uds("/run/membrane/beardog.sock")
TransportEndpoint::tcp("192.168.1.144", 7700)
TransportEndpoint::mesh_relay("strand-gate", "security")
```

---

## Resolution Flow

```
Consumer                    Songbird                     Provider
   |                           |                            |
   |-- ipc.resolve("crypto") ->|                            |
   |                           |-- lookup registry -------->|
   |                           |<-- socket path, caps ------|
   |<-- ResolveResult --------|                            |
   |    { endpoint: { transport: "uds", path: "..." } }    |
   |                           |                            |
   |-- connect(endpoint) ----->| (if mesh_relay)            |
   |                           |-- relay via mesh --------->|
```

---

## Backward Compatibility

The `endpoint` field is **additive**. Existing `socket`, `native_endpoint`,
and `virtual_endpoint` fields remain for consumers that haven't adopted the
structured type. New consumers SHOULD use `endpoint` exclusively.

---

## Extension Points (Future)

| Variant | Fields | Purpose |
|---------|--------|---------|
| `quic` | `host, port, alpn` | QUIC/HTTP3 transport |
| `wasm` | `module_id, function` | In-process WASM component |
| `onion` | `address, port` | Tor hidden service |
| `bluetooth` | `device_id, service_uuid` | BLE GATT transport |

New variants are backward-compatible — unknown `"transport"` values
deserialize as `serde_json::Value` for forward-compatible consumers.

---

## Primal Adoption Checklist

For primals consuming `ipc.resolve` responses:

- [ ] Deserialize `response["endpoint"]` into `TransportEndpoint`
- [ ] Match on variant to select connection strategy
- [ ] Remove any URI string parsing (`split("://")`, regex, etc.)
- [ ] Log using `endpoint.display_uri()` for diagnostics
- [ ] Prefer `endpoint.is_local()` for fast-path optimization

For primals providing services:

- [ ] Include `endpoint` in registration metadata
- [ ] Populate from known bind address at registration time
- [ ] Support TCP variant for cross-host deployments

---

## Reference Implementation

- **Canonical type**: `crates/songbird-types/src/transport.rs`
- **IPC wire integration**: `crates/songbird-universal-ipc/src/service_types.rs`
- **Registry population**: `crates/songbird-universal-ipc/src/service/ipc_registry/mod.rs`
- **Tests**: 18 unit tests in `songbird-types`, 10+ integration tests in `songbird-universal-ipc`
