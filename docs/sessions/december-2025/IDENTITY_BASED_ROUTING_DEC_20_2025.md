# Identity-Based Routing Evolution

**Date:** December 20, 2025  
**Status:** ✅ Implemented  
**Commit:** Phase 5 - Identity-Based Routing  

---

## 🎯 Executive Summary

Evolved Songbird from **address-centric** to **identity-based** routing, enabling:
- Stable node identities across network changes
- Multi-interface coalescence (Ethernet + WiFi = 1 logical node)
- Multiple Songbird subsystems per physical tower
- Routing by `node_id`, not IP address

---

## 🔍 Problem Identified

### Westgate Federation Status (Before Fix):
```
Node 1: westgate (192.168.1.123) ✅ - Named correctly
Node 2: peer-4ec224f8 (192.168.1.134) ❌ - Session ID!
Node 3: peer-9d5eedfc (192.168.1.185) ❌ - Session ID!
Node 4: peer-68970675 (192.168.1.123) ❌ - Duplicate westgate!
```

**Root Cause:**
- Discovery v3.0 was broadcasting `node_id` and `node_name`
- But federation bridge was still using `session_id` for registration
- Result: Peers named "peer-4ec224f8" instead of "eastgate"

---

## 🏗️ Architectural Evolution

### Before (Address-Centric):
```
Tower A → 192.168.1.123:8080 → Tower B
```
- Identity = IP Address
- Single Songbird per IP
- New interface = New node
- Session rotation = New node ID

### After (Identity-Centric):
```
Songbird A (node_id) → [encrypted birdsong] → Songbird B (node_id)
```
- Identity = Stable UUID (machine-based)
- Multiple Songbirds per IP (subsystems!)
- Multiple interfaces = Multiple paths to same node
- Routing: `node_id` → best transport path

---

## 🔧 Implementation Changes

### 1. Discovery Protocol (`DiscoveredPeer`)

**File:** `crates/songbird-discovery/src/anonymous_discovery.rs`

**Changes:**
```rust
pub struct DiscoveredPeer {
    /// v2.x: session_id (deprecated)
    pub session_id: String,
    
    /// v3.0: Stable node identity
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub endpoints: Option<Vec<TransportEndpointMessage>>,
    
    // ... rest of fields
}
```

**Impact:** Discovery listener now captures v3.0 identity fields from broadcast messages.

---

### 2. Federation Bridge (Identity Extraction)

**File:** `crates/songbird-orchestrator/src/app/mod.rs`  
**Function:** `start_discovery_federation_bridge()`

**Changes:**
```rust
// Extract identity based on protocol version
let (node_id, node_name) = if peer.version == "3.0" {
    // v3.0: Use stable node_id and node_name
    match (&peer.node_id, &peer.node_name) {
        (Some(id), Some(name)) => (id.clone(), name.clone()),
        _ => {
            // Fallback to session_id if v3.0 fields missing
            (peer.session_id.clone(), format!("peer-{}", &peer.session_id[..8]))
        }
    }
} else {
    // v2.x: Fall back to session_id (legacy)
    (peer.session_id.clone(), format!("peer-{}", &peer.session_id[..8]))
};

// Convert v3.0 endpoints to federation format
let endpoints = peer.endpoints.as_ref().map(|eps| {
    eps.iter().map(|ep| {
        TransportEndpointInfo {
            interface_type: ep.interface_type.clone(),
            address: format!("{}:{}", peer.address.ip(), ep.port),
            protocols: ep.protocols.clone(),
            preference: ep.preference,
            status: EndpointStatus::Active,
            last_check: chrono::Utc::now(),
        }
    }).collect()
});

// Create node registration with stable identity
let node_registration = NodeRegistration {
    node_id,      // ✅ Now uses stable UUID for v3.0!
    node_name,    // ✅ Now uses human-readable name!
    endpoints,    // ✅ Multi-endpoint support!
    // ... rest of fields
};
```

**Impact:** 
- v3.0 peers registered with human-readable names ("eastgate" not "peer-4ec224f8")
- Multiple endpoints tracked per logical node

---

### 3. Federation State (Endpoint Coalescence)

**File:** `crates/songbird-network-federation/src/state.rs`  
**Function:** `register_node()`

**Changes:**
```rust
pub async fn register_node(&self, registration: NodeRegistration) {
    let mut nodes = self.nodes.write().await;
    
    // Check if this node_id already exists
    if let Some(existing) = nodes.get_mut(&registration.node_id) {
        // Node exists - coalesce endpoints instead of replacing
        tracing::debug!("🔄 Coalescing endpoints for existing node '{}'", existing.node_name);
        
        // Update heartbeat and status
        existing.last_heartbeat = Utc::now();
        existing.status = NodeStatus::Active;
        
        // Merge endpoints if new registration has any
        if let Some(new_endpoints) = registration.endpoints {
            for endpoint in new_endpoints {
                existing.add_endpoint(endpoint);
            }
        }
        
        // Merge capabilities (union)
        for capability in registration.capabilities {
            if !existing.capabilities.contains(&capability) {
                existing.capabilities.push(capability);
            }
        }
    } else {
        // New node - insert
        nodes.insert(registration.node_id.clone(), registration);
    }
}
```

**Impact:**
- Same `node_id` = Same logical node (even from different IPs)
- Multiple interfaces coalesced into single federation entry
- Endpoint list grows as interfaces are discovered

---

### 4. Identity-Based Routing Layer

**File:** `crates/songbird-network-federation/src/state.rs`  
**New Methods:**

```rust
/// Get best endpoint for a node (identity-based routing)
pub async fn get_best_endpoint(&self, node_id: &str) -> Option<String> {
    let nodes = self.nodes.read().await;
    let node = nodes.get(node_id)?;
    
    // Try to get preferred endpoint (highest preference + active)
    if let Some(endpoint) = node.preferred_endpoint() {
        return Some(format!("https://{}", endpoint.address));
    }
    
    // Fall back to primary address
    Some(node.node_address.clone())
}

/// Get all endpoints for a node (for connection fallback)
pub async fn get_all_endpoints(&self, node_id: &str) -> Vec<String> {
    let nodes = self.nodes.read().await;
    let node = match nodes.get(node_id) {
        Some(n) => n,
        None => return vec![],
    };
    
    let mut endpoints = vec![];
    
    // Add all active endpoints
    for endpoint in node.active_endpoints() {
        endpoints.push(format!("https://{}", endpoint.address));
    }
    
    // Add primary address as fallback
    if !endpoints.contains(&node.node_address) {
        endpoints.push(node.node_address.clone());
    }
    
    endpoints
}
```

**Impact:**
- Applications connect by `node_id`, not IP
- Router selects best transport path automatically
- Fallback to alternate paths if primary fails

---

## 📊 Expected Results

### After Full Deployment:

**Federation View:**
```
Node 1: eastgate
  - node_id: 526c1e31-abcd-...
  - Endpoints:
    * Ethernet: 192.168.1.144:8080 (preference: 200)
    * WiFi:     192.168.1.185:8080 (preference: 100)
  - Primary: 192.168.1.144:8080

Node 2: westgate
  - node_id: a8f3c9d2-1234-...
  - Endpoints:
    * Ethernet: 192.168.1.123:8080 (preference: 200)
  - Primary: 192.168.1.123:8080

Node 3: strandgate
  - node_id: e7b5d1a4-5678-...
  - Endpoints:
    * Ethernet: 192.168.1.134:8080 (preference: 200)
  - Primary: 192.168.1.134:8080
```

**Benefits:**
- ✅ 3 logical nodes (not 4+ with duplicates)
- ✅ Human-readable names
- ✅ Stable across restarts
- ✅ Multi-path transport ready
- ✅ Subsystem support ready

---

## 🎭 Future: Subsystem Support

### Architectural Vision

**Scenario:** A single tower runs multiple Songbird subsystems

```
Tower: "westgate" (physical machine)
  ├─ songbird-orchestrator (main)
  │  └─ node_id: a8f3c9d2-1234-...
  │  └─ capabilities: ["orchestration", "coordination"]
  │
  ├─ songbird-ml-worker (GPU subsystem)
  │  └─ node_id: f2a8e3b1-5678-...
  │  └─ capabilities: ["ml-inference", "gpu-compute"]
  │
  └─ songbird-storage (disk subsystem)
     └─ node_id: c4d9f7e2-9abc-...
     └─ capabilities: ["storage", "caching"]
```

**Communication:**
```rust
// Application layer: Connect to node_id
let ml_worker_id = "f2a8e3b1-5678-...";
let endpoint = federation.get_best_endpoint(ml_worker_id).await?;
let response = client.post(endpoint).send().await?;
```

**Routing Layer:**
- Abstracts physical network
- Handles multi-path selection
- Provides automatic failover
- Enables "birdsong" protocol (encrypted node-to-node)

---

## 🔍 Testing & Verification

### Test Commands:

```bash
# 1. Check federation status
curl -sk https://localhost:8080/api/federation/status | jq '.nodes[] | {name, id, endpoints}'

# 2. Verify node names (should be human-readable)
curl -sk https://localhost:8080/api/federation/status | jq -r '.nodes[].node_name'

# Expected: "eastgate", "westgate", "strandgate"
# NOT: "peer-4ec224f8", "peer-9d5eedfc"

# 3. Check endpoint coalescence (eastgate should have 2 endpoints)
curl -sk https://localhost:8080/api/federation/status | \
  jq '.nodes[] | select(.node_name=="eastgate") | .endpoints'

# Expected: Array with 2 entries (Ethernet + WiFi)
```

### Success Criteria:

- ✅ Node names are human-readable (not session ID hashes)
- ✅ Eastgate appears as 1 node (not 2)
- ✅ Westgate appears as 1 node (not duplicate entries)
- ✅ Multiple endpoints coalesced under single `node_id`
- ✅ Preference-based path selection available

---

## 📚 Related Documentation

- [MULTI_PATH_TRANSPORT_ARCHITECTURE_DEC_20_2025.md](./MULTI_PATH_TRANSPORT_ARCHITECTURE_DEC_20_2025.md)
- [NODE_IDENTITY_STABLE_ID_DEC_20_2025.md](./NODE_IDENTITY_STABLE_ID_DEC_20_2025.md)
- [DISCOVERY_PROTOCOL_V3_DEC_20_2025.md](./DISCOVERY_PROTOCOL_V3_DEC_20_2025.md)

---

## 🎯 Metrics

| Metric | Before | After |
|--------|--------|-------|
| **Federation Key** | session_id (rotates) | node_id (stable) |
| **Node Names** | "peer-4ec224f8" | "eastgate" |
| **Eastgate Entries** | 2 (duplicate) | 1 (coalesced) |
| **Routing** | IP-based | Identity-based |
| **Multi-Interface** | ❌ Separate nodes | ✅ Coalesced |
| **Subsystem Support** | ❌ Not possible | ✅ Ready |

---

## ✅ Completion Checklist

- [x] Update `DiscoveredPeer` to include v3.0 fields
- [x] Extract `node_id` and `node_name` in federation bridge
- [x] Implement endpoint coalescence in `register_node()`
- [x] Add `get_best_endpoint()` for routing layer
- [x] Add `get_all_endpoints()` for fallback paths
- [x] Build successful (no compilation errors)
- [ ] Live testing on Eastgate
- [ ] Verification on Westgate
- [ ] Verification on Strandgate

---

**Status:** Implementation complete, ready for live testing across all towers.


