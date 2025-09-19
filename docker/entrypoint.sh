#!/bin/bash

set -euo pipefail

# Set default values
export RUST_ENV=${RUST_ENV:-production}
export SONGBIRD_ENVIRONMENT=${SONGBIRD_ENVIRONMENT:-production}
export SONGBIRD_BIND_ADDRESS=${SONGBIRD_BIND_ADDRESS:-0.0.0.0}
export SONGBIRD_PORT=${SONGBIRD_PORT:-8080}
export SONGBIRD_LOG_LEVEL=${SONGBIRD_LOG_LEVEL:-info}

# Create necessary directories
mkdir -p /app/{data,logs,metrics}

# Print startup information
echo "🎵 Starting Songbird in ${SONGBIRD_ENVIRONMENT} mode..."
echo "📍 Binding to: ${SONGBIRD_BIND_ADDRESS}:${SONGBIRD_PORT}"
echo "📊 Log level: ${SONGBIRD_LOG_LEVEL}"

# Set up logging
export RUST_LOG="songbird=${SONGBIRD_LOG_LEVEL},tower_http=info"

# Wait for dependencies if needed
if [ -n "${WAIT_FOR:-}" ]; then
    echo "⏳ Waiting for dependencies: ${WAIT_FOR}"
    for service in ${WAIT_FOR//,/ }; do
        until curl -f "http://${service}/health" >/dev/null 2>&1; do
            echo "⏳ Waiting for ${service}..."
            sleep 2
        done
        echo "✅ ${service} is ready"
    done
fi

# Execute the command
echo "🚀 Starting: $*"
exec "$@" 