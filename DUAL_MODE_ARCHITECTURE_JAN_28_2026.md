# Dual-Mode Architecture: External TCP + Internal Unix

**Date**: January 28, 2026 (Evening)  
**Status**: 🟢 **ARCHITECTURAL DESIGN** - Implementation in progress  
**Priority**: HIGH - Fixes Port:0 beacon issue for biomeOS

---

## Executive Summary

Songbird operates in **dual-mode** to support both LAN discovery (external) and inter-primal communication (internal):

```
┌─────────────────────────────────────────────────────────────────┐
│                 SONGBIRD DUAL-MODE OPERATION                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EXTERNAL GATEWAY (TCP Port 8080)     INTERNAL IPC (Unix:0)    │
│  ────────────────────────────────     ─────────────────────    │
│  • LAN beacon broadcasts              • Inter-primal JSON-RPC  │
│  • Initial peer handshake             • BearDog ↔ Songbird     │
│  • Federation discovery               • Squirrel ↔ Neural API  │
│  • External API gateway               • Zero network exposure  │
│                                                                 │
│  ESCALATION: TCP discovery → Unix secure RPC                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Problem: Port:0 Beacon Issue

### Root Cause

Discovery beacons contain `port: 0`, causing peer rejection:

```
WARN songbird_discovery::anonymous::listener: Invalid discovery message from 192.168.1.134:40488: Invalid port: 0
```

**Why This Happens**:
1. Discovery message validation explicitly rejects `port: 0` (messages.rs:309)
2. The `port` field is required for v2.1 backward compatibility
3. Even in v3.0 protocol with multi-endpoint, the port field must be non-zero

### Current Validation

```rust
// crates/songbird-discovery/src/anonymous/messages.rs:309
if self.port == 0 {
    return Err("Invalid port: 0".to_string());
}
```

---

## Architecture

### External Mode (TCP for LAN Discovery)

**Purpose**: Enable peer-to-peer discovery across network boundaries

**Binds To**: `0.0.0.0:8080` (configurable)  
**Protocol**: TCP (HTTP/HTTPS)  
**Used For**:
- Broadcasting discovery beacons
- Initial peer handshake
- Federation negotiation
- External API access

**Configuration**:
```toml
[network]
base_port = 8080  # Must be > 0 when discovery enabled

[discovery]
enabled = true    # Requires external TCP binding
```

### Internal Mode (Unix for IPC)

**Purpose**: Secure, high-performance inter-primal communication

**Binds To**: `/run/user/1000/biomeos/songbird-nat0.sock` (XDG-compliant)  
**Protocol**: Unix domain sockets (JSON-RPC 2.0)  
**Used For**:
- BearDog ↔ Songbird (crypto operations)
- Neural API ↔ Songbird (orchestration)
- Squirrel ↔ Songbird (compute requests)
- Zero network exposure (security)

**Configuration**:
```bash
--socket /run/user/1000/biomeos/songbird-nat0.sock
```

---

## Current Implementation Status

### ✅ Already Implemented

1. **HTTP Server Binding** (`http_server.rs`):
   - Binds to TCP address
   - Returns actual port
   - Supports TLS/HTTPS
   - Smart port fallback (if busy, tries next port)

2. **Port Propagation** (`core.rs:312-318`):
   ```rust
   let actual_https_port = crate::app::http_server::start_http_server(
       Arc::clone(&self.federation_state),
       Arc::clone(&federated_service_registry),
       Arc::clone(&self.service_registry),
       bind_address,
   ).await?;
   info!("✅ HTTP server started on port {}", actual_https_port);
   ```

3. **Discovery Integration** (`core.rs:414-455`):
   ```rust
   node_identity.detect_all_endpoints(actual_https_port)?;
   
   let listener_arc = super::discovery_startup::start_discovery_system(
       self._config.discovery.port,
       actual_https_port,  // ✅ Actual port passed to discovery
       &node_identity,
       endpoint_messages,
       capabilities,
       broadcast_addrs,
   ).await?;
   ```

### 🔴 Issue: Configuration Gap

**Problem**: If `network.base_port = 0` is set, the HTTP server never binds TCP properly.

**Impact**:
- Discovery beacons contain `port: 0`
- Peers reject the beacons
- LAN discovery fails

---

## Solution: Validation & Configuration

### 1. Add Configuration Validation

```rust
// crates/songbird-types/src/config/consolidated_canonical/mod.rs
impl CanonicalSongbirdConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Existing validation...
        
        // NEW: Validate discovery + port combination
        if self.discovery.enabled && self.network.base_port == 0 {
            return Err(
                "Discovery requires external TCP port (network.base_port > 0). \n\
                 Set network.base_port = 8080 or disable discovery.".to_string()
            );
        }
        
        Ok(())
    }
}
```

### 2. Add CLI Option for External Port

```rust
// crates/songbird-orchestrator/src/bin_interface.rs
#[derive(Args, Debug, Clone)]
pub struct ServerArgs {
    /// HTTP server port (external discovery gateway)
    #[arg(long, short, default_value = "8080")]
    pub port: u16,
    
    /// Unix socket path for inter-primal IPC
    #[arg(long)]
    pub socket: Option<String>,
    
    /// Alias for --port (clearer intent for federation)
    #[arg(long, conflicts_with = "port")]
    pub federation_port: Option<u16>,
    
    // ... existing fields
}
```

### 3. Add Default Configuration Guard

```rust
// crates/songbird-types/src/config/consolidated_canonical/network.rs
impl Default for CanonicalNetworkConfig {
    fn default() -> Self {
        Self {
            bind_host: "0.0.0.0".to_string(),  // Listen on all interfaces
            base_port: 8080,                    // Must be > 0 for discovery
            // ...
        }
    }
}
```

---

## Usage Examples

### Correct Configuration (Discovery Enabled)

```bash
# Minimal (uses defaults)
./songbird server

# Explicit (recommended)
./songbird server \
    --port 8080 \
    --socket /run/user/1000/biomeos/songbird-nat0.sock

# biomeOS deployment (full XDG compliance)
XDG_RUNTIME_DIR=/run/user/1000 \
FAMILY_ID=nat0 \
./songbird server \
    --port 8080 \
    --socket /run/user/1000/biomeos/songbird-nat0.sock
```

### Discovery Disabled (Unix Only)

```bash
# IPC only - no external port needed
./songbird server \
    --socket /run/user/1000/biomeos/songbird-nat0.sock \
    --config disable_discovery.toml

# disable_discovery.toml:
# [discovery]
# enabled = false
```

---

## Discovery Protocol Versions

### v2.1 (Backward Compatible)

**Uses `port` field for single endpoint**:
```json
{
  "version": "2.1",
  "capabilities": ["orchestration", "federation"],
  "protocols": ["https"],
  "port": 8080,  // ✅ Must be > 0
  "session_id": "abc123..."
}
```

### v3.0 (Multi-Endpoint)

**Uses `endpoints` array + fallback `port`**:
```json
{
  "version": "3.0",
  "node_id": "tower-stable-id",
  "node_name": "tower0",
  "endpoints": [
    {
      "interface_type": "ethernet",
      "address": "192.168.1.100:8080",
      "protocols": ["https"],
      "preference": 255
    }
  ],
  "capabilities": ["orchestration", "federation"],
  "port": 8080,  // ✅ Still required for v2.1 clients
  "session_id": "abc123..."
}
```

**Key Point**: Even v3.0 requires a valid `port` field for backward compatibility with v2.1 clients.

---

## Escalation Flow

### 1. TCP Discovery (External)

```
Tower A → UDP multicast beacon → Tower B
  ↓
Beacon contains:
  - node_id: "tower-a-stable-id"
  - endpoints: [
      {address: "192.168.1.100:8080", ...}
    ]
  - port: 8080 (v2.1 fallback)
  ↓
Tower B receives beacon
Tower B validates port > 0 ✅
Tower B adds Tower A to peer list
```

### 2. TCP Handshake (External)

```
Tower B → HTTPS GET https://192.168.1.100:8080/health → Tower A
  ↓
Tower A responds: 200 OK {status: "healthy", capabilities: [...]}
  ↓
Trust verification via BearDog lineage
```

### 3. Unix Socket Escalation (Internal)

```
Tower B → Establishes Unix socket connection
  ↓
Tower A provides Unix socket path: /run/user/1000/biomeos/songbird-nat0.sock
  ↓
All future communication via Unix sockets (faster, more secure)
```

---

## Verification

### Test 1: Port Validation

```bash
# Should REJECT: port = 0 with discovery enabled
cat > invalid_config.toml << EOF
[network]
base_port = 0

[discovery]
enabled = true
EOF

./songbird server --config invalid_config.toml
# Expected: Error: "Discovery requires external TCP port (network.base_port > 0)"
```

### Test 2: Valid Configuration

```bash
# Should ACCEPT: port = 8080 with discovery enabled
./songbird server --port 8080

# Check beacon contains correct port
# Expected in logs: "🌐 Starting anonymous discovery with actual HTTPS port 8080"
```

### Test 3: LAN Discovery

```bash
# Tower A (ethernet)
./songbird server --port 8080

# Tower B (wifi, different subnet)
./songbird server --port 8080

# Expected: Both towers discover each other within 30 seconds
# Expected in logs: "✅ Discovered peer: tower-{id} at 192.168.x.x:8080"
```

---

## Implementation Tasks

- [ ] Add configuration validation (network.base_port > 0 when discovery enabled)
- [ ] Add `--federation-port` CLI alias for clarity
- [ ] Update default configuration comments
- [ ] Add error message with fix suggestions
- [ ] Document dual-mode architecture (this file)
- [ ] Test LAN discovery across wifi/ethernet boundaries
- [ ] Verify beacons contain correct port

---

## References

- **Discovery Broadcaster**: `crates/songbird-discovery/src/anonymous/broadcaster.rs`
- **Discovery Messages**: `crates/songbird-discovery/src/anonymous/messages.rs`
- **HTTP Server**: `crates/songbird-orchestrator/src/app/http_server.rs`
- **Core Orchestrator**: `crates/songbird-orchestrator/src/app/core.rs`
- **Configuration**: `crates/songbird-types/src/config/`

---

**Generated**: 2026-01-28 (Evening)  
**Status**: 🟢 Design complete, implementation in progress  
**Impact**: Fixes Port:0 beacon issue, enables full LAN discovery

🎊 **DUAL-MODE ARCHITECTURE: TCP EXTERNAL + UNIX INTERNAL** 🎊

