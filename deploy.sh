#!/bin/bash

# Songbird Universal Orchestrator Deployment Script
# Usage: ./deploy.sh [environment] [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
BINARY_NAME="songbird-orchestrator"

# Default values
ENVIRONMENT="${1:-development}"
BUILD_TYPE="${2:-debug}"
SKIP_TESTS="${3:-false}"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check if we're in the project directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        print_error "Not in the project root directory. Please run from the project root."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to set environment variables
set_environment_vars() {
    print_info "Setting environment variables for $ENVIRONMENT..."
    
    case $ENVIRONMENT in
        development|dev)
            export SONGBIRD_ENV=development
            export SONGBIRD_BIND_ADDRESS=127.0.0.1
            export SONGBIRD_BIND_PORT=8080
            export SONGBIRD_LOG_LEVEL=debug
            export SONGBIRD_METRICS_ENABLED=true
            export SONGBIRD_GAMING_MODE=false
            export RUST_LOG=debug
            ;;
        staging)
            export SONGBIRD_ENV=staging
            export SONGBIRD_BIND_ADDRESS=0.0.0.0
            export SONGBIRD_BIND_PORT=8080
            export SONGBIRD_LOG_LEVEL=info
            export SONGBIRD_METRICS_ENABLED=true
            export SONGBIRD_GAMING_MODE=true
            export RUST_LOG=info
            ;;
        production|prod)
            export SONGBIRD_ENV=production
            export SONGBIRD_BIND_ADDRESS=0.0.0.0
            export SONGBIRD_BIND_PORT=8080
            export SONGBIRD_LOG_LEVEL=warn
            export SONGBIRD_METRICS_ENABLED=true
            export SONGBIRD_GAMING_MODE=true
            export SONGBIRD_SECURITY_ENABLED=true
            export RUST_LOG=warn
            ;;
        *)
            print_error "Unknown environment: $ENVIRONMENT"
            print_info "Valid environments: development, staging, production"
            exit 1
            ;;
    esac
    
    print_success "Environment variables set for $ENVIRONMENT"
}

# Function to run tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        print_warning "Skipping tests..."
        return 0
    fi
    
    print_info "Running tests..."
    
    # Check formatting
    print_info "Checking code formatting..."
    cargo fmt --all -- --check
    
    # Run clippy
    print_info "Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    # Run tests
    print_info "Running test suite..."
    cargo test --workspace --lib --quiet
    
    # Check documentation
    print_info "Checking documentation..."
    cargo doc --no-deps --document-private-items --quiet
    
    print_success "All tests passed"
}

# Function to build the application
build_application() {
    print_info "Building application for $ENVIRONMENT ($BUILD_TYPE)..."
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo build --release
        BINARY_PATH="$PROJECT_ROOT/target/release/$BINARY_NAME"
    else
        cargo build
        BINARY_PATH="$PROJECT_ROOT/target/debug/$BINARY_NAME"
    fi
    
    if [[ -f "$BINARY_PATH" ]]; then
        print_success "Build completed: $BINARY_PATH"
    else
        print_error "Build failed: Binary not found at $BINARY_PATH"
        exit 1
    fi
}

# Function to create systemd service file
create_systemd_service() {
    if [[ "$ENVIRONMENT" == "development" ]]; then
        return 0
    fi
    
    print_info "Creating systemd service file..."
    
    cat > /tmp/songbird-orchestrator.service << EOF
[Unit]
Description=Songbird Universal Orchestrator
After=network.target

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=$PROJECT_ROOT
ExecStart=$BINARY_PATH
Restart=always
RestartSec=10
Environment=SONGBIRD_ENV=$ENVIRONMENT
Environment=SONGBIRD_BIND_ADDRESS=$SONGBIRD_BIND_ADDRESS
Environment=SONGBIRD_BIND_PORT=$SONGBIRD_BIND_PORT
Environment=SONGBIRD_LOG_LEVEL=$SONGBIRD_LOG_LEVEL
Environment=RUST_LOG=$RUST_LOG

[Install]
WantedBy=multi-user.target
EOF
    
    print_success "Systemd service file created at /tmp/songbird-orchestrator.service"
    print_info "To install: sudo cp /tmp/songbird-orchestrator.service /etc/systemd/system/"
}

# Function to run health check
run_health_check() {
    print_info "Starting health check..."
    
    # Start service in background
    "$BINARY_PATH" &
    SERVICE_PID=$!
    
    # Wait for service to start
    sleep 5
    
    # Check if service is running
    if ! kill -0 $SERVICE_PID 2>/dev/null; then
        print_error "Service failed to start"
        exit 1
    fi
    
    # Check health endpoint
    if curl -f "http://$SONGBIRD_BIND_ADDRESS:$SONGBIRD_BIND_PORT/health" > /dev/null 2>&1; then
        print_success "Health check passed"
    else
        print_warning "Health endpoint not available (this may be normal for some configurations)"
    fi
    
    # Clean up
    kill $SERVICE_PID
    wait $SERVICE_PID 2>/dev/null
    
    print_success "Health check completed"
}

# Function to show deployment summary
show_deployment_summary() {
    print_info "Deployment Summary:"
    echo "==================="
    echo "Environment: $ENVIRONMENT"
    echo "Build Type: $BUILD_TYPE"
    echo "Binary Path: $BINARY_PATH"
    echo "Bind Address: $SONGBIRD_BIND_ADDRESS"
    echo "Bind Port: $SONGBIRD_BIND_PORT"
    echo "Log Level: $SONGBIRD_LOG_LEVEL"
    echo ""
    
    print_info "To run the service:"
    echo "  $BINARY_PATH"
    echo ""
    
    print_info "To run with custom configuration:"
    echo "  SONGBIRD_BIND_PORT=9090 $BINARY_PATH"
    echo ""
    
    if [[ "$ENVIRONMENT" != "development" ]]; then
        print_info "For production deployment:"
        echo "  1. Copy systemd service: sudo cp /tmp/songbird-orchestrator.service /etc/systemd/system/"
        echo "  2. Create user: sudo useradd -r -s /bin/false songbird"
        echo "  3. Set permissions: sudo chown -R songbird:songbird $PROJECT_ROOT"
        echo "  4. Enable service: sudo systemctl enable songbird-orchestrator"
        echo "  5. Start service: sudo systemctl start songbird-orchestrator"
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [environment] [build_type] [skip_tests]"
    echo ""
    echo "Arguments:"
    echo "  environment   : development, staging, production (default: development)"
    echo "  build_type    : debug, release (default: debug)"
    echo "  skip_tests    : true, false (default: false)"
    echo ""
    echo "Examples:"
    echo "  $0                          # Development build with tests"
    echo "  $0 staging release          # Staging release build with tests"
    echo "  $0 production release true  # Production release build without tests"
    echo ""
    echo "Environment Variables:"
    echo "  SONGBIRD_BIND_ADDRESS      : Address to bind to"
    echo "  SONGBIRD_BIND_PORT         : Port to bind to"
    echo "  SONGBIRD_LOG_LEVEL         : Log level (debug, info, warn, error)"
    echo "  SONGBIRD_GAMING_MODE       : Enable gaming optimizations"
    echo "  SONGBIRD_SECURITY_ENABLED  : Enable security features"
}

# Main deployment function
main() {
    print_info "Starting Songbird Universal Orchestrator deployment..."
    print_info "Environment: $ENVIRONMENT, Build Type: $BUILD_TYPE"
    
    # Check for help
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then
        show_usage
        exit 0
    fi
    
    # Run deployment steps
    check_prerequisites
    set_environment_vars
    run_tests
    build_application
    create_systemd_service
    
    # Run health check for development
    if [[ "$ENVIRONMENT" == "development" ]]; then
        run_health_check
    fi
    
    show_deployment_summary
    
    print_success "Deployment completed successfully!"
}

# Run main function
main "$@" 