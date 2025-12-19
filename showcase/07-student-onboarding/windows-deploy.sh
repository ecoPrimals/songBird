#!/bin/bash
# Windows Deployment Helper
# Prepares files for Windows laptop deployment

set -e

echo "🪟 Windows Deployment Helper"
echo "============================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create deployment directory
DEPLOY_DIR="target/windows-deploy"
echo "Creating deployment directory: $DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR/config"
mkdir -p "$DEPLOY_DIR/docs"
echo ""

# Copy binary
echo "1️⃣  Copying binary..."
if [ -f "target/release/songbird-orchestrator" ]; then
    cp target/release/songbird-orchestrator "$DEPLOY_DIR/songbird-orchestrator.exe"
    SIZE=$(du -h "$DEPLOY_DIR/songbird-orchestrator.exe" | cut -f1)
    echo -e "${GREEN}   ✅ Binary copied: $SIZE${NC}"
else
    echo -e "${YELLOW}   ⚠️  Binary not found, building...${NC}"
    cargo build --release --bin songbird-orchestrator
    cp target/release/songbird-orchestrator "$DEPLOY_DIR/songbird-orchestrator.exe"
    SIZE=$(du -h "$DEPLOY_DIR/songbird-orchestrator.exe" | cut -f1)
    echo -e "${GREEN}   ✅ Binary built and copied: $SIZE${NC}"
fi
echo ""

# Copy config
echo "2️⃣  Copying configuration..."
cp showcase/07-student-onboarding/config/local-network.toml "$DEPLOY_DIR/config/"
echo -e "${GREEN}   ✅ Config copied${NC}"
echo ""

# Copy documentation
echo "3️⃣  Copying documentation..."
cp showcase/07-student-onboarding/DEPLOYMENT_GUIDE.md "$DEPLOY_DIR/docs/"
cp showcase/07-student-onboarding/WINDOWS_TESTING.md "$DEPLOY_DIR/docs/"
cp showcase/07-student-onboarding/00_START_HERE.md "$DEPLOY_DIR/docs/"
echo -e "${GREEN}   ✅ Documentation copied${NC}"
echo ""

# Create README for Windows
cat > "$DEPLOY_DIR/README.txt" << 'EOF'
Songbird Orchestrator - Windows Deployment
=========================================

QUICK START:
1. Edit config\local-network.toml (set your registry IP)
2. Run: songbird-orchestrator.exe --config config\local-network.toml
3. Open firewall: Port 8080
4. Test: http://localhost:8080/health

FULL GUIDE:
See docs\DEPLOYMENT_GUIDE.md

TROUBLESHOOTING:
- Firewall blocking? Run PowerShell as admin:
  New-NetFirewallRule -DisplayName "Songbird" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow

- Can't connect to registry? Check:
  - Is Eastgate running?
  - Are you on the same network?
  - Can you ping 192.168.1.144?

SUPPORT:
Kevin Mok: mokkevin@msu.edu
EOF
echo -e "${GREEN}   ✅ README created${NC}"
echo ""

# Create start script for Windows
cat > "$DEPLOY_DIR/start.bat" << 'EOF'
@echo off
echo Starting Songbird Orchestrator...
echo.
songbird-orchestrator.exe --config config\local-network.toml
pause
EOF
echo -e "${GREEN}   ✅ Start script created${NC}"
echo ""

# Create directory listing
echo "4️⃣  Files ready for Windows:"
find "$DEPLOY_DIR" -type f -exec basename {} \; | sort | sed 's/^/   - /'
echo ""

echo "============================="
echo -e "${GREEN}✅ Deployment package ready!${NC}"
echo ""
echo "Next steps:"
echo "1. Copy $DEPLOY_DIR to Windows laptop"
echo "2. Edit config\local-network.toml (update registry URL)"
echo "3. Run start.bat (or use PowerShell)"
echo "4. Test: http://localhost:8080/health"
echo ""
echo "See docs/DEPLOYMENT_GUIDE.md for full instructions"
echo ""

