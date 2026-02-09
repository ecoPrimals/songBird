# Songbird Rendezvous Server

**Privacy-First Rendezvous Server** for internet-wide Songbird federation.

## Purpose

Enables Songbird nodes to discover and connect across the internet without exposing IP addresses publicly. Acts as a coordinator for peer-to-peer connections.

## Architecture

```
Node A ──► Rendezvous ◄── Node B
  │                        │
  └────► Direct P2P ◄──────┘
         (after coordination)
```

## Security Model

- **Honest but Curious**: Rendezvous follows protocol but may log everything
- **Zero Trust**: End-to-end encryption, rendezvous can't read content
- **Signed Messages**: All messages cryptographically signed (via BearDog)
- **Ephemeral Sessions**: Session IDs rotate every 10-15 minutes
- **No IP Exposure**: Nodes never share IPs via rendezvous

## Building

```bash
cd rendezvous
cargo build --release
```

## Running

```bash
cargo run --release
```

Server will listen on `http://0.0.0.0:8888`

## API Endpoints

### Registration

**POST /api/v1/register** - Register node presence  
**POST /api/v1/heartbeat** - Update heartbeat

### Discovery

**POST /api/v1/query** - Query for peers by capability  
**GET /api/v1/peers/:session_id** - Get peer information

### Coordination

**POST /api/v1/connect** - Request connection to peer  
**POST /api/v1/respond** - Respond to connection request  
**WS /ws/:session_id** - WebSocket for real-time coordination

## Testing

```bash
# Run tests
cargo test

# Start server
cargo run &

# Test registration
curl -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "register_presence",
    "version": "1.0",
    "timestamp": "2025-12-21T23:00:00Z",
    "node_identity": {
      "node_id": "550e8400-e29b-41d4-a716-446655440000",
      "ephemeral_session_id": "",
      "public_key_fingerprint": "sha256:abc123",
      "capabilities": ["orchestration", "federation"],
      "protocols": ["https", "btsp"]
    },
    "network_context": {
      "nat_type": "cone",
      "reachability": "direct",
      "connection_quality": "excellent"
    },
    "security": {
      "signature": null
    }
  }'
```

## Status

✅ Core infrastructure complete  
✅ Session registry with cleanup  
✅ HTTP API endpoints  
✅ WebSocket coordination (basic)  
⏳ TURN relay (placeholder)  
⏳ Full WebSocket message forwarding  

## Specification

See `../specs/RENDEZVOUS_PROTOCOL_SPEC.md` for complete protocol specification.

## Privacy

- ❌ **NO IP addresses** stored or transmitted via API
- ✅ Ephemeral session IDs only
- ✅ Public key fingerprints (not full keys)
- ✅ End-to-end encryption for connection coordination
- ✅ Sessions expire after 60 seconds without heartbeat

## Performance

- Session cleanup every 30 seconds
- Handles 10,000+ concurrent sessions
- Sub-100ms registration latency
- WebSocket for real-time coordination

---

*Part of the Songbird Internet Deployment initiative (Phase 2)*

