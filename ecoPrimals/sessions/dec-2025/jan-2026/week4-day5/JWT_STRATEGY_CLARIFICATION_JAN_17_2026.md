# JWT Strategy Clarification
**Date**: January 17, 2026  
**Status**: Architectural Decision

---

## Two JWT Use Cases

### 1. **External HTTP Authentication** (BearDog Delegation) ✅

**Use Case**: Authenticate external HTTP requests from clients

**Strategy**: **BearDog Delegation** (IMPLEMENTED TODAY!)
- Songbird provisions JWT secret from BearDog at startup
- BearDog generates 512-bit Ed25519 secret
- Songbird uses secret for HTTP authentication
- Pure Rust IPC (JSON-RPC over Unix socket)

**Status**: ✅ Phases 1-3 complete!

### 2. **Internal Access Control** (Local JWT) ⏳

**Use Case**: Internal role-based access control (Student, TA, Professor, Admin)

**Current**: Uses `jsonwebtoken` (ring-based, C dependency)

**Strategy**: **Keep for now, evolve later**
- This is internal-only (not exposed to external HTTP)
- Used for role-based access control within Songbird
- Low priority for Pure Rust migration
- Can be evolved to HMAC-based JWT (Pure Rust) in Q2 2026

**Rationale**:
- External HTTP auth (higher priority) now uses BearDog ✅
- Internal access control is isolated
- `jsonwebtoken` uses `ring` (same as `rustls` TLS)
- Both will be migrated together in Q4 2026 (rustls-rustcrypto)

---

## Revised ecoBin Timeline

### Today (95% ecoBin)
- ✅ Compression: Pure Rust (flate2)
- ✅ USB: Pure Rust (nusb)
- ⚠️ JWT (external): **BearDog delegation (Pure Rust IPC!)** ✅
- ⚠️ JWT (internal): jsonwebtoken → ring (C)
- ⚠️ TLS: rustls → ring (C)

### Q2 2026 (97% ecoBin) - Optional
- JWT (internal): Migrate to HMAC-based Pure Rust JWT
- Result: Only TLS remains

### Q4 2026 (100% ecoBin!)
- TLS: Migrate to rustls-rustcrypto
- Result: ZERO C dependencies!

---

## Decision

**KEEP `jsonwebtoken` for now**

**Rationale**:
1. ✅ External HTTP auth now uses BearDog (TODAY'S GOAL!)
2. ✅ Internal access control is isolated
3. ✅ `jsonwebtoken` and `rustls` both use `ring`
4. ✅ Can migrate both together in Q4 2026
5. ✅ Simpler, faster path to 97% ecoBin

**Impact**:
- Today: 95% ecoBin (BearDog delegation for external auth!)
- Q4 2026: 100% ecoBin (rustls-rustcrypto migration)

---

## Summary

**External HTTP Authentication**: ✅ BearDog delegation (Pure Rust!) - DONE TODAY!  
**Internal Access Control**: ⏳ Keep `jsonwebtoken` for now, migrate in Q4 2026

**Result**: Pragmatic path to 100% ecoBin, with external auth (highest priority) using Pure Rust TODAY! 🦀✨

