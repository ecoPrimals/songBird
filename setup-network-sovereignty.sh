#!/bin/bash
# Songbird Self-Sovereign Network Setup
# 
# This script gives Songbird the capability to manage its own network configuration
# without requiring full root privileges or manual intervention.
#
# Run once during initial setup: sudo ./setup-network-sovereignty.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORCHESTRATOR_BIN="$SCRIPT_DIR/target/release/songbird-orchestrator"

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║      🦅 SONGBIRD SELF-SOVEREIGN NETWORK SETUP                     ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ This script must be run with sudo for initial setup"
    echo "   Usage: sudo ./setup-network-sovereignty.sh"
    exit 1
fi

echo "📋 This script will:"
echo "   1. Grant Songbird CAP_NET_BIND_SERVICE (bind to ports < 1024)"
echo "   2. Grant Songbird CAP_NET_ADMIN (manage network configuration)"
echo "   3. Configure firewall rules for Songbird ports"
echo "   4. Create systemd service (optional)"
echo "   5. Make configuration persistent across reboots"
echo ""

# Build if needed
if [ ! -f "$ORCHESTRATOR_BIN" ]; then
    echo "🔨 Building Songbird orchestrator..."
    cd "$SCRIPT_DIR"
    cargo build --release
    echo "✅ Build complete"
fi

# Grant Linux capabilities to the binary
echo ""
echo "🔐 Granting Linux capabilities to Songbird..."
echo "   This allows Songbird to manage its own networking without full root"

# CAP_NET_BIND_SERVICE: Bind to ports < 1024
# CAP_NET_ADMIN: Configure network interfaces, routing, iptables
setcap 'cap_net_bind_service,cap_net_admin=+ep' "$ORCHESTRATOR_BIN"

if [ $? -eq 0 ]; then
    echo "✅ Capabilities granted: cap_net_bind_service, cap_net_admin"
    echo "   Songbird can now:"
    echo "   • Bind to any port (including privileged ports)"
    echo "   • Configure its own firewall rules"
    echo "   • Manage network interfaces"
else
    echo "❌ Failed to grant capabilities"
    exit 1
fi

# Verify capabilities
echo ""
echo "🔍 Verifying capabilities..."
getcap "$ORCHESTRATOR_BIN"

# Configure default firewall rules
echo ""
echo "🔥 Configuring firewall rules..."

# Detect firewall system
if command -v ufw &> /dev/null; then
    echo "   Detected: ufw"
    ufw allow 8080/tcp comment "Songbird HTTPS"
    ufw allow 2300/udp comment "Songbird Discovery"
    echo "✅ ufw rules added"
elif command -v firewall-cmd &> /dev/null; then
    echo "   Detected: firewalld"
    firewall-cmd --permanent --add-port=8080/tcp
    firewall-cmd --permanent --add-port=2300/udp
    firewall-cmd --reload
    echo "✅ firewalld rules added"
else
    echo "   No firewall manager detected, using iptables directly"
    
    # Add rules
    iptables -C INPUT -p tcp --dport 8080 -j ACCEPT 2>/dev/null || \
        iptables -I INPUT -p tcp --dport 8080 -j ACCEPT
    
    iptables -C INPUT -p udp --dport 2300 -j ACCEPT 2>/dev/null || \
        iptables -I INPUT -p udp --dport 2300 -j ACCEPT
    
    # Make persistent
    if command -v iptables-save &> /dev/null; then
        mkdir -p /etc/iptables
        iptables-save > /etc/iptables/rules.v4
        echo "✅ iptables rules added and saved"
        
        # Install iptables-persistent if available
        if command -v apt-get &> /dev/null; then
            echo "   Installing iptables-persistent for automatic restore..."
            DEBIAN_FRONTEND=noninteractive apt-get install -y iptables-persistent 2>&1 | grep -v "^Reading" || true
        fi
    fi
fi

# Create systemd service (optional)
echo ""
read -p "📦 Create systemd service for automatic startup? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    SERVICE_FILE="/etc/systemd/system/songbird.service"
    
    cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Songbird Orchestrator - Self-Sovereign Distributed Computing
Documentation=https://github.com/ecoPrimals/songBird
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$SUDO_USER
Group=$SUDO_USER
WorkingDirectory=$SCRIPT_DIR
Environment="RUST_LOG=info"
Environment="SONGBIRD_TLS_ENABLED=true"
Environment="SONGBIRD_FEDERATION_ENABLED=true"
Environment="SONGBIRD_ANONYMOUS_DISCOVERY=true"
ExecStart=$ORCHESTRATOR_BIN
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=songbird

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$SCRIPT_DIR/logs $SCRIPT_DIR/certs $SCRIPT_DIR/data
CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_ADMIN
AmbientCapabilities=CAP_NET_BIND_SERVICE CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    
    echo "✅ Systemd service created: $SERVICE_FILE"
    echo ""
    echo "   To enable automatic startup:"
    echo "   sudo systemctl enable songbird"
    echo ""
    echo "   To start now:"
    echo "   sudo systemctl start songbird"
    echo ""
    echo "   To check status:"
    echo "   sudo systemctl status songbird"
fi

# Summary
echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║      ✅ SONGBIRD IS NOW SELF-SOVEREIGN                            ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "🦅 Songbird can now:"
echo "   • Start without sudo or manual firewall configuration"
echo "   • Manage its own network configuration"
echo "   • Auto-configure firewall rules as needed"
echo "   • Work on new deployments out-of-the-box"
echo ""
echo "🚀 To start Songbird (no sudo needed):"
echo "   ./start-tower.sh"
echo ""
echo "📊 To verify sovereignty:"
echo "   getcap $ORCHESTRATOR_BIN"
echo "   iptables -L -n | grep -E '8080|2300'"
echo ""
echo "🎯 Next deployment will work automatically!"
echo ""

