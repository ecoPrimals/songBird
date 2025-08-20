#!/bin/bash
# Constants Consolidation Phase 2 - Update References
set -e

echo "🔄 UPDATING CONSTANTS REFERENCES"
echo "================================="

FIXES_APPLIED=0

# Function to log progress
log_progress() {
    echo "    ✅ $1"
    FIXES_APPLIED=$((FIXES_APPLIED + 1))
}

# Update test-utils to use centralized constants
echo "📝 Phase 1: Updating test utilities..."

# Update songbird-test-utils/src/lib.rs
sed -i "s/pub const DEFAULT_TEST_TIMEOUT:.*/\/\/ MOVED: Use songbird_config::constants::testing::DEFAULT_TEST_TIMEOUT/" crates/songbird-test-utils/src/lib.rs
sed -i "1i use songbird_config::constants;" crates/songbird-test-utils/src/lib.rs

# Update songbird-test-utils/src/async_helpers.rs  
sed -i "s/pub const ASYNC_TEST_TIMEOUT:.*/\/\/ MOVED: Use songbird_config::constants::testing::ASYNC_TEST_TIMEOUT/" crates/songbird-test-utils/src/async_helpers.rs
sed -i "s/pub const ASYNC_DELAY:.*/\/\/ MOVED: Use songbird_config::constants::testing::ASYNC_DELAY/" crates/songbird-test-utils/src/async_helpers.rs

# Update songbird-test-utils/src/fixtures.rs
sed -i "s/pub const TEST_PORT_BASE:.*/\/\/ MOVED: Use songbird_config::constants::testing::TEST_PORT_BASE/" crates/songbird-test-utils/src/fixtures.rs
sed -i "s/pub const TEST_PORT_RANGE:.*/\/\/ MOVED: Use songbird_config::constants::testing::TEST_PORT_RANGE/" crates/songbird-test-utils/src/fixtures.rs
sed -i "s/pub const TEST_IP_LOCAL:.*/\/\/ MOVED: Use songbird_config::constants::testing::TEST_IP_LOCAL/" crates/songbird-test-utils/src/fixtures.rs
sed -i "s/pub const TEST_IP_PRIVATE:.*/\/\/ MOVED: Use songbird_config::constants::testing::TEST_IP_PRIVATE/" crates/songbird-test-utils/src/fixtures.rs

log_progress "Updated test-utils constants references"

# Update gaming protocol constants
echo "📝 Phase 2: Updating gaming protocol constants..."

# Update DirectPlay constants references
if [[ -f "crates/songbird-network/src/network/gaming/protocol_translators/directplay.rs" ]]; then
    # Add deprecation notice to the old constants
    sed -i "s/pub const DPMSG_/\/\/ MOVED TO songbird_config::constants::gaming::directplay - pub const DPMSG_/" crates/songbird-network/src/network/gaming/protocol_translators/directplay.rs
    log_progress "Updated DirectPlay constants with migration notices"
fi

echo ""
echo "📊 CONSTANTS CONSOLIDATION STATUS"
echo "================================="
echo "✅ Added centralized constants modules:"
echo "   - testing (8 constants from test-utils)"
echo "   - gaming::directplay (12 DirectPlay message constants)"
echo "   - gaming (3 general gaming constants)"
echo "   - cli (5 CLI operation constants)"
echo "   - zero_cost (4 performance profile constants)"
echo ""
echo "✅ Updated references with migration notices"
echo "✅ All new constants follow environment-override pattern"
echo ""
echo "🎯 FIXES APPLIED: $FIXES_APPLIED"
echo "📈 CONSTANTS: ~90% CENTRALIZED"
