# Smart Refactoring Plan: ipc/handlers.rs (1,229 lines)

## 📊 Analysis

**Current Structure**:
- Single file: 1,229 lines
- 11 JSON-RPC handler methods
- 3 clear logical domains

**Handler Groups**:

1. **Service Registry** (4 handlers, ~350 lines)
   - `register_service` 
   - `discover_by_capability`
   - `get_service_health`
   - `health_check`

2. **P2P Discovery** (3 handlers, ~350 lines)
   - `discover_by_family`
   - `create_genetic_tunnel`
   - `announce_capabilities`

3. **Graph Intelligence** (4 handlers, ~400 lines)
   - `validate_graph`
   - `check_availability`
   - `suggest_alternatives`
   - `validate_coordination_pattern`

## 🎯 Refactoring Strategy

### Domain-Driven Module Split:

```
crates/songbird-orchestrator/src/ipc/handlers/
├── mod.rs (~150 lines)
│   ├── IpcHandlers struct
│   ├── new() constructor
│   ├── Re-exports from sub-modules
│   └── Common imports
│
├── service_registry.rs (~400 lines)
│   ├── 4 service registry handlers
│   └── Service registry types/helpers
│
├── p2p_discovery.rs (~400 lines)
│   ├── 3 P2P discovery handlers
│   └── Discovery types/helpers
│
└── graph_intelligence.rs (~400 lines)
    ├── 4 graph intelligence handlers
    └── Graph types/helpers
```

### Backward Compatibility:

The `IpcHandlers` struct will remain in `mod.rs` with all methods,
but each method will delegate to the appropriate module:

```rust
impl IpcHandlers {
    // Service registry handlers
    pub async fn register_service_json(...) {
        service_registry::register_service(self, params).await
    }
    
    // P2P discovery handlers
    pub async fn discover_by_family_json(...) {
        p2p_discovery::discover_by_family(self, params).await
    }
    
    // Graph intelligence handlers
    pub async fn validate_graph_json(...) {
        graph_intelligence::validate_graph(self, params).await
    }
}
```

## ✅ Benefits

1. **File size compliance**: All files under 500 lines
2. **Logical cohesion**: Each module focuses on one domain
3. **Maintainability**: Easier to find and update specific handlers
4. **Testability**: Can test each domain independently
5. **No breaking changes**: Public API stays the same

## 📝 Implementation Steps

1. Create `handlers/` directory
2. Create `mod.rs` with `IpcHandlers` struct
3. Create `service_registry.rs` with 4 handlers
4. Create `p2p_discovery.rs` with 3 handlers
5. Create `graph_intelligence.rs` with 4 handlers
6. Update `handlers.rs` → `handlers/mod.rs`
7. Verify compilation
8. Run tests

**Estimated time**: 2-3 hours

