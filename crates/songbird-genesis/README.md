# songbird-genesis

Physical genesis bootstrap for Songbird nodes with multi-primal witness coordination.

## Overview

This crate provides the infrastructure for securely birthing new Songbird nodes through physical proximity verification and multi-primal witness signatures.

**"Never let a bird be alone in the dark forest"** 🐦✨

## Features

- 🔐 Physical proximity verification (SoloKey, QR, Bluetooth)
- ✍️ Multi-primal witness coordination
- 🌳 Cryptographic lineage from birth
- 🔒 Trust level computation (Basic → Maximum)
- 📜 Genesis certificate generation
- 100% safe Rust (zero unsafe blocks)

## Usage

```rust
use songbird_genesis::{GenesisCeremony, GenesisConfig, PhysicalChannelType};

// Create genesis configuration
let config = GenesisConfig {
    new_node_id: "pixel-8a-device".to_string(),
    physical_channel: PhysicalChannelType::HardwareKey,
    witness_device_id: "solokey-123".to_string(),
    // ...
};

// Perform genesis ceremony
let ceremony = GenesisCeremony::new(config)?;
let certificate = ceremony.perform_genesis().await?;
```

## Architecture

See parent project documentation for complete architecture details.

## License

Licensed under AGPL-3.0-only as part of the ecoPrimals ecosystem.

Part of the scyBorg provenance trio: AGPL-3.0-only + ORC + CC-BY-SA 4.0
