# 🧅 Songbird Sovereign Onion

**Pure Rust minimal onion service protocol** for sovereign device-to-device communication.

## Overview

Provides cryptographically-derived `.onion` addresses for reachability across NAT without port forwarding. Inspired by Tor v3 onion services but simplified for family mesh use cases.

## Features

- ✅ **100% Pure Rust** - Zero C dependencies
- ✅ **Ed25519 Identity** - Cryptographic device IDs
- ✅ **X25519 Key Exchange** - Forward secrecy
- ✅ **ChaCha20-Poly1305** - Fast AEAD encryption
- ✅ **IPC Storage** - Identity and peer storage via JSON-RPC `storage.*` capability
- ✅ **Minimal Protocol** - ~10% of Tor complexity

## NOT Tor

This is **not** a full Tor implementation:
- ❌ No directory authorities
- ❌ No consensus documents
- ❌ No onion routing (3 hops)
- ❌ No anonymity guarantees

**Goal**: Reachable addresses, not anonymity

## Usage

```rust
use songbird_sovereign_onion::{OnionService, OnionConnector};

// Create onion service
let service = OnionService::new(9735).await?;
println!("Address: {}", service.onion_address());

// Accept connections
while let Ok(conn) = service.accept().await {
    // Handle encrypted connection
    conn.write(b"Hello from onion service").await?;
}

// Connect to onion address
let connector = OnionConnector::new();
let conn = connector.connect("vww6ybal...npyyd.onion", 9735).await?;
conn.write(b"Hello").await?;
```

## Architecture

```
Device ID → Ed25519 Identity → .onion Address
                 ↓
           X25519 Ephemeral Keys (per session)
                 ↓
           HKDF Session Keys
                 ↓
           ChaCha20-Poly1305 Encryption
```

## License

AGPL-3.0

## Status

**Phase 1**: ✅ Identity and Addressing (Complete)  
**Phase 2**: 🚧 Protocol Implementation (In Progress)  
**Phase 3**: ⚠️ Service Mode (Pending)  
**Phase 4**: ⚠️ Connector Mode (Pending)  
**Phase 5**: ✅ IPC Storage (sled removed Wave 135, replaced with JSON-RPC `storage.*` capability)
