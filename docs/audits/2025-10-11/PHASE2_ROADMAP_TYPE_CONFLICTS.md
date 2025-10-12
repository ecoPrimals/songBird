# Phase 2 Roadmap: Type Conflicts

**Status**: Ready to start  
**Estimated Time**: 6-10 hours  
**Goal**: Fix all type conflicts → 11/12 crates compiling

---

## 📊 Error Analysis

**Total Errors**: 51

**Error Breakdown:**
- E0560 (missing field): 4 errors - ServiceInfo field mismatches
- E0609 (no field): 2 errors - Field access issues  
- E0599 (no method): 2 errors - Missing methods
- E0432 (unresolved import): 2 errors - Import issues
- E0308 (type mismatch): 2 errors - Type conversions needed
- E0433 (unresolved): 1 error - Missing types
- E0412 (cannot find type): 1 error - Result type
- E0407 (not trait member): 1 error - health_check
- E0046 (missing impl): 1 error - Incomplete traits

---

## 🎯 Fix Strategy

### Priority 1: ServiceInfo Alignment (HIGH IMPACT)
**Issue**: Multiple ServiceInfo definitions with different fields

**Files affected**:
- `songbird_universal::ServiceInfo` (has: primal_type, endpoint, health)
- `songbird_discovery::ServiceInfo` (has: version, endpoints, health_status, last_seen)

**Solution**:
1. Create conversion helpers
2. Use proper ServiceInfo from traits
3. Fix field access patterns

**Estimated**: 2-3 hours

### Priority 2: Missing Trait Implementations
**Issue**: Incomplete ServiceDiscovery implementations

**Methods missing**:
- `watch()`
- `update_health()`
- `list_all()`
- `exists()`
- `is_registered()`
- `update_metadata()`
- `as_any()`

**Solution**: Add stub implementations or proper implementations

**Estimated**: 1-2 hours

### Priority 3: Import & Type Issues
**Issues**:
- Missing imports (UniversalCapabilityAdapter, DiscoveryConfig)
- Result type not in scope
- ServiceHealthStatus not found
- DiscoveryBackend missing as_str()

**Solution**: Add imports, fix type paths

**Estimated**: 1-2 hours

### Priority 4: SongbirdError Methods
**Issue**: Missing error constructor methods
- `discovery_error()`
- `validation_error()`
- etc.

**Solution**: Use existing error constructors or add new ones

**Estimated**: 1 hour

---

## 📝 Systematic Approach

### Step 1: Fix Imports (Quick Win)
- Add missing use statements
- Fix Result type scope
- Add ServiceHealthStatus imports

### Step 2: ServiceInfo Conversion Layer
- Create converter functions
- Handle field mapping
- Add default values for missing fields

### Step 3: Complete Trait Implementations
- Add missing methods
- Use reasonable defaults
- Test compilation

### Step 4: Error Handling
- Fix SongbirdError calls
- Use proper constructors
- Test error paths

### Step 5: Final Cleanup
- Fix remaining type mismatches
- Add any missing pieces
- Full compilation test

---

## ⏱️ Time Estimates

| Task | Time | Cumulative |
|------|------|------------|
| Import fixes | 0.5h | 0.5h |
| ServiceInfo conversion | 2-3h | 3.5h |
| Trait implementations | 1-2h | 5.5h |
| Error handling | 1h | 6.5h |
| Final cleanup | 1-2h | 8.5h |
| Buffer | 1.5h | 10h |

**Total**: 6-10 hours

---

## 🎯 Success Criteria

- [ ] All imports resolved
- [ ] ServiceInfo conversions working
- [ ] All traits fully implemented
- [ ] Zero compilation errors in discovery
- [ ] 11/12 crates compiling (92%)

---

## 📚 Key Files to Modify

1. **Import fixes** (multiple files)
2. **Conversion helpers** (new file or in lib.rs)
3. **Trait implementations** (factory.rs, enhanced_discovery.rs, etc.)
4. **Error handling** (multiple files)

---

**Status**: Roadmap complete, ready to execute  
**Next**: Start with Quick Wins (imports)

