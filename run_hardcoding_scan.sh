#!/bin/bash
# Hardcoding Detection and Analysis Script
# Analyzes hardcoded IPs, ports, and constants across the codebase

echo "🔍 SONGBIRD HARDCODING ANALYSIS"
echo "================================"
echo ""

cd "$(dirname "$0")"

echo "📊 Analyzing hardcoded IP addresses..."
echo ""

# Hardcoded localhost/loopback
echo "1. Localhost/Loopback (127.0.0.1):"
grep -r "127\.0\.0\.1" crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

# Private network ranges
echo "2. Private Network (192.168.*):"
grep -r "192\.168\." crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo "3. Private Network (10.0.0.*):"
grep -r "10\.0\.0\." crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo "4. Private Network (172.16-31.*):"
grep -r "172\.1[6-9]\.\|172\.2[0-9]\.\|172\.3[0-1]\." crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo ""
echo "📊 Analyzing hardcoded ports..."
echo ""

# Common hardcoded ports
echo "5. Port 8080:"
grep -r ":8080\|port.*8080\|8080.*port" crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo "6. Port 9000:"
grep -r ":9000\|port.*9000\|9000.*port" crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo "7. Port 3000:"
grep -r ":3000\|port.*3000\|3000.*port" crates/ src/ --include="*.rs" | grep -v "tests/" | grep -v "archive/" | wc -l

echo ""
echo "📊 Files with most hardcoding (Top 10):"
echo ""
grep -r "127\.0\.0\.1\|192\.168\.\|10\.0\.0\.\|:8080\|:9000\|:3000" crates/ src/ --include="*.rs" -l | \
  grep -v "tests/" | grep -v "archive/" | head -10

echo ""
echo "✅ Analysis complete!"
echo ""
echo "💡 Recommendation: Review files listed above"
echo "💡 Consider using environment variables instead of hardcoded values"
echo ""
echo "🔧 To migrate, the zero_hardcoding_migration tool is ready in:"
echo "   crates/songbird-config/src/zero_hardcoding_migration.rs"

