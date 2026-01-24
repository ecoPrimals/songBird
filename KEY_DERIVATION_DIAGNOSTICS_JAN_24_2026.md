# Key Derivation Diagnostics - v5.12.5

**Date**: January 24, 2026 (1:00 AM)  
**Priority**: CRITICAL - Final 0.5%  
**Status**: ✅ COMPLETE  
**Purpose**: Diagnose the "invisible 0.5%" - key derivation from CLIENT_TRAFFIC_SECRET_0

---

## 🎯 The "Invisible 0.5%" Problem

**From biomeOS**: 99.5% complete, all visible parameters validated!

**What's Validated**:
- ✅ Sequence number: 0 (CORRECT!)
- ✅ Nonce construction: RFC 8446 compliant
- ✅ AAD construction: RFC 8446 compliant
- ✅ Plaintext composition: CORRECT
- ✅ BearDog HKDF: RFC 8448 EXACT MATCHES

**What's NOT Validated** (The "Invisible 0.5%"):
- 🔍 **Key derivation**: CLIENT_TRAFFIC_SECRET_0 → client_write_key/client_write_iv
- 🔍 **HKDF-Expand-Label**: Is it correctly expanding the keys?

**The Analogy**: "Perfect lock, wrong key!" 🔑

---

## 🔬 What Was Added

### 1. Application Key Derivation Results (handshake.rs)

**Location**: After `tls_derive_application_secrets` call (line ~519)

**Shows**:
```
════════════════════════════════════════════════════════════
🔑 APPLICATION KEY DERIVATION RESULTS (DIAGNOSTIC)
════════════════════════════════════════════════════════════
This is the 'invisible 0.5%' - verifying key expansion:

Input to HKDF-Expand-Label (in BearDog):
  • CLIENT_TRAFFIC_SECRET_0 (from tls_derive_application_secrets)
  • Label: 'tls13 key' (for write key)
  • Label: 'tls13 iv' (for write IV)
  • Cipher suite: 0xXXXX

Output (what we'll use for HTTP request encryption):
  client_write_key (X bytes): [hex]
  client_write_iv (X bytes): [hex]

Expected key length for cipher 0xXXXX: X bytes
Expected IV length: 12 bytes (all TLS 1.3 ciphers)

⚠️  CRITICAL CHECK:
✅ client_write_key length is CORRECT (X bytes)
✅ client_write_iv length is CORRECT (12 bytes)

These keys will be used with:
  • Sequence number: 0 (for first HTTP request)
  • Nonce: client_write_iv XOR sequence_number
  • AAD: TLS record header (ContentType 0x17, version, length)
════════════════════════════════════════════════════════════
```

**Validates**:
- Key lengths match cipher suite
- IV length is 12 bytes
- Shows the actual keys that will be used

### 2. BearDog Application Key Derivation (beardog_client.rs)

**Location**: In `tls_derive_application_secrets` after parsing response (line ~298)

**Shows**:
```
════════════════════════════════════════════════════════════
🔬 BEARDOG APPLICATION KEY DERIVATION (DIAGNOSTIC)
════════════════════════════════════════════════════════════
BearDog RPC call: tls.derive_application_secrets

Inputs (sent to BearDog):
  pre_master_secret: X bytes
    [hex dump]
  client_random: X bytes
    [hex dump]
  server_random: X bytes
    [hex dump]
  transcript_hash: X bytes (SHA-256)
    [hex dump]
  cipher_suite: 0xXXXX

⚠️  CRITICAL: BearDog should derive:
  1. Master Secret (from pre_master_secret + transcript_hash)
  2. CLIENT_TRAFFIC_SECRET_0 (HKDF-Expand-Label)
  3. SERVER_TRAFFIC_SECRET_0 (HKDF-Expand-Label)
  4. client_write_key (HKDF-Expand-Label from CLIENT_TRAFFIC_SECRET_0)
  5. client_write_iv (HKDF-Expand-Label from CLIENT_TRAFFIC_SECRET_0)

Outputs (received from BearDog):
  client_write_key: X bytes
    [hex dump]
  server_write_key: X bytes
    [hex dump]
  client_write_iv: X bytes
    [hex dump]
  server_write_iv: X bytes
    [hex dump]

🧪 RFC 8448 Test Vector Comparison:
  If CLIENT_TRAFFIC_SECRET_0 = 48d566dbe8bb07d33ab06fc01a71a8fe...
  Then client_write_key should = 02ba47f1a767ba883ee776e329080865
  And client_write_iv should = 0393d92b4ff5ee2768bd4f4a
  (Compare with your output above!)
════════════════════════════════════════════════════════════
```

**Provides**:
- Complete hex dumps of all inputs to BearDog
- Complete hex dumps of all outputs from BearDog
- RFC 8448 test vectors for comparison
- Clear derivation chain

---

## 🧪 How to Diagnose

### Test Command

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
RUST_LOG=info ./target/release/examples/test_https https://example.com 2>&1 | tee key_derivation_diagnostic.log
```

### What to Look For

**1. In the Songbird Log** (from `handshake.rs`):
```
🔑 APPLICATION KEY DERIVATION RESULTS
```
- Note the `client_write_key` hex
- Note the `client_write_iv` hex
- Verify lengths are correct

**2. In the BearDog Log** (from `beardog_client.rs`):
```
🔬 BEARDOG APPLICATION KEY DERIVATION
```
- Note all inputs (pre_master_secret, randoms, transcript_hash)
- Note all outputs (keys and IVs)
- Compare with RFC 8448 test vectors

**3. Cross-Check**:
- Do the keys in step 1 match the keys in step 2? (They should!)
- Do the inputs to BearDog look correct?
- Do the outputs from BearDog match RFC 8448?

---

## 🎯 Diagnostic Decision Tree

### If Keys Match RFC 8448 Test Vectors ✅

**Issue**: Not in key derivation!

**Next Steps**:
1. Check if server expects different cipher suite
2. Verify TLS record format
3. Check for other protocol issues

### If Keys DON'T Match RFC 8448 Test Vectors ❌

**Issue**: BearDog's HKDF-Expand-Label implementation

**Possible Root Causes**:

#### 1. Label Format Wrong (40% likely)

**RFC 8446 requires**: `"tls13 " + label`
- For key: `"tls13 key"`
- For IV: `"tls13 iv"`

**Common mistakes**:
- Missing space: `"tls13key"` ❌
- Wrong label: `"key"` ❌
- Wrong encoding: UTF-16 instead of UTF-8 ❌

#### 2. HKDF Context/Length Wrong (30% likely)

**RFC 8446 HKDF-Expand-Label format**:
```
struct {
    uint16 length;
    opaque label<7..255>;
    opaque context<0..255>;
} HkdfLabel;
```

**Common mistakes**:
- Wrong length field (little-endian vs big-endian)
- Missing context field
- Wrong label length prefix

#### 3. Hash Function Wrong (20% likely)

**Cipher suite determines hash**:
- 0x1301 (AES-128-GCM): SHA-256
- 0x1302 (AES-256-GCM): SHA-384
- 0x1303 (ChaCha20): SHA-256

**Common mistakes**:
- Always using SHA-256
- Always using SHA-384

#### 4. Key Length Wrong (10% likely)

**Cipher suite determines key length**:
- 0x1301: 16 bytes (128 bits)
- 0x1302: 32 bytes (256 bits)
- 0x1303: 32 bytes (256 bits)

**IV length**: Always 12 bytes for TLS 1.3

---

## 🔧 Priority Actions (From biomeOS)

### Priority 1: HKDF-Expand-Label Validation (30 min)

**Test with known values**:
```
CLIENT_TRAFFIC_SECRET_0 = 48d566dbe8bb07d33ab06fc01a71a8fe1ae62ba4cc2a05c57d8e5290f70bde98

Expected outputs:
  client_write_key = 02ba47f1a767ba883ee776e329080865
  client_write_iv = 0393d92b4ff5ee2768bd4f4a
```

**Action**: Compare actual output with expected

**If matches**: Issue is elsewhere  
**If differs**: Fix HKDF-Expand-Label in BearDog

### Priority 2: OpenSSL Cross-Check (20 min)

**Capture real CLIENT_TRAFFIC_SECRET_0** from OpenSSL:
```bash
python3 scripts/tls_key_capture.py example.com
```

**Compare with ours**:
- If matches: Key expansion is wrong
- If differs: Transcript/master derivation is wrong

### Priority 3: Transcript Validation (20 min)

**Verify transcript contains**:
- ClientHello (plaintext, no TLS header)
- ServerHello (plaintext, no TLS header)
- EncryptedExtensions (decrypted plaintext)
- Certificate (decrypted plaintext)
- CertificateVerify (decrypted plaintext)
- Server Finished (decrypted plaintext)
- ❌ NOT Client Finished

### Priority 4: Multiple Servers (15 min)

**Test against**:
- github.com
- google.com
- cloudflare.com

**Rule out**: Server-specific issues

---

## 📊 What This Reveals

### Scenario 1: Keys Match RFC 8448 ✅

**Conclusion**: BearDog's key derivation is correct!

**Issue must be**:
- Server expects different cipher suite
- TLS record format issue
- Other protocol mismatch

**Next**: Check server logs, try different cipher suites

### Scenario 2: Keys DON'T Match RFC 8448 ❌

**Conclusion**: BearDog's HKDF-Expand-Label has a bug!

**Most likely**:
1. Label format wrong (`"tls13 key"` vs `"key"`)
2. HKDF context encoding wrong
3. Hash function mismatch
4. Key length wrong

**Next**: Fix BearDog's HKDF-Expand-Label implementation

### Scenario 3: Inputs to BearDog Look Wrong ❌

**Conclusion**: Songbird is sending wrong data!

**Check**:
- pre_master_secret: Should be ECDH shared secret
- client_random: Should be from ClientHello
- server_random: Should be from ServerHello
- transcript_hash: Should be SHA-256 of all handshake messages

**Next**: Fix Songbird's input preparation

---

## ⏱️ Timeline Estimate

**From biomeOS**: 2 hours to 100% HTTPS

**Breakdown**:
- Deploy v5.12.5: 5 min
- Run diagnostic test: 5 min
- Analyze logs: 15 min
- Identify exact issue: 15 min
- **Validate HKDF-Expand-Label**: 30 min
- **Implement fix** (if needed): 30 min
- Test and validate: 20 min

**Total**: ~2 hours 🎯

---

## 💡 Key Insights

### From biomeOS Analysis

**What's Working** (99.5%):
- ✅ All visible encryption parameters
- ✅ BearDog's HKDF for handshake keys (RFC 8448 validated!)
- ✅ Complete TLS handshake
- ✅ Semantic translation
- ✅ Infrastructure

**What's Unknown** (0.5%):
- 🔍 Are the APPLICATION keys correct?
- 🔍 Is HKDF-Expand-Label working for application keys?

**The Critical Question**:
> "Do the keys we're using to encrypt HTTP requests match what the server expects?"

**This logging answers that question!**

---

## 🎯 Success Criteria

**After this diagnostic**:
1. ✅ See all inputs to BearDog key derivation
2. ✅ See all outputs from BearDog key derivation
3. ✅ Compare with RFC 8448 test vectors
4. ✅ Identify if keys are correct or wrong
5. ✅ Know exactly what to fix (if anything)

**Then**:
- If keys are correct: Look elsewhere (cipher suite mismatch?)
- If keys are wrong: Fix BearDog's HKDF-Expand-Label

---

## 📋 Files Modified

### crates/songbird-http-client/src/tls/handshake.rs

**Changes**:
- Lines ~519-560: Added application key derivation results
  - Shows CLIENT_TRAFFIC_SECRET_0 → key/IV expansion
  - Validates key/IV lengths
  - Shows what will be used for HTTP encryption

### crates/songbird-http-client/src/beardog_client.rs

**Changes**:
- Lines ~298-340: Added BearDog key derivation diagnostics
  - Shows all inputs to BearDog (hex dumps)
  - Shows all outputs from BearDog (hex dumps)
  - Provides RFC 8448 test vectors for comparison
  - Shows complete derivation chain

---

## 🏆 Status

**Version**: v5.12.4 → v5.12.5  
**Build**: ✅ Success (zero errors)  
**Tests**: ✅ 102/102 passing (100%)  
**Key Derivation Diagnostics**: ✅ Comprehensive  
**Ready**: ✅ YES - Test now!

---

## 📞 Next Steps

### For biomeOS (Immediate - 20 min)

1. Deploy v5.12.5
2. Run diagnostic test:
   ```bash
   RUST_LOG=info ./target/release/examples/test_https https://example.com 2>&1 | tee key_diag.log
   ```
3. Capture logs showing:
   - 🔑 APPLICATION KEY DERIVATION RESULTS
   - 🔬 BEARDOG APPLICATION KEY DERIVATION
4. Compare keys with RFC 8448 test vectors
5. Share findings

### For BearDog Team (If needed - 30 min)

If keys don't match RFC 8448:
1. Review HKDF-Expand-Label implementation
2. Check label format (`"tls13 key"` vs `"key"`)
3. Verify HKDF context encoding
4. Test with known CLIENT_TRAFFIC_SECRET_0
5. Fix and redeploy

### For Songbird Team (After analysis - 20 min)

1. Analyze diagnostic logs
2. Determine if issue is in Songbird or BearDog
3. Coordinate fix
4. Validate end-to-end HTTPS

---

**Status**: Key derivation diagnostics complete  
**Next**: Test, analyze, identify exact issue, fix  
**ETA**: ~2 hours to 100% Pure Rust HTTPS! 🎉

**"The invisible 0.5% is now visible!"** 🔬🎯

