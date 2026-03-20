# Relay Handler IPC Wiring - ✅ COMPLETE

**Status**: ✅ **WIRED AND VERIFIED** (Feb 5, 2026)  
**Effort**: ~30 minutes  
**Priority**: High (completes sovereign NAT traversal)

---

## ✅ COMPLETED

The relay handler is now fully wired to the IPC service:

| Component | Location | Status |
|-----------|----------|--------|
| `relay_protocol.rs` | `songbird-lineage-relay/src/` | ✅ Complete |
| `relay_server.rs` | `songbird-lineage-relay/src/` | ✅ Complete |
| `relay_handler.rs` | `songbird-lineage-relay/src/` | ✅ Complete |
| **IPC Integration** | `songbird-universal-ipc/` | ✅ **WIRED** |

### Verified Test Results:
```
relay.serve:    ✅ Server listening on 0.0.0.0:3479
rpc.discover:   ✅ Shows relay.serve, relay.stop, relay.status, relay.allocate
ss -ulnp:       ✅ Songbird process bound to UDP :3479
```

**Note**: `relay.status` shows `running: false` due to state tracking bug,
but the server IS running (verified via `ss -ulnp`). Bug to fix in relay_handler.rs.

---

## Required Changes

### 1. Add Relay Handler to IPC Service

File: `crates/songbird-universal-ipc/src/service.rs`

```rust
// Add import at top
use songbird_lineage_relay::relay_handler::RelayHandler;

// In IpcServiceHandler struct, add field:
relay_handler: Arc<RelayHandler>,

// In IpcServiceHandler::new(), initialize:
let relay_handler = Arc::new(RelayHandler::new(relay_authority));

// In handle() match statement, add these cases:
"relay.serve" => self.relay_handler.handle_serve(params).await,
"relay.stop" => self.relay_handler.handle_stop(params).await,
"relay.status" => self.relay_handler.handle_status(params).await,
"relay.allocate" => self.relay_handler.handle_allocate(params).await,
```

### 2. Update rpc.discover Method

Add to the methods list in `handle_rpc_discover_standard()`:

```rust
"relay.serve", "relay.stop", "relay.status", "relay.allocate",
```

### 3. Add Dependency to Cargo.toml

File: `crates/songbird-universal-ipc/Cargo.toml`

```toml
[dependencies]
songbird-lineage-relay = { path = "../songbird-lineage-relay" }
```

---

## Mock RelayAuthority for Testing

The `RelayHandler` requires a `RelayAuthority` implementation. For testing without BearDog:

```rust
use songbird_lineage_relay::relay::RelayAuthority;
use songbird_lineage_relay::types::{MaskingLevel, NodeId, RelayAuthorization};

pub struct MockRelayAuthority;

#[async_trait]
impl RelayAuthority for MockRelayAuthority {
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        Ok(RelayAuthorization {
            relay_node: relay_node.clone(),
            requester: requester.clone(),
            authorized: true,
            masking_level: MaskingLevel::Masked,
            ttl_seconds: 300,
            issued_at: SystemTime::now(),
            audit_token: "mock".to_string(),
        })
    }
    
    async fn determine_masking(
        &self,
        _relay_node: &NodeId,
        _requester: &NodeId,
    ) -> Result<MaskingLevel> {
        Ok(MaskingLevel::Masked)
    }
}
```

---

## Test After Wiring

```bash
# Start relay server
echo '{"jsonrpc":"2.0","method":"relay.serve","params":{"bind_addr":"0.0.0.0:3479"},"id":1}' \
  | nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Check status
echo '{"jsonrpc":"2.0","method":"relay.status","params":{},"id":2}' \
  | nc -U /run/user/1000/biomeos/songbird-nat0.sock
```

---

## Validation Checklist

- [x] `relay.serve` starts relay server ✅
- [x] `relay.stop` stops relay server ✅
- [x] `relay.status` returns stats ⚠️ (state tracking bug)
- [x] `relay.allocate` ready for allocation ✅
- [x] Methods appear in `rpc.discover` ✅

---

**Created**: February 5, 2026  
**Completed**: February 5, 2026  
**For**: Songbird evolution to complete sovereign NAT traversal

## What's Next

1. **Fix state tracking bug** in `relay_handler.rs` - `handle_status` returns "running: false" after serve
2. **Complete `RelaySession.send()`** - Currently a stub, needs actual UDP forwarding
3. **Cross-NAT testing** - Test relay between Tower and Pixel over symmetric NAT
