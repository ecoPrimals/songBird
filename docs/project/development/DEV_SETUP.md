# Development Setup

This guide covers the complete development environment setup for contributing to the Songbird Orchestrator.

## 🔧 Prerequisites

### Required Software
- **Rust**: 1.70 or later (latest stable recommended)
- **Git**: Latest version
- **IDE**: VS Code, IntelliJ IDEA, or preferred Rust IDE

### System Requirements
- **OS**: Linux, macOS, or Windows with WSL2
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Storage**: 2GB free space for dependencies and builds

## 🚀 Initial Setup

### 1. Install Rust
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart shell or source environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Clone Repository
```bash
# Clone the repository
git clone https://github.com/your-org/songbird-orchestrator.git
cd songbird-orchestrator

# Verify repository structure
ls -la
```

### 3. Install Development Dependencies
```bash
# Install additional Rust components
rustup component add clippy rustfmt

# Install helpful cargo tools
cargo install cargo-edit cargo-watch cargo-tree
```

## 🏗️ Build & Verification

### Initial Build
```bash
# Build the library
cargo build

# Run tests to verify setup
cargo test

# Check for issues
cargo check
cargo clippy
```

### Expected Results
- **Build**: Should complete without errors
- **Tests**: 97 tests should pass
- **Check**: Should show 0 errors (2 warnings acceptable)

## 🔍 Development Workflow

### Code Formatting
```bash
# Format code before commits
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Quality Checks
```bash
# Run clippy for lint checks
cargo clippy -- -D warnings

# Run tests
cargo test

# Build examples
cargo build --examples
```

### Continuous Development
```bash
# Watch for changes and rebuild
cargo watch -x check -x test

# Run specific example
cargo run --example api_demo
cargo run --example websocket_demo
```

## 🧪 Testing Setup

### Test Categories
- **Unit Tests**: `cargo test --lib`
- **Integration Tests**: `cargo test --test integration`
- **Example Tests**: `cargo test --example *`

### Test Configuration
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_orchestrator_creation

# Test with specific features
cargo test --features "full"
```

## 📋 IDE Configuration

### VS Code (Recommended)
Install these extensions:
- **rust-analyzer**: Primary Rust language server
- **CodeLLDB**: Debugging support
- **Better TOML**: TOML file support
- **crates**: Dependency management

#### VS Code Settings
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true
}
```

### IntelliJ IDEA
Install:
- **Rust plugin**: Full Rust support
- **TOML plugin**: Configuration file support

## 🔐 Environment Variables

### Development Environment
```bash
# Optional: Set log level for development
export RUST_LOG=debug

# Optional: Set backtrace for debugging
export RUST_BACKTRACE=1

# Optional: Enable colored output
export CARGO_TERM_COLOR=always
```

## 📊 Performance Monitoring

### Benchmarking
```bash
# Run benchmarks (if available)
cargo bench

# Profile builds
cargo build --release --profile release
```

### Memory Usage
```bash
# Check memory usage during tests
cargo test --release -- --nocapture
```

## 🚨 Troubleshooting

### Common Issues

#### Rust Version Issues
```bash
# Update Rust to latest stable
rustup update stable
rustup default stable
```

#### Dependency Issues
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

#### IDE Issues
```bash
# Restart rust-analyzer
# VS Code: Ctrl+Shift+P → "Rust Analyzer: Restart Server"
# Or restart IDE
```

### Build Errors
```bash
# Check for outdated dependencies
cargo tree

# Verify feature flags
cargo check --features "full"
```

## 📚 Additional Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)

### Development Tools
- [cargo-edit](https://github.com/killercup/cargo-edit)
- [cargo-watch](https://github.com/watchexec/cargo-watch)
- [rustfmt](https://github.com/rust-lang/rustfmt)

## ✅ Setup Verification Checklist

Before contributing, verify:

- [ ] Rust 1.70+ installed and working
- [ ] Repository cloned and builds successfully
- [ ] All tests pass (`cargo test`)
- [ ] Code formatting works (`cargo fmt`)
- [ ] Linting works (`cargo clippy`)
- [ ] Examples compile (`cargo build --examples`)
- [ ] IDE/editor configured with Rust support

## 🤝 Contributing

Once setup is complete:
1. Create feature branch from `main`
2. Make changes following project conventions
3. Run full test suite before commit
4. Submit pull request with clear description

---

**Need Help?** Open an issue in the repository if you encounter setup problems. 