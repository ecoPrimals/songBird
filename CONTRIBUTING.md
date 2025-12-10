# Contributing to Songbird

Thank you for your interest in contributing to Songbird, the ecoPrimals distributed orchestration system!

---

## 🎯 Code Quality Standards

### Error Handling Policy

#### Production Code: **No Unwrap**

**Rule**: Production code MUST use proper `Result<T, E>` propagation with the `?` operator.

```rust
// ❌ BAD - Don't use unwrap in production code
pub fn load_config() -> Config {
    let file = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&file).unwrap()
}

// ✅ GOOD - Use Result and ? operator
pub fn load_config() -> SongbirdResult<Config> {
    let file = std::fs::read_to_string("config.toml")?;
    let config = toml::from_str(&file)?;
    Ok(config)
}
```

**Rationale**: Unwraps cause panics. Production systems must handle errors gracefully.

#### Test Code: **Unwrap is Idiomatic**

**Rule**: Test code MAY use `.unwrap()` and `.expect()` for cleaner test logic.

```rust
// ✅ GOOD - Unwrap is acceptable in tests
#[tokio::test]
async fn test_service_discovery() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);
    
    // Unwrap here is fine - test will fail with clear panic message
    let providers = adapter.find_capability_providers("compute").await;
    assert!(!providers.is_empty());
}
```

**Rationale**: Test panics are acceptable - they indicate test failures with clear stack traces.

#### Example Code: **Documented Unwrap**

**Rule**: Example code MAY use `.unwrap()` with clear comments explaining why.

```rust
// ✅ GOOD - Example with documented unwrap
/// # Example
/// ```rust
/// use songbird_universal::UniversalCapabilityAdapter;
/// 
/// let config = DiscoveryConfig::default();
/// let adapter = UniversalCapabilityAdapter::new(config);
/// 
/// // For this example, we unwrap for brevity
/// // In production, use proper error handling with ?
/// let providers = adapter.find_providers("compute").await.unwrap();
/// ```
```

### Enforcing the Policy

#### Critical Crates

For security-critical crates, add this to the crate root (`lib.rs`):

```rust
// Deny unwrap in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

// Allow in tests
#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
```

**Critical crates** (consider adding deny pragmas):
- `songbird-types` (core types)
- `songbird-config` (configuration)
- `songbird-orchestrator` (orchestration logic)
- `songbird-network-federation` (networking)

#### Code Review

All PRs will be reviewed for:
1. No unwraps in production code paths
2. Proper `Result<T, E>` usage
3. `?` operator for error propagation
4. Tests can use unwrap/expect

---

## 🦀 Idiomatic Rust Patterns

### Error Types

Use the canonical `SongbirdResult<T>` type:

```rust
use songbird_types::{SongbirdResult, SongbirdError};

pub async fn my_function() -> SongbirdResult<Value> {
    let result = fallible_operation()?;
    Ok(result)
}
```

### Async/Await

Use native `async fn` (not `#[async_trait]` unless necessary):

```rust
// ✅ GOOD - Native async
pub async fn discover_services(&self) -> SongbirdResult<Vec<Service>> {
    self.discovery_engine.find_all().await
}

// ❌ AVOID - async_trait has overhead
#[async_trait]
trait MyTrait {
    async fn method(&self) -> Result<()>;
}
```

### Zero-Copy Patterns

Prefer borrowing over cloning:

```rust
// ❌ AVOID - Unnecessary clone
fn process_name(name: String) { }
let n = service.name.clone();
process_name(n);

// ✅ GOOD - Borrow
fn process_name(name: &str) { }
process_name(&service.name);
```

### Module Organization

Keep files under 1000 lines:
- Implementation in main file
- Tests in separate `tests.rs` or module `tests/`
- Large modules split by concern

---

## 🧪 Testing Standards

### Test Coverage Target

**Minimum**: 90% code coverage for library code

Run coverage with:
```bash
cargo llvm-cov --workspace --lib --html
```

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            // Test configuration
        }
    }

    #[tokio::test]
    async fn test_feature_works() {
        // Arrange
        let config = create_test_config();
        let adapter = Adapter::new(config);
        
        // Act
        let result = adapter.do_something().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Test Categories

1. **Unit Tests**: In `#[cfg(test)]` modules or `tests.rs` files
2. **Integration Tests**: In `tests/integration/` directory
3. **E2E Tests**: In `tests/e2e/` directory
4. **Chaos Tests**: In `tests/chaos/` directory
5. **Benchmarks**: In `benches/` directory

---

## 🔒 Safety & Security

### Unsafe Code

**Minimize unsafe usage**. When necessary:

1. Document with clear safety comments
2. Explain invariants being maintained
3. Get peer review
4. Consider alternatives first

```rust
// ✅ GOOD - Documented unsafe
/// SAFETY: This is safe because:
/// 1. The pointer is guaranteed to be valid
/// 2. The lifetime is bounded by 'a
/// 3. No mutable aliasing occurs
unsafe {
    // unsafe operation
}
```

### Human Dignity & Sovereignty

This project follows strict ethical guidelines:
- ✅ No surveillance code
- ✅ No user tracking without explicit consent
- ✅ Privacy-first architecture
- ✅ User control over data
- ✅ Transparent operations

See `specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md` for details.

---

## 📝 Documentation Standards

### Doc Comments

All public APIs must have doc comments:

```rust
/// Discovers capabilities for a specific primal
///
/// # Arguments
///
/// * `primal_name` - The name of the primal to discover
///
/// # Errors
///
/// Returns an error if:
/// - The primal is unreachable
/// - The response is invalid
///
/// # Examples
///
/// ```rust,no_run
/// # use songbird_universal::UniversalCapabilityAdapter;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let adapter = UniversalCapabilityAdapter::new(config);
/// let capabilities = adapter.discover_capabilities("compute").await?;
/// # Ok(())
/// # }
/// ```
pub async fn discover_capabilities(
    &self,
    primal_name: &str,
) -> SongbirdResult<Vec<Capability>> {
    // Implementation
}
```

---

## 🚀 Pull Request Process

1. **Create a branch**: `feature/your-feature-name`
2. **Write tests**: Ensure >90% coverage
3. **Run checks**:
   ```bash
   cargo fmt
   cargo clippy --workspace -- -D warnings
   cargo test --workspace --lib
   ```
4. **Update docs**: Keep documentation in sync
5. **Submit PR**: Include description and rationale
6. **Code review**: Address feedback
7. **Merge**: Squash and merge when approved

---

## 📊 Code Quality Checklist

Before submitting a PR:

- [ ] All tests passing (`cargo test --workspace --lib`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Clippy clean (`cargo clippy --workspace -- -D warnings`)
- [ ] No unwraps in production code
- [ ] Proper error handling with `?` operator
- [ ] Doc comments on public APIs
- [ ] Files under 1000 lines
- [ ] Test coverage >90% for new code
- [ ] No unsafe code (or justified with safety comments)
- [ ] Idiomatic Rust patterns followed

---

## 🎓 Learning Resources

### Rust Best Practices
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Project-Specific
- `specs/` - Architecture specifications
- `docs/` - Detailed documentation
- `ARCHITECTURE.md` - System architecture
- `README.md` - Quick start guide

---

## 💬 Communication

- **Issues**: Use GitHub issues for bugs and features
- **Discussions**: Use GitHub discussions for questions
- **PRs**: Keep focused and well-documented

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

**Thank you for contributing to Songbird!** 🎵✨

Your efforts help build a production-grade, ethical distributed orchestration system.
