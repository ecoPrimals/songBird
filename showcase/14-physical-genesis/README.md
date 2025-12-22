# 🔐 Physical Genesis Bootstrap Showcase

**Status**: 🚧 In Development  
**Goal**: Demonstrate physical proximity genesis for new nodes

---

## What This Demonstrates

**"Never let a bird be alone in the dark forest"**

This showcase demonstrates physical genesis bootstrap - creating new nodes with:
- ✅ Physical proximity verification (SoloKey, QR, Bluetooth)
- ✅ Witnessed genesis with cryptographic signatures
- ✅ Multi-primal coordination (Songbird + BearDog)
- ✅ Strong lineage from birth
- ✅ **Never vulnerable, never alone!**

---

## Test Scripts

### 01. Mock Genesis Ceremony
**Script**: `01-mock-genesis-ceremony.sh`  
**Status**: ✅ Ready  
**Tests**: Basic genesis flow with mock physical channel

### 02. SoloKey Genesis (TODO)
**Script**: `02-solokey-genesis.sh`  
**Status**: 🚧 Pending SoloKey implementation  
**Tests**: Real hardware key tap genesis

### 03. QR Code Genesis (TODO)
**Script**: `03-qr-code-genesis.sh`  
**Status**: 🚧 Pending QR implementation  
**Tests**: QR code + out-of-band verification

### 04. Multi-Primal Coordination (TODO)
**Script**: `04-multi-primal-coordination.sh`  
**Status**: 🚧 Pending BearDog integration  
**Tests**: Songbird + BearDog coordinated genesis

### 05. Genesis to Discovery (TODO)
**Script**: `05-genesis-to-discovery.sh`  
**Status**: 🚧 Pending BirdSong integration  
**Tests**: Genesis → LAN discovery flow

---

## Prerequisites

```bash
# 1. Build Songbird with genesis module
cargo build --release

# 2. For real tests (later):
# - SoloKey or compatible FIDO2 key
# - Bluetooth LE support
# - Camera for QR scanning
```

---

## Running Tests

### Quick Test (Mock Genesis)
```bash
./01-mock-genesis-ceremony.sh
```

### Full Test Suite (When Ready)
```bash
for test in 0*.sh; do
    echo "Running $test..."
    ./$test || echo "FAILED: $test"
done
```

---

## Architecture

### Genesis Flow

```
1. Physical Proximity Verification
   ↓
2. Genesis Credential Exchange
   ↓
3. Witness Signs New Identity
   ↓
4. Multi-Primal Coordination
   - Songbird: Federation lineage
   - BearDog: Genetic lineage
   - Toadstool: Compute lineage (future)
   ↓
5. Unified Genesis Certificate
   ↓
6. New Node Born with Full Identity! ✅
```

### Trust Levels

| Channel | Trust | Hardware |
|---------|-------|----------|
| **SoloKey** | ⭐⭐⭐⭐⭐ | Yes |
| **QR + OOB** | ⭐⭐⭐⭐ | No |
| **Bluetooth** | ⭐⭐⭐ | No |

---

## Next Steps

1. ✅ **Complete**: Genesis module foundation
2. 🚧 **In Progress**: Mock genesis test
3. 🔜 **Next**: SoloKey implementation
4. 🔜 **Then**: BearDog coordination
5. 🔜 **Finally**: BirdSong integration

---

**🔐 Physical Genesis: The Right Way to Birth a Node!** ✨

