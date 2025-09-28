# Production Dockerfile for Songbird Gaming Bridge
# Optimized for <50ms latency and high throughput

# Build stage
FROM rust:1.75-slim-bullseye AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpcap-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build for release with optimization flags
ENV CARGO_TARGET_DIR=/app/target
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3"

# Build the gaming bridge binary
RUN cargo build --release --bin songbird

# Runtime stage
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    libpcap0.8 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN groupadd -r songbird && useradd -r -g songbird songbird

# Create necessary directories
RUN mkdir -p /app/logs /app/config /app/data && \
    chown -R songbird:songbird /app

# Copy the built binary
COPY --from=builder /app/target/release/songbird /usr/local/bin/songbird

# Copy example configurations
COPY --from=builder /app/examples/config/ /app/config/

# Set up performance-optimized configuration
COPY docker/songbird-production.toml /app/config/production.toml

# Switch to app user
USER songbird

# Set working directory
WORKDIR /app

# Performance optimization environment variables
ENV RUST_LOG=info
ENV SONGBIRD_CONFIG_PATH=/app/config/production.toml
ENV SONGBIRD_LOG_PATH=/app/logs
ENV SONGBIRD_DATA_PATH=/app/data

# Gaming bridge specific optimizations
ENV SONGBIRD_GAMING_BATCH_SIZE=64
ENV SONGBIRD_GAMING_WORKER_THREADS=8
ENV SONGBIRD_GAMING_BUFFER_SIZE=65536
ENV SONGBIRD_GAMING_QUEUE_SIZE=10000

# Network performance optimizations
ENV SONGBIRD_NET_SO_REUSEPORT=true
ENV SONGBIRD_NET_TCP_NODELAY=true
ENV SONGBIRD_NET_KEEPALIVE=true

# Expose ports for gaming bridge
EXPOSE 7000-8000/udp
EXPOSE 8080/tcp

# Health check for container orchestration
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD songbird health || exit 1

# Start the gaming bridge
CMD ["songbird", "gaming", "bridge", "--production"] 