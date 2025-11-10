# 🔧 Unified Error Handling Quick Reference

**Last Updated**: November 7, 2025  
**Status**: Production-Ready ✅  
**Location**: `crates/songbird-types/src/errors.rs`

---

## 🎯 Quick Start (2 Minutes)

### Import Everything You Need

```rust
use songbird_types::{
    SongbirdError,    // Main error type
    SongbirdResult,   // Result<T, SongbirdError>
};
```

### Basic Usage

```rust
pub async fn my_function() -> SongbirdResult<Data> {
    let data = fetch_data().await?;  // Errors auto-convert!
    Ok(data)
}
```

---

## 📋 Error Types (13 Variants)

### 1. **Configuration** - Config-related errors

```rust
// Create
SongbirdError::configuration("Missing required field: database_url")

// With details
SongbirdError::Configuration {
    message: "Invalid port number".to_string(),
    field: Some("port".to_string()),
    suggestion: Some("Port must be between 1-65535".to_string()),
}
```

**Use When**: Configuration parsing, validation, missing fields

### 2. **Network** - Network-related errors

```rust
// Create
SongbirdError::network("Connection timeout after 5s")

// With details  
SongbirdError::Network {
    message: "Failed to connect".to_string(),
    interface: Some("eth0".to_string()),
    suggestion: Some("Check network connectivity".to_string()),
}
```

**Use When**: Connection failures, timeouts, DNS resolution

### 3. **Security** - Security & authentication errors

```rust
// Create
SongbirdError::security("Unauthorized access attempt")

// With context
let mut err = SongbirdError::security("Invalid token");
err.with_context("authentication");
err.with_suggestion("Obtain a new token");
```

**Use When**: Auth failures, permission denied, invalid credentials

### 4. **Service** - Service-related errors

```rust
// Create
SongbirdError::service("api-gateway", "Service unavailable")

// With recovery hints
SongbirdError::Service {
    service: "database".to_string(),
    message: "Connection pool exhausted".to_string(),
    suggested_alternatives: vec!["cache".to_string()],
    recovery_actions: vec!["retry".to_string(), "fallback".to_string()],
}
```

**Use When**: Service failures, unavailability, degradation

### 5. **Discovery** - Service discovery errors

```rust
// Create
SongbirdError::discovery("No services found matching criteria")

// With backend info
SongbirdError::Discovery {
    message: "Consul unavailable".to_string(),
    backend: Some("consul".to_string()),
    retry_strategy: Some("exponential-backoff".to_string()),
}
```

**Use When**: Service lookup failures, discovery backend issues

### 6. **Registry** - Service registry errors

```rust
// Create
SongbirdError::registry("Service not found", "lookup")

// With operation details
SongbirdError::Registry {
    message: "Registration failed".to_string(),
    service_name: Some("my-service".to_string()),
    operation: "register".to_string(),
}
```

**Use When**: Registration failures, lookup errors

### 7. **LoadBalancing** - Load balancing errors

```rust
// Create
SongbirdError::load_balancing("No healthy instances", "round-robin")

// With instance info
SongbirdError::LoadBalancing {
    message: "All instances failing".to_string(),
    available_instances: 0,
    strategy: "least-connections".to_string(),
}
```

**Use When**: No available instances, routing failures

### 8. **Protocol** - Protocol version errors

```rust
// Create
SongbirdError::protocol("Unsupported protocol version")

// With version details
SongbirdError::Protocol {
    message: "Version mismatch".to_string(),
    expected_version: Some("2.0".to_string()),
    actual_version: Some("1.5".to_string()),
}
```

**Use When**: Version mismatches, incompatible protocols

### 9. **Metrics** - Metrics collection errors

```rust
// Create
SongbirdError::metrics("Failed to record metric", "increment")

// With metric details
SongbirdError::Metrics {
    message: "Prometheus unreachable".to_string(),
    metric_name: Some("request_count".to_string()),
    operation: "record".to_string(),
}
```

**Use When**: Metric recording failures, observability issues

### 10. **Event** - Event processing errors

```rust
// Create
SongbirdError::event("Event processing failed")

// With event details
SongbirdError::Event {
    message: "Invalid event format".to_string(),
    event_type: Some("ServiceRegistered".to_string()),
    processing_stage: Some("validation".to_string()),
}
```

**Use When**: Event handler failures, invalid events

### 11. **Validation** - Validation errors

```rust
// Create
SongbirdError::validation("Invalid email format")

// With field details
SongbirdError::Validation {
    message: "Port out of range".to_string(),
    field: Some("port".to_string()),
    suggestion: Some("Use 1-65535".to_string()),
}
```

**Use When**: Input validation, schema validation

### 12. **Serialization** - Serialization errors

```rust
// Automatic from serde_json::Error
let result: SongbirdResult<Value> = 
    serde_json::from_str(json_str).map_err(Into::into);

// Manual
SongbirdError::Serialization {
    format: Some("YAML".to_string()),
    message: "Parse error".to_string(),
    debug_info: Some(details),
}
```

**Use When**: JSON/YAML parsing, serialization failures

### 13. **Runtime** - Async runtime errors

```rust
SongbirdError::Runtime {
    message: "Task panicked".to_string(),
    component: Some("worker-pool".to_string()),
    debug_info: None,
}
```

**Use When**: Tokio task failures, runtime issues

---

## 🔄 Automatic Conversions

### Standard Library Types

```rust
// From String/&str (becomes Configuration error)
let err: SongbirdError = "Something went wrong".into();

// From std::io::Error (becomes Network error)
let file = std::fs::File::open("config.toml")?;  // Auto-converts!

// From std::net::AddrParseError (becomes Network error)
let addr: SocketAddr = "127.0.0.1:8080".parse()?;  // Auto-converts!
```

### External Crate Types

```rust
// From serde_json::Error (becomes Serialization error)
let value: Value = serde_json::from_str(json)?;  // Auto-converts!
```

---

## 💡 Common Patterns

### Pattern 1: Error with Context

```rust
pub async fn connect_database(url: &str) -> SongbirdResult<Connection> {
    let conn = Database::connect(url)
        .await
        .map_err(|e| {
            let mut err = SongbirdError::service("database", e.to_string());
            err.suggested_alternatives = vec!["cache".to_string()];
            err.recovery_actions = vec!["retry".to_string()];
            err
        })?;
    
    Ok(conn)
}
```

### Pattern 2: Error Chaining

```rust
pub async fn process_request() -> SongbirdResult<Response> {
    // All these errors auto-convert to SongbirdError via ?
    let config = load_config()?;           // Config error
    let service = discover_service()?;     // Discovery error
    let response = service.call()?;        // Network error
    
    Ok(response)
}
```

### Pattern 3: Early Return with Custom Error

```rust
pub fn validate_port(port: u16) -> SongbirdResult<()> {
    if port == 0 {
        return Err(SongbirdError::validation("Port cannot be 0"));
    }
    if port < 1024 {
        let mut err = SongbirdError::validation("Port below 1024 requires privileges");
        err.field = Some("port".to_string());
        err.suggestion = Some("Use port >= 1024".to_string());
        return Err(err);
    }
    Ok(())
}
```

### Pattern 4: Result Mapping

```rust
pub async fn fetch_data(url: &str) -> SongbirdResult<Data> {
    reqwest::get(url)
        .await
        .map_err(|e| SongbirdError::network(format!("HTTP request failed: {}", e)))?
        .json()
        .await
        .map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parse failed: {}", e),
            debug_info: None,
        })
}
```

---

## 🎨 Error Enhancement

### Adding Suggestions

```rust
let mut err = SongbirdError::configuration("Missing API key");
err.with_suggestion("Set SONGBIRD_API_KEY environment variable");
```

### Adding Context

```rust
let mut err = SongbirdError::security("Authentication failed");
err.with_context("user-login-flow");
```

### Chaining Enhancements

```rust
let mut err = SongbirdError::service("api", "Unavailable");
err.with_suggestion("Check service health")
   .with_context("request-processing");
```

---

## ✅ Best Practices

### DO ✅

- Use specific error types for clear intent
- Add suggestions when possible
- Include context for debugging
- Use `?` operator for automatic conversion
- Provide recovery hints for service errors

### DON'T ❌

- Don't use `unwrap()` or `expect()` in production code
- Don't lose error information when converting
- Don't create generic "something failed" errors
- Don't panic - return errors instead
- Don't ignore errors silently

---

## 📊 Error Severity Guide

| Severity | Error Types | Action |
|----------|-------------|--------|
| **Critical** | Security, Runtime | Immediate alerting |
| **High** | Service, Discovery, LoadBalancing | Retry with backoff |
| **Medium** | Network, Registry, Protocol | Log and retry |
| **Low** | Configuration, Validation | User notification |
| **Info** | Metrics, Event | Log only |

---

## 🔍 Debugging Tips

### 1. Enable Full Error Display

```rust
match result {
    Err(e) => {
        eprintln!("Error: {}", e);           // Display trait
        eprintln!("Debug: {:?}", e);         // Debug trait
        eprintln!("JSON: {}", serde_json::to_string_pretty(&e)?);  // Serializable!
    }
    Ok(v) => { /* ... */ }
}
```

### 2. Check Error Variant

```rust
match error {
    SongbirdError::Network { interface, .. } => {
        println!("Network issue on {}", interface.unwrap_or_default());
    }
    SongbirdError::Service { service, suggested_alternatives, .. } => {
        println!("Service {} failed. Try: {:?}", service, suggested_alternatives);
    }
    _ => println!("Other error"),
}
```

### 3. Extract Specific Information

```rust
if let SongbirdError::Discovery { backend, retry_strategy, .. } = error {
    println!("Discovery backend {} failed", backend.unwrap_or_default());
    if let Some(strategy) = retry_strategy {
        println!("Retry with: {}", strategy);
    }
}
```

---

## 🚀 Quick Reference Table

| Need | Use | Example |
|------|-----|---------|
| Config issue | `configuration()` | `SongbirdError::configuration("missing field")` |
| Network fail | `network()` | `SongbirdError::network("timeout")` |
| Auth fail | `security()` | `SongbirdError::security("unauthorized")` |
| Service down | `service()` | `SongbirdError::service("api", "unavailable")` |
| Not found | `discovery()` | `SongbirdError::discovery("no services")` |
| Validation | `validation()` | `SongbirdError::validation("invalid input")` |

---

## 📚 Related Documentation

- **Full Error System**: `crates/songbird-types/src/errors.rs`
- **Result Types**: `UNIFIED_RESULTS_QUICKREF.md`
- **Trait System**: `UNIFIED_TRAITS_QUICKREF.md`
- **Architecture**: `ARCHITECTURE_OVERVIEW.md`

---

**Need Help?** Check the error source code or ask in #songbird-dev!

✅ **This error system is production-ready and battle-tested!**
