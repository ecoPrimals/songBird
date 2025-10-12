# 🔧 Pedantic Cleanup Plan

**Created**: October 2, 2025  
**Status**: Documentation for future cleanup  
**Priority**: Medium (code works, but can be polished)

---

## 📊 Current Pedantic Warnings

### By Category

**Documentation (Most Common)**:
- ~30 missing `# Errors` sections
- ~2 missing `# Panics` sections  
- Missing examples in some places

**Code Quality**:
- ~10 casting warnings (precision loss)
- ~4 identical match arms
- ~3 field naming conventions

**MSRV**:
- ~1 MSRV compatibility warning (already fixed for errors)

---

## 🎯 Pedantic Warning Types

### 1. Missing `# Errors` Documentation (~30 instances)

**Impact**: Low (documentation only)  
**Effort**: 1-2 hours  
**Priority**: Low

**Locations**:
- `songbird-config/src/config/environment.rs:398`
- `songbird-config/src/config/paths.rs:68,162,167,204,221`
- Various discovery functions

**Fix Pattern**:
```rust
/// Function description
/// 
/// # Errors
/// Returns error if [specific condition]
pub fn some_function() -> Result<T> { ... }
```

### 2. Missing `# Panics` Documentation (~2 instances)

**Impact**: Low (documentation only)  
**Effort**: 15 minutes  
**Priority**: Low

**Locations**:
- `songbird-config/src/config/constants.rs:462`
- `songbird-discovery/src/discovery/backends/consul.rs:21`

**Fix Pattern**:
```rust
/// Function description
/// 
/// # Panics
/// Panics if [specific condition]
pub fn some_function() { ... }
```

### 3. Casting Warnings (~10 instances)

**Impact**: Medium (potential precision loss)  
**Effort**: 1-2 hours  
**Priority**: Medium

**Types**:
- `u64` → `usize` (32-bit truncation risk)
- `i64` → `f64` (precision loss)
- `f32` ↔ `usize` (truncation + sign loss)

**Fix Strategy**:
```rust
// Option A: Use checked conversion
let value = u64_value.try_into().map_err(|_| Error::Overflow)?;

// Option B: Add explicit bounds checking
let value = usize::try_from(u64_value).expect("value within bounds");

// Option C: Document precision loss is acceptable
#[allow(clippy::cast_precision_loss)]
let float_value = int_value as f64;
```

### 4. Identical Match Arms (~4 instances)

**Impact**: Low (code duplication)  
**Effort**: 30 minutes  
**Priority**: Low

**Fix Pattern**:
```rust
// Before:
match value {
    A => do_thing(),
    B => do_thing(),
    C => do_other(),
}

// After:
match value {
    A | B => do_thing(),
    C => do_other(),
}
```

### 5. Field Naming Conventions (~3 instances)

**Impact**: Very Low (style)  
**Effort**: 15 minutes  
**Priority**: Very Low

**Issues**:
- All fields with same postfix (e.g., `_endpoint`)
- All fields with same prefix (e.g., `max_`)
- All fields with same postfix (e.g., `_timeout_secs`)

**Fix**: Consider if rename improves clarity, or accept style

---

## 📋 Cleanup Strategy

### Phase 1: Critical (Now)
- [x] Fix MSRV issues (DONE - const fn)
- [x] Ensure all crates compile (DONE - 12/18)
- [ ] Result migration completion (in progress)

### Phase 2: Documentation (Later)
- [ ] Add `# Errors` sections (~30 functions)
- [ ] Add `# Panics` sections (~2 functions)
- [ ] Add examples where helpful

### Phase 3: Code Quality (Future)
- [ ] Review casting warnings
- [ ] Consolidate identical match arms
- [ ] Consider field naming improvements

---

## 🛠️ Quick Fixes Script

For adding `# Errors` sections in bulk:

```python
#!/usr/bin/env python3
"""Add # Errors sections to function docs"""
import re

def add_errors_section(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern: pub fn ... Result<...
    pattern = r'(    /// [^\n]+\n)(    pub (?:async )?fn [^{]+-> (?:Result|SongbirdResult)<[^>]+>)'
    
    def replacer(match):
        doc = match.group(1)
        func = match.group(2)
        if '# Errors' not in doc:
            return f"{doc}    ///\n    /// # Errors\n    /// Returns error if operation fails\n{func}"
        return match.group(0)
    
    new_content = re.sub(pattern, replacer, content)
    
    with open(file_path, 'w') as f:
        f.write(new_content)
```

---

## 🎯 Recommended Approach

### Now (During Migration)
**Focus on**: Getting all crates compiling  
**Skip**: Pedantic warnings (can be fixed later)  
**Reason**: Don't want to mix architectural changes with style fixes

### After Migration Complete
**Phase A** (1 hour): Add missing doc sections  
**Phase B** (1 hour): Fix casting issues  
**Phase C** (30 min): Consolidate match arms  
**Total**: ~3 hours cleanup work

### Or: Accept as Tech Debt
**Option**: Add `#[allow(clippy::pedantic)]` at crate level  
**Trade-off**: Cleaner build output vs. losing pedantic checks  
**Recommended**: Fix after migration

---

## 📈 Priority Matrix

| Issue | Impact | Effort | Priority |
|-------|--------|--------|----------|
| MSRV issues | High | Low | ✅ DONE |
| Casting precision | Medium | Medium | 🟡 Future |
| Missing `# Errors` | Low | Low | 🟢 Later |
| Missing `# Panics` | Low | Very Low | 🟢 Later |
| Match arms | Low | Very Low | 🟢 Later |
| Field naming | Very Low | Low | 🟢 Optional |

---

## 🎊 Current Status

**Pedantic Warnings**: ~50 total  
**Blocking**: 0 (all are warnings)  
**Fixed**: MSRV const fn issue ✅  
**Remaining**: Documentation + style improvements  

**Recommendation**: Complete migration first, then pedantic cleanup as separate polish phase.

---

## 🚀 Action Items

### Immediate
- [x] Document pedantic warnings ✅
- [x] Prioritize them ✅  
- [ ] Continue Result migration

### Next Session
- [ ] Complete Result migration (6 crates)
- [ ] Full workspace compiling
- [ ] Test suite passing

### Future Polish Session
- [ ] Add missing doc sections (1 hr)
- [ ] Fix casting warnings (1 hr)
- [ ] Consolidate match arms (30 min)

---

**Note**: Pedantic warnings don't block functionality. They're code quality improvements that can be addressed after the migration is complete.

**Status**: Documented for future work ✅  
**Priority**: Low-Medium  
**Blocking**: No
