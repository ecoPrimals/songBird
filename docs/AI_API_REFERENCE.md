# Songbird AI-Friendly API Reference

## Overview

Songbird is a universal network orchestration platform designed to be AI-friendly. This document provides comprehensive API documentation optimized for AI agents and programmatic usage.

## Authentication

All API endpoints support:
- API Key authentication via `X-API-Key` header
- Bearer token authentication via `Authorization: Bearer <token>`
- Environment variable authentication via `SONGBIRD_API_KEY`

## Base URLs

- Production: `https://api.songbird.network`
- Development: `http://localhost:8080`
- Auto-discovery: Use mDNS resolution for `_songbird._tcp.local`

## Core API Endpoints

### Service Management

#### GET /api/v1/services
List all registered services

**Parameters:**
- `status` (optional): Filter by service status (`running`, `stopped`, `failed`)
- `type` (optional): Filter by service type (`gaming`, `federation`, `storage`)
- `limit` (optional): Number of results (default: 50, max: 200)

**Response:**
```json
{
  "services": [
    {
      "id": "service-uuid",
      "name": "Gaming Bridge",
      "version": "1.0.0",
      "status": "running",
      "health": "healthy",
      "endpoint": "http://localhost:8080",
      "metadata": {
        "capabilities": ["gaming", "p2p"],
        "primal_assignments": ["toadstool", "nestgate"]
      }
    }
  ],
  "total": 1,
  "page": 1
}
```

#### POST /api/v1/services
Register a new service

**Request Body:**
```json
{
  "name": "My Service",
  "version": "1.0.0",
  "endpoint": "http://localhost:9000",
  "capabilities": ["storage", "compute"],
  "health_check": {
    "endpoint": "/health",
    "interval_secs": 30
  }
}
```

#### GET /api/v1/services/{service_id}
Get detailed service information

#### DELETE /api/v1/services/{service_id}
Unregister a service

### Universal Primal Coordination

#### POST /api/v1/coordinate
Universal coordination endpoint that works with any Primal

**Request Body:**
```json
{
  "primal_name": "toadstool",
  "capabilities": ["orchestration", "deployment"],
  "manifest": {
    "metadata": {
      "name": "my-biome",
      "version": "1.0.0"
    },
    "services": {
      "web-server": {
        "endpoint": "http://localhost:8080",
        "depends_on": ["database"]
      }
    }
  }
}
```

**Response:**
```json
{
  "coordination_id": "coord-uuid",
  "status": "success",
  "primal_response": {
    "endpoints": ["http://toadstool:8080/api"],
    "capabilities_matched": ["orchestration"]
  }
}
```

#### GET /api/v1/coordinate/primals
List available Primals and their capabilities

#### POST /api/v1/coordinate/all
Coordinate with all available Primals

### Gaming Bridge Management

#### POST /api/v1/gaming/sessions
Create a gaming session

**Request Body:**
```json
{
  "game_name": "Minecraft",
  "max_players": 10,
  "port_range": {
    "start": 25565,
    "end": 25575
  }
}
```

#### GET /api/v1/gaming/sessions
List active gaming sessions

#### GET /api/v1/gaming/sessions/{session_id}
Get session details

#### DELETE /api/v1/gaming/sessions/{session_id}
Close a gaming session

### Federation Management

#### GET /api/v1/federation/status
Get federation status

**Response:**
```json
{
  "mode": "hybrid",
  "connected_nodes": 5,
  "federation_health": 0.95,
  "local_node_id": "node-uuid"
}
```

#### POST /api/v1/federation/discover
Discover federated services

#### POST /api/v1/federation/connect
Connect to a federation node

### BYOB (Bring Your Own Biome) Deployment

#### POST /api/v1/byob/deploy
Deploy a biome manifest

**Request Body:**
```json
{
  "team_id": "team-uuid",
  "manifest": {
    "metadata": {
      "name": "my-biome",
      "version": "1.0.0"
    },
    "services": {
      "web-server": {
        "endpoint": "http://localhost:8080"
      }
    },
    "primals": {
      "toadstool": {
        "enabled": true,
        "capabilities": ["orchestration"]
      }
    }
  }
}
```

#### GET /api/v1/byob/deployments
List active deployments

#### GET /api/v1/byob/deployments/{deployment_id}
Get deployment status

#### DELETE /api/v1/byob/deployments/{deployment_id}
Stop a deployment

### Health and Monitoring

#### GET /api/v1/health
System health check

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "components": {
    "gaming": "healthy",
    "federation": "healthy",
    "storage": "degraded"
  }
}
```

#### GET /api/v1/metrics
Prometheus-compatible metrics

#### GET /api/v1/status
Detailed system status

## CLI Reference

### Installation

```bash
# Install from crates.io
cargo install songbird-cli

# Or use the binary
curl -sSL https://get.songbird.network/install.sh | sh
```

### Basic Usage

```bash
# Initialize configuration
songbird init

# Quick start gaming bridge
songbird quick gaming

# Deploy a biome
songbird compose deploy biome.yaml

# Check status
songbird status

# Show version
songbird version
```

### Command Categories

#### Quick Commands
- `songbird quick gaming` - Start gaming bridge
- `songbird quick federation` - Join federation
- `songbird quick storage` - Setup storage
- `songbird quick ai` - Enable AI features

#### Discovery Commands
- `songbird discovery scan` - Scan for services
- `songbird discovery peers` - Find peers
- `songbird discovery primals` - Find Primals

#### Federation Commands
- `songbird federation join <node>` - Join federation
- `songbird federation leave` - Leave federation
- `songbird federation status` - Show federation status

#### Service Commands
- `songbird service list` - List services
- `songbird service start <service>` - Start service
- `songbird service stop <service>` - Stop service
- `songbird service logs <service>` - View logs

#### Composition Commands
- `songbird compose deploy <manifest>` - Deploy biome
- `songbird compose scale <service> <replicas>` - Scale service
- `songbird compose stop <deployment>` - Stop deployment

### AI Integration Examples

#### Python Example
```python
import requests
import json

# Connect to Songbird
api_base = "http://localhost:8080"
api_key = "your-api-key"

headers = {
    "X-API-Key": api_key,
    "Content-Type": "application/json"
}

# Create gaming session
session_data = {
    "game_name": "Minecraft",
    "max_players": 10
}

response = requests.post(
    f"{api_base}/api/v1/gaming/sessions",
    headers=headers,
    json=session_data
)

if response.status_code == 201:
    session = response.json()
    print(f"Created session: {session['id']}")
```

#### cURL Example
```bash
# Create gaming session
curl -X POST http://localhost:8080/api/v1/gaming/sessions \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "game_name": "Minecraft",
    "max_players": 10
  }'

# List services
curl -X GET http://localhost:8080/api/v1/services \
  -H "X-API-Key: your-api-key"

# Deploy biome
curl -X POST http://localhost:8080/api/v1/byob/deploy \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d @biome.json
```

## Error Handling

All API endpoints return consistent error responses:

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Missing required field: game_name",
    "details": {
      "field": "game_name",
      "expected": "string"
    }
  }
}
```

### Common Error Codes
- `INVALID_REQUEST` - Malformed request
- `UNAUTHORIZED` - Authentication required
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `CONFLICT` - Resource already exists
- `INTERNAL_ERROR` - Server error

## Rate Limiting

- Standard endpoints: 100 requests/minute
- Health endpoints: 1000 requests/minute
- Administrative endpoints: 10 requests/minute

Rate limit headers:
- `X-RateLimit-Limit`: Request limit
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Reset time (Unix timestamp)

## WebSocket API

For real-time updates, connect to:
- `ws://localhost:8080/api/v1/ws`

### Event Types
- `service_status_changed`
- `gaming_session_created`
- `gaming_session_closed`
- `federation_node_joined`
- `federation_node_left`

### Example WebSocket Message
```json
{
  "type": "service_status_changed",
  "data": {
    "service_id": "service-uuid",
    "old_status": "running",
    "new_status": "stopped",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

## SDKs and Libraries

### Official SDKs
- **Rust**: `songbird-sdk` crate
- **Python**: `songbird-python` package
- **Node.js**: `@songbird/sdk` package

### Community SDKs
- **Go**: `github.com/songbird-network/go-sdk`
- **Java**: `com.songbird:songbird-sdk`

## Best Practices for AI Integration

1. **Use API Keys**: Always authenticate with API keys
2. **Handle Rate Limits**: Implement exponential backoff
3. **Monitor Health**: Check `/health` endpoint regularly
4. **Use WebSockets**: For real-time updates
5. **Batch Operations**: Use batch endpoints when available
6. **Error Handling**: Always check error responses
7. **Versioning**: Specify API version in requests

## Configuration

### Environment Variables
```bash
# API Configuration
SONGBIRD_API_HOST=localhost
SONGBIRD_API_PORT=8080
SONGBIRD_API_KEY=your-api-key

# Discovery Configuration
SONGBIRD_DISCOVERY_ENABLED=true
SONGBIRD_DISCOVERY_PORTS=8080,8081,8082
SONGBIRD_DISCOVERY_TIMEOUT_MS=500

# Gaming Configuration
SONGBIRD_GAMING_ENABLED=true
SONGBIRD_GAMING_PORT_RANGE=25565-25575

# Federation Configuration
SONGBIRD_FEDERATION_MODE=hybrid
SONGBIRD_FEDERATION_DISCOVERY=true
```

### Configuration File (songbird.toml)
```toml
[api]
host = "localhost"
port = 8080
key = "your-api-key"

[gaming]
enabled = true
port_range = "25565-25575"

[federation]
mode = "hybrid"
discovery = true

[primals]
toadstool = { enabled = true, endpoint = "http://toadstool:8080" }
nestgate = { enabled = true, endpoint = "http://nestgate:8080" }
```

## Troubleshooting

### Common Issues
1. **Connection refused**: Check if Songbird is running
2. **Authentication failed**: Verify API key
3. **Service not found**: Check service registration
4. **Port conflicts**: Verify port availability

### Debug Mode
```bash
RUST_LOG=debug songbird start
```

### Health Checks
```bash
# Check system health
curl http://localhost:8080/api/v1/health

# Check specific service
curl http://localhost:8080/api/v1/services/service-id
```

## Support

- **Documentation**: https://docs.songbird.network
- **Community**: https://discord.gg/songbird
- **Issues**: https://github.com/songbird-network/songbird/issues
- **API Support**: api-support@songbird.network 