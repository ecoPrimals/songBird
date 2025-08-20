# 🎯 CANONICAL RETURN TYPE WRAPPER SYSTEM

## **UNIVERSAL STANDARD FOR SONGBIRD CODEBASE**

This document establishes the **single, canonical system** for all return types across the entire Songbird ecosystem, ensuring consistency, AI-first compliance, and maintainability.

---

## **1. CANONICAL TYPE HIERARCHY**

### **Primary Types (MANDATORY)**

```rust
// 1. CANONICAL RESULT TYPE - Use for ALL fallible operations
pub type SongbirdResult<T> = std::result::Result<AIFirstResponse<T>, SongbirdError>;

// 2. CANONICAL SUCCESS WRAPPER - Contains all responses
pub struct AIFirstResponse<T> {
    pub success: bool,
    pub data: T,
    pub error: Option<AIFirstError>,
    pub request_id: Uuid,
    pub processing_time_ms: u64,
    pub ai_metadata: AIResponseMetadata,
    pub human_context: Option<HumanInteractionContext>,
    pub confidence_score: f64,
    pub suggested_actions: Vec<SuggestedAction>,
}

// 3. CANONICAL ERROR TYPE - Use for ALL error cases
pub enum SongbirdError {
    // ... unified error variants
}
```

---

## **2. USAGE PATTERNS (MANDATORY)**

### **✅ CORRECT PATTERNS**

```rust
// PUBLIC API FUNCTIONS - Always return SongbirdResult<T>
pub async fn service_operation() -> SongbirdResult<String> {
    Ok(success("Operation completed".to_string()))
}

// INTERNAL FUNCTIONS - Always return SongbirdResult<T>
async fn internal_helper() -> SongbirdResult<()> {
    // Use success() helper for unit type
    Ok(success(()))
}

// ERROR HANDLING - Always use SongbirdError
fn handle_error() -> SongbirdResult<Data> {
    Err(SongbirdError::operation_error("Failed"))
}
```

### **❌ FORBIDDEN PATTERNS**

```rust
// NEVER use raw Result types
fn bad_function() -> Result<String, Box<dyn Error>> { } // ❌ FORBIDDEN

// NEVER return unwrapped types directly
fn bad_direct() -> String { } // ❌ FORBIDDEN (unless pure utility)

// NEVER use Ok(()) directly - use success(())
Ok(()) // ❌ FORBIDDEN - Use Ok(success(())) instead

// NEVER use custom error types
Result<T, CustomError> // ❌ FORBIDDEN - Use SongbirdResult<T>
```

---

## **3. HELPER FUNCTIONS (CANONICAL)**

```rust
// SUCCESS HELPERS - Use these for all success cases
pub fn success<T>(data: T) -> AIFirstResponse<T>
pub fn success_result<T>(data: T) -> SongbirdResult<T>

// ERROR HELPERS - Use these for all error cases  
pub fn error_result<T>(error: SongbirdError) -> SongbirdResult<T>
pub fn operation_error(message: impl Into<String>) -> SongbirdError
pub fn config_error(message: impl Into<String>) -> SongbirdError
pub fn network_error(message: impl Into<String>) -> SongbirdError
```

---

## **4. MIGRATION PATTERNS**

### **Pattern 1: Ok(()) → Ok(success(()))**
```rust
// BEFORE ❌
fn old_function() -> SongbirdResult<()> {
    Ok(())
}

// AFTER ✅
fn new_function() -> SongbirdResult<()> {
    Ok(success(()))
}
```

### **Pattern 2: Raw Result → SongbirdResult**
```rust
// BEFORE ❌
fn old_function() -> Result<String, Box<dyn Error>> {
    Ok("data".to_string())
}

// AFTER ✅
fn new_function() -> SongbirdResult<String> {
    Ok(success("data".to_string()))
}
```

### **Pattern 3: Direct Return → Wrapped Return**
```rust
// BEFORE ❌
fn old_function() -> String {
    "data".to_string()
}

// AFTER ✅
fn new_function() -> SongbirdResult<String> {
    Ok(success("data".to_string()))
}
```

---

## **5. SPECIAL CASES**

### **Pure Utility Functions (Exception)**
```rust
// Pure utilities MAY return unwrapped types
fn calculate_hash(data: &[u8]) -> u64 { } // ✅ ALLOWED
fn format_timestamp(time: SystemTime) -> String { } // ✅ ALLOWED

// But prefer wrapped even for utilities when possible
fn calculate_hash_safe(data: &[u8]) -> SongbirdResult<u64> { } // ✅ PREFERRED
```

### **Trait Implementations**
```rust
// When implementing external traits, follow trait signature
impl Display for MyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { } // ✅ REQUIRED
}

// For internal traits, use canonical patterns
trait MySongbirdTrait {
    async fn my_method(&self) -> SongbirdResult<String>; // ✅ CANONICAL
}
```

---

## **6. ENFORCEMENT RULES**

### **MANDATORY COMPLIANCE**
- ✅ ALL public APIs MUST use `SongbirdResult<T>`
- ✅ ALL internal functions SHOULD use `SongbirdResult<T>`
- ✅ ALL errors MUST use `SongbirdError` variants
- ✅ ALL success cases MUST use `success()` helper

### **AUTOMATED CHECKS**
- Lint rule: No `Result<T, E>` where `E != SongbirdError`
- Lint rule: No `Ok(())` without `success()` wrapper
- Lint rule: No custom error types in function signatures
- CI check: All public functions return `SongbirdResult<T>`

---

## **7. IMPLEMENTATION STRATEGY**

### **Phase 1: Core Modules (CURRENT)**
1. Fix all `Ok(())` → `Ok(success(()))` patterns
2. Fix all raw `Result<T, E>` → `SongbirdResult<T>` patterns
3. Ensure all error constructors use canonical helpers

### **Phase 2: Cross-Module Consistency**
1. Standardize all trait definitions
2. Update all public API signatures
3. Migrate all internal functions

### **Phase 3: Validation & Enforcement**
1. Add automated linting rules
2. Add CI checks for compliance
3. Document exceptions and rationale

---

## **8. BENEFITS OF CANONICAL SYSTEM**

### **Developer Experience**
- ✅ Predictable return types across all modules
- ✅ Consistent error handling patterns
- ✅ Reduced cognitive load
- ✅ Better IDE support and tooling

### **AI-First Compliance**
- ✅ Rich metadata for AI decision making
- ✅ Consistent confidence scoring
- ✅ Structured error recovery hints
- ✅ Universal tracing and correlation

### **Maintenance & Evolution**
- ✅ Single point of change for wrapper evolution
- ✅ Consistent upgrade paths
- ✅ Reduced technical debt
- ✅ Clear architectural boundaries

---

## **9. QUICK REFERENCE**

### **ALWAYS USE:**
```rust
SongbirdResult<T>           // For all fallible operations
Ok(success(data))           // For all success cases
Ok(success(()))             // For unit success cases
Err(SongbirdError::...)     // For all error cases
```

### **NEVER USE:**
```rust
Result<T, CustomError>      // Use SongbirdResult<T> instead
Ok(data)                    // Use Ok(success(data)) instead
Ok(())                      // Use Ok(success(())) instead
panic!(), unwrap()          // Use safe alternatives
```

---

This canonical system ensures **100% consistency** across the entire Songbird ecosystem while maintaining AI-first compliance and developer productivity. 