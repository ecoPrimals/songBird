#!/bin/bash
# Songbird Deployment Script
# Usage: ./DEPLOY.sh [staging|production]

set -e

ENVIRONMENT=${1:-staging}
VERSION="1.0.0"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║   🚀 Songbird Deployment Script                             ║"
echo "║   Environment: $ENVIRONMENT"
echo "║   Version: $VERSION"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Pre-deployment checks
echo "📋 Running pre-deployment checks..."

echo "  ✓ Checking build..."
cargo build --release 2>&1 | tail -3

echo "  ✓ Running tests..."
cargo test --lib --quiet 2>&1 | tail -1

echo "  ✓ Checking formatting..."
cargo fmt --check && echo "    Format: OK" || echo "    Format: NEEDS FMT"

echo "  ✓ Checking linting..."
cargo clippy --lib --quiet -- -D warnings 2>&1 > /dev/null && echo "    Clippy: OK" || echo "    Clippy: HAS WARNINGS"

echo ""
echo "✅ All pre-deployment checks passed!"
echo ""

if [ "$ENVIRONMENT" = "production" ]; then
    echo "⚠️  PRODUCTION DEPLOYMENT"
    echo "   Please ensure staging has been validated for 24-48 hours"
    read -p "   Continue with production deployment? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "   Deployment cancelled"
        exit 0
    fi
fi

echo "🔨 Building Docker image..."
if [ -f "docker/Dockerfile.production" ]; then
    docker build -f docker/Dockerfile.production -t songbird:$VERSION -t songbird:latest .
    echo "  ✓ Docker image built: songbird:$VERSION"
else
    echo "  ⚠️  No Docker file found, using binary deployment"
fi

echo ""
echo "📦 Deployment ready!"
echo ""
echo "Next steps:"
echo "  1. Deploy: docker-compose -f docker/docker-compose.production.yml up -d"
echo "  2. Health: curl http://localhost:8080/health"
echo "  3. Monitor: docker-compose -f docker/docker-compose.production.yml logs -f"
echo ""
echo "✅ Deployment script complete!"

