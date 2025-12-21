#!/usr/bin/env bash
#
# Federation Deployment Script
# Deploy Universal Port Authority updates across the tower federation
#
# This script helps coordinate deployment across Eastgate, Westgate, and Strandgate

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

TOWERS=("eastgate" "westgate" "strandgate")
CURRENT_TOWER=$(hostname | tr '[:upper:]' '[:lower:]')

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARN:${NC} $*"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $*"
}

section() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $*"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Check if running on a known tower
check_tower() {
    section "Tower Identification"
    
    local found=false
    for tower in "${TOWERS[@]}"; do
        if [[ "$CURRENT_TOWER" == *"$tower"* ]]; then
            found=true
            break
        fi
    done
    
    if ! $found; then
        warn "Running on unknown tower: ${CURRENT_TOWER}"
        warn "Expected one of: ${TOWERS[*]}"
    else
        log "✅ Running on: ${CURRENT_TOWER}"
    fi
}

# Git operations
ensure_clean_workspace() {
    section "Git Workspace Check"
    
    cd ~/Development/ecoPrimals/songbird
    
    if [[ -n $(git status --porcelain) ]]; then
        warn "Workspace has uncommitted changes"
        git status --short
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            error "Aborting deployment"
            exit 1
        fi
    else
        log "✅ Workspace is clean"
    fi
}

# Pull latest changes
pull_updates() {
    section "Pulling Latest Updates"
    
    cd ~/Development/ecoPrimals/songbird
    
    log "Fetching from origin..."
    git fetch origin
    
    local current_branch=$(git branch --show-current)
    log "Current branch: ${current_branch}"
    
    log "Pulling updates..."
    if git pull origin "$current_branch"; then
        log "✅ Updates pulled successfully"
    else
        error "Failed to pull updates"
        exit 1
    fi
}

# Build
build_release() {
    section "Building Release Binary"
    
    cd ~/Development/ecoPrimals/songbird
    
    log "Building with cargo..."
    if cargo build --release 2>&1 | tee /tmp/songbird-build.log; then
        log "✅ Build successful"
    else
        error "Build failed. Check /tmp/songbird-build.log"
        exit 1
    fi
}

# Stop existing instance
stop_songbird() {
    section "Stopping Existing Songbird Instance"
    
    local pid_file=~/.songbird/songbird.pid
    
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            log "Stopping Songbird (PID: ${pid})..."
            kill "$pid"
            sleep 2
            
            if ps -p "$pid" > /dev/null 2>&1; then
                warn "Graceful shutdown failed, forcing..."
                kill -9 "$pid"
            fi
            
            log "✅ Songbird stopped"
        else
            log "PID file exists but process not running"
            rm -f "$pid_file"
        fi
    else
        log "No existing instance found"
    fi
}

# Start new instance
start_songbird() {
    section "Starting Songbird"
    
    cd ~/Development/ecoPrimals/songbird
    
    log "Starting new instance..."
    nohup ./target/release/songbird > ~/.songbird/songbird.log 2>&1 &
    
    local pid=$!
    echo "$pid" > ~/.songbird/songbird.pid
    
    log "Songbird started (PID: ${pid})"
    log "Waiting for startup..."
    
    # Wait for health check
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -sk https://localhost:8080/health > /dev/null 2>&1; then
            log "✅ Songbird is healthy"
            return 0
        fi
        
        sleep 1
        attempt=$((attempt + 1))
    done
    
    error "Songbird failed to start"
    tail -20 ~/.songbird/songbird.log
    exit 1
}

# Verify federation
check_federation() {
    section "Checking Federation Status"
    
    log "Querying federation nodes..."
    
    local response=$(curl -sk https://localhost:8080/api/v1/federation/nodes 2>/dev/null)
    
    if [[ -z "$response" ]]; then
        error "Failed to query federation"
        return 1
    fi
    
    local node_count=$(echo "$response" | jq -r '.nodes | length')
    log "Federation has ${node_count} node(s)"
    
    echo "$response" | jq -r '.nodes[] | "  - \(.node_name) (\(.status))"'
    
    log "✅ Federation check complete"
}

# Full deployment
full_deploy() {
    section "🚀 Full Deployment on ${CURRENT_TOWER}"
    
    check_tower
    ensure_clean_workspace
    pull_updates
    build_release
    stop_songbird
    start_songbird
    sleep 3
    check_federation
    
    section "✅ Deployment Complete on ${CURRENT_TOWER}"
    
    log ""
    log "Songbird is running with:"
    log "  - Enhanced Capability Router"
    log "  - Universal Port Authority"
    log "  - Service Registry endpoints"
    log "  - Federation discovery"
    log ""
    log "Logs: tail -f ~/.songbird/songbird.log"
    log "Status: curl -sk https://localhost:8080/health | jq"
    log ""
}

# Quick restart (no pull/build)
quick_restart() {
    section "Quick Restart on ${CURRENT_TOWER}"
    
    stop_songbird
    start_songbird
    sleep 3
    check_federation
    
    section "✅ Restart Complete"
}

# Show deployment status
show_status() {
    section "Songbird Status on ${CURRENT_TOWER}"
    
    local pid_file=~/.songbird/songbird.pid
    
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            log "✅ Running (PID: ${pid})"
            
            if curl -sk https://localhost:8080/health > /dev/null 2>&1; then
                log "✅ Health check passing"
                check_federation
            else
                warn "Health check failing"
            fi
        else
            error "PID file exists but process not running"
        fi
    else
        error "Not running"
    fi
}

# Main menu
main() {
    cat << 'EOF'

╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║  🎵 Songbird Federation Deployment                                ║
║     Universal Port Authority - Cross-Tower Updates               ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝

EOF

    if [[ $# -eq 0 ]]; then
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  deploy     - Full deployment (pull, build, restart)"
        echo "  restart    - Quick restart (no pull/build)"
        echo "  status     - Show current status"
        echo "  stop       - Stop Songbird"
        echo "  start      - Start Songbird"
        echo "  build      - Build only"
        echo "  federation - Check federation"
        echo ""
        exit 1
    fi
    
    case "$1" in
        deploy)
            full_deploy
            ;;
        restart)
            quick_restart
            ;;
        status)
            show_status
            ;;
        stop)
            stop_songbird
            ;;
        start)
            start_songbird
            ;;
        build)
            build_release
            ;;
        federation)
            check_federation
            ;;
        *)
            error "Unknown command: $1"
            exit 1
            ;;
    esac
}

main "$@"

