# Tor Transport Integration Specification

**Version**: 1.0.0  
**Date**: February 6, 2026  
**Status**: Draft  
**Implementation**: Phase 1A

---

## Abstract

This specification defines the Tor transport layer for the Sovereign Beacon Mesh using **Arti** (Pure Rust Tor implementation). The transport enables sovereign bootstrap by creating and connecting to onion services without requiring port forwarding or external infrastructure.

---

## 1. Overview

### 1.1 Purpose

The Tor transport provides:
1. **Bootstrap capability** - First device creates .onion address
2. **Outbound connections** - All devices can connect to .onion addresses
3. **Pure Rust** - Zero C dependencies via Arti
4. **Minimal usage** - Signaling only, not main data path

### 1.2 Architecture

```
┌─────────────────────────────────────────────────┐
│            TorTransport (Arti)                   │
├─────────────────────────────────────────────────┤
│                                                  │
│  Phase 1A: OUTBOUND ONLY (Stable API)          │
│  ├── Bootstrap Tor client                       │
│  ├── Connect to .onion addresses                │
│  └── Error handling & timeouts                  │
│                                                  │
│  Phase 1B: FULL (When API Stable)              │
│  ├── Create onion services                      │
│  ├── Accept inbound connections                 │
│  └── Advertise .onion in beacon                 │
│                                                  │
└─────────────────────────────────────────────────┘
```

---

## 2. Arti Integration

### 2.1 Dependencies

```toml
[dependencies]
# Phase 1A: Outbound connections (stable)
arti-client = "0.24"
tor-rtcompat = { version = "0.24", features = ["tokio"] }

# Phase 1B: Onion services (experimental)
tor-hsservice = { version = "0.24", optional = true }
```

### 2.2 Bootstrap Process

```rust
use arti_client::{TorClient, TorClientConfig};

// 1. Create default config
let config = TorClientConfig::default();

// 2. Bootstrap (downloads consensus, connects to relays)
let client = TorClient::create_bootstrapped(config).await?;
// Takes 10-30s depending on network

// 3. Client is ready to use
```

**Performance**:
- First bootstrap: 10-30s (downloads consensus)
- Subsequent: <5s (uses cached consensus)
- Memory: ~50MB for Tor client

---

## 3. API Specification

### 3.1 TorTransport

```rust
pub struct TorTransport {
    client: TorClient<PreferredRuntime>,
    bootstrap_time: Duration,
}

impl TorTransport {
    /// Bootstrap new Tor client
    pub async fn new() -> Result<Self>;
    
    /// Connect to onion service
    pub async fn connect(&self, onion_addr: &str, port: u16) -> Result<TorStream>;
    
    /// Get bootstrap duration (for metrics)
    pub fn bootstrap_time(&self) -> Duration;
    
    /// Check if client is connected
    pub fn is_connected(&self) -> bool;
}
```

### 3.2 TorStream

```rust
pub struct TorStream {
    stream: arti_client::DataStream,
}

impl TorStream {
    /// Read data from stream
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    
    /// Write data to stream
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize>;
    
    /// Flush buffered data
    pub async fn flush(&mut self) -> Result<()>;
}
```

---

## 4. Error Handling

### 4.1 Error Types

```rust
#[derive(Error, Debug)]
pub enum OnionRelayError {
    #[error("Tor bootstrap failed: {0}")]
    TorBootstrapFailed(String),
    
    #[error("Tor connection timeout")]
    TorTimeout,
    
    #[error("Onion service unreachable: {0}")]
    OnionUnreachable(String),
    
    #[error("Tor network error: {0}")]
    TorNetwork(String),
}
```

### 4.2 Timeout Strategy

| Operation | Timeout | Rationale |
|-----------|---------|-----------|
| Bootstrap | 60s | Allow for slow network |
| Connect | 30s | Standard Tor connection time |
| Read/Write | 10s | Detect stalled connections |

---

## 5. Usage Examples

### 5.1 Bootstrap and Connect

```rust
use songbird_onion_relay::TorTransport;

// Bootstrap Tor client (once at startup)
let transport = TorTransport::new().await?;
println!("Bootstrapped in {:?}", transport.bootstrap_time());

// Connect to onion service
let mut stream = transport.connect("abc123...xyz.onion", 80).await?;

// Send signaling message
let msg = b"HELLO";
stream.write(msg).await?;
stream.flush().await?;

// Receive response
let mut buf = [0u8; 1024];
let n = stream.read(&mut buf).await?;
println!("Received {} bytes", n);
```

### 5.2 Integration with Mesh

```rust
use songbird_onion_relay::{TorTransport, BeaconMesh};

// Bootstrap Tor
let tor = Arc::new(TorTransport::new().await?);

// Create mesh
let mesh = BeaconMesh::new(
    "tower-abc123".to_string(),
    vec!["bootstrap.onion".to_string()],
);

// Connect to bootstrap onion
let stream = tor.connect("bootstrap.onion", 9735).await?;

// Exchange signaling messages
// ... (hole punch coordination)
```

---

## 6. Performance Characteristics

### 6.1 Latency

| Operation | Latency | Notes |
|-----------|---------|-------|
| Bootstrap | 10-30s | One-time per device boot |
| Connect | 2-5s | Tor circuit creation |
| Round-trip | 300-800ms | 3-hop Tor latency |
| Bandwidth | 1-5 Mbps | Sufficient for signaling |

### 6.2 Resource Usage

| Resource | Usage | Notes |
|----------|-------|-------|
| Memory | ~50MB | Tor client overhead |
| CPU | ~5% | During bootstrap |
| Disk | ~10MB | Cached consensus |
| Network | ~2MB | Bootstrap download |

---

## 7. Security Considerations

### 7.1 Threat Model

**Protected Against**:
- ✅ Network surveillance (Tor encryption)
- ✅ IP address exposure (onion routing)
- ✅ Port scanning (no exposed ports)

**Not Protected Against**:
- ⚠️ Traffic analysis (Tor known limitation)
- ⚠️ Compromised Tor relays (mitigated by 3-hop)
- ⚠️ Timing attacks (acceptable for signaling)

### 7.2 Privacy Properties

**Onion Addresses**:
- .onion addresses are family-encrypted in beacons
- Only family members can decrypt and connect
- Rendezvous server sees encrypted blobs only

**Usage Pattern**:
- Tor used only for bootstrap signaling
- Disconnected after hole punch succeeds
- Minimal fingerprinting (short sessions)

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[test]
fn test_tor_transport_creation() { /* ... */ }

#[test]
#[ignore = "Requires network"]
async fn test_tor_bootstrap_real() { /* ... */ }

#[test]
async fn test_connection_timeout() { /* ... */ }

#[test]
async fn test_stream_read_write() { /* ... */ }
```

### 8.2 Integration Tests

```rust
#[test]
#[ignore = "Slow - requires Tor"]
async fn test_bootstrap_and_connect() {
    let transport = TorTransport::new().await.unwrap();
    // Test with known onion service
}

#[test]
async fn test_mesh_tor_integration() {
    // Full flow: bootstrap → connect → signaling
}
```

---

## 9. Future Enhancements

### 9.1 Onion Service Creation (Phase 1B)

**When**: Arti API stabilizes

```rust
impl TorTransport {
    pub async fn create_onion_service(&mut self, port: u16) -> Result<String> {
        use tor_hsservice::{OnionServiceConfig};
        
        let config = OnionServiceConfig::builder()
            .nickname("songbird-beacon")
            .build()?;
        
        let service = self.client.launch_onion_service(config).await?;
        let address = service.onion_name().to_string();
        
        Ok(address)
    }
}
```

### 9.2 Pluggable Transports

**Goal**: Obfuscate Tor traffic

**Options**:
- obfs4 (obfuscates Tor protocol)
- meek (domain fronting)
- snowflake (WebRTC proxy)

**Effort**: 1-2 weeks  
**Value**: Bypass Tor blocking

---

## 10. References

### Arti Documentation
- **Main**: https://tpo.pages.torproject.net/core/doc/rust/arti_client/
- **GitHub**: https://gitlab.torproject.org/tpo/core/arti
- **Spec**: https://spec.torproject.org/

### Tor Protocol
- [Tor Protocol Specification] https://spec.torproject.org/tor-spec
- [Onion Service Protocol] https://spec.torproject.org/rend-spec-v3
- [RFC 7686] The ".onion" Special-Use Domain Name

---

**Specification Version**: 1.0.0  
**Implementation**: crates/songbird-onion-relay/src/tor_transport.rs  
**Status**: Draft - Phase 1A in progress

🦀 **Pure Rust** | 🧅 **Tor via Arti** | 🔒 **Sovereign Bootstrap**
