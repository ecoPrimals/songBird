# 🛡️ **Songbird Error Handling Guide**

**Version**: 1.0  
**Last Updated**: October 12, 2025  
**Status**: ✅ **SOVEREIGN SCIENCE GRADE**

---

## 📋 **Table of Contents**

1. [Overview & Philosophy](#overview--philosophy)
2. [Error Types](#error-types)
3. [Best Practices](#best-practices)
4. [Common Patterns](#common-patterns)
5. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
6. [Testing Error Handling](#testing-error-handling)
7. [Examples & Recipes](#examples--recipes)
8. [Migration Guide](#migration-guide)

---

## 🎯 **Overview & Philosophy**

### **Sovereign Science Error Handling Principles**

**Core Philosophy**: Errors are not failures—they are opportunities to provide clarity and enable recovery.

**Five Pillars**:

1. **Explicit Over Implicit**
   - All errors must be explicitly handled
   - Use `Result<T, E>` for all fallible operations
   - Never silently ignore errors

2. **Context is King**
   - Errors should provide actionable context
   - Include relevant details for debugging
   - Help users resolve issues themselves

3. **User-Centric Messages**
   - Error messages should be understandable
   - Suggest concrete next steps
   - Avoid technical jargon when user-facing

4. **Developer-Friendly Debugging**
   - Provide stack traces when needed
   - Include relevant state information
   - Enable easy root cause analysis

5. **Type-Safe & Composable**
   - Use strong typing for error categories
   - Enable error composition and transformation
   - Support error recovery patterns

---

## 🔧 **Error Types**

### **SongbirdError** - Our Canonical Error Type

Located in `crates/songbird-types/src/errors.rs`

```rust
use songbird_types::{SongbirdError, SongbirdResult};

// Primary error type
pub enum SongbirdError {
    // Configuration errors
    ConfigError {
        message: String,
        context: Option<String>,
    },
    
    // Network errors
    NetworkError {
        message: String,
        endpoint: Option<String>,
        retry_after: Option<Duration>,
    },
    
    // Service errors
    ServiceError {
        service_name: String,
        message: String,
        recoverable: bool,
    },
    
    // Authentication/Authorization errors
    AuthError {
        message: String,
        required_permission: Option<String>,
    },
    
    // And more...
}

// Convenience type alias
pub type SongbirdResult<T> = Result<T, SongbirdError>;
```

### **Error Categories**

| Category | Use When | Recoverable |
|----------|----------|-------------|
| **ConfigError** | Configuration issues | Often |
| **NetworkError** | Network/connectivity issues | Usually |
| **ServiceError** | Service-specific errors | Depends |
| **AuthError** | Auth/permissions issues | Rarely |
| **InternalError** | Unexpected internal errors | No |
| **ValidationError** | Input validation failures | Yes |

---

## ✅ **Best Practices**

### **1. Use Result<T, E> Everywhere**

```rust
// ✅ GOOD: Explicit error handling
pub fn load_config(path: &Path) -> SongbirdResult<Config> {
    let contents = fs::read_to_string(path)
        .map_err(|e| SongbirdError::config_error(&format!(
            "Failed to read config from {}: {}",
            path.display(),
            e
        )))?;
    
    let config: Config = toml::from_str(&contents)
        .map_err(|e| SongbirdError::config_error(&format!(
            "Failed to parse config: {}",
            e
        )))?;
    
    Ok(config)
}

// ❌ BAD: Panics in library code
pub fn load_config(path: &Path) -> Config {
    let contents = fs::read_to_string(path).unwrap(); // DON'T DO THIS!
    toml::from_str(&contents).unwrap() // OR THIS!
}
```

### **2. Add Context to Errors**

```rust
// ✅ GOOD: Rich context
pub async fn connect_to_service(name: &str) -> SongbirdResult<Connection> {
    let endpoint = get_endpoint(name)
        .map_err(|e| SongbirdError::service_error(
            name,
            &format!("Failed to resolve endpoint: {}", e)
        ))?;
    
    let conn = TcpStream::connect(&endpoint.url()).await
        .map_err(|e| SongbirdError::network_error(&format!(
            "Failed to connect to {} at {}: {}. Check that the service is running.",
            name,
            endpoint.url(),
            e
        )))?;
    
    Ok(Connection::new(conn))
}

// ❌ BAD: No context
pub async fn connect_to_service(name: &str) -> SongbirdResult<Connection> {
    let endpoint = get_endpoint(name)?; // What failed?
    let conn = TcpStream::connect(&endpoint.url()).await?; // Why did it fail?
    Ok(Connection::new(conn))
}
```

### **3. Use expect() in Tests with Clear Messages**

```rust
// ✅ GOOD: Test code with descriptive expect()
#[tokio::test]
async fn test_service_connection() {
    let config = Config::load()
        .expect("test config should load successfully");
    
    let service = connect_to_service("test-service")
        .await
        .expect("should connect to test service on localhost:8080");
    
    assert!(service.is_connected());
}

// ❌ BAD: Unwrap with no message
#[tokio::test]
async fn test_service_connection() {
    let config = Config::load().unwrap(); // What failed?
    let service = connect_to_service("test-service").await.unwrap(); // Why?
    assert!(service.is_connected());
}
```

### **4. Document When unwrap() is Safe**

```rust
// ✅ GOOD: Documented unwrap
pub fn parse_port(s: &str) -> u16 {
    // JUSTIFICATION: This unwrap is safe because:
    // - Invariant: Input is validated by clap to be a valid u16
    // - Invariant: Regex pattern ensures only digits
    // - Fallback: Default value provided if parsing fails
    s.parse().unwrap_or(8080)
}

// Better: Avoid unwrap entirely
pub fn parse_port(s: &str) -> u16 {
    s.parse().unwrap_or(8080) // Provides fallback
}
```

### **5. Proper Error Propagation**

```rust
// ✅ GOOD: Use ? operator
pub async fn process_request(req: Request) -> SongbirdResult<Response> {
    let validated = validate_request(&req)?;
    let result = call_service(validated).await?;
    let response = format_response(result)?;
    Ok(response)
}

// ❌ BAD: Nested match statements
pub async fn process_request(req: Request) -> SongbirdResult<Response> {
    match validate_request(&req) {
        Ok(validated) => {
            match call_service(validated).await {
                Ok(result) => {
                    match format_response(result) {
                        Ok(response) => Ok(response),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
```

---

## 🎨 **Common Patterns**

### **Pattern 1: Error Conversion with map_err**

```rust
use std::fs;
use songbird_types::{SongbirdError, SongbirdResult};

pub fn read_config(path: &Path) -> SongbirdResult<String> {
    fs::read_to_string(path)
        .map_err(|e| SongbirdError::config_error(&format!(
            "Failed to read config file at {}: {}",
            path.display(),
            e
        )))
}
```

### **Pattern 2: Optional to Result Conversion**

```rust
pub fn get_required_config(config: &Config) -> SongbirdResult<String> {
    config.api_key
        .clone()
        .ok_or_else(|| SongbirdError::config_error(
            "API key is required but not configured. \
             Set SONGBIRD_API_KEY environment variable or add to config file."
        ))
}
```

### **Pattern 3: Multiple Error Sources**

```rust
pub async fn initialize_system() -> SongbirdResult<System> {
    // Each operation can fail with different errors
    let config = load_config()?;
    let db = connect_database(&config.db_url).await?;
    let cache = init_cache(&config.cache_config)?;
    
    Ok(System { config, db, cache })
}
```

### **Pattern 4: Error Recovery**

```rust
pub async fn fetch_with_retry(url: &str) -> SongbirdResult<Response> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match fetch_url(url).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < max_attempts => {
                tracing::warn!(
                    "Fetch attempt {} failed: {}. Retrying...",
                    attempts,
                    e
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => {
                return Err(SongbirdError::network_error(&format!(
                    "Failed to fetch {} after {} attempts: {}",
                    url,
                    max_attempts,
                    e
                )));
            }
        }
    }
}
```

### **Pattern 5: Collecting Multiple Errors**

```rust
pub fn validate_all_services(services: &[Service]) -> SongbirdResult<()> {
    let mut errors = Vec::new();
    
    for service in services {
        if let Err(e) = validate_service(service) {
            errors.push(format!("Service '{}': {}", service.name, e));
        }
    }
    
    if !errors.is_empty() {
        return Err(SongbirdError::validation_error(&format!(
            "Service validation failed:\n{}",
            errors.join("\n")
        )));
    }
    
    Ok(())
}
```

---

## ⛔ **Anti-Patterns to Avoid**

### **Anti-Pattern 1: Silent Failures**

```rust
// ❌ BAD: Silently ignoring errors
let _ = save_to_disk(data); // Error is lost!

// ✅ GOOD: Handle or propagate
save_to_disk(data)?;
// or
if let Err(e) = save_to_disk(data) {
    tracing::error!("Failed to save data: {}", e);
    // Handle appropriately
}
```

### **Anti-Pattern 2: Generic Error Messages**

```rust
// ❌ BAD: Unhelpful error message
return Err(SongbirdError::internal_error("something failed"));

// ✅ GOOD: Specific, actionable message
return Err(SongbirdError::service_error(
    "authentication",
    "Failed to verify JWT token: signature invalid. \
     Check that AUTH_SECRET environment variable matches the token issuer."
));
```

### **Anti-Pattern 3: Panicking in Library Code**

```rust
// ❌ BAD: Panic in library function
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Cannot divide by zero!"); // DON'T DO THIS!
    }
    a / b
}

// ✅ GOOD: Return Result
pub fn divide(a: i32, b: i32) -> SongbirdResult<i32> {
    if b == 0 {
        return Err(SongbirdError::validation_error(
            "Division by zero is not allowed"
        ));
    }
    Ok(a / b)
}
```

### **Anti-Pattern 4: Swallowing Errors**

```rust
// ❌ BAD: Converting errors to Ok(())
pub async fn process_all(items: Vec<Item>) -> SongbirdResult<()> {
    for item in items {
        let _ = process_item(item).await; // Errors lost!
    }
    Ok(())
}

// ✅ GOOD: Propagate or handle explicitly
pub async fn process_all(items: Vec<Item>) -> SongbirdResult<Vec<ProcessedItem>> {
    let mut results = Vec::new();
    for item in items {
        let processed = process_item(item).await?;
        results.push(processed);
    }
    Ok(results)
}
```

---

## 🧪 **Testing Error Handling**

### **Testing Success Cases**

```rust
#[tokio::test]
async fn test_valid_config_loads() {
    let config = load_config("tests/fixtures/valid_config.toml")
        .await
        .expect("should load valid config successfully");
    
    assert_eq!(config.port, 8080);
    assert!(config.enabled);
}
```

### **Testing Error Cases**

```rust
#[tokio::test]
async fn test_missing_config_returns_error() {
    let result = load_config("nonexistent.toml").await;
    
    assert!(result.is_err(), "should fail for missing file");
    
    match result {
        Err(SongbirdError::ConfigError { message, .. }) => {
            assert!(message.contains("Failed to read config"));
        }
        _ => panic!("expected ConfigError"),
    }
}

#[tokio::test]
async fn test_invalid_config_returns_error() {
    let result = load_config("tests/fixtures/invalid_config.toml").await;
    
    assert!(result.is_err(), "should fail for invalid config");
    
    let err = result.unwrap_err();
    assert!(matches!(err, SongbirdError::ConfigError { .. }));
}
```

### **Testing Error Messages**

```rust
#[tokio::test]
async fn test_error_message_includes_context() {
    let result = connect_to_service("nonexistent-service").await;
    
    assert!(result.is_err());
    
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("nonexistent-service"));
    assert!(error_message.contains("Failed to"));
}
```

---

## 📚 **Examples & Recipes**

### **Recipe 1: Configuration Loading**

```rust
use std::fs;
use std::path::Path;
use songbird_types::{SongbirdError, SongbirdResult};

pub fn load_config(path: &Path) -> SongbirdResult<Config> {
    // Check file exists
    if !path.exists() {
        return Err(SongbirdError::config_error(&format!(
            "Config file not found at {}. Create it with 'songbird init'",
            path.display()
        )));
    }
    
    // Read file
    let contents = fs::read_to_string(path)
        .map_err(|e| SongbirdError::config_error(&format!(
            "Failed to read config from {}: {}. Check file permissions.",
            path.display(),
            e
        )))?;
    
    // Parse TOML
    let config: Config = toml::from_str(&contents)
        .map_err(|e| SongbirdError::config_error(&format!(
            "Failed to parse config file: {}. Check TOML syntax.",
            e
        )))?;
    
    // Validate
    config.validate()?;
    
    Ok(config)
}
```

### **Recipe 2: Network Requests**

```rust
pub async fn fetch_data(url: &str) -> SongbirdResult<Data> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| SongbirdError::network_error(&format!(
            "Failed to fetch data from {}: {}. Check network connectivity.",
            url,
            e
        )))?;
    
    if !response.status().is_success() {
        return Err(SongbirdError::service_error(
            "api",
            &format!(
                "API returned error status {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )
        ));
    }
    
    let data = response.json::<Data>()
        .await
        .map_err(|e| SongbirdError::service_error(
            "api",
            &format!("Failed to parse response: {}", e)
        ))?;
    
    Ok(data)
}
```

### **Recipe 3: Database Operations**

```rust
pub async fn save_user(db: &Database, user: &User) -> SongbirdResult<UserId> {
    // Validate before saving
    user.validate()?;
    
    // Execute database operation
    let id = db.execute(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
        &[&user.name, &user.email]
    )
    .await
    .map_err(|e| {
        // Check for specific database errors
        if is_duplicate_key_error(&e) {
            SongbirdError::validation_error(&format!(
                "User with email {} already exists",
                user.email
            ))
        } else {
            SongbirdError::internal_error(&format!(
                "Database error while saving user: {}",
                e
            ))
        }
    })?;
    
    Ok(id)
}
```

---

## 🔄 **Migration Guide**

### **Converting Existing Code**

**Step 1: Replace unwrap() in Production Code**

```rust
// Before
let value = some_operation().unwrap();

// After
let value = some_operation()
    .map_err(|e| SongbirdError::internal_error(&format!(
        "Operation failed: {}",
        e
    )))?;
```

**Step 2: Add Context to Errors**

```rust
// Before
database.save(data)?;

// After
database.save(data)
    .map_err(|e| SongbirdError::internal_error(&format!(
        "Failed to save data to database: {}. Check database connectivity.",
        e
    )))?;
```

**Step 3: Convert Test unwrap() to expect()**

```rust
// Before
#[test]
fn test_something() {
    let value = setup().unwrap();
    assert_eq!(value, 42);
}

// After
#[test]
fn test_something() {
    let value = setup()
        .expect("test setup should succeed");
    assert_eq!(value, 42);
}
```

---

## 🎯 **Quick Reference**

### **When to Use What**

| Situation | Use | Example |
|-----------|-----|---------|
| Production code | `Result<T, E>` + `?` | `load_config()?` |
| Test code | `expect("message")` | `load_config().expect("test should load")` |
| Infallible ops | Document + `unwrap()` or refactor | See "Documented unwrap" above |
| Error conversion | `map_err()` | `fs::read().map_err(...)?` |
| Optional handling | `ok_or_else()` | `opt.ok_or_else(|| error)?` |
| Multiple errors | Collect + return | See Pattern 5 above |
| Recovery | Loop + match | See Pattern 4 above |

### **Error Message Checklist**

- [ ] What failed?
- [ ] Why did it fail?
- [ ] What was the input/context?
- [ ] How can the user fix it?
- [ ] Is the message user-friendly?

---

## 🏆 **Sovereign Science Standards**

### **Zero Tolerance**

- ❌ `unwrap()` in production critical paths
- ❌ Generic error messages
- ❌ Silent error swallowing
- ❌ Panics in library code
- ❌ Errors without context

### **Acceptable**

- ✅ `unwrap()` with JUSTIFICATION comments
- ✅ `expect()` in tests with clear messages
- ✅ `unwrap()` in examples/demos for brevity
- ✅ Panics for truly unrecoverable states (with documentation)

### **Excellence**

- ✅ Rich error context
- ✅ Actionable error messages
- ✅ Composable error types
- ✅ Comprehensive error tests
- ✅ Recovery patterns where appropriate

---

## 📖 **Additional Resources**

**Internal Documentation**:
- `crates/songbird-types/src/errors.rs` - Error type definitions
- `SOVEREIGN_SCIENCE_GRADE_100.md` - Overall standards
- `PATH_TO_100_ACTION_NOW.md` - Roadmap

**Rust Error Handling**:
- [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Rust by Example - Error Handling](https://doc.rust-lang.org/rust-by-example/error.html)
- [Error Handling Crate: anyhow](https://docs.rs/anyhow/)
- [Error Handling Crate: thiserror](https://docs.rs/thiserror/)

---

## ✅ **Summary**

**Key Takeaways**:

1. **Use Result everywhere** - Explicit error handling
2. **Add context** - Help users and developers
3. **Test error cases** - Verify error behavior
4. **Use expect() in tests** - Clear failure messages
5. **Document exceptions** - When unwrap() is justified

**Remember**: Good error handling is not about preventing all errors—it's about handling them gracefully and providing clarity when they occur.

---

**Version**: 1.0  
**Last Updated**: October 12, 2025  
**Maintained By**: Songbird Core Team  
**Status**: ✅ **SOVEREIGN SCIENCE GRADE**

🎯 **ERROR HANDLING: NOT JUST CODE, BUT USER EXPERIENCE** 🎯

