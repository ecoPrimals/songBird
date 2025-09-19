# Songbird Primal SDK

[![Crates.io](https://img.shields.io/crates/v/songbird-primal-sdk.svg)](https://crates.io/crates/songbird-primal-sdk)
[![Documentation](https://docs.rs/songbird-primal-sdk/badge.svg)](https://docs.rs/songbird-primal-sdk)

The **Songbird Primal SDK** provides a type-safe, zero-cost abstraction layer for building integrations with Songbird Primal services. This SDK enables developers to create robust, production-ready applications that interact seamlessly with the Songbird Universal Orchestrator ecosystem.

## Features

- 🚀 **Zero-Cost Abstractions**: Compile-time optimizations with no runtime overhead
- 🔒 **Type Safety**: Comprehensive type system prevents common integration errors
- ⚡ **Async-First**: Built on Tokio for high-performance concurrent operations
- 🛡️ **Error Handling**: Comprehensive error types with actionable suggestions
- 📊 **Observability**: Built-in tracing and metrics collection
- 🔌 **Extensible**: Plugin architecture for custom Primal implementations

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
songbird-primal-sdk = "0.1.0"
```

## Basic Usage

```rust
use songbird_primal_sdk::{PrimalClient, PrimalConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PrimalConfig::default();
    let client = PrimalClient::new(config).await?;
    
    // Register your service with the orchestrator
    client.register_service("my-service", "1.0.0").await?;
    
    Ok(())
}
```

## Documentation

For complete API documentation, visit [docs.rs/songbird-primal-sdk](https://docs.rs/songbird-primal-sdk).

## License

Licensed under the AGPL-3.0 license. See [LICENSE](../../LICENSE) for details. 