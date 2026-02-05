# STUN Server Capability Specification

**Version**: 1.0.0  
**Status**: ✅ Ready for Implementation (Phase 1 - MVP)  
**Created**: February 5, 2026  
**Priority**: Medium  
**Upstream Request**: biomeOS Integration Team

---

## 📋 Overview

This specification defines the Pure Rust STUN (Session Traversal Utilities for NAT) server capability for Songbird, enabling self-hosted NAT traversal without external dependencies.

**Purpose**: Eliminate coturn C dependency and enable single-binary deployment with integrated STUN server functionality.

---

## 🎯 Objectives

### Primary Goals

1. **Pure Rust Implementation** - RFC 5389 compliant STUN server
2. **ecoBin Compliance** - Zero C dependencies, zero unsafe code
3. **Single Binary** - Integrated into Songbird orchestrator
4. **JSON-RPC Integration** - Manageable via standard IPC
5. **Performance** - <1ms response time, <5MB memory for 1000 clients

### Future Goals (Deferred)

- NAT type detection (RFC 5780)
- Genetic lineage integration (family-only STUN)
- High-throughput optimizations

---

## 🏗️ Architecture

### Component Hierarchy

```
songbird-orchestrator
    ↓ JSON-RPC IPC
songbird-stun (crate)
    ├── client.rs (✅ exists, 383 lines)
    ├── server.rs (❌ new, ~280 lines)
    ├── message.rs (✅ exists, 493 lines)
    ├── types.rs (✅ exists, 61 lines)
    ├── error.rs (✅ exists, 43 lines)
    └── lib.rs (🔄 update)
```

### Existing Infrastructure (80% Complete)

| Component | Status | Purpose |
|-----------|--------|---------|
| **Message Encoding** | ✅ Complete | RFC 5389 binary encoding |
| **Message Decoding** | ✅ Complete | RFC 5389 binary decoding |
| **Attribute Handling** | ✅ Complete | MAPPED-ADDRESS, XOR-MAPPED-ADDRESS |
| **Error Types** | ✅ Complete | StunError, StunResult |
| **NAT Types** | ✅ Complete | Classification enums |
| **Client Reference** | ✅ Complete | Shows expected message flow |

---

## 📐 API Specification

### Server API

```rust
/// Pure Rust STUN Server (RFC 5389)
pub struct StunServer {
    bind_addr: SocketAddr,
    alternate_addr: Option<SocketAddr>,
    stats: Arc<RwLock<StunServerStats>>,
    shutdown: watch::Receiver<bool>,
}

impl StunServer {
    /// Create new STUN server
    pub fn new(bind_addr: SocketAddr) -> Self;
    
    /// Create with alternate address for NAT detection (Phase 2)
    pub fn with_alternate(
        bind_addr: SocketAddr, 
        alternate_addr: SocketAddr
    ) -> Self;
    
    /// Run server (blocks until shutdown)
    pub async fn run(&mut self) -> Result<(), StunError>;
    
    /// Get server statistics
    pub fn stats(&self) -> StunServerStats;
}
```

### JSON-RPC Methods

#### `stun.serve` - Start STUN Server

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "stun.serve",
  "params": {
    "bind_addr": "0.0.0.0:3478"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "started",
    "bind_addr": "0.0.0.0:3478",
    "server_id": "stun-1",
    "comment": "STUN server running in background"
  },
  "id": 1
}
```

#### `stun.stop` - Stop STUN Server

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "stun.stop",
  "params": {},
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "stopped",
    "requests_handled": 1234,
    "uptime_seconds": 3600
  },
  "id": 2
}
```

#### `stun.status` - Get Server Status

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "stun.status",
  "params": {},
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "running": true,
    "bind_addr": "0.0.0.0:3478",
    "requests_handled": 1234,
    "errors": 5,
    "uptime_seconds": 3600,
    "last_request_seconds_ago": 10
  },
  "id": 3
}
```

---

## 🔄 Message Flow

### STUN Binding Request/Response (RFC 5389)

```
Client                                Server
   │                                     │
   │  1. UDP Binding Request             │
   │  ─────────────────────────────────> │
   │     (transaction_id: random 96bit)  │
   │                                     │
   │  2. UDP Binding Response            │
   │  <───────────────────────────────── │
   │     (transaction_id: same)          │
   │     MAPPED-ADDRESS: client's IP:port│
   │     XOR-MAPPED-ADDRESS: XORed addr  │
   │                                     │
```

### Server Processing Steps

1. **Receive** UDP packet on bind_addr
2. **Parse** STUN message header (20 bytes)
3. **Validate** magic cookie (0x2112A442)
4. **Extract** transaction ID (12 bytes)
5. **Check** message type (0x0001 = Binding Request)
6. **Create** response with same transaction ID
7. **Add** MAPPED-ADDRESS attribute (client's src addr)
8. **Add** XOR-MAPPED-ADDRESS attribute (XORed)
9. **Add** SOFTWARE attribute ("songbird-stun/1.0")
10. **Encode** response message
11. **Send** UDP response to client
12. **Update** statistics

---

## 📊 Performance Requirements

| Metric | Target | Verification Method |
|--------|--------|---------------------|
| Response Time | <1ms | Benchmark |
| Throughput | >10,000 req/sec | Load test |
| Memory | <5MB for 1000 clients | Memory profiler |
| CPU | <5% single core | `top` monitoring |
| Binary Impact | <50KB | `cargo bloat` |
| Packet Loss | <0.1% | Stress test |

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[test]
fn test_binding_response_generation() {
    let server = StunServer::new("127.0.0.1:3478".parse().unwrap());
    let request = StunMessage::new_binding_request();
    let client_addr = "192.168.1.100:54321".parse().unwrap();
    
    let response = server.create_binding_response(&request, client_addr).unwrap();
    
    assert_eq!(response.message_type, MessageType::BindingResponse);
    assert_eq!(response.transaction_id, request.transaction_id);
    assert_eq!(response.get_mapped_address(), Some(client_addr));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_server_client_integration() {
    // Start server
    let mut server = StunServer::new("127.0.0.1:0".parse().unwrap());
    let server_addr = server.bind_addr;
    tokio::spawn(async move { server.run().await });
    
    // Test with existing client
    let client = StunClient::new();
    let discovered = client.discover_public_address(&server_addr.to_string()).await.unwrap();
    
    assert!(discovered.ip().is_loopback());
}
```

### Test Coverage Target

- **Unit Tests**: >85% coverage
- **Integration Tests**: All JSON-RPC methods
- **Stress Tests**: 10,000 concurrent requests
- **Chaos Tests**: Network failures, timeouts

---

## 🔒 Security Considerations

### Attack Surface

| Threat | Mitigation |
|--------|-----------|
| **UDP Amplification** | Rate limiting per source IP |
| **Resource Exhaustion** | Max concurrent requests (10,000) |
| **Invalid Messages** | Strict RFC 5389 validation |
| **Memory Leaks** | Bounded buffers (1500 bytes) |

### Safety Guarantees

- ✅ **Zero Unsafe Code** - All operations use safe Rust
- ✅ **Memory Safety** - Compiler-enforced bounds checking
- ✅ **Thread Safety** - Arc + RwLock for shared state
- ✅ **Error Handling** - All errors explicitly handled

---

## 📦 Dependencies

### Existing (No New Dependencies)

```toml
[dependencies]
tokio = { version = "1", features = ["net", "time", "sync"] }
tracing = "0.1"
bytes = "1.5"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
```

### Zero New Dependencies Required ✅

All infrastructure exists in `songbird-stun` crate.

---

## 🚀 Implementation Plan

### Phase 1: MVP (3-5 days) ⭐ CURRENT

**Scope**:
- Basic UDP server loop
- Binding request/response handling
- JSON-RPC integration
- Unit + integration tests

**Files**:
- `crates/songbird-stun/src/server.rs` (new, ~280 lines)
- `crates/songbird-stun/src/lib.rs` (update, +2 lines)
- `crates/songbird-universal-ipc/src/handlers/stun_handler.rs` (new, ~150 lines)
- Tests (~100 lines)

**Success Criteria**:
- ✅ Existing StunClient can use Songbird as STUN server
- ✅ JSON-RPC methods working
- ✅ Zero unsafe code
- ✅ >80% test coverage

### Phase 2: NAT Type Detection (2-3 days) 🔮 FUTURE

**Scope**: RFC 5780 support with alternate addresses

**Deferred**: Low priority, Phase 1 provides 90% of value

### Phase 3: Lineage Integration (3-4 days) 🧬 FUTURE

**Scope**: Family-only STUN with BearDog verification

**Deferred**: Requires BearDog lineage API

---

## 📈 Success Metrics

### Technical Metrics

- ✅ RFC 5389 compliance (binding request/response)
- ✅ <1ms response time (99th percentile)
- ✅ >10,000 req/sec throughput
- ✅ Zero unsafe code
- ✅ <50KB binary impact
- ✅ >80% test coverage

### Business Metrics

- ✅ coturn eliminated (zero C dependencies)
- ✅ Single-binary deployment
- ✅ ecoBin v2.0 compliance maintained
- ✅ biomeOS integration requirement met

---

## 🔗 Related Specifications

| Specification | Relationship |
|--------------|--------------|
| `CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md` | STUN is a discovery capability |
| `LINEAGE_GATED_RELAY_PROTOCOL.md` | Phase 3 integration point |
| `PRIMAL_COORDINATION_ARCHITECTURE.md` | Service coordination |
| `RENDEZVOUS_PROTOCOL_SPEC.md` | NAT traversal alternative |

---

## 📚 References

### RFC Standards

- **RFC 5389**: Session Traversal Utilities for NAT (STUN)
  - https://datatracker.ietf.org/doc/html/rfc5389
  - Sections 7-11: Server behavior

- **RFC 5780**: NAT Behavior Discovery Using STUN (Phase 2)
  - https://datatracker.ietf.org/doc/html/rfc5780

### Implementation References

- **Existing Code**: `crates/songbird-stun/src/` (1,030 lines, 80% complete)
- **Investigation**: `ecoPrimals/sessions/2026-02-february/STUN_SERVER_INVESTIGATION_FEB_05_2026.md`
- **Handoff**: `ecoPrimals/handoffs/PURE_RUST_STUN_SERVER_HANDOFF.md`

---

## ✅ Approval Status

| Stakeholder | Status | Date |
|-------------|--------|------|
| **Technical Review** | ✅ Approved | Feb 5, 2026 |
| **Architecture Review** | ✅ Approved | Feb 5, 2026 |
| **Security Review** | ✅ Approved | Feb 5, 2026 |
| **biomeOS Integration** | ✅ Ready | Feb 5, 2026 |

---

## 🎯 Implementation Status

**Current Phase**: Phase 1 (MVP)  
**Status**: ✅ Ready to Implement  
**Effort**: 3-5 days  
**Risk**: Low  
**Value**: High

---

**Specification Version**: 1.0.0  
**Last Updated**: February 5, 2026  
**Next Review**: After Phase 1 completion

🦀🧬✨ **Pure Rust STUN Server - Specified & Ready!** ✨🧬🦀
