# 🦀 Pure Rust Bluetooth Implementation Plan

**Date**: December 23, 2025  
**Goal**: Replace btleplug → dbus with 100% Pure Rust Bluetooth  
**Status**: Research complete, implementation ready

---

## 🎯 THE PURE RUST SOLUTION

### Option 1: **bluer** - Official BlueZ Pure Rust Bindings ⭐ RECOMMENDED

**What is it**: Official Rust interface to Linux Bluetooth stack (BlueZ) via **zbus** (pure Rust D-Bus)

**Key Details**:
- ✅ **100% Pure Rust** - Uses `zbus` instead of `dbus` C bindings
- ✅ **Official** - Maintained by BlueZ project
- ✅ **Modern** - Async/await, tokio-based
- ✅ **Well-maintained** - v0.17.4 (actively developed)
- ✅ **Full-featured** - GATT, L2CAP, RFCOMM support
- ⚠️ **Linux-only** - Designed for BlueZ (Linux Bluetooth stack)

**Crate**: `bluer = "0.17"`

**Architecture**:
```
songbird-genesis
  └── bluer = "0.17"
      └── zbus = "4.x"  (pure Rust D-Bus)
          └── rustix (pure Rust POSIX)
              └── No C dependencies! ✅
```

**Example Code**:
```rust
use bluer::{Adapter, AdapterEvent};
use futures::stream::StreamExt;

async fn scan_bluetooth_devices() -> bluer::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    
    // Start discovery
    adapter.set_powered(true).await?;
    adapter.set_discovery_filter(Default::default()).await?;
    
    let mut discover = adapter.discover_devices().await?;
    while let Some(evt) = discover.next().await {
        match evt {
            AdapterEvent::DeviceAdded(addr) => {
                println!("Found device: {}", addr);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

---

### Option 2: **bluest** - Cross-Platform Pure Rust BLE

**What is it**: Cross-platform BLE library with pure Rust implementations per platform

**Key Details**:
- ✅ **100% Pure Rust** on all platforms
- ✅ **Cross-platform** - Windows, macOS, Linux, iOS, Android
- ✅ **Modern** - Async/await, futures-based
- ✅ **Active** - Regular updates
- ⚠️ **Newer** - Less mature than btleplug
- ⚠️ **Different API** - Not compatible with btleplug

**Crate**: `bluest = "0.5"`

**Platform Backends**:
| Platform | Backend | Pure Rust? |
|----------|---------|------------|
| Linux | BlueZ via D-Bus | ✅ Yes (via zbus) |
| Windows | WinRT | ✅ Yes |
| macOS/iOS | CoreBluetooth | ✅ Yes |
| Android | Android BLE | ✅ Yes |

**Example Code**:
```rust
use bluest::{Adapter, Device};

async fn scan_bluetooth() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = Adapter::default().await?;
    adapter.wait_available().await?;
    
    let mut scan = adapter.scan(&[]).await?;
    while let Some(discovered_device) = scan.next().await {
        println!("Found: {:?}", discovered_device.device.name().await?);
    }
    
    Ok(())
}
```

---

### Option 3: **btleplug with zbus backend** (Future)

**What is it**: Contribute pure Rust backend to btleplug

**Status**: Would require upstream contribution to `bluez-async` crate

**Effort**: 2-4 weeks + upstream acceptance  
**Benefit**: Helps entire Rust ecosystem  
**Risk**: Depends on maintainer acceptance

---

## 📊 COMPARISON MATRIX

| Feature | bluer | bluest | btleplug (current) |
|---------|-------|--------|-------------------|
| **Pure Rust** | ✅ Yes | ✅ Yes | ❌ No (Linux) |
| **Cross-platform** | ❌ Linux only | ✅ All platforms | ✅ All platforms |
| **Maturity** | ✅ Stable | 🟡 Good | ✅ Very stable |
| **API Quality** | ✅ Excellent | ✅ Good | ✅ Excellent |
| **Maintenance** | ✅ Official BlueZ | ✅ Active | ✅ Active |
| **Documentation** | ✅ Good | 🟡 Growing | ✅ Excellent |
| **Zero system deps** | ✅ Yes | ✅ Yes | ❌ No (Linux) |
| **Performance** | ✅ Native | ✅ Native | ✅ Native |

---

## 🎯 RECOMMENDED SOLUTION: Hybrid Approach

### Strategy: Platform-Specific Pure Rust

Use the **best pure Rust solution per platform**:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
bluer = { version = "0.17", optional = true }

[target.'cfg(not(target_os = "linux"))'.dependencies]
bluest = { version = "0.5", optional = true }

[features]
bluetooth = ["bluer", "bluest"]
```

**Why Hybrid?**
- ✅ **Linux**: `bluer` is official, mature, pure Rust
- ✅ **Windows/macOS**: `bluest` provides unified API
- ✅ **100% Pure Rust** on all platforms
- ✅ **Best of both worlds**

**Alternative: bluest everywhere**
```toml
[dependencies]
bluest = { version = "0.5", optional = true }

[features]
bluetooth = ["bluest"]
```

**Why bluest-only?**
- ✅ **Simpler**: One API for all platforms
- ✅ **Pure Rust**: Everywhere
- ✅ **Unified**: Same code paths
- 🟡 **Newer**: Less battle-tested than bluer on Linux

---

## 🚀 IMPLEMENTATION PLAN

### Phase 1: Research & Prototype (1 week)

**Week 1: Evaluate both libraries**

```bash
# Create test project
cd /tmp
cargo new bluetooth-test
cd bluetooth-test

# Test bluer (Linux)
cargo add bluer tokio --features tokio/full
# Implement basic scan

# Test bluest (cross-platform)
cargo add bluest tokio --features tokio/full
# Implement basic scan

# Compare APIs, features, ergonomics
```

**Deliverable**: Recommendation document with working prototypes

---

### Phase 2: Implementation (2-3 weeks)

#### Step 1: Create Pure Rust Bluetooth Module

**File**: `crates/songbird-genesis/src/physical_channels/bluetooth_pure.rs`

```rust
//! Pure Rust Bluetooth LE pairing
//! 
//! Uses platform-specific pure Rust implementations:
//! - Linux: bluer (official BlueZ bindings via zbus)
//! - Windows/macOS: bluest (cross-platform BLE)

use crate::{error::*, types::*};
use async_trait::async_trait;
use chrono::Utc;

#[cfg(target_os = "linux")]
use bluer::{Adapter, AdapterEvent};

#[cfg(not(target_os = "linux"))]
use bluest::{Adapter, Device};

use super::PhysicalChannelProvider;

/// Pure Rust Bluetooth LE channel
#[derive(Debug)]
pub struct PureRustBluetoothChannel {
    adapter: Option<Adapter>,
}

impl PureRustBluetoothChannel {
    /// Create new pure Rust Bluetooth channel
    pub async fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        let adapter = {
            let session = bluer::Session::new().await
                .map_err(|e| GenesisError::PhysicalChannel(format!("BlueZ session: {}", e)))?;
            Some(session.default_adapter().await
                .map_err(|e| GenesisError::PhysicalChannel(format!("No adapter: {}", e)))?)
        };
        
        #[cfg(not(target_os = "linux"))]
        let adapter = {
            let a = bluest::Adapter::default().await
                .map_err(|e| GenesisError::PhysicalChannel(format!("No adapter: {}", e)))?;
            a.wait_available().await
                .map_err(|e| GenesisError::PhysicalChannel(format!("Adapter unavailable: {}", e)))?;
            Some(a)
        };
        
        Ok(Self { adapter })
    }
    
    /// Scan for nearby devices
    async fn scan_for_witness(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            let adapter = self.adapter.as_ref().ok_or_else(|| 
                GenesisError::PhysicalChannel("No adapter".into()))?;
            
            adapter.set_powered(true).await
                .map_err(|e| GenesisError::PhysicalChannel(format!("Power on: {}", e)))?;
            
            let mut discover = adapter.discover_devices().await
                .map_err(|e| GenesisError::PhysicalChannel(format!("Discovery: {}", e)))?;
            
            use futures::stream::StreamExt;
            while let Some(evt) = discover.next().await {
                if let AdapterEvent::DeviceAdded(addr) = evt {
                    devices.push(addr.to_string());
                    if devices.len() >= 10 { break; } // Limit scan
                }
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            let adapter = self.adapter.as_ref().ok_or_else(|| 
                GenesisError::PhysicalChannel("No adapter".into()))?;
            
            let mut scan = adapter.scan(&[]).await
                .map_err(|e| GenesisError::PhysicalChannel(format!("Scan: {}", e)))?;
            
            use futures::stream::StreamExt;
            while let Some(discovered) = scan.next().await {
                if let Ok(name) = discovered.device.name().await {
                    devices.push(name);
                    if devices.len() >= 10 { break; }
                }
            }
        }
        
        Ok(devices)
    }
}

#[async_trait]
impl PhysicalChannelProvider for PureRustBluetoothChannel {
    async fn verify_proximity(&self) -> Result<ProximityProof> {
        // Scan for witness device
        let devices = self.scan_for_witness().await?;
        
        // Look for genesis witness device
        let witness_found = devices.iter()
            .any(|d| d.contains("Genesis") || d.contains("Witness"));
        
        if !witness_found {
            return Err(GenesisError::PhysicalChannel(
                "No genesis witness device found nearby".into()
            ));
        }
        
        Ok(ProximityProof {
            channel_type: PhysicalChannelType::Bluetooth,
            timestamp: Utc::now(),
            proof_data: format!("Found {} devices", devices.len()).into_bytes(),
            attestation: Some("pure-rust-bluetooth".to_string()),
        })
    }

    async fn secure_exchange(&self) -> Result<Vec<u8>> {
        // TODO: Implement GATT characteristic exchange
        // 1. Connect to witness device
        // 2. Read genesis credentials from GATT characteristic
        // 3. Verify signature
        // 4. Return credentials
        
        Ok(b"pure_rust_bluetooth_genesis_creds".to_vec())
    }

    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Medium
    }

    fn channel_type(&self) -> PhysicalChannelType {
        PhysicalChannelType::Bluetooth
    }
}
```

#### Step 2: Update Cargo.toml

```toml
[dependencies]
# Platform-specific pure Rust Bluetooth
bluer = { version = "0.17", optional = true }
bluest = { version = "0.5", optional = true }
futures = { version = "0.3", optional = true }

[target.'cfg(target_os = "linux")'.dependencies]
bluer = { version = "0.17", optional = true }

[target.'cfg(not(target_os = "linux"))'.dependencies]
bluest = { version = "0.5", optional = true }

[features]
default = ["solokey", "qr"]
bluetooth-legacy = ["btleplug"]  # Old implementation (has C deps on Linux)
bluetooth = ["bluer", "bluest", "futures"]  # Pure Rust! ✅
```

#### Step 3: Update mod.rs

```rust
// In physical_channels/mod.rs

#[cfg(feature = "bluetooth")]
pub mod bluetooth_pure;

#[cfg(feature = "bluetooth-legacy")]
pub mod bluetooth;  // Old implementation

#[cfg(feature = "bluetooth")]
pub use bluetooth_pure::PureRustBluetoothChannel as BluetoothChannel;

#[cfg(feature = "bluetooth-legacy")]
pub use bluetooth::BluetoothChannel;
```

#### Step 4: Testing

```bash
# Test on Linux (bluer)
cargo test --features bluetooth

# Test on macOS/Windows (bluest)
cargo test --features bluetooth

# Verify no C dependencies
cargo tree --features bluetooth | grep -i dbus
# Should return nothing! ✅
```

---

### Phase 3: Documentation & Migration (1 week)

#### Update Documentation

```markdown
## Bluetooth Genesis (Pure Rust ✅)

**Implementation**: 100% Pure Rust on all platforms

**Backends**:
- Linux: `bluer` (official BlueZ via zbus)
- Windows/macOS: `bluest` (native APIs)

**Usage**:
```bash
# Build with pure Rust Bluetooth
cargo build --features bluetooth

# No system dependencies required! ✅
```

**Migration from Legacy**:
```bash
# Old (has C deps on Linux)
cargo build --features bluetooth-legacy

# New (pure Rust everywhere)
cargo build --features bluetooth
```
```

#### Update PURE_RUST_BLUETOOTH_EVOLUTION.md

Add Phase 4 section documenting the implementation.

---

## 📊 EFFORT ESTIMATION

| Phase | Tasks | Time | Complexity |
|-------|-------|------|------------|
| **Phase 1: Research** | Prototype both libs | 1 week | Low |
| **Phase 2: Implementation** | Core functionality | 2-3 weeks | Medium |
| **Phase 3: Documentation** | Docs & migration | 1 week | Low |
| **Total** | | **4-5 weeks** | **Medium** |

**Breakdown**:
- Week 1: Research, prototypes, API comparison
- Week 2-3: Implement scan, connect, GATT exchange
- Week 4: Testing on all platforms
- Week 5: Documentation, migration guide

---

## 🎯 DECISION CRITERIA

### Choose **bluer + bluest** (Hybrid) if:
- ✅ Want official Linux support
- ✅ Need maximum stability on Linux
- ✅ Willing to maintain two backends

### Choose **bluest** (Unified) if:
- ✅ Want single codebase
- ✅ Prefer simplicity over "official"
- ✅ Okay with slightly newer library

### Keep **btleplug** (Current) if:
- ✅ Bluetooth is low priority
- ✅ System dep acceptable for Linux users
- ✅ Want to focus on other features first

---

## ✅ RECOMMENDATION

### Immediate (Now): ✅ DONE
- Keep Bluetooth optional
- Zero system deps by default

### Short Term (Next 2-3 months):
- **Implement bluest** (unified, simpler)
- Test on all platforms
- Document migration

### Long Term (6+ months):
- Evaluate hybrid approach if needed
- Contribute improvements upstream
- Share learnings with ecosystem

---

## 🚀 NEXT STEPS

### 1. Get Approval
- Review this plan with team
- Decide: bluest (unified) vs bluer+bluest (hybrid)
- Set timeline (4-5 weeks)

### 2. Start Prototype
```bash
cd /tmp
cargo new bluetooth-pure-test
cd bluetooth-pure-test

# Test bluest
cargo add bluest tokio --features tokio/full
# Write basic scan example
# Test on available platforms

# Compare with btleplug
cargo add btleplug tokio --features tokio/full
# Same example, compare APIs
```

### 3. Implementation
- Create `bluetooth_pure.rs`
- Implement PhysicalChannelProvider
- Add tests
- Document

### 4. Migration
- Keep `bluetooth-legacy` feature for compatibility
- Default to pure Rust implementation
- Deprecate legacy after 1-2 releases

---

## 💡 KEY INSIGHTS

### 1. Pure Rust BLE is Mature
- `bluer`: Official, stable, well-maintained
- `bluest`: Cross-platform, active development
- Both are production-ready

### 2. Platform Differences are Okay
- Linux: BlueZ (D-Bus based)
- Windows: WinRT APIs
- macOS: CoreBluetooth
- Each platform has pure Rust solution!

### 3. Incremental Evolution Works
- Phase 1: Make optional ✅ (done in 5 min)
- Phase 2: Pure Rust implementation (4-5 weeks)
- Phase 3: Deprecate legacy (when ready)

### 4. Community Benefits
- Our work helps entire Rust ecosystem
- Document learnings
- Share best practices

---

## 📚 RESOURCES

**Crates**:
- `bluer`: https://crates.io/crates/bluer
- `bluest`: https://crates.io/crates/bluest
- `zbus`: https://crates.io/crates/zbus

**Documentation**:
- bluer docs: https://docs.rs/bluer
- bluest docs: https://docs.rs/bluest
- BlueZ D-Bus API: https://git.kernel.org/pub/scm/bluetooth/bluez.git/tree/doc

**Examples**:
- bluer examples: https://github.com/bluez/bluer/tree/master/bluer/examples
- bluest examples: https://github.com/alexmoon/bluest/tree/master/examples

---

## ✅ CONCLUSION

**Yes, we can absolutely evolve to pure Rust Bluetooth!**

**Best Path**: 
1. ✅ **Immediate**: Optional feature (done!)
2. 🔄 **Next**: Implement `bluest` (4-5 weeks)
3. 🔮 **Future**: Evaluate hybrid if needed

**Timeline**: 4-5 weeks for full pure Rust implementation  
**Complexity**: Medium (well-documented libraries)  
**Benefit**: 100% sovereignty, zero system deps, better security

**Ready to start when you are!** 🚀

---

**Updated**: December 23, 2025  
**Status**: Plan complete, awaiting approval to implement  
**Estimated Start**: Q1 2026 (after SoloKey & QR genesis)

🦀 Pure Rust All The Way Down!

