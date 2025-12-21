# songbird-client: Hard Integration vs Capability-Based Pattern

**Date:** December 20, 2025  
**Critical Question:** Does `songbird-client` violate the "Each Primal Knows Only Itself" principle?

---

## 🚨 THE PROBLEM

### Current Design (As Specified)

```rust
// In Toadstool's main.rs
use songbird_client::SongbirdClient;  // ❌ HARDCODED DEPENDENCY!

let client = SongbirdClient::discover_local().await?;  // ❌ KNOWS "Songbird"
let registration = client.register_service(...).await?;
```

**This is a HARD INTEGRATION!** ❌

### Why It Violates Architecture

1. **Hardcoded Primal Name:** `songbird_client` explicitly names Songbird
2. **Compile-Time Dependency:** Toadstool knows Songbird exists before runtime
3. **Not Extensible:** What if a different orchestrator appears?
4. **Violates Self-Knowledge:** Toadstool should only know itself

---

## ✅ THE SOLUTION: Capability-Based Client

### Architectural Principle (Already Implemented in Songbird!)

From `specs/PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md`:

```rust
/// Each primal only knows ITSELF
pub struct PrimalSelfKnowledge {
    pub self_identity: PrimalIdentity,
    pub sovereign_capabilities: SovereignCapabilities,
    
    /// NO HARDCODED KNOWLEDGE OF OTHER PRIMALS
    /// Discovery happens dynamically through universal adapter
    _phantom: std::marker::PhantomData<()>,
}
```

From `specs/CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md`:

```rust
/// Generic capability routing - handles ANY capability without hardcoding
pub async fn capability_request(
    ctx: &AdapterContext,
    capability: &str,
    operation: &str,
    payload: Value,
) -> SongbirdResult<Value>
```

---

## 🎯 THE RIGHT DESIGN

### Option 1: Generic Orchestrator Client (RECOMMENDED)

**Name:** `primal-orchestrator-client` or `orchestrator-client`

```rust
// Toadstool's Cargo.toml
[dependencies]
orchestrator-client = { path = "../orchestrator-client" }  # ✅ GENERIC!
```

```rust
// Toadstool's main.rs
use orchestrator_client::OrchestratorClient;  // ✅ No "Songbird" mentioned!

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Discover ANY orchestrator with "service_registry" capability
    let orchestrator = OrchestratorClient::discover(
        capability: "service_registry"  // ✅ Capability-based!
    ).await?;
    
    // Could find Songbird, or Phoenix, or Enterprise-Orchestrator, etc.
    
    // 2. Register with whoever we found
    let registration = orchestrator.register_service(
        "Toadstool",
        vec![Capability { name: "compute", ... }]
    ).await?;
    
    // 3. Rest of the flow...
}
```

**Benefits:**
- ✅ No hardcoded "Songbird" dependency
- ✅ Works with ANY orchestrator that supports the protocol
- ✅ Toadstool only knows itself
- ✅ Follows existing Songbird architecture

### Option 2: Universal Primal SDK (ALIGN WITH EXISTING)

**Songbird ALREADY has this!** `crates/songbird-primal-sdk/`

```rust
// Toadstool's Cargo.toml
[dependencies]
songbird-primal-sdk = { path = "../songbird/crates/songbird-primal-sdk" }
```

```rust
// Toadstool's main.rs
use songbird_primal_sdk::discovery::{discover_capability, CapabilityType};
use songbird_primal_sdk::registration::{register_service, ServiceInfo};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Discover ANY service with "service_registry" capability
    let orchestrators = discover_capability(
        CapabilityType::ServiceRegistry
    ).await?;
    
    // 2. Register with the best one
    let registration = register_service(
        ServiceInfo {
            name: "Toadstool",
            capabilities: vec!["compute", "gpu"],
            ...
        }
    ).await?;
    
    // 3. Bind to assigned endpoint
    ...
}
```

**Benefits:**
- ✅ Aligns with existing `songbird-primal-sdk`
- ✅ Capability-based discovery
- ✅ No hardcoded orchestrator name
- ✅ Reuses existing architecture

---

## 📊 Comparison

| Aspect | Hard Integration (❌) | Capability-Based (✅) |
|--------|----------------------|----------------------|
| **Dependency Name** | `songbird-client` | `orchestrator-client` or `primal-sdk` |
| **Discovery** | `SongbirdClient::discover()` | `discover_capability("service_registry")` |
| **Knows About** | Songbird (hardcoded) | Capabilities (dynamic) |
| **Works With** | Only Songbird | ANY orchestrator |
| **Extensible** | No | Yes |
| **Follows Principle** | No | Yes |
| **Ecosystem Ready** | No | Yes |

---

## 🏗️ RECOMMENDED ARCHITECTURE

### Phase 1: Enhance Existing `songbird-primal-sdk`

**Location:** `crates/songbird-primal-sdk/`

**Add Registration Protocol:**

```rust
// crates/songbird-primal-sdk/src/registration.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Generic service registration - works with ANY orchestrator
#[async_trait]
pub trait ServiceRegistry {
    async fn register(&self, info: ServiceInfo) -> Result<Registration>;
    async fn heartbeat(&self, id: &str) -> Result<()>;
    async fn deregister(&self, id: &str) -> Result<()>;
}

/// Service information - what a primal knows about itself
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub protocols: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

/// Registration response from orchestrator
#[derive(Debug, Serialize, Deserialize)]
pub struct Registration {
    pub service_id: String,
    pub assigned_endpoint: Endpoint,
    pub heartbeat_interval_sec: u64,
    pub token: String,
}

/// Discovery - find orchestrators by capability
pub async fn discover_orchestrators() -> Result<Vec<OrchestratorInfo>> {
    // 1. Environment variables (ORCHESTRATOR_URL)
    // 2. UDP broadcast (any service with "service_registry")
    // 3. mDNS (_orchestrator._tcp.local)
    // 4. Well-known ports (8080, 8081)
}

/// Register with discovered orchestrator
pub async fn register_with_orchestrator(
    orchestrator: &OrchestratorInfo,
    service_info: ServiceInfo,
) -> Result<Registration> {
    // Use generic REST API or protocol
    // POST {orchestrator.url}/api/v1/services/register
}
```

**Usage in Toadstool:**

```rust
// toadstool/src/main.rs
use songbird_primal_sdk::registration::{
    discover_orchestrators,
    register_with_orchestrator,
    ServiceInfo,
    Capability,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Know thyself
    let my_info = ServiceInfo {
        name: "Toadstool".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            Capability {
                name: "compute".to_string(),
                type_: CapabilityType::Execution,
                ...
            },
            Capability {
                name: "ml_training".to_string(),
                type_: CapabilityType::AI,
                ...
            },
        ],
        ...
    };
    
    // 2. Discover orchestrators (could be Songbird, or anyone else)
    let orchestrators = discover_orchestrators().await?;
    
    if orchestrators.is_empty() {
        info!("No orchestrator found. Running standalone.");
        return run_standalone_mode().await;
    }
    
    let orchestrator = &orchestrators[0];
    info!("Found orchestrator: {} at {}", orchestrator.name, orchestrator.url);
    
    // 3. Register with orchestrator
    let registration = register_with_orchestrator(orchestrator, my_info).await?;
    info!("Registered! Assigned endpoint: {}", registration.assigned_endpoint);
    
    // 4. Bind to assigned endpoint
    let server = ToadstoolServer::new().await?;
    server.bind(registration.assigned_endpoint).await?;
    
    // 5. Start heartbeat
    tokio::spawn(heartbeat_loop(orchestrator, registration.service_id));
    
    // 6. Serve
    server.serve().await?;
}
```

**Key Points:**
- ✅ Toadstool never mentions "Songbird"
- ✅ Works with ANY orchestrator that implements the protocol
- ✅ Follows "Each Primal Knows Only Itself"
- ✅ Aligns with existing `songbird-primal-sdk`

---

## 🔄 Discovery Flow (Capability-Based)

```
┌──────────────┐
│  TOADSTOOL   │  "I am Toadstool. I can compute."
└──────┬───────┘
       │
       ↓ (Discovers via UDP/mDNS/env)
       
"Is there an orchestrator here?"
       ↓
       
┌─────────────────┐
│  SONGBIRD       │  "I am an orchestrator. I have service_registry."
│  (or Phoenix)   │
│  (or any other) │
└────┬────────────┘
     │
     ↓ "Register with me!"
     
┌──────────────┐
│  TOADSTOOL   │  "OK, here are my capabilities: [compute, gpu]"
└──────┬───────┘
       ↓
       
┌─────────────────┐
│  ORCHESTRATOR   │  "Great! Use port 8091"
└─────────────────┘
```

**No hardcoding! Pure capability-based discovery!**

---

## 🎯 Implementation Plan (REVISED)

### Step 1: Enhance `songbird-primal-sdk` ✅ RECOMMENDED

**Location:** `crates/songbird-primal-sdk/`

**Add:**
1. `src/registration.rs` - Registration protocol
2. `src/discovery.rs` - Orchestrator discovery (enhance existing)
3. `src/lifecycle.rs` - Heartbeat, deregistration

**Time:** 3-4 hours

### Step 2: Add Service Registry to Songbird

Same as before, but Songbird implements the **generic protocol**

**Time:** 2-3 hours

### Step 3: Wire Toadstool

```rust
// toadstool/Cargo.toml
[dependencies]
songbird-primal-sdk = { path = "../songbird/crates/songbird-primal-sdk" }
```

**Time:** 2-3 hours

### Step 4: Test End-to-End

Verify Toadstool can:
- Discover Songbird (without knowing it's Songbird)
- Register capabilities
- Receive tasks
- **Bonus:** Register with a DIFFERENT orchestrator if one appears

**Time:** 1-2 hours

---

## 🌟 BONUS: Why This Matters

### Scenario 1: New Orchestrator Appears

```rust
// Phoenix-Orchestrator appears on the network
// Implements same service_registry protocol
// Toadstool discovers it and registers
// NO CODE CHANGES NEEDED!
```

### Scenario 2: Multiple Orchestrators

```rust
// Toadstool discovers both Songbird and Phoenix
// Registers with both
// Load balances between them
// Network effects!
```

### Scenario 3: Future Primals

```rust
// NewPrimal appears
// Uses same songbird-primal-sdk
// Discovers orchestrator
// Registers capabilities
// Works immediately!
```

---

## ✅ ANSWER TO YOUR QUESTION

**Q:** "Does this follow the capability-based patterns or is this a hard integration?"

**A:** As initially specified, it's a **HARD INTEGRATION** ❌

But it **SHOULD BE** capability-based! ✅

**Recommended Fix:**

1. ~~Don't create `songbird-client`~~ (hardcoded name)
2. ✅ Enhance existing `songbird-primal-sdk` (generic, capability-based)
3. ✅ Add generic `registration` module
4. ✅ Toadstool uses SDK, discovers "orchestrators", not "Songbird"
5. ✅ Follows "Each Primal Knows Only Itself" principle

**Result:** Pure capability-based integration that works with ANY orchestrator! 🎉

---

## 📊 Updated TODO

| Original TODO | Revised TODO | Why |
|---------------|--------------|-----|
| Create `songbird-client` crate | Enhance `songbird-primal-sdk` | Aligns with existing architecture |
| `SongbirdClient::discover()` | `discover_orchestrators()` | Generic, not hardcoded |
| Wire Toadstool to Songbird | Wire Toadstool to ANY orchestrator | Capability-based |

---

**Status:** Architecture Violation Identified & Fixed!  
**Next:** Implement capability-based registration in `songbird-primal-sdk`  
**Outcome:** True ecosystem architecture, zero hardcoding! 🌱🎵

