#!/bin/bash
# Test Phase 1 trust parsing

echo "Testing Phase 1 trust parsing..."
echo ""

# Test 1: Integer format (BearDog)
echo "Test 1: Integer format"
cat > /tmp/test_trust_int.json << 'EOF'
{"trust_level": 1}
EOF

result=$(cargo run --release --bin songbird-orchestrator -- test-parse-trust < /tmp/test_trust_int.json 2>&1 || echo "PARSE_FAILED")

if echo "$result" | grep -q "PARSE_FAILED\|expected a string"; then
    echo "❌ FAILED: Cannot parse integer trust_level"
    exit 1
else
    echo "✅ PASSED: Integer parsing works"
fi

# Test 2: String format (Songbird)
echo ""
echo "Test 2: String format"
cat > /tmp/test_trust_str.json << 'EOF'
{"trust_level": "limited"}
EOF

result=$(cargo run --release --bin songbird-orchestrator -- test-parse-trust < /tmp/test_trust_str.json 2>&1 || echo "PARSE_FAILED")

if echo "$result" | grep -q "PARSE_FAILED"; then
    echo "❌ FAILED: Cannot parse string trust_level"
    exit 1
else
    echo "✅ PASSED: String parsing works"
fi

echo ""
echo "🎉 Phase 1 parsing verified!"

