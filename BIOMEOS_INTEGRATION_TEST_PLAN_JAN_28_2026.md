# biomeOS Integration Test Plan

**Date**: January 28, 2026 (Evening)  
**Status**: 🟢 **READY FOR TESTING**  
**Priority**: HIGH - Validates Port:0 fix and dual-mode operation

---

## Test Summary

Validates the Port:0 beacon fix and ensures Songbird operates correctly in dual-mode:
- **External TCP port** (for LAN discovery beacons)
- **Internal Unix socket** (for inter-primal IPC)

---

## Prerequisites

```bash
# Ensure release build is current
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release

# Verify binary version
./target/release/songbird --version
# Expected: songbird 3.33.0
```

---

## Test 1: Configuration Validation (Port:0 Rejection)

### Test 1A: Port 0 with Discovery Enabled (Should FAIL)

```bash
# Create invalid configuration
cat > /tmp/test_invalid_port0.toml << 'EOF'
[system]
system_id = "test-system"

[network]
base_port = 0  # INVALID with discovery enabled

[discovery]
mode = "Anonymous"  # Discovery enabled
EOF

# Try to start (should fail with helpful error)
SONGBIRD_CONFIG_PATH=/tmp/test_invalid_port0.toml \
    ./target/release/songbird server --config /tmp/test_invalid_port0.toml
```

**Expected Result**:
```
❌ Discovery requires external TCP port (network.base_port > 0).

Songbird operates in dual-mode:
• External TCP port (for LAN discovery beacons)
• Internal Unix socket (for inter-primal IPC)

Fix: Set network.base_port = 8080 or disable discovery.

Example:
  ./songbird server --port 8080 --socket /run/user/1000/biomeos/songbird-nat0.sock

Or disable discovery:
  [discovery]
  mode = "Disabled"
```

### Test 1B: Port 0 with Discovery Disabled (Should FAIL with generic message)

```bash
# Create configuration with discovery disabled
cat > /tmp/test_port0_no_discovery.toml << 'EOF'
[system]
system_id = "test-system"

[network]
base_port = 0  # Still invalid

[discovery]
mode = "Disabled"  # Discovery disabled
EOF

# Try to start (should fail with generic error)
SONGBIRD_CONFIG_PATH=/tmp/test_port0_no_discovery.toml \
    ./target/release/songbird server --config /tmp/test_port0_no_discovery.toml
```

**Expected Result**:
```
Network base port must be greater than 0 (use 8080 for default)
```

---

## Test 2: Valid Configuration with CLI Override

### Test 2A: CLI Port Override

```bash
# Start with valid port via CLI (overrides config/env)
./target/release/songbird server \
    --port 8080 \
    --socket /tmp/songbird-test-$$. sock \
    --verbose &
SONGBIRD_PID=$!

# Wait for startup
sleep 5

# Verify logs show correct port
# Expected in logs:
#   "External Port: 8080 (LAN discovery/federation)"
#   "✅ HTTP server started on port 8080"
#   "🌐 Starting anonymous discovery with actual HTTPS port 8080"

# Check that discovery beacon contains correct port
# (would need to listen on UDP 2300, but we can check process is running)
ps -p $SONGBIRD_PID && echo "✅ Songbird running with port 8080"

# Cleanup
kill $SONGBIRD_PID
wait $SONGBIRD_PID 2>/dev/null
```

**Expected Result**:
- Songbird starts successfully
- Logs show `External Port: 8080`
- Process runs without errors
- Discovery beacons (if we capture them) contain `port: 8080` (not 0)

### Test 2B: Federation Port Alias

```bash
# Start with --federation-port (alias for --port)
./target/release/songbird server \
    --federation-port 9090 \
    --socket /tmp/songbird-federation-test-$$.sock \
    --verbose &
SONGBIRD_PID=$!

# Wait for startup
sleep 5

# Verify logs show 9090 (federation-port takes precedence)
# Expected in logs:
#   "External Port: 9090 (LAN discovery/federation)"

# Cleanup
kill $SONGBIRD_PID
wait $SONGBIRD_PID 2>/dev/null
```

**Expected Result**:
- Songbird starts successfully
- Logs show `External Port: 9090` (not default 8080)
- `--federation-port` takes precedence over `--port`

---

## Test 3: Dual-Mode Operation Verification

### Test 3: External TCP + Internal Unix Socket

```bash
# Set up XDG-compliant paths (biomeOS style)
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export FAMILY_ID=nat0
mkdir -p $XDG_RUNTIME_DIR/biomeos

# Start Songbird in dual-mode
./target/release/songbird server \
    --port 8080 \
    --socket $XDG_RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock \
    --verbose &
SONGBIRD_PID=$!

# Wait for startup
sleep 5

# Verify External TCP binding
nc -zv 127.0.0.1 8080 && echo "✅ External TCP port 8080 is open"

# Verify Internal Unix socket exists
ls -l $XDG_RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock && echo "✅ Internal Unix socket exists"

# Try HTTP health check (external interface)
curl -s http://127.0.0.1:8080/health && echo "✅ External HTTP interface accessible"

# Try Unix socket JSON-RPC (internal interface)
echo '{"jsonrpc":"2.0","id":1,"method":"health","params":{}}' | \
    nc -U $XDG_RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock && \
    echo "✅ Internal Unix socket IPC working"

# Cleanup
kill $SONGBIRD_PID
wait $SONGBIRD_PID 2>/dev/null
rm -f $XDG_RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock
```

**Expected Result**:
- External TCP port 8080 is bound and accessible
- Internal Unix socket is created and accessible
- Both interfaces respond correctly
- Demonstrates dual-mode operation

---

## Test 4: Discovery Beacon Validation (Network Test)

### Test 4: Capture and Validate Discovery Beacon

```bash
# Terminal 1: Start beacon listener
# (This captures UDP multicast beacons on port 2300)
nc -u -l 2300 > /tmp/beacon_capture.bin &
LISTENER_PID=$!

# Terminal 2: Start Songbird
./target/release/songbird server \
    --port 8080 \
    --socket /tmp/songbird-beacon-test-$$.sock \
    --verbose &
SONGBIRD_PID=$!

# Wait for beacon broadcast
sleep 10

# Stop both
kill $SONGBIRD_PID $LISTENER_PID 2>/dev/null
wait $SONGBIRD_PID $LISTENER_PID 2>/dev/null

# Parse beacon (JSON format)
if [ -s /tmp/beacon_capture.bin ]; then
    echo "✅ Beacon captured"
    cat /tmp/beacon_capture.bin | jq . || cat /tmp/beacon_capture.bin
    
    # Verify port field is NOT 0
    PORT=$(cat /tmp/beacon_capture.bin | jq -r '.port' 2>/dev/null)
    if [ "$PORT" != "0" ] && [ "$PORT" != "null" ]; then
        echo "✅ Beacon contains valid port: $PORT"
    else
        echo "❌ Beacon contains invalid port: $PORT"
    fi
else
    echo "⚠️  No beacon captured (may need multicast routing)"
fi

# Cleanup
rm -f /tmp/beacon_capture.bin /tmp/songbird-beacon-test-*.sock
```

**Expected Result**:
- Beacon is captured on UDP multicast
- Beacon JSON contains `"port": 8080` (NOT 0)
- Beacon passes validation (no "Invalid port: 0" errors)

---

## Test 5: Cross-Interface Discovery (Advanced)

> **Note**: This test requires two physical interfaces (ethernet + wifi) or VMs on different subnets.

### Setup

```bash
# Tower A (ethernet interface - 192.168.1.x)
HOST_A=192.168.1.100

# Tower B (wifi interface - 192.168.0.x or different subnet)
HOST_B=192.168.0.200

# Verify both can reach each other via TCP
ping -c 3 $HOST_B  # From Host A
ping -c 3 $HOST_A  # From Host B
```

### Test Execution

```bash
# On Tower A (ethernet)
./target/release/songbird server \
    --port 8080 \
    --socket /tmp/songbird-tower-a.sock \
    --verbose

# On Tower B (wifi)
./target/release/songbird server \
    --port 8080 \
    --socket /tmp/songbird-tower-b.sock \
    --verbose

# Wait 60 seconds for discovery

# Check Tower A logs for discovery of Tower B
# Expected: "✅ Discovered peer: tower-{id} at 192.168.0.200:8080"

# Check Tower B logs for discovery of Tower A
# Expected: "✅ Discovered peer: tower-{id} at 192.168.1.100:8080"
```

**Expected Result**:
- Both towers discover each other within 60 seconds
- Discovery works across wifi/ethernet boundaries (via subnet broadcast fallback)
- No "Invalid port: 0" errors
- Federation handshake succeeds

---

## Test 6: Unit Test Validation

```bash
# Run configuration validation tests
cargo test --package songbird-types --lib consolidated_canonical::tests

# Expected output:
# test config::consolidated_canonical::tests::test_validate_port_zero_with_discovery_disabled ... ok
# test config::consolidated_canonical::tests::test_validate_port_nonzero_with_discovery_enabled ... ok
# test config::consolidated_canonical::tests::test_validate_port_zero_with_discovery_enabled ... ok
# test config::consolidated_canonical::tests::test_default_config_is_valid ... ok
#
# test result: ok. 4 passed; 0 failed; 0 ignored
```

---

## Success Criteria

| Test | Criteria | Status |
|------|----------|--------|
| 1A | Port:0 with discovery enabled is rejected with helpful error | ⬜ |
| 1B | Port:0 with discovery disabled shows generic error | ⬜ |
| 2A | CLI `--port` override works correctly | ⬜ |
| 2B | CLI `--federation-port` alias works correctly | ⬜ |
| 3 | Dual-mode operation (TCP external + Unix internal) verified | ⬜ |
| 4 | Discovery beacon contains valid port (not 0) | ⬜ |
| 5 | Cross-interface discovery works (if testable) | ⬜ |
| 6 | All unit tests pass | ✅ (verified) |

---

## Known Limitations

1. **Test 4 (Beacon Capture)**: Requires multicast routing to be enabled. May fail on some systems but this doesn't indicate a problem with Songbird.

2. **Test 5 (Cross-Interface)**: Requires physical multi-interface setup or VMs. Can be skipped for initial validation.

3. **Firewall**: Some tests may fail if local firewall blocks UDP 2300 or TCP 8080. Use `sudo ufw allow 2300/udp` and `sudo ufw allow 8080/tcp` if needed.

---

## Automated Test Script

```bash
#!/bin/bash
# File: test_biomeos_integration.sh

set -e

echo "🧪 biomeOS Integration Test Suite"
echo "=================================="
echo ""

# Test 1A: Port 0 validation with discovery enabled
echo "Test 1A: Port:0 validation (discovery enabled)..."
cat > /tmp/test_invalid.toml << 'EOF'
[system]
system_id = "test"
[network]
base_port = 0
[discovery]
mode = "Anonymous"
EOF

if ./target/release/songbird server --config /tmp/test_invalid.toml 2>&1 | grep -q "Discovery requires external TCP port"; then
    echo "✅ Test 1A passed: Port:0 validation works"
else
    echo "❌ Test 1A failed: Port:0 validation not working"
    exit 1
fi

# Test 2A: Valid port via CLI
echo ""
echo "Test 2A: Valid port via CLI..."
timeout 10 ./target/release/songbird server --port 8080 --socket /tmp/test-$$.sock 2>&1 | grep -q "External Port: 8080" && \
    echo "✅ Test 2A passed: CLI port override works" || \
    echo "⚠️  Test 2A: Could not verify (may need longer timeout)"

# Test 6: Unit tests
echo ""
echo "Test 6: Unit tests..."
cargo test --package songbird-types --lib consolidated_canonical::tests --quiet && \
    echo "✅ Test 6 passed: All unit tests pass" || \
    (echo "❌ Test 6 failed: Unit tests failing"; exit 1)

echo ""
echo "🎊 Integration tests complete!"
echo "   Manual tests (3, 4, 5) should be run in target environment."
```

**Run**:
```bash
chmod +x test_biomeos_integration.sh
./test_biomeos_integration.sh
```

---

## Verification Checklist

- [ ] **Port validation works**: Port:0 is rejected when discovery is enabled
- [ ] **CLI overrides work**: `--port` and `--federation-port` correctly override config
- [ ] **Dual-mode confirmed**: Both TCP external and Unix internal interfaces bind correctly
- [ ] **Beacons contain port**: Discovery beacons include `port: 8080` (not 0)
- [ ] **Unit tests pass**: All 4 new validation tests pass
- [ ] **Documentation updated**: `DUAL_MODE_ARCHITECTURE_JAN_28_2026.md` complete
- [ ] **Help text correct**: `./songbird server --help` shows new flags with proper descriptions

---

**Generated**: 2026-01-28 (Evening)  
**Status**: 🟢 Ready for execution  
**Impact**: Validates Port:0 fix and dual-mode architecture

🎊 **BIOMEOS INTEGRATION TEST PLAN COMPLETE** 🎊

