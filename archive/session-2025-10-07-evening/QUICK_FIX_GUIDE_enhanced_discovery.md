# Quick Fix Guide for enhanced_discovery.rs

## Pattern to Fix

You've successfully fixed this pattern in other files - the same issue exists throughout `enhanced_discovery.rs`:

```rust
// BROKEN (what you'll find):
pub field_name: Type)

// SHOULD BE:
pub field_name: Type,
```

## All Lines to Fix

Run this command to find them all:
```bash
grep -n ")" crates/songbird-discovery/src/discovery/enhanced_discovery.rs | grep -v "//" | grep -v "fn " | grep -v "impl " | grep -v "match " | grep -v "if "
```

Then for each struct field that ends with `)`, change it to `,`

## You're Almost There!

You've already fixed:
✅ service_discovery.rs  
✅ static_discovery.rs
✅ factory.rs
✅ 2 errors in enhanced_discovery.rs

Just need to fix the remaining `)` → `,` in enhanced_discovery.rs and you'll have **songbird-discovery compiling**!

Then just songbird-universal and you'll hit **100% compilation**! 🎉

