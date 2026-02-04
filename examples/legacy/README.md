# Legacy Examples

**Status**: Archived  
**Date**: February 3, 2026  
**Reason**: Pre-ecoBin v2.0 examples using `reqwest`

---

## About These Examples

These examples are **LEGACY** code that predates the ecoBin v2.0 migration (Feb 2026).
They demonstrate ecosystem integration patterns but use `reqwest` for HTTP communication,
which has C dependencies (ring/openssl) and is NOT compliant with 100% Pure Rust requirements.

## ⚠️ Not Recommended for Production

These examples are kept for:
- **Historical reference** - Understanding pre-v2.0 architecture
- **Migration documentation** - Comparing old vs new patterns
- **Learning purposes** - Understanding the evolution of ecoPrimals

**For production code, use modern examples instead!**

---

## Modern Alternatives

### Instead of these legacy examples, use:

1. **`../ipc_http_client_demo.rs`**  
   Drop-in replacement for reqwest using IpcHttpClient (100% Pure Rust, no C dependencies)

2. **Production adapters** in `crates/songbird-universal/src/adapters/`  
   - `ai.rs` - Modern AI adapter (replaces legacy Squirrel example)
   - `storage.rs` - Modern storage adapter (replaces legacy NestGate example)
   - `security.rs` - Modern security adapter (replaces legacy BearDog example)
   - `compute.rs` - Modern compute adapter (replaces legacy ToadStool example)

3. **Migration Guide**  
   See: `ecoPrimals/sessions/feb-2026/reqwest-removal/REQWEST_MIGRATION_GUIDE.md`

---

## What's in This Directory

### `ecosystem-primals/` - Legacy Primal Adapters

Pre-v2.0 adapters demonstrating ecosystem integration with HTTP-based communication:

- **`squirrel.rs`** - AI adapter (legacy, uses reqwest)
- **`nestgate.rs`** - Storage adapter (legacy, uses reqwest)
- **`beardog.rs`** - Security adapter (legacy, uses reqwest)
- **`toadstool.rs`** - Compute metrics adapter (legacy, uses reqwest)

All use `reqwest::Client` for HTTP communication with external services.

### Why These Were Archived

**Problem**: `reqwest` depends on:
- `ring` (C/assembly crypto library)
- `openssl-sys` (bindings to OpenSSL)
- Platform-specific native TLS

**Impact**:
- ❌ Not 100% Pure Rust
- ❌ Complex cross-compilation
- ❌ Large binary size
- ❌ Security audit burden

**Solution**: Migrated to `IpcHttpClient` via Unix sockets
- ✅ 100% Pure Rust
- ✅ Zero C dependencies
- ✅ Compile-time safety
- ✅ Better performance (connection pooling)

---

## Migration Timeline

| Date | Event |
|------|-------|
| Pre-2026 | Examples created using reqwest |
| Jan 2026 | IpcHttpClient developed (Pure Rust) |
| Feb 3, 2026 | Examples archived to legacy/ |
| Feb 3, 2026 | New production adapters available |

---

## Can I Still Use These?

**Yes, but not recommended:**
- ✅ These examples still compile (as of Feb 3, 2026)
- ✅ Useful for learning/comparison
- ⚠️ Not production-ready (C dependencies)
- ⚠️ Won't receive updates/fixes
- ⚠️ May break in future Rust versions

**For new projects**: Start with modern examples!

---

## Related Documentation

- [`CLEANUP_PLAN_FEB_03_2026.md`](../../CLEANUP_PLAN_FEB_03_2026.md) - Why these were archived
- [`REQWEST_ELIMINATION_EVOLUTION_PLAN.md`](../../ecoPrimals/sessions/feb-2026/reqwest-removal/) - Migration strategy
- [`ROOT_DOCS_INDEX.md`](../../ROOT_DOCS_INDEX.md) - Project overview

---

**Questions?** See the main examples directory (`../`) for modern alternatives.

**Migration help?** Read the reqwest removal migration guide in `ecoPrimals/sessions/feb-2026/`.

---

*Last updated: February 3, 2026*  
*Maintained by: ecoPrimals Team*
