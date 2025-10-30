# Songbird Unwrap Migrator - Quick Usage Guide

## 🎯 Goal
Systematically eliminate **210 unwraps** and **157 expects** from Songbird codebase.

---

## ⚡ Quick Commands

### 1. **Analyze the Codebase** (Start Here)
```bash
cd tools/songbird-unwrap-migrator
cargo run -- --stats-only
```

**Expected output**:
```
📊 Songbird Codebase Analysis:
   📁 Files scanned: ~400
   ⚠️  Total unwrap/expect calls: ~367
   🔧 Migrable patterns: ~350
   🧪 Test file patterns: ~50
   ✅ Songbird compatible: ~350
```

### 2. **Preview Changes** (Always Do This First)
```bash
cargo run -- --dry-run
```

Shows what would be changed without actually changing files.

### 3. **Apply All Migrations** (Production)
```bash
cargo run -- --apply
```

Migrates all unwrap/expect calls to proper error handling.

---

## 🎓 Recommended Workflow

### Step 1: Test on One File
```bash
# Pick a small file with unwraps
cargo run -- --file ../../crates/songbird-config/src/discoverable_endpoint.rs --dry-run

# If it looks good, apply
cargo run -- --file ../../crates/songbird-config/src/discoverable_endpoint.rs --apply

# Test the changes
cd ../..
cargo test --lib --package songbird-config
```

### Step 2: Process One Crate
```bash
# Preview
cargo run -- --path ../../crates/songbird-config --dry-run

# Apply
cargo run -- --path ../../crates/songbird-config --apply

# Test
cd ../..
cargo test --lib --package songbird-config
cargo fmt
git diff
```

### Step 3: Process All Crates
```bash
# High-priority crates first
cargo run -- --path ../../crates/songbird-orchestrator --apply
cargo run -- --path ../../crates/songbird-discovery --apply
cargo run -- --path ../../crates/songbird-universal --apply

# Then supporting crates
cargo run -- --path ../../crates/songbird-config --apply
cargo run -- --path ../../crates/songbird-registry --apply

# Finally all others
cargo run -- --apply
```

---

## 📋 Crate Priority Order

Based on audit findings:

### **P0 - Critical Path** (Fix First)
1. `songbird-orchestrator` (~60 unwraps)
2. `songbird-discovery` (~40 unwraps)
3. `songbird-universal` (~30 unwraps)

### **P1 - High Traffic**
4. `songbird-config` (~35 unwraps)
5. `songbird-registry` (~30 unwraps)
6. `songbird-network` (~20 unwraps)

### **P2 - Supporting**
7. `songbird-observability`
8. `songbird-types`
9. Other crates

---

## ✅ After Each Migration

```bash
# 1. Format
cargo fmt

# 2. Check compilation
cargo check --package <crate-name>

# 3. Run tests
cargo test --lib --package <crate-name>

# 4. Review changes
git diff crates/<crate-name>

# 5. Commit
git add crates/<crate-name>
git commit -m "chore: migrate unwraps in <crate-name>"
```

---

## 🔍 What Gets Changed

### Before:
```rust
let port = env::var("PORT").unwrap();
let config = serde_json::from_str(&data).expect("Invalid JSON");
let first = collection.first().unwrap();
```

### After:
```rust
let port = env::var("PORT").map_err(|e| SongbirdError::internal_error(&format!("Environment variable 'PORT' not found: {}", e)))?;
let config = serde_json::from_str(&data).map_err(|e| SongbirdError::internal_error(&format!("Invalid JSON: {}", e)))?;
let first = collection.first().ok_or_else(|| SongbirdError::internal_error("Collection is empty when accessing first element"))?;
```

---

## 🚨 What Needs Manual Review

After running the tool, manually check:

1. **Function signatures** - Some may need `-> SongbirdResult<T>`
2. **Import statements** - May need `use songbird_errors::SongbirdError;`
3. **Complex contexts** - Some errors may benefit from specific error variants
4. **Test code** - Some test unwraps are intentional

---

## 📊 Progress Tracking

Create a checklist as you go:

```
[ ] songbird-orchestrator (60 unwraps)
[ ] songbird-discovery (40 unwraps)
[ ] songbird-universal (30 unwraps)
[ ] songbird-config (35 unwraps)
[ ] songbird-registry (30 unwraps)
[ ] songbird-network (20 unwraps)
[ ] songbird-observability (15 unwraps)
[ ] Other crates (remaining)
```

---

## 💡 Pro Tips

1. **Start small**: Test on one file first
2. **Commit often**: After each successful crate
3. **Test thoroughly**: Run tests after each migration
4. **Review changes**: Use `git diff` to review
5. **Exclude tests if needed**: Use `--exclude-tests` flag

---

## 🎯 Expected Timeline

- **Week 1**: 3-4 hours → 100 migrations
- **Week 2**: 3-4 hours → 100 migrations  
- **Week 3**: 3-4 hours → 100 migrations
- **Week 4**: 2 hours → Final 67 + cleanup

**Total**: ~8-10 hours of systematic work

---

## 🚀 One-Command Full Migration

If you're confident:

```bash
# Backup first!
git checkout -b unwrap-migration

# Run full migration
cargo run -- --apply

# Format and test
cd ../..
cargo fmt
cargo test --lib

# Review
git diff --stat
git diff

# Commit if good
git commit -am "chore: systematic unwrap/expect migration to SongbirdError"
```

---

## 📞 Quick Reference

```bash
# Analyze
cargo run -- --stats-only

# Preview one file
cargo run -- --file path/to/file.rs --dry-run

# Apply to one file
cargo run -- --file path/to/file.rs --apply

# Preview one crate
cargo run -- --path ../../crates/<crate-name> --dry-run

# Apply to one crate
cargo run -- --path ../../crates/<crate-name> --apply

# Preview everything
cargo run -- --dry-run

# Apply everything
cargo run -- --apply

# Exclude tests
cargo run -- --apply --exclude-tests
```

---

**Good luck! You're about to eliminate 367 panic points! 🚀**
