#!/usr/bin/env bash
# 🎵 Songbird Federation - Interactive Quick Start

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

clear
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 Songbird Federation - Quick Start"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Choose your setup:"
echo
echo "1️⃣  Local Multi-Node (Single Machine)"
echo "   • Test federation with 3 local instances"
echo "   • Ports: 8000, 8001, 8002"
echo "   • Great for development and testing"
echo
echo "2️⃣  Start Seed Tower (Multi-Machine Setup)"
echo "   • Start a tower for others to connect to"
echo "   • Provides connection instructions"
echo "   • Use on your main tower"
echo
echo "3️⃣  Connect to Remote Tower"
echo "   • Connect to an existing tower"
echo "   • Interactive connection wizard"
echo "   • Use on secondary towers"
echo
echo "4️⃣  View Documentation"
echo "   • Multi-machine setup guide"
echo "   • Troubleshooting tips"
echo "   • Configuration reference"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo
        echo "🚀 Starting Local Multi-Node Demo..."
        echo
        cd "$SCRIPT_DIR/demos"
        ./01-mesh-formation.sh
        ;;
    
    2)
        echo
        echo "🚀 Starting Seed Tower..."
        echo
        cd "$SCRIPT_DIR/scripts"
        ./start-tower.sh
        
        echo
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "📋 For Other Towers to Connect:"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo
        echo "Run this command on other machines:"
        echo
        MY_IP=$(hostname -I | awk '{print $1}')
        echo "  SONGBIRD_PEERS=\"$MY_IP:8000\" ./scripts/start-tower.sh"
        echo
        echo "OR use the quick start:"
        echo "  cd showcase/02-federation"
        echo "  ./QUICK_START.sh  # Choose option 3"
        echo
        ;;
    
    3)
        echo
        echo "🚀 Connect to Remote Tower..."
        echo
        cd "$SCRIPT_DIR/demos"
        ./02-connect-to-remote.sh
        ;;
    
    4)
        echo
        echo "📚 Opening Documentation..."
        echo
        if command -v less &> /dev/null; then
            less "$SCRIPT_DIR/MULTI_MACHINE_SETUP.md"
        else
            cat "$SCRIPT_DIR/MULTI_MACHINE_SETUP.md"
        fi
        ;;
    
    *)
        echo
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

