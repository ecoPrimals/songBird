# 🔐 BearDog JSON-RPC Method Mapping Reference

**Date**: January 19, 2026  
**Source**: `/home/eastgate/Development/ecoPrimals/phase1/beardog/crates/beardog-tunnel/src/unix_socket_ipc/handlers.rs`  
**Purpose**: Enable Songbird → BearDog Unix socket migrations  
**Socket**: `/tmp/beardog.sock` (or discovered via environment)

---

## 📋 COMPLETE METHOD CATALOG

### **Universal Methods** (Health/Discovery)
| JSON-RPC Method | HTTP Endpoint | Description |
|-----------------|---------------|-------------|
| `ping` / `health` / `status` / `check` | `/health` | Health check |
| `capabilities` / `get_capabilities` | `/capabilities` | List all capabilities |
| `identity` / `whoami` / `get_identity` | `/identity` | Get primal identity |

### **Encryption Capability** (BirdSong)
| JSON-RPC Method | HTTP Endpoint | Description |
|-----------------|---------------|-------------|
| `birdsong.encrypt` | `/api/birdsong/encrypt` | Encrypt with family lineage |
| `birdsong.decrypt` | `/api/birdsong/decrypt` | Decrypt with family lineage |
| `encryption.encrypt` | `/encrypt` | Generic encrypt |
| `encryption.decrypt` | `/decrypt` | Generic decrypt |

### **Security/Trust Capability**
| JSON-RPC Method | HTTP Endpoint | Description |
|-----------------|---------------|-------------|
| `security.evaluate` / `trust.evaluate` | `/security/evaluate` | Evaluate trust level |
| `security.lineage` / `trust.lineage` | `/security/lineage` | Get genetic lineage |
| `security.generate_jwt_secret` | `/security/jwt/generate` | Generate JWT secret |

### **BTSP Capability** (Tunnel Protocol)
| JSON-RPC Method | Description |
|-----------------|-------------|
| `btsp.contact_exchange` | Exchange contact info |
| `btsp.tunnel_establish` | Establish secure tunnel |
| `btsp.tunnel_encrypt` | Encrypt via tunnel |
| `btsp.tunnel_decrypt` | Decrypt via tunnel |
| `btsp.tunnel_status` | Get tunnel status |
| `btsp.tunnel_close` | Close tunnel |

### **Crypto Capability** (Low-Level, used by Songbird TLS)
| JSON-RPC Method | Description |
|-----------------|-------------|
| `crypto.sign_ed25519` | Sign with Ed25519 |
| `crypto.verify_ed25519` | Verify Ed25519 signature |
| `crypto.x25519_generate_ephemeral` | Generate X25519 ephemeral keypair |
| `crypto.x25519_derive_secret` | Derive X25519 shared secret |
| `crypto.chacha20_poly1305_encrypt` | ChaCha20-Poly1305 encrypt |
| `crypto.chacha20_poly1305_decrypt` | ChaCha20-Poly1305 decrypt |
| `crypto.blake3_hash` | Blake3 hash |
| `crypto.hmac_sha256` | HMAC-SHA256 |

### **Graph Intelligence Capability**
| JSON-RPC Method | Description |
|-----------------|-------------|
| `graph.authorize_modification` | Authorize graph modification |
| `graph.validate_template` | Validate graph template |
| `graph.audit_origin` | Audit graph origin |

---

## 🔄 MIGRATION PATTERNS

### **Pattern 1: Health Check**

**Before** (HTTP):
```rust
let health_url = format!("{}/health", endpoint);
let response = client.get(&health_url).send().await?;
```

**After** (Unix RPC):
```rust
let client = UnixRpcClient::new("/tmp/beardog.sock")?;
let health: HealthResponse = client.call_no_params("health").await?;
```

### **Pattern 2: BirdSong Encrypt**

**Before** (HTTP):
```rust
let url = format!("{}/api/birdsong/encrypt", endpoint);
let response = client
    .post(&url)
    .json(&BearDogEncryptRequest { plaintext, family_id })
    .send()
    .await?;
let result: BearDogEncryptResponse = response.json().await?;
```

**After** (Unix RPC):
```rust
let client = UnixRpcClient::new("/tmp/beardog.sock")?;
let result: BearDogEncryptResponse = client
    .call("birdsong.encrypt", BearDogEncryptRequest { plaintext, family_id })
    .await?;
```

### **Pattern 3: Security Capability**

**Before** (HTTP):
```rust
let url = format!("{}/tokens", endpoint);
let response = http_client
    .post(&url)
    .json(&TokenRequest { ... })
    .send()
    .await?;
```

**After** (Unix RPC):
```rust
let client = UnixRpcClient::new("/tmp/beardog.sock")?;
let result: TokenResponse = client
    .call("security.generate_token", TokenRequest { ... })
    .await?;
```

**Note**: Need to verify actual method name - may be `security.generate_jwt_secret` or custom mapping

---

## 📁 FILES TO MIGRATE (10 Critical)

### **Priority 1: BearDog Integration** (2 files)

#### **1. `beardog_birdsong_provider.rs`**
**Location**: `crates/songbird-discovery/src/beardog_birdsong_provider.rs`

**HTTP Calls to Replace**:
- `GET /health` → `health` or `ping`
- `POST /api/birdsong/encrypt` → `birdsong.encrypt`
- `POST /api/birdsong/decrypt` → `birdsong.decrypt`

**Complexity**: Medium (7 HTTP calls, v1/v2 endpoint fallback)  
**Estimate**: 45-60 min

#### **2. `security_capability_client.rs`**
**Location**: `crates/songbird-primal-sdk/src/security_capability_client.rs`

**HTTP Calls to Replace**:
- `POST /tokens` → Need to verify method (possibly `security.generate_jwt_secret`)
- `POST /tokens/validate` → Need to verify method (possibly `security.validate_token`)
- `POST /encrypt` → `encryption.encrypt`
- `POST /decrypt` → `encryption.decrypt`

**Complexity**: Medium-High (4 HTTP calls, method names need verification)  
**Estimate**: 30-45 min

---

### **Priority 2: Core Primal SDK** (3 files)

#### **3. `toadstool.rs`**
**Location**: `crates/songbird-primal-sdk/src/toadstool.rs`

**HTTP Calls to Replace**: AI capability endpoints (need to verify Toadstool's RPC interface)  
**Estimate**: 20-30 min

#### **4. `ai_capability.rs`**
**Location**: `crates/songbird-primal-sdk/src/ai_capability.rs`

**HTTP Calls to Replace**: AI capability endpoints  
**Estimate**: 15-20 min

#### **5. `capability_orchestrator.rs`**
**Location**: `crates/songbird-primal-sdk/src/capability_orchestrator.rs`

**HTTP Calls to Replace**: Multi-primal orchestration  
**Estimate**: 20-30 min

---

### **Priority 3: Discovery Engine** (3 files)

#### **6. `discovery/engine.rs`**
**Location**: `crates/songbird-primal-sdk/src/discovery/engine.rs`

**HTTP Calls to Replace**: Primal discovery via HTTP → Unix socket discovery  
**Estimate**: 15-20 min

#### **7. `discovery/universal_primal_support.rs`**
**Location**: `crates/songbird-primal-sdk/src/discovery/universal_primal_support.rs`

**HTTP Calls to Replace**: Universal primal HTTP checks → Unix socket  
**Estimate**: 15-20 min

#### **8. `discovery/capability_probing.rs`**
**Location**: `crates/songbird-primal-sdk/src/discovery/capability_probing.rs`

**HTTP Calls to Replace**: Capability probing via HTTP → `/capabilities` RPC method  
**Estimate**: 10-15 min

---

### **Priority 4: Ecosystem Discovery** (2 files)

#### **9. `discovery/ecosystem/mod.rs`**
**Location**: `crates/songbird-primal-sdk/src/discovery/ecosystem/mod.rs`

**HTTP Calls to Replace**: Ecosystem-wide discovery  
**Estimate**: 15-20 min

#### **10. `discovery/ecosystem/filesystem.rs`**
**Location**: `crates/songbird-primal-sdk/src/discovery/ecosystem/filesystem.rs`

**HTTP Calls to Replace**: Filesystem-based discovery helpers  
**Estimate**: 10-15 min

---

## 🎯 MIGRATION STRATEGY

### **Step 1**: Update BearDog Integration (1-1.5 hrs)
- Migrate `beardog_birdsong_provider.rs`
- Migrate `security_capability_client.rs`
- Test encrypt/decrypt flows
- **Result**: Core BearDog integration on Unix sockets

### **Step 2**: Update Core SDK (30-45 min)
- Migrate `toadstool.rs`, `ai_capability.rs`, `capability_orchestrator.rs`
- May need to discover Toadstool's Unix socket interface
- **Result**: AI capabilities on Unix sockets

### **Step 3**: Update Discovery (30-45 min)
- Migrate `discovery/engine.rs`, `discovery/universal_primal_support.rs`, `discovery/capability_probing.rs`
- Use `capabilities` RPC method for discovery
- **Result**: Discovery system on Unix sockets

### **Step 4**: Update Ecosystem (20-30 min)
- Migrate `discovery/ecosystem/mod.rs`, `discovery/ecosystem/filesystem.rs`
- **Result**: Full ecosystem discovery on Unix sockets

---

## ✅ EXPECTED RESULTS

### **After Migration**
- **Pure Rust**: 99.2% → **99.5%+** 🎉
- **reqwest usage**: 95 files → **85 files** (10 migrated)
- **Ring sources**: Still 1 (reqwest), but 10 fewer usages
- **Inter-primal perf**: Faster (Unix sockets > HTTP)
- **Security**: Better (filesystem permissions)

---

## 📝 NOTES FOR NEXT SESSION

### **Prerequisites**
1. ✅ UnixRpcClient ready and tested
2. ✅ BearDog method mapping documented (this file!)
3. ❓ Need to verify Toadstool's Unix socket interface
4. ❓ May need to verify exact method names for token operations

### **Quick Wins**
- Health checks: `health` or `ping` method
- Capabilities: `capabilities` method
- Encrypt/decrypt: `birdsong.encrypt` / `birdsong.decrypt`

### **Challenges**
- Toadstool RPC interface may need discovery
- Token method names need verification (`security.generate_jwt_secret` vs custom)
- Some primals may not have Unix sockets yet (fallback to reqwest?)

### **Testing**
1. Unit tests for each migrated file
2. Integration test with live BearDog instance
3. Verify no regressions in discovery/security flows

---

## 🦀 **READY FOR EXECUTION!** 🦀

**Status**: Foundation complete, method mapping documented  
**Next Session**: Execute migration with this reference  
**Estimate**: 2-3 hours for all 10 files  
**Result**: 99.2% → 99.5%+ Pure Rust!

---

**See you next session for the final push to 99.5%+!** 🚀

