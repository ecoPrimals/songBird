# 🦀 Songbird Pure Rust Bluetooth Stack - Universal Comms

**Date**: December 23, 2025  
**Vision**: Build our own pure Rust Bluetooth stack for universal deployment  
**Goal**: Songbird works **anywhere** - no system dependencies, true sovereignty

---

## 🎯 THE VISION

> **"Songbird needs to be universal comms on its own"**

### Why Build Our Own?

**Current Problem**:
- btleplug → depends on system Bluetooth stacks (BlueZ, WinRT, CoreBluetooth)
- bluer → depends on BlueZ (Linux only)
- bluest → depends on platform APIs
- **All require OS-level Bluetooth support**

**The Songbird Way**:
- ✅ **Pure Rust** - No system dependencies
- ✅ **Universal** - Works on any platform with USB/UART
- ✅ **Sovereign** - We control the entire stack
- ✅ **Embedded-ready** - Same code on desktop and ARM
- ✅ **Toadstool-compatible** - Leave ARM optimization to Toadstool

---

## 🏗️ ARCHITECTURE: PURE RUST BLE STACK

### The Stack (Bottom to Top)

```
┌─────────────────────────────────────────────────┐
│  Songbird Genesis (Physical Bootstrap)         │
│  - Device pairing                               │
│  - Secure exchange                              │
│  - Witness coordination                         │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  GATT Client/Server (Pure Rust)                │
│  - Service discovery                            │
│  - Characteristic read/write                    │
│  - Notifications                                │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  ATT Protocol (Pure Rust)                      │
│  - Attribute protocol                           │
│  - MTU negotiation                              │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  L2CAP (Pure Rust)                             │
│  - Logical link control                         │
│  - Packet fragmentation                         │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  HCI (Pure Rust)                               │
│  - Host Controller Interface                    │
│  - Command/Event handling                       │
└─────────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────────┐
│  Transport Layer (Pure Rust)                   │
│  - USB (via rusb)                              │
│  - UART (via serialport)                       │
│  - No kernel drivers needed!                    │
└─────────────────────────────────────────────────┘
```

---

## 🔍 EXISTING PURE RUST SOLUTIONS

### Option 1: **trouble-host** (Embassy Project) ⭐ BEST FIT

**What is it**: Pure Rust async BLE host stack from Embassy (embedded Rust framework)

**Key Features**:
- ✅ **100% Pure Rust** - No system dependencies
- ✅ **no_std compatible** - Works on embedded and desktop
- ✅ **Async/await** - Modern Rust patterns
- ✅ **Both roles** - Central (client) and Peripheral (server)
- ✅ **Full GATT** - Complete GATT client/server
- ✅ **Well-maintained** - Embassy project (active development)
- ✅ **Transport agnostic** - Works over USB, UART, SPI

**Crates**:
```toml
trouble-host = "0.5"           # Main BLE host stack
bt-hci = "0.6"                 # HCI data types
```

**Architecture**:
```
Your App (Songbird Genesis)
    ↓
trouble-host (Pure Rust BLE stack)
    ↓
bt-hci (HCI protocol)
    ↓
Your transport (USB/UART via rusb/serialport)
    ↓
Bluetooth controller (hardware)
```

**Example**:
```rust
use trouble_host::{Host, BleHost, BleHostResources};
use bt_hci::controller::Controller;

// Your USB/UART transport
struct MyTransport { /* ... */ }

impl Controller for MyTransport {
    // Implement HCI transport
}

async fn bluetooth_genesis() -> Result<()> {
    let transport = MyTransport::new()?;
    let mut resources = BleHostResources::new();
    let host = BleHost::new(transport, &mut resources);
    
    // Scan for devices
    host.scan().await?;
    
    // Connect to witness device
    let connection = host.connect(witness_addr).await?;
    
    // GATT exchange
    let data = connection.read_characteristic(genesis_char).await?;
    
    Ok(())
}
```

**Platforms**:
- ✅ Linux (any distro)
- ✅ Windows
- ✅ macOS
- ✅ BSD
- ✅ Embedded (ARM, RISC-V, etc.)
- ✅ **Anywhere with Bluetooth USB dongle or UART controller**

---

### Option 2: **burble** - Cross-Platform Userspace BLE

**What is it**: Pure Rust BLE stack starting from USB transport (via libusb)

**Key Features**:
- ✅ **Pure Rust** - Userspace implementation
- ✅ **USB-based** - Works with USB Bluetooth dongles
- ✅ **Cross-platform** - Windows, Linux tested
- ✅ **GATT server** - Peripheral role implemented
- ✅ **LE Secure Connections** - Modern pairing
- ⚠️ **Newer** - Less mature than trouble-host
- ⚠️ **libusb dependency** - But it's widely available

**Crates**:
```toml
burble = "0.2"
burble-crypto = "0.2"     # BLE crypto
burble-const = "0.2"      # BLE constants
```

**Use Case**: Good for desktop-only deployments with USB dongles

---

### Option 3: **Custom Implementation** - Full Control

**What is it**: Build the entire stack ourselves using HCI primitives

**Components**:
```toml
bt-hci = "0.6"            # HCI data types
bluetooth-hci = "0.1"     # HCI implementation
rusb = "0.9"              # USB transport
serialport = "4.0"        # UART transport
```

**Effort**: 6-12 months  
**Benefit**: Complete control, perfect fit for Songbird  
**Risk**: Significant development and testing effort

---

## 🎯 RECOMMENDED APPROACH: trouble-host

### Why trouble-host?

1. **Pure Rust, no_std** - Works everywhere
2. **Embassy backing** - Well-maintained, active community
3. **Production-ready** - Used in embedded products
4. **Transport agnostic** - USB, UART, SPI, anything
5. **Both roles** - Central and Peripheral
6. **Modern async** - Tokio-compatible

### Implementation Plan

#### Phase 1: USB Transport (Desktop) - 2-3 weeks

**Goal**: Songbird works on any desktop with USB Bluetooth dongle

```toml
[dependencies]
trouble-host = "0.5"
bt-hci = "0.6"
rusb = "0.9"              # USB transport
tokio = { version = "1", features = ["full"] }
```

**Architecture**:
```
Songbird Genesis
    ↓
trouble-host (BLE stack)
    ↓
USB HCI Transport (rusb)
    ↓
USB Bluetooth Dongle
```

**Code**:
```rust
// crates/songbird-genesis/src/transport/usb_hci.rs

use rusb::{Context, Device, DeviceHandle};
use bt_hci::controller::{Controller, ControllerCmdSync, ControllerCmdAsync};
use bt_hci::cmd::SyncCmd;
use bt_hci::event::Event;

pub struct UsbHciTransport {
    handle: DeviceHandle<Context>,
    // HCI endpoints
    cmd_endpoint: u8,
    evt_endpoint: u8,
    acl_in_endpoint: u8,
    acl_out_endpoint: u8,
}

impl UsbHciTransport {
    pub fn new() -> Result<Self> {
        let context = Context::new()?;
        
        // Find Bluetooth USB device (Class 0xE0 - Wireless Controller)
        for device in context.devices()?.iter() {
            let desc = device.device_descriptor()?;
            if desc.class_code() == 0xE0 {
                let handle = device.open()?;
                // Find HCI endpoints
                return Ok(Self {
                    handle,
                    cmd_endpoint: 0x00,      // Control endpoint
                    evt_endpoint: 0x81,      // Interrupt IN
                    acl_in_endpoint: 0x82,   // Bulk IN
                    acl_out_endpoint: 0x02,  // Bulk OUT
                });
            }
        }
        
        Err("No Bluetooth USB device found".into())
    }
    
    pub async fn send_command(&mut self, cmd: &[u8]) -> Result<()> {
        // Send HCI command via control transfer
        self.handle.write_control(
            0x20,  // Class request
            0x00,  // HCI command
            0,
            0,
            cmd,
            Duration::from_secs(1),
        )?;
        Ok(())
    }
    
    pub async fn read_event(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 256];
        let len = self.handle.read_interrupt(
            self.evt_endpoint,
            &mut buf,
            Duration::from_secs(1),
        )?;
        buf.truncate(len);
        Ok(buf)
    }
}

// Implement Controller trait for trouble-host
impl Controller for UsbHciTransport {
    // HCI command/event handling
}
```

**Benefits**:
- ✅ Works on any OS with USB
- ✅ No system Bluetooth stack needed
- ✅ $5-10 USB dongle is all you need
- ✅ Complete control

---

#### Phase 2: UART Transport (Embedded) - 1-2 weeks

**Goal**: Same code works on embedded devices (Raspberry Pi, etc.)

```toml
[dependencies]
serialport = "4.0"        # UART transport
```

**Use Case**: Embedded deployments, Raspberry Pi, custom hardware

---

#### Phase 3: Genesis Protocol - 2-3 weeks

**Goal**: Implement full genesis ceremony over pure Rust BLE

```rust
// crates/songbird-genesis/src/physical_channels/pure_bluetooth.rs

use trouble_host::{BleHost, BleHostResources, Connection};
use crate::{error::*, types::*};

pub struct PureRustBluetoothChannel {
    host: BleHost<UsbHciTransport>,
}

impl PureRustBluetoothChannel {
    pub async fn new() -> Result<Self> {
        let transport = UsbHciTransport::new()?;
        let mut resources = BleHostResources::new();
        let host = BleHost::new(transport, &mut resources);
        
        Ok(Self { host })
    }
    
    async fn scan_for_witness(&mut self) -> Result<Vec<DeviceInfo>> {
        // Scan for devices advertising genesis service
        let mut devices = Vec::new();
        
        self.host.scan().await?;
        
        // Filter for genesis witness devices
        // Look for specific service UUID
        
        Ok(devices)
    }
    
    async fn connect_to_witness(&mut self, addr: Address) -> Result<Connection> {
        let conn = self.host.connect(addr).await?;
        Ok(conn)
    }
    
    async fn exchange_genesis_data(&mut self, conn: &Connection) -> Result<Vec<u8>> {
        // GATT service discovery
        let services = conn.discover_services().await?;
        
        // Find genesis service
        let genesis_service = services.iter()
            .find(|s| s.uuid == GENESIS_SERVICE_UUID)
            .ok_or_else(|| GenesisError::PhysicalChannel("No genesis service".into()))?;
        
        // Read genesis characteristic
        let data = conn.read_characteristic(GENESIS_CHAR_UUID).await?;
        
        // Verify signature (via BearDog)
        // ...
        
        Ok(data)
    }
}

#[async_trait]
impl PhysicalChannelProvider for PureRustBluetoothChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // Scan and verify witness is nearby
        let devices = self.scan_for_witness().await?;
        
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::Bluetooth,
            timestamp: Utc::now(),
            proof_data: format!("Found {} witnesses", devices.len()).into_bytes(),
            attestation: Some("pure-rust-ble-stack".to_string()),
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // Connect and exchange genesis data
        let witness = self.scan_for_witness().await?.first()
            .ok_or_else(|| GenesisError::PhysicalChannel("No witness".into()))?;
        
        let conn = self.connect_to_witness(witness.address).await?;
        let data = self.exchange_genesis_data(&conn).await?;
        
        Ok(data)
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::High  // Physical proximity + crypto
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::Bluetooth
    }
}
```

---

## 📊 COMPARISON: System BLE vs Pure Rust Stack

| Feature | System BLE (btleplug) | Pure Rust Stack (trouble-host) |
|---------|----------------------|-------------------------------|
| **System deps** | ❌ Requires OS BLE | ✅ None |
| **Platforms** | 🟡 OS-dependent | ✅ Universal |
| **Embedded** | ❌ No | ✅ Yes |
| **USB dongle** | ❌ No | ✅ Yes |
| **Sovereignty** | 🟡 Partial | ✅ Complete |
| **Control** | 🟡 Limited | ✅ Full |
| **Complexity** | ✅ Simple | 🟡 Medium |
| **Maturity** | ✅ Very stable | ✅ Stable |
| **Toadstool compat** | 🟡 Via OS | ✅ Native |

---

## 🚀 IMPLEMENTATION TIMELINE

### Phase 1: USB Transport (3 weeks)
- Week 1: USB HCI transport implementation
- Week 2: Integration with trouble-host
- Week 3: Testing on Windows/Linux/macOS

### Phase 2: Genesis Protocol (3 weeks)
- Week 1: GATT service/characteristic design
- Week 2: Witness device scanning & connection
- Week 3: Secure data exchange & verification

### Phase 3: Testing & Hardening (2 weeks)
- Week 1: Multi-platform testing
- Week 2: Edge cases, error handling, docs

**Total: 8 weeks to production-ready pure Rust BLE**

---

## 💰 COST ANALYSIS

### Development Cost
- **8 weeks** @ 1 developer = 2 months
- **Complexity**: Medium (well-documented libraries)
- **Risk**: Low (trouble-host is production-ready)

### Benefits
- ✅ **Universal deployment** - Works anywhere
- ✅ **Zero system deps** - True sovereignty
- ✅ **Embedded-ready** - Toadstool integration
- ✅ **Complete control** - No black boxes
- ✅ **Future-proof** - We own the stack

### Hardware Cost
- USB Bluetooth dongle: **$5-10** per deployment
- Or built-in Bluetooth controller (embedded)

---

## 🎯 DECISION MATRIX

| Approach | Time | Sovereignty | Universal | Toadstool | Recommendation |
|----------|------|-------------|-----------|-----------|----------------|
| **Keep btleplug** | 0 weeks | 🟡 60% | ❌ No | 🟡 Via OS | ⚠️ Not universal |
| **Use bluest** | 1 week | 🟡 80% | 🟡 Desktop | 🟡 Limited | 🟡 Better but not enough |
| **Pure Rust (trouble-host)** | 8 weeks | ✅ 100% | ✅ Yes | ✅ Native | ⭐ **RECOMMENDED** |
| **Build from scratch** | 6-12 months | ✅ 100% | ✅ Yes | ✅ Native | ⚠️ Too long |

---

## ✅ RECOMMENDATION: Build Pure Rust Stack

### Why This is the Right Choice

1. **Aligns with Vision**: "Songbird needs to be universal comms"
2. **True Sovereignty**: No system dependencies, ever
3. **Toadstool-Ready**: Same code on ARM and x86
4. **Reasonable Timeline**: 8 weeks, not years
5. **Proven Technology**: trouble-host is production-ready
6. **Future-Proof**: We control our destiny

### What We Get

```
Songbird + $10 USB dongle = Universal Bluetooth
    ↓
Works on:
- Any Linux (no BlueZ needed)
- Any Windows (no WinRT needed)
- Any macOS (no CoreBluetooth needed)
- Raspberry Pi
- Custom embedded hardware
- Literally anywhere with USB
```

---

## 🚀 NEXT STEPS

### 1. Prototype (This Week)

```bash
cd /tmp
cargo new bluetooth-pure-test
cd bluetooth-pure-test

# Add dependencies
cargo add trouble-host bt-hci rusb tokio --features tokio/full

# Implement basic USB HCI transport
# Test device discovery
# Verify it works
```

### 2. Integrate (Next 2 Weeks)

```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Create new module
mkdir -p crates/songbird-genesis/src/transport
touch crates/songbird-genesis/src/transport/usb_hci.rs
touch crates/songbird-genesis/src/transport/uart_hci.rs

# Update Cargo.toml
```

### 3. Implement (Next 6 Weeks)

- USB transport
- GATT client
- Genesis protocol
- Testing
- Documentation

### 4. Deploy (Week 8)

- Release binaries
- USB dongle recommendations
- Platform testing
- Production deployment

---

## 📚 RESOURCES

**Crates**:
- trouble-host: https://crates.io/crates/trouble-host
- bt-hci: https://crates.io/crates/bt-hci
- rusb: https://crates.io/crates/rusb
- serialport: https://crates.io/crates/serialport

**Documentation**:
- Embassy docs: https://docs.embassy.dev/trouble-host
- trouble GitHub: https://github.com/embassy-rs/trouble
- Bluetooth Core Spec: https://www.bluetooth.com/specifications/specs/

**Hardware**:
- USB Bluetooth dongles (CSR, Realtek chipsets work well)
- UART Bluetooth modules (for embedded)

---

## 💡 KEY INSIGHTS

### 1. Pure Rust BLE is Ready

trouble-host proves that pure Rust BLE stacks are production-ready. We don't need to build from scratch.

### 2. USB Changes Everything

With USB transport, we bypass the OS Bluetooth stack entirely. This is the key to universal deployment.

### 3. Embedded-First = Desktop-Ready

Code that works on embedded (no_std) automatically works on desktop. This is the Rust superpower.

### 4. Toadstool Synergy

Pure Rust stack means Songbird and Toadstool speak the same language. No OS translation layer.

---

## ✅ CONCLUSION

**Yes, we should build our own pure Rust Bluetooth stack!**

**Why**:
- ✅ Aligns with Songbird's universal comms vision
- ✅ True sovereignty (zero system deps)
- ✅ Reasonable timeline (8 weeks)
- ✅ Proven technology (trouble-host)
- ✅ Toadstool-ready (same code everywhere)

**How**:
- Use trouble-host (pure Rust BLE stack)
- USB transport via rusb
- UART transport for embedded
- 8 weeks to production

**Result**:
```
Songbird = Universal Comms
    ↓
Works anywhere with:
- USB Bluetooth dongle ($10)
- Or built-in Bluetooth controller
- No OS Bluetooth stack needed
- True primal sovereignty
```

**Let's build it!** 🚀

---

**Updated**: December 23, 2025  
**Status**: Plan complete, ready to prototype  
**Timeline**: 8 weeks to production  
**Next**: Prototype USB HCI transport

🦀 Pure Rust, Universal Comms, True Sovereignty!

