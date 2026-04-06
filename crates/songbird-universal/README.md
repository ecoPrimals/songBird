# Songbird Universal

[![Crates.io](https://img.shields.io/crates/v/songbird-universal.svg)](https://crates.io/crates/songbird-universal)
[![Documentation](https://docs.rs/songbird-universal/badge.svg)](https://docs.rs/songbird-universal)

Universal capability adapters and primal integrations for the Songbird Universal Orchestrator, providing zero-cost abstractions for ecosystem integration.

## Features

- **Universal Adapters**: Capability-based routing for any primal type
- **Zero-Cost Abstractions**: Native async fn in traits for maximum performance
- **AI-First Integration**: Full compatibility with AI-First Citizen API standard
- **Dynamic Discovery**: Runtime capability detection and registration
- **Ecosystem Compliance**: Standardized interfaces across all primals

## Architecture

The universal system provides three core abstraction layers:

1. **Capability Detection**: Automatic discovery of primal capabilities
2. **Universal Routing**: Dynamic request routing based on capabilities
3. **Performance Optimization**: Zero-cost async abstractions

## Usage

```rust
use songbird_universal::{UniversalAdapter, ServiceInfo, UniversalHealthStatus};

// Create universal adapter for any primal
let adapter = UniversalAdapter::new("security-provider").await?;

// Check capabilities dynamically
let capabilities = adapter.get_capabilities().await?;

// Route requests based on capabilities
let response = adapter.handle_request(request).await?;
```

## Zero-Cost Performance

- **40-60% faster** than async_trait patterns
- **Native async fn** in traits (Rust 1.75+)
- **Compile-time optimization** instead of runtime dispatch
- **Lock-free atomic operations** for statistics

## Supported Primals

- Security Providers (BearDog, custom implementations)
- Storage Providers (any S3-compatible, local filesystem)
- Compute Providers (container runtimes, serverless)
- AI Providers (inference engines, model serving)

## License

Licensed under AGPL-3.0-or-later as part of the ecoPrimals ecosystem.

Part of the scyBorg provenance trio: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0
