# Handshake.rs Smart Refactoring Plan - January 24, 2026

**Date**: January 24, 2026, 3:40 AM  
**Status**: 🟢 **EXECUTING PHASE 2**  
**File**: `crates/songbird-http-client/src/tls/handshake.rs` (2539 lines)  

---

## 🎯 OBJECTIVE

Smart refactoring by **functionality**, not arbitrary splitting.  
Create cohesive, reusable modules for both TLS client AND server.

---

## 📊 CURRENT STRUCTURE ANALYSIS

**handshake.rs breakdown** (2539 lines):
```
Lines 1-40:    Imports and module documentation
Lines 41-65:   TlsHandshake struct definition
Lines 66-133:  Transcript tracking (2 functions)
Lines 134-290: Message parsing (parse_handshake_messages)
Lines 291-850: Main handshake function (client logic)
Lines 851-1100: ClientHello building
Lines 1101-1300: ServerHello parsing
Lines 1301-1500: Certificate validation
Lines 1501-1650: Key derivation
Lines 1651-1800: Finished message handling
Lines 1801-2000: Helper functions
Lines 2001-2539: Tests
```

**Functions identified**: 45 total
- Public API: 3 functions
- Transcript: 2 functions
- Parsing: 1 function
- Handshake logic: ~15 functions
- Helpers: ~20 functions
- Tests: 4 test functions

---

## 🏗️ SMART REFACTORING STRATEGY

### **Module Structure** (by functionality)

```
crates/songbird-http-client/src/tls/handshake/
├── mod.rs                  (~200 lines)
│   ├── TlsHandshake struct
│   ├── Public API (new, handshake)
│   └── Re-exports
│
├── transcript.rs           (~200 lines)
│   ├── Transcript tracking
│   ├── update_transcript()
│   ├── update_transcript_with_logging()
│   ├── log_transcript_hex_dump()
│   └── REUSABLE for server!
│
├── parser.rs               (~300 lines)
│   ├── parse_handshake_messages()
│   ├── HandshakeMessage struct
│   ├── Message framing logic
│   └── REUSABLE for server!
│
├── client_hello.rs         (~350 lines)
│   ├── build_client_hello()
│   ├── Extension building
│   ├── Cipher suite selection
│   └── Client-specific logic
│
├── server_hello.rs         (~250 lines)
│   ├── parse_server_hello()
│   ├── Extract server random
│   ├── Verify cipher suite
│   └── Client-side parsing
│
├── encrypted_extensions.rs (~150 lines)
│   ├── parse_encrypted_extensions()
│   ├── ALPN handling
│   └── Extension processing
│
├── certificate.rs          (~450 lines)
│   ├── parse_certificate()
│   ├── Certificate chain extraction
│   ├── Validation logic
│   └── SHARED client/server
│
├── certificate_verify.rs   (~200 lines)
│   ├── parse_certificate_verify()
│   ├── Signature verification
│   └── SHARED client/server
│
├── finished.rs             (~350 lines)
│   ├── build_client_finished()
│   ├── verify_server_finished()
│   ├── Transcript hashing
│   └── SHARED client/server
│
├── keys.rs                 (~350 lines)
│   ├── Key derivation logic
│   ├── derive_handshake_secrets()
│   ├── derive_application_secrets()
│   └── SHARED client/server
│
└── tests.rs                (~300 lines)
    ├── All test functions
    ├── Test utilities
    └── Mock helpers
```

---

## 🎯 REFACTORING PRINCIPLES

### **1. Cohesion by Functionality**
- Each module has single, clear responsibility
- Functions grouped by what they DO, not where they're called

### **2. Reusability**
- Transcript module: Used by BOTH client and server
- Parser module: Used by BOTH client and server
- Certificate/Finished: SHARED logic

### **3. Dependencies**
- Clear dependency graph
- No circular dependencies
- Minimal coupling between modules

### **4. Testing**
- Tests move to `tests.rs`
- Each module can have unit tests
- Integration tests remain separate

---

## 📋 EXECUTION PLAN

### **Phase 2.1: Extract Core Modules** (2 hours)

**Step 1: transcript.rs** (30 min) ✅ PRIORITY
- Extract `update_transcript()`
- Extract `update_transcript_with_logging()`
- Extract hex dump logic
- Add module documentation
- **Why first**: Already well-defined, server needs it!

**Step 2: parser.rs** (30 min) ✅ PRIORITY
- Extract `parse_handshake_messages()`
- Extract `HandshakeMessage` struct
- Add comprehensive documentation
- **Why second**: Recently added, clean separation, server needs it!

**Step 3: keys.rs** (30 min)
- Extract key derivation functions
- Extract HKDF wrappers
- Add RFC 8446 documentation
- **Why third**: Well-isolated, SHARED logic

**Step 4: mod.rs** (30 min)
- Create module structure
- Define public API
- Add re-exports
- Update imports

### **Phase 2.2: Extract Message Handlers** (2 hours)

**Step 5: client_hello.rs** (30 min)
- Extract `build_client_hello()`
- Extract extension builders
- Client-specific logic

**Step 6: server_hello.rs** (30 min)
- Extract parsing logic
- Extract validation
- Client-side parsing

**Step 7: finished.rs** (30 min)
- Extract client/server Finished
- SHARED verification logic

**Step 8: certificate.rs + certificate_verify.rs** (30 min)
- Extract certificate handling
- Extract verification
- SHARED logic

### **Phase 2.3: Polish & Test** (1 hour)

**Step 9: tests.rs** (30 min)
- Move all tests
- Verify all pass
- Add module-specific tests

**Step 10: Documentation** (30 min)
- Add module docs
- Update main handshake docs
- Add examples

---

## 🔑 KEY BENEFITS

### **For TLS Client**
- ✅ Easier to understand (cohesive modules)
- ✅ Easier to test (isolated functionality)
- ✅ Easier to maintain (clear responsibilities)

### **For TLS Server** (v5.13.0 foundation)
- ✅ Can reuse transcript module directly!
- ✅ Can reuse parser module directly!
- ✅ Can reuse keys module directly!
- ✅ Can reuse certificate/finished modules!

### **For Future Evolution**
- ✅ Can add TLS 1.2 fallback easily
- ✅ Can add different cipher suites easily
- ✅ Can add extensions easily
- ✅ Can optimize individual modules

---

## 🎯 SUCCESS CRITERIA

### **After Refactoring**
- ✅ All 53 tests still pass
- ✅ Zero warnings
- ✅ Clean module structure
- ✅ Comprehensive documentation
- ✅ Server can import transcript/parser modules
- ✅ No functionality changes (pure refactoring)

### **Quality Checks**
- ✅ Each module < 500 lines
- ✅ Each module has single responsibility
- ✅ Clear public API
- ✅ Proper error handling
- ✅ RFC 8446 compliance maintained

---

## 📊 ESTIMATED TIME

**Phase 2.1**: 2 hours (core modules)  
**Phase 2.2**: 2 hours (message handlers)  
**Phase 2.3**: 1 hour (polish & test)  
**Total**: **5 hours** for complete smart refactoring

---

## 🔄 EXECUTION STATUS

**Current**: Preparing Phase 2.1  
**Next**: Extract transcript.rs (30 min)  
**ETA**: 5 hours to completion  

---

**Status**: Ready to execute ✅  
**Strategy**: Smart, not arbitrary ✅  
**Benefits**: Client + Server reusability ✅  

**"Cohesive modules by functionality, not location!"** 🎯

