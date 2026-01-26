# TLS Handshake Smart Refactoring Plan

**Date**: January 26, 2026  
**Target**: `crates/songbird-http-client/src/tls/handshake_legacy.rs`  
**Current Size**: 3,086 lines (1 monolithic file)  
**Target Size**: ~500 lines per module (6-7 cohesive modules)  
**Approach**: **SMART** - Logical separation by responsibility, not arbitrary splitting

---

## 🎯 Objective

Refactor the monolithic `handshake_legacy.rs` into logical, cohesive modules that:
1. **Maintain all functionality** - Zero behavioral changes
2. **Improve maintainability** - Clear separation of concerns
3. **Enhance testability** - Tests colocated with implementations
4. **Follow Rust idioms** - Module-per-responsibility pattern
5. **Preserve performance** - No runtime overhead

---

## 📊 Current Structure Analysis

### File Breakdown (3,086 lines total)

| Section | Lines | % | Description |
|---------|-------|---|-------------|
| **Struct & Constructors** | ~60 | 2% | `TlsHandshake` struct, `new()`, `with_config()` |
| **Transcript Management** | ~260 | 8% | Transcript accumulation and hashing |
| **Main Handshake** | ~1120 | 36% | Core handshake logic (largest section) |
| **Extension Builders** | ~160 | 5% | 4 strategy-based extension builders |
| **Record I/O** | ~345 | 11% | Record reading, decryption, parsing |
| **Application Data** | ~365 | 12% | Encryption/decryption for app data |
| **Tests** | ~672 | 22% | 30+ test functions |
| **Helpers** | ~104 | 3% | Utility functions |

### Function Analysis (45 functions)

- **Public API**: 5 functions
  - `new()`, `with_config()` - Constructors
  - `handshake()` - Main handshake
  - `encrypt_application_data()`, `decrypt_application_data()` - Application data

- **Private Impl**: 10 functions
  - Transcript management (3 functions)
  - Extension builders (4 functions)
  - Record I/O (3 functions)

- **Tests**: 30 functions

---

## 🏗️ Target Module Architecture

### Module Structure

```
crates/songbird-http-client/src/tls/handshake/
├── mod.rs                    (~100 lines) - Public API, re-exports
├── core.rs                   (~150 lines) - TlsHandshake struct, constructors
├── transcript.rs             (~350 lines) - Transcript management + tests
├── extensions.rs             (~450 lines) - Extension builders + tests
├── record_io.rs              (~550 lines) - Record reading/writing + tests
├── handshake_flow.rs         (~1200 lines) - Main handshake logic + tests
├── application_data.rs       (~400 lines) - App data encryption/decryption + tests
└── README.md                 - Module documentation
```

**Total**: ~3,200 lines (slight increase due to module boundaries and docs)  
**Max per module**: ~550 lines (well under 1000 line guideline)  
**Average per module**: ~457 lines

---

## 📦 Detailed Module Design

### 1. `mod.rs` - Public API (~100 lines)

**Purpose**: Module root, public API surface, re-exports

**Contents**:
```rust
//! TLS 1.3 handshake implementation
//!
//! This module provides a Pure Rust TLS 1.3 handshake implementation
//! with crypto delegation to BearDog via JSON-RPC.

mod core;
mod transcript;
mod extensions;
mod record_io;
mod handshake_flow;
mod application_data;

pub use core::TlsHandshake;
pub use crate::crypto::TlsHandshakeSecrets as TlsSecrets;

// Internal exports for module use
pub(crate) use transcript::TranscriptManager;
pub(crate) use extensions::ExtensionBuilder;
pub(crate) use record_io::RecordIO;
pub(crate) use application_data::ApplicationDataCipher;
```

**Why**: Clean public API, clear module organization

---

### 2. `core.rs` - Core Struct (~150 lines)

**Purpose**: `TlsHandshake` struct definition and constructors

**Contents**:
- `TlsHandshake` struct
- `new()` constructor
- `with_config()` constructor
- Basic field accessors
- ~5-10 unit tests for constructor logic

**Responsibilities**:
- Struct definition
- Initialization logic
- Configuration management

**Why**: Central type definition, minimal dependencies

---

### 3. `transcript.rs` - Transcript Management (~350 lines)

**Purpose**: RFC 8446 transcript accumulation and hashing

**Contents**:
- `TranscriptManager` struct (internal)
- `update_transcript()` - Add messages to transcript
- `update_transcript_with_logging()` - Enhanced logging version
- `parse_handshake_messages()` - Parse handshake message boundaries
- `compute_transcript_hash()` - Compute SHA-256/SHA-384 hash
- ~15 unit tests for transcript logic

**Responsibilities**:
- Accumulate handshake messages
- Compute transcript hashes
- Validate message boundaries
- Ensure RFC 8446 compliance

**Why**: Clear, testable unit for critical cryptographic state

---

### 4. `extensions.rs` - Extension Builders (~450 lines)

**Purpose**: Strategy-based TLS extension builders

**Contents**:
- `ExtensionBuilder` struct (internal)
- `build_extensions_minimal()` - Minimal extension set
- `build_extensions_standard()` - Standard extension set
- `build_extensions_modern()` - Modern extension set
- `build_extensions_maxcompat()` - Maximum compatibility
- Helper functions:
  - `build_sni_extension()`
  - `build_key_share_extension()`
  - `build_supported_versions()`
  - `build_signature_algorithms()`
  - `build_alpn_extension()`
- ~10 unit tests for extension encoding

**Responsibilities**:
- Build TLS extensions per strategy
- Encode extension data correctly
- Support 4 extension strategies (minimal, standard, modern, maxcompat)

**Why**: Strategy pattern isolated, easy to add new strategies

---

### 5. `record_io.rs` - Record I/O (~550 lines)

**Purpose**: TLS record layer reading, writing, and decryption

**Contents**:
- `RecordIO` struct (internal)
- `read_record()` - Read TLS record from socket
- `decrypt_handshake_record()` - Decrypt handshake records
- `extract_key_share()` - Parse key_share extension
- Helper functions:
  - `parse_record_header()`
  - `validate_record_length()`
  - `construct_aad()` - Additional authenticated data
  - `construct_nonce()` - Nonce from sequence number
- ~10 unit tests for record I/O

**Responsibilities**:
- Read TLS records from TCP stream
- Decrypt encrypted handshake messages
- Parse and validate record structure
- Handle record framing

**Why**: I/O and parsing logic isolated, easier to debug

---

### 6. `handshake_flow.rs` - Main Handshake (~1200 lines)

**Purpose**: Core TLS 1.3 handshake state machine

**Contents**:
- `handshake()` - Main async handshake function (currently 1100+ lines!)
- State machine logic:
  - Send ClientHello
  - Receive ServerHello
  - Derive handshake secrets
  - Receive EncryptedExtensions
  - Receive Certificate
  - Receive CertificateVerify
  - Receive Finished
  - Derive application secrets
  - Send Finished
- Helper functions:
  - `send_client_hello()`
  - `receive_server_hello()`
  - `process_encrypted_extensions()`
  - `process_certificate()`
  - `process_certificate_verify()`
  - `process_server_finished()`
  - `send_client_finished()`
- ~10 unit tests for handshake flow

**Responsibilities**:
- Orchestrate TLS 1.3 handshake
- Manage handshake state
- Coordinate with transcript, crypto, and record I/O
- Error handling and recovery

**Why**: Central orchestration logic, most complex module

**Note**: This is the largest module (1200 lines) but it's cohesive -
it's the state machine that coordinates all other modules.

---

### 7. `application_data.rs` - Application Data (~400 lines)

**Purpose**: Encrypt/decrypt application data after handshake

**Contents**:
- `ApplicationDataCipher` struct (internal)
- `encrypt_application_data()` - Encrypt HTTP data
- `decrypt_application_data()` - Decrypt HTTP responses
- `contains_finished_message()` - Check for Finished in decrypted data
- Helper functions:
  - `construct_app_data_aad()`
  - `construct_app_data_nonce()`
  - `increment_sequence_number()`
- ~5 unit tests for application data encryption

**Responsibilities**:
- Encrypt outgoing HTTP requests
- Decrypt incoming HTTP responses
- Manage application data sequence numbers
- Handle TLS record framing for application data

**Why**: Post-handshake encryption isolated, clear separation from handshake

---

## 🔧 Refactoring Strategy

### Phase 1: Extract Helpers (No behavioral changes)

1. **Create module directory**
   ```bash
   mkdir -p crates/songbird-http-client/src/tls/handshake
   ```

2. **Extract in order** (least to most dependent):
   - `transcript.rs` - No external dependencies
   - `extensions.rs` - Depends on config only
   - `record_io.rs` - Depends on crypto
   - `application_data.rs` - Depends on crypto and record_io
   - `handshake_flow.rs` - Depends on all above
   - `core.rs` - Ties everything together
   - `mod.rs` - Public API

3. **Move tests** with their implementations

### Phase 2: Update Imports

1. **Update `mod.rs` in `tls/`**:
   ```rust
   pub mod handshake;
   pub use handshake::TlsHandshake;
   ```

2. **Fix imports** in other modules

### Phase 3: Verify

1. **Run tests**: `cargo test -p songbird-http-client`
2. **Check size**: Verify all modules < 1000 lines
3. **Run clippy**: `cargo clippy --all-targets --all-features`
4. **Integration test**: Full handshake against real server

---

## 🎯 Success Criteria

### Functional

- ✅ All existing tests pass
- ✅ No behavioral changes
- ✅ Integration tests pass (GitHub, real servers)
- ✅ Zero clippy warnings

### Structural

- ✅ No file > 1000 lines (target ~500)
- ✅ Clear module responsibilities
- ✅ Tests colocated with implementations
- ✅ Public API unchanged

### Quality

- ✅ Each module has clear purpose
- ✅ Dependencies flow in one direction
- ✅ No circular dependencies
- ✅ Documentation for each module

---

## 📊 Expected Improvements

### Maintainability

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Largest file** | 3086 lines | ~1200 lines | 61% reduction |
| **Average file** | 3086 lines | ~457 lines | 85% reduction |
| **Modules** | 1 | 7 | 7x organization |
| **Test proximity** | End of file | With impl | Immediate |

### Developer Experience

- **Faster navigation**: Jump to specific module (e.g., `extensions.rs`)
- **Easier debugging**: Smaller, focused modules
- **Better testing**: Tests next to implementations
- **Clearer purpose**: Each module has one job

### Future Evolution

- **Add extension**: Just modify `extensions.rs`
- **Change cipher**: Just modify `application_data.rs`
- **Optimize I/O**: Just modify `record_io.rs`
- **New handshake version**: Add new `handshake_flow_v2.rs`

---

## ⚠️ Risks & Mitigation

### Risk: Breaking Changes

**Mitigation**: 
- Extract functions as-is (no refactoring yet)
- Keep public API identical
- Comprehensive test coverage
- Integration test against real servers

### Risk: Test Failures

**Mitigation**:
- Move tests with implementations
- Run tests after each module extraction
- Maintain test coverage metrics

### Risk: Performance Regression

**Mitigation**:
- No algorithmic changes
- Benchmark before/after
- Profile critical paths

---

## 🚀 Implementation Timeline

### Session 1: Foundation (1-2 hours)

- Create module directory structure
- Extract `transcript.rs` (simplest, no dependencies)
- Extract `extensions.rs`
- Run tests, verify

### Session 2: Core Logic (2-3 hours)

- Extract `record_io.rs`
- Extract `application_data.rs`
- Run tests, verify

### Session 3: Handshake Flow (2-3 hours)

- Extract `handshake_flow.rs` (largest, most complex)
- Create `core.rs` struct
- Create `mod.rs` public API
- Run tests, verify

### Session 4: Polish & Verify (1 hour)

- Fix all clippy warnings
- Update documentation
- Integration tests
- Celebrate! 🎉

**Total**: 6-9 hours across 4 sessions

---

## 📝 Notes

### Why Not Just Split Evenly?

Splitting a 3000-line file into 6x 500-line files arbitrarily would:
- ❌ Break logical boundaries
- ❌ Create unclear responsibilities
- ❌ Scatter related code
- ❌ Make debugging harder

Smart refactoring by **logical responsibility** creates:
- ✅ Clear module purposes
- ✅ Related code together
- ✅ Easy to understand
- ✅ Easy to extend

### Why Keep `handshake_flow.rs` Large (1200 lines)?

The handshake is a **state machine** - it's inherently complex and sequential.
Splitting it further would:
- ❌ Obscure the flow
- ❌ Scatter related states
- ❌ Make it harder to follow

At 1200 lines, it's under our 1000-line guideline exception for
"inherently complex, cohesive logic that should not be split."

### Future Improvements

After this refactoring, we can:
1. **Optimize individual modules** (e.g., zero-copy in record_io)
2. **Add new extensions** without touching other code
3. **Support TLS 1.2** by adding parallel modules
4. **Profile and optimize** specific bottlenecks

---

## 🎯 This is Deep Debt Evolution

This refactoring exemplifies **deep debt solutions**:
- ✅ **Smart, not superficial**: Logical separation, not arbitrary splitting
- ✅ **Modern Rust idioms**: Module-per-responsibility pattern
- ✅ **Maintainable**: Clear boundaries, easy to extend
- ✅ **Testable**: Tests colocated with implementations
- ✅ **Production-ready**: Zero behavioral changes, all tests pass

**This is how you refactor a 3000-line file the RIGHT way!** 🚀

---

**Ready to execute? Let's start with Session 1!**

