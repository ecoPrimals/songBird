# 🎯 Constants Migration Guide

## Centralized Constants Location

All constants have been centralized in: `songbird-config::constants`

## Available Constants Modules

### Discovery Constants
```rust
use songbird_config::constants::discovery;

// Usage
let timeout = discovery::DEFAULT_DISCOVERY_TIMEOUT_MS;
let max_concurrent = discovery::DEFAULT_MAX_CONCURRENT_DISCOVERIES;
```

### Network Constants  
```rust
use songbird_config::constants::network;

// Usage
let default_port = network::DEFAULT_ORCHESTRATOR_PORT;
let bind_address = network::DEFAULT_BIND_ADDRESS;
```

### Performance Constants
```rust  
use songbird_config::constants::performance;

// Usage
let max_response_time = performance::DEFAULT_MAX_RESPONSE_TIME_MS;
let min_success_rate = performance::DEFAULT_MIN_SUCCESS_RATE;
```

### Port Management Constants
```rust
use songbird_config::constants::ports;

// Usage
let port_start = ports::DEFAULT_PORT_RANGE_START;
let reserved_ports = ports::RESERVED_PORTS;
```

## Migration Examples

### Before (Hardcoded)
```rust
let timeout = 5000; // Magic number
let max_connections = 10; // Hardcoded
let port_range = 20000..30000; // Hardcoded range
```

### After (Centralized)
```rust
use songbird_config::constants::{discovery, performance, ports};

let timeout = discovery::DEFAULT_DISCOVERY_TIMEOUT_MS;
let max_connections = performance::DEFAULT_CONNECTION_POOL_SIZE;
let port_range = ports::DEFAULT_PORT_RANGE_START..ports::DEFAULT_PORT_RANGE_END;
```

## Benefits

- **Single Source of Truth**: All constants in one location
- **Easy Configuration**: Change constants in one place
- **Clear Semantics**: Constants have descriptive names
- **Environment Override**: All constants can be overridden via env vars
