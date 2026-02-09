# CREATE2 Cell Debug Status - February 7, 2026

## Problem
Tor relays are not responding to our CREATE2 cells. The link protocol completes successfully, but circuit creation fails with timeout.

## Environment
- **System Tor Version**: 0.4.6.10 (WORKS)
- **Our Implementation**: songbird-tor-protocol (pure Rust)
- **TLS Library**: rustls 0.21 + tokio-rustls 0.24

## What Works
1. ✅ TLS connection to relays
2. ✅ VERSIONS cell exchange
3. ✅ CERTS/AUTH_CHALLENGE reception
4. ✅ NETINFO exchange (bidirectional)
5. ✅ Connection stays open after CREATE2
6. ✅ System Tor client on same machine

## What Fails
- ❌ No CREATED2 response from ANY relay
- ❌ Multiple relays (lisdex, titamon3, StarAppsMobley, etc.) all fail the same way

## Verified Cell Format
```
CREATE2 cell (512 bytes):
[0-3]   CircID:  0x80000001 (MSB set for client-initiated)
[4]     Command: 0x0a (CREATE2 = 10)
[5-6]   HTYPE:   0x0002 (ntor handshake)
[7-8]   HLEN:    0x0054 (84 bytes)
[9-28]  node_id: 20 bytes (relay fingerprint from consensus)
[29-60] B:       32 bytes (relay ntor_key from descriptor)
[61-92] X:       32 bytes (client ephemeral X25519 pubkey)
[93-511] padding: zeros
```

## Verified Values
```
For relay "lisdex":
- Fingerprint: 000004ACBB9D29BCBA17256BB35928DDBFC8ABA9 ✓
- ntor_key: hUl33v3ViBHTnoHEe5VxFkhbjyuxDju8H9A2LPuZIis ✓
- Client X25519 key: valid 32-byte key from BearDog ✓
```

## Debug Findings
1. System Tor bootstraps to 100% on same network
2. Our TLS uses TLS 1.2/1.3 with safe defaults
3. Link protocol version 4 negotiated
4. We receive 2 mystery bytes `[00, 00]` after NETINFO sometimes
5. Connection remains writable after CREATE2 (can send PADDING)
6. 30+ second timeout with no relay response

## Theories
1. **TLS Session Issue**: Maybe relays check something about TLS we're not setting
2. **CERTS Processing**: We skip CERTS validation - maybe needed
3. **Timing**: Maybe specific timing patterns required
4. **Padding Negotiation**: Link v5 has padding, maybe v4 relays expect something

## Next Steps
1. Compare with Arti source code for differences
2. Packet capture analysis between working tor and our client
3. Check if CERTS cell validation is required
4. Test with Tor relay in debug mode

## Files for Reference
- `src/connection/link.rs` - Link protocol implementation
- `src/connection/tls.rs` - TLS configuration
- `src/protocol/cells.rs` - Cell encoding/decoding
- `examples/raw_test.rs` - Debug test harness
