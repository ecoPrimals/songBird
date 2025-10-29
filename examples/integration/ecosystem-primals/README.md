# 📚 Ecosystem Primal Integration Examples

**SOVEREIGNTY PRINCIPLE**: These files are **EXAMPLES ONLY** showing how
specific primals in the ecoPrimals ecosystem happen to implement various
capabilities.

## 🌿 Ecological Model

Like in ecology, **each organism exists independently**:

- **Songbird doesn't "know" these primals exist**
- Production code uses capability-based adapters
- These examples show "how ToadStool implements compute capability"
- But they're just ONE possible implementation

## 📁 Files

- `toadstool.rs` - Example: How ToadStool implements compute capability
- `beardog.rs` - Example: How BearDog implements security capability
- `nestgate.rs` - Example: How NestGate implements storage capability
- `squirrel.rs` - Example: How Squirrel implements AI capability

## ✅ Production Code

Production code in `crates/songbird-universal/src/adapters/` uses:

- `ComputeAdapter` - Generic compute capability (not ToadStool-specific)
- `SecurityAdapter` - Generic security capability (not BearDog-specific)
- `StorageAdapter` - Generic storage capability (not NestGate-specific)
- `AIAdapter` - Generic AI capability (not Squirrel-specific)

## 🎯 Key Principle

**Songbird has local storage for sovereign standalone operation**, but can
utilize whatever storage provider is available (happens to be NestGate in our
ecosystem) for network effects.

The code does **NOT** know about NestGate - only we do as observers of the
ecosystem.
