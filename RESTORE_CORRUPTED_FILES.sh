#!/bin/bash
# Quick script to restore corrupted files
# Run this to fix the 3 corrupted crates

echo "🔧 Restoring corrupted files from git history..."
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Run this from the songbird root directory"
    exit 1
fi

# Backup current corrupted versions (just in case)
echo "📦 Creating backups of corrupted files..."
cp crates/songbird-primal-sdk/src/adaptive_discovery.rs crates/songbird-primal-sdk/src/adaptive_discovery.rs.corrupted 2>/dev/null
cp crates/songbird-cli/src/cli/commands/status.rs crates/songbird-cli/src/cli/commands/status.rs.corrupted 2>/dev/null
cp crates/songbird-config/tests/comprehensive_config_tests.rs crates/songbird-config/tests/comprehensive_config_tests.rs.corrupted 2>/dev/null
echo "✅ Backups created with .corrupted extension"
echo ""

# Restore from previous commit
echo "⚡ Restoring files from HEAD~1..."
git checkout HEAD~1 -- crates/songbird-primal-sdk/src/adaptive_discovery.rs
git checkout HEAD~1 -- crates/songbird-cli/src/cli/commands/status.rs  
git checkout HEAD~1 -- crates/songbird-config/tests/comprehensive_config_tests.rs
echo "✅ Files restored"
echo ""

# Verify compilation
echo "🔨 Testing compilation..."
echo ""

echo "Testing songbird-primal-sdk..."
if cargo build --lib -p songbird-primal-sdk 2>&1 | grep -q "Finished"; then
    echo "✅ songbird-primal-sdk compiles!"
else
    echo "⚠️  songbird-primal-sdk has issues"
fi
echo ""

echo "Testing songbird-cli..."
if cargo build --lib -p songbird-cli 2>&1 | grep -q "Finished"; then
    echo "✅ songbird-cli compiles!"
else
    echo "⚠️  songbird-cli has issues"
fi
echo ""

echo "Testing songbird-config..."
if cargo test --lib -p songbird-config 2>&1 | grep -q "test result: ok"; then
    echo "✅ songbird-config tests pass!"
else
    echo "⚠️  songbird-config has test issues"
fi
echo ""

# Re-enable in Cargo.toml
echo "📝 Re-enabling crates in Cargo.toml..."
sed -i 's/# "crates\/songbird-cli",/"crates\/songbird-cli",/' Cargo.toml
sed -i 's/# "crates\/songbird-primal-sdk",/"crates\/songbird-primal-sdk",/' Cargo.toml
echo "✅ Crates re-enabled in workspace"
echo ""

# Final verification
echo "🎯 Final verification - building entire workspace..."
if cargo build --workspace --lib 2>&1 | tail -5 | grep -q "Finished"; then
    echo ""
    echo "🎉 SUCCESS! All crates compile!"
    echo ""
    echo "Next steps:"
    echo "1. Run: cargo test --workspace --lib"
    echo "2. Run: cargo fmt --all -- --check"
    echo "3. Commit the restored files: git add -u && git commit -m 'Restore corrupted files'"
    echo ""
else
    echo ""
    echo "⚠️  Some crates still have issues. Check the output above."
    echo ""
    echo "You can restore the corrupted versions with:"
    echo "  cp crates/songbird-primal-sdk/src/adaptive_discovery.rs.corrupted crates/songbird-primal-sdk/src/adaptive_discovery.rs"
    echo "  cp crates/songbird-cli/src/cli/commands/status.rs.corrupted crates/songbird-cli/src/cli/commands/status.rs"
    echo ""
fi

