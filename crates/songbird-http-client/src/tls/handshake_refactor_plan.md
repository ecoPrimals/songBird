# TLS Handshake Refactoring Plan

## Current State
- **File**: `handshake_legacy.rs` - 3086 lines
- **Problem**: `handshake()` method is 1688 lines (340-2028)
- **Violation**: 1000 lines/file max, large monolithic method

## Root Cause Analysis
The handshake() method implements all 13 steps of TLS 1.3 in a single function:
1. Generate client keypair
2. Generate client random
3. Send ClientHello
4. Receive ServerHello (with timeout, adaptive learning)
5. Parse ServerHello
6. Perform ECDH
7. Compute transcript hash for handshake keys
8. Derive handshake traffic keys
9. Read and decrypt post-handshake messages
10. Compute final transcript hash
11. Derive application traffic secrets
12. Send client Finished
13. Read post-handshake messages (NewSessionTicket)

## Refactoring Strategy: Phase-Based Decomposition

### New Module Structure
```
tls/handshake/
├── mod.rs              - Public API, TlsHandshake struct
├── client_hello.rs     - Steps 1-3: Client initialization & ClientHello
├── server_hello.rs     - Steps 4-5: ServerHello receive & parse
├── key_exchange.rs     - Steps 6-8: ECDH & handshake key derivation
├── post_handshake.rs   - Step 9: Decrypt post-handshake messages
├── finalization.rs     - Steps 10-13: App keys, Finished, tickets
├── transcript.rs       - Transcript management (extracted)
├── extensions.rs       - Extension building (extracted)
└── io.rs               - I/O primitives (read_record, write_record)
```

### Method Breakdown

#### `client_hello.rs` (Steps 1-3)
- `async fn send_client_hello(&mut self, stream, server_name) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)>`
  - Returns: (client_public, client_private, client_random)
  - Generates keypair, random, builds & sends ClientHello
  - Updates transcript
  - ~150 lines

#### `server_hello.rs` (Steps 4-5)
- `async fn receive_server_hello(&mut self, stream, server_name) -> Result<ServerHelloData>`
  - Receives ServerHello with timeout + adaptive learning
  - Parses ServerHello
  - Updates transcript
  - Returns: ServerHelloData { server_random, server_public, cipher_suite }
  - ~250 lines

#### `key_exchange.rs` (Steps 6-8)
- `async fn derive_handshake_keys(&mut self, client_private, server_public, client_random, server_random, cipher_suite) -> Result<SessionKeys>`
  - ECDH shared secret
  - Transcript hash computation
  - Handshake key derivation
  - ~100 lines

#### `post_handshake.rs` (Step 9)
- `async fn process_post_handshake_messages(&mut self, stream, handshake_keys) -> Result<()>`
  - Reads encrypted messages (EncryptedExtensions, Certificate, CertificateVerify, Finished)
  - Decrypts with handshake keys
  - Updates transcript with PLAINTEXT
  - ~300 lines (complex message parsing)

#### `finalization.rs` (Steps 10-13)
- `async fn finalize_handshake(&mut self, stream, shared_secret, client_random, server_random, cipher_suite, handshake_keys) -> Result<SessionKeys>`
  - Compute final transcript hash
  - Derive application traffic secrets
  - Send client Finished
  - Read NewSessionTicket messages
  - ~250 lines

#### `transcript.rs` (Extracted)
- Existing `update_transcript()` and related methods
- `compute_transcript_hash()`
- ~150 lines

#### `extensions.rs` (Extracted)
- Existing extension building methods
- ~300 lines

#### `io.rs` (Extracted)
- `async fn read_record(stream) -> Result<Vec<u8>>`
- `async fn write_record(stream, data) -> Result<()>`
- ~100 lines

### New `handshake()` Method (Orchestrator)
```rust
pub async fn handshake(&mut self, stream: &mut TcpStream, server_name: &str) -> Result<SessionKeys> {
    info!("🤝 Starting TLS 1.3 handshake with {}", server_name);
    
    // Phase 1: ClientHello (Steps 1-3)
    let (client_public, client_private, client_random) = 
        self.send_client_hello(stream, server_name).await?;
    
    // Phase 2: ServerHello (Steps 4-5)
    let server_hello = self.receive_server_hello(stream, server_name).await?;
    self.cipher_suite = server_hello.cipher_suite;
    
    // Phase 3: Key Exchange (Steps 6-8)
    let handshake_keys = self.derive_handshake_keys(
        &client_private, &server_hello.server_public,
        &client_random, &server_hello.server_random,
        server_hello.cipher_suite
    ).await?;
    
    // Phase 4: Post-Handshake Messages (Step 9)
    self.process_post_handshake_messages(stream, &handshake_keys).await?;
    
    // Phase 5: Finalization (Steps 10-13)
    let app_keys = self.finalize_handshake(
        stream, &shared_secret, &client_random, &server_hello.server_random,
        server_hello.cipher_suite, &handshake_keys
    ).await?;
    
    info!("✅ TLS 1.3 handshake complete!");
    Ok(app_keys)
}
```
**Size**: ~50 lines (clean orchestration!)

## Benefits
1. ✅ **Meets 1000-line limit**: Each file < 500 lines
2. ✅ **Testable**: Each phase can be unit tested independently
3. ✅ **Readable**: Clear separation of concerns
4. ✅ **Maintainable**: Easy to debug/modify specific phases
5. ✅ **Preserves logic**: No behavior changes, just reorganization

## Implementation Steps
1. Create new `tls/handshake/` module directory
2. Extract helper methods (transcript, extensions, io) first
3. Decompose handshake() into phase methods
4. Move phase methods to dedicated files
5. Update mod.rs with public API
6. Update imports in dependent code
7. Run full test suite to verify no regressions

## Timeline
- **Estimated**: 4-6 hours
- **Risk**: LOW (pure refactoring, no logic changes)
- **Priority**: HIGH (file size violation, maintainability)

