# Error System Consolidation Strategy - November 10, 2025

## 🎯 Mission

Consolidate Songbird's fragmented error system into a unified, AI-first error handling system based on `SongbirdError` with rich context, automation hints, and graceful degradation.

## 📊 Current State Analysis

### Error Landscape Audit

**Total Error Types Found**: 27 error definitions
**Canonical Error**: `SongbirdError` in `songbird-types/src/errors.rs` (12 variants)
**Integration Rate**: ~15% (4 of 27 integrated with canonical error)
**Consolidation Opportunity**: **85%** (23 error types can be unified)

### Error Types Inventory

#### ✅ Canonical Error System
1. **`SongbirdError`** (`songbird-types/src/errors.rs`)
   - **Status**: ✅ Primary canonical error
   - **Variants**: 12 comprehensive variants
     - Configuration
     - Network  
     - Security
     - Service
     - Serialization
     - Runtime
     - Validation
     - Discovery
     - Registry
     - LoadBalancing
     - Protocol
     - Metrics
     - Event
   - **Features**:
     - Rich context fields
     - Helper construction methods
     - From implementations for common types
     - Serializable (Serde)
     - thiserror integration

#### 🔄 Integrated Errors (Have `From<T> for SongbirdError`)
2. **`CliError`** (`songbird-cli/src/errors.rs`) ✅
   - **Status**: Properly integrated
   - **Issue**: ⚠️ **DUPLICATE DEFINITIONS** (3 copies!)
     - `songbird-cli/src/errors.rs`
     - `songbird-cli/src/cli/core/errors.rs`
     - `songbird-cli/src/cli/core/cli.rs`
   - **Action**: Consolidate to single definition

3. **`BiomeOSError`** (`songbird-orchestrator/src/core/primal_integration.rs`) 🔄
   - **Status**: Partially integrated (From impl exists but incomplete)
   - **Action**: Complete integration

#### ❌ Non-Integrated Errors (Need Integration)

4. **`ApiError`** (compute_api.rs) ❌
   - **Location**: `songbird-orchestrator/src/server/compute_api.rs`
   - **Target Variant**: `SongbirdError::Service`
   - **Priority**: HIGH (API surface)

5. **`ApiError`** (execution_api.rs) ❌
   - **Location**: `songbird-orchestrator/src/server/execution_api.rs`
   - **Issue**: ⚠️ **DUPLICATE NAME** with compute_api
   - **Target Variant**: `SongbirdError::Service`
   - **Priority**: HIGH (API surface)

6. **`CoordinationError`** ❌
   - **Location**: `songbird-orchestrator/src/core/biome/byob_coordinator/integration.rs`
   - **Target Variant**: `SongbirdError::Service`
   - **Priority**: MEDIUM

7. **`ByobError`** ❌
   - **Location**: `songbird-orchestrator/src/core/biome/modules/types.rs`
   - **Target Variant**: `SongbirdError::Service`
   - **Priority**: MEDIUM

8. **`BulkheadError`** ❌
   - **Location**: `songbird-orchestrator/src/core/robustness/bulkhead.rs`
   - **Target Variant**: `SongbirdError::Runtime`
   - **Priority**: MEDIUM (reliability pattern)

9. **`RetryableError`** ❌
   - **Location**: `songbird-orchestrator/src/core/robustness/error_types.rs`
   - **Target Variant**: `SongbirdError::Runtime` or wrapper
   - **Priority**: MEDIUM (reliability pattern)

10. **`ExecutionError`** ❌
    - **Location**: `songbird-orchestrator/src/core/execution/client.rs`
    - **Target Variant**: `SongbirdError::Runtime`
    - **Priority**: HIGH (execution path)

11. **`SubstrateError`** ❌
    - **Location**: `songbird-orchestrator/src/core/substrate/os_substrate.rs`
    - **Target Variant**: `SongbirdError::Runtime`
    - **Priority**: LOW

12. **`SerializationError`** ❌
    - **Location**: `songbird-orchestrator/src/core/zero_copy.rs`
    - **Target Variant**: `SongbirdError::Serialization`
    - **Priority**: MEDIUM (zero-copy patterns)

13. **`MetricsError`** ❌
    - **Location**: `songbird-orchestrator/src/core/metrics/capability_adapters.rs`
    - **Target Variant**: `SongbirdError::Metrics` (already exists!)
    - **Priority**: LOW (observability)

14. **`UniversalAdapterError`** ❌
    - **Location**: `songbird-universal/src/unified_adapter.rs`
    - **Target Variant**: `SongbirdError::Service`
    - **Priority**: HIGH (universal adapter)

15. **`CapabilityError`** ❌
    - **Location**: `songbird-universal/src/capabilities/error.rs`
    - **Target Variant**: `SongbirdError::Service`
    - **Priority**: HIGH (capability system)

16. **`DiscoveryError`** ❌
    - **Location**: `songbird-universal/src/discovery.rs`
    - **Target Variant**: `SongbirdError::Discovery` (already exists!)
    - **Priority**: HIGH (discovery system)

17. **`RegistryError`** ❌
    - **Location**: `songbird-discovery/src/abstraction/registry.rs`
    - **Target Variant**: `SongbirdError::Registry` (already exists!)
    - **Priority**: HIGH (registry system)

#### 📝 Non-Error Enums (Named *Error but aren't errors)

18. **`AIErrorCategory`** 📝
    - **Location**: `songbird-orchestrator/src/core/api/ai_first_response.rs`
    - **Type**: Enum for AI error classification
    - **Action**: Keep as-is (not an actual error type)

19. **`ErrorSeverity`** 📝 (2 duplicates!)
    - **Locations**:
      - `songbird-orchestrator/src/core/traits/validation.rs`
      - `songbird-discovery/src/traits/validation.rs`
    - **Type**: Enum for error severity levels
    - **Action**: Consolidate to single definition in `songbird-types`

20. **`HookErrorHandling`** 📝 (2 duplicates!)
    - **Locations**:
      - `songbird-orchestrator/src/core/traits/hooks.rs`
      - `songbird-discovery/src/traits/hooks.rs`
    - **Type**: Enum for hook error handling strategies
    - **Action**: Consolidate to single definition in `songbird-types`

21. **`LogLevel`** 📝
    - **Location**: `songbird-discovery/src/traits/observability.rs`
    - **Type**: Enum for logging levels (includes Error variant)
    - **Action**: Keep as-is (not an error type)

## 🎯 Consolidation Strategy

### Phase 1: Foundation (Week 1, Days 1-2) ✅ IN PROGRESS

#### 1.1 Audit Complete ✅
- [x] Identify all error types
- [x] Map to canonical variants
- [x] Identify duplicates
- [x] Document integration status

#### 1.2 Enhance Canonical Error (Day 2)
- [ ] Add missing variants if needed
- [ ] Add AI-First metadata fields:
  - `request_id: Option<Uuid>`
  - `confidence_score: Option<f64>`
  - `suggested_actions: Vec<String>`
  - `automation_hint: Option<AutomationHint>`
- [ ] Add builder pattern for complex errors
- [ ] Add error chaining support

### Phase 2: Consolidate Duplicates (Week 1, Days 3-4)

#### 2.1 Consolidate `CliError` (Priority: CRITICAL)
**Issue**: 3 duplicate definitions
**Solution**:
1. Keep `songbird-cli/src/errors.rs` as canonical
2. Delete `songbird-cli/src/cli/core/errors.rs`
3. Delete `CliError` from `songbird-cli/src/cli/core/cli.rs`
4. Update all imports

#### 2.2 Consolidate `ErrorSeverity` (Priority: HIGH)
**Issue**: 2 duplicate definitions
**Solution**:
1. Create canonical definition in `songbird-types/src/types/severity.rs`
2. Export from `songbird-types`
3. Replace both usages with canonical import
4. Delete duplicates

#### 2.3 Consolidate `HookErrorHandling` (Priority: HIGH)
**Issue**: 2 duplicate definitions
**Solution**:
1. Create canonical definition in `songbird-types/src/types/hooks.rs`
2. Export from `songbird-types`
3. Replace both usages with canonical import
4. Delete duplicates

### Phase 3: High-Priority Integrations (Week 1, Days 5-7)

#### 3.1 API Errors Integration
**Files**:
- `songbird-orchestrator/src/server/compute_api.rs`
- `songbird-orchestrator/src/server/execution_api.rs`

**Pattern**:
```rust
// OLD
pub enum ApiError {
    NotFound(String),
    InvalidRequest(String),
}

// NEW - Add From impl
impl From<ApiError> for SongbirdError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound(msg) => Self::Service {
                service: "api".to_string(),
                message: format!("Not found: {}", msg),
                suggested_alternatives: vec![],
                recovery_actions: vec!["Check endpoint availability".to_string()],
            },
            ApiError::InvalidRequest(msg) => Self::Validation {
                message: format!("Invalid API request: {}", msg),
                field: None,
                suggestion: Some("Check API documentation".to_string()),
            },
        }
    }
}

// FUTURE - Replace with direct SongbirdError usage
return Err(SongbirdError::service("api", "Not found"));
```

#### 3.2 Discovery & Registry Integration
**Files**:
- `songbird-universal/src/discovery.rs` (`DiscoveryError`)
- `songbird-discovery/src/abstraction/registry.rs` (`RegistryError`)

**Target**: Use existing `SongbirdError::Discovery` and `SongbirdError::Registry`

**Pattern**:
```rust
// OLD
return Err(DiscoveryError::ServiceNotFound("primal".to_string()));

// NEW  
return Err(SongbirdError::Discovery {
    message: "Service not found: primal".to_string(),
    backend: Some("universal".to_string()),
    retry_strategy: Some("exponential_backoff".to_string()),
});
```

#### 3.3 Capability System Integration
**Files**:
- `songbird-universal/src/capabilities/error.rs` (`CapabilityError`)
- `songbird-universal/src/unified_adapter.rs` (`UniversalAdapterError`)

**Pattern**:
```rust
impl From<CapabilityError> for SongbirdError {
    fn from(error: CapabilityError) -> Self {
        match error {
            CapabilityError::NetworkError(msg) => Self::Network {
                message: format!("Capability network error: {}", msg),
                interface: None,
                suggestion: Some("Check primal connectivity".to_string()),
            },
            CapabilityError::PrimalNotFound(name) => Self::Discovery {
                message: format!("Primal not found: {}", name),
                backend: Some("capability_system".to_string()),
                retry_strategy: Some("check_registry".to_string()),
            },
            // ...
        }
    }
}
```

### Phase 4: Medium-Priority Integrations (Week 2, Days 1-3)

#### 4.1 Robustness Patterns
- `BulkheadError` → `SongbirdError::Runtime`
- `RetryableError` → `SongbirdError::Runtime` (may need wrapper)
- `ExecutionError` → `SongbirdError::Runtime`

#### 4.2 BiomeOS Integration
- `BiomeOSError` → Complete integration
- `CoordinationError` → `SongbirdError::Service`
- `ByobError` → `SongbirdError::Service`

#### 4.3 Serialization
- `SerializationError` → `SongbirdError::Serialization`

### Phase 5: Low-Priority & Cleanup (Week 2, Days 4-5)

#### 5.1 Remaining Integrations
- `SubstrateError` → `SongbirdError::Runtime`
- `MetricsError` → `SongbirdError::Metrics`

#### 5.2 Deprecation & Removal
1. Add `#[deprecated]` attributes to old error types
2. Update all usage sites
3. Remove deprecated types after migration

## 🏗️ AI-First Error Enhancement

### Target Structure (Based on Ecosystem Standards)

```rust
/// Enhanced canonical error with AI-first metadata
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum SongbirdError {
    // ... existing variants ...
    
    // Common metadata fields to add to each variant:
    // - request_id: Option<Uuid>
    // - automation_hint: Option<AutomationHint>
    // - confidence_score: Option<f64>
}

/// Automation hint for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationHint {
    /// Retry with exponential backoff
    RetryExponential { max_attempts: u32, base_delay_ms: u64 },
    /// Retry with fixed interval
    RetryFixed { max_attempts: u32, interval_ms: u64 },
    /// Fallback to alternative service
    FallbackService { alternatives: Vec<String> },
    /// Escalate to human intervention
    EscalateHuman { urgency: Urgency },
    /// Safe to ignore (non-critical)
    Ignore,
    /// Circuit breaker open - stop retrying
    CircuitOpen { retry_after_secs: u64 },
}

/// Error urgency level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    Critical,
    High,
    Medium,
    Low,
}
```

### Integration with AIFirstResponse

From `UNIFIED_ERROR_HANDLING_SPECIFICATION.md`, all API responses should use:

```rust
pub struct AIFirstResponse<T> {
    pub success: bool,
    pub data: T,
    pub error: Option<AIFirstError>,
    pub request_id: Uuid,
    pub processing_time_ms: u64,
    pub ai_metadata: AIResponseMetadata,
    pub confidence_score: f64,
    pub suggested_actions: Vec<SuggestedAction>,
}
```

**Action**: Create `From<SongbirdError> for AIFirstError` implementation

## 📋 Migration Checklist

### Week 1
- [x] Day 1-2: Complete audit ✅
- [ ] Day 2: Enhance canonical error with AI-first fields
- [ ] Day 3: Consolidate `CliError` duplicates
- [ ] Day 4: Consolidate `ErrorSeverity` and `HookErrorHandling`
- [ ] Day 5: Integrate API errors (compute_api, execution_api)
- [ ] Day 6: Integrate discovery & registry errors
- [ ] Day 7: Integrate capability system errors

### Week 2
- [ ] Day 1-2: Integrate robustness pattern errors
- [ ] Day 3: Integrate BiomeOS errors
- [ ] Day 4: Integrate remaining errors (substrate, metrics, serialization)
- [ ] Day 5: Add deprecation warnings and create migration guide

## 🎯 Success Metrics

### Quantitative
- **Error Types**: 27 → 1 canonical (96% reduction)
- **Duplicate Definitions**: 7 duplicates → 0
- **Integration Rate**: 15% → 100%
- **From Implementations**: 4 → 17+

### Qualitative
- ✅ Single source of truth for all errors
- ✅ AI-first metadata on all errors
- ✅ Consistent error handling patterns
- ✅ Rich context and automation hints
- ✅ Backward compatible migration path

## 🔗 Related Documents

- [UNIFIED_ERROR_HANDLING_SPECIFICATION.md](./specs/UNIFIED_ERROR_HANDLING_SPECIFICATION.md)
- [CONFIG_MIGRATION_COMPLETE_NOV_10_2025.md](./CONFIG_MIGRATION_COMPLETE_NOV_10_2025.md)
- [UNIFICATION_AUDIT_REPORT_NOV_10_2025.md](./UNIFICATION_AUDIT_REPORT_NOV_10_2025.md)
- [TECHNICAL_DEBT_CLEANUP_PLAN_NOV_10.md](./TECHNICAL_DEBT_CLEANUP_PLAN_NOV_10.md)

## 📊 Impact Analysis

### Benefits
1. **Unified Error Handling**: Single consistent error system
2. **AI-First Ready**: Automation hints and context for AI agents
3. **Better DX**: Consistent error creation and handling
4. **Type Safety**: Compile-time error handling guarantees
5. **Observability**: Rich context for debugging and monitoring

### Risks & Mitigation
| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes | HIGH | Use deprecated aliases during transition |
| Large-scale refactoring | MEDIUM | Phased approach over 2 weeks |
| Incomplete integration | LOW | Comprehensive audit and tracking |
| Type conversion overhead | LOW | Zero-cost From implementations |

## 🚀 Next Actions

1. ✅ **Complete audit** (DONE)
2. **Enhance canonical error** with AI-first fields (NEXT)
3. **Consolidate duplicates** (CliError, ErrorSeverity, HookErrorHandling)
4. **Integrate high-priority errors** (API, discovery, capabilities)
5. **Create migration examples** and patterns
6. **Add deprecation warnings** to old error types

---

**Status**: 🚧 IN PROGRESS - Audit Complete, Ready for Implementation  
**Lead**: AI Assistant (Claude Sonnet 4.5)  
**Date**: November 10, 2025  
**Next Milestone**: Enhance canonical error with AI-first metadata

