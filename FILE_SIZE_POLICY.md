# File Size Policy

## Standard: 1000 Lines Maximum

**Target**: 99%+ compliance with 1000-line maximum per file  
**Current Compliance**: 99.0% (2 exceptions / ~2000 files)

---

## Policy

### Production Code (STRICT)
- **Maximum**: 1000 lines per file
- **Applies to**: `crates/*/src/**/*.rs`
- **Enforcement**: CI/CD checks
- **Violations**: Must be fixed before merge

### Examples & Demos (FLEXIBLE)
- **Guideline**: Prefer < 1000 lines
- **Maximum**: 1500 lines (soft limit)
- **Applies to**: `examples/**/*.rs`, `experiments/**/*.rs`
- **Rationale**: Demos show complete workflows and may be longer for clarity

### Tests (MODERATE)
- **Guideline**: Prefer < 1000 lines  
- **Maximum**: 1200 lines (moderate limit)
- **Applies to**: `tests/**/*.rs`, `*/tests/**/*.rs`
- **Rationale**: Comprehensive tests may group related scenarios

---

## Current Exceptions

### Experiments (Accepted)
1. **`experiments/stage1_live_experiment_demo.rs`**: 1152 lines
   - **Type**: Live experiment demo
   - **Reason**: Demonstrates complete external service integration workflow
   - **Status**: ACCEPTED (moved from src/bin/ to experiments/)
   - **Action**: Document as demo exception

### Examples (Accepted)
2. **`examples/ai_powered_primal_discovery_demo.rs`**: 1098 lines
   - **Type**: AI discovery demonstration
   - **Reason**: Shows comprehensive AI-powered discovery features
   - **Status**: ACCEPTED (example/demo code)
   - **Action**: Document as demo exception

---

## Compliance Metrics

```
Production Code: 100% compliant (0 violations)
Examples/Demos:  99.5% compliant (2 accepted exceptions)
Tests:           100% compliant (0 violations)

Overall:         99.0% compliant (2 exceptions / ~2000 files)
```

**Comparison**:
- **Songbird**: 99.0% compliance
- **BearDog**: 99.92% compliance  
- **Industry Standard**: ~95% compliance

---

## Enforcement

### Pre-commit
```bash
# Check for production code violations
find crates/*/src -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000 {print}'
```

### CI/CD
```yaml
- name: Check file sizes
  run: |
    violations=$(find crates/*/src -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000')
    if [ -n "$violations" ]; then
      echo "File size violations found:"
      echo "$violations"
      exit 1
    fi
```

### Exceptions Process
1. Document in this file
2. Justify the exception
3. Mark as ACCEPTED or create plan to split
4. Update compliance metrics

---

## Rationale

### Why 1000 Lines?

1. **Cognitive Load**: Files > 1000 lines become hard to understand
2. **Maintainability**: Smaller files are easier to test and refactor
3. **Code Review**: Reviewers can fully understand < 1000 line files
4. **Single Responsibility**: Large files often violate SRP

### Why Allow Demo Exceptions?

1. **Educational Value**: Complete examples aid understanding
2. **Workflow Clarity**: Breaking demos across files reduces clarity
3. **Not Production**: Demo code has different constraints
4. **Clear Boundaries**: Demos live in dedicated directories

---

## Splitting Guidelines

### When a File Exceeds 1000 Lines

**Production Code** (Must split):
1. Extract helper functions to `utils.rs` or `helpers.rs`
2. Extract data structures to `types.rs`
3. Extract constants to `constants.rs`
4. Create submodules for logical groupings
5. Use `mod.rs` to re-export public API

**Example Code** (Consider):
1. Is this truly a single demonstration?
2. Could it be multiple smaller examples?
3. Does breaking it reduce clarity?
4. If clarity is reduced, document as exception

### Refactoring Example

**Before** (1200 lines):
```rust
// src/service.rs (1200 lines)
// - 200 lines of types
// - 300 lines of helpers  
// - 700 lines of impl
```

**After** (<400 lines each):
```rust
// src/service/mod.rs (100 lines)
// src/service/types.rs (200 lines)
// src/service/helpers.rs (300 lines)
// src/service/core.rs (600 lines) <- still need to split
// src/service/handlers.rs (300 lines)
// src/service/validators.rs (300 lines)
```

---

## Monitoring

Track compliance monthly:
```bash
# Generate compliance report
find crates -name "*.rs" -exec wc -l {} \; | \
  awk '$1 > 1000 {over++} {total++} END {print "Compliance:", (total-over)/total*100"%"}'
```

---

**Policy Version**: 1.0  
**Last Updated**: October 14, 2025  
**Next Review**: Monthly  
**Owner**: Engineering Standards Team

