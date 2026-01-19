# 🌍 Songbird Universal IPC

**Platform-agnostic IPC for ecoPrimals** - Works on ALL platforms!

## Overview

Universal IPC eliminates platform-specific code (`#[cfg(unix)]`, `#[cfg(windows)]`) from application primals by providing a unified API that works consistently everywhere.

**Before** (platform-specific mess):
```rust
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(windows)]
use tokio::net::windows::named_pipe;

#[cfg(unix)]
let stream = UnixStream::connect("/tmp/beardog.sock").await?;
#[cfg(windows)]
let stream = ClientOptions::new().open(r"\\.\pipe\beardog")?;
```

**After** (universal!):
```rust
use songbird_universal_ipc::ipc;

let stream = ipc::connect("/primal/beardog").await?;
// Works on Linux, macOS, Windows, RISC-V, everywhere!
```

## Features

- ✅ **Platform-Agnostic**: Same API on all platforms
- ✅ **Unix Sockets**: Linux, macOS, BSD (implemented)
- ✅ **Named Pipes**: Windows (TODO)
- ✅ **TCP Fallback**: For platforms without native IPC
- ✅ **Service Discovery**: Find services by capability
- ✅ **Type-Safe**: Rust type system ensures correctness
- ✅ **Async**: Built on Tokio for high performance
- ✅ **Zero Unsafe**: 100% safe Rust

## Quick Start

### Server (Register and Listen)

```rust
use songbird_universal_ipc::ipc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize universal IPC
    ipc::init()?;
    
    // Register this primal with capabilities
    let endpoint = ipc::register("myprimal", vec!["capability1"]).await?;
    
    // Listen for connections
    let mut listener = ipc::listen(endpoint).await?;
    
    // Accept connections
    while let Ok(mut stream) = listener.accept().await {
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                // Handle request...
                stream.write_all(&buf[..n]).await.ok();
            }
        });
    }
    
    Ok(())
}
```

### Client (Connect)

```rust
use songbird_universal_ipc::ipc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize universal IPC
    ipc::init()?;
    
    // Connect to a primal (platform-agnostic path!)
    let mut stream = ipc::connect("/primal/beardog").await?;
    
    // Use stream (same on ALL platforms!)
    stream.write_all(b"hello").await?;
    
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
    
    Ok(())
}
```

## Service Discovery

Find services by capability:

```rust
use songbird_universal_ipc::ipc;

// Register with capabilities
ipc::register("beardog", vec!["crypto".to_string()]).await?;
ipc::register("squirrel", vec!["ai".to_string()]).await?;

// Find services by capability
let crypto_services = ipc::find_by_capability("crypto").await;
// Returns: ["/primal/beardog"]

let ai_services = ipc::find_by_capability("ai").await;
// Returns: ["/primal/squirrel"]

// List all services
let all_services = ipc::list_services().await;
// Returns: ["beardog", "squirrel"]
```

## Platform Support

| Platform | Implementation | Status |
|----------|---------------|--------|
| Linux | Unix sockets | ✅ Implemented |
| macOS | Unix sockets | ✅ Implemented |
| BSD | Unix sockets | ✅ Implemented |
| Windows | Named pipes | 🚧 TODO |
| Others | TCP localhost | ✅ Fallback |

## Examples

Run the examples:

```bash
# Terminal 1: Start server
cargo run --example simple_server

# Terminal 2: Run client
cargo run --example simple_client

# Service discovery demo
cargo run --example discovery
```

## Architecture

```
Application Layer (BearDog, Squirrel, etc.):
  - Uses virtual paths: "/primal/beardog"
  - Platform-agnostic code!

Universal IPC Layer (this crate):
  - Translates virtual → native endpoints
  - Service registry
  - Platform abstraction

Platform Layer:
  - Unix: /tmp/primal-beardog.sock
  - Windows: \\.\pipe\primal-beardog
  - Fallback: 127.0.0.1:{port}
```

## Integration with Tower Atomic

Works seamlessly with Tower Atomic (JSON-RPC over IPC):

```rust
// Get universal stream
let stream = ipc::connect("/primal/beardog").await?;

// Use with Tower Atomic (JSON-RPC)
// ... Tower Atomic code here ...
```

## Testing

Run tests:

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# All tests with logging
RUST_LOG=debug cargo test
```

## License

AGPL-3.0

## Contributing

Part of the ecoPrimals ecosystem. See main repository for contribution guidelines.

---

🌍🦀✨ **Universal IPC - Write Once, Run Everywhere!** ✨🦀🌍

