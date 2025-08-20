# 🏗️ SONGBIRD ARCHITECTURAL PATTERNS & CODING STANDARDS

**Version**: 2.0  
**Status**: Production Standard  
**Scope**: All Songbird development  
**Authority**: Songbird Architecture Committee  

---

## 📋 TABLE OF CONTENTS

1. [Configuration Patterns](#configuration-patterns)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Type System Patterns](#type-system-patterns)
4. [Performance Patterns](#performance-patterns)
5. [Trait Design Patterns](#trait-design-patterns)
6. [Module Organization Patterns](#module-organization-patterns)
7. [Testing Patterns](#testing-patterns)
8. [Documentation Patterns](#documentation-patterns)

---

## 🔧 CONFIGURATION PATTERNS

### **MANDATORY**: Single Configuration Source

**Pattern**: All configuration MUST use `UnifiedSongbirdConfig`

```rust
// ✅ CORRECT: Use unified configuration
use songbird_config::UnifiedSongbirdConfig;

fn initialize_service() -> Result<()> {
    let config = UnifiedSongbirdConfig::from_env();
    config.validate()?;
    
    // Access specific sections
    let network_config = &config.network;
    let security_config = &config.security;
    // ...
}

// ❌ INCORRECT: Custom configuration structs
struct MyCustomConfig { /* ... */ } // DON'T DO THIS
```

### **MANDATORY**: Environment-First Configuration

**Pattern**: All configuration values MUST support environment variable override

```rust
// ✅ CORRECT: Environment-driven with smart defaults
impl Default for MyConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("SONGBIRD_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("SONGBIRD_PORT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
        }
    }
}

// ❌ INCORRECT: Hardcoded values
const HOST: &str = "localhost"; // DON'T DO THIS
```

### **MANDATORY**: Configuration Validation

**Pattern**: All configuration MUST be validated before use

```rust
// ✅ CORRECT: Built-in validation
impl UnifiedSongbirdConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.network.port == 0 {
            return Err("Network port must be specified".to_string());
        }
        // More validation...
        Ok(())
    }
}
```

---

## 🛡️ ERROR HANDLING PATTERNS

### **MANDATORY**: No Panic in Production

**Pattern**: Production code MUST NOT use `unwrap()`, `expect()`, or `panic!()`

```rust
// ✅ CORRECT: Proper error handling
fn parse_config(data: &str) -> SongbirdResult<Config> {
    serde_json::from_str(data)
        .map_err(|e| SongbirdError::config_parsing("Invalid JSON", e))
}

// ❌ INCORRECT: Panic-prone patterns
fn parse_config(data: &str) -> Config {
    serde_json::from_str(data).unwrap() // DON'T DO THIS
}
```

### **MANDATORY**: AI-First Error Responses

**Pattern**: All errors MUST provide AI-compatible context

```rust
// ✅ CORRECT: AI-first error with automation hints
return Err(SongbirdError::config_field(
    "network.port",
    "Port must be between 1024-65535".to_string(),
).with_suggestion("Set SONGBIRD_PORT environment variable")
 .with_automation_hint("config_validation"));

// ❌ INCORRECT: Generic error messages
return Err("Invalid port".into()); // DON'T DO THIS
```

### **MANDATORY**: Unified Error Hierarchy

**Pattern**: All errors MUST use `SongbirdError` hierarchy

```rust
// ✅ CORRECT: Unified error system
use songbird_errors::{SongbirdError, SongbirdResult};

fn my_function() -> SongbirdResult<Data> {
    match operation() {
        Ok(data) => Ok(data),
        Err(e) => Err(SongbirdError::service_error("my_service", e)),
    }
}
```

---

## 🎯 TYPE SYSTEM PATTERNS

### **MANDATORY**: Universal Type Usage

**Pattern**: Use universal types from `songbird_universal::adapters::types`

```rust
// ✅ CORRECT: Universal types
use songbird_universal::adapters::types::{
    UniversalMessageType,
    UniversalServiceType,
    UniversalConnectionType,
};

// ❌ INCORRECT: Local type definitions
enum MessageType { /* ... */ } // DON'T DO THIS
```

### **MANDATORY**: Type Safety First

**Pattern**: Prefer compile-time safety over runtime flexibility

```rust
// ✅ CORRECT: Type-safe approach
fn process_message(msg: UniversalMessageType) -> SongbirdResult<()> {
    match msg {
        UniversalMessageType::Request => handle_request(),
        UniversalMessageType::Response => handle_response(),
        // Compiler ensures all variants are handled
    }
}

// ❌ INCORRECT: String-based types
fn process_message(msg_type: &str) -> SongbirdResult<()> {
    match msg_type { // Runtime errors possible
        "request" => handle_request(),
        // Missing cases not caught at compile time
        _ => Err("Unknown message type".into()),
    }
}
```

### **MANDATORY**: Structured Configuration Types

**Pattern**: Use structured types instead of generic maps

```rust
// ✅ CORRECT: Structured configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub timeout: TimeoutConfig,
}

// ❌ INCORRECT: Generic maps
type Config = HashMap<String, serde_json::Value>; // DON'T DO THIS
```

---

## ⚡ PERFORMANCE PATTERNS

### **MANDATORY**: Zero-Cost Abstractions

**Pattern**: Use native async fn in traits, not `async_trait`

```rust
// ✅ CORRECT: Zero-cost native async
pub trait MyTrait: Send + Sync {
    async fn my_method(&self) -> Result<Data>;
}

// ❌ INCORRECT: async_trait overhead
#[async_trait::async_trait]
pub trait MyTrait: Send + Sync {
    async fn my_method(&self) -> Result<Data>; // DON'T DO THIS
}
```

### **MANDATORY**: Compile-Time Specialization

**Pattern**: Use generics and const generics for performance

```rust
// ✅ CORRECT: Compile-time specialization
pub struct ZeroCostRegistry<const MAX_SERVICES: usize = 1000> {
    services: [Option<ServiceInfo>; MAX_SERVICES],
}

// ❌ INCORRECT: Runtime dispatch
pub struct Registry {
    services: Vec<Box<dyn Service>>, // Runtime overhead
}
```

### **MANDATORY**: Memory Efficiency

**Pattern**: Prefer stack allocation and avoid unnecessary heap allocations

```rust
// ✅ CORRECT: Stack-allocated array
let buffer: [u8; 4096] = [0; 4096];

// ❌ INCORRECT: Unnecessary heap allocation
let buffer = vec![0u8; 4096]; // DON'T DO THIS for fixed-size buffers
```

---

## 🎨 TRAIT DESIGN PATTERNS

### **MANDATORY**: Three-Tier Trait Hierarchy

**Pattern**: Follow the canonical trait hierarchy

```rust
// Tier 1: Base trait for all primals
pub trait PrimalProvider: Send + Sync {
    async fn get_capabilities(&self) -> Vec<Capability>;
}

// Tier 2: Service lifecycle management
pub trait UniversalService: PrimalProvider {
    async fn start(&mut self) -> SongbirdResult<()>;
    async fn stop(&mut self) -> SongbirdResult<()>;
}

// Tier 3: Dynamic composition capabilities
pub trait ComposablePlugin: UniversalService {
    async fn compose_with(&self, other: &dyn ComposablePlugin) -> SongbirdResult<()>;
}
```

### **MANDATORY**: Capability-Based Design

**Pattern**: Design traits around capabilities, not implementations

```rust
// ✅ CORRECT: Capability-based trait
pub trait StorageCapable: Send + Sync {
    async fn store(&self, key: &str, data: &[u8]) -> SongbirdResult<()>;
    async fn retrieve(&self, key: &str) -> SongbirdResult<Vec<u8>>;
}

// ❌ INCORRECT: Implementation-specific trait
pub trait DatabaseConnection { // Too specific
    async fn execute_sql(&self, query: &str) -> Result<Rows>;
}
```

---

## 📁 MODULE ORGANIZATION PATTERNS

### **MANDATORY**: File Size Limits

**Pattern**: Maximum 2000 lines per file

```rust
// ✅ CORRECT: Focused modules
// src/network/mod.rs (500 lines)
// src/network/discovery.rs (400 lines)
// src/network/routing.rs (600 lines)

// ❌ INCORRECT: Monolithic files
// src/network.rs (3000 lines) // DON'T DO THIS
```

### **MANDATORY**: Logical Cohesion

**Pattern**: Group related functionality together

```rust
// ✅ CORRECT: Cohesive module structure
mod network {
    pub mod discovery;    // Service discovery
    pub mod routing;      // Message routing
    pub mod transport;    // Transport layer
}

// ❌ INCORRECT: Mixed concerns
mod utils {
    // Network, config, and error utilities mixed together
}
```

### **MANDATORY**: Clear Module Boundaries

**Pattern**: Each module should have a single responsibility

```rust
// ✅ CORRECT: Clear responsibility
/// Network discovery module - handles service discovery only
pub mod discovery {
    pub trait DiscoveryProvider { /* ... */ }
    pub struct ServiceRegistry { /* ... */ }
}
```

---

## 🧪 TESTING PATTERNS

### **MANDATORY**: Test Organization

**Pattern**: Tests MUST be properly scoped and organized

```rust
// ✅ CORRECT: Proper test scoping
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        // Unit test
    }
}

// ✅ CORRECT: Integration tests in separate files
// tests/integration_tests.rs
```

### **MANDATORY**: Mock Scoping

**Pattern**: Mocks MUST be test-only

```rust
// ✅ CORRECT: Test-scoped mock
#[cfg(test)]
pub struct MockService {
    // Only available in test builds
}

// ❌ INCORRECT: Production-accessible mock
pub struct MockService { // DON'T DO THIS
    // Available in production builds
}
```

---

## 📚 DOCUMENTATION PATTERNS

### **MANDATORY**: Deprecation Notices

**Pattern**: All deprecated items MUST include migration guides

```rust
/// **⚠️ DEPRECATION NOTICE**: This type is deprecated.
/// Please migrate to `songbird_universal::adapters::types::UniversalMessageType`.
///
/// ## Migration Guide
/// ```rust
/// // OLD (deprecated)
/// use crate::MessageType;
///
/// // NEW (universal)
/// use songbird_universal::adapters::types::UniversalMessageType as MessageType;
/// ```
#[deprecated(note = "Use UniversalMessageType instead")]
pub enum MessageType { /* ... */ }
```

### **MANDATORY**: Performance Documentation

**Pattern**: Document performance characteristics

```rust
/// **PERFORMANCE**: Uses native async in traits (Rust 1.75+) for zero-cost async - 40-60% faster than async_trait
pub trait MyTrait: Send + Sync {
    async fn my_method(&self) -> Result<Data>;
}
```

---

## 🎯 COMPLIANCE CHECKLIST

### **Before Submitting Code**:

- [ ] **Configuration**: Uses `UnifiedSongbirdConfig` only
- [ ] **Errors**: No `unwrap()`, `expect()`, or `panic!()` in production code
- [ ] **Types**: Uses universal types from `songbird_universal`
- [ ] **Performance**: Uses native async fn, not `async_trait`
- [ ] **Traits**: Follows three-tier hierarchy
- [ ] **Files**: Under 2000 lines per file
- [ ] **Tests**: Proper scoping and organization
- [ ] **Docs**: Includes migration guides for deprecated items

### **Code Review Requirements**:

- [ ] **Architecture**: Follows established patterns
- [ ] **Performance**: Zero-cost abstractions used where possible
- [ ] **Safety**: No panic sources in production paths
- [ ] **Consistency**: Matches existing codebase patterns
- [ ] **Documentation**: Clear and comprehensive

---

## 📞 PATTERN EVOLUTION

### **Adding New Patterns**:

1. **Propose**: Document new pattern with rationale
2. **Review**: Architecture committee review
3. **Pilot**: Test in limited scope
4. **Adopt**: Update this document
5. **Migrate**: Update existing code gradually

### **Pattern Violations**:

- **Minor**: Document as technical debt
- **Major**: Block merge until fixed
- **Critical**: Immediate fix required

---

**Document Maintained By**: Songbird Architecture Committee  
**Last Updated**: January 2025  
**Next Review**: Quarterly or when major patterns change 