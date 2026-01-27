# Smart Refactoring Plan: client.rs (1,160 lines)

## Analysis

Current file has clear logical modules:

### 1. **Construction & Configuration** (~200 lines)
- `new()`, `from_env()`, `with_tls_config()`, `with_http_config()`, `with_full_config()`
- `with_crypto()`, `with_config()`
- Getters: `http_config()`, `http_config_mut()`

### 2. **Request Orchestration** (~300 lines)
- `request()` - Main entry point with adaptive headers
- `request_follow_redirects()` - Redirect handling (3xx responses)
- Redirect logic and same-origin policy

### 3. **HTTPS Implementation** (~500 lines)
- `https_request()` - TLS handshake + HTTP over TLS
- `attempt_handshake_with_fallback()` - Adaptive TLS (1.3 → 1.2 fallback)
- Connection pooling logic
- TLS record layer management

### 4. **HTTP Implementation** (~200 lines)
- `http_request()` - Plain HTTP (non-TLS)
- HTTP/1.1 implementation

### 5. **Convenience Methods** (~60 lines)
- `get()`, `post()`, `put()`, `delete()`, `patch()`
- Simple wrappers around `request()`

## Proposed Module Structure

```
crates/songbird-http-client/src/client/
├── mod.rs              (~150 lines) - SongbirdHttpClient struct, Debug impl, re-exports
├── constructors.rs     (~200 lines) - All construction & configuration methods
├── request.rs          (~300 lines) - request() and request_follow_redirects()
├── https.rs            (~500 lines) - HTTPS implementation with TLS
├── http.rs             (~200 lines) - Plain HTTP implementation
└── convenience.rs      (~60 lines)  - get(), post(), put(), delete(), patch()
```

**Total**: ~1,410 lines across 6 modules (accounting for imports/docs)
**Reduction**: Clearer separation, easier maintenance

## Benefits

1. **Single Responsibility**: Each module has one clear purpose
2. **Easier Testing**: Can test HTTPS vs HTTP separately
3. **Better Documentation**: Each module can have focused docs
4. **Maintainability**: Changes to TLS don't affect HTTP
5. **Zero Behavioral Changes**: Pure refactoring, no logic changes

## Implementation Strategy

1. Create `client/` directory
2. Extract modules one at a time
3. Run tests after each extraction
4. Update imports
5. Mark original as deprecated (fossil record)

## Status

- [ ] Create directory structure
- [ ] Extract constructors.rs
- [ ] Extract convenience.rs (easiest, no dependencies)
- [ ] Extract http.rs
- [ ] Extract https.rs (most complex)
- [ ] Extract request.rs (orchestration)
- [ ] Create mod.rs with re-exports
- [ ] Update tests
- [ ] Mark client.rs as legacy

**Estimated Time**: 2-3 hours (careful, methodical extraction)

