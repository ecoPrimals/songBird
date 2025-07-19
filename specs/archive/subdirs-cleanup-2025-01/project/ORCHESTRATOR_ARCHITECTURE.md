# NestGate Orchestrator Architecture

## Overview

NestGate v2 implements a clean orchestrator-based architecture where **all services are managed through a single entry point** with **zero hardcoded values**. The orchestrator serves as the central coordination hub for the entire system.

## Architecture Principles

### ✅ Single Entry Point
- **Main Binary**: `nestgate-bin` contains only one `main()` function
- **Orchestrator**: All services are started, managed, and stopped through the orchestrator
- **No Independent Services**: No service starts independently or runs standalone

### ✅ Zero Hardcoding  
- **Configuration-Driven**: All endpoints, ports, and addresses come from configuration files
- **Environment-Aware**: Uses `Config::for_environment()` to load appropriate settings
- **Dynamic Port Assignment**: Services get ports dynamically based on orchestrator configuration
- **No Magic Numbers**: No hardcoded "localhost:8080" or similar values anywhere

### ✅ Service Discovery & Registration
- **Central Registry**: All services register with the orchestrator's service registry
- **Health Monitoring**: Orchestrator monitors all service health automatically
- **Load Balancing**: Built-in load balancing and routing through connection proxy
- **MCP Integration**: Full MCP protocol support for federation and communication

## System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    NestGate Main Binary                     │
│                  (Single Entry Point)                       │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                   Orchestrator                              │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │ Service Registry│ │Connection Proxy │ │Health Monitor │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │  Load Balancer  │ │ MCP Federation  │ │    Metrics    │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
└─────────────────────┬───────────────────────────────────────┘
                      │ (Manages All Services)
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Managed Services                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │ ZFS Service │ │ API Service │ │Network Svc  │ │MCP Svc  │ │
│  │   (Storage) │ │    (REST)   │ │(Networking) │ │(Proto)  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Configuration Architecture

### Configuration Sources (No Hardcoding)
```rust
// Load from environment-specific config files
let config = Config::for_environment("development").unwrap_or_else(|_| {
    Config::default() // Even defaults are not hardcoded
});

// All addresses come from configuration
let orchestrator_config = OrchestratorConfig {
    bind_address: format!("{}:{}", 
        config.orchestrator.bind_address,  // e.g., "0.0.0.0" from config
        config.orchestrator.port          // e.g., 8090 from config
    ),
    // All other settings from config files
};
```

### Dynamic Service Registration
```rust
// Services get endpoints dynamically based on config + offset
let zfs_service = ServiceInfo {
    service_id: "nestgate-zfs".to_string(),
    service_type: "storage".to_string(),
    endpoint: format!("http://{}:{}", 
        config.orchestrator.bind_address, 
        config.orchestrator.port + 1    // Dynamic port assignment
    ),
    // No hardcoded values
};
```

## Service Lifecycle Management

### 1. System Startup
1. **Load Configuration** from environment-specific files
2. **Create Orchestrator** with loaded configuration 
3. **Start Orchestrator** components (registry, proxy, health monitor, etc.)
4. **Register Services** dynamically with orchestrator
5. **Service Discovery** - services find each other through orchestrator

### 2. Service Management
- **Health Monitoring**: Orchestrator continuously monitors all services
- **Auto-Recovery**: Failed services are automatically restarted
- **Load Balancing**: Requests are distributed across healthy instances
- **Service Discovery**: Services discover each other through the registry

### 3. System Shutdown
- **Graceful Shutdown**: Orchestrator stops all services in correct order
- **Resource Cleanup**: All resources are properly cleaned up
- **State Persistence**: Service states are saved for next startup

## Key Features

### ✅ No Hardcoded Values
- All endpoints come from configuration
- Port assignments are dynamic
- Environment-specific configurations
- Default values are configurable

### ✅ Central Management
- Single point of control for all services
- Unified logging and monitoring
- Centralized configuration management
- Service lifecycle coordination

### ✅ Service Discovery
- Services register themselves with orchestrator
- Dynamic service discovery
- Health-aware routing
- Load balancing support

### ✅ MCP Integration
- Full MCP protocol support
- Federation capabilities
- Inter-service communication
- Protocol-level service management

## Configuration Files

### Environment-Specific Config
```yaml
# config/development.yaml
orchestrator:
  bind_address: "0.0.0.0"
  port: 8090
  
storage:
  data_dir: "/tmp/nestgate-dev"
  
# config/production.yaml  
orchestrator:
  bind_address: "127.0.0.1"
  port: 8080
  
storage:
  data_dir: "/var/lib/nestgate"
```

## Benefits

### 🚀 Clean Architecture
- Single responsibility principle
- Clear separation of concerns
- Maintainable codebase
- Testable components

### 🔧 Operational Excellence
- Easy deployment
- Configuration management
- Service monitoring
- Health checks

### 🌐 Scalability
- Service federation
- Load balancing
- Auto-scaling ready
- Multi-node support

### 🛡️ Reliability
- Health monitoring
- Auto-recovery
- Graceful degradation
- Circuit breaker patterns

## Usage

### Starting the System
```bash
# Development environment
NESTGATE_ENV=development cargo run --bin nestgate

# Production environment  
NESTGATE_ENV=production cargo run --bin nestgate
```

### Service Discovery
Services automatically discover each other through the orchestrator:
```rust
// Get a service endpoint dynamically
let zfs_endpoint = orchestrator.get_service("nestgate-zfs").await?;
```

### Health Monitoring
All services are monitored automatically:
```bash
# Check system health (no hardcoded ports)
curl http://$(orchestrator-address)/health
```

## Migration from Previous Architecture

### Before (Hardcoded)
- Multiple `main()` functions in different crates
- Hardcoded `localhost:8080`, `localhost:8081` everywhere
- Independent service startup
- Manual service coordination

### After (Orchestrator-Based)
- Single `main()` function in `nestgate-bin`
- Configuration-driven endpoints
- Orchestrator manages all services
- Automatic service discovery and health monitoring

This architecture ensures clean separation of concerns, eliminates hardcoding, and provides a robust foundation for the NestGate v2 system. 