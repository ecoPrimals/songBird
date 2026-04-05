# 📚 Ecosystem Primal Integration Examples

**SOVEREIGNTY PRINCIPLE**: These files are **EXAMPLES ONLY** showing how
specific primals in the ecoPrimals ecosystem happen to implement various
capabilities.

## 🌿 Ecological Model

Like in ecology, **each organism exists independently**:

- **Songbird doesn't "know" these primals exist**
- Production code uses capability-based adapters
- These examples show "how Compute Provider implements compute capability"
- But they're just ONE possible implementation

## 📁 Files

- `compute_provider.rs` - Example: How Compute Provider implements compute capability
- `security_provider.rs` - Example: How Security Provider implements security capability
- `storage_provider.rs` - Example: How Storage Provider implements storage capability
- `ai_provider.rs` - Example: How AI Provider implements AI capability

## ✅ Production Code

Production code in `crates/songbird-universal/src/adapters/` uses:

- `ComputeAdapter` - Generic compute capability (not Compute Provider-specific)
- `SecurityAdapter` - Generic security capability (not Security Provider-specific)
- `StorageAdapter` - Generic storage capability (not Storage Provider-specific)
- `AIAdapter` - Generic AI capability (not AI Provider-specific)

## 🎯 Key Principle

**Songbird has local storage for sovereign standalone operation**, but can
utilize whatever storage provider is available (happens to be Storage Provider in our
ecosystem) for network effects.

The code does **NOT** know about Storage Provider - only we do as observers of the
ecosystem.
