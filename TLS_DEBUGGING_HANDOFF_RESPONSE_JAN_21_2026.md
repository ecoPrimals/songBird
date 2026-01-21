# 🔍 Songbird TLS Debugging Response to biomeOS

**Date**: January 21, 2026  
**Status**: 🟢 Logging Already Comprehensive - Ready for Diagnosis  
**From**: Songbird Team  
**To**: biomeOS Integration Team

---

## ✅ Good News: Logging Already Comprehensive!

The TLS handshake code **already has detailed logging** from Session 8-10 evolution work. No code changes needed!

### What's Already Logged:

```rust
// In crates/songbird-http-client/src/tls/handshake.rs:

info!("📥 Waiting for ServerHello (10 second timeout)");
error!("❌ TIMEOUT waiting for ServerHello after {:?}", elapsed);
info!("✅ Received ServerHello: {} bytes in {:?}", len, elapsed);

debug!("Step 5: Parsing ServerHello");
error!("❌ Failed to parse ServerHello: {}", e);

debug!("Step 6: Computing shared secret via BearDog ECDH");
error!("❌ BearDog ECDH derivation failed: {}", e);

debug!("Step 7: Deriving TLS session secrets via BearDog");
error!("❌ BearDog TLS secret derivation failed: {}", e);
```

### What's Already Logged in read_record():

```rust
// Record header reading:
trace!("Reading TLS record header (5 bytes)");
error!("❌ Failed to read TLS record header: {}", e);

// Record parsing:
debug!("📥 TLS record: type={:#04x} ({}), version={:#06x}, length={} bytes");
error!("❌ Invalid TLS content type: {:#04x}", content_type);
```

---

## 🎯 Diagnosis Strategy

### Step 1: Run with RUST_LOG=info (Start Here)

```bash
RUST_LOG=info \
NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock \
BEARDOG_SOCKET=/tmp/beardog-nat0.sock \
./songbird server 2>&1 | tee /tmp/songbird-tls-test.log &

# Test HTTPS
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | timeout 25 nc -U /tmp/songbird-nat0.sock
```

### Step 2: Search Logs for These Patterns

```bash
# Where does it hang?
grep -E "(Waiting for ServerHello|TIMEOUT|Failed)" /tmp/songbird-tls-test.log

# Did keypair generation work?
grep -i "keypair" /tmp/songbird-tls-test.log

# Did ECDH work?
grep -i "ecdh\|shared secret" /tmp/songbird-tls-test.log

# Did secret derivation work?
grep -i "session secrets\|session keys" /tmp/songbird-tls-test.log
```

---

## 🔍 Expected Log Patterns

### Scenario A: Hang During Keypair Generation

```
[INFO] 🤝 Starting TLS 1.3 handshake with api.github.com
[DEBUG] Generating X25519 keypair via BearDog
... HANGS HERE (no more logs)
```

**Diagnosis**: BearDog RPC call hanging  
**Check**:
```bash
# Test BearDog directly
echo '{"jsonrpc":"2.0","method":"crypto.generate_keypair","params":{"algorithm":"x25519"},"id":1}' | timeout 5 nc -U /tmp/beardog-nat0.sock
```

### Scenario B: Hang Waiting for ServerHello

```
[INFO] 🤝 Starting TLS 1.3 handshake with api.github.com
[DEBUG] Generated client keypair: 32 bytes public
[INFO] 📤 Sending ClientHello: XXX bytes to api.github.com
[INFO] 📥 Waiting for ServerHello (10 second timeout)
[ERROR] ❌ TIMEOUT waiting for ServerHello after 10s
```

**Diagnosis**: Server not responding OR ClientHello malformed  
**Check**: Packet capture to see if server sends ServerHello

### Scenario C: Hang During ECDH

```
[INFO] ✅ Received ServerHello: XXX bytes in YYms
[DEBUG] Step 5: Parsing ServerHello
[DEBUG] ✅ Parsed ServerHello - server_random: 32 bytes, server_public: 32 bytes
[DEBUG] Step 6: Computing shared secret via BearDog ECDH
... HANGS HERE
```

**Diagnosis**: BearDog ECDH call hanging  
**Check**: Test ECDH via BearDog directly

### Scenario D: Hang Reading Post-Handshake Messages

```
[INFO] 🔐 TLS session keys derived in XXms
[DEBUG] Step 8: Reading post-handshake encrypted messages
[DEBUG] Waiting for post-handshake message 1 (5 second timeout)
[TRACE] Read post-handshake record 1: XXX bytes
[DEBUG] Waiting for post-handshake message 2 (5 second timeout)
... HANGS HERE (gets stuck in loop)
```

**Diagnosis**: Not enough post-handshake messages OR timeout too short  
**Solution**: Code already handles this (breaks after 3 messages)

---

## 🔧 Most Likely Root Causes

### 1. **BearDog RPC Calls Timing Out** (80% probability)

**Symptom**: Logs show "Generating keypair" or "Computing shared secret" but never complete

**Why**: BearDog might not be responding to RPC calls, or taking too long

**Test**:
```bash
# Time a keypair generation
time (echo '{"jsonrpc":"2.0","method":"crypto.generate_keypair","params":{"algorithm":"x25519"},"id":1}' | nc -U /tmp/beardog-nat0.sock)

# Should complete in < 100ms
# If it takes > 1 second, BearDog has performance issue
```

**Solution**: Check BearDog logs, verify BearDog is healthy

### 2. **ServerHello Never Arrives** (15% probability)

**Symptom**: Logs show "Waiting for ServerHello" then timeout

**Why**: TCP connected but server doesn't like our ClientHello

**Test**: Packet capture
```bash
sudo tcpdump -i any -w /tmp/tls.pcap host api.github.com and port 443 &
# Run HTTPS test
# Analyze with: tshark -r /tmp/tls.pcap -V
```

**Solution**: Compare our ClientHello with openssl's

### 3. **Post-Handshake Messages Not Arriving** (5% probability)

**Symptom**: Logs show some post-handshake records but timeout before getting 3

**Why**: Server sends fewer messages than expected, or our parsing is wrong

**Test**: Check how many records we actually get before timeout

**Solution**: Adjust timeout or message count threshold

---

## 📊 Debugging Decision Tree

```
1. Run with RUST_LOG=info

2. Does log show "Generating X25519 keypair"?
   NO  → Songbird not even trying TLS (check HTTP vs HTTPS detection)
   YES → Go to 3

3. Does log show "Generated client keypair: 32 bytes"?
   NO  → BearDog keypair generation hanging (test BearDog directly)
   YES → Go to 4

4. Does log show "Sending ClientHello: XXX bytes"?
   NO  → ClientHello construction failing
   YES → Go to 5

5. Does log show "Waiting for ServerHello"?
   NO  → Something wrong with send logic
   YES → Go to 6

6. Does log show "Received ServerHello: XXX bytes"?
   NO  → Server not responding (packet capture needed)
   YES → Go to 7

7. Does log show "Parsed ServerHello - server_random: 32 bytes"?
   NO  → ServerHello parsing failing
   YES → Go to 8

8. Does log show "Computing shared secret via BearDog ECDH"?
   NO  → Logic error after ServerHello parse
   YES → Go to 9

9. Does log show "Computed shared secret: 32 bytes"?
   NO  → BearDog ECDH hanging (test BearDog directly)
   YES → Go to 10

10. Does log show "Deriving TLS session secrets"?
    NO  → Logic error after ECDH
    YES → Go to 11

11. Does log show "TLS session keys derived"?
    NO  → BearDog tls_derive_secrets hanging
    YES → Go to 12

12. Does log show "Reading post-handshake encrypted messages"?
    NO  → Logic error after key derivation
    YES → Go to 13

13. Does log show multiple "Read post-handshake record X" messages?
    NO  → Timeout reading first post-handshake message (packet capture)
    YES → Check if gets to 3+ messages, if not it's timeout issue
```

---

## 🚀 Quick Diagnostic Script

```bash
#!/bin/bash

echo "=== Songbird TLS Diagnostics ==="

# 1. Start Songbird with logging
echo "[1/5] Starting Songbird with trace logging..."
RUST_LOG=info \
NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock \
BEARDOG_SOCKET=/tmp/beardog-nat0.sock \
./songbird server > /tmp/songbird-tls-diag.log 2>&1 &
SONGBIRD_PID=$!

sleep 2

# 2. Test HTTPS
echo "[2/5] Testing HTTPS (20 second timeout)..."
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | timeout 20 nc -U /tmp/songbird-nat0.sock > /tmp/https-response.log 2>&1 &
TEST_PID=$!

# 3. Wait for test
sleep 15

# 4. Kill test if still running
if kill -0 $TEST_PID 2>/dev/null; then
    echo "[3/5] Test still running after 15s, killing..."
    kill $TEST_PID 2>/dev/null
fi

# 5. Kill Songbird
echo "[4/5] Stopping Songbird..."
kill $SONGBIRD_PID 2>/dev/null
sleep 1

# 6. Analyze logs
echo "[5/5] Analyzing logs..."
echo ""
echo "=== KEY LOG MESSAGES ==="
grep -E "(Starting TLS|keypair|ClientHello|ServerHello|ECDH|session|TIMEOUT|ERROR)" /tmp/songbird-tls-diag.log | tail -20

echo ""
echo "=== DIAGNOSIS ==="

if grep -q "Generated client keypair" /tmp/songbird-tls-diag.log; then
    echo "✅ Keypair generation: Working"
else
    echo "❌ Keypair generation: FAILED or HANGING"
    echo "   → Test BearDog directly"
fi

if grep -q "Sending ClientHello" /tmp/songbird-tls-diag.log; then
    echo "✅ ClientHello construction: Working"
else
    echo "❌ ClientHello construction: FAILED"
fi

if grep -q "Received ServerHello" /tmp/songbird-tls-diag.log; then
    echo "✅ ServerHello reception: Working"
else
    echo "❌ ServerHello reception: FAILED or TIMEOUT"
    echo "   → Run packet capture"
fi

if grep -q "Computed shared secret" /tmp/songbird-tls-diag.log; then
    echo "✅ ECDH: Working"
else
    echo "❌ ECDH: FAILED or HANGING"
    echo "   → Test BearDog ECDH directly"
fi

if grep -q "TLS session keys derived" /tmp/songbird-tls-diag.log; then
    echo "✅ Key derivation: Working"
else
    echo "❌ Key derivation: FAILED or HANGING"
    echo "   → Test BearDog tls_derive_secrets directly"
fi

echo ""
echo "=== FULL LOGS ==="
echo "Songbird: /tmp/songbird-tls-diag.log"
echo "HTTPS Response: /tmp/https-response.log"
```

---

## 📋 What to Send Songbird Team

### Minimum Information:

1. **Log file**: `/tmp/songbird-tls-diag.log` (with RUST_LOG=info)
2. **Last 50 lines showing hang point**:
   ```bash
   tail -50 /tmp/songbird-tls-diag.log
   ```
3. **Grep for key events**:
   ```bash
   grep -E "(keypair|ClientHello|ServerHello|ECDH|session|TIMEOUT)" /tmp/songbird-tls-diag.log
   ```

### Ideal Information:

- Full log file with RUST_LOG=info
- BearDog logs during same test
- Packet capture if ServerHello not received
- Output of diagnostic script above

---

## 🎯 Expected Outcome

After running with logging, you should see **exactly where it hangs** in the decision tree above. The logs are comprehensive enough to pinpoint:

1. **BearDog RPC issue** → Test BearDog directly
2. **Network issue** → Packet capture
3. **Protocol issue** → Compare with openssl
4. **Timeout tuning** → Adjust thresholds

**No code changes needed yet!** Let's diagnose first, then fix precisely.

---

## 📞 Next Steps

1. ✅ Run diagnostic script above
2. ✅ Send logs showing where it hangs
3. ⏳ Songbird team analyzes hang point
4. ⏳ Targeted fix (not speculative logging)
5. ⏳ Retest with fix

**Timeline**: 30 minutes to diagnose, 1-2 hours to fix once diagnosed

---

**Status**: Ready for diagnosis 🔍  
**Action**: Run with RUST_LOG=info and send logs  
**Logs**: Already comprehensive from Sessions 8-10  

---

*Response Created: January 21, 2026*  
*Team: Songbird TLS Debugging*  
*Next: Await logs from biomeOS showing exact hang point*

🐦🔐 **The logging is already there - let's see where it hangs!** 🔐🐦

