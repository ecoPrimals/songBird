#!/usr/bin/env bash
# Fix async test issues in songbird-universal
# All adapter constructors are async but tests were calling them synchronously

set -euo pipefail

echo "🔧 Fixing async test issues in songbird-universal..."

# Files that need fixing
TEST_FILES=(
    "crates/songbird-universal/tests/security_adapter_comprehensive_coverage_tests.rs"
    "crates/songbird-universal/tests/ai_adapter_comprehensive_coverage_tests.rs"
    "crates/songbird-universal/tests/storage_adapter_comprehensive_coverage_tests.rs"
    "crates/songbird-universal/tests/compute_adapter_comprehensive_coverage_tests.rs"
    "crates/songbird-universal/tests/security_adapter_integration_tests.rs"
    "crates/songbird-universal/tests/ai_adapter_async_integration_tests.rs"
    "crates/songbird-universal/tests/compute_adapter_async_integration_tests.rs"
)

for file in "${TEST_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "  Processing: $file"
        
        # Convert #[test] to #[tokio::test] for tests that call async functions
        sed -i 's/#\[test\]/#[tokio::test]/' "$file"
        
        # Convert fn test_ to async fn test_
        sed -i 's/^fn test_/async fn test_/' "$file"
        
        # Add .await to adapter constructors
        sed -i 's/SecurityAdapter::new(\([^)]*\))\.is_ok()/SecurityAdapter::new(\1).await.is_ok()/' "$file"
        sed -i 's/SecurityAdapter::new(\([^)]*\))\.expect(/SecurityAdapter::new(\1).await.expect(/' "$file"
        sed -i 's/SecurityAdapter::new(\([^)]*\))\.unwrap(/SecurityAdapter::new(\1).await.unwrap(/' "$file"
        
        sed -i 's/AIAdapter::new(\([^)]*\))\.is_ok()/AIAdapter::new(\1).await.is_ok()/' "$file"
        sed -i 's/AIAdapter::new(\([^)]*\))\.expect(/AIAdapter::new(\1).await.expect(/' "$file"
        sed -i 's/AIAdapter::new(\([^)]*\))\.unwrap(/AIAdapter::new(\1).await.unwrap(/' "$file"
        
        sed -i 's/StorageAdapter::new(\([^)]*\))\.is_ok()/StorageAdapter::new(\1).await.is_ok()/' "$file"
        sed -i 's/StorageAdapter::new(\([^)]*\))\.expect(/StorageAdapter::new(\1).await.expect(/' "$file"
        sed -i 's/StorageAdapter::new(\([^)]*\))\.unwrap(/StorageAdapter::new(\1).await.unwrap(/' "$file"
        
        sed -i 's/ComputeAdapter::new(\([^)]*\))\.is_ok()/ComputeAdapter::new(\1).await.is_ok()/' "$file"
        sed -i 's/ComputeAdapter::new(\([^)]*\))\.expect(/ComputeAdapter::new(\1).await.expect(/' "$file"
        sed -i 's/ComputeAdapter::new(\([^)]*\))\.unwrap(/ComputeAdapter::new(\1).await.unwrap(/' "$file"
        
        # Handle cases where adapter is assigned to variable first
        sed -i 's/let adapter = \(.*Adapter::new([^;]*\);/let adapter = \1.await;/' "$file"
        
        # Handle map_err patterns
        sed -i 's/\(.*Adapter::new([^)]*)\)\.map_err(/\1.await.map_err(/' "$file"
    fi
done

echo "✅ Async test fixes applied!"
echo ""
echo "Next: Run 'cargo test -p songbird-universal' to verify"

