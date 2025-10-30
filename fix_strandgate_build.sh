#!/usr/bin/env bash
#
# Strandgate Build Fix Script
# 
# This script diagnoses and fixes the build failures reported from Strandgate
#

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        🔧 Songbird Strandgate Build Fix Script 🔧             ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || ! grep -q "songbird" Cargo.toml; then
    echo "❌ ERROR: Not in songbird project root"
    echo "   Please cd to ~/Development/ecoPrimals/songbird first"
    exit 1
fi

echo "✅ In songbird project root"
echo ""

# Step 1: Check for the problematic core.rs file
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 1: Checking discovery/core.rs..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

CORE_FILE="crates/songbird-discovery/src/discovery/core.rs"

if [ -f "$CORE_FILE" ]; then
    FILE_SIZE=$(stat -f%z "$CORE_FILE" 2>/dev/null || stat -c%s "$CORE_FILE" 2>/dev/null)
    echo "✅ core.rs exists (${FILE_SIZE} bytes)"
    head -5 "$CORE_FILE"
else
    echo "❌ core.rs is MISSING!"
    echo "   This is the root cause of the build failure"
fi

echo ""

# Step 2: Check git status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 2: Checking git status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

git status --short | head -20
echo ""

# Step 3: Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: $CURRENT_BRANCH"
echo ""

if [ "$CURRENT_BRANCH" != "type-unification-capability" ]; then
    echo "⚠️  WARNING: You're on branch '$CURRENT_BRANCH'"
    echo "   Expected: 'type-unification-capability'"
    echo ""
    read -p "Switch to type-unification-capability branch? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout type-unification-capability
    fi
fi

# Step 4: Offer to clean and rebuild
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Step 3: Build Fix Options"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Choose a fix option:"
echo ""
echo "  1) Clean build (cargo clean + build)"
echo "  2) Hard reset from origin (WARNING: loses local changes)"
echo "  3) Manual inspection (don't fix automatically)"
echo ""
read -p "Enter choice (1-3): " CHOICE

case $CHOICE in
    1)
        echo ""
        echo "🧹 Cleaning build artifacts..."
        cargo clean
        echo ""
        echo "🔨 Building..."
        cargo build --release --workspace
        echo ""
        echo "✅ Build complete!"
        ;;
    2)
        echo ""
        echo "⚠️  WARNING: This will discard ALL local changes!"
        read -p "Are you sure? Type 'yes' to confirm: " CONFIRM
        if [ "$CONFIRM" = "yes" ]; then
            echo ""
            echo "📥 Fetching from origin..."
            git fetch origin
            echo ""
            echo "🔄 Hard reset to origin/type-unification-capability..."
            git reset --hard origin/type-unification-capability
            echo ""
            echo "🧹 Cleaning build artifacts..."
            cargo clean
            echo ""
            echo "🔨 Building..."
            cargo build --release --workspace
            echo ""
            echo "✅ Build complete!"
        else
            echo "Cancelled."
        fi
        ;;
    3)
        echo ""
        echo "📋 Manual inspection mode"
        echo ""
        echo "Check these files:"
        echo "  - $CORE_FILE"
        echo "  - crates/songbird-discovery/src/discovery/mod.rs"
        echo ""
        echo "Try these commands:"
        echo "  cargo clean"
        echo "  cargo build --release --workspace"
        echo "  git fetch origin && git reset --hard origin/type-unification-capability"
        ;;
    *)
        echo "Invalid choice."
        exit 1
        ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Done!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "If build still fails, see FEDERATION_SETUP_CORRECTED.md for details."
echo ""

