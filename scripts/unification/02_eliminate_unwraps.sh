#!/bin/bash
# unwrap() Elimination Script - Finds and helps eliminate panic sources
# Part of Songbird Unification Initiative - Nov 10, 2025

set -e

OUTPUT_FILE="UNWRAP_ANALYSIS.txt"

echo "🔍 Analyzing unwrap() and expect() usage in production code..."
echo ""

# Find all unwrap() calls (excluding tests)
echo "Scanning for panic sources..."
grep -rn "\.unwrap()\|\.expect(" crates/songbird-*/src --include="*.rs" 2>/dev/null | \
  grep -v "/tests/" | grep -v "test.rs" | grep -v "#\[test\]" > "$OUTPUT_FILE" || true

TOTAL=$(wc -l < "$OUTPUT_FILE" || echo "0")

if [ "$TOTAL" -eq 0 ]; then
    echo "✅ No unwrap() or expect() calls found in production code!"
    echo "🎉 Production safety achieved!"
    exit 0
fi

echo "⚠️  Found $TOTAL panic sources in production code"
echo ""

# Generate report
cat > "UNWRAP_REPORT.md" << EOF
# Songbird unwrap() Elimination Report
**Generated**: $(date)  
**Total panic sources**: $TOTAL  
**Status**: 🔴 CRITICAL - Needs immediate attention

---

## 📊 Summary

Found **$TOTAL** instances of \`.unwrap()\` or \`.expect()\` in production code paths.

These are **production-blocking issues** that can cause crashes:
- \`.unwrap()\` panics if Option is None or Result is Err
- \`.expect("msg")\` panics with message if condition fails

---

## 📋 All Panic Sources

\`\`\`
EOF

cat "$OUTPUT_FILE" >> "UNWRAP_REPORT.md"

cat >> "UNWRAP_REPORT.md" << 'EOF'
```

---

## 🛠️ Recommended Patterns

### Pattern 1: Option::unwrap() → ?
```rust
// ❌ BEFORE (will panic)
let value = map.get(key).unwrap();

// ✅ AFTER (safe)
let value = map.get(key)
    .ok_or_else(|| SongbirdError::NotFound { 
        resource: format!("key: {}", key) 
    })?;
```

### Pattern 2: Result::unwrap() → map_err + ?
```rust
// ❌ BEFORE (will panic)
let config = load_config().unwrap();

// ✅ AFTER (safe)
let config = load_config()
    .map_err(|e| SongbirdError::Configuration { 
        message: format!("Failed to load config: {}", e) 
    })?;
```

### Pattern 3: Parse errors
```rust
// ❌ BEFORE (will panic)
let addr = "127.0.0.1:8080".parse().unwrap();

// ✅ AFTER (safe)
let addr = "127.0.0.1:8080".parse()
    .map_err(|e| SongbirdError::Network { 
        message: format!("Invalid address: {}", e) 
    })?;
```

---

## 📋 Top Files Requiring Attention

EOF

# Top 10 files with most unwraps
echo "### Files with Most Panic Sources" >> "UNWRAP_REPORT.md"
echo "" >> "UNWRAP_REPORT.md"
grep -r "\.unwrap()\|\.expect(" crates/songbird-*/src --include="*.rs" 2>/dev/null | \
  grep -v "/tests/" | cut -d: -f1 | sort | uniq -c | sort -rn | head -10 | \
  while read count file; do
    echo "- **$file**: $count instances" >> "UNWRAP_REPORT.md"
  done

cat >> "UNWRAP_REPORT.md" << 'EOF'

---

## 🎯 Action Plan

1. **Create SafeOps utility** (if not exists):
   ```bash
   # Add to crates/songbird-types/src/safe_ops.rs
   cargo new --lib crates/songbird-types/src/safe_ops.rs
   ```

2. **Fix high-priority files first** (files with most unwraps)

3. **Use sed for common patterns** (verify each change):
   ```bash
   # Example - use with caution!
   sed -i 's/\.unwrap()/\.expect("TODO: add error context")?/g' target_file.rs
   ```

4. **Manual review required** for complex cases

5. **Validate after each change**:
   ```bash
   cargo check --workspace
   cargo test --workspace
   ```

6. **Track progress**:
   ```bash
   ./scripts/unification/track_progress.sh
   ```

---

## ✅ Success Criteria

- [ ] Zero `.unwrap()` calls in production code
- [ ] Zero `.expect()` calls in production code  
- [ ] All tests passing
- [ ] CI check added to prevent future unwraps

EOF

echo "📄 Reports generated:"
echo "   - UNWRAP_ANALYSIS.txt (raw data)"
echo "   - UNWRAP_REPORT.md (formatted report)"
echo ""
echo "⚠️  CRITICAL: $TOTAL panic sources found"
echo ""
echo "🎯 Next steps:"
echo "   1. Review UNWRAP_REPORT.md"
echo "   2. Fix high-priority files (listed in report)"
echo "   3. Run ./scripts/unification/track_progress.sh to monitor"

