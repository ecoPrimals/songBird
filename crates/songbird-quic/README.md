# Songbird QUIC Implementation

## Overview

`songbird-quic` provides a modern QUIC (Quick UDP Internet Connections) protocol layer for Songbird, enabling fast, secure, multiplexed connections with advanced features like 0-RTT, connection migration, and congestion control.

## Architecture

### Components

- **`QuicServer`**: Accepts incoming QUIC connections on IPv6 dual-stack
- **`QuicClient`**: Establishes outgoing QUIC connections with 0-RTT support
- **`QuicConnection`**: Manages a QUIC connection and multiplexed streams
- **`QuicStream`**: Bidirectional or unidirectional stream within a connection
- **`QuicConfig`**: Configuration with runtime BearDog socket discovery

### Dark Forest Compliance

All cryptographic operations are **delegated to BearDog**:

- ✅ Zero hardcoded secrets - BearDog socket discovered at runtime
- ✅ TLS 1.3 encryption (currently using rustls, will be replaced with BearDog provider)
- ✅ Zero metadata leakage design
- ✅ Timing protection via constant-time operations (delegated to BearDog)

### Features

#### 0-RTT (Zero Round Trip Time)
- Faster reconnection using cached session data
- Reduces latency for repeated connections
- Automatically falls back to 1-RTT if unavailable

#### Connection Migration
- Seamless IP address changes (mobile roaming, network switches)
- Maintains connection state across network transitions
- No reconnection required

#### Stream Multiplexing
- Multiple independent streams over single connection
- No head-of-line blocking between streams
- Efficient resource utilization

## Usage

### Server

```rust
use songbird_quic::{QuicServer, QuicConfig};

let config = QuicConfig::new()
    .with_idle_timeout(Duration::from_secs(60))
    .with_0rtt(true)
    .with_migration(true);

// Bind to IPv6 dual-stack (supports both IPv4 and IPv6)
let server = QuicServer::new("[::]:4433", config).await?;

// Accept connections
let mut incoming = server.accept();
while let Some(conn) = incoming.recv().await {
    tokio::spawn(async move {
        // Handle bidirectional streams
        loop {
            let mut stream = conn.accept_bi().await?;
            
            // Read/write data
            let mut buf = vec![0u8; 4096];
            let n = stream.read(&mut buf).await?;
            stream.write(&buf[..n]).await?;
        }
    });
}
```

### Client

```rust
use songbird_quic::{QuicClient, QuicConfig};

let config = QuicConfig::new()
    .with_0rtt(true)
    .with_migration(true);

let client = QuicClient::new(config).await?;

// Connect with 0-RTT if possible
let conn = client.connect_0rtt("[::1]:4433").await?;

// Open bidirectional stream
let mut stream = conn.open_bi().await?;

// Send/receive data
stream.write(b"Hello QUIC").await?;
let mut buf = vec![0u8; 1024];
let n = stream.read(&mut buf).await?;
```

### Stream Multiplexing Example

```rust
// Open multiple streams simultaneously
for i in 0..10 {
    let mut stream = conn.open_bi().await?;
    tokio::spawn(async move {
        stream.write(format!("Stream {}", i).as_bytes()).await?;
        // Each stream is independent - no head-of-line blocking
    });
}
```

## Configuration

### BearDog Socket Discovery

`QuicConfig` discovers the BearDog socket at runtime (no hardcoding):

1. `BEARDOG_SOCKET` environment variable
2. `SONGBIRD_SECURITY_PROVIDER` environment variable
3. `$XDG_RUNTIME_DIR/biomeos/beardog.sock`
4. `/tmp/biomeos/beardog.sock` (fallback)

### Transport Configuration

```rust
let config = QuicConfig::new()
    .with_idle_timeout(Duration::from_secs(30))      // Connection timeout
    .with_0rtt(true)                                  // Enable 0-RTT
    .with_migration(true)                             // Enable connection migration
    .with_beardog_socket(PathBuf::from("/custom/beardog.sock"));
```

## Integration with Sovereign Multi-Path Protocol

QUIC is **Tier 8** in the Sovereign Multi-Path Protocol (after IPv6, Onion, IPv4, LAN, STUN, Family Relay, DNS Beacon).

### Why QUIC?

- **Performance**: UDP-based, faster than TCP
- **Mobile-friendly**: Connection migration for network switches
- **Modern**: Built-in congestion control, loss recovery
- **Multiplexed**: No head-of-line blocking
- **Secure**: Mandatory TLS 1.3 encryption

### Integration Points

```rust
// In Multi-Path connection manager
async fn try_quic_connection(&self, target: &PrimalAddress) -> Result<Connection> {
    if let Some(quic_endpoint) = target.quic_endpoint {
        let quic_client = QuicClient::new(self.quic_config.clone()).await?;
        let conn = quic_client.connect_0rtt(&quic_endpoint).await?;
        return Ok(Connection::Quic(conn));
    }
    Err(TorError::NoQuicEndpoint)
}
```

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Connection setup | 1-RTT (or 0-RTT with session resumption) |
| Head-of-line blocking | None (per-stream) |
| Congestion control | Built-in (cubic/bbr) |
| Connection migration | Seamless |
| Multiplexing | Unlimited streams |
| MTU discovery | Automatic |

## Security Model

### Current (Temporary)

- **TLS 1.3** via rustls
- Self-signed certificates (development only)
- Certificate verification skipped (will be replaced)

### Future (BearDog Integration)

- **Custom crypto provider** for rustls
- **BearDog-delegated** certificate generation
- **BearDog-verified** peer certificates
- **Zero secrets** in Songbird code

## Deep Debt Compliance

✅ **Zero unsafe code** - `#![forbid(unsafe_code)]`  
✅ **Runtime discovery** - BearDog socket discovered, not hardcoded  
✅ **Pure Rust** - No C dependencies (quinn, rustls)  
✅ **Modern idioms** - Async/await, Result, thiserror  
✅ **No mocks** - Real implementations only  

## Examples

Run examples:

```bash
# Server
cargo run --example quic_echo_server

# Client (in another terminal)
cargo run --example quic_echo_client
```

## Future Enhancements

1. **BearDog Crypto Provider**: Replace rustls crypto with BearDog delegation
2. **Peer Certificate Verification**: Use BearDog for primal identity verification
3. **Datagrams**: Unreliable datagram support (for voice/video)
4. **Priority Streams**: QoS for critical data
5. **Metrics**: Connection statistics and performance monitoring
6. **GSO/GRO**: Generic segmentation/receive offload for performance

## References

- [QUIC RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [quinn documentation](https://docs.rs/quinn/latest/quinn/)
- [rustls documentation](https://docs.rs/rustls/latest/rustls/)
- [PROTOCOL_EVOLUTION_REFINED_FEB_08_2026.md](../../../PROTOCOL_EVOLUTION_REFINED_FEB_08_2026.md)
