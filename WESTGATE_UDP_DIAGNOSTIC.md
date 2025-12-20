# Westgate UDP Discovery Diagnostic - December 20, 2025

## 🎯 Problem

**Asymmetric UDP Discovery:**
- Westgate → Eastgate: ✅ Working
- Eastgate → Westgate: ❌ Broken

**User Observation:** Eastgate is the only tower asking for password at startup (for `sudo lsof`).

## 🔍 Root Cause Hypothesis

**Westgate's firewall is blocking INBOUND UDP from eastgate.**

### Evidence

1. **Westgate broadcasts successfully** (eastgate receives 8 session IDs)
2. **Westgate doesn't see eastgate** (0 active nodes)
3. **Eastgate's broadcaster is running** (code verified, UDP port 2300 bound)
4. **Password difference is irrelevant** (`sudo lsof` is just diagnostic)

### Why This Happens

When you run `./start-tower.sh` on a fresh system:
- Songbird binds to UDP 2300 (no firewall rule yet)
- Sends broadcasts (no firewall rule needed for OUTBOUND)
- But **INBOUND UDP is blocked** by default firewall

On subsequent runs:
- If you previously added `iptables` rules, INBOUND works
- But westgate might not have those rules

## 🔧 Fix for Westgate

### On Westgate, Run:

```bash
# Allow INBOUND UDP on port 2300 (discovery)
sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT

# Allow INBOUND TCP on port 8080 (HTTPS)
sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT

# Verify rules
sudo iptables -L INPUT -n -v | grep -E "2300|8080"

# Should show:
#     0     0 ACCEPT     udp  --  *      *       0.0.0.0/0            0.0.0.0/0            udp dpt:2300
#     0     0 ACCEPT     tcp  --  *      *       0.0.0.0/0            0.0.0.0/0            tcp dpt:8080

# Save rules (persist across reboots)
sudo iptables-save | sudo tee /etc/iptables/rules.v4 > /dev/null
```

### Verify Fix

```bash
# On westgate, wait 30 seconds after adding rules, then check:
curl -k https://localhost:8080/api/federation/status | jq '{active_nodes, nodes: [.nodes[] | .node_name]}'

# Expected:
{
  "active_nodes": 1,  # Should now see eastgate!
  "nodes": ["westgate", "peer-XXXXXXXX"]  # peer-XXX is eastgate's session ID
}
```

## 🧪 Test UDP Reception

### Manual Test (On Westgate):

```bash
# Terminal 1 (westgate) - Listen for UDP:
nc -u -l 2300

# Terminal 2 (eastgate) - Send UDP:
echo "test-from-eastgate" | nc -u 192.168.1.123 2300

# If Terminal 1 shows "test-from-eastgate", UDP works!
# If nothing appears, firewall is blocking.
```

## 📊 Expected Results

### Before Fix:
```json
// Westgate federation status:
{
  "active_nodes": 0,
  "nodes": [{"node_name": "westgate"}]  // Only self
}
```

### After Fix:
```json
// Westgate federation status:
{
  "active_nodes": 1,  // Now sees eastgate!
  "nodes": [
    {"node_name": "westgate"},
    {"node_name": "peer-XXXXXXXX", "capabilities": ["orchestration", "federation"]}
  ]
}
```

## 🎓 Why This Happened

### Password Prompt Clue

**Your observation was KEY:**
- Eastgate asks for password (`sudo lsof`)
- Westgate doesn't

This suggests:
- Different system configurations
- Different firewall states
- Possibly different previous `iptables` rules

### Firewall Asymmetry

**Common Pattern:**
1. User runs Songbird on Tower A
2. Adds `iptables` rules to fix connectivity
3. Tower A works perfectly
4. User runs Songbird on Tower B (fresh install)
5. Forgets to add `iptables` rules
6. Tower B can SEND but not RECEIVE

### Why Outbound Works, Inbound Doesn't

**Linux Default Firewall (iptables/ufw):**
- **OUTBOUND**: Usually allowed by default
- **INBOUND**: Usually blocked by default (except established connections)

So:
- Westgate can SEND UDP (outbound) → eastgate receives ✅
- Eastgate can SEND UDP (outbound) → westgate blocks (inbound) ❌

## 🚀 Complete Fix Script

### Create: `fix-westgate-firewall.sh`

```bash
#!/bin/bash
# Fix westgate firewall for Songbird discovery

echo "🔧 Adding Songbird firewall rules..."

# Allow discovery (UDP 2300)
sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT
echo "✅ UDP port 2300 (discovery) allowed"

# Allow HTTPS (TCP 8080)
sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT
echo "✅ TCP port 8080 (HTTPS) allowed"

# Save rules
if [ -d /etc/iptables ]; then
    sudo mkdir -p /etc/iptables
    sudo iptables-save | sudo tee /etc/iptables/rules.v4 > /dev/null
    echo "✅ Rules saved to /etc/iptables/rules.v4"
else
    echo "⚠️  /etc/iptables not found, rules are temporary"
    echo "   Install iptables-persistent: sudo apt install iptables-persistent"
fi

echo ""
echo "🔍 Verifying rules..."
sudo iptables -L INPUT -n -v | grep -E "2300|8080"

echo ""
echo "✅ Firewall configured for Songbird!"
echo ""
echo "📋 Next steps:"
echo "  1. Wait 30 seconds for discovery"
echo "  2. Check federation: curl -k https://localhost:8080/api/federation/status | jq"
echo "  3. Should now see eastgate as a peer!"
```

### Run on Westgate:

```bash
chmod +x fix-westgate-firewall.sh
./fix-westgate-firewall.sh
```

## 💡 Prevention

### For Future Deployments

**Add to `start-tower.sh`:**
```bash
# Auto-configure firewall on first run
if ! sudo iptables -L INPUT -n | grep -q "dpt:2300"; then
    echo "🔧 Configuring firewall for first-time use..."
    sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT
    sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT
    echo "✅ Firewall rules added"
fi
```

Or use the sovereign socket's capability-based approach (already implemented).

## 🏆 Summary

**Your Observation:** Password prompt difference
**Real Issue:** Westgate firewall blocking INBOUND UDP
**Solution:** Add `iptables` rule for UDP port 2300
**Expected Result:** Both towers see each other in federation

**Time to Fix:** 30 seconds
**Commands:** 3 lines (`iptables -I INPUT ...`)

---

Run the fix on westgate, wait 30 seconds, and check the federation status!

