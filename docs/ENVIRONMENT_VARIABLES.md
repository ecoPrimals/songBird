# Songbird Environment Variables Reference

**Version**: 0.2.0  
**Last Updated**: November 24, 2025  
**Status**: Complete

---

## 📋 **Table of Contents**

1. [Core Service Ports](#core-service-ports)
2. [External Service Ports](#external-service-ports)
3. [Gaming Ports](#gaming-ports)
4. [Host Configuration](#host-configuration)
5. [Environment & Runtime](#environment--runtime)
6. [Timeouts & Limits](#timeouts--limits)
7. [Discovery & Federation](#discovery--federation)
8. [Security & TLS](#security--tls)
9. [Observability](#observability)
10. [Advanced Configuration](#advanced-configuration)

---

## Core Service Ports

Main Songbird service ports for orchestration, discovery, and management.

### `SONGBIRD_ORCHESTRATOR_PORT`
- **Default**: `8080`
- **Type**: `u16`
- **Description**: Main orchestrator service port for task coordination
- **Example**: `SONGBIRD_ORCHESTRATOR_PORT=8080`
- **Used by**: Orchestrator service, API gateway

### `SONGBIRD_DISCOVERY_PORT`
- **Default**: `8081`
- **Type**: `u16`
- **Description**: Service discovery port for capability-based routing
- **Example**: `SONGBIRD_DISCOVERY_PORT=8081`
- **Used by**: Discovery service, service registry

### `SONGBIRD_DASHBOARD_PORT`
- **Default**: `3000`
- **Type**: `u16`
- **Description**: Web dashboard UI port
- **Example**: `SONGBIRD_DASHBOARD_PORT=3000`
- **Used by**: Dashboard service, web UI

### `SONGBIRD_METRICS_PORT`
- **Default**: `9090`
- **Type**: `u16`
- **Description**: Metrics and observability port (Prometheus-compatible)
- **Example**: `SONGBIRD_METRICS_PORT=9090`
- **Used by**: Metrics exporter, Prometheus scraper

### `SONGBIRD_SECURITY_PORT`
- **Default**: `8443`
- **Type**: `u16`
- **Description**: Security service port (BearDog) for authentication/authorization
- **Example**: `SONGBIRD_SECURITY_PORT=8443`
- **Used by**: Security service, BearDog primal

### `SONGBIRD_HEALTH_PORT`
- **Default**: `8002`
- **Type**: `u16`
- **Description**: Health check endpoint port
- **Example**: `SONGBIRD_HEALTH_PORT=8002`
- **Used by**: Health monitoring, load balancers

### `SONGBIRD_FEDERATION_PORT`
- **Default**: `8082`
- **Type**: `u16`
- **Description**: Federation coordination port for multi-tower communication
- **Example**: `SONGBIRD_FEDERATION_PORT=8082`
- **Used by**: Federation service, multi-tower coordination

### `SONGBIRD_WEBSOCKET_PORT`
- **Default**: `8080`
- **Type**: `u16`
- **Description**: WebSocket streaming port for real-time updates
- **Example**: `SONGBIRD_WEBSOCKET_PORT=8080`
- **Used by**: WebSocket server, real-time clients

### `SONGBIRD_TARPC_PORT`
- **Default**: `8091`
- **Type**: `u16`
- **Description**: High-performance tarpc RPC port (~50μs latency, 100x faster than JSON-RPC)
- **Example**: `SONGBIRD_TARPC_PORT=8091`
- **Used by**: Native Rust client-to-server communication

---

## External Service Ports

Ports for external service registries and coordination systems.

### `CONSUL_PORT`
- **Default**: `8500`
- **Type**: `u16`
- **Description**: HashiCorp Consul service registry port
- **Example**: `CONSUL_PORT=8500`
- **Used by**: Service discovery, external registry integration
- **Documentation**: https://www.consul.io/

### `ETCD_PORT`
- **Default**: `2379`
- **Type**: `u16`
- **Description**: etcd distributed key-value store port
- **Example**: `ETCD_PORT=2379`
- **Used by**: Service discovery, configuration storage
- **Documentation**: https://etcd.io/

### `ZOOKEEPER_PORT`
- **Default**: `2181`
- **Type**: `u16`
- **Description**: Apache ZooKeeper coordination service port
- **Example**: `ZOOKEEPER_PORT=2181`
- **Used by**: Service coordination, leader election
- **Documentation**: https://zookeeper.apache.org/

### `EUREKA_PORT`
- **Default**: `8761`
- **Type**: `u16`
- **Description**: Netflix Eureka service registry port
- **Example**: `EUREKA_PORT=8761`
- **Used by**: Service discovery (Spring Cloud compatible)
- **Documentation**: https://github.com/Netflix/eureka

---

## Gaming Ports

Ports for gaming services and classic game server support.

### `SONGBIRD_GAMING_PORT`
- **Default**: `6112`
- **Type**: `u16`
- **Description**: Main gaming service port (StarCraft IPX default)
- **Example**: `SONGBIRD_GAMING_PORT=6112`
- **Used by**: Gaming service, game servers

### `SONGBIRD_STARCRAFT_PORT`
- **Default**: `6112`
- **Type**: `u16`
- **Description**: StarCraft game server port
- **Example**: `SONGBIRD_STARCRAFT_PORT=6112`
- **Used by**: StarCraft/Brood War servers

### `SONGBIRD_AOE2_PORT`
- **Default**: `2300`
- **Type**: `u16`
- **Description**: Age of Empires 2 game server port
- **Example**: `SONGBIRD_AOE2_PORT=2300`
- **Used by**: Age of Empires 2 servers

### `SONGBIRD_GAMING_PORT_START`
- **Default**: `7000`
- **Type**: `u16`
- **Description**: Start of gaming port range for dynamic allocation
- **Example**: `SONGBIRD_GAMING_PORT_START=7000`
- **Used by**: Dynamic game server port allocation

### `SONGBIRD_GAMING_PORT_END`
- **Default**: `7100`
- **Type**: `u16`
- **Description**: End of gaming port range for dynamic allocation
- **Example**: `SONGBIRD_GAMING_PORT_END=7100`
- **Used by**: Dynamic game server port allocation

### `SONGBIRD_CNC_PORT_START`
- **Default**: `1234`
- **Type**: `u16`
- **Description**: Command & Conquer port range start
- **Example**: `SONGBIRD_CNC_PORT_START=1234`
- **Used by**: C&C game servers

### `SONGBIRD_CNC_PORT_END`
- **Default**: `1240`
- **Type**: `u16`
- **Description**: Command & Conquer port range end
- **Example**: `SONGBIRD_CNC_PORT_END=1240`
- **Used by**: C&C game servers

---

## Host Configuration

Network host and bind address configuration.

### `SONGBIRD_HOST`
- **Default**: `"127.0.0.1"`
- **Type**: `String`
- **Description**: Default service host for all services
- **Example**: `SONGBIRD_HOST=127.0.0.1`
- **Used by**: All services as default host

### `SONGBIRD_BIND_ADDRESS`
- **Default**: `"0.0.0.0"`
- **Type**: `String`
- **Description**: Bind address for services
- **Notes**:
  - `"0.0.0.0"` - Bind to all interfaces (production)
  - `"127.0.0.1"` - Bind to localhost only (development)
- **Example**: `SONGBIRD_BIND_ADDRESS=0.0.0.0`
- **Used by**: All services for network binding

### `SONGBIRD_DISCOVERY_HOST`
- **Default**: Value of `SONGBIRD_HOST`
- **Type**: `String`
- **Description**: Discovery service host
- **Example**: `SONGBIRD_DISCOVERY_HOST=discovery.local`
- **Used by**: Discovery service clients

### `SONGBIRD_ORCHESTRATOR_HOST`
- **Default**: Value of `SONGBIRD_HOST`
- **Type**: `String`
- **Description**: Orchestrator service host
- **Example**: `SONGBIRD_ORCHESTRATOR_HOST=orchestrator.local`
- **Used by**: Orchestrator service clients

### `SONGBIRD_SECURITY_HOST`
- **Default**: `"localhost"`
- **Type**: `String`
- **Description**: Security service (BearDog) host
- **Example**: `SONGBIRD_SECURITY_HOST=security.local`
- **Used by**: Security service clients

---

## Environment & Runtime

Environment configuration and runtime behavior.

### `SONGBIRD_ENVIRONMENT`
- **Default**: `"development"`
- **Type**: `String`
- **Values**: `"production"`, `"staging"`, `"development"`, `"test"`
- **Description**: Runtime environment mode
- **Example**: `SONGBIRD_ENVIRONMENT=production`
- **Used by**: All services for environment-specific behavior

### `SONGBIRD_LOG_LEVEL`
- **Default**: `"info"`
- **Type**: `String`
- **Values**: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`
- **Description**: Logging verbosity level
- **Example**: `SONGBIRD_LOG_LEVEL=debug`
- **Used by**: Logging subsystem

### `SONGBIRD_DEBUG`
- **Default**: `false`
- **Type**: `bool`
- **Description**: Enable debug mode with verbose logging and diagnostics
- **Example**: `SONGBIRD_DEBUG=true`
- **Used by**: All services for debug features

---

## Timeouts & Limits

Timeout and resource limit configuration.

### `SONGBIRD_TIMEOUT_MS`
- **Default**: `5000` (5 seconds)
- **Type**: `u64`
- **Description**: Default operation timeout in milliseconds
- **Example**: `SONGBIRD_TIMEOUT_MS=10000`
- **Used by**: All async operations

### `SONGBIRD_CONNECTION_TIMEOUT`
- **Default**: `30` (seconds)
- **Type**: `u64`
- **Description**: Network connection timeout
- **Example**: `SONGBIRD_CONNECTION_TIMEOUT=60`
- **Used by**: Network clients

### `SONGBIRD_REQUEST_TIMEOUT`
- **Default**: `60` (seconds)
- **Type**: `u64`
- **Description**: HTTP/RPC request timeout
- **Example**: `SONGBIRD_REQUEST_TIMEOUT=120`
- **Used by**: HTTP/RPC clients

### `SONGBIRD_DISCOVERY_TIMEOUT_MS`
- **Default**: `5000` (5 seconds)
- **Type**: `u64`
- **Description**: Service discovery timeout
- **Example**: `SONGBIRD_DISCOVERY_TIMEOUT_MS=10000`
- **Used by**: Discovery clients

### `SONGBIRD_MAX_CONNECTIONS`
- **Default**: `1000`
- **Type**: `usize`
- **Description**: Maximum concurrent connections
- **Example**: `SONGBIRD_MAX_CONNECTIONS=2000`
- **Used by**: Server connection pooling

### `SONGBIRD_MAX_SERVICES`
- **Default**: `1000`
- **Type**: `usize`
- **Description**: Maximum registered services
- **Example**: `SONGBIRD_MAX_SERVICES=5000`
- **Used by**: Service registry

---

## Discovery & Federation

Service discovery and federation configuration.

### `SONGBIRD_ENABLE_DISCOVERY`
- **Default**: `true`
- **Type**: `bool`
- **Description**: Enable automatic service discovery
- **Example**: `SONGBIRD_ENABLE_DISCOVERY=true`
- **Used by**: Discovery system

### `SONGBIRD_CLUSTER_ID`
- **Default**: `"cluster_{pid}"`
- **Type**: `String`
- **Description**: Unique cluster identifier
- **Example**: `SONGBIRD_CLUSTER_ID=cluster_prod_01`
- **Used by**: Federation, multi-tower coordination

### `SONGBIRD_CLUSTER_NAME`
- **Default**: None
- **Type**: `String`
- **Description**: Human-readable cluster name
- **Example**: `SONGBIRD_CLUSTER_NAME=Production Cluster`
- **Used by**: Dashboard, monitoring

### `SONGBIRD_CLUSTER_ENDPOINTS`
- **Default**: Empty
- **Type**: Comma-separated list
- **Description**: Other cluster nodes for federation
- **Example**: `SONGBIRD_CLUSTER_ENDPOINTS=node1:8080,node2:8080,node3:8080`
- **Used by**: Federation service

---

## Security & TLS

Security and TLS configuration.

### `SONGBIRD_TLS_ENABLED`
- **Default**: `false`
- **Type**: `bool`
- **Description**: Enable TLS for all services
- **Example**: `SONGBIRD_TLS_ENABLED=true`
- **Used by**: All network services

### `SONGBIRD_VERIFY_CERTIFICATES`
- **Default**: `true`
- **Type**: `bool`
- **Description**: Verify TLS certificates
- **Example**: `SONGBIRD_VERIFY_CERTIFICATES=false`
- **Used by**: TLS clients (disable only for development)

### `BEARDOG_SECURITY_ENDPOINT`
- **Default**: `"https://localhost:8443"`
- **Type**: `String`
- **Description**: BearDog security service full endpoint
- **Example**: `BEARDOG_SECURITY_ENDPOINT=https://security.local:8443`
- **Used by**: Security clients

---

## Observability

Metrics, tracing, and health check configuration.

### `SONGBIRD_ENABLE_METRICS`
- **Default**: `true`
- **Type**: `bool`
- **Description**: Enable Prometheus metrics export
- **Example**: `SONGBIRD_ENABLE_METRICS=true`
- **Used by**: Metrics subsystem

### `SONGBIRD_ENABLE_HEALTH_CHECK`
- **Default**: `true`
- **Type**: `bool`
- **Description**: Enable health check endpoints
- **Example**: `SONGBIRD_ENABLE_HEALTH_CHECK=true`
- **Used by**: Health monitoring

### `SONGBIRD_TRACING_ENABLED`
- **Default**: `true`
- **Type**: `bool`
- **Description**: Enable distributed tracing
- **Example**: `SONGBIRD_TRACING_ENABLED=false`
- **Used by**: Tracing subsystem

---

## Advanced Configuration

Advanced and specialized configuration.

### Dynamic Service Ports

For any custom service, use the pattern:
```bash
SONGBIRD_{SERVICE}_PORT=<port>
```

**Example**:
```bash
SONGBIRD_CUSTOM_SERVICE_PORT=9000
```

### Dynamic Service Hosts

For any custom service host:
```bash
SONGBIRD_{SERVICE}_HOST=<host>
```

**Example**:
```bash
SONGBIRD_CUSTOM_SERVICE_HOST=custom.local
```

---

## 📖 **Usage Examples**

### Development Configuration

```bash
# Development environment
export SONGBIRD_ENVIRONMENT=development
export SONGBIRD_LOG_LEVEL=debug
export SONGBIRD_DEBUG=true
export SONGBIRD_BIND_ADDRESS=127.0.0.1
export SONGBIRD_HOST=127.0.0.1

# Start services
cargo run --bin songbird-orchestrator
```

### Production Configuration

```bash
# Production environment
export SONGBIRD_ENVIRONMENT=production
export SONGBIRD_LOG_LEVEL=info
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_HOST=0.0.0.0

# TLS enabled
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_SECURITY_PORT=8443

# Metrics
export SONGBIRD_ENABLE_METRICS=true
export SONGBIRD_METRICS_PORT=9090

# Start services
./songbird-orchestrator
```

### Docker Configuration

```dockerfile
ENV SONGBIRD_ENVIRONMENT=production
ENV SONGBIRD_ORCHESTRATOR_PORT=8080
ENV SONGBIRD_DISCOVERY_PORT=8081
ENV SONGBIRD_METRICS_PORT=9090
ENV SONGBIRD_BIND_ADDRESS=0.0.0.0
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: songbird-config
data:
  SONGBIRD_ENVIRONMENT: "production"
  SONGBIRD_ORCHESTRATOR_PORT: "8080"
  SONGBIRD_DISCOVERY_PORT: "8081"
  SONGBIRD_METRICS_PORT: "9090"
  SONGBIRD_BIND_ADDRESS: "0.0.0.0"
  CONSUL_PORT: "8500"
  ETCD_PORT: "2379"
```

### External Service Integration

```bash
# Consul integration
export CONSUL_PORT=8500
export SONGBIRD_ENABLE_DISCOVERY=true

# Etcd integration
export ETCD_PORT=2379

# ZooKeeper integration
export ZOOKEEPER_PORT=2181

# Eureka integration
export EUREKA_PORT=8761
```

---

## 🔧 **Configuration Best Practices**

### 1. Environment-Specific Configuration

**Development**:
- Use `localhost` / `127.0.0.1` for security
- Enable debug logging
- Disable TLS for easier testing

**Production**:
- Use `0.0.0.0` to bind to all interfaces
- Enable TLS and certificate verification
- Use `info` or `warn` log level
- Set appropriate timeouts and limits

### 2. Security Considerations

- **Never commit secrets to environment files**
- Use secret management (Vault, AWS Secrets Manager, etc.)
- Enable TLS in production (`SONGBIRD_TLS_ENABLED=true`)
- Verify certificates in production
- Use strong bind addresses

### 3. Performance Tuning

- Adjust `SONGBIRD_MAX_CONNECTIONS` based on load
- Tune timeout values for your network conditions
- Monitor metrics port (`SONGBIRD_METRICS_PORT`)
- Enable only needed features

### 4. Service Discovery

- Use external registries (Consul, etcd) for production
- Set appropriate discovery timeouts
- Configure cluster endpoints for federation
- Use unique cluster IDs

---

## 📊 **Environment Variable Summary**

### By Category

```
Core Service Ports:       9 variables
External Service Ports:   4 variables
Gaming Ports:             7 variables
Host Configuration:       5 variables
Environment & Runtime:    3 variables
Timeouts & Limits:        6 variables
Discovery & Federation:   4 variables
Security & TLS:           3 variables
Observability:            3 variables
Advanced:                 Dynamic patterns

Total:                    44+ environment variables
```

---

## 🔗 **Related Documentation**

- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Configuration Guide](./CONFIGURATION_GUIDE.md)
- [Architecture Overview](./ARCHITECTURE.md)
- [API Reference](./API_REFERENCE.md)

---

## 📝 **Version History**

- **0.2.0** (Nov 24, 2025): Comprehensive environment variable documentation
- **0.1.0** (Initial): Basic environment variables

---

**Last Updated**: November 24, 2025  
**Maintained by**: Songbird Team  
**License**: See LICENSE file

