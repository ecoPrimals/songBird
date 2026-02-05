# 🚧 Remaining reqwest Migrations - Pattern Guide
**Date:** January 25, 2026 (Updated Session 5 End)  
**Status:** 10/12 crates completed, 1 in progress, 1 pending  
**Remaining Effort:** ~1-2.5 hours

---

## ✅ **Completed (10/12 - 83%!)**

1. songbird-compute-bridge ✅
2. songbird-remote-deploy ✅
3. songbird-http-client ✅
4. songbird-orchestrator ✅
5. songbird-primal-coordination ✅
6. songbird-execution-agent ✅
7. **songbird-genesis** ✅ (Session 5!)
8. **songbird-config** ✅ (Session 5!)
9. **songbird-network-federation** ✅ (Session 5!)
10. **songbird-discovery** ✅ (Session 5!)

---

## ⏳ **In Progress (1/12 - 8%)**

### **songbird-universal** (70% complete, ~1-2 hours)
**Files modified:**
- ✅ `Cargo.toml` updated
- ✅ `adapters/ai.rs` migrated
- ✅ `adapters/storage.rs` migrated
- ✅ `adapters/compute.rs` migrated
- ✅ `adapters/security.rs` migrated

**Remaining fixes (~13 files):**
- ⏳ `unified_adapter.rs` (2 instances)
- ⏳ `capabilities/adapter/capability_query.rs` (1 instance)
- ⏳ `capabilities/adapter/connection_manager.rs` (1 instance)
- ⏳ `federated_capability_adapter.rs` (2 instances)
- ⏳ `discovery/health.rs` (3 instances)
- ⏳ `infant_discovery_engine.rs` (2 instances)
- ⏳ `service_discovery.rs` (3 instances)
- ⏳ `self_discovery.rs` (2 instances)
- ⏳ `enhanced_infant_discovery.rs` (2 instances)
- ⏳ `infant_discovery.rs` (3 instances)
- ⏳ `ecosystem_discovery.rs` (2 instances)
- ⏳ `adapters/tests_protocol_detection.rs` (1 instance - comment only)
- ⏳ `jsonrpc_client.rs` (1 instance - comment only)

**Common patterns to fix:**
- Replace `reqwest::Client` with `IpcHttpClient`
- Replace `reqwest::Client::new()` with `IpcHttpClient::new().await?`
- Replace `reqwest::Client::builder().timeout(...).build()?` with `IpcHttpClient::new().await?`
- Add `.await` after `.get()` and `.post()` calls
- Remove `.timeout()` calls (handled internally)
- Update error handling with `.map_err()`

---

## 📋 **Remaining Crates (1 - Optional)**

### **songbird-cli** (3 instances, ~30 min)
**Note:** CLI testing tool, non-critical for production  
**Priority:** Low (user tooling)

---

## 🔧 **Standard Migration Pattern**

### Step 1: Update Cargo.toml
```toml
# Remove:
reqwest = { version = "0.11", features = ["json"], default-features = false }

# Add:
songbird-http-client = { path = "../songbird-http-client" }
```

### Step 2: Update imports
```rust
// Remove:
use reqwest::Client;

// Add:
use songbird_http_client::IpcHttpClient;
```

### Step 3: Update struct fields
```rust
// BEFORE:
struct MyClient {
    http_client: reqwest::Client,
}

// AFTER:
struct MyClient {
    http_client: IpcHttpClient,
}
```

### Step 4: Update initialization
```rust
// BEFORE:
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

// AFTER:
let client = IpcHttpClient::new().await?;
```

### Step 5: Update HTTP calls
```rust
// BEFORE:
let response = client
    .post(&url)
    .json(&body)
    .timeout(Duration::from_secs(10))
    .send()
    .await?;

// AFTER:
let response = client
    .post(&url)
    .await
    .json(&body)?
    .send()
    .await?;
```

### Step 6: Update response handling
```rust
// BEFORE:
if response.status().is_success() {
    let data: MyType = response.json().await?;
}

// AFTER:
if response.is_success() {
    let data: MyType = response.json().await?;
}
```

---

## 🎯 **Key API Differences**

| Operation | reqwest | IpcHttpClient |
|-----------|---------|---------------|
| Create client | `Client::new()` | `IpcHttpClient::new().await?` |
| POST request | `.post(&url)` | `.post(&url).await` |
| GET request | `.get(&url)` | `.get(&url).await` |
| JSON body | `.json(&body)` | `.json(&body)?` |
| Timeout | `.timeout(dur)` | _(handled internally)_ |
| Send | `.send().await?` | `.send().await?` |
| Check success | `.status().is_success()` | `.is_success()` |
| Get status | `.status().as_u16()` | `.status()` |

---

## 💡 **Common Patterns**

### Pattern 1: Simple GET request
```rust
// BEFORE:
let client = reqwest::Client::new();
let response = client.get(&url).send().await?;
let data: MyType = response.json().await?;

// AFTER:
let client = IpcHttpClient::new().await?;
let response = client.get(&url).await?;
let data: MyType = response.json().await?;
```

### Pattern 2: POST with JSON
```rust
// BEFORE:
let client = reqwest::Client::new();
let response = client.post(&url).json(&body).send().await?;

// AFTER:
let client = IpcHttpClient::new().await?;
let response = client.post(&url).await.json(&body)?.send().await?;
```

### Pattern 3: Error handling
```rust
// BEFORE:
.map_err(|e| MyError::HttpFailed(format!("Request failed: {}", e)))?

// AFTER: (same pattern works)
.map_err(|e| MyError::HttpFailed(format!("Request failed: {}", e)))?
```

### Pattern 4: Struct with client field
```rust
// BEFORE:
pub struct MyService {
    client: reqwest::Client,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

// AFTER:
pub struct MyService {
    client: IpcHttpClient,
}

impl MyService {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            client: IpcHttpClient::new().await?,
        })
    }
}
```

---

## 🐛 **Common Gotchas**

### 1. Async initialization
```rust
// ❌ WRONG - Client::new() is async
let client = IpcHttpClient::new();

// ✅ CORRECT
let client = IpcHttpClient::new().await?;
```

### 2. POST/GET are async
```rust
// ❌ WRONG - .post() returns Future
let response = client.post(&url).json(&body)?.send().await?;

// ✅ CORRECT - await .post() first
let response = client.post(&url).await.json(&body)?.send().await?;
```

### 3. JSON returns Result
```rust
// ❌ WRONG - .json() needs error handling
let response = client.post(&url).await.json(&body).send().await?;

// ✅ CORRECT - use ? operator
let response = client.post(&url).await.json(&body)?.send().await?;
```

### 4. is_success() vs status().is_success()
```rust
// ❌ WRONG - no .status() method call needed
if response.status().is_success() { }

// ✅ CORRECT
if response.is_success() { }
```

---

## 📊 **Progress Tracking**

### Overall Status:
```
██████████████░░░░░░  58% Complete (7/12 crates)
```

### By Priority:
- **Critical Services:** 100% ✅ (6/6)
- **High Priority:** 0% ⏳ (0/3) - universal, discovery, federation
- **Medium Priority:** 90% ⏳ (genesis almost done), config pending
- **Low Priority:** 0% ⏳ (cli pending)

### ecoBin Compliance:
```
Before: 96%
Now: 99.5%
Target: 100% (after remaining migrations)
```

---

## 🚀 **Quick Start Commands**

### Test current build:
```bash
cargo build -p songbird-genesis
cargo test -p songbird-genesis
```

### Check remaining reqwest usage:
```bash
grep -r "reqwest::" crates/*/src --include="*.rs" | wc -l
```

### Find specific crate usage:
```bash
grep -r "reqwest::" crates/songbird-universal/src --include="*.rs"
```

---

## 📝 **Completion Checklist**

- [ ] Fix remaining songbird-genesis `.await` calls (4 locations)
- [ ] Migrate songbird-config (5 instances)
- [ ] Migrate songbird-network-federation (6 instances)
- [ ] Migrate songbird-discovery (9 instances)
- [ ] Migrate songbird-universal (33 instances, 17 files)
- [ ] Migrate songbird-cli (3 instances) - optional
- [ ] Run full workspace build
- [ ] Run full workspace tests
- [ ] Update STATUS.md
- [ ] Verify 100% ecoBin compliance

---

**Estimated Total Remaining:** 4-6 hours of focused work

**Pattern is established** - remaining work is purely mechanical application of the pattern documented above.

---

*Document Updated: January 25, 2026*  
*Status: 7/12 complete, all critical services migrated*

