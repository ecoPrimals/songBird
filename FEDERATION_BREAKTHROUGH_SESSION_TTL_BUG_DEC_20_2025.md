# Federation Breakthrough + Session TTL Bug - December 20, 2025

## 🎉 BREAKTHROUGH: 4-Tower Federation Discovered!

### ✅ UDP Discovery Working

**Verified via tcpdump on westgate:**
```
192.168.1.144 → Eastgate   (confirmed broadcasting)
192.168.1.123 → Westgate   (confirmed broadcasting)
192.168.1.185 → Strandgate (confirmed broadcasting)
192.168.1.134 → Mystery    (confirmed broadcasting)
```

**All 4 towers broadcasting successfully on UDP port 2300!**

### ✅ Westgate Firewall Fix Success

**Root Cause:** Westgate's firewall was blocking INBOUND UDP
**Solution:** `sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT`
**Result:** Westgate now receives broadcasts from all towers

**User Observation Key:** Password prompt difference revealed firewall asymmetry

## 🐛 CRITICAL BUG: Session ID Accumulation

### The Problem

**Eastgate shows 69 nodes** but only **4 physical towers** exist!

```json
{
  "active_nodes": 18,
  "tower_count": 69,  ← WRONG! Should be 4!
  "towers": [/* 69 peer-XXXXXXXX entries */]
}
```

### Root Cause

**Session IDs rotate every hour** (by design for anonymity), but:
1. **Old session IDs are NOT removed**
2. **No TTL (time-to-live) cleanup**
3. **No heartbeat expiration**
4. **Federation accumulates stale entries**

### Timeline

```
Hour 1: peer-AAAAAAAA (westgate session 1)
Hour 2: peer-BBBBBBBB (westgate session 2) + peer-AAAAAAAA still registered
Hour 3: peer-CCCCCCCC (westgate session 3) + peer-AAAAAAAA + peer-BBBBBBBB
...
Hour 8: peer-HHHHHHHH (westgate session 8) + 7 stale entries
```

After running for days/weeks → hundreds of stale nodes!

### Why This Is Bad

1. **Federation state inaccurate**
   - Shows 69 nodes instead of 4
   - Can't tell which are active
   - Coordination impossible

2. **Resource waste**
   - Heartbeat attempts to stale nodes
   - Memory accumulation
   - Database bloat

3. **UX confusion**
   - Users see 69 nodes
   - Don't know which are real
   - Can't debug federation

## ✅ Solution: Session TTL Cleanup

### Design

**Node Expiration Policy:**
1. **Last Heartbeat Timeout**: Remove nodes after N minutes without heartbeat
2. **Session Rotation Tracking**: When new session detected, mark old one for removal
3. **Periodic Cleanup Task**: Every 5 minutes, remove expired nodes
4. **Grace Period**: Keep nodes for 10 minutes after last heartbeat (2x heartbeat interval)

### Implementation Plan

```rust
// In FederationState:
pub async fn cleanup_stale_nodes(&self) {
    let timeout = Duration::from_secs(600); // 10 minutes
    let now = Utc::now();
    
    let mut nodes = self.nodes.write().await;
    nodes.retain(|_, node| {
        let elapsed = now - node.last_heartbeat;
        elapsed < chrono::Duration::seconds(timeout.as_secs() as i64)
    });
    
    info!("🧹 Cleaned up stale nodes. Active: {}", nodes.len());
}

// Spawn cleanup task in orchestrator:
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 min
    loop {
        interval.tick().await;
        federation_state.cleanup_stale_nodes().await;
    }
});
```

### Expected Result

**Before Cleanup:**
```json
{
  "active_nodes": 18,
  "tower_count": 69  ← Accumulated stale sessions
}
```

**After Cleanup:**
```json
{
  "active_nodes": 3,
  "tower_count": 4  ← Only active towers!
}
```

## 📊 Federation Status Analysis

### Current State (Dec 20, 2025, 22:25 UTC)

**Eastgate (192.168.1.144):**
- Sees: 69 nodes (4 real + 65 stale sessions)
- Broadcasting: ✅ Yes
- Receiving: ✅ Yes

**Westgate (192.168.1.123):**
- Firewall: ✅ Fixed (INBOUND UDP now allowed)
- Broadcasting: ✅ Yes
- Receiving: ✅ Yes (confirmed via tcpdump)

**Strandgate (192.168.1.185):**
- Status: Just relaunched
- Broadcasting: ✅ Yes (confirmed via tcpdump)
- Receiving: ❓ Unknown (likely needs firewall fix too)

**Mystery Tower (192.168.1.134):**
- Identity: Unknown
- Broadcasting: ✅ Yes (confirmed via tcpdump)
- Receiving: ❓ Unknown

### Physical Tower Count: 4
### Registered Node Count: 69
### **Stale Session Ratio: 94%** ← CRITICAL!

## 🎯 Priority Actions

### P0 (Blocking):
1. ✅ **Fix westgate firewall** → DONE
2. ⏳ **Implement session TTL cleanup** → NEXT
3. ⏳ **Test cleanup on eastgate** → After #2

### P1 (High):
4. **Fix strandgate firewall** (likely same issue as westgate)
5. **Identify mystery tower** (192.168.1.134)
6. **Add firewall auto-configuration** to `start-tower.sh`

### P2 (Medium):
7. **Test HTTPS connectivity** between towers
8. **Verify connectivity verification fix** is working
9. **Add session rotation notification**

## 🏆 Achievements Today

1. ✅ Fixed HTTPS double-bind bug
2. ✅ Added connectivity verification before registration
3. ✅ Identified UDP discovery asymmetry
4. ✅ Fixed westgate firewall (INBOUND UDP)
5. ✅ Discovered 4-tower federation
6. ✅ Identified session TTL bug (69 stale nodes)

## 💡 User Observation Impact

**User:** "this tower is the only of teh 3 that has asked me for a password at terminal for songbird. is that useful?"

**Result:** 
- Revealed firewall configuration differences
- Led to diagnosing INBOUND UDP blocking
- Fixed westgate's discovery reception
- Discovered 4-tower network

**Grade:** 🏆 GOLD TIER observation

## 🚀 Next Step

**Implement session TTL cleanup to reduce 69 nodes → 4 nodes.**

This will make federation usable and accurate!

---

**Status:** UDP Discovery ✅ Working  
**Blocker:** Session accumulation bug  
**ETA:** 20 minutes for TTL implementation + testing

