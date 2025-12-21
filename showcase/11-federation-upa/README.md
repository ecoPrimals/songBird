# Showcase 11: Federation UPA Testing

## Overview

This showcase demonstrates the Universal Port Authority operating across a federated network of Songbird instances. It validates that the Enhanced Capability Router works seamlessly across multiple towers.

## Prerequisites

**All towers must have:**
1. Latest Songbird build with Enhanced Router
2. Songbird running (`./target/release/songbird-orchestrator`)
3. Federation discovery operational
4. Network connectivity between towers

## Quick Start

### Deploy to All Towers

**Option 1: Manual deployment (recommended)**
```bash
# On each tower (Eastgate, Westgate, Strandgate):
cd ~/Development/ecoPrimals/songbird
git pull
cargo build --release
./scripts/federation-deploy.sh restart
```

**Option 2: Use deployment script**
```bash
# On Eastgate (or any tower):
./scripts/federation-deploy.sh deploy
```

### Run Federation Tests

```bash
# Set tower URLs (adjust IPs as needed)
export EASTGATE_URL="https://192.168.1.10:8080"
export WESTGATE_URL="https://192.168.1.20:8080"
export STRANDGATE_URL="https://192.168.1.30:8080"

# Or use localhost if testing from each tower
export EASTGATE_URL="https://localhost:8080"
export WESTGATE_URL="https://localhost:8080"
export STRANDGATE_URL="https://localhost:8080"

# Run comprehensive federation test
./showcase/11-federation-upa/01-federation-test.sh
```

## Test Scenarios

### 1. Tower Health Check
Verifies all towers are running and responding to health checks.

### 2. Federation Discovery
Confirms each tower can see other towers in the federation.

### 3. Service Registration
Registers a "Toadstool" compute service on each tower via Universal Port Authority.

### 4. Service Query
Queries registered services from each tower to verify local and remote visibility.

### 5. Task Routing
Submits compute tasks to each tower and verifies they route to local services first.

### 6. Capability Query
Tests capability-based service discovery across the federation.

### 7. Load Distribution
Submits multiple tasks and measures distribution across services.

## Expected Results

**Successful Federation:**
- ✅ All towers healthy
- ✅ Each tower sees 3 nodes (self + 2 peers)
- ✅ Services registered on each tower
- ✅ Tasks route to local services (Priority 1: UPA)
- ✅ Capability queries return all matching services
- ✅ Load distributed intelligently

**Routing Priority Validation:**
1. **Priority 1 (UPA):** Tasks route to registered local services
2. **Priority 2 (Federation):** If local unavailable, route to peer Songbird
3. **Priority 3 (Static):** Last resort fallback

## Troubleshooting

### Tower Not Responding
```bash
# Check if Songbird is running
ps aux | grep songbird

# Check logs
tail -f ~/.songbird/songbird.log

# Restart
./scripts/federation-deploy.sh restart
```

### Federation Not Discovering
```bash
# Check UDP broadcast (port 8888)
sudo netstat -ulnp | grep 8888

# Check firewall
sudo ufw status

# Verify network interfaces
ip addr show
```

### Services Not Registering
```bash
# Check service registry API
curl -sk https://localhost:8080/api/v1/info | jq

# Verify endpoints are available
curl -sk https://localhost:8080/api/v1/services | jq
```

## Manual Testing

### Register a Service
```bash
curl -sk -X POST https://localhost:8080/api/v1/services/register \
  -H "Content-Type: application/json" \
  -d '{
    "service_info": {
      "name": "TestService",
      "version": "1.0.0",
      "capabilities": [
        {
          "name": "compute",
          "version": "1.0.0",
          "protocols": ["http"]
        }
      ]
    },
    "preferred_port": 9000
  }' | jq
```

### Query Services
```bash
# All services
curl -sk https://localhost:8080/api/v1/services | jq

# By capability
curl -sk https://localhost:8080/api/v1/services/query/compute | jq
```

### Submit Task
```bash
curl -sk -X POST https://localhost:8080/api/v1/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "ml_training",
      "metadata": {"requires_gpu": "true"}
    }
  }' | jq
```

### Check Federation
```bash
curl -sk https://localhost:8080/api/v1/federation/nodes | jq
```

## Architecture Validation

This showcase validates:
- ✅ Enhanced Capability Router across federation
- ✅ Universal Port Authority with dynamic registration
- ✅ Priority-based routing (UPA → Peer → Static)
- ✅ Capability-based service discovery
- ✅ Load distribution across towers
- ✅ Backward compatibility maintained

## Performance Benchmarks

**Expected Performance:**
- Service registration: < 10ms
- Service query: < 5ms
- Task routing decision: < 2ms
- Cross-tower latency: < 100ms (depends on network)

## Files

- `01-federation-test.sh` - Comprehensive federation test suite
- `README.md` - This file

## Next Steps

After successful federation testing:
1. Deploy actual Toadstool instances
2. Test real compute workloads
3. Benchmark performance under load
4. Monitor federation health
5. Production deployment

---

*ecoPrimals - Universal Port Authority - Phase 5*

