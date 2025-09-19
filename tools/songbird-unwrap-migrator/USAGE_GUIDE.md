# 🔄 Security Primal Enhanced Unwrap Migrator - Usage Guide

**Version**: 2.0.0  
**Status**: Production Ready  
**Context-Aware**: ✅ Yes

---

## 🎯 Overview

The Security Primal Enhanced Unwrap Migrator is a sophisticated tool for systematically eliminating unwrap/expect/panic patterns from Rust codebases with context-aware intelligence.

### 🚀 Key Features

- **Context-Aware Processing**: Different migration strategies for production/examples/benchmarks/tests
- **Unicode-Safe Handling**: Proper boundary detection for international characters
- **Pattern Recognition**: 50+ specific migration patterns for different contexts
- **Safety Classification**: Automatic assessment of migration safety levels
- **Security PrimalError Integration**: Intelligent error handling integration

---

## 📋 Usage Examples

### Basic Analysis
```bash
# Analyze production code only
./target/release/beardog-unwrap-migrator --path ../crates --stats-only --exclude-tests

# Analyze entire codebase with context awareness
./target/release/beardog-unwrap-migrator --path .. --stats-only --context-aware
```

### Migration Operations
```bash
# Dry run with context awareness
./target/release/beardog-unwrap-migrator --path ../crates --dry-run --context-aware

# Apply migrations to production code
./target/release/beardog-unwrap-migrator --path ../crates --apply --exclude-tests

# Apply with specific strategies for different contexts
./target/release/beardog-unwrap-migrator --path ../examples --apply --context-aware --examples-strategy expect
```

### Advanced Options
```bash
# Focus on Security PrimalError compatible patterns only
./target/release/beardog-unwrap-migrator --path ../crates --apply --beardog-errors-only

# Custom strategies for different code types
./target/release/beardog-unwrap-migrator --path .. --apply --context-aware \
  --examples-strategy expect \
  --benchmarks-strategy expect
```

---

## 🎛️ Command Line Options

| Option | Description | Values |
|--------|-------------|---------|
| `--path` | Root path to scan | Directory path (default: `./crates`) |
| `--dry-run` | Preview changes without applying | Flag |
| `--apply` | Apply migrations to files | Flag |
| `--stats-only` | Show statistics only | Flag |
| `--exclude-tests` | Skip test files | Flag |
| `--context-aware` | Use enhanced context analysis | Flag |
| `--examples-strategy` | Strategy for example code | `safe`, `expect`, `skip` |
| `--benchmarks-strategy` | Strategy for benchmark code | `safe`, `expect`, `skip` |
| `--beardog-errors-only` | Only Security PrimalError compatible | Flag |

---

## 🔍 Migration Strategies

### Production Code Strategy
- **Unwrap → Error Propagation**: Convert to `?` operator where possible
- **Panic → Security PrimalError**: Convert panics to proper error returns
- **Expect → Context**: Add meaningful error context

### Example Code Strategy (`expect`)
- **Unwrap → Expect**: Add clear error messages
- **Focus on Clarity**: Prioritize educational value
- **Non-Blocking**: Allow examples to demonstrate concepts

### Benchmark Code Strategy (`expect`)
- **Performance Focus**: Minimize overhead
- **Clear Failures**: Obvious benchmark setup failures
- **Non-Critical**: Benchmark failures shouldn't crash

### Test Code Strategy (preserved)
- **Standard Practice**: Tests legitimately use unwrap
- **Panic on Failure**: Expected test behavior
- **No Migration**: Preserve existing patterns

---

## 🛡️ Safety Levels

### ✅ Safe
- Can be automatically migrated
- No risk of changing behavior
- Improves error handling

### ⚠️ Caution
- Requires review after migration
- May change error propagation
- Generally safe but verify

### ❌ Unsafe
- Should not be auto-migrated
- Complex context dependencies
- Manual review required

### 🧪 TestOnly
- Only applies to test code
- Standard test practices
- Preserve existing behavior

---

## 📊 Pattern Recognition

### Supported Patterns

#### Option Patterns
- `option.unwrap()` → `option.ok_or_else(...)?`
- `option.expect("msg")` → Enhanced context

#### Result Patterns  
- `result.unwrap()` → `result?`
- `result.expect("msg")` → Enhanced error propagation

#### Panic Patterns
- `panic!("msg")` → `return Err(Security PrimalError::...)`
- Context-specific panic handling

#### Runtime Patterns
- `Runtime::new().unwrap()` → Context-appropriate expect
- JSON serialization unwraps → Proper error handling
- Buffer creation unwraps → Safe alternatives

---

## 🎯 Best Practices

### Before Running
1. **Backup Code**: Ensure version control is clean
2. **Review Context**: Understand your codebase structure
3. **Start Small**: Begin with `--dry-run` on subset
4. **Test Strategy**: Plan testing after migration

### During Migration
1. **Use Context-Aware**: Enable `--context-aware` for intelligence
2. **Exclude Tests**: Use `--exclude-tests` for production focus
3. **Review Output**: Check migration suggestions carefully
4. **Incremental**: Migrate in small batches

### After Migration
1. **Compile Check**: Ensure code still compiles
2. **Run Tests**: Verify functionality unchanged
3. **Review Changes**: Manual review of critical paths
4. **Performance Test**: Ensure no performance regression

---

## 🔧 Building the Tool

```bash
# Build release version
cargo build --release

# Run tests
cargo test

# Check tool help
./target/release/beardog-unwrap-migrator --help
```

---

## 📈 Expected Results

### Typical Migration Results
- **Production Code**: 80-90% unwrap reduction
- **Examples**: Clear error messages added
- **Benchmarks**: Performance-focused error handling
- **Tests**: Appropriately preserved

### Quality Improvements
- **Error Handling**: Comprehensive error propagation
- **Code Clarity**: Better error messages and context
- **Maintainability**: Easier debugging and maintenance
- **Robustness**: Graceful failure handling

---

## 🚨 Troubleshooting

### Common Issues

#### Unicode Errors
- **Fixed**: Tool handles Unicode boundaries safely
- **Solution**: Use latest version (2.0.0+)

#### Context Detection
- **Issue**: Tool doesn't detect test/example context
- **Solution**: Use `--context-aware` flag

#### Over-Migration
- **Issue**: Tool migrates test code unnecessarily
- **Solution**: Use `--exclude-tests` for production focus

#### Compilation Errors
- **Issue**: Migrated code doesn't compile
- **Solution**: Review Security PrimalError imports and usage

---

## 🎊 Success Stories

### Security Primal Codebase Results
- **Before**: 50+ unwrap calls in production
- **After**: 9 unwrap calls (acceptable patterns)
- **Improvement**: 82% reduction in unwrap usage
- **Status**: Production ready

### Migration Statistics
- **Files Processed**: 773 production files
- **Patterns Migrated**: 41 successful migrations
- **Safety Level**: 100% safe migrations applied
- **Compilation**: Zero errors after migration

---

## 🔮 Future Enhancements

### Planned Features
- **Custom Pattern Support**: User-defined migration patterns
- **IDE Integration**: VSCode/IntelliJ plugins
- **Batch Processing**: Multi-project migration support
- **Rollback Support**: Automatic rollback on failure

### Contributing
- **Pattern Suggestions**: Submit new migration patterns
- **Bug Reports**: Report issues with context
- **Feature Requests**: Suggest improvements
- **Testing**: Help test on different codebases

---

*Tool Documentation - Version 2.0.0*  
*Last Updated: January 12, 2025*  
*Status: Production Ready* 