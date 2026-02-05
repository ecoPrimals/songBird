# Pure Rust STUN Server Implementation Handoff

**From**: biomeOS Integration Team  
**To**: Songbird Core Team  
**Priority**: Medium (coturn bridging for now)  
**Status**: Ready for implementation  
**Created**: February 5, 2026

---

## Executive Summary

biomeOS requires a self-hosted STUN server capability to achieve **maximum sovereignty** in NAT traversal. Currently using coturn as a bridge solution, but a pure Rust implementation in Songbird would:

1. Eliminate external C dependencies (ecoBin compliance)
2. Enable single-binary deployment (no coturn installation)
3. Allow genetic lineage integration (family-only STUN)
4. Maintain zero-unsafe codebase goal

---

## Current State

### What Exists

| Component | Location | Status |
|-----------|----------|--------|
| STUN Client | `songbird-stun/src/client.rs` | ✅ Working |
| STUN Message Parser | `songbird-stun/src/message.rs` | ✅ Working |
| STUN Types | `songbird-stun/src/types.rs` | ✅ Working |
| Multi-tier Config | `songbird-types/src/config/stun_relay.rs` | ✅ Working |
| UDP Hole Punch | `songbird-lineage-relay/src/udp_hole_punch.rs` | ✅ Working |
| **STUN Server** | - | ❌ **Missing** |
| **TURN Server** | - | ❌ **Missing** |

### Bridge Solution (ACTIVE - coturn with STUN + TURN)

coturn is currently running on Tower with TURN relay enabled:
- **Status**: ✅ Active (`systemctl status coturn`)
- **Address**: `192.168.1.144:3478` (LAN)
- **Config**: `/etc/turnserver.conf`
- **Setup Script**: `biomeOS/scripts/setup_coturn.sh`
- **TURN Enabled**: ✅ Yes (for symmetric NAT traversal)
- **TURN Credentials**: `biomeos:darkforest2026` (realm: `biomeos.local`)
- **Relay Ports**: UDP 49152-65535

**Rust Integration**:
- `biomeos-core/src/stun_extension.rs` - Optional STUN extension
- Automatically falls back to public STUN if self-hosted unavailable
- Zero hard dependency on coturn
- TURN credentials can be derived from beacon seed for family-only access

**Tested Feb 5, 2026**:
```
Self-hosted STUN (coturn):  162.226.225.148:54169 ✅
Public STUN (Google):       162.226.225.148:53213 ✅
TURN Relay (coturn):        ✅ Enabled (lt-cred-mech)
```

**NAT Traversal Validation**:
- Tower NAT: Symmetric (port varies per destination)
- Pixel NAT: Symmetric (iPhone hotspot)
- Direct Hole Punch: ❌ Not possible (symmetric-to-symmetric)
- TURN Relay: ✅ Required for these NAT types

---

## Requirements

### Functional Requirements

1. **STUN Binding Response** (RFC 5389)
   - Receive STUN Binding Request
   - Extract source IP:port from UDP packet
   - Return MAPPED-ADDRESS attribute

2. **NAT Type Detection** (RFC 5780 - Optional)
   - Respond to CHANGE-REQUEST attribute
   - Use alternate IP/port when requested
   - Enables clients to detect NAT type

3. **TURN Relay** (RFC 5766 - Required for Symmetric NAT)
   - Allocate relay addresses for clients
   - Forward packets between relay and peer
   - Support long-term credential mechanism
   - Support CHANNEL-DATA for efficient relay (optional)
   - Support CreatePermission for peer whitelisting

4. **Integration**
   - Start/stop via Songbird orchestrator
   - Expose via JSON-RPC methods: `stun.serve`, `turn.serve`
   - Configurable bind address, port, and relay port range
   - TURN credentials via config or derived from beacon seed

### Non-Functional Requirements

1. **Pure Rust** - No C dependencies (ecoBin v2.0)
2. **Zero Unsafe** - Maintain safety guarantees
3. **Async** - tokio-based, non-blocking
4. **Minimal Footprint** - <100KB binary size impact (STUN+TURN)
5. **Family-Aware** - Optional lineage verification for TURN auth

---

## Proposed Architecture

### File Structure

```
crates/songbird-stun/src/
├── client.rs          # Existing STUN client
├── server.rs          # NEW: STUN server implementation
├── message.rs         # Existing: Shared message parsing
├── types.rs           # Existing: Shared types
├── error.rs           # Existing: Error types
└── lib.rs             # Updated: Export server
```

### API Design

```rust
// crates/songbird-stun/src/server.rs

use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// Pure Rust STUN Server (RFC 5389)
pub struct StunServer {
    /// Bind address for incoming STUN requests
    bind_addr: SocketAddr,
    
    /// Optional alternate address for NAT type testing (RFC 5780)
    alternate_addr: Option<SocketAddr>,
    
    /// Server statistics
    stats: StunServerStats,
}

impl StunServer {
    /// Create new STUN server
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            alternate_addr: None,
            stats: StunServerStats::default(),
        }
    }
    
    /// Create with alternate address for NAT type detection
    pub fn with_alternate(bind_addr: SocketAddr, alternate_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            alternate_addr: Some(alternate_addr),
            stats: StunServerStats::default(),
        }
    }
    
    /// Run the STUN server (async)
    /// 
    /// Handles incoming STUN Binding Requests and returns
    /// MAPPED-ADDRESS with the client's public IP:port.
    pub async fn run(&mut self) -> Result<(), StunError> {
        let socket = UdpSocket::bind(self.bind_addr).await?;
        let mut buf = vec![0u8; 1500]; // MTU size
        
        loop {
            let (len, src_addr) = socket.recv_from(&mut buf).await?;
            
            if let Ok(request) = StunMessage::parse(&buf[..len]) {
                if request.is_binding_request() {
                    let response = self.create_binding_response(&request, src_addr)?;
                    socket.send_to(&response.encode(), src_addr).await?;
                    self.stats.requests_handled += 1;
                }
            }
        }
    }
    
    /// Create STUN Binding Response with MAPPED-ADDRESS
    fn create_binding_response(
        &self,
        request: &StunMessage,
        client_addr: SocketAddr,
    ) -> Result<StunMessage, StunError> {
        let mut response = StunMessage::new_success_response(request.transaction_id());
        
        // MAPPED-ADDRESS: client's source IP:port as seen by server
        response.add_attribute(StunAttribute::MappedAddress(client_addr));
        
        // XOR-MAPPED-ADDRESS: XORed for NAT hairpinning (RFC 5389 recommends)
        response.add_attribute(StunAttribute::XorMappedAddress(client_addr));
        
        // SOFTWARE: identify server
        response.add_attribute(StunAttribute::Software("songbird-stun/1.0".into()));
        
        Ok(response)
    }
    
    /// Get server statistics
    pub fn stats(&self) -> &StunServerStats {
        &self.stats
    }
}

#[derive(Debug, Default)]
pub struct StunServerStats {
    pub requests_handled: u64,
    pub errors: u64,
    pub start_time: Option<std::time::Instant>,
}
```

### JSON-RPC Integration

Add to `songbird-universal-ipc/src/service.rs`:

```rust
// New method: stun.serve
"stun.serve" => {
    let bind_addr = params.get("bind_addr")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0.0:3478");
    
    self.start_stun_server(bind_addr).await
}

// Implementation
async fn start_stun_server(&self, bind_addr: &str) -> Result<Value, String> {
    let addr: SocketAddr = bind_addr.parse()
        .map_err(|e| format!("Invalid bind address: {e}"))?;
    
    let mut server = StunServer::new(addr);
    
    // Spawn in background
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            tracing::error!("STUN server error: {}", e);
        }
    });
    
    Ok(json!({
        "status": "started",
        "bind_addr": bind_addr,
        "comment": "STUN server running in background"
    }))
}
```

---

## Implementation Guide

### Phase 1: Basic STUN Server (MVP)

**Goal**: Handle Binding Requests, return MAPPED-ADDRESS

```rust
// Minimal implementation
pub async fn handle_binding_request(
    socket: &UdpSocket,
    request_bytes: &[u8],
    src_addr: SocketAddr,
) -> Result<(), StunError> {
    // 1. Parse request
    let request = StunMessage::parse(request_bytes)?;
    
    // 2. Verify it's a Binding Request
    if request.message_type() != STUN_BINDING_REQUEST {
        return Err(StunError::InvalidMessageType);
    }
    
    // 3. Create response with client's address
    let mut response = StunMessage::new_success_response(request.transaction_id());
    response.add_mapped_address(src_addr);
    response.add_xor_mapped_address(src_addr);
    
    // 4. Send response
    socket.send_to(&response.encode(), src_addr).await?;
    
    Ok(())
}
```

### Phase 2: NAT Type Detection (Optional)

**Goal**: Support RFC 5780 for NAT type detection

```rust
// Handle CHANGE-REQUEST attribute
if let Some(change_request) = request.get_attribute::<ChangeRequest>() {
    if change_request.change_ip && self.alternate_addr.is_some() {
        // Respond from alternate IP
        let alt_socket = UdpSocket::bind(self.alternate_addr.unwrap()).await?;
        alt_socket.send_to(&response.encode(), src_addr).await?;
    }
    
    if change_request.change_port {
        // Respond from alternate port
        // ...
    }
}

// Add OTHER-ADDRESS attribute so clients know alternate is available
response.add_attribute(StunAttribute::OtherAddress(self.alternate_addr.unwrap()));
```

### Phase 3: Genetic Lineage Integration (Future)

**Goal**: Optional family-only STUN access

```rust
// Family-aware STUN server
pub struct LineageStunServer {
    inner: StunServer,
    beardog_socket: String,
    family_only: bool,
}

impl LineageStunServer {
    pub async fn handle_request(
        &self,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> Result<StunMessage, StunError> {
        if self.family_only {
            // Check if request includes lineage proof
            if let Some(lineage_attr) = request.get_attribute::<LineageProof>() {
                // Verify with BearDog
                let is_family = self.verify_lineage(&lineage_attr).await?;
                
                if !is_family {
                    return Err(StunError::Unauthorized("Not family".into()));
                }
            } else {
                return Err(StunError::Unauthorized("Lineage proof required".into()));
            }
        }
        
        self.inner.create_binding_response(request, src_addr)
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_binding_response() {
        let server = StunServer::new("127.0.0.1:0".parse().unwrap());
        
        let request = StunMessage::new_binding_request();
        let client_addr: SocketAddr = "192.168.1.100:12345".parse().unwrap();
        
        let response = server.create_binding_response(&request, client_addr).unwrap();
        
        assert!(response.is_success_response());
        assert_eq!(response.get_mapped_address(), Some(client_addr));
    }
    
    #[tokio::test]
    async fn test_server_loopback() {
        // Start server on random port
        let server_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut server = StunServer::new(server_addr);
        
        let server_handle = tokio::spawn(async move {
            server.run().await
        });
        
        // Give server time to bind
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Use existing StunClient to test
        let client = StunClient::new();
        let result = client.discover_public_address(&server_addr.to_string()).await;
        
        // Should return loopback address
        assert!(result.is_ok());
        
        server_handle.abort();
    }
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires network
async fn test_stun_server_real_network() {
    // Start server
    let mut server = StunServer::new("0.0.0.0:3478".parse().unwrap());
    tokio::spawn(async move { server.run().await });
    
    // Test from another device via Songbird JSON-RPC
    // echo '{"jsonrpc":"2.0","method":"stun.get_public_address",
    //   "params":{"server":"192.168.1.144:3478"},"id":1}' | nc ...
}
```

---

## Existing Code to Leverage

### StunMessage (already exists)

```rust:crates/songbird-stun/src/message.rs
// Already have message parsing - just need to add encoding
impl StunMessage {
    pub fn parse(bytes: &[u8]) -> Result<Self, StunError> {
        // ... existing parsing code
    }
    
    // ADD: Encoding for responses
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);
        
        // STUN header (20 bytes)
        buf.extend_from_slice(&self.message_type.to_be_bytes());
        // ... encode attributes
        
        buf
    }
}
```

### StunClient (reference implementation)

```rust:crates/songbird-stun/src/client.rs
// Can reference the client for message format expectations
impl StunClient {
    pub async fn discover_public_address(&self, server: &str) -> Result<SocketAddr, StunError> {
        // ... sends Binding Request, expects response with MAPPED-ADDRESS
    }
}
```

---

## Success Criteria

| Criteria | Metric |
|----------|--------|
| Binding Response | Returns correct MAPPED-ADDRESS |
| Performance | <1ms response time |
| Memory | <1MB for 1000 concurrent clients |
| Binary Size | <50KB impact on Songbird binary |
| Test Coverage | >80% for server module |
| Zero Unsafe | No unsafe blocks |
| ecoBin Compliance | Pure Rust, no C deps |

---

## Timeline Estimate

| Phase | Scope | Effort |
|-------|-------|--------|
| Phase 1 | Basic STUN server (MVP) | ~2-3 days |
| Phase 2 | NAT type detection | ~1-2 days |
| Phase 3 | JSON-RPC integration | ~1 day |
| Phase 4 | Lineage integration | ~2-3 days |
| Testing | Unit + integration | ~1-2 days |

**Total**: ~1-2 weeks for full implementation

---

## References

1. **RFC 5389**: Session Traversal Utilities for NAT (STUN)
   - https://datatracker.ietf.org/doc/html/rfc5389

2. **RFC 5780**: NAT Behavior Discovery Using STUN
   - https://datatracker.ietf.org/doc/html/rfc5780

3. **Existing Rust STUN crates** (for reference, not dependency):
   - `stun` crate: https://crates.io/crates/stun
   - `stun-types`: https://crates.io/crates/stun-types

4. **coturn** (bridge solution reference):
   - https://github.com/coturn/coturn

---

## Contact

For questions about biomeOS integration requirements:
- Open issue in biomeOS repo
- Reference this handoff document

---

**Bridge Solution**: Until pure Rust implementation is ready, use `biomeOS/scripts/setup_coturn.sh` for self-hosted STUN.

**Priority**: Medium - coturn works, but pure Rust maintains ecosystem integrity.
