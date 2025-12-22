# App Module Refactoring Plan

**Current State**: `mod.rs` is 1434 lines (43% over 1000 line limit)

**Goal**: Intelligent refactoring into cohesive, maintainable modules

## Current Structure

```
mod.rs (1434 lines)
├── Imports & utilities (lines 1-122)
├── SongbirdOrchestrator impl (lines 123-1381) ← MAIN BULK
├── Status & Health structs (lines 1382-1402)
└── Startup functions (lines 1403-1434)
```

## Target Structure

```
app/
├── mod.rs (< 100 lines) - Module aggregation
├── core.rs - SongbirdOrchestrator struct definition
├── orchestrator_impl.rs - Main implementation logic
├── network.rs ✅ DONE - Network utilities
├── health.rs ✅ DONE - Health & status
├── startup.rs ✅ DONE - Lifecycle management
├── federation.rs - Federation initialization
├── discovery.rs - Discovery setup
└── http_server.rs (already exists)
```

## Migration Strategy

### Phase 1: Extract Utilities ✅ DONE
- [x] network.rs - Network detection & parsing
- [x] health.rs - Health checks & status types
- [x] startup.rs - Main entry points

### Phase 2: Split Core (In Progress)
- [ ] core.rs - Struct definition + simple getters
- [ ] orchestrator_impl.rs - Complex initialization & lifecycle

### Phase 3: Domain Separation
- [ ] federation.rs - Federation-specific logic
- [ ] discovery.rs - Discovery-specific logic

### Phase 4: Clean Module Aggregator
- [ ] mod.rs - Minimal re-exports and module declarations

## Implementation Notes

**Principles**:
- Keep related functionality together
- Maintain clear module boundaries
- Preserve all existing APIs (no breaking changes)
- Add comprehensive documentation
- Include tests in each new module

**Dependencies**:
- network.rs has no internal dependencies
- health.rs depends on core.rs
- startup.rs depends on core.rs
- All depend on external crates (songbird_types, etc.)

## Current Status

**Completed**: 3 of 4 phases
**Lines Reduced**: TBD (need to update mod.rs)
**Compilation**: Must verify after each step
