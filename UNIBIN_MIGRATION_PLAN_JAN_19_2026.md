# 🎯 Songbird UniBin Migration Plan

**Date**: January 19, 2026  
**Goal**: Consolidate 5 separate binaries → 1 unified `songbird` binary  
**Approach**: Deep debt solution with modern idiomatic Rust

---

## 📊 CURRENT STATE ANALYSIS

### **Existing Binaries** (5)

1. **songbird-orchestrator** (`crates/songbird-orchestrator/src/main.rs`)
   - Size: 580 lines
   - Already has clap CLI structure! (server, doctor, config)
   - Main service mode
   - ✅ Good foundation to build on

2. **songbird-cli** (`crates/songbird-cli/src/main.rs`)
   - CLI interface for interacting with Songbird
   - Needs review (not opened yet)

3. **songbird-compute-bridge** (`crates/songbird-compute-bridge/src/main.rs`)
   - Size: 474 lines
   - Compute service bridge
   - Agnostic, capability-based

4. **songbird-remote-deploy** (`crates/songbird-remote-deploy/src/main.rs`)
   - Size: 501 lines
   - SSH-based deployment tool
   - Deploy, DeployHttp, List, Status subcommands

5. **songbird-rendezvous** (`rendezvous/src/main.rs`)
   - Size: 111 lines
   - Rendezvous server for internet federation
   - Privacy-first P2P coordination

---

## 🎯 TARGET STATE (UniBin)

### **Single Binary**: `songbird`

```bash
# Main service (from songbird-orchestrator)
songbird server [OPTIONS]
songbird doctor [--comprehensive] [--format text|json|yaml]
songbird config <show|validate|init>

# CLI interface (from songbird-cli)
songbird cli <SUBCOMMAND>

# Compute bridge (from songbird-compute-bridge)
songbird compute-bridge [OPTIONS]

# Deployment (from songbird-remote-deploy)
songbird deploy <deploy|deploy-http|list|status>

# Rendezvous (from songbird-rendezvous)
songbird rendezvous [OPTIONS]

# Standard commands
songbird --help
songbird --version
```

---

## 🏗️ ARCHITECTURE DESIGN

### **Directory Structure**

```
songbird/
├── src/
│   └── main.rs              # NEW: Unified entry point
├── crates/
│   ├── songbird-orchestrator/
│   │   ├── src/
│   │   │   ├── lib.rs       # REFACTOR: main.rs → lib.rs
│   │   │   └── ...          # Existing modules
│   │   └── Cargo.toml
│   ├── songbird-cli/
│   │   ├── src/
│   │   │   ├── lib.rs       # REFACTOR: main.rs → lib.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   ├── songbird-compute-bridge/
│   │   ├── src/
│   │   │   ├── lib.rs       # REFACTOR: main.rs → lib.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   ├── songbird-remote-deploy/
│   │   ├── src/
│   │   │   ├── lib.rs       # REFACTOR: main.rs → lib.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   └── rendezvous/
│       ├── src/
│       │   ├── lib.rs       # REFACTOR: main.rs → lib.rs
│       │   └── ...
│       └── Cargo.toml
└── Cargo.toml               # UPDATE: Single [[bin]] entry
```

### **Unified Entry Point** (`src/main.rs`)

```rust
//! Songbird - Network Orchestration & Discovery Primal
//!
//! UniBin Architecture (Ecosystem Standard v1.0.0)
//! Single binary with multiple operational modes

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Network Orchestration & Discovery Primal")]
#[command(version)]
#[command(author = "ecoPrimals <contact@ecoprimals.dev>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird orchestrator (main service)
    Server {
        #[command(flatten)]
        args: songbird_orchestrator::ServerArgs,
    },
    
    /// Run health diagnostics
    Doctor {
        #[command(flatten)]
        args: songbird_orchestrator::DoctorArgs,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        config_cmd: songbird_orchestrator::ConfigCommands,
    },
    
    /// Interactive CLI
    Cli {
        #[command(flatten)]
        args: songbird_cli::CliArgs,
    },
    
    /// Compute bridge service
    ComputeBridge {
        #[command(flatten)]
        args: songbird_compute_bridge::Args,
    },
    
    /// Deploy services to remote towers
    Deploy {
        #[command(subcommand)]
        deploy_cmd: songbird_remote_deploy::Commands,
    },
    
    /// Rendezvous server
    Rendezvous {
        #[command(flatten)]
        args: rendezvous::Args,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { args } => {
            songbird_orchestrator::run_server(args).await?;
        }
        Commands::Doctor { args } => {
            songbird_orchestrator::run_doctor(args).await?;
        }
        Commands::Config { config_cmd } => {
            songbird_orchestrator::run_config(config_cmd).await?;
        }
        Commands::Cli { args } => {
            songbird_cli::run(args).await?;
        }
        Commands::ComputeBridge { args } => {
            songbird_compute_bridge::run(args).await?;
        }
        Commands::Deploy { deploy_cmd } => {
            songbird_remote_deploy::run(deploy_cmd).await?;
        }
        Commands::Rendezvous { args } => {
            rendezvous::run(args).await?;
        }
    }
    
    Ok(())
}
```

---

## 🔨 IMPLEMENTATION STEPS

### **Phase 1: Prepare Crates** (Convert main.rs → lib.rs)

**For each crate**:
1. Rename `src/main.rs` → `src/lib.rs`
2. Extract CLI args into public struct
3. Extract main logic into public `run()` function
4. Make necessary types/functions public
5. Update `Cargo.toml` to remove `[[bin]]` entry

**Example Pattern**:
```rust
// OLD: src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    // ... logic ...
}

// NEW: src/lib.rs
pub use clap::Parser; // Re-export for main.rs

#[derive(Parser, Debug)]
pub struct Args {
    // ... fields ...
}

pub async fn run(args: Args) -> Result<()> {
    // ... logic ...
}
```

### **Phase 2: Create Unified Entry Point**

1. Create `src/main.rs` at workspace root
2. Add `clap` dependency to workspace root `Cargo.toml`
3. Implement routing logic
4. Add proper error handling

### **Phase 3: Update Workspace Cargo.toml**

```toml
[package]
name = "songbird"
version = "3.33.0"  # Bump for UniBin compliance
edition = "2021"
license = "AGPL-3.0"
authors = ["ecoPrimals <contact@ecoprimals.dev>"]

[[bin]]
name = "songbird"
path = "src/main.rs"

[dependencies]
songbird-orchestrator = { path = "crates/songbird-orchestrator" }
songbird-cli = { path = "crates/songbird-cli" }
songbird-compute-bridge = { path = "crates/songbird-compute-bridge" }
songbird-remote-deploy = { path = "crates/songbird-remote-deploy" }
rendezvous = { path = "rendezvous" }

anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.46", features = ["full"] }
```

### **Phase 4: Test & Validate**

1. Build unified binary: `cargo build --release`
2. Test each subcommand:
   ```bash
   ./target/release/songbird server --help
   ./target/release/songbird doctor
   ./target/release/songbird config validate
   ./target/release/songbird cli --help
   ./target/release/songbird compute-bridge --help
   ./target/release/songbird deploy list --help
   ./target/release/songbird rendezvous --help
   ./target/release/songbird --version
   ```
3. Verify all tests still pass
4. Check binary size (should be ~30-40MB)

### **Phase 5: Update Documentation**

1. Update README with new commands
2. Create migration guide for users
3. Update deployment scripts
4. Update wateringHole status

---

## 🎯 MODERN IDIOMATIC RUST PATTERNS

### **1. Public API Design**

```rust
// GOOD: Clean public API
pub struct ServerArgs { /* ... */ }
pub async fn run_server(args: ServerArgs) -> Result<()> { /* ... */ }

// BAD: Exposing internals
pub fn main() { /* ... */ }
```

### **2. Error Handling**

```rust
// GOOD: Proper error propagation
pub async fn run(args: Args) -> Result<()> {
    let config = Config::from_env()
        .context("Failed to load configuration")?;
    Ok(())
}

// BAD: unwrap/expect in library code
pub async fn run(args: Args) {
    let config = Config::from_env().unwrap();
}
```

### **3. RAII Resource Management**

```rust
// GOOD: Automatic cleanup
pub async fn run(args: Args) -> Result<()> {
    let _guard = ProcessManager::acquire_lock()?;
    // ... work ...
    Ok(())
    // _guard drops here, releasing lock
}
```

### **4. Async/Await**

```rust
// GOOD: Modern async
pub async fn run(args: Args) -> Result<()> {
    let client = Client::new().await?;
    client.connect().await?;
    Ok(())
}

// BAD: Blocking in async
pub async fn run(args: Args) -> Result<()> {
    std::thread::sleep(Duration::from_secs(1)); // ❌ Blocks executor
}
```

---

## 📋 CHECKLIST

### **Crate Refactoring**

- [ ] songbird-orchestrator: main.rs → lib.rs + public API
- [ ] songbird-cli: main.rs → lib.rs + public API
- [ ] songbird-compute-bridge: main.rs → lib.rs + public API
- [ ] songbird-remote-deploy: main.rs → lib.rs + public API
- [ ] rendezvous: main.rs → lib.rs + public API

### **Unified Entry Point**

- [ ] Create src/main.rs with routing
- [ ] Add proper error handling
- [ ] Add comprehensive --help output
- [ ] Add version information

### **Cargo.toml Updates**

- [ ] Add workspace [package] section
- [ ] Add [[bin]] entry for songbird
- [ ] Add crate dependencies
- [ ] Remove old [[bin]] entries from crates

### **Testing**

- [ ] All tests pass: `cargo test`
- [ ] Binary builds: `cargo build --release`
- [ ] Each subcommand works: `songbird <mode> --help`
- [ ] Version check: `songbird --version`
- [ ] Size check: Binary < 50MB

### **Documentation**

- [ ] Update README with UniBin commands
- [ ] Create MIGRATION_GUIDE.md for users
- [ ] Update deployment scripts
- [ ] Update wateringHole/SONGBIRD_STATUS

---

## 🎊 SUCCESS CRITERIA

- ✅ Single `songbird` binary in target/release/
- ✅ All 5 modes accessible via subcommands
- ✅ Professional --help output
- ✅ All tests passing
- ✅ Zero breaking changes (same functionality)
- ✅ Modern idiomatic Rust (no unwrap in libs)
- ✅ UniBin compliant per ecosystem standard

---

## 📊 ESTIMATED EFFORT

| Phase | Task | Time |
|-------|------|------|
| 1 | Refactor orchestrator | 1h |
| 1 | Refactor cli | 30m |
| 1 | Refactor compute-bridge | 30m |
| 1 | Refactor remote-deploy | 30m |
| 1 | Refactor rendezvous | 30m |
| 2 | Create unified main.rs | 1h |
| 3 | Update Cargo.toml | 30m |
| 4 | Test & validate | 1h |
| 5 | Documentation | 30m |
| **TOTAL** | | **6h 30m** |

---

## 🚀 NEXT STEPS

1. Start with Phase 1: Refactor songbird-orchestrator
2. Iterate through each crate
3. Create unified entry point
4. Test thoroughly
5. Update documentation
6. Celebrate UniBin compliance! 🎉

---

🦀✨ **Let's build a world-class UniBin!** ✨🦀

