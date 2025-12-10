# Mock Migration & Cleanup Plan - November 20, 2025
## Completing the Capability-Based Mock Migration

**Status**: 📊 **ASSESSMENT COMPLETE**  
**Current State**: Hybrid (Modern + Deprecated)  
**Target State**: 100% Capability-Based  
**Estimated Time**: 12-16 hours total  
**Priority**: P2 (Quality improvement, improves maintainability)

---

## 🎯 EXECUTIVE SUMMARY

### Current Situation
The mock system has been **partially modernized** but still contains deprecated primal-specific mocks for backward compatibility.

**Modern System** (✅ EXISTS):
- `MockCapabilityServer` - Generic capability-based testing
- `MockCapabilityEnvironment` - Multi-capability environments
- `CapabilityType` enum - Abstracted capability types
- Zero hardcoding approach

**Deprecated System** (⚠️ STILL PRESENT):
- `MockBearDog` - Hardcoded security primal
- `MockNestGate` - Hardcoded storage primal
- `MockToadStool` - Hardcoded compute primal
- `MockSquirrel` - Hardcoded AI primal
- `MockPrimalServer` trait - Legacy interface

### Migration Status
- ✅ **Modern infrastructure**: Complete
- ✅ **Migration guide**: Documented in mod.rs
- ⚠️ **Test migration**: Partially complete
- ❌ **Deprecated removal**: Not started

---

## 📊 DETAILED ANALYSIS

### Files Involved
| File | Type | Status | Action |
|------|------|--------|--------|
| `mocks/capability_mocks.rs` | Modern | ✅ Complete | Keep |
| `mocks/common.rs` | Shared | ✅ Used by both | Keep, update |
| `mocks/beardog.rs` | Deprecated | ⚠️ Legacy | Migrate away, remove |
| `mocks/nestgate.rs` | Deprecated | ⚠️ Legacy | Migrate away, remove |
| `mocks/toadstool.rs` | Deprecated | ⚠️ Legacy | Migrate away, remove |
| `mocks/squirrel.rs` | Deprecated | ⚠️ Legacy | Migrate away, remove |
| `mocks/mod.rs` | Module | 📝 Hybrid | Update after migration |

### Usage Analysis
```bash
# References to deprecated mocks
MockBearDog: ~15 usages
MockNestGate: ~12 usages
MockToadStool: ~10 usages
MockSquirrel: ~8 usages
MockPrimalServer trait: ~45 usages
Total: ~90 migration points
```

### Affected Test Files
Estimated: 20-30 test files across the codebase

---

## 🚀 MIGRATION STRATEGY

### Phase 1: Audit & Inventory (2-3 hours)

#### Tasks
1. **Find all usages** of deprecated mocks
   ```bash
   grep -r "MockBearDog\|MockNestGate\|MockToadStool\|MockSquirrel\|MockPrimalServer" \
     --include="*.rs" crates/
   ```

2. **Categorize by complexity**
   - Simple: Direct mock instantiation
   - Medium: Mock with custom configuration
   - Complex: Mock with state management

3. **Create migration checklist**
   - List of all files to update
   - Complexity rating
   - Estimated time per file

4. **Identify blockers**
   - Features unique to deprecated mocks
   - Tests that require special handling

#### Deliverables
- [ ] Complete usage inventory
- [ ] Categorized migration list
- [ ] Identified any missing capabilities in modern system

### Phase 2: Test Migration (6-8 hours)

#### Migration Pattern

**Before (Deprecated)**:
```rust
use songbird_test_utils::mocks::MockBearDog;

#[tokio::test]
async fn test_security_integration() {
    let mut beardog = MockBearDog::new();
    beardog.start().await.unwrap();
    
    let endpoint = format!("http://localhost:{}", beardog.port());
    // Test code...
    
    beardog.shutdown().await;
}
```

**After (Modern)**:
```rust
use songbird_test_utils::mocks::{MockCapabilityServer, CapabilityType};

#[tokio::test]
async fn test_security_integration() {
    let mut security = MockCapabilityServer::new(CapabilityType::Security);
    security.start().await.unwrap();
    
    let endpoint = format!("http://localhost:{}", security.port());
    // Test code (unchanged)...
    
    security.shutdown().await;
}
```

#### Migration Steps (Per Test File)

1. **Update imports**
   ```rust
   // Remove
   use songbird_test_utils::mocks::MockBearDog;
   
   // Add
   use songbird_test_utils::mocks::{MockCapabilityServer, CapabilityType};
   ```

2. **Update instantiation**
   ```rust
   // Before
   let mut mock = MockBearDog::new();
   
   // After
   let mut mock = MockCapabilityServer::new(CapabilityType::Security);
   ```

3. **Update method calls** (if any differences)
   - Most methods should be identical
   - Check for primal-specific methods

4. **Run tests**
   - Verify each test still passes
   - Fix any differences

5. **Commit**
   - One commit per file or small group
   - Clear commit message

#### Priority Order
1. **High-traffic tests** (run frequently)
2. **Simple migrations** (build confidence)
3. **Complex tests** (learn patterns)
4. **Integration tests** (verify compatibility)

### Phase 3: Cleanup Deprecated Code (2-3 hours)

Once all tests migrated:

1. **Remove deprecated mock files**
   - `mocks/beardog.rs`
   - `mocks/nestgate.rs`
   - `mocks/toadstool.rs`
   - `mocks/squirrel.rs`

2. **Clean up common.rs**
   - Remove `MockPrimalServer` trait
   - Keep only shared utilities

3. **Update mod.rs**
   - Remove deprecated exports
   - Update documentation
   - Remove migration guide (no longer needed)

4. **Update tests**
   - Final verification
   - Check for any missed references

### Phase 4: Documentation & Prevention (1-2 hours)

1. **Update documentation**
   - Architecture docs
   - Testing guide
   - Example tests

2. **Add linting rules** (optional)
   - Deny use of removed types
   - CI checks

3. **Create examples**
   - Modern mock usage examples
   - Best practices

---

## 📋 DETAILED MIGRATION CHECKLIST

### Pre-Migration
- [ ] Backup current state
- [ ] Run full test suite (baseline)
- [ ] Create migration branch
- [ ] Document current test pass rate

### Migration Process

#### Inventory Phase
- [ ] Find all MockBearDog usages
- [ ] Find all MockNestGate usages
- [ ] Find all MockToadStool usages
- [ ] Find all MockSquirrel usages
- [ ] Find all MockPrimalServer usages
- [ ] Categorize by complexity
- [ ] Create detailed checklist

#### Test Migration Phase
- [ ] Migrate simple tests (10-15 files)
- [ ] Migrate medium complexity tests (5-10 files)
- [ ] Migrate complex tests (3-5 files)
- [ ] Verify all tests pass
- [ ] Update test documentation

#### Cleanup Phase
- [ ] Remove beardog.rs
- [ ] Remove nestgate.rs
- [ ] Remove toadstool.rs
- [ ] Remove squirrel.rs
- [ ] Clean up common.rs
- [ ] Update mod.rs exports
- [ ] Remove deprecated docs

#### Verification Phase
- [ ] Run full test suite
- [ ] Check for remaining references
- [ ] Verify no regressions
- [ ] Update architecture docs

---

## 🎯 MIGRATION EXAMPLES

### Example 1: Simple Test Migration

**Before**:
```rust
#[tokio::test]
async fn test_basic_security() {
    let mut beardog = MockBearDog::new();
    beardog.start().await.unwrap();
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/health", beardog.port()))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    beardog.shutdown().await;
}
```

**After**:
```rust
#[tokio::test]
async fn test_basic_security() {
    let mut security = MockCapabilityServer::new(CapabilityType::Security);
    security.start().await.unwrap();
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://localhost:{}/health", security.port()))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    security.shutdown().await;
}
```

**Changes**: 2 lines modified, identical behavior

### Example 2: Multi-Capability Test

**Before**:
```rust
#[tokio::test]
async fn test_multi_service() {
    let mut beardog = MockBearDog::new();
    let mut nestgate = MockNestGate::new();
    
    beardog.start().await.unwrap();
    nestgate.start().await.unwrap();
    
    // Test code...
    
    beardog.shutdown().await;
    nestgate.shutdown().await;
}
```

**After (Option 1: Individual mocks)**:
```rust
#[tokio::test]
async fn test_multi_service() {
    let mut security = MockCapabilityServer::new(CapabilityType::Security);
    let mut storage = MockCapabilityServer::new(CapabilityType::Storage);
    
    security.start().await.unwrap();
    storage.start().await.unwrap();
    
    // Test code (unchanged)...
    
    security.shutdown().await;
    storage.shutdown().await;
}
```

**After (Option 2: Environment builder)**:
```rust
#[tokio::test]
async fn test_multi_service() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_security()
        .with_storage()
        .build()
        .await
        .unwrap();
    
    // Test code (simpler!)...
    
    env.shutdown().await;
}
```

**Changes**: 4-6 lines modified, potentially simpler with environment builder

### Example 3: Custom Configuration

**Before**:
```rust
#[tokio::test]
async fn test_custom_config() {
    let mut beardog = MockBearDog::new();
    beardog.set_port(9000);
    beardog.set_response_delay(Duration::from_millis(100));
    beardog.start().await.unwrap();
    
    // Test code...
    
    beardog.shutdown().await;
}
```

**After**:
```rust
#[tokio::test]
async fn test_custom_config() {
    let mut security = MockCapabilityServer::new(CapabilityType::Security)
        .with_port(9000)
        .with_response_delay(Duration::from_millis(100));
    security.start().await.unwrap();
    
    // Test code (unchanged)...
    
    security.shutdown().await;
}
```

**Changes**: Builder pattern, slightly cleaner

---

## ⚠️ POTENTIAL ISSUES & SOLUTIONS

### Issue 1: Primal-Specific Methods
**Problem**: Old mocks have primal-specific methods  
**Solution**: 
- Check if modern mocks have equivalent
- Add to modern mocks if needed
- Or refactor test to not need it

### Issue 2: Different Default Ports
**Problem**: Old mocks had specific port ranges  
**Solution**:
- Modern mocks auto-assign ports
- Explicitly set port if test requires specific port
- Update test if port-agnostic

### Issue 3: State Management Differences
**Problem**: Different internal state handling  
**Solution**:
- Review state usage in test
- Ensure modern mock supports same state
- Refactor test if needed

### Issue 4: Performance Differences
**Problem**: New mocks might have different perf characteristics  
**Solution**:
- Profile if performance-critical
- Adjust timeouts if needed
- Generally should be similar

---

## 📈 EXPECTED BENEFITS

### Code Quality
- ✅ **Zero Hardcoding**: No primal names in tests
- ✅ **Better Abstraction**: Capability-based thinking
- ✅ **Easier Maintenance**: Single mock implementation
- ✅ **Clearer Intent**: Tests show capability needs
- ✅ **Less Duplication**: Reusable mock infrastructure

### Developer Experience
- ✅ **Simpler Tests**: Environment builder for complex scenarios
- ✅ **Better Docs**: Clear migration path
- ✅ **Easier Onboarding**: One mock system to learn
- ✅ **Faster Development**: Less boilerplate

### Technical Debt
- ✅ **Reduced Code**: ~1,500 lines removed (4 mock files)
- ✅ **Simplified**: One implementation vs five
- ✅ **Maintainable**: Changes in one place
- ✅ **Flexible**: Easy to add new capability types

---

## 🕐 TIMELINE & EFFORT

### Week 1: Audit & Simple Migrations
- **Day 1**: Complete audit and inventory (3 hours)
- **Day 2**: Migrate simple tests (4 hours)
- **Day 3**: Migrate simple tests continued (4 hours)
- **Day 4**: Begin medium complexity tests (4 hours)
- **Day 5**: Buffer/testing (2 hours)

**Week 1 Total**: 17 hours

### Week 2: Complex Migrations & Cleanup
- **Day 1**: Complete medium tests (4 hours)
- **Day 2**: Complex tests (4 hours)
- **Day 3**: Verification & fixes (3 hours)
- **Day 4**: Remove deprecated code (2 hours)
- **Day 5**: Documentation & polish (2 hours)

**Week 2 Total**: 15 hours

**Grand Total**: 32 hours over 2 weeks

---

## 🎯 SUCCESS CRITERIA

### Mandatory
- ✅ All tests passing
- ✅ Zero usages of deprecated mocks
- ✅ Deprecated files removed
- ✅ Documentation updated
- ✅ No functionality regressions

### Optional (Nice to Have)
- ✅ Improved test clarity
- ✅ Reduced test boilerplate
- ✅ Better test organization
- ✅ Performance maintained/improved

---

## 💡 BEST PRACTICES

### During Migration
1. **One file at a time** - Don't try to migrate everything at once
2. **Test immediately** - Run tests after each file
3. **Commit frequently** - Small, focused commits
4. **Document issues** - Note any problems encountered
5. **Ask questions** - Clarify unclear patterns

### After Migration
1. **Update docs** - Reflect new patterns
2. **Share knowledge** - Team training
3. **Monitor usage** - Ensure no new deprecated usage
4. **Continuous improvement** - Refine mock system

---

## 📚 REFERENCE

### Modern Mock Quick Reference

**Basic Usage**:
```rust
let mut mock = MockCapabilityServer::new(CapabilityType::Security);
mock.start().await?;
// ... test code ...
mock.shutdown().await;
```

**Environment Builder**:
```rust
let mut env = MockCapabilityEnvironment::builder()
    .with_security()
    .with_storage()
    .with_ai()
    .build()
    .await?;
// ... test code ...
env.shutdown().await;
```

**Custom Configuration**:
```rust
let mock = MockCapabilityServer::new(CapabilityType::Compute)
    .with_port(8080)
    .with_response_delay(Duration::from_millis(50));
```

### Capability Types
- `CapabilityType::Security` - Security services
- `CapabilityType::Storage` - Storage services
- `CapabilityType::Compute` - Compute services
- `CapabilityType::Ai` - AI services
- `CapabilityType::Custom(String)` - Extensible

---

## 🏁 CONCLUSION

### Current State
- Modern mock infrastructure: ✅ Complete
- Migration in progress: ⚠️ Partial
- Deprecated code: ⚠️ Still present

### Target State
- All tests use modern mocks: ✅
- Deprecated code removed: ✅
- Zero hardcoding: ✅
- Clean, maintainable: ✅

### Effort Required
- **Estimated Time**: 32 hours
- **Duration**: 2 weeks at 16 hours/week
- **Priority**: P2 (Quality improvement)
- **Risk**: Low (well-defined migration path)

### Recommendation
**Proceed with systematic migration**:
1. Week 1: Audit + Simple/Medium tests
2. Week 2: Complex tests + Cleanup

Benefits far outweigh the effort. This cleanup will:
- Eliminate hardcoding
- Simplify test maintenance
- Improve code quality
- Reduce technical debt

---

**Plan Created**: November 20, 2025  
**Estimated Completion**: 2 weeks (32 hours)  
**Priority**: P2 - Quality Improvement  
**Status**: 📋 READY FOR EXECUTION

**This migration will eliminate mock-related technical debt! 🚀**

