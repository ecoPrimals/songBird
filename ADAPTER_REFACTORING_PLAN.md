# 🎯 Adapter.rs Smart Refactoring Plan

**File**: `crates/songbird-universal/src/capabilities/adapter.rs`  
**Current**: 1080 lines, monolithic  
**Target**: 200 lines core + 6 focused modules

---

## 📊 Current Structure Analysis

**Identified Sections** (by line ranges and responsibility):

1. **Lines 1-50**: Core struct, constructor, imports
2. **Lines 52-290**: Discovery logic (238 lines)
3. **Lines 291-480**: Capability querying & inference (189 lines)
4. **Lines 498-595**: Connection management (97 lines)
5. **Lines 605-665**: Federation methods (60 lines, mostly stubs)
6. **Lines 666-1080**: Helper methods, utilities, caching (414 lines)

**Method Count**: 45+ public/private methods

---

## 🎯 Extraction Strategy

### Module 1: `adapter/discovery.rs` (250 lines)

**Extract lines 52-290**:
- `discover_primal_capabilities()`
- `find_capability_providers()`
- `discover_capability_providers_from_env()`
- `discover_capability_providers_from_network()`
- `infer_capability_providers()`

**New struct**:
```rust
pub struct CapabilityDiscovery {
    registry: Arc<RwLock<CapabilityRegistry>>,
    config: DiscoveryConfig,
}

impl CapabilityDiscovery {
    pub async fn discover_primal(&self, name: &str) -> Result<Vec<Capability>>;
    pub async fn find_providers(&self, cap_type: &str) -> Vec<String>;
    async fn discover_from_env(&self, cap_type: &str) -> Vec<String>;
    async fn discover_from_network(&self, cap_type: &str) -> Vec<String>;
    async fn infer_providers(&self, cap_type: &str) -> Vec<String>;
}
```

### Module 2: `adapter/capability_query.rs` (200 lines)

**Extract lines 291-480**:
- `check_if_primal_provides_capability()`
- `get_best_primal_for_capability()`
- `query_primal_capabilities()`
- `infer_basic_capabilities()`

**New struct**:
```rust
pub struct CapabilityQuery {
    registry: Arc<RwLock<CapabilityRegistry>>,
    http_client: reqwest::Client,
}

impl CapabilityQuery {
    pub async fn query_capabilities(&self, endpoint: &str) -> Result<Vec<Capability>>;
    pub async fn get_best_primal(&self, cap_type: &str) -> Option<String>;
    async fn check_provides(&self, primal: &str, capability: &str) -> bool;
    async fn infer_capabilities(&self, name: &str, endpoint: &str) -> Vec<Capability>;
}
```

### Module 3: `adapter/connection.rs` (150 lines)

**Extract lines 498-595**:
- `establish_connection()`
- `test_primal_health()`
- `get_all_connections()`
- `disconnect_from_primal()`
- `update_connection_health()`

**New struct**:
```rust
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, PrimalConnection>>>,
    registry: Arc<RwLock<CapabilityRegistry>>,
}

impl ConnectionManager {
    pub async fn connect(&self, primal_info: &PrimalInfo) -> Result<()>;
    pub async fn disconnect(&self, primal_name: &str) -> Result<()>;
    pub async fn test_health(&self, connection: &PrimalConnection) -> Result<bool>;
    pub async fn update_all_health(&self) -> Result<()>;
    pub async fn get_all(&self) -> Vec<PrimalConnection>;
}
```

### Module 4: `adapter/federation.rs` (80 lines)

**Extract lines 605-665**:
- `get_recent_federation_events()`
- `verify_federation_state_consistency()`
- `mark_federation_node_down()`
- `discover_providers_across_federation()`
- `emit_federation_event()`

**New struct**:
```rust
pub struct FederationCoordinator {
    events: Arc<RwLock<Vec<FederationEvent>>>,
    nodes: Arc<RwLock<HashMap<String, NodeState>>>,
}

impl FederationCoordinator {
    pub async fn get_events(&self, since: DateTime<Utc>) -> Result<Vec<FederationEvent>>;
    pub async fn verify_consistency(&self) -> Result<bool>;
    pub async fn mark_node_down(&self, node_id: &str) -> Result<()>;
    pub async fn discover_federated(&self, cap_type: &str) -> Result<Vec<String>>;
    pub async fn emit_event(&self, event: FederationEvent) -> Result<()>;
}
```

### Module 5: `adapter/cache.rs` (150 lines)

**Extract caching logic from lines 666-880**:
- Response caching
- Capability cache management
- Cache invalidation
- TTL handling

**New struct**:
```rust
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
}

impl ResponseCache {
    pub async fn get(&self, key: &str) -> Option<CapabilityResponse>;
    pub async fn set(&self, key: String, response: CapabilityResponse);
    pub async fn invalidate(&self, key: &str);
    pub async fn cleanup_expired(&self);
}
```

### Module 6: `adapter/metrics.rs` (130 lines)

**Extract QoS and metrics from lines 880-1010**:
- QoS metrics collection
- Performance tracking
- Latency measurement
- Health scoring

**New struct**:
```rust
pub struct QoSMetricsCollector {
    metrics: Arc<RwLock<HashMap<String, QoSMetrics>>>,
}

impl QoSMetricsCollector {
    pub async fn record_latency(&self, primal: &str, latency: Duration);
    pub async fn update_availability(&self, primal: &str, available: bool);
    pub async fn get_metrics(&self, primal: &str) -> Option<QoSMetrics>;
    pub async fn calculate_score(&self, primal: &str) -> f64;
}
```

### Module 7: `adapter/mod.rs` (200 lines)

**Orchestration layer**:
```rust
pub struct UniversalCapabilityAdapter {
    discovery: Arc<CapabilityDiscovery>,
    query: Arc<CapabilityQuery>,
    connections: Arc<ConnectionManager>,
    federation: Arc<FederationCoordinator>,
    cache: Arc<ResponseCache>,
    metrics: Arc<QoSMetricsCollector>,
}

impl UniversalCapabilityAdapter {
    pub fn new(config: DiscoveryConfig) -> Self {
        // Create all sub-components
        let discovery = Arc::new(CapabilityDiscovery::new(...));
        let query = Arc::new(CapabilityQuery::new(...));
        // ... etc
        
        Self { discovery, query, connections, federation, cache, metrics }
    }
    
    // Delegate to sub-components
    pub async fn discover_primal_capabilities(&self, name: &str) -> Result<Vec<Capability>> {
        self.discovery.discover_primal(name).await
    }
    
    pub async fn find_capability_providers(&self, cap_type: &str) -> Vec<String> {
        self.discovery.find_providers(cap_type).await
    }
    
    // ... delegate all public methods to appropriate sub-components
}
```

### Module 8: `adapter.rs` (Backward Compatibility, 50 lines)

Keep original file as re-export facade:
```rust
//! Universal Capability Adapter
//! 
//! This module provides backward compatibility.
//! All implementation is now in focused sub-modules.

mod adapter;

pub use adapter::*;
pub use adapter::discovery::*;
pub use adapter::connection::*;
// ... etc
```

---

## 📋 Refactoring Steps

### Phase 1: Preparation (30 mins)
1. ✅ Create `adapter/` directory
2. [ ] Create empty module files with structs
3. [ ] Add mod declarations to lib.rs
4. [ ] Run `cargo build` to verify structure

### Phase 2: Extract Discovery (45 mins)
1. [ ] Copy lines 52-290 to `discovery.rs`
2. [ ] Create `CapabilityDiscovery` struct
3. [ ] Update imports and internal references
4. [ ] Update `adapter/mod.rs` to use discovery
5. [ ] Run tests: `cargo test capabilities::adapter`

### Phase 3: Extract Capability Query (45 mins)
1. [ ] Copy lines 291-480 to `capability_query.rs`
2. [ ] Create `CapabilityQuery` struct
3. [ ] Update imports and references
4. [ ] Wire into mod.rs
5. [ ] Run tests

### Phase 4: Extract Connection Management (30 mins)
1. [ ] Copy lines 498-595 to `connection.rs`
2. [ ] Create `ConnectionManager` struct
3. [ ] Update references
4. [ ] Wire into mod.rs
5. [ ] Run tests

### Phase 5: Extract Federation (20 mins)
1. [ ] Copy lines 605-665 to `federation.rs`
2. [ ] Create `FederationCoordinator` struct
3. [ ] Wire into mod.rs
4. [ ] Run tests

### Phase 6: Extract Cache & Metrics (1 hour)
1. [ ] Extract caching to `cache.rs`
2. [ ] Extract metrics to `metrics.rs`
3. [ ] Wire both into mod.rs
4. [ ] Run tests

### Phase 7: Create Orchestration Layer (1 hour)
1. [ ] Create final `adapter/mod.rs`
2. [ ] Wire all sub-components together
3. [ ] Implement delegation methods
4. [ ] Remove old adapter.rs content
5. [ ] Update imports across codebase
6. [ ] Run full test suite

### Phase 8: Polish & Document (30 mins)
1. [ ] Add module-level documentation
2. [ ] Document architectural decisions
3. [ ] Update capability crate docs
4. [ ] Run clippy and fmt
5. [ ] Final test run

---

## ✅ Success Criteria

- [ ] All 501 tests still pass
- [ ] No public API changes (backward compatible)
- [ ] Each module < 250 lines
- [ ] Main orchestration < 200 lines
- [ ] Clear separation of concerns
- [ ] Improved code navigation
- [ ] Maintained performance
- [ ] Zero new clippy warnings

---

## 📊 Expected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Main file size** | 1080 lines | 200 lines | 81% reduction |
| **Largest module** | 1080 lines | 250 lines | 77% smaller |
| **Module count** | 1 | 7 | Better organization |
| **Avg module size** | 1080 lines | 165 lines | Manageable |
| **Cohesion** | Low | High | Clear boundaries |

---

## 💡 Benefits

1. **Maintainability**: Easier to find and modify specific functionality
2. **Testability**: Each module can be tested independently
3. **Readability**: Clear responsibility per module
4. **Extensibility**: Easy to add new capabilities without bloating main file
5. **Team Collaboration**: Multiple devs can work on different modules
6. **IDE Performance**: Smaller files = faster navigation and completion

---

## ⚠️ Risks & Mitigation

### Risk: Breaking existing code
**Mitigation**: Keep backward-compatible re-exports in original file

### Risk: Performance overhead from indirection
**Mitigation**: Use Arc for zero-cost cloning, inline critical paths

### Risk: Circular dependencies
**Mitigation**: Clear dependency hierarchy (mod.rs orchestrates, modules independent)

### Risk: Test failures
**Mitigation**: Test after each extraction phase, not at end

---

## 🚀 Estimated Time

**Total**: 5-6 hours of focused work

**Breakdown**:
- Preparation: 30 mins
- Extraction (6 modules): 3.5 hours
- Orchestration: 1 hour
- Testing & Polish: 1 hour

**Could be split across multiple sessions**

---

## 📝 Notes

- This is **smart refactoring**, not just line-count splitting
- Each module has clear, cohesive responsibility
- Architecture improves, not just metrics
- Maintains all existing functionality
- Sets foundation for future enhancements

**Ready to execute!** 🎵

