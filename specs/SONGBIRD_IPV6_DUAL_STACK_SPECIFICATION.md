# 🌐 Songbird IPv6 Dual-Stack Support Specification
## Critical Network Binding Enhancement

**Version**: 1.0  
**Date**: November 10, 2025  
**Status**: 🔴 CRITICAL SHORTFALL IDENTIFIED  
**Priority**: P0 - Immediate Fix Required  
**Discovery**: NestGate Integration Session

---

## 📊 EXECUTIVE SUMMARY

**Problem**: Songbird currently binds only to IPv4 (`0.0.0.0`), causing connection failures when clients use `localhost` (which resolves to IPv6 `[::1]` on modern systems).

**Impact**: 
- ❌ NestGate cannot connect via `localhost`
- ❌ Modern systems fail discovery
- ❌ Non-compliant with RFC standards
- ❌ Blocks service mesh federation

**Solution**: Enable IPv6 dual-stack binding (`[::]`) to support both IPv4 and IPv6 simultaneously.

**Effort**: 15 minutes (single-file change)

---

## 🔍 ROOT CAUSE ANALYSIS

### **Current Implementation**

```rust
// File: crates/songbird-orchestrator/src/app/mod.rs
// Line: 363

let bind_address = SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "0.0.0.0");
let port = SafeEnv::get_port("SONGBIRD_PORT", orchestrator_port());
let addr: SocketAddr = format!("{bind_address}:{port}").parse()?;
```

**Binding**: `0.0.0.0:8080` (IPv4 only)

### **The Discovery Flow That Failed**

```rust
// NestGate Discovery Sequence
let discovery_order = [
    "http://localhost:8080",       // Resolves to [::1] → FAILS
    "http://127.0.0.1:8080",       // Works (IPv4)
    "http://192.0.2.10:8080",   // Works (IPv4)
];
```

### **Why This Matters**

Modern Linux systems (kernel 3.0+) resolve `localhost` to IPv6 first:

```bash
$ getent hosts localhost
::1             localhost
127.0.0.1       localhost
```

**Result**: 
- `curl http://localhost:8080` → tries `[::1]:8080` → **Connection refused**
- `curl http://127.0.0.1:8080` → works

---

## 🎯 TECHNICAL SPECIFICATION

### **1. Dual-Stack Binding**

**Requirement**: Songbird MUST support both IPv4 and IPv6 connections simultaneously.

**Implementation**:

```rust
// BEFORE (IPv4 only)
let bind_address = SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "0.0.0.0");

// AFTER (Dual-stack)
let bind_address = SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "[::]");
```

**Behavior**:
- `[::]` → Binds to `[::]:8080` (IPv6 wildcard)
- Kernel automatically handles IPv4 via IPv4-mapped IPv6 addresses
- Both `::ffff:127.0.0.1` (IPv4-mapped) and `::1` (IPv6) work

### **2. Backward Compatibility**

**Support All Binding Modes**:

```rust
enum BindMode {
    DualStack,    // [::] - RECOMMENDED
    IPv6Only,     // [::1] - Specific IPv6
    IPv4Only,     // 0.0.0.0 - Legacy
    Specific(IP), // 192.0.2.10 - Custom
}

fn parse_bind_address(addr: &str, port: u16) -> Result<SocketAddr> {
    match addr {
        "[::]" => {
            // Dual-stack: IPv6 wildcard (handles IPv4 too)
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port))
        }
        "[::1]" => {
            // IPv6 localhost
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port))
        }
        "0.0.0.0" => {
            // IPv4 wildcard
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))
        }
        "127.0.0.1" => {
            // IPv4 localhost
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        }
        _ => {
            // Parse as-is (supports IPv4 or IPv6 addresses)
            format!("{addr}:{port}").parse()
                .map_err(|e| SongbirdError::configuration(format!("Invalid bind address: {e}")))
        }
    }
}
```

### **3. Environment Variable Support**

```bash
# Dual-stack (recommended - default)
export SONGBIRD_BIND_ADDRESS="[::]"

# IPv4 only (legacy)
export SONGBIRD_BIND_ADDRESS="0.0.0.0"

# IPv6 only (specific)
export SONGBIRD_BIND_ADDRESS="[::1]"

# Specific interface
export SONGBIRD_BIND_ADDRESS="192.0.2.10"
export SONGBIRD_BIND_ADDRESS="[fe80::1]"
```

---

## 📋 IMPLEMENTATION PLAN

### **Phase 1: Immediate Fix (15 minutes)** 🔴 NOW

**File**: `crates/songbird-orchestrator/src/app/mod.rs`

```rust
// Line 363: Change default binding
let bind_address = SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "[::]");

// Line 367: Add intelligent parsing
let addr: SocketAddr = parse_bind_address(&bind_address, port)?;

// Add helper function
fn parse_bind_address(addr: &str, port: u16) -> SongbirdResult<SocketAddr> {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    
    match addr {
        "[::]" => Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port)),
        "[::1]" => Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port)),
        "0.0.0.0" => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)),
        "127.0.0.1" => Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)),
        _ => {
            // Try to parse as full address or just IP
            if addr.contains(':') && addr.starts_with('[') {
                // IPv6 format: [addr]:port or [addr]
                let ip_part = addr.trim_start_matches('[').trim_end_matches(']');
                let ip: IpAddr = ip_part.parse()
                    .map_err(|e| SongbirdError::configuration(format!("Invalid IPv6: {e}")))?;
                Ok(SocketAddr::new(ip, port))
            } else {
                // IPv4 format or hostname
                format!("{addr}:{port}").parse()
                    .map_err(|e| SongbirdError::configuration(format!("Invalid address: {e}")))
            }
        }
    }
}
```

**Testing**:
```bash
# Test IPv6
curl http://[::1]:8080/health

# Test IPv4
curl http://127.0.0.1:8080/health

# Test localhost (should use IPv6)
curl http://localhost:8080/health

# Verify binding
ss -tlnp | grep :8080
# Should show: LISTEN [::]:8080
```

---

### **Phase 2: Configuration Enhancement (1 week)** 🟡 NEXT

**Add to Configuration**:

```rust
// File: crates/songbird-config/src/canonical/network/core.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBindingConfig {
    /// Bind address (supports IPv4, IPv6, dual-stack)
    /// Examples: "[::]", "0.0.0.0", "[::1]", "192.0.2.10"
    pub bind_address: String,
    
    /// Primary port
    pub port: u16,
    
    /// Enable IPv6 dual-stack (auto-detect from bind_address)
    pub ipv6_enabled: bool,
    
    /// Enable IPv4 (auto-detect from bind_address)
    pub ipv4_enabled: bool,
}

impl NetworkBindingConfig {
    pub fn dual_stack(port: u16) -> Self {
        Self {
            bind_address: "[::]".to_string(),
            port,
            ipv6_enabled: true,
            ipv4_enabled: true,
        }
    }
    
    pub fn ipv4_only(port: u16) -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port,
            ipv6_enabled: false,
            ipv4_enabled: true,
        }
    }
    
    pub fn ipv6_only(port: u16) -> Self {
        Self {
            bind_address: "[::]".to_string(),
            port,
            ipv6_enabled: true,
            ipv4_enabled: false,
        }
    }
}
```

---

### **Phase 3: Discovery Enhancement (1 week)** 🟡 NEXT

**Update Service Discovery**:

```rust
// File: crates/songbird-discovery/src/discovery/network/mod.rs

pub struct NetworkDiscovery {
    /// Try IPv6 first (modern systems)
    ipv6_priority: bool,
    
    /// Fallback to IPv4
    ipv4_fallback: bool,
}

impl NetworkDiscovery {
    pub async fn discover_service(&self, hostname: &str, port: u16) -> Vec<SocketAddr> {
        let mut addresses = Vec::new();
        
        // Try DNS resolution
        if let Ok(resolved) = tokio::net::lookup_host(format!("{hostname}:{port}")).await {
            for addr in resolved {
                match addr {
                    SocketAddr::V6(_) if self.ipv6_priority => {
                        addresses.insert(0, addr); // IPv6 first
                    }
                    SocketAddr::V4(_) if self.ipv4_fallback => {
                        addresses.push(addr); // IPv4 fallback
                    }
                    _ => {}
                }
            }
        }
        
        addresses
    }
}
```

---

## ✅ VERIFICATION CRITERIA

### **Must Pass**:

```bash
# 1. Dual-stack binding works
ss -tlnp | grep :8080
# Expected: LISTEN [::]:8080

# 2. IPv6 connections work
curl -v http://[::1]:8080/health
# Expected: 200 OK

# 3. IPv4 connections still work
curl -v http://127.0.0.1:8080/health
# Expected: 200 OK

# 4. localhost resolves correctly
curl -v http://localhost:8080/health
# Expected: 200 OK (uses IPv6 [::1])

# 5. NestGate can connect
# From NestGate:
curl http://localhost:8080/api/federation/services
# Expected: 200 OK with service list
```

---

## 📊 IMPACT ASSESSMENT

### **Before Fix**:
```
IPv6:     ❌ Not supported
IPv4:     ✅ Supported
localhost: ❌ Fails (resolves to IPv6)
Modern:   ❌ Incompatible
Legacy:   ✅ Works
```

### **After Fix**:
```
IPv6:     ✅ Fully supported
IPv4:     ✅ Fully supported (via IPv4-mapped)
localhost: ✅ Works (IPv6 or IPv4)
Modern:   ✅ Compatible
Legacy:   ✅ Still works
```

---

## 🔐 SECURITY CONSIDERATIONS

1. **Firewall Rules**: Update to allow both IPv4 and IPv6
   ```bash
   # Allow IPv6
   ip6tables -A INPUT -p tcp --dport 8080 -j ACCEPT
   
   # Allow IPv4 (existing)
   iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
   ```

2. **Binding Scope**: `[::]` binds to all interfaces
   - More permissive than `0.0.0.0`
   - Consider firewall rules for production

3. **IPv4-mapped Addresses**: Automatic translation is secure
   - Kernel handles mapping: `::ffff:192.0.2.1`
   - No application-level translation needed

---

## 📚 STANDARDS COMPLIANCE

### **RFC Requirements**:

✅ **RFC 4291**: IPv6 Addressing Architecture  
✅ **RFC 3493**: Basic Socket Interface Extensions for IPv6  
✅ **RFC 6724**: Default Address Selection for IPv6  
✅ **RFC 4038**: Application Aspects of IPv6 Transition

### **Best Practices**:

1. ✅ Support dual-stack by default
2. ✅ Prefer IPv6 when available
3. ✅ Maintain IPv4 compatibility
4. ✅ Use `[::]` for wildcard binding
5. ✅ Handle both address families transparently

---

## 🎯 SUCCESS METRICS

**Immediate** (After Phase 1):
- ✅ NestGate connects via `localhost`
- ✅ Both IPv4 and IPv6 work
- ✅ Zero breaking changes
- ✅ Discovery works universally

**Long-term** (After Phase 3):
- ✅ 100% modern system compatibility
- ✅ Standards-compliant networking
- ✅ Future-proof architecture
- ✅ Seamless multi-protocol support

---

## 🚀 NEXT STEPS

1. **Immediate** (15 min):
   - [ ] Change default binding to `[::]`
   - [ ] Add `parse_bind_address()` helper
   - [ ] Test with curl (IPv4 + IPv6)
   - [ ] Verify NestGate connection

2. **This Week**:
   - [ ] Add configuration structures
   - [ ] Update discovery to prefer IPv6
   - [ ] Document in user guides
   - [ ] Add integration tests

3. **Next Sprint**:
   - [ ] gRPC dual-stack support
   - [ ] WebSocket dual-stack support
   - [ ] Universal protocol framework

---

## 📝 REFERENCES

- **Discovery Source**: NestGate Integration Session (Nov 10, 2025)
- **Related Specs**: 
  - `UNIVERSAL_PROTOCOL_FRAMEWORK_SPECIFICATION.md` (to be created)
  - `HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md` (existing)
  - `TRANSPORT_SYSTEM_EVOLUTION_SPEC.md` (existing)

---

**Status**: 🔴 **CRITICAL - REQUIRES IMMEDIATE FIX**  
**Owner**: Songbird Core Team  
**Reporter**: NestGate Integration Team  
**Date Identified**: November 10, 2025

**This specification documents a critical shortfall that blocks service mesh federation. Implementation should begin immediately.**

