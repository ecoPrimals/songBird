# E2E Testing Guide - Unix Socket IPC

**Version**: v3.19.3  
**Date**: January 8, 2026

---

## Overview

End-to-end tests for the Unix socket JSON-RPC IPC server. These tests verify that the server can accept real client connections and correctly handle API calls.

---

## Running E2E Tests

### Step 1: Start Songbird Server

```bash
# Set node ID
export SONGBIRD_NODE_ID="test"

# Start orchestrator
cargo run --package songbird-orchestrator

# Should see:
# ✅ Unix Socket IPC server started successfully
# Socket: /tmp/songbird-test.sock
```

### Step 2: Run E2E Tests

```bash
# Run all E2E tests (in another terminal)
cargo test --test e2e_unix_socket_ipc -- --ignored --nocapture

# Run specific test
cargo test --test e2e_unix_socket_ipc test_discover_by_family_api -- --ignored --nocapture
```

---

## Test Coverage

### 1. Connection Test ✅
**File**: `test_unix_socket_connection`  
**Tests**: Basic socket connection

```bash
cargo test --test e2e_unix_socket_ipc test_unix_socket_connection -- --ignored --nocapture
```

### 2. discover_by_family API ✅
**File**: `test_discover_by_family_api`  
**Tests**: Family-based peer discovery

```bash
cargo test --test e2e_unix_socket_ipc test_discover_by_family_api -- --ignored --nocapture
```

**Expected**: Returns list of discovered nodes filtered by family tags

### 3. create_genetic_tunnel API ✅
**File**: `test_create_genetic_tunnel_api`  
**Tests**: BTSP tunnel creation with genetic proof

```bash
cargo test --test e2e_unix_socket_ipc test_create_genetic_tunnel_api -- --ignored --nocapture
```

**Expected**: Returns tunnel_id and status

### 4. announce_capabilities API ✅
**File**: `test_announce_capabilities_api`  
**Tests**: Capability announcement

```bash
cargo test --test e2e_unix_socket_ipc test_announce_capabilities_api -- --ignored --nocapture
```

**Expected**: Returns status and broadcasting flag

### 5. Error Handling ✅
**Files**: 
- `test_invalid_method`
- `test_invalid_params`

```bash
cargo test --test e2e_unix_socket_ipc test_invalid -- --ignored --nocapture
```

**Expected**: Returns JSON-RPC error responses

### 6. Concurrent Connections ✅
**File**: `test_concurrent_connections`  
**Tests**: Multiple clients simultaneously

```bash
cargo test --test e2e_unix_socket_ipc test_concurrent_connections -- --ignored --nocapture
```

**Expected**: All connections succeed

---

## Manual Testing

### Using netcat

```bash
# Connect to socket
nc -U /tmp/songbird-test.sock

# Send JSON-RPC request
{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["nat0"]},"id":1}
```

### Using Python Client

```python
import socket
import json

# Connect
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/tmp/songbird-test.sock')

# Send request
request = {
    "jsonrpc": "2.0",
    "method": "discover_by_family",
    "params": {"family_tags": ["nat0"], "timeout_ms": 5000},
    "id": 1
}
sock.sendall(json.dumps(request).encode() + b'\n')

# Read response
response = sock.recv(4096)
print(json.loads(response))
```

### Using Rust Client

See `crates/songbird-orchestrator/tests/e2e_unix_socket_ipc.rs` for `UnixSocketClient` implementation.

---

## Troubleshooting

### Socket Not Found

**Error**: `No such file or directory: /tmp/songbird-test.sock`

**Solution**: 
1. Check if Songbird is running
2. Verify `SONGBIRD_NODE_ID=test` env var
3. Check logs for IPC server startup

### Connection Refused

**Error**: `Connection refused`

**Solution**:
1. Server may not be listening yet (wait 1-2 seconds)
2. Check server logs for errors
3. Verify socket file permissions

### Timeout

**Error**: `Operation timed out`

**Solution**:
1. Server may be overloaded
2. Increase timeout in test
3. Check for deadlocks in server logs

### Invalid JSON

**Error**: `Failed to parse JSON`

**Solution**:
1. Verify JSON format (use `jq` to validate)
2. Ensure newline after JSON object
3. Check for proper escaping

---

## Test Infrastructure

### UnixSocketClient

Simple blocking client for testing:

```rust
use std::os::unix::net::UnixStream;

let mut client = UnixSocketClient::connect("/tmp/songbird-test.sock")?;
let result = client.call("discover_by_family", params)?;
```

### Helper Functions

- `wait_for_socket()`: Wait for socket file to exist
- `call()`: Send JSON-RPC request, receive response
- Timeout handling
- Error parsing

---

## CI/CD Integration

### GitHub Actions

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Start Songbird
        run: |
          export SONGBIRD_NODE_ID=test
          cargo run --package songbird-orchestrator &
          sleep 5
      
      - name: Run E2E Tests
        run: cargo test --test e2e_unix_socket_ipc -- --ignored
      
      - name: Stop Songbird
        run: pkill songbird-orchestrator
```

---

## Performance Benchmarks

### Latency

Expected response times:

- **discover_by_family**: < 10ms
- **create_genetic_tunnel**: < 50ms (includes BTSP setup)
- **announce_capabilities**: < 5ms (just logging)

### Throughput

Expected requests per second:

- **Single connection**: ~100-200 RPS
- **Concurrent connections**: ~500-1000 RPS

### Measure with

```bash
# Simple benchmark
time for i in {1..100}; do
  echo '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["nat0"]},"id":1}' | \
  nc -U /tmp/songbird-test.sock > /dev/null
done
```

---

## Next Steps

### Phase 3 (Current)
- [x] Create test client
- [ ] Run all E2E tests
- [ ] Fix any issues found
- [ ] Performance benchmarks
- [ ] Documentation

### Phase 4 (Next)
- [ ] biomeOS integration
- [ ] Real USB spore testing
- [ ] Production deployment

---

**Status**: Tests created, ready to run!  
**Confidence**: 95% (infrastructure complete, awaiting server run)

🧪 **Ready for E2E Testing!** 🧪

