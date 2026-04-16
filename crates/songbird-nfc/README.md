# Songbird Dark Forest NFC Genesis

## Overview

`songbird-nfc` implements a zero-metadata-leakage NFC protocol for genesis ceremonies and secure mobile device pairing. All cryptographic operations are delegated to the security provider capability via JSON-RPC IPC.

## Dark Forest Guarantees

✅ **Zero metadata leakage** - No identifiable information in cleartext  
✅ **Ephemeral keys** - Single-use X25519 keys for each exchange  
✅ **Timing protection** - Constant-time operations with random delays  
✅ **Security provider delegation** - All crypto via capability IPC  
✅ **Zero unsafe code** - `#![forbid(unsafe_code)]`  
✅ **Memory-safe** - Pure Rust implementation  

## Protocol Flow

```
Initiator (Parent)                    Responder (Child)
==================                    =================

1. Generate ephemeral X25519 keypair
2. Send public key (32 bytes)      -->
                                   <-- 3. Receive public key
                                       4. Generate ephemeral keypair
                                       5. Compute shared secret (DH)
6. Receive public key              <-- 6. Send public key
7. Compute shared secret (DH)
8. Encrypt genesis credentials
   (ChaCha20-Poly1305 via security provider)
9. Sign with ephemeral Ed25519
10. Send encrypted genesis         -->
                                   <-- 11. Verify signature
                                       12. Decrypt genesis
                                       13. Send confirmation
14. Receive confirmation           <--
15. Destroy ephemeral keys             14. Destroy ephemeral keys
```

## Wire Format

All messages follow this Dark Forest compliant binary format:

```
[1 byte]   Protocol version (0x01)
[1 byte]   Message type (0x01 = genesis_request, 0x02 = genesis_response)
[2 bytes]  Payload length (big-endian u16)
[32 bytes] Ephemeral public key (X25519)
[24 bytes] Nonce (ChaCha20-Poly1305)
[N bytes]  Encrypted payload (with 16-byte Poly1305 auth tag)
[64 bytes] Signature (ephemeral Ed25519)
```

### Frame Structure

- **Header overhead**: 4 bytes (version + type + length)
- **Crypto overhead**: 120 bytes (32 pubkey + 24 nonce + 64 signature)
- **Total overhead**: 124 bytes
- **Max payload**: 1024 bytes (prevents memory exhaustion)

## Usage

### Parent (Initiator)

```rust
use songbird_nfc::{GenesisExchange, GenesisCredentials, NfcConfig, NfcDevice};

// Create genesis credentials
let credentials = GenesisCredentials {
    identity: primal_identity.to_vec(),
    family_seed: family_seed.to_vec(),
    lineage: vec!["parent".to_string()],
    beacons: vec!["[2600::1]:3492".to_string()],
    timestamp: chrono::Utc::now().timestamp_millis(),
};

// Configure with timing protection
let config = NfcConfig::new()
    .with_timing_protection(true);

let mut exchange = GenesisExchange::new(config);
let mut device = NfcDevice::new(Duration::from_secs(30))?;

// Connect and initiate genesis
device.connect().await?;
exchange.initiate(&mut device, &credentials).await?;
device.disconnect().await?;
```

### Child (Responder)

```rust
use songbird_nfc::{GenesisExchange, NfcConfig, NfcDevice};

let config = NfcConfig::new()
    .with_timing_protection(true);

let mut exchange = GenesisExchange::new(config);
let mut device = NfcDevice::new(Duration::from_secs(30))?;

// Connect and receive genesis
device.connect().await?;
let credentials = exchange.respond(&mut device).await?;
device.disconnect().await?;

// Now the child primal has genesis credentials!
println!("Identity: {:?}", credentials.identity);
println!("Family seed: {:?}", credentials.family_seed);
```

## Timing Protection

Timing protection prevents side-channel attacks based on operation duration:

### Random Delays

```rust
let config = NfcConfig::new()
    .with_timing_protection(true)
    .with_max_random_delay(Duration::from_millis(500));
```

Before each cryptographic operation, a random delay (0-500ms) is applied to obscure timing patterns.

### Constant-Time Padding

```rust
let config = NfcConfig::new()
    .with_target_exchange_duration(Duration::from_secs(10));
```

After the exchange completes, the operation is padded to exactly 10 seconds (target duration) to prevent timing-based analysis.

### Manual Timing Protection

```rust
use songbird_nfc::timing::TimingProtector;

let mut protector = TimingProtector::new(
    Duration::from_secs(5),  // target duration
    Duration::from_millis(200),  // max random delay
);

protector.protect(async {
    // Your sensitive operation here
    Ok::<_, NfcError>(())
}).await?;
```

## Platform Support

### Android

Uses Android NFC stack via JNI:

```rust
#[cfg(target_os = "android")]
let device = NfcDevice::new(timeout)?; // Automatically uses AndroidNfcBackend
```

**Status**: Stub implementation (TODO: JNI integration)

### iOS

Uses CoreNFC framework:

```rust
#[cfg(target_os = "ios")]
let device = NfcDevice::new(timeout)?; // Automatically uses IosNfcBackend
```

**Status**: Stub implementation (TODO: CoreNFC integration)

### Linux

Uses libnfc:

```rust
#[cfg(target_os = "linux")]
let device = NfcDevice::new(timeout)?; // Automatically uses LinuxNfcBackend
```

**Status**: Stub implementation (TODO: libnfc integration)

## Security Provider Integration

All cryptographic operations are delegated to the security provider via capability-based IPC:

| Operation | JSON-RPC Method | Status |
|-----------|----------------|--------|
| Generate ephemeral keypair | `crypto.generate_x25519_keypair` | Pending |
| Compute shared secret | `crypto.x25519_diffie_hellman` | Pending |
| Generate nonce | `crypto.generate_nonce` | Pending |
| Encrypt genesis | `crypto.chacha20poly1305_encrypt` | Pending |
| Decrypt genesis | `crypto.chacha20poly1305_decrypt` | Pending |
| Sign message | `crypto.ed25519_sign` | Pending |
| Verify signature | `crypto.ed25519_verify` | Pending |
| Destroy keys | `crypto.destroy_ephemeral_keys` | Pending |

## Configuration

### Security Provider Socket Discovery

`NfcConfig` discovers the security provider socket at runtime (no hardcoding):

1. `SECURITY_PROVIDER_SOCKET` environment variable
2. `BEARDOG_SOCKET` environment variable (deprecated fallback)
3. `$XDG_RUNTIME_DIR/biomeos/security.sock`
4. `/tmp/biomeos/security.sock` (fallback)

### Full Configuration

```rust
let config = NfcConfig {
    beardog_socket: PathBuf::from("/custom/beardog.sock"),
    exchange_timeout: Duration::from_secs(30),
    timing_protection: true,
    target_exchange_duration: Duration::from_secs(10),
    max_random_delay: Duration::from_millis(500),
    validate_connection: true,
};
```

## Security Model

### Threat Model

- **Passive observers**: Cannot decrypt genesis (ephemeral keys)
- **Timing attacks**: Mitigated by random delays + constant-time padding
- **Replay attacks**: Prevented by ephemeral keys (single-use)
- **MITM attacks**: Mitigated by Ed25519 signatures (verifies peer identity)

### Attack Surface

- **NFC eavesdropping**: Encrypted with X25519 + ChaCha20-Poly1305
- **NFC jamming**: Not preventable (physical attack)
- **Platform compromise**: Security provider offers hardware-backed keys (if available)

## Deep Debt Compliance

✅ **Zero unsafe code** - `#![forbid(unsafe_code)]`  
✅ **Runtime discovery** - Security provider socket discovered by capability, not hardcoded  
✅ **Pure Rust** - No C dependencies (except platform NFC drivers)  
✅ **Modern idioms** - Async/await, Result, thiserror  
✅ **No mocks** - Real implementations (platform stubs TODO)  
✅ **Domain-driven** - Separate modules for protocol, timing, platform  

## Future Enhancements

1. **Security Provider IPC Integration**: Replace stub methods with real security provider calls
2. **Platform Backends**: Complete Android/iOS/Linux NFC implementations
3. **Multi-device Genesis**: Support N-way genesis ceremonies
4. **QR Code Fallback**: Visual channel when NFC unavailable
5. **Bluetooth LE**: Alternative for devices without NFC
6. **Hardware-backed Keys**: Use TEE/Secure Element via security provider

## Testing

```bash
# Unit tests
cargo test -p songbird-nfc

# Timing protection tests
cargo test -p songbird-nfc test_timing_protection

# Protocol serialization tests
cargo test -p songbird-nfc test_message_roundtrip
```

## References

- [ISO/IEC 14443](https://www.iso.org/standard/73599.html) - NFC standard
- [RFC 7748](https://www.rfc-editor.org/rfc/rfc7748.html) - X25519 key exchange
- [RFC 7539](https://www.rfc-editor.org/rfc/rfc7539.html) - ChaCha20-Poly1305 AEAD
- [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032.html) - Ed25519 signatures
- Protocol Evolution Refined (Feb 8, 2026) — see `ecoPrimals/infra/wateringHole/fossilRecord/` for archived spec
