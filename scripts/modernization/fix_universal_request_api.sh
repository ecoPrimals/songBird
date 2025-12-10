#!/usr/bin/env bash
# Fix UniversalRequest API mismatches
#
# The UniversalRequest API changed from:
#   capability_type, payload, metadata
# To:
#   request_id, source, target, action, parameters, security_context

set -euo pipefail

echo "🔧 Fixing UniversalRequest API mismatches..."

TARGET_FILE="crates/songbird-universal/tests/sovereignty_adapter_error_tests.rs"

if [ ! -f "$TARGET_FILE" ]; then
    echo "❌ Error: $TARGET_FILE not found"
    exit 1
fi

echo "📝 File: $TARGET_FILE"
echo "   Found $(grep -c 'capability_type:' "$TARGET_FILE" 2>/dev/null || echo 0) instances to fix"

echo ""
echo "⚠️  Manual intervention required!"
echo "   The API has changed completely and each instance needs context-specific values."
echo ""
echo "   OLD API:"
echo "     capability_type: String"
echo "     payload: Vec<u8>"
echo "     metadata: HashMap<String, String>"
echo ""
echo "   NEW API:"
echo "     request_id: String"
echo "     source: String  
echo "     target: String"
echo "     action: String"
echo "     parameters: HashMap<String, serde_json::Value>"
echo "     security_context: Option<SecurityContext>"
echo ""
echo "   Each test needs meaningful values based on what it's testing."
echo ""
echo "   Example transformation:"
echo "     // OLD"
echo "     let request = UniversalRequest {"
echo "         capability_type: \"test\".to_string(),"
echo "         payload: vec![],"
echo "         metadata: HashMap::new(),"
echo "     };"
echo ""
echo "     // NEW"
echo "     let request = UniversalRequest {"
echo "         request_id: uuid::Uuid::new_v4().to_string(),"
echo "         source: \"test-client\".to_string(),"
echo "         target: \"test-service\".to_string(),"
echo "         action: \"test\".to_string(),"
echo "         parameters: HashMap::new(),"
echo "         security_context: None,"
echo "     };"
echo ""
echo "📍 Open file to fix: $TARGET_FILE"
echo "   Line 56, 82, 108, 134, 160, 177, 194, 216, 235, 268, 312, 337, 364"
echo ""
echo "✅ After fixing, run: cargo test -p songbird-universal --lib --no-run"

