# ⚡ Testing Gaps - Immediate Action Items

**Status:** 🚨 URGENT - Start immediately  
**Goal:** Fix critical testing gaps blocking 90% coverage  
**Current:** 60% estimated coverage → **Target:** 90% coverage

---

## 🚨 WEEK 1 CRITICAL TASKS (Start Now)

### **1. Fix Failing Tests** ❌ → ✅
```bash
# CONFIRMED: 2 failing regression tests found
cargo test --test regression_testing

# Failing tests:
# - test_service_discovery_regression (Expected 3 legacy services, found 0)
# - test_observability_regression (Should have at least one service registered)
```

**Specific Issues:**
- **Service Discovery Regression**: Expected 3 legacy services but found 0
- **Observability Regression**: No services registered during test execution
- **Performance Regression**: Service discovery taking >1.4 seconds

### **2. Create Validation Test Suite** 📝
```bash
# Create test structure
mkdir -p tests/unit/validation
mkdir -p tests/integration/validation

# Priority: src/errors/validation.rs (365 lines, 0% coverage)
touch tests/unit/validation/port_validation_tests.rs
touch tests/unit/validation/url_validation_tests.rs  
touch tests/unit/validation/file_path_tests.rs
touch tests/unit/validation/cross_field_tests.rs
```

### **3. Remove Panic Risks** 🚨
```bash
# CONFIRMED: Multiple unwrap() calls found that could panic in production

# Critical locations:
# src/proxy.rs:319 - .unwrap() in proxy configuration
# src/http_server/mod.rs:258 - .unwrap() in query parameter serialization
# src/observability/dashboard.rs:153, 160 - .unwrap() in dashboard rendering

# Plus 70+ unwrap() calls in observability/metrics modules (test code)
```

**Immediate Actions:**
1. Replace `src/proxy.rs:319` unwrap() with proper error handling
2. Add error handling to `src/http_server/mod.rs:258` query parameter processing
3. Fix dashboard rendering unwrap() calls in `src/observability/dashboard.rs`

---

## 🎯 WEEK 2 HIGH-PRIORITY TASKS

### **4. API Module Test Suite** 🔌
```bash
# Priority: src/api/mod.rs (692 lines, <10% coverage)
mkdir -p tests/unit/api
mkdir -p tests/integration/api

# Test categories needed:
# - REST endpoint functionality (30 tests)
# - GraphQL operations (25 tests)  
# - Authentication integration (15 tests)
# - Error response handling (10 tests)
```

### **5. Security Integration Tests** 🔒
```bash
# Focus areas:
# - OAuth2/OIDC end-to-end flows
# - Encryption/decryption scenarios
# - Authentication failure handling
# - Security audit logging
```

---

## 📊 QUICK COVERAGE CHECK

### **Generate Coverage Report**
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate current coverage report
cargo tarpaulin --out Html --output-dir coverage/
open coverage/tarpaulin-report.html

# Check specific modules
cargo tarpaulin --packages songbird-orchestrator -- --test validation
```

### **Module Priority Matrix**
| Module | Lines | Current % | Risk | Action |
|--------|-------|-----------|------|--------|
| validation | 365 | 0% | 🚨 | Week 1 |
| api | 692 | 10% | 🚨 | Week 2 |
| registry | ~300 | 20% | ⚠️ | Week 3 |
| security | ~500 | 50% | ⚠️ | Week 2 |

---

## 🛠️ QUICK SETUP COMMANDS

### **Install Testing Dependencies**
```bash
cargo add --dev proptest mockall criterion wiremock tempfile
```

### **Create Test Directory Structure**
```bash
# Unit tests
mkdir -p tests/unit/{validation,api,registry,security,network}

# Integration tests  
mkdir -p tests/integration/{cross_module,production_scenarios,security}

# Performance tests
mkdir -p tests/performance/{load,stress,regression}
```

### **Basic Test Template**
```rust
// tests/unit/validation/mod.rs
#[cfg(test)]
mod validation_tests {
    use super::*;
    use songbird_orchestrator::errors::validation::*;

    #[test]
    fn test_port_validation_boundaries() {
        // Test port 0 (invalid)
        // Test port 65535 (valid max)
        // Test port 65536 (invalid)
    }

    #[test]  
    fn test_url_validation_malformed_input() {
        // Test malformed URLs
        // Test edge cases
        // Test security scenarios
    }
}
```

---

## 🚀 IMMEDIATE COMMANDS TO RUN

### **Step 1: Assessment**
```bash
# Check current test status - CONFIRMED 2 FAILING
cargo test --test regression_testing

# Run all integration tests
cargo test --test integration_test

# Generate coverage baseline
cargo tarpaulin --out Json --output-dir baseline/
```

### **Step 2: Setup**
```bash
# Install dependencies
cargo add --dev proptest mockall criterion

# Create test structure
mkdir -p tests/{unit,integration,performance}/{validation,api,security}

# Start with validation tests
touch tests/unit/validation/mod.rs
```

### **Step 3: Fix Critical Issues**
```bash
# Fix failing regression tests first
cargo test --test regression_testing -- --nocapture

# Address unwrap() panic risks
grep -r "\.unwrap()" src/proxy.rs src/http_server/ src/observability/dashboard.rs
```

---

## 📈 DAILY PROGRESS TRACKING

### **Day 1-2: Critical Fixes**
- [ ] Fix `test_service_discovery_regression` (Expected 3 legacy services, found 0)
- [ ] Fix `test_observability_regression` (No services registered)  
- [ ] Remove unwrap() panic in `src/proxy.rs:319`
- [ ] Fix HTTP server query parameter unwrap() in `src/http_server/mod.rs:258`

### **Day 3-4: Validation Tests** 
- [ ] Complete port validation tests (20 tests)
- [ ] Complete URL validation tests (15 tests)
- [ ] Start file path validation tests (10 tests)

### **Day 5-7: Validation Completion**
- [ ] Complete file path validation tests
- [ ] Add cross-field validation tests (15 tests)  
- [ ] Validate validation module >90% coverage

---

## ⚠️ SPECIFIC BLOCKERS IDENTIFIED

### **Immediate Blockers**
- **test_service_discovery_regression**: Service discovery not finding legacy services
- **test_observability_regression**: Service registration not working in test context
- **Performance Regression**: Service discovery >1.4s (should be <100ms)
- **Panic Risk**: `src/proxy.rs:319` unwrap() could crash in production

### **Code Quality Issues**
- **50+ unused imports/variables** generating warnings  
- **45 tests filtered out** - many tests not running
- **70+ unwrap() calls** in observability modules (test code, but still risky)

### **Escalation Triggers**
- **More than 1 day to fix failing regression tests**
- **Cannot achieve 70% coverage by end of Week 2**
- **Critical security gaps not addressed by Week 4**

---

## 🎯 SUCCESS CRITERIA

### **Week 1 Success** ✅
- **Zero failing tests** (fix 2 regression tests)
- **Validation module >60% coverage** (from 0%)
- **Panic risks removed** (fix 4+ unwrap() locations)
- **Test infrastructure setup complete**

### **Week 2 Success** ✅
- **API module >70% coverage** (from <10%)
- **Security integration tests added**
- **Overall coverage >70%** (from ~60%)
- **Solid foundation for Phase 2**

---

## 🔧 DEBUGGING COMMANDS FOR FAILING TESTS

### **Service Discovery Regression Debug**
```bash
# Debug why legacy services aren't found
RUST_BACKTRACE=1 cargo test test_service_discovery_regression -- --nocapture

# Check service registration logic
grep -r "legacy.*service" tests/regression_testing.rs src/
```

### **Observability Regression Debug**
```bash
# Debug service registration in observability
RUST_BACKTRACE=1 cargo test test_observability_regression -- --nocapture

# Check observability engine initialization
grep -r "register_service" src/observability/
```

### **Performance Regression Analysis**
```bash
# Run performance tests to establish baseline
cargo test --test regression_testing -- --nocapture | grep "Execution Time"

# Profile service discovery performance
cargo bench discovery
```

---

**🚨 START NOW: Begin with fixing the 2 failing regression tests. These are blocking CI/CD pipeline and indicate core functionality issues.**

**⏰ TIME CRITICAL: Week 1 tasks must be completed to enable Week 2-8 roadmap success.**

**🔍 VERIFIED ISSUES: All critical issues confirmed through code analysis - 2 failing tests, 4+ critical unwrap() locations, 0% validation coverage.** 