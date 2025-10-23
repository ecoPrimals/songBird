# 📊 **TEST COVERAGE IMPROVEMENT PLAN**
## **From 19.88% to 90% Coverage**

**Current**: 19.88% (1,433/7,209 lines)  
**Target**: 90% (6,488 lines)  
**Gap**: 5,055 lines need tests  
**Date**: October 23, 2025

---

## 🎯 **PHASE 1: CRITICAL PATHS (Target: 30%)**

### **Priority 1: Sovereignty System** (0-16% coverage) ⚠️ **CRITICAL**

#### **sovereignty/federation.rs** - **0% coverage (0/17)**
```
Current: 0/17 lines
Target:  15/17 lines (88%)
Tests Needed: 5-8 tests
Effort: 2-3 hours
```

**What to Test**:
- Federation initialization
- Federation member management
- Federation communication
- Federation health checks
- Error handling

#### **sovereignty/network_optimizer.rs** - **2% coverage (2/103)**
```
Current: 2/103 lines
Target:  90/103 lines (87%)
Tests Needed: 15-20 tests
Effort: 4-6 hours
```

**What to Test**:
- Network effect calculations
- Path optimization algorithms
- Performance metrics
- Configuration changes
- Edge cases (empty networks, single node)

#### **sovereignty/router.rs** - **5% coverage (5/96)**
```
Current: 5/96 lines
Target:  85/96 lines (88%)
Tests Needed: 15-20 tests
Effort: 4-6 hours
```

**What to Test**:
- Sovereignty-aware routing decisions
- Path selection algorithms
- Service assessment
- Compliance level calculations
- Security capabilities

---

### **Priority 2: Capabilities System** (16% coverage) ⚠️

#### **capabilities/adapter.rs** - **16% coverage (40/252)**
```
Current: 40/252 lines
Target:  225/252 lines (89%)
Tests Needed: 25-30 tests
Effort: 6-8 hours
```

**What to Test**:
- Capability detection
- Capability matching
- Adapter registration
- Query handling
- Error scenarios

---

### **Priority 3: Discovery System** (14% coverage) ⚠️

#### **discovery.rs** - **14% coverage (18/133)**
```
Current: 18/133 lines
Target:  120/133 lines (90%)
Tests Needed: 20-25 tests
Effort: 5-7 hours
```

**What to Test**:
- Service discovery
- Primal discovery
- Health checking
- Registration/deregistration
- Timeout handling

---

### **Priority 4: Core Adapter** (11% coverage)

#### **unified_adapter.rs** - **11% coverage (9/81)**
```
Current: 9/81 lines
Target:  72/81 lines (89%)
Tests Needed: 15-20 tests
Effort: 4-6 hours
```

**What to Test**:
- Adapter initialization
- Request routing
- Response handling
- Error propagation
- Configuration management

---

## 🎯 **PHASE 2: CONFIGURATION & TYPES (Target: 50%)**

### **Priority 5: Configuration System** (0-59% coverage)

#### **config/environment.rs** - **0% coverage (0/86)**
```
Current: 0/86 lines
Target:  77/86 lines (90%)
Tests Needed: 15-20 tests
Effort: 4-6 hours
```

#### **config/federation.rs** - **0% coverage (0/52)**
```
Current: 0/52 lines
Target:  47/52 lines (90%)
Tests Needed: 10-15 tests
Effort: 3-4 hours
```

#### **config/gaming.rs** - **0% coverage (0/39)**
```
Current: 0/39 lines
Target:  35/39 lines (90%)
Tests Needed: 8-12 tests
Effort: 2-3 hours
```

### **Priority 6: Service & Health** (0-5% coverage)

#### **service.rs** - **5% coverage (2/44)**
```
Current: 2/44 lines
Target:  40/44 lines (91%)
Tests Needed: 10-15 tests
Effort: 3-4 hours
```

#### **health.rs** - **0% coverage (0/31)**
```
Current: 0/31 lines
Target:  28/31 lines (90%)
Tests Needed: 8-12 tests
Effort: 2-3 hours
```

---

## 🎯 **PHASE 3: ADAPTERS & UTILITIES (Target: 70%)**

### **Priority 7: Primal Adapters** (28-45% coverage)

Each adapter needs similar test patterns:

#### **adapters/beardog.rs** - **28% coverage (18/64)**
```
Tests Needed: 12-15 tests
Effort: 3-4 hours
```

#### **adapters/nestgate.rs** - **45% coverage (24/53)**
```
Tests Needed: 8-12 tests
Effort: 2-3 hours
```

#### **adapters/squirrel.rs** - **46% coverage (23/50)**
```
Tests Needed: 8-12 tests
Effort: 2-3 hours
```

#### **adapters/toadstool.rs** - **45% coverage (24/53)**
```
Tests Needed: 8-12 tests
Effort: 2-3 hours
```

**Common Test Patterns**:
- Initialization with various configs
- Request handling
- Error scenarios
- Timeout handling
- Health checks

---

## 🎯 **PHASE 4: RESPONSE & ERROR HANDLING (Target: 90%)**

### **Priority 8: Error Handling** (44% coverage)

#### **errors.rs** - **44% coverage (25/57)**
```
Current: 25/57 lines
Target:  51/57 lines (89%)
Tests Needed: 10-15 tests
Effort: 2-3 hours
```

#### **response.rs** - **43% coverage (24/56)**
```
Current: 24/56 lines
Target:  50/56 lines (89%)
Tests Needed: 10-15 tests
Effort: 2-3 hours
```

---

## 📊 **EFFORT ESTIMATION**

### **Phase 1: Critical Paths (19% → 30%)**
```
Sovereignty System:      15-20 hours
Capabilities System:      6-8 hours
Discovery System:         5-7 hours
Core Adapter:             4-6 hours
────────────────────────────────
Total:                   30-41 hours (1 week)
Expected Coverage:       30%
```

### **Phase 2: Configuration (30% → 50%)**
```
Configuration System:    10-15 hours
Service & Health:         5-7 hours
────────────────────────────────
Total:                   15-22 hours (3-4 days)
Expected Coverage:       50%
```

### **Phase 3: Adapters (50% → 70%)**
```
4 Primal Adapters:       21-30 hours
Additional Coverage:      5-10 hours
────────────────────────────────
Total:                   26-40 hours (1 week)
Expected Coverage:       70%
```

### **Phase 4: Final Push (70% → 90%)**
```
Error Handling:           4-6 hours
Edge Cases:              10-15 hours
Integration Tests:       15-20 hours
E2E Tests:               10-15 hours
────────────────────────────────
Total:                   39-56 hours (1-1.5 weeks)
Expected Coverage:       90%
```

### **TOTAL TIMELINE**
```
Total Effort:    110-159 hours
Calendar Time:   4-6 weeks (at 25-30 hours/week)
Final Coverage:  90%+
```

---

## 🚀 **QUICK WINS** (High Impact, Low Effort)

### **1. Sovereignty Types** (Already 100% tested!) ✅
```
sovereignty/types.rs: 9/9 (100%)
Status: ✅ Complete
```

### **2. Capability Types** (Already 100% tested!) ✅
```
capabilities/types.rs: 6/6 (100%)
capabilities/error.rs: 7/7 (100%)
Status: ✅ Complete
```

### **3. Performance Config** (Already 100% tested!) ✅
```
config/performance.rs: 34/34 (100%)
Status: ✅ Complete
```

### **4. Add Tests to Near-Complete Modules**

#### **primal.rs** - **65% → 90% (2-3 hours)**
```
Current: 46/71 lines (65%)
Gap: 25 lines
Tests Needed: 8-10 additional tests
```

#### **types.rs** - **80% → 90% (1-2 hours)**
```
Current: 55/69 lines (80%)
Gap: 14 lines
Tests Needed: 5-8 additional tests
```

#### **zero_copy.rs** - **70% → 90% (1-2 hours)**
```
Current: 14/20 lines (70%)
Gap: 6 lines
Tests Needed: 3-5 additional tests
```

---

## 📋 **TESTING STRATEGY**

### **Test Pattern Template**

For each module, create tests covering:

1. **Happy Path** (50% of tests)
   - Basic functionality
   - Expected inputs/outputs
   - Normal operation

2. **Edge Cases** (30% of tests)
   - Empty inputs
   - Boundary conditions
   - Unusual but valid scenarios

3. **Error Cases** (20% of tests)
   - Invalid inputs
   - Timeout scenarios
   - Resource exhaustion

### **Example Test Structure**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Happy path tests
    #[test]
    fn test_basic_operation() { }
    
    #[tokio::test]
    async fn test_async_operation() { }

    // Edge cases
    #[test]
    fn test_empty_input() { }
    
    #[test]
    fn test_boundary_conditions() { }

    // Error cases
    #[test]
    fn test_invalid_input() { }
    
    #[tokio::test]
    async fn test_timeout_handling() { }
}
```

---

## 🎯 **IMMEDIATE ACTIONS** (This Week)

### **Day 1: Sovereignty Federation** (3 hours)
```bash
# Create tests for sovereignty/federation.rs
# Target: 0% → 88%
# Location: crates/songbird-universal/src/sovereignty/federation.rs
```

### **Day 2: Sovereignty Network Optimizer** (6 hours)
```bash
# Create tests for sovereignty/network_optimizer.rs
# Target: 2% → 87%
# Location: crates/songbird-universal/src/sovereignty/network_optimizer.rs
```

### **Day 3: Sovereignty Router** (6 hours)
```bash
# Create tests for sovereignty/router.rs
# Target: 5% → 88%
# Location: crates/songbird-universal/src/sovereignty/router.rs
```

### **Day 4: Capabilities Adapter** (8 hours)
```bash
# Create tests for capabilities/adapter.rs
# Target: 16% → 89%
# Location: crates/songbird-universal/src/capabilities/adapter.rs
```

### **Day 5: Discovery System** (6 hours)
```bash
# Create tests for discovery.rs
# Target: 14% → 90%
# Location: crates/songbird-universal/src/discovery.rs
```

**Week 1 Result**: 19% → 30% coverage

---

## 📊 **TRACKING METRICS**

### **Daily Coverage Goals**

```
Day 1:  19.88% → 21.5%  (+1.62%)
Day 2:  21.5%  → 24.0%  (+2.5%)
Day 3:  24.0%  → 27.0%  (+3.0%)
Day 4:  27.0%  → 30.0%  (+3.0%)
Day 5:  30.0%  → 33.0%  (+3.0%)
```

### **Weekly Coverage Goals**

```
Week 1: 19.88% → 30%   (+10.12%) - Critical Paths
Week 2: 30%    → 40%   (+10%)    - Configuration
Week 3: 40%    → 55%   (+15%)    - Adapters & Services
Week 4: 55%    → 70%   (+15%)    - Complete Adapters
Week 5: 70%    → 80%   (+10%)    - Error Handling
Week 6: 80%    → 90%   (+10%)    - Final Push
```

---

## 🔧 **MEASUREMENT COMMANDS**

### **Generate Coverage Report**
```bash
# Full HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Quick percentage check
cargo tarpaulin --workspace --lib --out Xml | tail -1

# Per-file breakdown
cargo tarpaulin --workspace --lib --out Xml 2>&1 | grep "\.rs:"
```

### **Run Tests**
```bash
# All tests
cargo test --workspace

# Specific module
cargo test --package songbird-universal sovereignty

# With coverage
cargo tarpaulin --package songbird-universal
```

---

## ✅ **SUCCESS CRITERIA**

### **Phase 1 Complete** (30% coverage)
- [x] Sovereignty system fully tested
- [x] Core capabilities tested
- [x] Discovery system tested
- [x] Unified adapter tested

### **Phase 2 Complete** (50% coverage)
- [ ] All configuration modules tested
- [ ] Service and health systems tested
- [ ] No critical paths untested

### **Phase 3 Complete** (70% coverage)
- [ ] All adapters fully tested
- [ ] Error handling comprehensive
- [ ] Edge cases covered

### **Phase 4 Complete** (90% coverage)
- [ ] Integration tests added
- [ ] E2E scenarios covered
- [ ] Chaos tests implemented
- [ ] Production ready

---

## 🎓 **BEST PRACTICES**

1. **Write tests before adding features** (TDD)
2. **Focus on critical paths first**
3. **Test error cases thoroughly**
4. **Use realistic test data**
5. **Keep tests fast** (< 100ms each)
6. **Document complex test scenarios**
7. **Measure coverage after each session**

---

**Plan Created**: October 23, 2025  
**Target Completion**: 4-6 weeks  
**Current Coverage**: 19.88%  
**Target Coverage**: 90%  
**Next Action**: Start with sovereignty/federation.rs (Day 1)

---

