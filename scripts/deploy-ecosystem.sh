#!/bin/bash

set -euo pipefail

# EcoPrimals Ecosystem Deployment Script
# Songbird Universal Orchestrator - Production Deployment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENVIRONMENT="${ENVIRONMENT:-production}"
DEPLOY_MODE="${DEPLOY_MODE:-full}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Banner
echo "🎵 EcoPrimals Ecosystem Deployment"
echo "=================================="
echo "Environment: $ENVIRONMENT"
echo "Deploy Mode: $DEPLOY_MODE"
echo "Project Root: $PROJECT_ROOT"
echo ""

# Pre-deployment checks
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check Rust (for local builds)
    if ! command -v cargo &> /dev/null; then
        log_warning "Rust/Cargo not found - using Docker builds only"
    fi
    
    log_success "Prerequisites check passed"
}

# Build Songbird
build_songbird() {
    log_info "Building Songbird..."
    
    cd "$PROJECT_ROOT"
    
    # Run tests first
    log_info "Running tests..."
    if cargo test --lib; then
        log_success "All tests passed"
    else
        log_error "Tests failed"
        exit 1
    fi
    
    # Build release
    log_info "Building release..."
    if cargo build --release --lib; then
        log_success "Build completed successfully"
    else
        log_error "Build failed"
        exit 1
    fi
    
    # Build Docker images
    log_info "Building Docker images..."
    docker build -f docker/Dockerfile.production -t songbird:latest .
    docker build -f docker/Dockerfile.production --target observability -t songbird-observability:latest .
    docker build -f docker/Dockerfile.production --target discovery -t songbird-discovery:latest .
    
    log_success "Docker images built successfully"
}

# Deploy ecosystem services
deploy_ecosystem() {
    log_info "Deploying ecosystem services..."
    
    cd "$PROJECT_ROOT"
    
    # Create necessary directories
    mkdir -p data/{songbird,beardog,toadstool,nestgate}
    mkdir -p logs/{songbird,beardog,toadstool,nestgate}
    mkdir -p config/{songbird,beardog,toadstool,nestgate}
    
    # Copy configuration files
    cp config/ecosystem-integration.toml config/songbird/
    
    # Deploy based on mode
    case $DEPLOY_MODE in
        "songbird-only")
            log_info "Deploying Songbird only..."
            docker-compose -f docker/docker-compose.production.yml up -d songbird-core songbird-observability songbird-discovery
            ;;
        "ecosystem")
            log_info "Deploying full ecosystem..."
            # Deploy BearDog if available
            if [ -f "docker/docker-compose.beardog.yml" ]; then
                docker-compose -f docker/docker-compose.beardog.yml up -d
            else
                log_warning "BearDog compose file not found - using Songbird security fallback"
            fi
            
            # Deploy ToadStool if available
            if [ -f "docker/docker-compose.toadstool.yml" ]; then
                docker-compose -f docker/docker-compose.toadstool.yml up -d
            else
                log_warning "ToadStool compose file not found - using local storage fallback"
            fi
            
            # Deploy NestGate if available
            if [ -f "docker/docker-compose.nestgate.yml" ]; then
                docker-compose -f docker/docker-compose.nestgate.yml up -d
            else
                log_warning "NestGate compose file not found - using direct networking fallback"
            fi
            
            # Deploy Songbird
            docker-compose -f docker/docker-compose.production.yml up -d
            ;;
        "full")
            log_info "Deploying full stack with monitoring..."
            # Deploy monitoring stack first
            if [ -f "docker/docker-compose.monitoring.yml" ]; then
                docker-compose -f docker/docker-compose.monitoring.yml up -d
                sleep 10  # Wait for monitoring to be ready
            fi
            
            # Deploy ecosystem
            deploy_ecosystem_services
            
            # Deploy Songbird
            docker-compose -f docker/docker-compose.production.yml up -d
            ;;
        *)
            log_error "Unknown deploy mode: $DEPLOY_MODE"
            exit 1
            ;;
    esac
    
    log_success "Ecosystem deployment completed"
}

# Deploy ecosystem services helper
deploy_ecosystem_services() {
    log_info "Deploying ecosystem services..."
    
    # BearDog Security
    if [ -f "docker/docker-compose.beardog.yml" ]; then
        log_info "Deploying BearDog security..."
        docker-compose -f docker/docker-compose.beardog.yml up -d
    fi
    
    # ToadStool Storage
    if [ -f "docker/docker-compose.toadstool.yml" ]; then
        log_info "Deploying ToadStool storage..."
        docker-compose -f docker/docker-compose.toadstool.yml up -d
    fi
    
    # NestGate Networking
    if [ -f "docker/docker-compose.nestgate.yml" ]; then
        log_info "Deploying NestGate networking..."
        docker-compose -f docker/docker-compose.nestgate.yml up -d
    fi
    
    # Wait for services to be ready
    log_info "Waiting for ecosystem services to be ready..."
    sleep 30
}

# Health checks
run_health_checks() {
    log_info "Running health checks..."
    
    # Check Songbird
    if curl -f http://localhost:8080/health >/dev/null 2>&1; then
        log_success "Songbird is healthy"
    else
        log_warning "Songbird health check failed"
    fi
    
    # Check Observability
    if curl -f http://localhost:9091/health >/dev/null 2>&1; then
        log_success "Observability service is healthy"
    else
        log_warning "Observability service health check failed"
    fi
    
    # Check Discovery
    if curl -f http://localhost:8081/health >/dev/null 2>&1; then
        log_success "Discovery service is healthy"
    else
        log_warning "Discovery service health check failed"
    fi
}

# Show deployment status
show_status() {
    log_info "Deployment Status:"
    echo ""
    docker-compose -f docker/docker-compose.production.yml ps
    echo ""
    
    log_info "Service Endpoints:"
    echo "🎵 Songbird Core: http://localhost:8080"
    echo "📊 Observability: http://localhost:9091"
    echo "🔍 Discovery: http://localhost:8081"
    echo "📈 Metrics: http://localhost:9090"
    echo ""
    
    log_info "Logs:"
    echo "View logs: docker-compose -f docker/docker-compose.production.yml logs -f"
    echo "View specific service: docker-compose -f docker/docker-compose.production.yml logs -f songbird-core"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    docker-compose -f docker/docker-compose.production.yml down
    log_success "Cleanup completed"
}

# Main deployment flow
main() {
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            build_songbird
            deploy_ecosystem
            sleep 10
            run_health_checks
            show_status
            ;;
        "cleanup")
            cleanup
            ;;
        "status")
            show_status
            ;;
        "health")
            run_health_checks
            ;;
        *)
            echo "Usage: $0 [deploy|cleanup|status|health]"
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 