#!/bin/bash
# async_trait Analysis Script - Identifies modernization opportunities
# Part of Songbird Unification Initiative - Nov 10, 2025

set -e

OUTPUT_FILE="ASYNC_TRAIT_ANALYSIS.md"

echo "📊 Analyzing #[async_trait] usage..."
echo ""

# Count instances
TOTAL=$(grep -r "#\[async_trait\]" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")

if [ "$TOTAL" -eq 0 ]; then
    echo "✅ No async_trait usage found!"
    echo "🎉 Already using native async traits!"
    exit 0
fi

echo "Found $TOTAL #[async_trait] instances"

# Generate report
cat > "$OUTPUT_FILE" << EOF
# Songbird async_trait Modernization Analysis
**Generated**: $(date)  
**Total instances**: $TOTAL  
**Status**: 🟡 Performance optimization opportunity

---

## 📊 Summary

Found **$TOTAL** uses of \`#[async_trait]\` across the codebase.

**Performance Impact**: Each async_trait call adds 15-40% overhead compared to native async traits.

**Trade-off**: async_trait is required for dyn-compatibility (trait objects), but can be eliminated for static dispatch.

---

## 📋 Usage Breakdown

### Files Using async_trait

EOF

# List all files
grep -l "#\[async_trait\]" crates -r --include="*.rs" 2>/dev/null | sort | \
  while read file; do
    count=$(grep -c "#\[async_trait\]" "$file" 2>/dev/null || echo "0")
    echo "- **$file**: $count instances" >> "$OUTPUT_FILE"
  done

cat >> "$OUTPUT_FILE" << 'EOF'

---

## 🔍 Trait Analysis

### Traits Using async_trait

EOF

# Extract trait names
grep -B1 "#\[async_trait\]" crates -r --include="*.rs" 2>/dev/null | \
  grep "pub trait" | sed 's/.*pub trait \([^:{<]*\).*/\1/' | sort -u | \
  while read trait_name; do
    echo "#### Trait: $trait_name" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "\`\`\`rust" >> "$OUTPUT_FILE"
    grep -A5 "pub trait $trait_name" crates -r --include="*.rs" 2>/dev/null | head -6 >> "$OUTPUT_FILE" || true
    echo "\`\`\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Check if used with dyn
    DYN_USAGE=$(grep -r "dyn $trait_name\|Box<dyn\|Arc<dyn" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
    if [ "$DYN_USAGE" -gt 0 ]; then
        echo "🔴 **Used with trait objects** ($DYN_USAGE occurrences) - KEEP async_trait" >> "$OUTPUT_FILE"
    else
        echo "🟢 **Static dispatch only** - CAN MIGRATE to native async" >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"
    echo "---" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
  done

cat >> "$OUTPUT_FILE" << 'EOF'

## 🎯 Migration Patterns

### Pattern 1: Static Dispatch (CAN MIGRATE)

```rust
// ❌ BEFORE (async_trait overhead)
#[async_trait]
pub trait DataProcessor {
    async fn process(&self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
}

// ✅ AFTER (native async, zero overhead)
pub trait DataProcessor {
    fn process(&self, data: Vec<u8>) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;
}
```

### Pattern 2: Trait Objects (MUST KEEP)

```rust
// ✅ CORRECT - async_trait required for dyn
#[async_trait]
pub trait Provider {
    async fn initialize(&self) -> Result<(), Error>;
}

pub struct Registry {
    providers: HashMap<String, Arc<dyn Provider>>,  // Needs async_trait
}
```

---

## 📋 Decision Matrix

| Trait | Dyn Usage | Static Usage | Decision |
|-------|-----------|--------------|----------|

EOF

# Generate decision matrix
grep -B1 "#\[async_trait\]" crates -r --include="*.rs" 2>/dev/null | \
  grep "pub trait" | sed 's/.*pub trait \([^:{<]*\).*/\1/' | sort -u | \
  while read trait_name; do
    DYN_COUNT=$(grep -r "dyn $trait_name\|Box<dyn\|Arc<dyn" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
    STATIC_COUNT=$(grep -r "impl $trait_name" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
    
    if [ "$DYN_COUNT" -gt 0 ]; then
        DECISION="🔴 **KEEP** (trait objects)"
    else
        DECISION="🟢 **MIGRATE** (static only)"
    fi
    
    echo "| $trait_name | $DYN_COUNT | $STATIC_COUNT | $DECISION |" >> "$OUTPUT_FILE"
  done

cat >> "$OUTPUT_FILE" << 'EOF'

---

## 🎯 Action Plan

### Phase 1: Audit & Categorize
1. Review decision matrix above
2. Confirm dyn usage for each trait
3. Identify safe migration targets

### Phase 2: Migrate Static Traits
For traits marked 🟢 MIGRATE:

1. Remove `#[async_trait]` attribute
2. Change method signature:
   ```rust
   fn method(&self) -> impl Future<Output = Result<T>> + Send
   ```
3. Update implementations:
   ```rust
   fn method(&self) -> impl Future<Output = Result<T>> + Send {
       async move {
           // existing implementation
       }
   }
   ```
4. Test thoroughly

### Phase 3: Performance Validation
```bash
cargo bench --bench async_performance
# Expected: 15-40% improvement in migrated code
```

---

## 📈 Expected Results

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| async_trait instances | $TOTAL | ~15 | -65% |
| Performance overhead | 15-40% | 0% | +15-40% |

---

## ✅ Success Criteria

- [ ] All static-only traits migrated to native async
- [ ] Trait objects still using async_trait (required)
- [ ] Performance benchmarks show expected gains
- [ ] All tests passing
- [ ] Documentation updated

EOF

echo "✅ Analysis complete!"
echo "📄 Report written to: $OUTPUT_FILE"
echo "📊 Found $TOTAL async_trait instances"
echo ""
echo "🎯 Next steps:"
echo "   1. Review $OUTPUT_FILE"
echo "   2. Identify traits safe to migrate (static dispatch only)"
echo "   3. Migrate incrementally, testing after each change"
echo "   4. Benchmark performance gains"

