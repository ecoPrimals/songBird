# Songbird Universal Network Orchestrator API Reference

## Overview

The Songbird Universal Network Orchestrator provides comprehensive REST and WebSocket APIs for service orchestration, management, and coordination. This document covers all public APIs, request/response formats, and integration patterns.

## API Endpoints

Base URL: `http://localhost:8080/api/v1`

All API responses follow this standard format:

```json
{
  "success": true,
  "data": { /* response data */ },
  "message": "Operation completed successfully",
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "req-12345"
}
```

## Authentication

For production deployments with authentication enabled:

```bash
# Get authentication token
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "your-password"}'

# Use token in subsequent requests
curl -X GET http://localhost:8080/api/v1/services \
  -H "Authorization: Bearer your-jwt-token"
```

## Health and System APIs

### System Health Check

#### `GET /health`
Basic health check endpoint.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "uptime": "72h30m15s",
    "version": "1.0.0"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `GET /health/detailed`
Comprehensive health check with component status.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "components": {
      "orchestrator": "healthy",
      "service_registry": "healthy",
      "load_balancer": "healthy",
      "communication": "healthy",
      "primals": "healthy"
    },
    "services": {
      "total": 25,
      "healthy": 24,
      "unhealthy": 1
    },
    "performance": {
      "avg_response_time": "2.5ms",
      "requests_per_second": 1250,
      "active_connections": 150
    }
  }
}
```

### System Information

#### `GET /system/info`
System information and configuration.

**Response:**
```json
{
  "success": true,
  "data": {
    "name": "Songbird Universal Network Orchestrator",
    "version": "1.0.0",
    "build": "release-2024.01.01",
    "uptime_seconds": 261015,
    "configuration": {
      "max_services": 1000,
      "auto_discovery": true,
      "primals_enabled": true,
      "gaming_enabled": true
    },
    "capabilities": [
      "service-orchestration",
      "primal-coordination",
      "gaming-bridge",
      "byob-deployment"
    ]
  }
}
```

#### `GET /system/metrics`
System-level metrics and performance data.

**Response:**
```json
{
  "success": true,
  "data": {
    "performance": {
      "hashmap_ops_per_sec": 2500000,
      "coordination_latency_ms": 0.8,
      "memory_usage_mb": 256,
      "cpu_usage_percent": 15.2
    },
    "services": {
      "total_services": 25,
      "active_services": 24,
      "total_requests": 1000000,
      "avg_response_time_ms": 2.5
    },
    "connections": {
      "active_websockets": 150,
      "active_http": 50,
      "total_connections": 200
    }
  }
}
```

## Service Management APIs

### Service Operations

#### `GET /services`
List all registered services.

**Query Parameters:**
- `status` (optional): Filter by status (`healthy`, `unhealthy`, `starting`, `stopped`)
- `type` (optional): Filter by service type (`web-api`, `microservice`, `gaming`, etc.)
- `tag` (optional): Filter by tag
- `limit` (optional): Maximum number of results (default: 100)
- `offset` (optional): Pagination offset (default: 0)

**Response:**
```json
{
  "success": true,
  "data": {
    "services": [
      {
        "id": "service-12345",
        "name": "my-web-service",
        "type": "web-api",
        "version": "1.0.0",
        "status": "healthy",
        "endpoints": [
          {
            "protocol": "http",
            "address": "192.168.1.100:3000",
            "health_check": "/health"
          }
        ],
        "metadata": {
          "replicas": 2,
          "deployed_at": "2024-01-01T00:00:00Z",
          "last_health_check": "2024-01-01T12:00:00Z"
        }
      }
    ],
    "total": 25,
    "offset": 0,
    "limit": 100
  }
}
```

#### `POST /services`
Deploy a new service using BYOB manifest.

**Request Body:**
```json
{
  "manifest": {
    "apiVersion": "v1",
    "kind": "Service",
    "metadata": {
      "name": "my-web-service",
      "description": "My example web service",
      "version": "1.0.0"
    },
    "spec": {
      "type": "web-api",
      "port": 3000,
      "healthCheck": {
        "path": "/health",
        "interval": "30s",
        "timeout": "10s"
      },
      "deployment": {
        "replicas": 2,
        "resources": {
          "cpu": "500m",
          "memory": "512Mi"
        }
      }
    }
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "service_id": "service-12345",
    "name": "my-web-service",
    "status": "deploying",
    "estimated_ready_time": "2024-01-01T12:02:00Z"
  }
}
```

#### `GET /services/{service_id}`
Get detailed information about a specific service.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "service-12345",
    "name": "my-web-service",
    "type": "web-api",
    "version": "1.0.0",
    "status": "healthy",
    "configuration": {
      "replicas": 2,
      "resources": {
        "cpu": "500m",
        "memory": "512Mi"
      }
    },
    "endpoints": [
      {
        "protocol": "http",
        "address": "192.168.1.100:3000",
        "health_check": "/health",
        "status": "healthy"
      }
    ],
    "metrics": {
      "requests_per_second": 45,
      "avg_response_time_ms": 12.5,
      "error_rate": 0.02
    }
  }
}
```

#### `PUT /services/{service_id}`
Update service configuration.

**Request Body:**
```json
{
  "replicas": 3,
  "resources": {
    "cpu": "1000m",
    "memory": "1Gi"
  }
}
```

#### `DELETE /services/{service_id}`
Stop and remove a service.

**Response:**
```json
{
  "success": true,
  "data": {
    "service_id": "service-12345",
    "status": "stopping",
    "estimated_stopped_time": "2024-01-01T12:01:00Z"
  }
}
```

### Service Health and Metrics

#### `GET /services/{service_id}/health`
Get health status of a specific service.

**Response:**
```json
{
  "success": true,
  "data": {
    "service_id": "service-12345",
    "status": "healthy",
    "last_check": "2024-01-01T12:00:00Z",
    "health_checks": [
      {
        "type": "http",
        "endpoint": "/health",
        "status": "healthy",
        "response_time_ms": 5,
        "last_success": "2024-01-01T12:00:00Z"
      }
    ],
    "uptime": "72h30m15s"
  }
}
```

#### `GET /services/{service_id}/metrics`
Get metrics for a specific service.

**Response:**
```json
{
  "success": true,
  "data": {
    "service_id": "service-12345",
    "metrics": {
      "requests": {
        "total": 50000,
        "per_second": 45,
        "avg_response_time_ms": 12.5
      },
      "errors": {
        "total": 100,
        "rate": 0.02,
        "last_error": "2024-01-01T11:45:00Z"
      },
      "resources": {
        "cpu_usage_percent": 25.5,
        "memory_usage_mb": 256,
        "network_bytes_in": 1000000,
        "network_bytes_out": 2000000
      }
    },
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

#### `GET /services/{service_id}/logs`
Get service logs.

**Query Parameters:**
- `lines` (optional): Number of lines to return (default: 100)
- `since` (optional): ISO timestamp to get logs since
- `level` (optional): Log level filter (`debug`, `info`, `warn`, `error`)

**Response:**
```json
{
  "success": true,
  "data": {
    "service_id": "service-12345",
    "logs": [
      {
        "timestamp": "2024-01-01T12:00:00Z",
        "level": "info",
        "message": "Service started successfully",
        "metadata": {
          "component": "main",
          "thread": "main"
        }
      }
    ],
    "total_lines": 1000,
    "lines_returned": 100
  }
}
```

## Primal Coordination APIs

### Primal Management

#### `GET /primals`
List all configured Primals.

**Response:**
```json
{
  "success": true,
  "data": {
    "primals": [
      {
        "name": "toadstool",
        "endpoint": "https://toadstool.example.com",
        "status": "healthy",
        "capabilities": ["data-processing", "analytics"],
        "last_check": "2024-01-01T12:00:00Z",
        "response_time_ms": 25
      },
      {
        "name": "nestgate",
        "endpoint": "https://nestgate.example.com",
        "status": "healthy",
        "capabilities": ["communication", "messaging"],
        "last_check": "2024-01-01T12:00:00Z",
        "response_time_ms": 18
      }
    ]
  }
}
```

#### `POST /primals`
Register a new Primal.

**Request Body:**
```json
{
  "name": "toadstool",
  "endpoint": "https://toadstool.example.com",
  "api_key": "your-api-key",
  "capabilities": ["data-processing", "analytics"],
  "configuration": {
    "timeout": "30s",
    "retry_attempts": 3
  }
}
```

#### `GET /primals/{primal_name}`
Get detailed information about a specific Primal.

#### `PUT /primals/{primal_name}`
Update Primal configuration.

#### `DELETE /primals/{primal_name}`
Remove a Primal.

### Primal Operations

#### `POST /primals/{primal_name}/execute`
Execute an operation on a specific Primal.

**Request Body:**
```json
{
  "operation": "process_data",
  "data": {
    "input": "data-to-process",
    "options": {
      "format": "json",
      "timeout": "60s"
    }
  },
  "metadata": {
    "request_id": "req-12345",
    "priority": "high"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "primal": "toadstool",
    "operation": "process_data",
    "result": {
      "output": "processed-data",
      "status": "completed"
    },
    "processing_time_ms": 150,
    "metadata": {
      "request_id": "req-12345",
      "primal_request_id": "primal-req-67890"
    }
  }
}
```

#### `POST /primals/coordinate`
Coordinate operations across multiple Primals.

**Request Body:**
```json
{
  "primals": ["toadstool", "nestgate"],
  "operation": "sync_data",
  "data": {
    "source": "service-12345",
    "destination": "service-67890"
  },
  "coordination_type": "sequential"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "coordination_id": "coord-12345",
    "status": "completed",
    "results": [
      {
        "primal": "toadstool",
        "status": "success",
        "result": { "processed": true }
      },
      {
        "primal": "nestgate",
        "status": "success",
        "result": { "notified": true }
      }
    ],
    "total_time_ms": 300
  }
}
```

#### `GET /primals/{primal_name}/health`
Check health of a specific Primal.

#### `GET /primals/{primal_name}/metrics`
Get metrics for a specific Primal.

## Gaming Bridge APIs

### Gaming Session Management

#### `GET /gaming/sessions`
List active gaming sessions.

**Response:**
```json
{
  "success": true,
  "data": {
    "sessions": [
      {
        "id": "session-12345",
        "game": "quake",
        "protocol": "IPX",
        "host": "192.168.1.100",
        "port": 26000,
        "players": 4,
        "max_players": 8,
        "status": "active",
        "created_at": "2024-01-01T12:00:00Z"
      }
    ],
    "total": 5
  }
}
```

#### `POST /gaming/sessions`
Create a new gaming session.

**Request Body:**
```json
{
  "game": "quake",
  "protocol": "IPX",
  "max_players": 8,
  "settings": {
    "map": "dm1",
    "fraglimit": 50,
    "timelimit": 30
  }
}
```

#### `GET /gaming/sessions/{session_id}`
Get details of a specific gaming session.

#### `DELETE /gaming/sessions/{session_id}`
End a gaming session.

### Gaming Discovery

#### `GET /gaming/discovery`
List discoverable games on the network.

#### `POST /gaming/discovery/scan`
Initiate a network scan for games.

#### `GET /gaming/discovery/protocols`
List supported gaming protocols.

## Load Balancer APIs

### Load Balancer Configuration

#### `GET /load-balancer/config`
Get current load balancer configuration.

#### `PUT /load-balancer/config`
Update load balancer settings.

**Request Body:**
```json
{
  "default_algorithm": "health-aware",
  "health_required": true,
  "max_retries": 3,
  "retry_delay": "1s",
  "algorithms": {
    "round-robin": { "enabled": true },
    "weighted-round-robin": { "enabled": true },
    "least-connections": { "enabled": true },
    "health-aware": { "enabled": true }
  }
}
```

#### `GET /load-balancer/stats`
Get load balancer statistics.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_requests": 1000000,
    "requests_per_second": 250,
    "avg_response_time_ms": 12.5,
    "algorithms": {
      "round-robin": {
        "requests": 400000,
        "avg_response_time_ms": 10.2
      },
      "health-aware": {
        "requests": 600000,
        "avg_response_time_ms": 14.1
      }
    },
    "endpoints": [
      {
        "address": "192.168.1.100:3000",
        "requests": 250000,
        "avg_response_time_ms": 11.5,
        "health": "healthy"
      }
    ]
  }
}
```

## WebSocket API

### Connection

Connect to the WebSocket endpoint:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### Message Format

All WebSocket messages use this format:

```json
{
  "type": "message_type",
  "data": { /* message data */ },
  "timestamp": "2024-01-01T12:00:00Z",
  "request_id": "req-12345"
}
```

### Subscriptions

#### Subscribe to Service Events

```json
{
  "type": "subscribe",
  "channel": "services",
  "filters": {
    "service_id": "service-12345",
    "event_types": ["health_change", "status_change"]
  }
}
```

#### Subscribe to Primal Events

```json
{
  "type": "subscribe",
  "channel": "primals",
  "filters": {
    "primal_name": "toadstool",
    "event_types": ["operation_complete", "health_change"]
  }
}
```

#### Subscribe to Gaming Events

```json
{
  "type": "subscribe",
  "channel": "gaming",
  "filters": {
    "event_types": ["session_created", "session_ended", "player_joined"]
  }
}
```

### Real-time Operations

#### Real-time Service Management

```json
{
  "type": "service_operation",
  "operation": "scale",
  "service_id": "service-12345",
  "data": {
    "replicas": 3
  }
}
```

#### Real-time Primal Coordination

```json
{
  "type": "primal_operation",
  "primal": "toadstool",
  "operation": "process_data",
  "data": {
    "input": "real-time-data"
  }
}
```

## Error Handling

### Standard Error Response

```json
{
  "success": false,
  "error": {
    "code": "SERVICE_NOT_FOUND",
    "message": "Service with ID 'service-12345' not found",
    "details": {
      "service_id": "service-12345",
      "suggestion": "Check service ID and try again"
    }
  },
  "timestamp": "2024-01-01T12:00:00Z",
  "request_id": "req-12345"
}
```

### Common Error Codes

- `SERVICE_NOT_FOUND` - Requested service doesn't exist
- `PRIMAL_UNAVAILABLE` - Primal endpoint is not reachable
- `INVALID_MANIFEST` - BYOB manifest validation failed
- `INSUFFICIENT_RESOURCES` - Not enough resources to deploy service
- `AUTHENTICATION_REQUIRED` - Authentication token required
- `AUTHORIZATION_FAILED` - Insufficient permissions
- `RATE_LIMIT_EXCEEDED` - Too many requests
- `INTERNAL_ERROR` - Internal server error

## Rate Limiting

The API implements rate limiting with the following headers:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459200
```

Default limits:
- **Authenticated users**: 10,000 requests per hour
- **Unauthenticated users**: 1,000 requests per hour
- **WebSocket connections**: 100 per IP address

## API Versioning

The API uses URL path versioning:

- **Current version**: `/api/v1/`
- **Beta features**: `/api/v1/beta/`
- **Deprecated endpoints**: Include `X-Deprecated` header

## SDK and Client Libraries

Official SDKs are available for:

- **JavaScript/TypeScript**: `npm install @songbird/sdk`
- **Python**: `pip install songbird-sdk`
- **Rust**: `cargo add songbird-sdk`
- **Go**: `go get github.com/songbird/sdk-go`

Example usage:

```javascript
// JavaScript SDK
import { SongbirdClient } from '@songbird/sdk';

const client = new SongbirdClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key'
});

// Deploy service
const service = await client.services.deploy({
  name: 'my-service',
  manifest: require('./service-manifest.json')
});

// Coordinate with Primal
const result = await client.primals.execute('toadstool', {
  operation: 'process_data',
  data: { input: 'data' }
});
```

For detailed examples and integration guides, see the [Getting Started Guide](GETTING_STARTED.md). 