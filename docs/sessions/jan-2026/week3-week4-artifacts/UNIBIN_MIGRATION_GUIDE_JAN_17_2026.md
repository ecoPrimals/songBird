# UniBin Migration Guide

**For**: Songbird Users & Operators  
**Date**: January 17, 2026  
**Version**: v3.24.0  
**Status**: ✅ Complete

---

## 🎯 **What Changed?**

Songbird has migrated to **UniBin Architecture** (Ecosystem Standard v1.0.0), bringing professional CLI and improved UX.

### **Binary Rename**

**Before** (v3.23.x and earlier):
```bash
songbird-orchestrator       # ❌ Old binary name (with suffix)
```

**After** (v3.24.0+):
```bash
songbird                    # ✅ UniBin compliant (no suffix!)
```

---

## 🚀 **New Commands**

### **Server Mode** (Orchestrator)
```bash
# Start Songbird orchestrator
songbird server

# With options
songbird server --port 9000
songbird server --daemon
songbird server --verbose

# Help
songbird server --help
```

### **Health Diagnostics**
```bash
# Basic health check
songbird doctor

# Comprehensive check (includes primal connectivity)
songbird doctor --comprehensive

# JSON output
songbird doctor --format json
```

### **Configuration Management**
```bash
# Validate configuration
songbird config validate

# Show current configuration
songbird config show

# Generate configuration template
songbird config init --output songbird.toml

# Force overwrite
songbird config init --output songbird.toml --force
```

### **Help & Version**
```bash
# Show all commands
songbird --help

# Show version
songbird --version

# Help for specific command
songbird server --help
songbird doctor --help
songbird config --help
```

---

## 📋 **Migration Steps**

### **For Local Development**

**Before**:
```bash
# Build
cargo build --release --bin songbird-orchestrator

# Run
./target/release/songbird-orchestrator

# Or with cargo
cargo run --bin songbird-orchestrator
```

**After**:
```bash
# Build
cargo build --release

# Run server mode
./target/release/songbird server

# Or with cargo
cargo run -- server

# Health check
./target/release/songbird doctor

# Config management
./target/release/songbird config validate
```

---

### **For Production Deployments**

**Binary Path Change**:
```bash
# Before
/usr/local/bin/songbird-orchestrator
plasmidBin/primals/songbird-orchestrator

# After
/usr/local/bin/songbird
plasmidBin/primals/songbird
```

**Systemd Service** (if applicable):
```ini
# Before
[Service]
ExecStart=/usr/local/bin/songbird-orchestrator

# After
[Service]
ExecStart=/usr/local/bin/songbird server --daemon
```

**BiomeOS Graph**:
```toml
# Before
[[nodes]]
id = "launch_songbird"
[nodes.config]
binary_path = "plasmidBin/primals/songbird-orchestrator"

# After
[[nodes]]
id = "launch_songbird"
[nodes.config]
primal_name = "songbird"
binary_path = "plasmidBin/primals/songbird"
mode = "server"
args = ["server", "--daemon"]
```

---

## ⚠️ **Backward Compatibility**

### **Transition Period** (v3.24.0 - v3.26.0)

For smooth transition, create a symlink:

```bash
# Create symlink for old scripts
ln -s songbird songbird-orchestrator

# Or in plasmidBin
cd plasmidBin/primals/
ln -s songbird songbird-orchestrator
```

This allows old scripts/configs to work during migration.

### **Deprecation Timeline**

- **v3.24.0** (Jan 17, 2026): UniBin introduced, symlink recommended
- **v3.25.0** (Feb 2026): Deprecation warning for old name
- **v3.26.0** (Mar 2026): Symlink removed, full UniBin only

---

## ✅ **Verification**

### **Check Binary**
```bash
# Verify binary exists
ls -lh target/release/songbird

# Check version
./target/release/songbird --version
# Expected: songbird 0.1.0

# Check help
./target/release/songbird --help
# Should show: server, doctor, config subcommands
```

### **Test Modes**
```bash
# Test doctor mode
./target/release/songbird doctor
# Should show health diagnostics

# Test config mode
./target/release/songbird config validate
# Should validate configuration

# Test server mode (ctrl+c to stop)
./target/release/songbird server
# Should start orchestrator
```

---

## 💡 **Benefits**

### **Professional CLI**
- ✅ Consistent with ecosystem (like NestGate)
- ✅ Self-documenting (`--help`)
- ✅ Multiple operational modes
- ✅ Professional UX (like `kubectl`, `docker`)

### **Better Deployment**
- ✅ Mode-based graphs (robust)
- ✅ No binary naming confusion
- ✅ Easier maintenance
- ✅ Clear operational intent

### **Improved Diagnostics**
- ✅ Built-in health checks (`doctor` mode)
- ✅ Configuration validation
- ✅ Template generation
- ✅ Multiple output formats

---

## 🆘 **Troubleshooting**

### **Problem: Old binary not found**
```bash
# Error: songbird-orchestrator: command not found

# Solution: Use new name
songbird server
```

### **Problem: Scripts use old name**
```bash
# Quick fix: Create symlink
ln -s songbird songbird-orchestrator

# Long-term fix: Update scripts to use 'songbird server'
```

### **Problem: Unknown mode server**
```bash
# Error: Unknown subcommand 'server'

# Check version
songbird --version

# If < v3.24.0, update:
git pull
cargo build --release
```

### **Problem: Port in use**
```bash
# Check with doctor mode
songbird doctor

# Or specify different port
songbird server --port 9000
```

---

## 📚 **Examples**

### **Development Workflow**
```bash
# Validate config
songbird config validate

# Check health
songbird doctor

# Start server
songbird server --verbose

# In another terminal, check health
curl http://localhost:8080/health
```

### **Production Deployment**
```bash
# Generate config
songbird config init --output /etc/songbird/config.env

# Edit config
vim /etc/songbird/config.env

# Validate
songbird config validate

# Start as daemon
songbird server --daemon --port 8080
```

### **Debugging**
```bash
# Verbose server
songbird server --verbose

# Comprehensive health check
songbird doctor --comprehensive

# Show configuration
songbird config show
```

---

## 🎯 **Quick Reference**

| Old Command | New Command |
|-------------|-------------|
| `songbird-orchestrator` | `songbird server` |
| *(no equivalent)* | `songbird doctor` |
| *(no equivalent)* | `songbird config validate` |
| *(no equivalent)* | `songbird config init` |
| `songbird-orchestrator --help` | `songbird --help` |

---

## 📞 **Support**

### **Questions?**
- Check `songbird --help`
- Read `README.md`
- See `QUICK_START.md`

### **Issues?**
- Run `songbird doctor --comprehensive`
- Check logs
- Verify configuration with `songbird config validate`

### **Need More Info?**
- Documentation: `ROOT_DOCS_INDEX.md`
- Architecture: `UNIBIN_MIGRATION_PLAN_JAN_16_2026.md`
- Status: `STATUS.md`

---

**Migration Guide**: v3.24.0  
**UniBin Architecture**: Ecosystem Standard v1.0.0  
**Status**: ✅ Production Ready

🦀🎯✨ **Welcome to UniBin Songbird!** ✨🎯🦀

*Professional | Consistent | Modern*

