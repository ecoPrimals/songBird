#!/bin/bash
# Quick Deploy Script - Songbird Production
# Generated: November 21, 2025

set -e

echo "🚀 Songbird Production Deployment"
echo "=================================="
echo ""

# Set production environment
export SONGBIRD_ENV=production
export SONGBIRD_BIND_ADDRESS=0.0.0.0

echo "✅ Environment: $SONGBIRD_ENV"
echo "✅ Bind Address: $SONGBIRD_BIND_ADDRESS"
echo ""

# Build release
echo "📦 Building production release..."
cargo build --workspace --release

echo ""
echo "🧪 Running final tests..."
cargo test --workspace --release --quiet

echo ""
echo "✅ All tests passed!"
echo ""
echo "🚀 Ready to deploy!"
echo ""
echo "To start Songbird:"
echo "  ./target/release/songbird-orchestrator"
echo ""
echo "Or run this script with 'start' argument:"
echo "  ./DEPLOY_NOW.sh start"

if [ "$1" = "start" ]; then
    echo ""
    echo "🚀 Starting Songbird..."
    ./target/release/songbird-orchestrator
fi
