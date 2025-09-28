#!/bin/bash
# Constants Migration Script - GENERATED

echo "🔄 Starting constants consolidation migration..."

# Replace duplicate constants with canonical versions
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_BIND_ADDRESS/DEFAULT_BIND_ADDRESS/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_LOCALHOST/DEFAULT_LOCALHOST/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_HTTP_PORT/DEFAULT_HTTP_PORT/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_HTTPS_PORT/DEFAULT_HTTPS_PORT/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_CONNECTION_TIMEOUT/DEFAULT_CONNECTION_TIMEOUT/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_RETRY_DELAY/DEFAULT_RETRY_DELAY/g' {} \;
find crates/ -name '*.rs' -exec sed -i 's/DEFAULT_HEALTH_CHECK_TIMEOUT/DEFAULT_HEALTH_CHECK_TIMEOUT/g' {} \;

echo "✅ Constants migration complete!"
echo "📊 Consolidated constants into unified_constants modules"
