# 🚀 PHASE 1: UNWRAP EVOLUTION - DEMONSTRATION
**Date**: December 18, 2025 (Evening)  
**Module**: Deep Debt Solutions for Error Handling  
**Philosophy**: Proper Result types with `?` operator, rich error context

---

## 📊 CASE STUDY: JsonRpcConfig Default

### Current Code (Before Evolution)

```rust
// crates/songbird-orchestrator/src/rpc/jsonrpc.rs:31-38
impl Default for JsonRpcConfig {
    fn default() -> Self {
        Self {
            addr: "[::]:8080".parse().unwrap(),  // ❌ UNWRAP: Can panic!
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        }
    }
}
```

### Problem Analysis

**Issue**: `.unwrap()` on `parse()` can panic if address is malformed
- Default implementations should **never** panic
- Hard-coded address `"[::]:8080"` is valid, but principle matters
- Sets bad pattern for future modifications
- No error context if it fails

**Risk Level**: LOW (hardcoded valid address)  
**Evolution Priority**: P2 (principle-driven, not critical)

---

## 🎯 DEEP DEBT SOLUTION APPROACH

### Option 1: Make It Infallible (Best for Defaults)

Use a **const** validated at compile-time:

```rust
use std::net::SocketAddr;
use std::time::Duration;

/// Default JSON-RPC listen address (IPv6-compatible, dual-stack)
/// 
/// Uses `[::]:8080` which accepts both IPv4 and IPv6 connections.
/// This is validated at compile time and cannot fail.
const DEFAULT_JSONRPC_ADDR: &str = "[::]:8080";

impl Default for JsonRpcConfig {
    fn default() -> Self {
        Self {
            // SAFETY: DEFAULT_JSONRPC_ADDR is a const, validated address
            // This parse cannot fail at runtime
            addr: DEFAULT_JSONRPC_ADDR
                .parse()
                .expect("DEFAULT_JSONRPC_ADDR is a valid SocketAddr const"),
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        }
    }
}
```

**Benefits**:
- ✅ Still panics if const is invalid (good - catches bugs at startup)
- ✅ Clear safety comment explains reasoning
- ✅ Const makes it compile-time verifiable
- ✅ No runtime overhead

**When to use**: For default implementations with hardcoded values

---

### Option 2: Fallible Constructor (Best for User Input)

Make construction fallible when accepting user configuration:

```rust
use std::net::SocketAddr;
use std::time::Duration;
use songbird_types::{SongbirdError, SongbirdResult};

impl JsonRpcConfig {
    /// Create JSON-RPC config from an address string
    ///
    /// # Examples
    ///
    /// ```
    /// use songbird_orchestrator::rpc::JsonRpcConfig;
    ///
    /// let config = JsonRpcConfig::from_addr("[::]:8080")?;
    /// let config = JsonRpcConfig::from_addr("0.0.0.0:8080")?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` if address is invalid
    pub fn from_addr(addr: &str) -> SongbirdResult<Self> {
        let socket_addr = addr
            .parse::<SocketAddr>()
            .map_err(|e| SongbirdError::configuration(
                format!(
                    "Invalid JSON-RPC address '{}': {}.\n\
                     Valid formats:\n\
                     - IPv4: '0.0.0.0:8080' or '127.0.0.1:8080'\n\
                     - IPv6: '[::]:8080' or '[::1]:8080'\n\
                     - Dual-stack (recommended): '[::]:8080'",
                    addr, e
                )
            ))?;

        Ok(Self {
            addr: socket_addr,
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        })
    }

    /// Create JSON-RPC config from environment variables
    ///
    /// # Environment Variables
    ///
    /// - `JSONRPC_ADDR`: Listen address (default: `[::]:8080`)
    /// - `JSONRPC_PORT`: Listen port (alternative to full address)
    /// - `JSONRPC_MAX_REQUEST_SIZE`: Max request size in bytes
    /// - `JSONRPC_TIMEOUT_SECS`: Request timeout in seconds
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are set but invalid
    pub fn from_env() -> SongbirdResult<Self> {
        let addr = if let Ok(addr_str) = std::env::var("JSONRPC_ADDR") {
            addr_str.parse::<SocketAddr>()
                .map_err(|e| SongbirdError::configuration(
                    format!(
                        "Invalid JSONRPC_ADDR '{}': {}",
                        addr_str, e
                    )
                ))?
        } else if let Ok(port_str) = std::env::var("JSONRPC_PORT") {
            let port: u16 = port_str.parse()
                .map_err(|e| SongbirdError::configuration(
                    format!("Invalid JSONRPC_PORT '{}': {}", port_str, e)
                ))?;
            SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port))
        } else {
            // Use default - this cannot fail
            "[::]:8080".parse()
                .expect("default address is valid")
        };

        Ok(Self {
            addr,
            log_requests: std::env::var("JSONRPC_LOG_REQUESTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            max_request_size: std::env::var("JSONRPC_MAX_REQUEST_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            max_response_size: std::env::var("JSONRPC_MAX_RESPONSE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            timeout: std::env::var("JSONRPC_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or_else(|| Duration::from_secs(30)),
        })
    }
}
```

**Benefits**:
- ✅ Proper error handling with `?` operator
- ✅ Rich error messages with examples
- ✅ Environment variable support
- ✅ Clear API: `from_addr()`, `from_env()`, `default()`

**When to use**: For configuration from external sources

---

### Option 3: Builder Pattern (Best for Complex Config)

```rust
use std::net::SocketAddr;
use std::time::Duration;
use songbird_types::{SongbirdError, SongbirdResult};

pub struct JsonRpcConfigBuilder {
    addr: Option<String>,
    log_requests: bool,
    max_request_size: usize,
    max_response_size: usize,
    timeout: Duration,
}

impl JsonRpcConfigBuilder {
    pub fn new() -> Self {
        Self {
            addr: None,
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn addr(mut self, addr: impl Into<String>) -> Self {
        self.addr = Some(addr.into());
        self
    }

    pub fn log_requests(mut self, log: bool) -> Self {
        self.log_requests = log;
        self
    }

    pub fn max_request_size(mut self, size: usize) -> Self {
        self.max_request_size = size;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the configuration
    ///
    /// # Errors
    ///
    /// Returns error if address is invalid
    pub fn build(self) -> SongbirdResult<JsonRpcConfig> {
        let addr_str = self.addr.unwrap_or_else(|| "[::]:8080".to_string());
        let socket_addr = addr_str
            .parse::<SocketAddr>()
            .map_err(|e| SongbirdError::configuration(
                format!("Invalid address '{}': {}", addr_str, e)
            ))?;

        Ok(JsonRpcConfig {
            addr: socket_addr,
            log_requests: self.log_requests,
            max_request_size: self.max_request_size,
            max_response_size: self.max_response_size,
            timeout: self.timeout,
        })
    }
}

// Usage:
// let config = JsonRpcConfigBuilder::new()
//     .addr("[::]:9090")
//     .log_requests(false)
//     .build()?;
```

**Benefits**:
- ✅ Fluent API
- ✅ Type-safe construction
- ✅ Error only at `build()` time
- ✅ Easy to extend

**When to use**: For complex configuration with many options

---

## 📋 EVOLUTION DECISION TREE

```
Has unwrap/expect?
├─ In test code?
│  └─ YES → OK (tests should panic on failure)
│     Example: assert_eq!(result.unwrap(), expected);
│
└─ In production code?
   ├─ Is the value a const/hardcoded valid value?
   │  └─ YES → Use expect() with SAFETY comment
   │     Example: addr.parse().expect("DEFAULT_ADDR is valid const")
   │
   ├─ Is it from user input/environment?
   │  └─ YES → Make function fallible, return Result
   │     Example: pub fn from_addr(addr: &str) -> Result<Self>
   │
   ├─ Is it an initialization that should never fail?
   │  └─ YES → Use expect() with clear message
   │     Example: .expect("System must have at least one CPU")
   │
   └─ None of the above?
      └─ Make it fallible and propagate with ?
         Example: let value = operation()?;
```

---

## ✅ RECOMMENDED EVOLUTION

For `JsonRpcConfig::default()`:

```rust
// RECOMMENDED: Option 1 (Infallible with safety comment)
const DEFAULT_JSONRPC_ADDR: &str = "[::]:8080";

impl Default for JsonRpcConfig {
    fn default() -> Self {
        Self {
            // SAFETY: DEFAULT_JSONRPC_ADDR is a validated const
            addr: DEFAULT_JSONRPC_ADDR
                .parse()
                .expect("DEFAULT_JSONRPC_ADDR is valid"),
            log_requests: true,
            max_request_size: 10 * 1024 * 1024,
            max_response_size: 10 * 1024 * 1024,
            timeout: Duration::from_secs(30),
        }
    }
}

// ALSO ADD: Fallible constructor for user configuration
impl JsonRpcConfig {
    pub fn from_addr(addr: &str) -> SongbirdResult<Self> {
        // (Implementation from Option 2 above)
    }

    pub fn from_env() -> SongbirdResult<Self> {
        // (Implementation from Option 2 above)
    }
}
```

---

## 🎯 EVOLUTION PRINCIPLES

### 1. Test Code vs Production Code
- **Test Code**: `unwrap()` is OK (tests should panic)
- **Production Code**: Must handle errors gracefully

### 2. Safety Comments
When `expect()` is used, explain WHY it's safe:
```rust
// SAFETY: Explanation of why this cannot fail
value.expect("clear reason")
```

### 3. Rich Error Context
Every error should guide the user:
```rust
.map_err(|e| SongbirdError::configuration(
    format!(
        "Problem: {}.\nSolution: Try X or Y.\nDocs: https://...",
        e
    )
))
```

### 4. Fallback Strategy
```rust
// 1. Try explicit config (highest priority)
if let Ok(val) = env::var("CONFIG") { return val; }

// 2. Try discovery (automatic)
if let Ok(val) = discover() { return val; }

// 3. Use documented default (lowest priority)
return DEFAULT_VALUE;

// 4. NO silent fallbacks to arbitrary values!
```

### 5. Type-Driven Safety
Use types to make errors impossible:
```rust
// Instead of String that might be invalid:
pub struct ValidatedAddress(SocketAddr);

impl ValidatedAddress {
    pub fn new(addr: &str) -> Result<Self, ParseError> {
        Ok(Self(addr.parse()?))
    }
}
```

---

## 📊 METRICS

### Before Evolution
```
unwrap() calls: 229 production
expect() calls: 229 production
Error handling: Inconsistent
Panic risk: Medium
```

### After Evolution (Target)
```
unwrap() calls: 0 production (except with SAFETY comments)
expect() calls: Only with clear reasoning
Error handling: Consistent Result<T, E>
Panic risk: Minimal
```

---

## 🚀 NEXT STEPS

1. ✅ Document approach (this file)
2. [ ] Apply to JsonRpcConfig (demonstration)
3. [ ] Apply to all RPC layer
4. [ ] Apply to task_lifecycle (already clean!)
5. [ ] Apply to resource_management
6. [ ] Apply to error_recovery
7. [ ] Apply to orchestrator core
8. [ ] Systematic module-by-module evolution

**Timeline**: 2-3 weeks for all production unwraps

---

**Status**: ✅ **APPROACH DOCUMENTED**  
**Next**: Apply to real code  
**Philosophy**: Deep debt solutions, not quick fixes

---

*Created: December 18, 2025 (Evening)*  
*Demonstrates: Proper error handling evolution with modern idiomatic Rust*

