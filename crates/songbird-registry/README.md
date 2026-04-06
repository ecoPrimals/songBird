# Songbird Registry

Service registry and discovery system for Songbird Universal Orchestrator.

## Features

- Dynamic service registration and discovery
- Health monitoring and status tracking
- Load balancing and failover support
- Capability-based service routing

## Usage

```rust
use songbird_registry::ServiceRegistry;

let registry = ServiceRegistry::new().await?;
registry.register_service(service_info).await?;
let services = registry.discover_services("compute").await?;
```

## License

Licensed under AGPL-3.0-or-later - the harshest copyleft license ensuring all derivatives remain open source. 