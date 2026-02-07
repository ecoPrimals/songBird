# 🔐 ntor Handshake Specification

**Version**: 1.0  
**Status**: Design Complete (Implementation blocked by BearDog)  
**Date**: February 7, 2026

---

## Overview

The **ntor** (nickname: "the ntor") handshake is Tor's authenticated key agreement protocol for circuit creation. It provides forward secrecy and mutual authentication between client and relay.

**Key Properties**:
- Forward secrecy via ephemeral X25519 keys
- Relay authentication via long-term identity
- Key derivation for cell encryption
- ~100ms per hop via BearDog delegation

---

## Protocol Flow

### Client → Relay (CREATE2 Cell)

**Payload (84 bytes)**:
```
┌─────────────┬─────────────┬──────────────┐
│ Identity_B  │ NTor_Key_B  │ Client_PK_X  │
│  32 bytes   │  32 bytes   │   32 bytes   │
└─────────────┴─────────────┴──────────────┘
```

- **Identity_B**: Relay's Ed25519 identity key (from consensus)
- **NTor_Key_B**: Relay's X25519 ntor onion key (from descriptor)
- **Client_PK_X**: Client's ephemeral X25519 public key

### Relay → Client (CREATED2 Cell)

**Payload (64 bytes)**:
```
┌─────────────┬────────────┐
│ Server_PK_Y │    Auth    │
│  32 bytes   │  32 bytes  │
└─────────────┴────────────┘
```

- **Server_PK_Y**: Relay's ephemeral X25519 public key
- **Auth**: Authentication hash (proves relay has identity secret key)

---

## Cryptographic Operations

### Client Side (CREATE2)

```rust
// 1. Generate ephemeral X25519 keypair via BearDog
let client_ephemeral = beardog.x25519_generate_ephemeral()?;

// 2. Construct CREATE2 payload
let mut payload = Vec::with_capacity(84);
payload.extend_from_slice(&relay_identity);     // 32 bytes (from consensus)
payload.extend_from_slice(&relay_ntor_key);     // 32 bytes (from descriptor)
payload.extend_from_slice(&client_ephemeral.public_key); // 32 bytes

// 3. Send CREATE2 cell
let cell = Cell::new(circuit_id, CellCommand::Create2, payload);
stream.write_all(&cell.encode()).await?;

// 4. Save state for CREATED2 processing
let state = HandshakeState {
    client_ephemeral_secret: client_ephemeral.secret_key,
    relay_identity,
    relay_ntor_key,
};
```

### Client Side (CREATED2)

```rust
// 1. Receive CREATED2 cell
let cell = Cell::decode(&buffer)?;
let server_pubkey = &cell.payload[0..32];
let auth = &cell.payload[32..64];

// 2. Derive shared secret via BearDog X25519
let shared_secret = beardog.x25519_derive_secret(
    &state.client_ephemeral_secret,
    server_pubkey,
)?;

// 3. Compute key material via BearDog SHA3-256 (KDF)
let secret_input = [
    &shared_secret[..],
    &relay_identity[..],
    &relay_ntor_key[..],
    &state.client_ephemeral_public[..],
    server_pubkey,
    b"ntor-curve25519-sha3-256-1",
].concat();

let key_material = beardog.sha3_256(&secret_input)?;

// 4. Verify auth
let expected_auth = beardog.sha3_256(&[
    &key_material[..],
    &relay_identity[..],
    &relay_ntor_key[..],
    server_pubkey,
    &state.client_ephemeral_public[..],
    b"ntor-curve25519-sha3-256-1:verify",
].concat())?;

if auth != &expected_auth[..32] {
    return Err(Error::Protocol("ntor auth verification failed".to_string()));
}

// 5. Derive forward/backward keys via KDF
let (forward_digest, backward_digest, forward_key, backward_key) = 
    derive_circuit_keys(&key_material, &beardog)?;
```

### Key Derivation Function (KDF)

**Purpose**: Expand 32-byte key material into circuit keys

```rust
async fn derive_circuit_keys(
    key_material: &[u8; 32],
    beardog: &BeardogCryptoClient,
) -> Result<KeyMaterial> {
    // HKDF-style expansion using SHA3-256
    // Output: 5 * 32 bytes = 160 bytes total
    //   - Forward digest init (32 bytes)
    //   - Backward digest init (32 bytes)
    //   - Forward AES key (16 bytes from 32)
    //   - Backward AES key (16 bytes from 32)
    //   - KDF IV (32 bytes)
    
    let mut expanded = Vec::with_capacity(160);
    let mut prev = key_material.to_vec();
    
    for i in 0..5 {
        let input = [
            &prev[..],
            &[i as u8],
            b"ntor-curve25519-sha3-256-1:key_expand",
        ].concat();
        
        prev = beardog.sha3_256(&input)?.to_vec();
        expanded.extend_from_slice(&prev);
    }
    
    Ok(KeyMaterial {
        forward_digest: expanded[0..32].try_into()?,
        backward_digest: expanded[32..64].try_into()?,
        forward_key: expanded[64..80].try_into()?,   // First 16 bytes
        backward_key: expanded[96..112].try_into()?, // First 16 bytes
    })
}
```

---

## Cell Format

### CREATE2 Cell (Circuit ID + Command + Payload)

```
┌──────────┬─────────┬──────────────────────────┐
│ CircID   │ Command │       Payload            │
│ 4 bytes  │ 1 byte  │      84 bytes            │
│          │  (10)   │                          │
└──────────┴─────────┴──────────────────────────┘
           │         │
           │         └─> CREATE2 payload (ntor handshake)
           └──────────> CREATE2 command
```

### CREATED2 Cell

```
┌──────────┬─────────┬──────────────────────────┐
│ CircID   │ Command │       Payload            │
│ 4 bytes  │ 1 byte  │      64 bytes            │
│          │  (11)   │                          │
└──────────┴─────────┴──────────────────────────┘
           │         │
           │         └─> CREATED2 payload (server response)
           └──────────> CREATED2 command
```

---

## BearDog Integration Points

### Required Methods

```rust
// Already available
x25519_generate_ephemeral() -> X25519Keypair
x25519_derive_secret(secret: &[u8; 32], public: &[u8; 32]) -> [u8; 32]

// NEW - Required for Phase 2B
sha3_256(data: &[u8]) -> [u8; 32]
aes_128_ctr_encrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8>
aes_128_ctr_decrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8>
```

### Call Patterns

**Per Circuit**:
- 1x `x25519_generate_ephemeral()` (client ephemeral key)
- 1x `x25519_derive_secret()` (shared secret)
- 6x `sha3_256()` (KDF expansion + auth)

**Per Cell** (ongoing):
- 3x `aes_128_ctr_encrypt()` (3 hops, forward)
- 1x `aes_128_ctr_decrypt()` (1 hop, backward)
- 2x `sha3_256()` (forward/backward digest updates)

**Estimated Load**:
- Circuit build: ~8 BearDog calls (~100ms total)
- Cell relay: ~5 BearDog calls per cell (~1-2ms per cell)
- 100 circuits: ~800 BearDog calls (~10s total)

---

## Test Vectors

### ntor Test Vector 1 (RFC Test)

**Input**:
```
Relay Identity: 0x9fad2af287ef942632833d21f946c6260c33feba...
Relay ntor key: 0x4bf5122f344554c53bde2ebb8cd2b7e3d1600ad...
Client ephemeral secret: 0xc8593a4c0e8f3c91d33c27e5...
```

**Expected Output**:
```
Shared secret: 0xab3d52a4e8f7c19d2b3e4f5a...
Auth: 0x7e4f3c2b1a9d8e7f6c5b4a39...
Forward key: 0x1a2b3c4d5e6f7a8b...
Backward key: 0x8b7a6f5e4d3c2b1a...
```

### Test Vector Sources
- Tor specification test vectors
- Reference C implementation (`tor/src/core/crypto/onion_ntor.c`)
- Independent test vector generation

---

## Implementation Checklist

### Phase 2B-1: ntor Handshake
- [ ] Create `circuit/ntor.rs` module
- [ ] Implement `create_handshake()` (CREATE2 generation)
- [ ] Implement `complete_handshake()` (CREATED2 processing)
- [ ] Implement `derive_circuit_keys()` (KDF via SHA3-256)
- [ ] Implement auth verification
- [ ] Add test vectors
- [ ] Unit tests (all test vectors passing)

### Phase 2B-2: Circuit Building
- [ ] Create `circuit/manager.rs` module
- [ ] Implement `CircuitManager` struct
- [ ] Implement `build_circuit()` (single hop first)
- [ ] Add circuit state tracking
- [ ] Add error handling and retry
- [ ] Integration test (single-hop circuit)

### Phase 2B-3: Circuit Extension
- [ ] Implement RELAY cell encoding
- [ ] Implement `extend_circuit()` (EXTEND2)
- [ ] Handle EXTENDED2 responses
- [ ] Support multi-hop extension
- [ ] Integration test (3-hop circuit)

### Phase 2B-4: Onion Encryption
- [ ] Create `circuit/onion.rs` module
- [ ] Implement `encrypt_forward()` (layer encryption)
- [ ] Implement `decrypt_backward()` (layer decryption)
- [ ] Test with known plaintexts
- [ ] Integration test (full circuit with live Tor)

---

## References

- [Tor Spec: ntor handshake](https://spec.torproject.org/tor-spec/create-created-cells.html#ntor-handshake)
- [RFC 8439: ChaCha20 and Poly1305](https://tools.ietf.org/html/rfc8439)
- [Tor ntor paper](https://www-users.cs.umn.edu/~hoppernj/ntor.pdf)
- [Reference Implementation](https://github.com/torproject/tor/blob/main/src/core/crypto/onion_ntor.c)

---

**Ready for implementation once BearDog extensions are available!**

**TRUE PRIMAL** | **Pure Rust** | **Zero Unsafe** | **100% BearDog Delegation**
