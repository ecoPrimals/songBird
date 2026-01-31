# 🧬 genomeBin Week 3: Execution Plan
**Self-Extracting Deployment + neuralAPI Graph Integration**

**Date**: January 31, 2026 (Evening)  
**Upstream**: genomeBin Evolution Roadmap from wateringHole  
**Status**: Planning Complete - Ready for Execution  
**Priority**: 🔴 CRITICAL (Final genomeBin compliance)

---

## 🎯 Executive Summary

### Where We Are (Weeks 1+2 Complete):
- ✅ **Week 1**: ARM64 cross-compilation (4 targets: musl, Android, Windows, macOS planned)
- ✅ **Week 2**: Deployment infrastructure (systemd, USB, Windows, Android)
- ✅ **Archive**: Cleaned root directory (50% reduction, 14 files archived)

### Where We're Going (Week 3):
- 🎯 **Self-Extracting genomeBin**: `songbird.genome` (one file, works everywhere)
- 🎯 **neuralAPI Graph Integration**: TOML-based orchestrated deployment
- 🎯 **Deep Debt Evolution**: Full async/concurrent patterns
- 🎯 **Platform Optimization**: Android-specific fast paths

---

## 📊 Current Status vs Upstream Requirements

### ✅ Phase 1: ARM64 Cross-Compilation (COMPLETE)
```
Upstream Requirements:
  1. ✅ Add aarch64 target
  2. ✅ Configure .cargo/config.toml for Android NDK
  3. ✅ Build for aarch64-linux-android
  4. ⏳ Validate on Pixel 8a (hardware required)
  5. ✅ Document process

OUR STATUS:
  ✅ 100% Priority Targets (Linux, Android, Windows, musl)
  ✅ Documentation: CROSS_COMPILATION_PROGRESS_JAN_31_2026.md
  ✅ Week 1 Victory: GENOMEBIN_WEEK1_VICTORY_JAN_31_2026.md
```

### ✅ Phase 2: Deployment Wrapper Creation (90% COMPLETE)
```
Upstream Requirements:
  1. ✅ Adapt template for primal-specific needs
  2. ✅ Add health checks and validation
  3. ✅ Add service installation (systemd, etc.)
  4. ⏳ Test self-extraction and execution (MISSING!)
  5. ✅ Document deployment process

OUR STATUS:
  ✅ systemd services (single + multi-instance)
  ✅ USB Live Spore launcher (bash)
  ✅ Windows launcher (PowerShell)
  ✅ Android deployment guide
  ✅ XDG-compliant configuration (TOML)
  ✅ Documentation: 10 files, ~3,170 lines
  
  ⚠️ MISSING: Self-extracting .genome wrapper!
```

### ❌ Phase 3: neuralAPI Graph Integration (NOT STARTED)
```
Upstream Requirements:
  1. ⏳ Create primal-specific deployment graphs
  2. ⏳ Test graph deployment via neuralAPI
  3. ⏳ Validate health checks post-deployment
  4. ⏳ Test rollback on failure
  5. ⏳ Document graph patterns

OUR STATUS:
  ❌ NOT STARTED
  
  REQUIRED:
    • Create songbird deployment graphs (TOML)
    • Integrate with biomeOS neuralAPI
    • Implement health check hooks
    • Implement rollback mechanism
    • Document patterns
```

---

## 🎯 Week 3 Objectives

### Objective #1: Create `songbird.genome` (Self-Extracting Wrapper)
**Priority**: 🔴 CRITICAL  
**Estimated**: 1-2 days  
**Deliverable**: Single self-extracting binary that works on all platforms

**Requirements**:
1. **Architecture Detection**: Auto-detect x86_64, aarch64, riscv64
2. **Platform Detection**: Auto-detect Linux, Android, macOS, Windows
3. **Binary Embedding**: Embed all compiled binaries in tar archive
4. **Self-Extraction**: Extract to appropriate location
5. **Execution**: Run correct binary for detected arch+platform
6. **Health Check**: Validate deployment success
7. **Rollback**: Clean up on failure

**Implementation Strategy**:
```bash
#!/usr/bin/env bash
# songbird.genome - Universal self-deploying genomeBin

# Header: Detection & extraction logic
# Body: Embedded multi-arch tar.gz archive
# Footer: Checksum validation

ARCH=$(uname -m)
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')

case "${PLATFORM}_${ARCH}" in
  linux_x86_64)     BINARY="songbird-x86_64-linux-musl" ;;
  linux_aarch64)    BINARY="songbird-aarch64-linux-musl" ;;
  android_aarch64)  BINARY="songbird-aarch64-linux-android" ;;
  darwin_x86_64)    BINARY="songbird-x86_64-darwin" ;;
  darwin_aarch64)   BINARY="songbird-aarch64-darwin" ;;
  windows_x86_64)   BINARY="songbird.exe" ;;
  *) echo "Unsupported: ${PLATFORM}_${ARCH}"; exit 1 ;;
esac

# Self-extract from embedded archive
ARCHIVE_LINE=$(awk '/^__ARCHIVE_START__/ {print NR + 1; exit 0; }' "$0")
tail -n +${ARCHIVE_LINE} "$0" | tar xzf - -C "${INSTALL_DIR}"

# Execute extracted binary
exec "${INSTALL_DIR}/${BINARY}" "$@"
exit 0

__ARCHIVE_START__
<embedded tar.gz follows>
```

**Deep Debt Opportunity**:
- **Current**: 4 separate deployment mechanisms (systemd, USB, Windows, Android)
- **Target**: Universal self-extracting archive
- **Benefit**: One file works everywhere! Download once, run anywhere!

---

### Objective #2: neuralAPI Graph Integration
**Priority**: 🔴 CRITICAL  
**Estimated**: 2-3 days  
**Deliverable**: TOML-based deployment graphs for biomeOS orchestration

**Requirements**:
1. **Deployment Graphs**: Create `.toml` graphs for various scenarios
2. **Health Check Integration**: Add post-deployment validation hooks
3. **Rollback Mechanism**: Implement failure recovery
4. **Graph Testing**: Validate via biomeOS neuralAPI
5. **Documentation**: Graph pattern library

**Implementation Strategy**:

**Graph 1: `songbird_deploy.toml`** (Single instance)
```toml
[[nodes]]
id = "detect_platform"
type = "platform.detect"
config = { arch = "auto", os = "auto" }

[[nodes]]
id = "deploy_songbird"
type = "genome.deploy"
config = { 
  genome = "songbird.genome",
  target = "auto",
  install_dir = "/opt/biomeos" 
}
depends_on = ["detect_platform"]

[[nodes]]
id = "health_check"
type = "health.check_primal"
config = { primal = "songbird", timeout_ms = 5000 }
depends_on = ["deploy_songbird"]

[[nodes]]
id = "register_capabilities"
type = "capability.register"
config = { primal = "songbird" }
depends_on = ["health_check"]

[[edges]]
on_error = { from = "deploy_songbird", to = "rollback" }

[[nodes]]
id = "rollback"
type = "genome.rollback"
config = { primal = "songbird" }
```

**Graph 2: `tower_genome.toml`** (BearDog + Songbird)
```toml
# Deploy security foundation first
[[nodes]]
id = "deploy_beardog"
type = "genome.deploy"
config = { genome = "beardog.genome", target = "auto" }

# Deploy discovery after security
[[nodes]]
id = "deploy_songbird"
type = "genome.deploy"
config = { genome = "songbird.genome", target = "auto" }
depends_on = ["deploy_beardog"]

# Validate TOWER is operational
[[nodes]]
id = "verify_tower"
type = "health.check_atomic"
config = { primals = ["beardog", "songbird"] }
depends_on = ["deploy_songbird"]

# Register cross-primal capabilities
[[nodes]]
id = "wire_capabilities"
type = "capability.wire_tower"
depends_on = ["verify_tower"]
```

**Graph 3: `cross_platform_deploy.toml`** (USB + Android simultaneously)
```toml
[[nodes]]
id = "deploy_usb"
type = "genome.deploy"
config = { 
  genome = "songbird.genome",
  target = "usb:/dev/sdb1",
  mode = "live-spore"
}

[[nodes]]
id = "deploy_android"
type = "genome.deploy"
config = { 
  genome = "songbird.genome",
  target = "android:adb",
  mode = "termux"
}

# Parallel deployment - no dependency

[[nodes]]
id = "establish_handshake"
type = "federation.handshake"
config = { 
  nodes = ["usb_songbird", "android_songbird"],
  protocol = "dark_forest"
}
depends_on = ["deploy_usb", "deploy_android"]
```

**Deep Debt Opportunity**:
- **Current**: Manual, imperative deployment
- **Target**: Declarative, graph-based orchestration
- **Benefit**: Atomic, reversible, auditable deployments!

---

### Objective #3: Deep Debt Evolution (Async/Concurrent Patterns)
**Priority**: 🟡 HIGH  
**Estimated**: 2-3 days (ongoing)  
**Deliverable**: Fully async/concurrent discovery and IPC

**Current State Analysis**:
```
Async Functions: 7,295 across 775 files ✅
Concurrent Patterns: 1,830 spawn/join/select/race ✅

GOOD SIGNS:
  • Heavy use of async/await throughout
  • Extensive tokio concurrency primitives
  • Proper async propagation in most modules

OPPORTUNITIES:
  • Some blocking file I/O in config loading
  • Sequential discovery patterns (could be parallel)
  • mDNS queries not fully concurrent
  • STUN server probing one-at-a-time
```

**Evolution Areas**:

**1. Concurrent Discovery Racing**
```rust
// CURRENT: Sequential STUN server probing
for server in stun_servers {
    if let Ok(result) = client.discover(server).await {
        return Ok(result);
    }
}

// EVOLVED: Concurrent racing (first to respond wins)
use tokio::time::{timeout, Duration};
use futures::future::select_ok;

let futures: Vec<_> = stun_servers
    .iter()
    .map(|server| {
        let client = client.clone();
        let server = server.clone();
        tokio::spawn(async move {
            timeout(Duration::from_secs(5), client.discover(&server)).await
        })
    })
    .collect();

match select_ok(futures).await {
    Ok((result, _remaining)) => Ok(result),
    Err(e) => Err(anyhow!("All STUN servers failed: {}", e)),
}
```

**2. Async File I/O (Config Loading)**
```rust
// CURRENT: Blocking file I/O
let config_content = std::fs::read_to_string(&path)?;

// EVOLVED: Async file I/O
let config_content = tokio::fs::read_to_string(&path).await?;
```

**3. Parallel mDNS Service Discovery**
```rust
// CURRENT: Sequential service queries
let beardog = discover_service("beardog").await?;
let squirrel = discover_service("squirrel").await?;
let toadstool = discover_service("toadstool").await?;

// EVOLVED: Parallel discovery with join_all
use futures::future::join_all;

let services = vec!["beardog", "squirrel", "toadstool"];
let futures: Vec<_> = services
    .iter()
    .map(|s| discover_service(s))
    .collect();

let results = join_all(futures).await;
```

**Deep Debt Opportunity**:
- **Current**: Mostly async, some sequential patterns
- **Target**: Fully concurrent, racing patterns
- **Benefit**: Faster discovery, better responsiveness!

---

### Objective #4: Platform-Specific Optimizations (Android Fast Paths)
**Priority**: 🟢 MEDIUM  
**Estimated**: 1-2 days  
**Deliverable**: Android-specific performance optimizations

**Strategy**:
```rust
// Universal fallback (always works)
#[cfg(not(target_os = "android"))]
pub async fn discover_services() -> Result<Vec<Service>> {
    // Standard mDNS discovery
    mdns::discover().await
}

// Android fast path (use Android-specific APIs)
#[cfg(target_os = "android")]
pub async fn discover_services() -> Result<Vec<Service>> {
    // Use Android NSD (Network Service Discovery) API
    // Much faster and more battery-efficient than generic mDNS
    android_nsd::discover().await
}
```

**Areas for Platform Optimization**:
1. **Android NSD**: Use native Android network service discovery
2. **Linux io_uring**: High-performance async I/O for IPC
3. **Windows IOCP**: Native Windows async I/O completion ports
4. **macOS XPC**: Native Apple IPC (when available)

**Deep Debt Opportunity**:
- **Current**: Universal code, platform-agnostic
- **Target**: Platform-specific fast paths with fallback
- **Benefit**: Best performance on each platform!

---

## 📋 Week 3 Task Breakdown

### Day 1-2: Self-Extracting genomeBin Wrapper
**Owner**: Songbird Team  
**Priority**: 🔴 CRITICAL

**Tasks**:
1. Create `songbird.genome` wrapper template
2. Implement architecture & platform detection
3. Embed all compiled binaries in tar.gz
4. Add self-extraction logic
5. Add health check validation
6. Add rollback on failure
7. Test on Linux x86_64 (primary)
8. Test on Linux musl (USB Live Spore)
9. Document usage & customization

**Deliverables**:
- `songbird.genome` (self-extracting, multi-arch)
- `deployment/genome/README.md` (guide)
- `deployment/genome/create_genome.sh` (builder script)

---

### Day 2-4: neuralAPI Graph Integration
**Owner**: Songbird Team + biomeOS Coordination  
**Priority**: 🔴 CRITICAL

**Tasks**:
1. Create `songbird_deploy.toml` (single instance)
2. Create `tower_genome.toml` (BearDog + Songbird)
3. Create `cross_platform_deploy.toml` (USB + Android)
4. Implement health check hooks (post-deployment)
5. Implement rollback mechanism (on failure)
6. Test via biomeOS neuralAPI (requires biomeOS integration)
7. Validate atomic deployment
8. Validate rollback on simulated failure
9. Document graph patterns & examples

**Deliverables**:
- `deployment/graphs/songbird_deploy.toml`
- `deployment/graphs/tower_genome.toml`
- `deployment/graphs/cross_platform_deploy.toml`
- `deployment/graphs/README.md` (graph guide)

**Blockers**:
- ⚠️ Requires biomeOS neuralAPI graph executor (coordinate with biomeOS team)
- ⚠️ May need to wait for biomeOS genomeBin evolution

---

### Day 3-5: Deep Debt Evolution (Async/Concurrent)
**Owner**: Songbird Team  
**Priority**: 🟡 HIGH (Ongoing)

**Tasks**:
1. Audit all blocking operations (file I/O, network)
2. Replace blocking with async equivalents
3. Implement concurrent discovery racing
4. Parallel mDNS service queries
5. STUN server racing (first to respond)
6. Dark Forest endpoint parallel probing
7. Benchmark improvements (before/after)
8. Document async/concurrent patterns

**Deliverables**:
- `ASYNC_EVOLUTION_COMPLETE_JAN_31_2026.md` (report)
- Benchmarks showing improvements
- Updated patterns documentation

---

### Day 4-6: Platform-Specific Optimizations
**Owner**: Songbird Team  
**Priority**: 🟢 MEDIUM

**Tasks**:
1. Research Android NSD API
2. Implement Android-specific discovery fast path
3. Benchmark Android NSD vs generic mDNS
4. Add Linux io_uring support (if beneficial)
5. Document platform-specific optimizations
6. Document fallback behavior

**Deliverables**:
- Android NSD integration
- Platform optimization guide
- Benchmarks (platform comparison)

---

## 🚀 Success Metrics

### genomeBin Week 3 Complete When:

1. ✅ `songbird.genome` created and tested
2. ✅ Self-extraction works on Linux x86_64
3. ✅ Self-extraction works on Linux musl (USB)
4. ✅ neuralAPI deployment graphs created
5. ✅ Health check hooks implemented
6. ✅ Rollback mechanism implemented
7. ✅ Graph deployment tested (via biomeOS)
8. ✅ Concurrent discovery patterns implemented
9. ✅ Benchmarks show improvements
10. ✅ Documentation complete

### Demonstration Scenarios:

**Scenario 1: One-Command USB Deployment**
```bash
# Download and deploy to USB in one command
curl https://biomeos.org/songbird.genome > /media/usb/songbird.genome
chmod +x /media/usb/songbird.genome
/media/usb/songbird.genome
# → Auto-detects musl target, self-extracts, runs!
```

**Scenario 2: Graph-Orchestrated TOWER Deployment**
```bash
# Deploy BearDog + Songbird via neuralAPI graph
biomeos deploy --graph tower_genome.toml
# → Atomic deployment: BearDog first, then Songbird
# → Health checks validate both
# → Capabilities wired automatically
```

**Scenario 3: Cross-Platform Deployment**
```bash
# Deploy to USB and Android simultaneously
biomeos deploy --graph cross_platform_deploy.toml
# → Parallel deployment to both targets
# → Establish Dark Forest handshake
# → Validate federation
```

---

## 🎓 Deep Debt Philosophy

### What is Deep Debt?
**Deep debt** is systemic architectural debt that requires comprehensive refactoring rather than surface-level fixes.

### Examples in This Plan:

**1. Self-Extracting genomeBin (Deep Debt Solution)**
- **Surface Fix**: Keep 4 separate deployment scripts
- **Deep Solution**: One universal self-extracting archive
- **Benefit**: Eliminates deployment complexity, works everywhere

**2. neuralAPI Graph Integration (Deep Debt Solution)**
- **Surface Fix**: Shell scripts with manual error handling
- **Deep Solution**: Declarative graphs with atomic operations
- **Benefit**: Reversible, auditable, orchestrated deployments

**3. Concurrent Discovery (Deep Debt Solution)**
- **Surface Fix**: Make existing sequential code faster
- **Deep Solution**: Fundamentally concurrent architecture
- **Benefit**: Parallel discovery, racing patterns, faster overall

**4. Platform Optimizations (Deep Debt Solution)**
- **Surface Fix**: Tune universal code for Android
- **Deep Solution**: Platform-specific fast paths with fallback
- **Benefit**: Best performance on each platform

---

## 📊 Current Blockers & Dependencies

### Blocker #1: biomeOS neuralAPI Graph Executor
**Impact**: Can't test graph deployment without biomeOS  
**Workaround**: Create graphs speculatively, test manually  
**Timeline**: Wait for biomeOS genomeBin evolution  
**Priority**: 🔴 CRITICAL

### Blocker #2: Physical Android Hardware
**Impact**: Can't validate Android deployment on real device  
**Workaround**: Build for Android, test on Linux/musl  
**Timeline**: Requires hardware access  
**Priority**: 🟡 HIGH (Deferred until hardware available)

### Blocker #3: macOS Cross-Compilation Toolchain
**Impact**: Can't build macOS binaries on Linux  
**Workaround**: Defer macOS target to Phase 2  
**Timeline**: Requires osxcross setup  
**Priority**: 🟢 MEDIUM (Phase 2)

---

## 🎯 Next Actions

### Immediate (Start Now):
1. **Create genomeBin wrapper template** (Day 1)
2. **Implement self-extraction logic** (Day 1-2)
3. **Create neuralAPI deployment graphs** (Day 2-3)

### Week 3 Goals:
- ✅ `songbird.genome` working on Linux x86_64 + musl
- ✅ neuralAPI graphs created (3 scenarios)
- ✅ Health check & rollback implemented
- ✅ Concurrent discovery patterns evolved
- ✅ Documentation complete

### Week 4 Goals (If Needed):
- ✅ biomeOS integration testing (graph deployment)
- ✅ Android NSD optimization
- ✅ Platform-specific fast paths
- ✅ Comprehensive benchmarks

---

## 📚 Documentation Deliverables

### New Files Created (Week 3):
1. `deployment/genome/songbird.genome` - Self-extracting wrapper
2. `deployment/genome/README.md` - genomeBin guide
3. `deployment/genome/create_genome.sh` - Builder script
4. `deployment/graphs/songbird_deploy.toml` - Single deploy graph
5. `deployment/graphs/tower_genome.toml` - TOWER deploy graph
6. `deployment/graphs/cross_platform_deploy.toml` - Cross-platform graph
7. `deployment/graphs/README.md` - Graph patterns guide
8. `ASYNC_EVOLUTION_COMPLETE_JAN_31_2026.md` - Async/concurrent report
9. `GENOMEBIN_WEEK3_COMPLETE_FEB_2026.md` - Week 3 summary

### Updated Files (Week 3):
1. `ROOT_DOCS_INDEX.md` - Add Week 3 achievements
2. `ROADMAP.md` - Update genomeBin status
3. `STATUS.md` - Update build matrix

---

## 🎊 Vision: TRUE genomeBin Songbird

**One Month from Now**:

```bash
# Single command, works everywhere
curl https://biomeos.org/songbird.genome | sh

# Or via neuralAPI
biomeos deploy --graph nucleus.toml  # Deploy all 5 primals!
```

**Characteristics**:
- ✅ Multi-architecture (x86_64, ARM64, RISC-V)
- ✅ Self-deploying (one file, auto-extracts)
- ✅ Graph-orchestrated (atomic, reversible)
- ✅ Fully async/concurrent (racing patterns)
- ✅ Platform-optimized (fast paths per OS)
- ✅ Production-ready (health checks, rollback)

**Result**: TRUE genomeBin - Universal, autonomous, optimized! 🧬🚀

---

**Status**: Ready to Execute  
**Expected Completion**: Week 3 (6-7 days)  
**Blockers**: biomeOS neuralAPI integration (can proceed with graphs)  
**Impact**: REVOLUTIONARY - One file deploys everywhere!

**Created**: January 31, 2026 (Evening)  
**Last Updated**: January 31, 2026
