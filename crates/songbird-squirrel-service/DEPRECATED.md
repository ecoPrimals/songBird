# ⚠️ DEPRECATED: songbird-squirrel-service

**Deprecation Date**: January 16, 2026  
**Reason**: TRUE PRIMAL Architecture Violation  
**Status**: ⛔ **DO NOT USE**  
**Migration**: Use separate Squirrel primal from `phase1/squirrel/`

---

## 🚨 Why This Is Deprecated

This crate **violates TRUE PRIMAL architecture principles**:

### TRUE PRIMAL Principle:
> "Primal code only has self-knowledge and discovers other primals at runtime"

### Violation:
- ❌ Squirrel embedded inside Songbird codebase
- ❌ Hardcoded dependency (Songbird spawns Squirrel)
- ❌ Prevents independent deployment
- ❌ Violates primal autonomy

---

## ✅ Correct Architecture

### Separate Primals (TRUE PRIMAL):
```
phase1/squirrel/   ← Canonical Squirrel primal (AI/MCP)
  - Independent deployment
  - Discovers Songbird at runtime
  - Self-managed lifecycle

phase1/songbird/   ← Songbird primal (Discovery)
  - Independent deployment  
  - Discovers Squirrel via registration
  - No embedded primals
```

### Runtime Discovery:
```rust
// Squirrel discovers Songbird
let songbird_endpoint = env::var("SONGBIRD_ENDPOINT")?;
let discovery_client = connect_to_songbird(songbird_endpoint).await?;

// Squirrel registers capabilities
discovery_client.register_capabilities(vec!["ai", "mcp", "llm"]).await?;

// Songbird accepts registration (no hardcoded knowledge!)
// Communication via JSON-RPC over Unix sockets
```

---

## 🔧 Migration Guide

### For BiomeOS/Neural API Deployments:

**OLD (Embedded, DO NOT USE)**:
```toml
# songbird-orchestrator spawns embedded squirrel
# This creates /tmp/squirrel-squirrel.sock
# ❌ ARCHITECTURE VIOLATION
```

**NEW (Separate, TRUE PRIMAL)**:
```toml
# Add Squirrel as separate primal to deployment graph
[[nodes]]
id = "launch_squirrel"
node_type = "primal.launch"
description = "Launch Squirrel (AI/MCP primal)"
depends_on = ["launch_songbird"]

[nodes.config]
primal_name = "squirrel"
binary_path = "plasmidBin/primals/squirrel"  # From phase1/squirrel!
socket_path = "/tmp/squirrel-nat0.sock"
family_id = "nat0"
capabilities = ["ai", "mcp", "llm"]
environment = { SONGBIRD_ENDPOINT = "http://localhost:8080" }
```

### Build Squirrel from Canonical Source:

```bash
# NOT THIS (embedded)
cd phase1/songbird
cargo build --release --bin squirrel  # ❌ Builds embedded version

# THIS (canonical)
cd phase1/squirrel
cargo build --release --bin squirrel  # ✅ Builds canonical primal
```

---

## 📊 Impact of Migration

### Benefits:
- ✅ TRUE PRIMAL architecture validated
- ✅ Independent deployment & scaling
- ✅ Proper primal boundaries
- ✅ Runtime discovery proven
- ✅ No hardcoded dependencies

### Affected Components:
- ❌ Remove from Songbird workspace
- ❌ Remove Squirrel spawning code from Songbird orchestrator
- ✅ Deploy Squirrel from `phase1/squirrel/` repo
- ✅ Configure runtime discovery

---

## 🎯 Timeline

- **Jan 16, 2026**: Marked as DEPRECATED
- **Q1 2026**: Remove from workspace (after migration complete)
- **Q2 2026**: Delete crate entirely

---

## 📞 Support

### Questions?
- **Canonical Squirrel**: See `phase1/squirrel/` repository
- **Migration Help**: See `SONGBIRD_ARCHITECTURE_VIOLATION_JAN_16_2026.md`
- **BiomeOS Integration**: See `phase2/biomeOS/docs/`

### Already Using This?
You need to migrate to the separate Squirrel primal. This embedded version will be removed.

---

## 🏆 TRUE PRIMAL Architecture

**Each primal**:
- Has self-knowledge only ✅
- Discovers others at runtime ✅
- Independent lifecycle ✅
- Capability-based communication ✅

**No primal**:
- Embeds another primal ❌
- Hardcodes primal dependencies ❌
- Manages other primals' lifecycles ❌

---

⚠️ **DO NOT USE THIS CRATE**  
✅ **USE**: `phase1/squirrel/` (canonical Squirrel primal)

**Deprecated**: January 16, 2026  
**Removal**: Q2 2026

