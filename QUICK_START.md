# 🚀 Quick Start Guide - Songbird

Get up and running with Songbird in 5 minutes.

---

## Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo package manager
- Basic understanding of async Rust

---

## Installation

### Clone the Repository

```bash
git clone https://github.com/ecoPrimals/songbird
cd songbird
```

### Build the Project

```bash
# Build all workspace members
cargo build --workspace

# Or build with optimizations
cargo build --workspace --release
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test --package songbird-orchestrator
```

---

## Your First Service Discovery

### 1. Create a Basic Example

```rust
// examples/my_first_discovery.rs
use songbird_orchestrator::SongbirdOrchestrator;
use songbird_config::SongbirdConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with default configuration
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Discover services by capability
    let services = orchestrator
        .discovery()
        .find_by_capability("compute")
        .await?;

    println!("Found {} compute services", services.len());

    Ok(())
}
```

### 2. Run Your Example

```bash
cargo run --example my_first_discovery
```

---

## Common Tasks

### Service Discovery

```rust
// Find services by capability
let services = orchestrator
    .discovery()
    .find_by_capability("storage")
    .await?;

// Find services by type
let ai_services = orchestrator
    .discovery()
    .find_by_type("ai")
    .await?;
```

### Health Checking

```rust
// Check service health
let health = orchestrator
    .registry()
    .check_service_health("service-id")
    .await?;

if health.is_healthy() {
    println!("Service is healthy!");
}
```

### Request Routing

```rust
// Route request to best available service
let response = orchestrator
    .route_request("compute", request_data)
    .await?;
```

---

## Configuration

### Using Environment Variables

```bash
export SONGBIRD_DISCOVERY_INTERVAL=300
export SONGBIRD_HEALTH_CHECK_INTERVAL=60
export SONGBIRD_LOG_LEVEL=info
```

### Using Config File

```toml
# config/songbird.toml
[discovery]
interval_secs = 300
timeout_secs = 10

[health]
check_interval_secs = 60
timeout_secs = 5

[logging]
level = "info"
```

```rust
let config = SongbirdConfig::from_file("config/songbird.toml")?;
let orchestrator = SongbirdOrchestrator::new(config).await?;
```

---

## Running Examples

### Service Discovery

```bash
cargo run --example service_discovery
```

### Health Checks

```bash
cargo run --example health_checks
```

### Capability-Based Routing

```bash
cargo run --example capability_routing
```

### Full List

```bash
ls examples/
```

---

## Development Workflow

### 1. Make Changes

```bash
# Edit your code
vim crates/songbird-orchestrator/src/lib.rs
```

### 2. Format Code

```bash
cargo fmt --all
```

### 3. Run Lints

```bash
cargo clippy --workspace
```

### 4. Test Your Changes

```bash
cargo test --workspace
```

### 5. Build Release

```bash
cargo build --workspace --release
```

---

## Troubleshooting

### Build Fails

```bash
# Clean and rebuild
cargo clean
cargo build --workspace
```

### Tests Fail

```bash
# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test --package songbird-orchestrator test_name
```

### Missing Dependencies

```bash
# Update dependencies
cargo update
```

---

## Next Steps

### Learn More
- Read [Architecture Overview](ARCHITECTURE_OVERVIEW.md)
- Explore [API Reference](docs/API_REFERENCE.md)
- Check [Testing Guide](docs/TESTING.md)

### Explore Features
- [Capability-Based Discovery](docs/core/capability-discovery.md)
- [Intelligent Routing](docs/core/routing.md)
- [Health Monitoring](docs/core/health-monitoring.md)

### Join Community
- [GitHub Discussions](https://github.com/ecoPrimals/songbird/discussions)
- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

---

## Common Patterns

### Async/Await

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your async code here
    Ok(())
}
```

### Error Handling

```rust
use songbird_types::SongbirdError;

match orchestrator.discover().await {
    Ok(services) => println!("Found services: {:?}", services),
    Err(SongbirdError::Network { message, .. }) => {
        eprintln!("Network error: {}", message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Configuration Builder

```rust
let config = SongbirdConfig::builder()
    .discovery_interval(Duration::from_secs(300))
    .health_check_interval(Duration::from_secs(60))
    .log_level("info")
    .build()?;
```

---

## Tips

1. **Start Simple**: Begin with basic discovery, add complexity gradually
2. **Use Examples**: Run existing examples to understand patterns
3. **Check Logs**: Enable debug logging to understand behavior
4. **Test Locally**: Use mock services for local development
5. **Read Docs**: API documentation has detailed explanations

---

## Need Help?

- **Documentation**: [docs/](docs/)
- **Examples**: [examples/](examples/)
- **Issues**: [GitHub Issues](https://github.com/ecoPrimals/songbird/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ecoPrimals/songbird/discussions)

---

**Ready to build?** Check out the [examples/](examples/) directory for more patterns!

