#!/bin/bash
# Update Songbird on Towers B and C via HTTP Deployment API
# Uses federation-native deployment (no SSH required)

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 SONGBIRD HTTP UPDATE - TOWERS B & C"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Tower configuration
TOWER_A="http://192.168.1.144:8080"  # Eastgate (local)
TOWER_B="http://192.168.1.134:8080"  # Strandgate
TOWER_C="http://192.168.1.207:8080"  # Southgate

# Binary to deploy
BINARY_PATH="${1:-./target/release/songbird}"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Binary not found: $BINARY_PATH"
    echo ""
    echo "Build first:"
    echo "  cargo build --release --bin songbird"
    exit 1
fi

BINARY_SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
echo "📦 Binary: $BINARY_PATH"
echo "   Size: $(numfmt --to=iec-i --suffix=B $BINARY_SIZE 2>/dev/null || echo $BINARY_SIZE bytes)"
echo ""

# Function to deploy to a tower
deploy_to_tower() {
    local TOWER_NAME=$1
    local TOWER_URL=$2
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 DEPLOYING TO: $TOWER_NAME"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Test connectivity
    echo "🔍 Testing connectivity..."
    if ! curl -s -f -m 5 "$TOWER_URL/health" > /dev/null; then
        echo "❌ $TOWER_NAME not responding at $TOWER_URL"
        echo "   Check if Songbird is running on $TOWER_NAME"
        return 1
    fi
    echo "✅ $TOWER_NAME is online"
    echo ""
    
    # Check deployment API availability
    echo "🔍 Checking deployment API..."
    if ! curl -s -f -m 5 "$TOWER_URL/api/deployment/capabilities" > /dev/null; then
        echo "⚠️  Deployment API not available on $TOWER_NAME"
        echo "   This Songbird instance may not have deployment API enabled"
        return 1
    fi
    echo "✅ Deployment API available"
    echo ""
    
    # Show current capabilities
    echo "📊 Current capabilities:"
    curl -s "$TOWER_URL/api/deployment/capabilities" | jq -r '{
        node_id,
        network_type: .network.type,
        single_upload: .deployment_methods.single.enabled,
        max_size_mb: .deployment_methods.single.max_size_mb,
        storage_gb: .resources.available_storage_gb
    }' 2>/dev/null || echo "   (capabilities query failed)"
    echo ""
    
    # Deploy the binary
    echo "📤 Uploading new Songbird binary..."
    RESPONSE=$(curl -s -X POST "$TOWER_URL/api/deployment/binary" \
        -F "binary=@$BINARY_PATH" \
        -F "service_name=songbird-orchestrator-updated" \
        -F 'env_vars={}' \
        -F "auto_start=false")
    
    if echo "$RESPONSE" | jq -e '.deployment_id' > /dev/null 2>&1; then
        DEPLOYMENT_ID=$(echo "$RESPONSE" | jq -r '.deployment_id')
        echo "✅ Upload successful!"
        echo "   Deployment ID: $DEPLOYMENT_ID"
        echo "$RESPONSE" | jq '{deployment_id, status, message}'
        echo ""
        
        # Get deployment path
        DEPLOYED_PATH=$(curl -s "$TOWER_URL/api/deployment/status/$DEPLOYMENT_ID" | jq -r '.binary_path')
        echo "📍 Binary deployed to: $DEPLOYED_PATH"
        echo ""
        
        echo "⚠️  MANUAL STEP REQUIRED:"
        echo "   SSH to $TOWER_NAME and run:"
        echo "   sudo systemctl stop songbird"
        echo "   sudo cp $DEPLOYED_PATH /usr/local/bin/songbird"
        echo "   sudo systemctl start songbird"
        echo ""
        
        return 0
    else
        echo "❌ Deployment failed"
        echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
        return 1
    fi
}

# Deploy to Tower B
echo ""
if deploy_to_tower "TOWER B (Strandgate)" "$TOWER_B"; then
    echo "✅ Tower B deployment successful"
else
    echo "❌ Tower B deployment failed"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Deploy to Tower C
if deploy_to_tower "TOWER C (Southgate)" "$TOWER_C"; then
    echo "✅ Tower C deployment successful"
else
    echo "❌ Tower C deployment failed"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ HTTP DEPLOYMENT COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "1. SSH to each tower and replace the running binary"
echo "2. Restart Songbird service"
echo "3. Verify federation: curl $TOWER_A/api/federation/nodes | jq '.'"
echo ""

