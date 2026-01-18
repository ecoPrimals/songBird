# UniBin Migration Plan - Songbird

**Status**: 🔴 **HIGH PRIORITY DEBT**  
**Compliance**: ❌ **NON-COMPLIANT**  
**Target**: Week 4 Migration  
**Authority**: WateringHole Consensus (Ecosystem Standard)

---

## 📋 **Current Status**

### **Songbird's Current State**

**Binary Name**: `songbird-orchestrator` ❌  
**Target Name**: `songbird` ✅  
**Priority**: 🔴 **HIGH** (per ecosystem standard)

**Current Structure**:
```toml
[[bin]]
name = "songbird-orchestrator"  # ❌ Non-compliant!
path = "src/main.rs"
```

**Issues**:
- ❌ Binary has `-orchestrator` suffix
- ❌ Not UniBin architecture
- ❌ Deployment fragility (hardcoded binary name)
- ❌ Inconsistent with ecosystem standard

---

## 🎯 **Compliance Requirements**

Per **UniBin Architecture v1.0.0** (Ecosystem Standard):

### **Mandatory Requirements**

1. **Binary Naming**: ✅ Single binary named `songbird`
2. **Subcommand Structure**: ⚠️ Need to implement
3. **Help Documentation**: ⚠️ Need comprehensive `--help`
4. **Version Information**: ⚠️ Need `--version`
5. **Error Messages**: ⚠️ Need helpful errors

### **Minimum Required Modes**

- `server` or `service`: Long-running orchestrator ✅ (exists as default)
- `--help`: Show all commands ⚠️ (basic exists, needs expansion)
- `--version`: Show version ⚠️ (basic exists, needs expansion)

### **Recommended Optional Modes**

- `cli`: Interactive CLI mode (consider `songbird-cli` integration)
- `doctor`: Health check/diagnostics
- `config`: Configuration management

---

## 🏗️ **Migration Architecture**

### **Target Structure**

```
songbird                    # UniBin executable
├── server                  # Orchestrator mode (default, current main.rs)
├── cli                     # Interactive CLI (from songbird-cli crate)
├── doctor                  # Health diagnostics (NEW)
├── config                  # Configuration management (NEW)
└── --help/--version        # Standard info commands
```

### **Subcommand Mapping**

| Mode | Purpose | Source | Status |
|------|---------|--------|--------|
| `server` | Main orchestrator service | current `main.rs` | ✅ Exists |
| `cli` | Interactive CLI | `songbird-cli` crate | ⚠️ Integrate |
| `doctor` | Health checks | NEW implementation | ❌ Create |
| `config` | Config management | NEW implementation | ❌ Create |

---

## 📐 **Implementation Plan**

### **Phase 1: Assessment & Design** (2-4 hours)

**Tasks**:
- [x] Review current binary structure ✅
- [x] Study UniBin standard ✅
- [x] Design subcommand structure ✅
- [ ] Plan `songbird-cli` integration
- [ ] Design `doctor` mode
- [ ] Design `config` mode

**Deliverables**:
- [x] Migration plan document (this file)
- [ ] Subcommand specification
- [ ] Integration architecture

---

### **Phase 2: Core UniBin Implementation** (4-6 hours)

**Tasks**:

#### **2.1 Rename Binary** (30 min)
```toml
# crates/songbird-orchestrator/Cargo.toml
[[bin]]
name = "songbird"  # ✅ UniBin compliant!
path = "src/main.rs"
```

#### **2.2 Implement Subcommand Structure** (2-3 hours)

**Using clap (recommended)**:
```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Network Orchestration & Discovery Primal", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird orchestrator in server mode
    Server {
        #[arg(long, short, default_value = "8080")]
        port: u16,
        
        #[arg(long)]
        daemon: bool,
        
        #[arg(long)]
        config: Option<String>,
    },
    
    /// Interactive CLI mode
    Cli {
        #[arg(long)]
        endpoint: Option<String>,
    },
    
    /// Run health diagnostics
    Doctor {
        #[arg(long)]
        comprehensive: bool,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Validate configuration
    Validate,
    /// Generate default config
    Init,
}
```

#### **2.3 Implement Server Mode** (1-2 hours)
- Move current `main()` logic to `run_server()` function
- Add proper signal handling
- Add startup logging with mode and version

#### **2.4 Implement Help & Version** (30 min)
- Comprehensive `--help` output
- `--version` with build info
- Helpful error messages

**Deliverables**:
- [ ] Renamed binary (`songbird`)
- [ ] Clap-based subcommand structure
- [ ] Server mode refactored
- [ ] Help & version implemented

---

### **Phase 3: Additional Modes** (4-6 hours)

#### **3.1 CLI Mode Integration** (2-3 hours)

**Option A: Embed `songbird-cli`**
```rust
Commands::Cli { endpoint } => {
    // Use songbird-cli crate as library
    songbird_cli::run_interactive(endpoint).await?;
}
```

**Option B: Proxy to `songbird-cli`** (if kept separate)
```rust
Commands::Cli { endpoint } => {
    println!("Interactive CLI mode - use 'songbird-cli' for full features");
    println!("Or install: cargo install songbird-cli");
}
```

**Recommendation**: Option A (embed) for UniBin compliance

#### **3.2 Doctor Mode** (1-2 hours)

```rust
Commands::Doctor { comprehensive } => {
    run_doctor(comprehensive).await?;
}

async fn run_doctor(comprehensive: bool) -> Result<()> {
    println!("🏥 Songbird Health Diagnostics\n");
    
    // Check binary
    println!("✅ Binary: songbird v{}", env!("CARGO_PKG_VERSION"));
    
    // Check config
    if let Ok(config) = load_config() {
        println!("✅ Configuration: Valid");
    } else {
        println!("❌ Configuration: Invalid or missing");
    }
    
    // Check sockets
    check_socket_availability().await?;
    
    // Check dependencies (BearDog, etc.)
    if comprehensive {
        check_primal_connectivity().await?;
    }
    
    // Check ports
    check_port_availability().await?;
    
    println!("\n✅ Health check complete!");
    Ok(())
}
```

#### **3.3 Config Mode** (1 hour)

```rust
Commands::Config { config_cmd } => {
    match config_cmd {
        ConfigCommands::Show => show_config().await?,
        ConfigCommands::Validate => validate_config().await?,
        ConfigCommands::Init => init_config().await?,
    }
}
```

**Deliverables**:
- [ ] CLI mode integrated
- [ ] Doctor mode implemented
- [ ] Config mode implemented

---

### **Phase 4: Testing** (3-4 hours)

**Test Coverage**:

#### **4.1 Subcommand Tests**
```rust
#[test]
fn test_server_mode() {
    let cli = Cli::parse_from(["songbird", "server", "--port", "9000"]);
    assert!(matches!(cli.command, Commands::Server { port: 9000, .. }));
}

#[test]
fn test_help_output() {
    // Verify --help shows all modes
}

#[test]
fn test_version_output() {
    // Verify --version format
}
```

#### **4.2 Integration Tests**
- Test each mode starts correctly
- Test signal handling
- Test error messages
- Test backward compatibility (if needed)

#### **4.3 E2E Tests**
```bash
# Test server mode
songbird server --port 8080 &
sleep 2
curl http://localhost:8080/health
kill %1

# Test doctor mode
songbird doctor --comprehensive

# Test config mode
songbird config show
songbird config validate
```

**Deliverables**:
- [ ] Unit tests for subcommands
- [ ] Integration tests
- [ ] E2E test script

---

### **Phase 5: Documentation & Deployment** (2-3 hours)

#### **5.1 Update Documentation** (1 hour)

**Files to Update**:
- [ ] `README.md` - Update binary name and usage
- [ ] `QUICK_START.md` - Update commands
- [ ] `docs/` - Update all references
- [ ] Examples - Update binary name

**Example Updates**:
```markdown
# OLD
Run Songbird:
    cargo run --bin songbird-orchestrator

# NEW  
Run Songbird:
    songbird server --port 8080
    songbird doctor
    songbird --help
```

#### **5.2 Update Deployment Graphs** (1 hour)

**BiomeOS Graph Update**:
```toml
# OLD (non-compliant)
[[nodes]]
id = "launch_songbird"
[nodes.config]
binary_path = "plasmidBin/primals/songbird-orchestrator"

# NEW (UniBin compliant!)
[[nodes]]
id = "launch_songbird"
[nodes.config]
primal_name = "songbird"
binary_path = "plasmidBin/primals/songbird"
mode = "server"
args = ["server", "--daemon", "--port", "8080"]
```

#### **5.3 Migration Guide** (30 min)

Create `UNIBIN_MIGRATION_GUIDE.md`:
- Old vs new commands
- Backward compatibility notes
- Troubleshooting

#### **5.4 CI/CD Updates** (30 min)
- Update build scripts
- Update test scripts
- Update deployment scripts

**Deliverables**:
- [ ] Documentation updated
- [ ] Deployment graphs updated
- [ ] Migration guide created
- [ ] CI/CD updated

---

### **Phase 6: Verification & Rollout** (2-3 hours)

#### **6.1 Compliance Verification**

**UniBin Checklist**:
- [ ] Single binary named `songbird` (no suffixes)
- [ ] Subcommand structure implemented (clap)
- [ ] `--help` shows all modes with descriptions
- [ ] `--version` implemented
- [ ] At least `server` mode exists
- [ ] Error messages helpful and actionable
- [ ] Logging includes mode and version
- [ ] Signal handling (graceful shutdown)
- [ ] Documentation updated with CLI examples
- [ ] Deployment graphs updated to UniBin pattern
- [ ] Tests cover all modes
- [ ] Old binary name removed

#### **6.2 Build & Test**
```bash
# Build UniBin
cargo build --release

# Verify binary name
ls target/release/songbird  # ✅ Should exist

# Test modes
./target/release/songbird --help
./target/release/songbird --version
./target/release/songbird server --help
./target/release/songbird doctor
./target/release/songbird config show

# Run tests
cargo test
```

#### **6.3 Deployment Test**
- Deploy to test environment
- Verify all modes work
- Check logs for mode/version
- Verify graceful shutdown

**Deliverables**:
- [ ] Compliance checklist complete
- [ ] Build successful
- [ ] All tests passing
- [ ] Deployment verified

---

## 📊 **Timeline & Effort**

### **Total Effort Estimate**

| Phase | Tasks | Hours | Priority |
|-------|-------|-------|----------|
| Phase 1 | Assessment & Design | 2-4h | High |
| Phase 2 | Core UniBin | 4-6h | High |
| Phase 3 | Additional Modes | 4-6h | Medium |
| Phase 4 | Testing | 3-4h | High |
| Phase 5 | Docs & Deployment | 2-3h | Medium |
| Phase 6 | Verification | 2-3h | High |
| **Total** | | **17-26h** | |

**Recommended Timeline**: **Week 4** (4-5 working days)

---

## 🎯 **Success Criteria**

### **Technical**
- ✅ Binary named `songbird` (no suffix)
- ✅ All subcommands work correctly
- ✅ Tests passing (100%)
- ✅ Documentation updated
- ✅ Deployment graphs updated
- ✅ CI/CD updated

### **Compliance**
- ✅ All UniBin mandatory requirements met
- ✅ Reference implementation quality
- ✅ WateringHole approval

### **User Experience**
- ✅ Professional CLI (`--help`, `--version`)
- ✅ Clear error messages
- ✅ Consistent with NestGate (reference)
- ✅ Easy to learn and use

---

## 🚨 **Risks & Mitigation**

### **Risk 1: Breaking Changes**
**Impact**: High  
**Probability**: Medium  
**Mitigation**:
- Symlink `songbird-orchestrator` → `songbird` during transition
- Clear migration guide
- Deprecation warnings

### **Risk 2: Integration Complexity**
**Impact**: Medium  
**Probability**: Medium  
**Mitigation**:
- Start with core modes only
- Phase additional modes
- Thorough testing

### **Risk 3: Deployment Coordination**
**Impact**: High  
**Probability**: Low  
**Mitigation**:
- Update BiomeOS graphs first
- Coordinate with team
- Staged rollout

---

## 📝 **Notes**

### **Backward Compatibility**

**Option 1: Symlink** (Recommended)
```bash
ln -s songbird songbird-orchestrator
```

**Option 2: Wrapper Script**
```bash
#!/bin/bash
# songbird-orchestrator wrapper
exec songbird server "$@"
```

**Option 3: Hard Cutover**
- Update all references immediately
- Clear communication

**Recommendation**: Symlink for 1-2 releases, then remove

---

### **CLI Integration Decision**

**Question**: Should `songbird-cli` be embedded or stay separate?

**Pros of Embedding**:
- ✅ UniBin compliance (single binary)
- ✅ Consistent UX
- ✅ Easier deployment

**Cons of Embedding**:
- ❌ Larger binary size
- ❌ More complex codebase
- ❌ Dependency management

**Recommendation**: 
- **Short-term**: Keep separate, add symlink
- **Long-term**: Embed as subcommand (Week 5-6)

---

### **Doctor Mode Ideas**

**Basic Checks**:
- Binary version
- Configuration validity
- Socket availability
- Port availability

**Comprehensive Checks** (`--comprehensive`):
- BearDog connectivity
- Squirrel connectivity
- ToadStool connectivity
- NestGate connectivity
- Network configuration
- System resources
- Permissions

---

## 🔗 **Related Work**

### **Dependencies**
- **None** - UniBin is self-contained

### **Blockers**
- **None** - Ready to start

### **Unblocks**
- Squirrel integration (cleaner interface)
- BiomeOS deployment (robust graphs)
- Professional UX (ecosystem consistency)

---

## 📚 **References**

### **Ecosystem Standard**
- **UniBin Architecture v1.0.0**: `/wateringHole/UNIBIN_ARCHITECTURE_ECOSYSTEM_STANDARD_JAN_16_2026.md`
- **Reference Implementation**: NestGate v0.11.0+
- **BiomeOS Implementation**: `/biomeOS/UNIBIN_DEBT_ELIMINATION_JAN_16_2026.md`

### **Technical Resources**
- **Clap Documentation**: https://docs.rs/clap/
- **Subcommand Pattern**: NestGate source code

---

## ✅ **Next Actions**

### **Immediate** (this session)
- [x] Create migration plan document ✅
- [x] Assess current state ✅
- [x] Design architecture ✅
- [ ] Document as technical debt
- [ ] Notify team

### **Week 4** (next session)
- [ ] Execute Phase 1: Assessment & Design
- [ ] Execute Phase 2: Core UniBin Implementation
- [ ] Execute Phase 3: Additional Modes
- [ ] Execute Phase 4: Testing
- [ ] Execute Phase 5: Docs & Deployment
- [ ] Execute Phase 6: Verification & Rollout

### **Post-Migration**
- [ ] Submit WateringHole compliance report
- [ ] Share learnings with ecosystem
- [ ] Update this document with lessons learned

---

## 🎊 **Conclusion**

**UniBin Migration** is a **HIGH PRIORITY** ecosystem debt that will:

✅ **Align** Songbird with ecosystem standard  
✅ **Improve** deployment robustness  
✅ **Enhance** professional UX  
✅ **Reduce** technical debt  
✅ **Enable** consistent CLI across ecosystem

**Timeline**: Week 4 (17-26 hours)  
**Priority**: 🔴 HIGH  
**Status**: Ready to execute

---

**Plan**: UniBin Migration - Songbird  
**Created**: January 16, 2026  
**Authority**: WateringHole Consensus  
**Compliance**: Ecosystem Standard v1.0.0

🦀🎯✨ **UniBin Songbird - One Binary, Infinite Possibilities!** ✨🎯🦀

*Consistent | Robust | Professional | Maintainable*

