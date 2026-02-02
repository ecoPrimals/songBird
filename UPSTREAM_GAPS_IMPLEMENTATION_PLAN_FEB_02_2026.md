# 🔧 Upstream Gaps - Implementation Plan

**Date**: February 2, 2026  
**Status**: ⚡ **READY FOR EXECUTION**  
**Estimated**: 7-13 hours total (broken into phases)

---

## 🎯 **EXECUTIVE SUMMARY**

**Situation**: Fresh binaries deployed, infrastructure exists, but 3 critical gaps prevent end-to-end functionality:

1. **BearDog CLI** ✅ **SOLVED** - Uses `beardog server --socket <path>`
2. **Introspection Missing** 🔴 **BLOCKER** - Primals can't self-describe
3. **Discovery Not Wired** 🟡 **BLOCKER** - Infrastructure exists but unused

**Impact**: Dark Forest and capability discovery blocked until gaps closed.

**Solution**: 6 focused implementation tasks (2-4 hours each)

---

## ✅ **GAP 1: BearDog CLI - SOLVED!**

### **Problem**: Deploy script used wrong flag

**Evidence**:
```bash
# WRONG (deploy script):
beardog --socket /path/to/socket

# RIGHT (actual CLI):
beardog server --socket /path/to/socket
```

### **Solution Found**:
```bash
$ beardog server --help
Start BearDog server (long-running service mode)

Usage: beardog server [OPTIONS]

Options:
      --socket <SOCKET>                    Unix socket path [default: /tmp/beardog.sock]
      --family-id <FAMILY_ID>              Family ID for BirdSong
      --orchestrator-id <ORCHESTRATOR_ID>  Orchestrator ID
```

### **Fix Required**:
Update deployment scripts to use:
```bash
beardog server --socket /run/user/1000/biomeos/beardog.sock --family-id <id>
```

**Estimated Time**: 30 minutes (find and update scripts)

---

## 🔴 **GAP 2: Primal Introspection**

### **Problem**: Primals don't expose self-description methods

**What's Missing**:
- `primal.info` - Return primal metadata
- `primal.capabilities` - List capabilities provided
- `rpc.methods` - List all available JSON-RPC methods

**Impact**: `CapabilityDiscoveryService` can't auto-discover primals

### **Solution**: Add 3 methods to songbird and beardog

---

### **2A: Songbird Introspection** (2 hours)

**File**: `phase1/songbird/crates/songbird-universal-ipc/src/service.rs`

**Location**: Line 457 (in `JsonRpcHandler` impl)

**Add Methods**:

```rust
/// Handle `primal.info` method
async fn handle_primal_info(&self, _params: Value) -> Result<Value, String> {
    let info = json!({
        "name": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Network Orchestration & Discovery Primal",
        "capabilities": ["discovery", "stun", "mdns", "http", "ipc"],
        "endpoints": {
            "unix": "/run/user/1000/biomeos/songbird.sock",
            "tcp": "127.0.0.1:8080"
        }
    });
    Ok(info)
}

/// Handle `primal.capabilities` method
async fn handle_primal_capabilities(&self, _params: Value) -> Result<Value, String> {
    let capabilities = json!({
        "capabilities": [
            {
                "name": "discovery",
                "operations": ["peers", "mdns", "broadcast"],
                "description": "Service discovery and peer finding"
            },
            {
                "name": "stun",
                "operations": ["get_public_address", "bind"],
                "description": "NAT traversal via STUN"
            },
            {
                "name": "http",
                "operations": ["request", "get", "post"],
                "description": "HTTP/HTTPS client"
            },
            {
                "name": "ipc",
                "operations": ["register", "resolve", "discover", "list"],
                "description": "Inter-primal communication"
            }
        ]
    });
    Ok(capabilities)
}

/// Handle `rpc.methods` method
async fn handle_rpc_methods(&self, _params: Value) -> Result<Value, String> {
    let methods = json!({
        "methods": [
            // Introspection
            "primal.info",
            "primal.capabilities",
            "rpc.methods",
            
            // IPC registry
            "ipc.register",
            "ipc.resolve",
            "ipc.discover",
            "ipc.list",
            
            // HTTP/HTTPS
            "http.request",
            "http.get",
            "http.post",
            
            // STUN/NAT
            "stun.get_public_address",
            "stun.bind",
            
            // Discovery
            "discovery.peers",
            
            // Rendezvous
            "rendezvous.register",
            "rendezvous.lookup",
            
            // Peer connection
            "peer.connect"
        ]
    });
    Ok(methods)
}
```

**Wire to Handler** (line 484, before `_ => Err(...)`):
```rust
// Introspection methods
"primal.info" => self.handle_primal_info(params).await,
"primal.capabilities" => self.handle_primal_capabilities(params).await,
"rpc.methods" => self.handle_rpc_methods(params).await,
```

**Test**:
```bash
echo '{"jsonrpc":"2.0","method":"primal.info","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird.sock
```

**Estimated Time**: 2 hours (implement + test)

---

### **2B: BearDog Introspection** (2 hours)

**File**: `phase1/beardog/crates/beardog-tunnel/src/unix_socket_ipc/handlers/mod.rs`

**Add Similar Methods** (adapt for beardog):

```rust
/// Handle `primal.info` method
pub async fn handle_primal_info(_params: Value) -> Result<Value, BearDogError> {
    let info = json!({
        "name": "beardog",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Sovereign Genetic Cryptography Primal",
        "capabilities": ["crypto", "genetic", "birdsong", "hsm"],
        "endpoints": {
            "unix": "/run/user/1000/biomeos/beardog.sock"
        }
    });
    Ok(info)
}

/// Handle `primal.capabilities` method
pub async fn handle_primal_capabilities(_params: Value) -> Result<Value, BearDogError> {
    let capabilities = json!({
        "capabilities": [
            {
                "name": "crypto",
                "operations": ["encrypt", "decrypt", "sign", "verify", "hash"],
                "description": "Cryptographic operations"
            },
            {
                "name": "genetic",
                "operations": [
                    "derive_lineage_key",
                    "mix_entropy",
                    "verify_lineage",
                    "generate_lineage_proof",
                    "generate_challenge",
                    "respond_to_challenge",
                    "verify_challenge_response"
                ],
                "description": "Genetic lineage cryptography"
            },
            {
                "name": "birdsong",
                "operations": ["encrypt", "decrypt", "request_key"],
                "description": "BirdSong lineage-based encryption"
            }
        ]
    });
    Ok(capabilities)
}

/// Handle `rpc.methods` method
pub async fn handle_rpc_methods(_params: Value) -> Result<Value, BearDogError> {
    let methods = json!({
        "methods": [
            // Introspection
            "primal.info",
            "primal.capabilities",
            "rpc.methods",
            
            // Crypto
            "crypto.chacha20_poly1305_encrypt",
            "crypto.chacha20_poly1305_decrypt",
            "crypto.blake3_hash",
            
            // Genetic
            "genetic.derive_lineage_key",
            "genetic.mix_entropy",
            "genetic.verify_lineage",
            "genetic.generate_lineage_proof",
            "genetic.generate_challenge",
            "genetic.respond_to_challenge",
            "genetic.verify_challenge_response",
            
            // BirdSong
            "birdsong.encrypt",
            "birdsong.decrypt",
            "birdsong.request_key"
        ]
    });
    Ok(methods)
}
```

**Wire to Router**: Update the main handler router to recognize these methods

**Estimated Time**: 2 hours (implement + test)

---

## 🟡 **GAP 3: Wire Capability Discovery**

### **Problem**: `CapabilityDiscoveryService` exists but not integrated

**What Exists**:
- ✅ `biomeos-core/src/capability_discovery.rs` - Discovery service
- ✅ `biomeos-atomic-deploy/src/handlers/capability.rs` - Capability handler
- ❌ Handler doesn't call discovery service
- ❌ Discovery service not wired to runtime

### **Solution**: Wire discovery into capability handler

---

### **3A: Update Capability Handler** (1-2 hours)

**File**: `phase2/biomeOS/crates/biomeos-atomic-deploy/src/handlers/capability.rs`

**Current** (simplified):
```rust
impl CapabilityHandler {
    pub async fn discover(&self, params: &Option<Value>) -> Result<Value> {
        // OLD: Uses static router.discover_capability
        let providers = self.router.discover_capability(&capability).await?;
        // ...
    }
}
```

**Update to**:
```rust
use biomeos_core::capability_discovery::CapabilityDiscoveryService;

impl CapabilityHandler {
    pub async fn discover(&self, params: &Option<Value>) -> Result<Value> {
        // NEW: Use runtime discovery
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/run/user/1000"))
            .join("biomeos");
            
        let mut discovery = CapabilityDiscoveryService::new(runtime_dir);
        
        // Discover all available primals
        let providers = discovery.discover_all().await?;
        
        // Filter by requested capability if specified
        let filtered = if let Some(cap_name) = extract_capability_from_params(params) {
            providers.into_iter()
                .filter(|p| p.capabilities.contains(&cap_name))
                .collect()
        } else {
            providers
        };
        
        Ok(json!({ "providers": filtered }))
    }
    
    pub async fn call(&self, params: &Option<Value>) -> Result<Value> {
        // Extract capability + operation from params
        let (capability, operation, data) = extract_call_params(params)?;
        
        // Runtime discovery of provider
        let runtime_dir = get_runtime_dir();
        let mut discovery = CapabilityDiscoveryService::new(runtime_dir);
        
        // Find provider for this capability
        let provider = discovery.find_capability(&capability).await
            .ok_or_else(|| format!("No provider found for capability: {}", capability))?;
            
        // Translate semantic call to primal method
        let method = self.translator.translate(&capability, &operation)
            .ok_or_else(|| format!("No translation for {}:{}", capability, operation))?;
            
        // Call the primal directly
        self.router.call_primal(&provider.socket_path, method, data).await
    }
}
```

**Estimated Time**: 1-2 hours (refactor + test)

---

### **3B: Register Capability Translations** (1 hour)

**File**: `phase2/biomeOS/crates/biomeos-atomic-deploy/src/capability_translation.rs`

**Add Default Registry**:
```rust
impl CapabilityTranslationRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        
        // BearDog translations
        registry.register("security", "encrypt", "beardog", "crypto.chacha20_poly1305_encrypt");
        registry.register("security", "decrypt", "beardog", "crypto.chacha20_poly1305_decrypt");
        registry.register("security", "hash", "beardog", "crypto.blake3_hash");
        
        // Genetic lineage translations
        registry.register("lineage", "derive_key", "beardog", "genetic.derive_lineage_key");
        registry.register("lineage", "mix_entropy", "beardog", "genetic.mix_entropy");
        registry.register("lineage", "verify", "beardog", "genetic.verify_lineage");
        registry.register("lineage", "prove", "beardog", "genetic.generate_lineage_proof");
        registry.register("lineage", "challenge", "beardog", "genetic.generate_challenge");
        registry.register("lineage", "respond", "beardog", "genetic.respond_to_challenge");
        registry.register("lineage", "verify_response", "beardog", "genetic.verify_challenge_response");
        
        // BirdSong translations
        registry.register("birdsong", "encrypt", "beardog", "birdsong.encrypt");
        registry.register("birdsong", "decrypt", "beardog", "birdsong.decrypt");
        registry.register("birdsong", "request_key", "beardog", "birdsong.request_key");
        
        // Songbird translations
        registry.register("discovery", "public_ip", "songbird", "stun.get_public_address");
        registry.register("discovery", "bind", "songbird", "stun.bind");
        registry.register("discovery", "peers", "songbird", "discovery.peers");
        
        // HTTP translations
        registry.register("http", "request", "songbird", "http.request");
        registry.register("http", "get", "songbird", "http.get");
        registry.register("http", "post", "songbird", "http.post");
        
        registry
    }
}
```

**Use in Handler**:
```rust
impl CapabilityHandler {
    pub fn new(router: NeuralRouter) -> Self {
        Self {
            router,
            translator: Arc::new(CapabilityTranslationRegistry::with_defaults()),
        }
    }
}
```

**Estimated Time**: 1 hour (add translations + integrate)

---

## 🌲 **GAP 4: Dark Forest Wiring** (Already Documented)

### **Status**: Documented in `DARK_FOREST_IMPLEMENTATION_COMPLETE_FEB_02_2026.md`

**Remaining**:
1. Wire `birdsong.*` methods to songbird JSON-RPC (2-3 hours)
2. Add beacon broadcast on startup (1 hour)
3. Hook beacon exchange into discovery (2 hours)

**Note**: Blocked by Gap 2 (introspection) and Gap 3 (discovery wiring)

---

## 📋 **EXECUTION ROADMAP**

### **Phase 1: BearDog Deployment** (30 min) 🔴 **URGENT**

- [x] Identify correct beardog CLI structure ✅
- [ ] Update deployment scripts
- [ ] Test beardog startup on USB
- [ ] Test beardog startup on Pixel

**Deliverable**: BearDog running on both devices

---

### **Phase 2: Songbird Introspection** (2 hours) 🟡 **HIGH**

- [ ] Add `primal.info` method
- [ ] Add `primal.capabilities` method
- [ ] Add `rpc.methods` method
- [ ] Wire to JSON-RPC handler
- [ ] Test with `nc -U socket`
- [ ] Commit and push

**Deliverable**: Songbird self-describes

---

### **Phase 3: BearDog Introspection** (2 hours) 🟡 **HIGH**

- [ ] Add `primal.info` method
- [ ] Add `primal.capabilities` method
- [ ] Add `rpc.methods` method
- [ ] Wire to JSON-RPC router
- [ ] Test with `nc -U socket`
- [ ] Commit and push

**Deliverable**: BearDog self-describes

---

### **Phase 4: Wire Discovery** (2 hours) 🟢 **MEDIUM**

- [ ] Update `CapabilityHandler::discover` to use `CapabilityDiscoveryService`
- [ ] Update `CapabilityHandler::call` to use runtime discovery
- [ ] Add helper for extracting runtime directory
- [ ] Test discovery finds songbird + beardog
- [ ] Commit and push

**Deliverable**: Runtime capability discovery works

---

### **Phase 5: Register Translations** (1 hour) 🟢 **MEDIUM**

- [ ] Add `with_defaults()` to `CapabilityTranslationRegistry`
- [ ] Register all songbird methods
- [ ] Register all beardog methods
- [ ] Update handler to use default registry
- [ ] Test semantic calls work
- [ ] Commit and push

**Deliverable**: `capability.call("security", "hash", data)` works

---

### **Phase 6: Dark Forest Integration** (5-9 hours) 🟣 **FINAL**

- [ ] Wire birdsong methods (songbird)
- [ ] Add genetic challenge methods (beardog)
- [ ] Add beacon broadcast (songbird)
- [ ] Hook beacon exchange (discovery)
- [ ] Test USB ↔ Pixel federation
- [ ] Document and celebrate!

**Deliverable**: Dark Forest Federation complete!

---

## 📊 **TIME ESTIMATES**

| Phase | Task | Estimate | Priority |
|-------|------|----------|----------|
| 1 | BearDog Deployment Fix | 30 min | 🔴 Urgent |
| 2 | Songbird Introspection | 2 hours | 🟡 High |
| 3 | BearDog Introspection | 2 hours | 🟡 High |
| 4 | Wire Discovery | 2 hours | 🟢 Medium |
| 5 | Register Translations | 1 hour | 🟢 Medium |
| 6 | Dark Forest Integration | 5-9 hours | 🟣 Final |

**Total Minimum**: 12.5 hours  
**Total Maximum**: 16.5 hours  
**Average**: 14.5 hours

**Breakdown by Priority**:
- 🔴 Urgent (blocking): 30 min
- 🟡 High (infrastructure): 4 hours
- 🟢 Medium (wiring): 3 hours
- 🟣 Final (feature): 5-9 hours

---

## ✅ **SUCCESS CRITERIA**

### **Infrastructure Complete**:
1. ✅ beardog running on USB + Pixel
2. ✅ songbird running on USB + Pixel
3. ✅ Both expose `primal.info`
4. ✅ `CapabilityDiscoveryService` finds both
5. ✅ `capability.call("security", "hash", data)` works
6. ✅ `capability.call("discovery", "public_ip", {})` works

### **Dark Forest Complete**:
1. ✅ USB tower generates encrypted beacon
2. ✅ Pixel tower receives beacon
3. ✅ Pixel tower decrypts beacon
4. ✅ Both complete lineage challenge-response
5. ✅ Session keys derived
6. ✅ Encrypted communication established

---

## 🎓 **KEY INSIGHTS**

### **1. Infrastructure is 95% Complete**
All the hard architectural work is done:
- ✅ `CapabilityDiscoveryService` exists
- ✅ `CapabilityHandler` exists
- ✅ `CapabilityTranslationRegistry` exists
- ✅ JSON-RPC infrastructure works

**Missing**: Just the wiring between components!

### **2. Introspection is the Linchpin**
Without `primal.info` / `primal.capabilities`:
- ❌ Can't auto-discover what primals offer
- ❌ Can't validate capability translations
- ❌ Must manually maintain registries

With introspection:
- ✅ Primals self-describe
- ✅ Discovery automatic
- ✅ Always current
- ✅ Zero manual work

### **3. Semantic Calls are the Future**
**Direct** (fragile):
```javascript
{ "method": "crypto.blake3_hash" }  // Hardcoded primal + method
```

**Semantic** (flexible):
```javascript
capability.call("security", "hash", data)
  → Translates to crypto.blake3_hash
  → Discovers beardog has "security"
  → Routes to beardog socket
```

**Benefits**:
- No hardcoded primal names
- Runtime discovery
- Flexible routing
- Graceful fallbacks

---

## 📚 **REFERENCE FILES**

### **Songbird**:
- `crates/songbird-universal-ipc/src/service.rs` - Add introspection here
- `crates/songbird-discovery/` - Beacon integration point

### **BearDog**:
- `crates/beardog-tunnel/src/unix_socket_ipc/handlers/` - Add introspection here
- `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto_handlers_genetic.rs` - Challenge-response

### **biomeOS**:
- `crates/biomeos-core/src/capability_discovery.rs` - Discovery service
- `crates/biomeos-atomic-deploy/src/handlers/capability.rs` - Wire here
- `crates/biomeos-atomic-deploy/src/capability_translation.rs` - Translations

---

## 🚀 **READY TO EXECUTE**

All gaps identified, solutions designed, implementation paths clear.

**Recommended Order**:
1. Fix BearDog deployment (30 min) - Unblocks everything
2. Add introspection (4 hours) - Enables discovery
3. Wire discovery (3 hours) - Enables semantic calls
4. Dark Forest integration (5-9 hours) - Feature complete!

**Total**: 12.5-16.5 hours to full system functionality

---

*Created*: February 2, 2026  
*Status*: Ready for execution  
*Next*: Start with Phase 1 (BearDog deployment fix)
