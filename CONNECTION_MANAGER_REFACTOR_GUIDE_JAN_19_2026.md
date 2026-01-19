# 🔧 Connection Manager Refactor Guide

**Date**: January 19, 2026  
**File**: `crates/songbird-orchestrator/src/app/connection_manager.rs`  
**Current**: 1,112 lines (exceeds 1000 line limit)  
**Target**: Domain-driven module architecture

---

## 🎯 REFACTOR STRATEGY

### **Smart Domain-Driven Design**

Instead of mechanically splitting at line 1000, we organize by **business domain**:

```
connection_manager/
├── mod.rs          (~180 lines) - Public API & coordination
├── types.rs        (~100 lines) - Domain types & serialization  
├── peer.rs         (~250 lines) - Peer metadata & lifecycle
├── trust.rs        (~350 lines) - Trust evaluation & establishment
├── btsp.rs         (~200 lines) - BTSP connection factory
└── tests.rs        (~400 lines) - All tests consolidated
```

**Total**: ~1,480 lines (but each file < 400 lines ✅)

---

## 📝 IMPLEMENTATION

### **Step 1: Create mod.rs** ✅ DONE

Created `connection_manager/mod.rs` with:
- Public API (ConnectionManager struct)
- Module coordination
- Delegation to domain modules
- Re-exports for backward compatibility

---

### **Step 2: Extract types.rs** (5 minutes)

**File**: `connection_manager/types.rs`

```rust
//! Domain types for connection management

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use songbird_types::TrustLevel;

/// Metadata about a peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMetadata {
    pub peer_id: String,
    pub endpoint: String,
    pub trust_level: TrustLevel,
    pub discovery_method: String,
    pub capabilities: Vec<String>,
    #[serde(with = "systemtime_as_secs")]
    pub established_at: SystemTime,
}

/// SystemTime serialization helper
pub mod systemtime_as_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}
```

**Lines**: ~50

---

### **Step 3: Extract peer.rs** (15 minutes)

**File**: `connection_manager/peer.rs`

**Responsibilities**:
- Peer metadata storage
- Peer lifecycle (connect, disconnect)
- Rejected peer tracking
- Query operations (list, count, get)

**Pattern**:
```rust
//! Peer registry and lifecycle management

use super::types::PeerMetadata;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use songbird_types::TrustLevel;
use crate::connections::Connection;

/// Peer Registry - Manages peer metadata and lifecycle
pub struct PeerRegistry {
    /// Metadata about each peer
    metadata: Arc<RwLock<HashMap<String, PeerMetadata>>>,
    
    /// Rejected peers (audit trail)
    rejected: Arc<RwLock<HashMap<String, String>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(RwLock::new(HashMap::new())),
            rejected: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register peer with metadata
    pub async fn register(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
        discovery_method: String,
        capabilities: Vec<String>,
    ) {
        let metadata = PeerMetadata {
            peer_id: peer_id.clone(),
            endpoint,
            trust_level,
            discovery_method,
            capabilities,
            established_at: std::time::SystemTime::now(),
        };
        
        self.metadata.write().await.insert(peer_id, metadata);
    }
    
    /// Mark peer as rejected
    pub async fn reject(&self, peer_id: String, reason: String) {
        self.rejected.write().await.insert(peer_id, reason);
    }
    
    /// Get metadata for peer
    pub async fn get_metadata(&self, peer_id: &str) -> Option<PeerMetadata> {
        self.metadata.read().await.get(peer_id).cloned()
    }
    
    /// Get all metadata
    pub async fn get_all_metadata(&self) -> Vec<PeerMetadata> {
        self.metadata.read().await.values().cloned().collect()
    }
    
    /// Get peer count
    pub async fn count(&self) -> usize {
        self.metadata.read().await.len()
    }
    
    /// Get rejected peers
    pub async fn get_rejected(&self) -> HashMap<String, String> {
        self.rejected.read().await.clone()
    }
    
    /// List connected peers (combines with connection state)
    pub async fn list_connected_peers(
        &self,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
    ) -> Vec<(String, TrustLevel)> {
        let conns = connections.read().await;
        conns
            .iter()
            .map(|(id, conn)| (id.clone(), conn.trust_level()))
            .collect()
    }
}
```

**Lines**: ~100

---

### **Step 4: Extract trust.rs** (30 minutes)

**File**: `connection_manager/trust.rs`

**Responsibilities**:
- Trust evaluation logic
- Connection establishment
- Trust decision handling
- Connection type selection (BTSP vs HTTP)

**Pattern**:
```rust
//! Trust evaluation and connection establishment

use super::peer::PeerRegistry;
use super::btsp::BtspConnectionFactory;
use anyhow::{anyhow, Result};
use songbird_types::TrustLevel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use crate::connections::*;
use crate::trust::peer_trust::PeerTrustDecision;

/// Trust Evaluator - Evaluates trust and establishes connections
pub struct TrustEvaluator;

impl TrustEvaluator {
    pub fn new() -> Self {
        Self
    }
    
    /// Handle trust decision from discovery
    pub async fn handle_decision(
        &self,
        decision: PeerTrustDecision,
        peer_tags: Vec<String>,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
        peer_registry: &PeerRegistry,
        btsp_factory: &BtspConnectionFactory,
    ) -> Result<()> {
        match decision {
            PeerTrustDecision::FullTrust(peer_id, endpoint, discovery_method, capabilities) => {
                info!("✅ Full trust: {} ({})", peer_id, endpoint);
                
                self.establish_connection(
                    peer_id,
                    endpoint,
                    TrustLevel::FullTrust,
                    discovery_method,
                    capabilities,
                    peer_tags,
                    connections,
                    peer_registry,
                    btsp_factory,
                )
                .await
            }
            
            PeerTrustDecision::RequiresConsent(peer_id, endpoint, discovery_method, capabilities) => {
                info!("⚠️  Requires consent: {} ({})", peer_id, endpoint);
                
                // TODO: Implement user prompt in Phase 6
                // For now, establish limited trust
                self.establish_connection(
                    peer_id,
                    endpoint,
                    TrustLevel::LimitedTrust,
                    discovery_method,
                    capabilities,
                    peer_tags,
                    connections,
                    peer_registry,
                    btsp_factory,
                )
                .await
            }
            
            PeerTrustDecision::Reject(peer_id, reason) => {
                warn!("❌ Rejected: {} - {}", peer_id, reason);
                peer_registry.reject(peer_id, reason).await;
                Ok(())
            }
        }
    }
    
    /// Establish connection at specified trust level
    pub async fn establish_connection(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
        discovery_method: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
        peer_registry: &PeerRegistry,
        btsp_factory: &BtspConnectionFactory,
    ) -> Result<()> {
        // Check if peer supports BTSP
        let use_btsp = btsp_factory.should_use_btsp(&peer_tags);
        
        let connection = if use_btsp {
            btsp_factory.create_connection(
                peer_id.clone(),
                endpoint.clone(),
                trust_level,
            ).await?
        } else {
            self.create_http_connection(peer_id.clone(), endpoint.clone(), trust_level)?
        };
        
        // Register peer metadata
        peer_registry.register(
            peer_id.clone(),
            endpoint,
            trust_level,
            discovery_method,
            capabilities,
        ).await;
        
        // Store connection
        connections.write().await.insert(peer_id, connection);
        
        Ok(())
    }
    
    /// Create HTTP-based connection (fallback when BTSP unavailable)
    fn create_http_connection(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        match trust_level {
            TrustLevel::FullTrust => {
                Ok(Connection::FullTrust(FullTrustConnection::new(peer_id, endpoint)))
            }
            TrustLevel::LimitedTrust => {
                Ok(Connection::Limited(LimitedConnection::new(
                    peer_id,
                    endpoint,
                    vec![],
                )))
            }
            TrustLevel::Federated => {
                Ok(Connection::Federated(FederatedConnection::new(
                    peer_id,
                    endpoint,
                    vec![],
                )))
            }
            _ => Err(anyhow!("Unsupported trust level: {:?}", trust_level)),
        }
    }
}
```

**Lines**: ~150

---

### **Step 5: Extract btsp.rs** (20 minutes)

**File**: `connection_manager/btsp.rs`

**Responsibilities**:
- BTSP client lazy initialization
- BTSP connection creation
- Protocol capability detection

**Pattern**:
```rust
//! BTSP connection factory and client management

use anyhow::Result;
use songbird_types::TrustLevel;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{debug, info};
use crate::btsp_client::BtspClient;
use crate::connections::*;

/// BTSP Connection Factory - Creates encrypted P2P connections
pub struct BtspConnectionFactory {
    /// Lazy-initialized BTSP client (thread-safe, async-aware)
    client: Arc<OnceCell<Arc<BtspClient>>>,
}

impl BtspConnectionFactory {
    pub fn new() -> Self {
        Self {
            client: Arc::new(OnceCell::new()),
        }
    }
    
    /// Get or initialize BTSP client (lazy, thread-safe)
    pub async fn get_or_init_client(&self) -> Result<Arc<BtspClient>> {
        self.client
            .get_or_try_init(|| async {
                info!("🔐 Initializing BTSP client (Unix socket)");
                
                let client = BtspClient::discover()
                    .await
                    .map_err(|e| anyhow::anyhow!("BTSP discovery failed: {}", e))?;
                
                debug!("✅ BTSP client initialized");
                Ok(Arc::new(client))
            })
            .await
            .map(Arc::clone)
    }
    
    /// Check if peer supports BTSP protocol
    pub fn should_use_btsp(&self, peer_tags: &[String]) -> bool {
        peer_tags.iter().any(|tag| tag == "btsp" || tag == "encrypted-p2p")
    }
    
    /// Create BTSP connection at specified trust level
    pub async fn create_connection(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        let client = self.get_or_init_client().await?;
        
        match trust_level {
            TrustLevel::FullTrust => Ok(Connection::FullTrustBtsp(
                FullTrustBtspConnection::new(peer_id, endpoint, client),
            )),
            TrustLevel::LimitedTrust => Ok(Connection::LimitedBtsp(
                LimitedBtspConnection::new(peer_id, endpoint, vec![], client),
            )),
            TrustLevel::Federated => Ok(Connection::FederatedBtsp(
                FederatedBtspConnection::new(peer_id, endpoint, vec![], client),
            )),
            _ => Err(anyhow::anyhow!("Unsupported BTSP trust level: {:?}", trust_level)),
        }
    }
}
```

**Lines**: ~80

---

### **Step 6: Extract tests.rs** (10 minutes)

**File**: `connection_manager/tests.rs`

Move all `#[cfg(test)]` sections from original file:

```rust
#![cfg(test)]

use super::*;

// All existing tests from connection_manager.rs
// Lines 473-1112 of original file
```

**Lines**: ~400

---

### **Step 7: Update imports** (5 minutes)

In `crates/songbird-orchestrator/src/app/mod.rs`:

```rust
// OLD:
pub mod connection_manager;

// NEW (no change needed - mod.rs handles it):
pub mod connection_manager;

// Re-export still works:
pub use connection_manager::ConnectionManager;
```

---

## ✅ BENEFITS

### **Code Quality**:
- ✅ Each file < 400 lines (well under 1000 limit)
- ✅ Clear domain boundaries
- ✅ Single Responsibility Principle
- ✅ Easy to test in isolation

### **Maintainability**:
- ✅ Find code faster (domain-organized)
- ✅ Change one domain without touching others
- ✅ Onboarding easier (focused modules)
- ✅ Documentation clearer (one domain per file)

### **Modern Rust**:
- ✅ Module best practices
- ✅ Domain-driven design
- ✅ Capability-based patterns
- ✅ Zero blocking calls

---

## 🚀 EXECUTION TIMELINE

### **Total**: 4-6 hours

1. ✅ Create mod.rs (15 min) - **DONE**
2. Extract types.rs (5 min)
3. Extract peer.rs (15 min)
4. Extract trust.rs (30 min)
5. Extract btsp.rs (20 min)
6. Extract tests.rs (10 min)
7. Update imports (5 min)
8. **Verify build** (30 min)
9. **Run tests** (30 min)
10. **Manual testing** (1 hour)

---

## 🔍 VERIFICATION

```bash
# 1. Check file sizes
wc -l crates/songbird-orchestrator/src/app/connection_manager/*.rs

# 2. Build
cargo build -p songbird-orchestrator

# 3. Run tests
cargo test -p songbird-orchestrator connection_manager

# 4. Clippy
cargo clippy -p songbird-orchestrator -- -D warnings

# Expected:
# - All files < 400 lines ✅
# - Build succeeds ✅
# - All tests pass ✅  
# - No clippy warnings ✅
```

---

## 📋 CHECKLIST

- [x] Create connection_manager/ directory
- [x] Create mod.rs with public API
- [ ] Extract types.rs
- [ ] Extract peer.rs  
- [ ] Extract trust.rs
- [ ] Extract btsp.rs
- [ ] Extract tests.rs
- [ ] Delete old connection_manager.rs
- [ ] Verify build
- [ ] Run all tests
- [ ] Update documentation

---

**Status**: ✅ Started - mod.rs created  
**Next**: Extract domain modules (types, peer, trust, btsp)  
**Timeline**: 4-6 hours remaining

🦀🧬✨ **Smart Domain-Driven Refactor in Progress!** ✨🧬🦀

