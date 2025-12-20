# Eastgate ↔ Westgate Verification & Cleanup Plan

**Date:** December 20, 2025  
**Status:** Testing identity routing between towers  

---

## 🎯 Current Status

### Verified Working:
- ✅ **Westgate**: Clean, running v3.0, showing `westgate (526c1e31-2f2...)`
- ✅ **Discovery Protocol v3.0**: Human-readable names working
- ✅ **Identity-Based Routing**: Code implemented and compiled

### Issues:
- ⚠️ **Eastgate**: Process running but HTTP not responding
- ⚠️ **Strandgate**: Having process cleanup issues

---

## 📋 Verification Commands

### On Eastgate (Local):

```bash
# 1. Kill all processes
pkill -9 songbird-orchestrator
sleep 3

# 2. Verify clean
ps aux | grep songbird-orchestrator | grep -v grep

# 3. Start fresh
cd /home/eastgate/Development/ecoPrimals/songbird
./target/release/songbird-orchestrator &

# 4. Wait for startup
sleep 20

# 5. Check health
curl -sk https://localhost:8080/health

# 6. Check federation
curl -sk https://localhost:8080/api/federation/status | \
  jq -r '.nodes[] | "\(.node_name) (\(.node_id[:12])...)"'
```

### From Westgate (Remote Check):

```bash
# Check if Westgate can see Eastgate
curl -sk https://localhost:8080/api/federation/status | \
  jq -r '.nodes[] | "\(.node_name) (\(.node_id[:12])...)"'

# Should see:
# westgate (526c1e31...)
# eastgate (SOME-UUID...)
# Maybe pop-os if Strandgate is up
```

### Cross-Tower Verification:

```bash
# From Eastgate → Westgate
curl -sk https://192.168.1.123:8080/health

# From Westgate → Eastgate
curl -sk https://192.168.1.144:8080/health
curl -sk https://192.168.1.185:8080/health  # WiFi interface
```

---

## ✅ Success Criteria (Eastgate ↔ Westgate)

- [ ] Eastgate HTTP server responds to `curl https://localhost:8080/health`
- [ ] Westgate sees Eastgate in federation with stable node_id
- [ ] Eastgate sees Westgate in federation with stable node_id
- [ ] Both towers agree on the same node_id for each node
- [ ] Node names are human-readable (not "peer-XXXXXXXX")
- [ ] Eastgate shows 2 endpoints (Ethernet + WiFi) if both active
- [ ] Total nodes: 2-3 (eastgate, westgate, maybe pop-os)

---

## 🧹 Cleanup Tasks (After Verification)

### 1. Remove Old Scripts

```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Remove deprecated network scripts
rm -f setup-network-sovereignty.sh
rm -f verify_secure_federation.sh
rm -f connect_to_westgate.sh

# Keep these diagnostic docs for reference:
# - NETWORK_CONNECTIVITY_DEEP_DEBT_DEC_20_2025.md
# - DISCOVERY_VERIFICATION_FIX_DEC_20_2025.md
# - FEDERATION_BREAKTHROUGH_SESSION_TTL_BUG_DEC_20_2025.md
# - IDENTITY_BASED_ROUTING_DEC_20_2025.md
# - MULTI_PATH_TRANSPORT_ARCHITECTURE_DEC_20_2025.md
```

### 2. Remove Obsolete Code

Files to consider removing or refactoring:

```bash
# Check for old network binding code
git log --oneline --all --grep="manual.*bind" | head -10

# Check for old firewall scripts
find . -name "*firewall*" -o -name "*iptables*" | grep -v ".git"

# Check for old session-based discovery code (if any)
grep -r "peer-[0-9a-f]" --include="*.rs" crates/
```

### 3. Update Documentation

- [ ] Update README.md to reflect identity-based routing
- [ ] Remove references to manual `iptables` configuration
- [ ] Update quickstart to show zero-config setup
- [ ] Archive old session-based discovery docs

### 4. Git Commit Strategy

```bash
# After cleanup, commit the evolution
git add -A
git commit -m "feat: Identity-based routing with multi-path transport

- Discovery v3.0: Stable node identities
- Federation coalescence: Multiple interfaces = 1 logical node
- Identity-based routing layer: Route by node_id, not IP
- Human-readable names: 'eastgate' not 'peer-4ec224f8'
- Removed deprecated scripts and manual network configs

Closes: Multi-interface identity problem
Enables: Subsystem support, birdsong protocol
"
```

---

## 🐛 Troubleshooting

### If Eastgate HTTP not responding:

```bash
# Check if port is in use
sudo lsof -i :8080

# Check logs
tail -100 /tmp/eastgate_songbird.log

# Check for binding errors
journalctl -xe | grep songbird | tail -20

# Nuclear option: reboot Eastgate
# (should not be necessary with clean process management)
```

### If nodes still showing duplicate node_ids:

This means NodeIdentity isn't reading from disk correctly. Check:

```bash
# Verify machine-id exists
cat /etc/machine-id

# Check if node identity file was created
cat ~/.local/share/songbird/node_identity.json | jq .

# Check startup logs for identity generation
grep -i "node_id\|machine-id" /tmp/eastgate_songbird.log
```

---

## 📊 Expected Final State

### Federation View (from any tower):

```json
{
  "active_nodes": 3,
  "nodes": [
    {
      "node_name": "eastgate",
      "node_id": "526c1e31-2f21-5abc-...",
      "endpoints": [
        {
          "interface_type": "ethernet",
          "address": "192.168.1.144:8080",
          "preference": 200,
          "status": "Active"
        },
        {
          "interface_type": "wifi",
          "address": "192.168.1.185:8080",
          "preference": 100,
          "status": "Active"
        }
      ]
    },
    {
      "node_name": "westgate",
      "node_id": "6e0ef1ad-2426-5def-...",
      "endpoints": [
        {
          "interface_type": "ethernet",
          "address": "192.168.1.123:8080",
          "preference": 200,
          "status": "Active"
        }
      ]
    },
    {
      "node_name": "pop-os",
      "node_id": "496fe99e-0c82-5ghi-...",
      "endpoints": [...]
    }
  ]
}
```

### Key Indicators:
- ✅ 3 logical nodes (not 4+ with duplicates)
- ✅ Human-readable names
- ✅ Stable UUIDs (persist across restarts)
- ✅ Eastgate with 2 endpoints coalesced
- ✅ Zero manual configuration needed

---

## 🎉 What We've Achieved

1. **Identity-Based Routing**: Route by node_id, not IP address
2. **Multi-Interface Coalescence**: Ethernet + WiFi = 1 logical node
3. **Discovery Protocol v3.0**: Stable node identities with human names
4. **Network Sovereignty**: Pure Rust, no manual `iptables` or `sudo`
5. **Foundation for Subsystems**: Multiple Songbirds per tower ready
6. **Birdsong Protocol Ready**: Identity abstraction layer complete

---

**Next:** Verify Eastgate ↔ Westgate communication, then clean up codebase.


