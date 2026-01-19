# 🚀 Phase 3 Execution Plan - reqwest Migration

**Date**: January 19, 2026  
**Status**: Ready to Execute  
**Scope**: 95 files → Focused subset first

---

## 🎯 PRAGMATIC APPROACH

### **Reality Check**
- **Full scope**: 95 files, 14-20 hours
- **Current session**: Already 6+ hours
- **Recommendation**: **Focus on highest-impact subset NOW**

### **Strategy**: Incremental Value Delivery

**Phase 3A** (NOW, 2-3 hours): **Core Inter-Primal** (8-10 critical files)
- BearDog client
- Core primal SDK
- Highest-traffic paths

**Phase 3B** (Future): Remaining inter-primal (20-30 files, 4-6 hours)

**Phase 3C** (Future): External HTTP (20-30 files, 4-6 hours)

**Phase 3D** (Future): Tests/Gateway (15-20 files, 4-6 hours)

---

## 📋 PHASE 3A: CORE INTER-PRIMAL (NOW)

### **Goal**: Migrate critical inter-primal HTTP → Unix sockets

### **Target Files** (8-10 files, highest impact)

#### **Priority 1: BearDog Integration** 🔐
1. `crates/songbird-discovery/src/beardog_birdsong_provider.rs`
   - **Current**: Uses `reqwest::Client` for HTTP
   - **Target**: Unix socket JSON-RPC
   - **Impact**: HIGH (crypto operations)

2. `crates/songbird-primal-sdk/src/security_capability_client.rs`
   - **Current**: Uses `reqwest::Client`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: HIGH (security operations)

#### **Priority 2: Core Primal SDK** 🌐
3. `crates/songbird-primal-sdk/src/toadstool.rs`
   - **Current**: Uses `reqwest::Client` for AI
   - **Target**: Unix socket JSON-RPC
   - **Impact**: HIGH (AI workloads)

4. `crates/songbird-primal-sdk/src/ai_capability.rs`
   - **Current**: Uses `reqwest::Client`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: MEDIUM

5. `crates/songbird-primal-sdk/src/capability_orchestrator.rs`
   - **Current**: Uses `reqwest::Client::builder()`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: MEDIUM

#### **Priority 3: Discovery Engine** 🔍
6. `crates/songbird-primal-sdk/src/discovery/engine.rs`
   - **Current**: Uses `reqwest::Client::new()`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: MEDIUM

7. `crates/songbird-primal-sdk/src/discovery/universal_primal_support.rs`
   - **Current**: Uses `reqwest::get()`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: MEDIUM

8. `crates/songbird-primal-sdk/src/discovery/capability_probing.rs`
   - **Current**: Uses `reqwest::Client::new()`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: LOW-MEDIUM

#### **Priority 4: Ecosystem Discovery** 🌍
9. `crates/songbird-primal-sdk/src/discovery/ecosystem/mod.rs`
   - **Current**: Uses `reqwest::Client::builder()`
   - **Target**: Unix socket JSON-RPC
   - **Impact**: LOW-MEDIUM

10. `crates/songbird-primal-sdk/src/discovery/ecosystem/filesystem.rs`
    - **Current**: Uses `reqwest::Client` parameter
    - **Target**: Unix socket JSON-RPC
    - **Impact**: LOW

---

## 🛠️ MIGRATION PATTERN

### **From: HTTP via reqwest**
```rust
use reqwest::Client;

let client = Client::new();
let response = client
    .get(&format!("{}/api/method", endpoint))
    .json(&params)
    .send()
    .await?;
let result: Response = response.json().await?;
```

### **To: Unix Socket JSON-RPC**
```rust
use tokio::net::UnixStream;
use serde_json::json;

let stream = UnixStream::connect("/tmp/primal.sock").await?;
let request = json!({
    "jsonrpc": "2.0",
    "method": "method_name",
    "params": params,
    "id": 1
});

// Send request
stream.write_all(&serde_json::to_vec(&request)?).await?;

// Read response
let mut buf = vec![0; 4096];
let n = stream.read(&mut buf).await?;
let response: JsonRpcResponse = serde_json::from_slice(&buf[..n])?;
```

### **Better: Reusable Helper**
```rust
// Create in songbird-primal-sdk/src/unix_rpc_client.rs

pub struct UnixRpcClient {
    socket_path: PathBuf,
}

impl UnixRpcClient {
    pub async fn call<P, R>(&self, method: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        // ... send and receive ...
        
        Ok(result)
    }
}
```

---

## 📊 PHASE 3A IMPACT

### **Before**
- reqwest in 10 critical files
- HTTP overhead for inter-primal communication
- ring dependency via reqwest → rustls

### **After**
- Unix sockets in 10 critical files
- Zero HTTP overhead
- Potential: 99.2% → 99.5%+ Pure Rust

### **Estimated Effort**
- Create `UnixRpcClient` helper: 30-45 min
- Migrate 10 files: 1.5-2 hours
- Test and verify: 30-45 min
- **Total**: 2.5-3.5 hours

---

## ✅ SUCCESS CRITERIA (Phase 3A)

### **Technical**
- [ ] `UnixRpcClient` helper created
- [ ] 10 critical files migrated
- [ ] All builds successfully
- [ ] Tests passing

### **Quality**
- [ ] Modern idiomatic Rust
- [ ] Zero unsafe code
- [ ] Comprehensive error handling
- [ ] Documentation updated

### **Impact**
- [ ] Core inter-primal → Unix sockets ✅
- [ ] 99.2% → 99.5%+ Pure Rust
- [ ] Foundation for remaining migrations

---

## 🎯 DECISION POINT

### **Option A**: Execute Phase 3A NOW (2.5-3.5 hours)
- Migrate 10 critical files
- Unix socket foundation
- Result: 99.2% → 99.5%+ Pure Rust

### **Option B**: Document and defer
- Already worked 6+ hours
- Excellent progress (99.2%)
- Complete Phase 3 in future sessions

---

## 💡 RECOMMENDATION

### **Option A: Execute Phase 3A NOW**

**Why**:
1. **High impact** - Core inter-primal communication
2. **Manageable scope** - 10 files, 2.5-3.5 hours
3. **Foundation** - Enables future migrations
4. **Momentum** - We're on a roll!

**Result**: 99.2% → 99.5%+ Pure Rust in ONE session!

---

## 📋 EXECUTION CHECKLIST

### **Step 1**: Create UnixRpcClient (30-45 min)
- [ ] Create `songbird-primal-sdk/src/unix_rpc_client.rs`
- [ ] Implement `call()` method
- [ ] Add error handling
- [ ] Write tests

### **Step 2**: Migrate Priority 1 (45-60 min)
- [ ] `beardog_birdsong_provider.rs`
- [ ] `security_capability_client.rs`

### **Step 3**: Migrate Priority 2 (30-45 min)
- [ ] `toadstool.rs`
- [ ] `ai_capability.rs`
- [ ] `capability_orchestrator.rs`

### **Step 4**: Migrate Priority 3 & 4 (30-45 min)
- [ ] `discovery/engine.rs`
- [ ] `discovery/universal_primal_support.rs`
- [ ] `discovery/capability_probing.rs`
- [ ] `discovery/ecosystem/mod.rs`
- [ ] `discovery/ecosystem/filesystem.rs`

### **Step 5**: Test and Verify (30-45 min)
- [ ] Build successful
- [ ] Tests passing
- [ ] Commit and push

---

## 🚀 READY TO EXECUTE!

**Estimated Time**: 2.5-3.5 hours  
**Impact**: 99.2% → 99.5%+ Pure Rust  
**Status**: Phase 3A ready to begin!

---

🦀✨ **Let's achieve 99.5%+ Pure Rust in this session!** ✨🦀

