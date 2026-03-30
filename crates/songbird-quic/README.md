# Songbird QUIC — Pure Rust RFC 9000 Transport

## Overview

`songbird-quic` is a pure Rust QUIC transport implementation for the ecoPrimals ecosystem. All cryptographic operations are delegated to BearDog via JSON-RPC IPC — the same Tower Atomic pattern used for HTTPS. Zero C dependencies (`quinn`, `rustls`, and `ring` have been fully replaced).

## Architecture

```text
Application Data
    ↓
QUIC Transport (native Rust — streams, congestion, loss recovery)
    ↓
QUIC Crypto (BearDog via JSON-RPC IPC — AEAD, HKDF, HP)
    ↓
TLS 1.3 Handshake (BearDog X25519 + key schedule)
    ↓
UDP (Tokio)
    ↓
IPv4/IPv6
```

### Components

- **`QuicServer`**: Accepts incoming QUIC connections on IPv6 dual-stack
- **`QuicClient`**: Establishes outgoing QUIC connections with 0-RTT support
- **`QuicConnection`**: Manages a QUIC connection and multiplexed streams
- **`QuicStream`**: Bidirectional or unidirectional stream within a connection
- **`QuicConfig`**: Configuration with runtime BearDog socket discovery

### Internal Modules

| Module | RFC | Purpose |
|--------|-----|---------|
| `varint` | 9000 §16 | Variable-length integer encoding |
| `packet::header` | 9000 §17 | Long and Short header parsing/serialization |
| `packet::frame` | 9000 §19 | All 24 QUIC frame types |
| `packet::number` | 9000 App A | Packet number codec and expansion |
| `crypto::provider` | — | `QuicCryptoProvider` trait delegating to BearDog |
| `crypto::initial_keys` | 9001 §5.2 | Initial secrets from DCID via HKDF |
| `crypto::packet_protection` | 9001 §5.3 | AEAD encrypt/decrypt with PN nonce |
| `crypto::header_protection` | 9001 §5.4 | Header protection masking |
| `crypto::key_update` | 9001 §6 | 1-RTT key rotation |
| `tls::transport_params` | 9000 §18 | Transport parameter encoding (extension 0x39) |
| `tls::handshake` | 8446 | TLS 1.3 handshake state machine for QUIC |
| `tls::session` | — | Encryption level key management |
| `transport::state` | 9000 §10 | Connection state machine |
| `transport::streams` | 9000 §2 | Bidi/uni stream multiplexing |
| `transport::flow_control` | 9000 §4 | Connection and stream-level flow control |
| `transport::loss` | 9002 | Loss detection, PTO, RTT estimation |
| `transport::congestion` | 9002 App B | NewReno congestion control |
| `endpoint::udp` | — | Tokio UDP socket management |
| `cert_gen` | — | Pure-Rust Ed25519 self-signed certificates |

### Dark Forest Compliance

- Zero hardcoded secrets — BearDog socket discovered at runtime
- TLS 1.3 encryption via BearDog crypto delegation
- Zero metadata leakage design
- Timing protection via constant-time operations (delegated to BearDog)

## Usage

### Server

```rust
use songbird_quic::{QuicServer, QuicConfig};

let config = QuicConfig::new()
    .with_idle_timeout(Duration::from_secs(60))
    .with_0rtt(true)
    .with_migration(true);

let server = QuicServer::new("[::]:4433", config).await?;

let mut incoming = server.accept();
while let Some(conn) = incoming.recv().await {
    tokio::spawn(async move {
        let mut stream = conn.open_bi().await?;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        stream.write(&buf[..n]).await?;
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
let conn = client.connect_0rtt("[::1]:4433").await?;

let mut stream = conn.open_bi().await?;
stream.write(b"Hello QUIC").await?;
let mut buf = vec![0u8; 1024];
let n = stream.read(&mut buf).await?;
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
    .with_idle_timeout(Duration::from_secs(30))
    .with_0rtt(true)
    .with_migration(true)
    .with_beardog_socket(PathBuf::from("/custom/beardog.sock"));
```

## ecoBin Compliance

- Zero C dependencies (`quinn`, `rustls`, `ring` fully eliminated)
- `#![forbid(unsafe_code)]`
- BearDog crypto delegation via JSON-RPC IPC
- Runtime discovery — no hardcoded endpoints
- Pure Rust — all protocol layers implemented natively

## Performance

| Metric | Value |
|--------|-------|
| Connection setup | 1-RTT (or 0-RTT with session resumption) |
| Head-of-line blocking | None (per-stream) |
| Congestion control | NewReno (RFC 9002 Appendix B) |
| Connection migration | Seamless |
| Multiplexing | Unlimited streams |
| Loss detection | RFC 9002 (packet threshold + time threshold + PTO) |

## Examples

```bash
cargo run --example quic_echo_server
cargo run --example quic_echo_client
```

## References

- [QUIC Transport — RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [QUIC-TLS — RFC 9001](https://www.rfc-editor.org/rfc/rfc9001.html)
- [QUIC Loss Detection — RFC 9002](https://www.rfc-editor.org/rfc/rfc9002.html)
- [TLS 1.3 — RFC 8446](https://www.rfc-editor.org/rfc/rfc8446.html)
