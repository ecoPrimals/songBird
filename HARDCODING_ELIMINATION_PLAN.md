
# HARDCODING ELIMINATION MASTER PLAN

## 🎯 TARGET: Zero Hardcoding Violations (Currently: 527)

### 📋 SYSTEMATIC APPROACH:

#### Phase 1: Configuration Infrastructure
```rust
// src/config/defaults.rs
pub struct HardcodedValues {
    pub default_bind_address: IpAddr,
    pub default_ports: PortConfig,
    pub default_timeouts: TimeoutConfig,
    pub buffer_sizes: BufferConfig,
}
```

#### Phase 2: Environment Variables
```bash
# Core networking
SONGBIRD_BIND_ADDRESS=0.0.0.0
SONGBIRD_DISCOVERY_PORT=8080
SONGBIRD_GAMING_PORT_RANGE=7000-7100

# Timeouts
SONGBIRD_CONNECTION_TIMEOUT=30s
SONGBIRD_REQUEST_TIMEOUT=10s
SONGBIRD_HEALTH_CHECK_TIMEOUT=5s

# Buffer sizes
SONGBIRD_BUFFER_SIZE=8192
SONGBIRD_MAX_PACKET_SIZE=65536
```

#### Phase 3: Automated Replacement
1. IP addresses → `config.network.bind_address`
2. Ports → `config.network.port_ranges`
3. Timeouts → `config.timeouts.*`
4. Buffer sizes → `config.buffers.*`

### 🚀 EXECUTION STATUS: READY TO IMPLEMENT

