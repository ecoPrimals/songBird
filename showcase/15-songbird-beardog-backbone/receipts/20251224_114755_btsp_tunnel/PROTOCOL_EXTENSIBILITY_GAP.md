# Integration Gap #5: Protocol Extensibility

**Date:** December 24, 2025  
**Found By:** ionChannel integration attempt  
**Status:** 🟡 WORKAROUND EXISTS, ENHANCEMENT NEEDED

---

## 🎯 The Problem

**Hardcoded Protocol Enum:**
```rust
pub enum Protocol {
    Http,
    Https,
    JsonRpc,
    Tarpc,
    Btsp,
    WebSocket,
    WebSocketSecure,
    // Can't add RemoteDesktop without changing Songbird code!
}
```

**Impact:**
- New services (like ionChannel) can't register custom protocols
- Requires Songbird code changes for each new protocol
- Breaks capability-based architecture principle
- Makes integration harder than it should be

---

## ✅ Current Workaround (WORKS NOW!)

**ionChannel CAN register without code changes** using the features system:

```rust
// ionChannel side (no Songbird changes needed!)
capability_manager.register_feature("remote-desktop".to_string()).await;

capability_manager.register_protocol(ProtocolCapability {
    protocol: Protocol::Https,  // Use HTTPS as transport
    port: 1985,
    path: Some("/org/freedesktop/portal/desktop".to_string()),
    status: ProtocolStatus::Active,
    metadata: {
        let mut m = HashMap::new();
        m.insert("service_type".to_string(), "remote-desktop".to_string());
        m.insert("mode".to_string(), "full".to_string());
        m.insert("capture_tier".to_string(), "dmabuf".to_string());
        m
    },
}).await;
```

**Client discovery (no code changes!):**
```rust
// Find towers with remote-desktop feature
let services = discovery.discover_services(&ServiceQuery::builder()
    .with_feature("remote-desktop")
    .build()).await?;

// Extract metadata
for service in services {
    if let Some(cap) = service.protocols.iter()
        .find(|p| p.metadata.get("service_type") == Some(&"remote-desktop".to_string())) 
    {
        let endpoint = format!("{}:{}{}", 
            service.endpoint, 
            cap.port, 
            cap.path.as_ref().unwrap_or(&"".to_string())
        );
        // Connect to ionChannel at endpoint
    }
}
```

**This works TODAY!** No Songbird code changes needed.

---

## 🚧 Better Solution (Enhancement)

### Make Protocol String-Based

```rust
// Instead of enum
pub enum Protocol {
    Http,
    // ...
}

// Use flexible struct
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Protocol {
    /// Protocol name (extensible!)
    name: String,
    
    /// Performance tier (1-5, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    performance_tier: Option<u8>,
    
    /// Whether protocol is encrypted
    #[serde(default)]
    encrypted: bool,
    
    /// Whether production-ready
    #[serde(default = "default_true")]
    production_ready: bool,
}

impl Protocol {
    // Well-known protocols as constants
    pub const HTTP: Protocol = Protocol {
        name: "http".to_string(),
        performance_tier: Some(1),
        encrypted: false,
        production_ready: false,
    };
    
    pub const HTTPS: Protocol = Protocol {
        name: "https".to_string(),
        performance_tier: Some(2),
        encrypted: true,
        production_ready: true,
    };
    
    // Custom protocol constructor
    pub fn custom(name: impl Into<String>) -> Self {
        Protocol {
            name: name.into(),
            performance_tier: None,
            encrypted: false,
            production_ready: true,
        }
    }
    
    pub fn with_encryption(mut self, encrypted: bool) -> Self {
        self.encrypted = encrypted;
        self
    }
    
    pub fn with_tier(mut self, tier: u8) -> Self {
        self.performance_tier = Some(tier);
        self
    }
}
```

**Usage:**
```rust
// Well-known protocols (backward compatible)
Protocol::HTTPS

// Custom protocols (no code change needed!)
Protocol::custom("remote-desktop")
    .with_encryption(true)
    .with_tier(4)
```

---

## 📊 Priority

**Current Status:** 🟢 **P2 (Enhancement)**

**Why P2?**
- Workaround exists (features + metadata)
- Not blocking integration
- Nice-to-have for cleaner API

**Why Not P0/P1?**
- ionChannel CAN integrate without code changes
- Features system provides flexibility
- Metadata HashMap is extensible

**When to Upgrade to P1:**
- If many services need custom protocols
- If workaround becomes cumbersome
- If we want cleaner discovery API

---

## 🎯 Action Items

### For ionChannel (NOW):

✅ **Use the workaround** (no Songbird changes needed):
1. Register `"remote-desktop"` feature
2. Use `Protocol::Https` with metadata
3. Put all ionChannel-specific info in metadata HashMap

Example code above shows how!

### For Songbird (FUTURE - P2):

🚧 **Make Protocol extensible:**
1. Change `Protocol` from enum to struct
2. Keep well-known protocols as constants
3. Allow arbitrary protocol names
4. Maintain backward compatibility

### For Both (DOCUMENTATION):

📚 **Document the pattern:**
1. Create guide: "Adding Custom Protocols"
2. Show features + metadata pattern
3. Document when to use each approach

---

## 📚 Related

- **Features System:** `crates/songbird-network-federation/src/protocol_capability.rs`
- **Metadata HashMap:** Already exists in `ProtocolCapability`
- **Discovery:** `crates/songbird-discovery/`

---

## 🎓 What This Teaches

### **The Gap:**
Hardcoded enum prevents extensibility

### **The Fix:**
Features + metadata provides flexibility TODAY

### **The Lesson:**
- Capability-based architecture should be fully extensible
- Workarounds exist but proper solution is better
- Document patterns for others

---

**Status:** 🟢 ionChannel can integrate NOW using workaround  
**Priority:** P2 (enhancement, not blocking)  
**Action:** ionChannel uses features + metadata pattern (code example above)

