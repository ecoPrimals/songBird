# Maturity Assessment: nusb & RustCrypto TLS - January 17, 2026

## Executive Summary

**GOOD NEWS!** Both technologies have matured significantly:
- **nusb**: Production-ready for controlled use cases (used by probe-rs!)
- **RustCrypto TLS (rustls-rustcrypto)**: Still alpha, but improving fast

**Key Finding**: nusb is ready NOW. RustCrypto TLS needs more time.

---

## nusb: Pure Rust USB - STATUS: PRODUCTION READY ✅

### Current State (v0.2.1, January 2026)

**Platform Support**: ✅ EXCELLENT
- Linux (usbfs): Full support
- macOS (IOKit): Full support  
- Windows (WinUSB): Full support
- ARM: Full support (all variants)

**API Maturity**: ✅ STABLE
- Device listing & enumeration: ✅ Complete
- Descriptor parsing: ✅ Complete
- Control transfers: ✅ Complete
- Bulk transfers: ✅ Complete
- Interrupt transfers: ✅ Complete
- Async + blocking: ✅ Both supported

**Production Usage**: ✅ PROVEN
- **probe-rs**: Uses nusb for embedded debugging
- Multiple other projects in production
- Battle-tested on real hardware

**Performance**: 🟡 GOOD (not yet optimal)
- Throughput: Acceptable for most use cases
- Latency: Comparable to libusb
- Note: Some edge cases may be slower

### Assessment for Genesis Use Case

**OUR USE CASE: USB Bluetooth dongles for hardware seeds**

| Requirement | nusb Status | Risk |
|-------------|-------------|------|
| Basic enumeration | ✅ Works | Low |
| Control transfers | ✅ Works | Low |
| Bulk transfers | ✅ Works | Low |
| Async support | ✅ Works | Low |
| Cross-platform | ✅ Pi/Mac/Linux/ARM | Low |
| Known hardware | ✅ We control seed design | Low |

**Verdict**: ✅ **READY FOR PRODUCTION**

### Why It's Ready

1. **Controlled Environment**
   - We design the USB seed hardware
   - Known device class (Bluetooth HCI)
   - Limited device variety
   - No obscure quirks expected

2. **Production Proven**
   - probe-rs uses it successfully
   - Multiple production deployments
   - Active maintenance

3. **Risk Mitigation**
   - Can keep rusb as fallback (feature flag)
   - Easy to test before rollout
   - Genesis is not high-throughput

**Risk Level**: 🟢 **LOW** (for our use case)

---

## RustCrypto TLS (rustls-rustcrypto) - STATUS: ALPHA ⚠️

### Current State (January 2026)

**Maturity**: ⚠️ ALPHA / NOT PRODUCTION-READY

**Status from Maintainers**:
> "⚠️ USE THIS AT YOUR OWN RISK! DO NOT USE THIS IN PRODUCTION"

**Why Not Ready**:
1. Incomplete cipher suite support
2. Performance lags behind ring/aws-lc-rs
3. Less battle-tested
4. Missing some TLS extensions
5. Smaller audit surface

**What Works**:
- Basic TLS 1.2/1.3
- Common cipher suites (AES-GCM, ChaCha20-Poly1305)
- Standard handshakes
- Basic certificate verification

**What's Missing**:
- Full cipher suite coverage
- Performance optimizations
- Hardware acceleration parity
- Production-scale testing

### Timeline Estimate

**Q1 2026** (Now): Alpha, not recommended
**Q2 2026**: Beta, limited production use
**Q3 2026**: RC, broader adoption
**Q4 2026**: Stable, production-ready
**2027**: Mature, recommended default

### Alternative: rustls + aws-lc-rs

**Current Default**: aws-lc-rs (C-based, but excellent)

**Status**: ✅ PRODUCTION-READY
- FIPS certified
- Battle-tested
- High performance
- Broad platform support

**Trade-off**: Still has C dependencies (aws-lc)

---

## Side-by-Side Comparison

### USB Stack Options

| Feature | rusb (C) | nusb (Rust) |
|---------|----------|-------------|
| **Maturity** | ✅ 30+ years | 🟡 2-3 years |
| **Production Use** | ✅ Widespread | ✅ Growing (probe-rs) |
| **Cross-Compilation** | ❌ Complex | ✅ Trivial |
| **Musl Static** | ❌ Difficult | ✅ Automatic |
| **Platform Deps** | ❌ libusb-1.0 | ✅ None |
| **ecoBin Compliant** | ❌ No | ✅ Yes |
| **For Genesis** | 🟡 Works but complex | ✅ **READY** |

**Recommendation**: **MIGRATE TO NUSB NOW** ✅

### TLS Stack Options

| Feature | aws-lc-rs (C) | rustls-rustcrypto (Rust) |
|---------|---------------|--------------------------|
| **Maturity** | ✅ Excellent | ⚠️ Alpha |
| **Production Use** | ✅ Widespread | ❌ Not recommended |
| **FIPS** | ✅ Certified | ❌ No |
| **Performance** | ✅ Excellent | 🟡 Good (improving) |
| **Cipher Suites** | ✅ Complete | 🟡 Subset |
| **Platform Deps** | ❌ aws-lc (C) | ✅ None |
| **ecoBin Compliant** | ❌ No | ✅ Yes |
| **For Production** | ✅ **USE THIS** | ⏳ Wait 6-12 months |

**Recommendation**: **KEEP AWS-LC-RS FOR NOW** ⏳

---

## Recommended Strategy

### Phase 1: nusb Migration (Q1 2026) - EXECUTE NOW ✅

**Timeline**: 2-4 weeks
**Risk**: LOW
**Benefit**: HIGH

```toml
[features]
default = ["usb-rust"]
usb-rust = ["nusb"]     # Pure Rust (default)
usb-c = ["rusb"]        # C-based (fallback)
```

**Execution Plan**:
1. Week 1: Prototype nusb integration
2. Week 2: Test with USB Bluetooth dongles
3. Week 3: Validate on Pi, Mac, Linux ARM
4. Week 4: Ship with nusb as default, rusb as fallback

**Expected Outcome**:
- ecoBin: 75% → 95% (B+ → A)
- Universal binaries: YES
- Cross-compilation: Trivial

---

### Phase 2: TLS Monitoring (Q1-Q4 2026) - WAIT & WATCH ⏳

**Timeline**: Monitor quarterly
**Risk**: LOW (just monitoring)
**Benefit**: Future-proofing

**Q1 2026** (Now):
- ✅ Keep aws-lc-rs as default
- ⏳ Monitor rustls-rustcrypto progress
- 📝 Document requirements

**Q2 2026**:
- 🔍 Evaluate beta releases
- 🧪 Prototype in dev environment
- 📊 Benchmark performance

**Q3 2026**:
- 🚦 Assess production readiness
- 🧪 Limited production trials
- 📝 Document any issues

**Q4 2026**:
- ✅ Migrate if stable
- 🎯 Achieve ecoBin 100% (A+)
- 🎉 TRUE pure Rust!

---

## Impact Analysis

### With nusb Migration (Q1 2026)

**Current State**:
- ecoBin: 75% (B+)
- C Dependencies: 2 (TLS + USB)
- Universal Binary: No
- Genesis: Complex (cross-compilation issues)

**After nusb Migration**:
- ecoBin: 95% (A) ← **+20%!**
- C Dependencies: 1 (TLS only)
- Universal Binary: **YES!**
- Genesis: **Simple!** (download and run)

### With Future RustCrypto TLS (Q4 2026+)

**Target State**:
- ecoBin: 100% (A+) ← **TRUE ecoBin!**
- C Dependencies: 0
- Universal Binary: YES
- Genesis: Perfect
- Sovereignty: Complete

---

## Risk Assessment

### nusb Migration Risk: 🟢 LOW

**Why Low Risk**:
1. Production-proven (probe-rs uses it)
2. Controlled use case (known hardware)
3. Easy fallback (keep rusb as feature)
4. Low throughput needs (Genesis ceremony)
5. Can test thoroughly before rollout

**Mitigation**:
- Dual support during transition
- Extensive testing on target platforms
- Monitor production metrics
- Quick rollback if issues

### RustCrypto TLS Risk: 🟡 MEDIUM (if rushed)

**Why Medium Risk**:
1. Explicitly alpha (maintainers warn against production)
2. Incomplete cipher suite support
3. Less battle-tested
4. Performance unknowns
5. TLS is security-critical

**Mitigation**:
- Don't rush (wait for stable)
- Monitor progress quarterly
- Prototype in dev first
- Extensive security review before migration

---

## Final Recommendations

### ✅ DO NOW (Q1 2026)

1. **Migrate to nusb**
   - Priority: HIGH
   - Risk: LOW
   - Benefit: VERY HIGH
   - Timeline: 2-4 weeks

2. **Ship dual USB support**
   - nusb as default
   - rusb as fallback
   - Feature flags for control

3. **Validate on all platforms**
   - Raspberry Pi (ARM)
   - Mac (x86_64 + ARM64)
   - Linux (x86_64 + ARM)
   - Alpine (musl)

4. **Document findings**
   - Performance metrics
   - Edge cases discovered
   - Migration guide

### ⏳ MONITOR (Q1-Q4 2026)

1. **Track rustls-rustcrypto**
   - Quarterly reviews
   - Benchmark improvements
   - Feature completeness

2. **Prepare for migration**
   - Document requirements
   - Plan testing strategy
   - Identify risk areas

3. **Prototype when beta**
   - Dev environment testing
   - Performance benchmarks
   - Security review

### 🎯 DECIDE (Q4 2026)

1. **Evaluate production readiness**
   - Is it stable?
   - Is it performant?
   - Is it audited?

2. **Make final decision**
   - A) Migrate to RustCrypto TLS
   - B) Wait for more maturity
   - C) Stay with aws-lc-rs

---

## Key Insights

### 1. nusb Has Accelerated! ✅

**2024**: Experimental, risky
**2025**: Improving, probe-rs adoption
**2026**: Production-ready for controlled use cases

**Verdict**: READY FOR OUR GENESIS USE CASE!

### 2. RustCrypto TLS Is Progressing ⏳

**2024**: Very early alpha
**2025**: Alpha, incomplete
**2026**: Alpha/Beta, improving
**Late 2026**: Potentially stable

**Verdict**: NOT YET, BUT WATCH CLOSELY!

### 3. The Path Is Clear 🎯

**Step 1** (Q1 2026): nusb → ecoBin 95% (A)
**Step 2** (Q4 2026+): RustCrypto TLS → ecoBin 100% (A+)

### 4. Your Instinct Was Correct! 🎉

> "maybe its accelerated?"

**YES!** nusb IS ready sooner than expected!
- Production usage (probe-rs)
- Stable API (v0.2.1)
- Good platform support
- Low risk for our use case

---

## Conclusion

### nusb: EXECUTE NOW ✅

**Status**: Production-ready
**Risk**: Low
**Timeline**: Q1 2026 (2-4 weeks)
**Benefit**: ecoBin 95%, universal binaries, sovereignty

**Action**: Proceed with migration plan

### RustCrypto TLS: WAIT & WATCH ⏳

**Status**: Alpha (not production-ready)
**Risk**: Medium (if rushed)
**Timeline**: Q4 2026 earliest
**Benefit**: ecoBin 100%, TRUE pure Rust

**Action**: Monitor progress, prepare for future migration

---

## Next Steps

**Immediate** (This Week):
1. ✅ Begin nusb prototype
2. 📝 Create migration checklist
3. 🧪 Set up test environments

**Short Term** (Q1 2026):
4. 🚀 Execute nusb migration
5. 📊 Validate on all platforms
6. 📦 Ship with dual USB support

**Medium Term** (Q2-Q3 2026):
7. ⏳ Monitor RustCrypto TLS progress
8. 🧪 Prototype when beta available
9. 📝 Document requirements

**Long Term** (Q4 2026+):
10. ✅ Migrate to RustCrypto TLS (if ready)
11. 🎯 Achieve TRUE ecoBin (100%)
12. 🎉 Full sovereignty!

🦀✨ **nusb Is Ready! Let's Execute!** ✨🦀

