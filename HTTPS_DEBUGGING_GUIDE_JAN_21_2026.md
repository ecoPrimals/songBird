# 🔍 HTTPS Debugging Guide for biomeOS

**Date**: January 21, 2026  
**Purpose**: Debug HTTPS timeout issues in Tower Atomic  
**Status**: HTTP Working ✅ | HTTPS Timing Out ⏱️

---

## Problem Summary

**Observation**: HTTPS requests to `https://api.github.com/zen` timeout after 15 seconds  
**Expected**: TLS handshake completes, HTTP response received  
**Code Status**: TLS 1.3 handshake implementation IS present in v0.2.1 ecoBin

---

## Immediate Debugging Steps

### 1. Enable Comprehensive Logging ✅

Run Songbird with full trace logging:

```bash
RUST_LOG=trace /path/to/songbird
```

Or for specific modules:

```bash
RUST_LOG=songbird_http_client=trace,songbird_orchestrator::ipc=debug /path/to/songbird
```

### 2. Test HTTPS and Capture Logs

```bash
# Start Songbird with trace logging
RUST_LOG=trace /path/to/songbird 2>&1 | tee songbird-https-debug.log &

# Wait for Songbird to start
sleep 2

# Test HTTPS request
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | nc -U /tmp/songbird-nat0.sock

# Let it run for 20 seconds
sleep 20

# Kill Songbird
pkill songbird
```

### 3. Analyze the Logs

Look for these key log messages in `songbird-https-debug.log`:

#### Expected Flow:
```
1. TCP Connection:
   "Connecting to api.github.com:443"
   "✅ TCP connection established"

2. TLS Handshake Start:
   "🤝 Starting TLS 1.3 handshake with api.github.com"
   
3. BearDog Keypair:
   "Generating X25519 keypair via BearDog"
   "→ BearDog RPC: crypto.generate_keypair"
   "← BearDog RPC: crypto.generate_keypair result"
   
4. ClientHello:
   "Sending ClientHello: XXX bytes"
   
5. ServerHello:
   "Received ServerHello: XXX bytes"
   
6. ECDH:
   "Performing ECDH via BearDog"
   "→ BearDog RPC: crypto.ecdh_derive"
   "← BearDog RPC: crypto.ecdh_derive result"
   
7. Session Keys:
   "Deriving TLS secrets via BearDog"
   "→ BearDog RPC: tls.derive_secrets"
   "← BearDog RPC: tls.derive_secrets result"
   
8. Post-Handshake Messages:
   "Reading TLS record: type=XX, length=XXX"
   (Should see 3+ records)
   
9. Success:
   "✅ TLS handshake complete"
```

#### Where It's Likely Hanging:

**A. TCP Connection**:
```
"Connecting to api.github.com:443"
... TIMEOUT (no success message)
```
→ **Problem**: Network/DNS/firewall

**B. Waiting for ServerHello**:
```
"Sending ClientHello: XXX bytes"
... TIMEOUT (no ServerHello message)
```
→ **Problem**: Server rejected ClientHello OR network issue

**C. BearDog RPC Hang**:
```
"→ BearDog RPC: crypto.generate_keypair"
... TIMEOUT (no result)
```
→ **Problem**: BearDog not responding

**D. Reading Post-Handshake**:
```
"Received ServerHello: XXX bytes"
"Reading TLS record: type=XX, length=XXX"
... TIMEOUT (stuck reading records)
```
→ **Problem**: TLS protocol mismatch or incomplete implementation

---

## Network-Level Debugging

### 1. Packet Capture ✅

Capture actual TLS traffic:

```bash
# Start packet capture
sudo tcpdump -i any -w https-debug.pcap host api.github.com and port 443 &
TCPDUMP_PID=$!

# Run HTTPS test
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | nc -U /tmp/songbird-nat0.sock &

# Wait
sleep 20

# Stop capture
sudo kill $TCPDUMP_PID

# Analyze with Wireshark or tshark
tshark -r https-debug.pcap -V | grep -A50 "Client Hello\|Server Hello"
```

### 2. Check What's Actually Sent

```bash
# What did we send?
tshark -r https-debug.pcap -Y "tls.handshake.type == 1" -V

# Did server respond?
tshark -r https-debug.pcap -Y "tls.handshake.type == 2" -V
```

---

## BearDog RPC Debugging

### 1. Verify BearDog is Running

```bash
# Check BearDog socket
ls -l /tmp/beardog-*.sock

# Test BearDog RPC
echo '{"jsonrpc":"2.0","method":"crypto.generate_keypair","params":{"algorithm":"x25519"},"id":1}' | nc -U /tmp/beardog-*.sock
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "public_key": "BASE64...",
    "private_key": "BASE64..."
  },
  "id": 1
}
```

### 2. Test Each RPC Method

```bash
# Test ECDH
echo '{"jsonrpc":"2.0","method":"crypto.ecdh_derive","params":{"private_key":"AAAA...","public_key":"BBBB..."},"id":2}' | nc -U /tmp/beardog-*.sock

# Test TLS secrets derivation
echo '{"jsonrpc":"2.0","method":"tls.derive_secrets","params":{"shared_secret":"CCCC...","client_random":"DDDD...","server_random":"EEEE..."},"id":3}' | nc -U /tmp/beardog-*.sock
```

---

## Common Issues and Solutions

### Issue 1: TCP Connection Fails
**Symptom**: "Failed to connect to api.github.com:443"  
**Causes**:
- DNS resolution failure
- Network/firewall blocking port 443
- No internet connectivity

**Solution**:
```bash
# Test DNS
nslookup api.github.com

# Test TCP
telnet api.github.com 443

# Test with curl (baseline)
curl -v https://api.github.com/zen
```

### Issue 2: ServerHello Never Arrives
**Symptom**: Timeout waiting for ServerHello  
**Causes**:
- ClientHello malformed
- TLS version/cipher suite mismatch
- Server doesn't speak TLS 1.3

**Solution**:
- Capture packet and analyze ClientHello format
- Compare with working TLS client (openssl s_client)
- Check server's supported TLS versions

### Issue 3: BearDog Not Responding
**Symptom**: RPC calls hang  
**Causes**:
- BearDog crashed/not running
- Unix socket permissions
- BearDog internal error

**Solution**:
```bash
# Check BearDog logs
journalctl -u beardog -f

# Restart BearDog
systemctl restart beardog

# Check socket
ls -l /tmp/beardog-*.sock
sudo lsof | grep beardog
```

### Issue 4: Post-Handshake Records Hang
**Symptom**: Reads some records then times out  
**Causes**:
- Incorrect record parsing
- Wrong number of expected messages
- Encryption/decryption issue

**Solution**:
- Check packet capture for actual TLS records sent
- Compare with TLS 1.3 spec (RFC 8446)
- Verify BearDog crypto operations

---

## Comparison: Working vs Broken

### HTTP (Working) ✅
```
1. TCP connect to example.com:80
2. Send HTTP request (plain text)
3. Receive HTTP response (plain text)
4. Parse and return
```

### HTTPS (Broken) ⏱️
```
1. TCP connect to api.github.com:443 ✅
2. TLS handshake:
   a. Generate keypair (BearDog) ✅?
   b. Send ClientHello ✅?
   c. Receive ServerHello ❓ (TIMEOUT HERE?)
   d. Read post-handshake messages ❓
   e. Send ChangeCipherSpec ❌ (Never reached)
3. Send HTTP request (encrypted) ❌
4. Receive HTTP response (encrypted) ❌
```

---

## Quick Diagnostic Commands

```bash
# 1. Is Songbird running?
ps aux | grep songbird

# 2. Is BearDog running?
ps aux | grep beardog

# 3. Can we reach the server?
curl -v https://api.github.com/zen

# 4. What does openssl say?
openssl s_client -connect api.github.com:443 -tls1_3

# 5. Check Songbird logs
journalctl -u songbird -f

# 6. Check BearDog logs
journalctl -u beardog -f
```

---

## What to Send to Songbird Team

### Minimum Information:
1. **Logs**: `songbird-https-debug.log` (with RUST_LOG=trace)
2. **Last 50 lines** showing where it hangs
3. **BearDog status**: Is it running? Any errors?
4. **Network test**: Does `curl https://api.github.com/zen` work?

### Ideal Information:
1. Full trace logs
2. Packet capture (https-debug.pcap)
3. BearDog logs during the test
4. Output of all diagnostic commands above

---

## Expected Logging Already in Code

The TLS handshake code already has these log points:

```rust
// In handshake.rs
debug!("🤝 Starting TLS 1.3 handshake with {}", server_name);
trace!("Sending ClientHello: {} bytes", ...);
trace!("Received ServerHello: {} bytes", ...);
debug!("🔐 Handshake traffic keys derived");
trace!("Read post-handshake record {} ({} bytes)", ...);
debug!("✅ TLS handshake complete");
```

```rust
// In beardog_client.rs
debug!("Generating X25519 keypair via BearDog");
debug!("Performing ECDH via BearDog");
debug!("Deriving TLS secrets via BearDog");
trace!("→ BearDog RPC: {} (id={})", method, id);
trace!("← BearDog RPC: {} result (id={})", method, response.id);
```

**Just run with `RUST_LOG=trace` to see them!**

---

## Next Steps

1. ✅ Run with `RUST_LOG=trace`
2. ✅ Capture logs during HTTPS timeout
3. ✅ Identify where it hangs (see "Expected Flow" above)
4. ✅ Run packet capture if needed
5. ✅ Send logs to Songbird team
6. ⏳ We'll add targeted logging based on findings

---

**Status**: Ready for debugging 🔍  
**Action**: biomeOS to run with trace logging and send results  
**ETA**: Should identify hang point within 5 minutes of testing

---

*Document Date*: January 21, 2026  
*Author*: AI Assistant + eastgate  
*Purpose*: Enable rapid HTTPS debugging for biomeOS

