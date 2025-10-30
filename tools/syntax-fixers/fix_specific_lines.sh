#!/bin/bash
# Fix specific syntax errors identified by cargo check

# Fix songbird-canonical/src/performance.rs:11
sed -i '11s/)$/);/' crates/songbird-canonical/src/performance.rs

# Fix songbird-universal/src/capabilities.rs format! calls missing semicolons
sed -i 's/format!("PRIMAL_{}_NAME ", i)$/format!("PRIMAL_{}_NAME ", i);/g' crates/songbird-universal/src/capabilities.rs
sed -i 's/format!("PRIMAL_{}_ENDPOINT ", i)$/format!("PRIMAL_{}_ENDPOINT ", i);/g' crates/songbird-universal/src/capabilities.rs

echo "✅ Fixed specific lines"

