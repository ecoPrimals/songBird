# Custom Primal Development Guide

## Making Your Primal Work with Songbird's Universal Adapter

Songbird's universal adapter allows any custom or forked Primal to seamlessly integrate with the biomeOS ecosystem. This guide shows you how to make your Primal compatible.

## Universal API Specification

### Standard Coordination Endpoint

Your Primal should implement at least one of these endpoints:

```
POST /api/v1/coordinate      # Universal coordination (recommended)
POST /api/v1/orchestrate     # For compute Primals
POST /api/v1/provision       # For storage Primals  
POST /api/v1/authenticate    # For security Primals
POST /api/v1/deploy-agents   # For AI/ML Primals
```

### Request Format

Songbird sends a standardized payload:

```json
{
  "coordination_request": {
    "from": "songbird",
    "to": "your-primal-name",
    "manifest": {
      "metadata": { "name": "biome-name", "version": "1.0.0" },
      "services": { /* service definitions */ },
      "networking": { /* network config */ }
    },
    "capabilities_requested": ["compute", "custom-feature"],
    "api_version": "universal/v1",
    "timestamp": "2024-01-01T00:00:00Z"
  },
  "songbird_context": {
    "orchestrator_id": "songbird-123",
    "biome_name": "my-biome",
    "services_count": 3,
    "networking_enabled": true
  }
}
```

### Response Format

Your Primal should respond with:

```json
{
  "coordination_response": {
    "status": "success",
    "primal_name": "your-primal-name", 
    "capabilities_provided": ["compute", "custom-feature"],
    "endpoints": {
      "primary": "http://your-primal:8080",
      "management": "http://your-primal:8081/admin"
    },
    "coordination_id": "coord-456",
    "expires_at": "2024-01-01T01:00:00Z"
  },
  "primal_context": {
    "resources_allocated": true,
    "additional_info": "Custom data your Primal wants to share"
  }
}
```

## Implementation Examples

### Minimal Custom Primal (Python)

```python
from flask import Flask, request, jsonify
import datetime

app = Flask(__name__)

@app.route('/api/v1/coordinate', methods=['POST'])
def coordinate():
    data = request.json
    coordination_req = data['coordination_request']
    
    # Process the coordination request
    manifest = coordination_req['manifest']
    capabilities = coordination_req['capabilities_requested']
    
    # Your custom logic here
    result = process_coordination(manifest, capabilities)
    
    return jsonify({
        "coordination_response": {
            "status": "success",
            "primal_name": "my-custom-primal",
            "capabilities_provided": capabilities,
            "endpoints": {
                "primary": "http://localhost:9000"
            },
            "coordination_id": f"coord-{datetime.datetime.now().timestamp()}",
            "expires_at": (datetime.datetime.now() + datetime.timedelta(hours=1)).isoformat()
        },
        "primal_context": {
            "resources_allocated": True,
            "custom_feature_enabled": True
        }
    })

def process_coordination(manifest, capabilities):
    # Implement your Primal's specific logic
    return {"status": "processed"}

if __name__ == '__main__':
    app.run(port=9000)
```

### Rust Custom Primal (using Axum)

```rust
use axum::{Json, extract::State, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct CoordinationRequest {
    coordination_request: SongbirdRequest,
    songbird_context: SongbirdContext,
}

#[derive(Serialize)]
struct CoordinationResponse {
    coordination_response: PrimalResponse,
    primal_context: HashMap<String, serde_json::Value>,
}

async fn coordinate_handler(
    Json(payload): Json<CoordinationRequest>
) -> Result<ResponseJson<CoordinationResponse>, StatusCode> {
    
    // Process the coordination request
    let manifest = &payload.coordination_request.manifest;
    let capabilities = &payload.coordination_request.capabilities_requested;
    
    // Your custom Primal logic here
    let result = process_coordination(manifest, capabilities).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response = CoordinationResponse {
        coordination_response: PrimalResponse {
            status: "success".to_string(),
            primal_name: "my-rust-primal".to_string(),
            capabilities_provided: capabilities.clone(),
            endpoints: {
                let mut endpoints = HashMap::new();
                endpoints.insert("primary".to_string(), "http://localhost:9001".to_string());
                endpoints
            },
            coordination_id: format!("coord-{}", uuid::Uuid::new_v4()),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        },
        primal_context: result,
    };
    
    Ok(ResponseJson(response))
}
```

## Configuration in biome.yaml

To use your custom Primal, add it to the `primals` section:

```yaml
primals:
  my-custom-primal:
    enabled: true
    endpoint: "http://localhost:9000"
    capabilities: ["compute", "blockchain", "custom-feature"]
```

## Capability-Based Routing

Songbird automatically routes to the appropriate endpoint based on capabilities:

- `compute`, `execution` → `/api/v1/orchestrate`
- `storage`, `data` → `/api/v1/provision`
- `security`, `authentication` → `/api/v1/authenticate`
- `ai`, `ml`, `agents` → `/api/v1/deploy-agents`
- `custom` or unknown → `/api/v1/coordinate`

## Best Practices

1. **Fail Gracefully**: Songbird continues if your Primal is unavailable
2. **Standard Responses**: Use the universal response format for consistency
3. **Capability Declaration**: Clearly declare what your Primal can do
4. **Endpoint Health**: Implement health checks at `/health`
5. **Documentation**: Document your custom capabilities
6. **Backward Compatibility**: Support both universal and legacy APIs during transition

## Testing Your Primal

Create a test biome.yaml:

```yaml
metadata:
  name: "test-custom-primal"
  version: "1.0.0"

services:
  test-service:
    endpoint: "http://localhost:8080"

primals:
  your-custom-primal:
    enabled: true
    endpoint: "http://localhost:9000"  
    capabilities: ["your-capabilities"]
```

Deploy with Songbird:

```bash
songbird deploy test-biome.yaml
```

## Community

- Share your custom Primals in the biomeOS community
- Follow semantic versioning for your Primal APIs
- Consider open-sourcing successful custom Primals
- Join the Primal developer community for support

The universal adapter makes the biomeOS ecosystem truly extensible - anyone can create a Primal that works seamlessly with the entire ecosystem! 