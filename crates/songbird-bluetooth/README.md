# songbird-bluetooth

**Status**: 🚧 **EXPERIMENTAL - Phase 2**

## Overview

Pure Rust Bluetooth LE stack for Songbird Genesis physical channels. This crate provides zero-dependency Bluetooth communication for sovereign device pairing and out-of-band verification.

## Current Status

This crate is **under active development** for Phase 2 (Genesis Implementation):

- ✅ Core architecture designed
- ✅ Basic HCI/GATT implementation
- ⚠️ **Not production-ready** - has known clippy violations
- ⏳ Pending full evolution to idiomatic Rust
- ⏳ Pending comprehensive test coverage

## Evolution Plan

See `REMAINING_WORK.md` at repository root for the current roadmap:

### Phase 2 Goals (Weeks 7-10):
1. Fix all 63+ clippy violations
2. Evolve to modern idiomatic Rust
3. Complete GATT client implementation
4. Add comprehensive test suite
5. Integrate with Genesis ceremony

### Current Architecture:

- **Transport Layer**: USB (HCI), UART (HCI)
- **Host Stack**: trouble-host (Embassy Bluetooth)
- **Protocols**: ATT, GATT, L2CAP
- **Zero unsafe code**: Memory-safe throughout

## Usage (When Ready)

```rust
// Example Genesis ceremony over Bluetooth (Phase 2)
use songbird_bluetooth::{BluetoothTransport, GattClient};

async fn genesis_over_bluetooth() -> Result<Credentials> {
    let transport = BluetoothTransport::usb().await?;
    let gatt = GattClient::new(transport).await?;
    
    // Discover Genesis service
    let services = gatt.discover_services().await?;
    
    // Read credential characteristic
    let creds = gatt.read_genesis_credential().await?;
    
    Ok(creds)
}
```

## Contributing

This crate follows the ecoPrimals philosophy:

- ✅ **Deep debt solutions** - not quick fixes
- ✅ **Modern idiomatic Rust** - latest best practices
- ✅ **Zero unsafe code** - fast AND safe
- ✅ **Capability-based** - runtime discovery only
- ✅ **Sovereignty-preserving** - user agency first

## License

Licensed under AGPL-3.0-or-later as part of the ecoPrimals ecosystem.

Part of the scyBorg provenance trio: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0
