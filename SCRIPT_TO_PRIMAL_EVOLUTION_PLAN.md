# Script → Primal Evolution Plan

**Date**: January 29, 2026  
**Version**: v8.19.0 → v8.20.0 (Future)  
**Goal**: 🦀 **100% Pure Rust Ecosystem** - Zero bash dependencies  
**Vision**: biomeOS orchestrates ALL primals, including validation tools

---

## Executive Summary

**Current State**: 3 bash scripts for validation/testing  
**Target State**: 3 Rust validation primals orchestrated by biomeOS  
**Benefit**: TRUE ecoBin compliance + biomeOS-native testing

---

## Why Evolve Scripts to Primals?

### Current State (Bash Scripts)

```
❌ Shell script dependencies (bash, nc, jq)
❌ Manual execution (not orchestrated)
❌ Not discoverable by Neural API
❌ Can't leverage primal ecosystem
❌ Violates TRUE ecoBin principle
```

### Target State (Rust Primals)

```
✅ 100% Pure Rust (zero shell dependencies)
✅ Orchestrated by biomeOS Neural API
✅ Discoverable via capability registration
✅ JSON-RPC based (standard protocol)
✅ TRUE ecoBin compliant
✅ Fellow primals in the ecosystem
```

---

## Scripts to Evolve

### 1. `test_method_wiring.sh` → **Validator Primal**

**Current**: Bash script testing JSON-RPC methods

**Evolution**: Rust primal that validates other primals

```toml
# graphs/validator.toml
[[primals]]
name = "validator"
binary = "./primals/validator/validator"
socket = "/run/user/1000/biomeos/validator-nat0.sock"
capabilities = ["validate", "test", "health_check"]
startup_order = 10  # Starts after other primals

[[validation.targets]]
primal = "songbird"
socket = "/primal/songbird"
methods = [
    "stun.get_public_address",
    "stun.bind",
    "discovery.peers",
    "rendezvous.register",
    "rendezvous.lookup",
    "peer.connect"
]
```

**New Rust Crate**: `crates/validator/`

**Capabilities**:
```rust
// Validator primal - validates other primals
pub struct ValidatorPrimal {
    targets: Vec<ValidationTarget>,
    results: Arc<RwLock<ValidationResults>>,
}

// JSON-RPC API
impl ValidatorPrimal {
    /// Validate all configured targets
    async fn validate_all(&self) -> Result<ValidationReport>;
    
    /// Validate specific primal
    async fn validate_primal(&self, primal: &str) -> Result<PrimalReport>;
    
    /// Health check all primals
    async fn health_check(&self) -> Result<HealthReport>;
    
    /// Get validation history
    async fn get_history(&self) -> Result<Vec<ValidationResult>>;
}
```

**biomeOS Integration**:
```bash
# Deploy validator primal
./neural-api deploy graphs/validator.toml

# Run validation
echo '{"jsonrpc":"2.0","method":"validate.all","params":{},"id":1}' | \
    nc -U /primal/validator
```

---

### 2. `test_two_spore_handshake.sh` → **Handshake Orchestrator Primal**

**Current**: Bash script testing STUN handshake between spores

**Evolution**: Rust primal that orchestrates cross-spore testing

```toml
# graphs/handshake_orchestrator.toml
[[primals]]
name = "handshake-orchestrator"
binary = "./primals/handshake-orchestrator/handshake-orchestrator"
socket = "/run/user/1000/biomeos/handshake-orchestrator-nat0.sock"
capabilities = ["orchestrate", "stun_test", "discovery_test"]
startup_order = 11

[[orchestrator.peers]]
node_id = "node-alpha"
socket = "/primal/songbird"
role = "initiator"

[[orchestrator.peers]]
node_id = "node-gamma"
socket = "/primal/songbird"  # Remote socket (cross-spore)
role = "responder"
```

**New Rust Crate**: `crates/handshake-orchestrator/`

**Capabilities**:
```rust
// Handshake Orchestrator - coordinates cross-spore testing
pub struct HandshakeOrchestrator {
    peers: Vec<PeerTarget>,
    stun_servers: Vec<String>,
    results: Arc<RwLock<HandshakeResults>>,
}

impl HandshakeOrchestrator {
    /// Initiate two-spore handshake test
    async fn test_handshake(&self, peer_a: &str, peer_b: &str) -> Result<HandshakeReport>;
    
    /// Test STUN discovery
    async fn test_stun(&self, peer: &str, server: &str) -> Result<StunReport>;
    
    /// Test UDP beacon discovery
    async fn test_discovery(&self, peers: Vec<String>) -> Result<DiscoveryReport>;
    
    /// Continuous monitoring
    async fn monitor(&self, interval_secs: u64) -> Result<()>;
}
```

**biomeOS Integration**:
```bash
# Deploy orchestrator on both spores
./neural-api deploy graphs/handshake_orchestrator.toml

# Run handshake test
echo '{"jsonrpc":"2.0","method":"handshake.test","params":{"peer_a":"node-alpha","peer_b":"node-gamma"},"id":1}' | \
    nc -U /primal/handshake-orchestrator
```

---

### 3. `test_local_stun_handshake.sh` → **Local Test Orchestrator Primal**

**Current**: Bash script for local multi-instance testing

**Evolution**: Rust primal that spawns and coordinates local test instances

```toml
# graphs/local_test_orchestrator.toml
[[primals]]
name = "local-test-orchestrator"
binary = "./primals/local-test-orchestrator/local-test-orchestrator"
socket = "/run/user/1000/biomeos/local-test-nat0.sock"
capabilities = ["local_test", "multi_instance", "chaos_test"]
startup_order = 12

[test.instances]
count = 2
base_port = 9000
family_id = "test-family"
```

**New Rust Crate**: `crates/local-test-orchestrator/`

**Capabilities**:
```rust
// Local Test Orchestrator - spawns and tests multiple instances
pub struct LocalTestOrchestrator {
    instances: Vec<TestInstance>,
    config: TestConfig,
}

impl LocalTestOrchestrator {
    /// Spawn N test instances
    async fn spawn_instances(&self, count: usize) -> Result<Vec<InstanceHandle>>;
    
    /// Run chaos test (random failures, network issues)
    async fn chaos_test(&self, duration_secs: u64) -> Result<ChaosReport>;
    
    /// Load test (stress testing)
    async fn load_test(&self, requests_per_sec: usize) -> Result<LoadReport>;
    
    /// Integration test suite
    async fn integration_test(&self) -> Result<IntegrationReport>;
}
```

---

## New Primal Crates Structure

```
crates/
├── validator/                          # NEW: Validation primal
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                    # Validator entry point
│   │   ├── validator.rs               # Core validation logic
│   │   ├── reporters.rs               # Report generation
│   │   └── targets.rs                 # Validation targets
│
├── handshake-orchestrator/            # NEW: Handshake orchestrator
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── orchestrator.rs            # Orchestration logic
│   │   ├── stun_tester.rs             # STUN test coordination
│   │   └── discovery_tester.rs        # Discovery test coordination
│
└── local-test-orchestrator/           # NEW: Local test orchestrator
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs
    │   ├── orchestrator.rs
    │   ├── instance_manager.rs        # Spawn/manage instances
    │   ├── chaos_engine.rs            # Chaos testing
    │   └── load_tester.rs             # Load testing
```

---

## Evolution Phases

### Phase 1: Validator Primal (v8.20.0)

**Goal**: Replace `test_method_wiring.sh` with Rust validator

**Scope**:
1. Create `crates/validator/` primal
2. Implement JSON-RPC method validation
3. Add health checking capabilities
4. Register with Neural API
5. Deployment graph

**Deliverables**:
- ✅ `validator` binary
- ✅ `graphs/validator.toml`
- ✅ Documentation
- ✅ Integration tests
- ❌ Remove `test_method_wiring.sh`

**Effort**: ~500 lines Rust, 2-3 hours

---

### Phase 2: Handshake Orchestrator Primal (v8.21.0)

**Goal**: Replace `test_two_spore_handshake.sh` with Rust orchestrator

**Scope**:
1. Create `crates/handshake-orchestrator/` primal
2. Implement cross-spore coordination
3. STUN testing capabilities
4. UDP discovery testing
5. Continuous monitoring mode

**Deliverables**:
- ✅ `handshake-orchestrator` binary
- ✅ `graphs/handshake_orchestrator.toml`
- ✅ Remote peer communication
- ✅ Documentation
- ❌ Remove `test_two_spore_handshake.sh`

**Effort**: ~800 lines Rust, 3-4 hours

---

### Phase 3: Local Test Orchestrator Primal (v8.22.0)

**Goal**: Replace `test_local_stun_handshake.sh` with Rust orchestrator

**Scope**:
1. Create `crates/local-test-orchestrator/` primal
2. Instance spawning/management
3. Chaos testing engine
4. Load testing capabilities
5. Full integration test suite

**Deliverables**:
- ✅ `local-test-orchestrator` binary
- ✅ `graphs/local_test_orchestrator.toml`
- ✅ Chaos engine
- ✅ Documentation
- ❌ Remove `test_local_stun_handshake.sh`

**Effort**: ~1,000 lines Rust, 4-5 hours

---

### Phase 4: biomeOS Integration (v8.23.0)

**Goal**: Full Neural API orchestration of validation primals

**Scope**:
1. Neural API awareness of validation primals
2. Automatic validation on deployment
3. Health monitoring integration
4. Chaos testing CI/CD integration
5. Validation dashboard

**Deliverables**:
- ✅ Neural API integration
- ✅ Auto-validation hooks
- ✅ Monitoring dashboard
- ✅ CI/CD integration
- ✅ Complete documentation

**Effort**: ~600 lines Rust + biomeOS integration, 3-4 hours

---

## Benefits of Primal Evolution

### 1. TRUE ecoBin Compliance ✅

```
Before: Bash scripts (external dependencies)
After:  Pure Rust primals (zero external deps)

Dependencies Eliminated:
  ❌ bash
  ❌ nc (netcat)
  ❌ jq (JSON parser)
  ❌ grep, sed, awk
  ❌ cat, echo, printf
```

### 2. biomeOS Native Orchestration ✅

```
Before: Manual script execution
  $ ./test_method_wiring.sh
  $ ./test_two_spore_handshake.sh

After: Neural API orchestration
  $ ./neural-api validate songbird
  $ ./neural-api test handshake node-alpha node-gamma
  $ ./neural-api chaos-test --duration 1h
```

### 3. Capability Discovery ✅

```
Before: Scripts don't register capabilities
After:  Primals discoverable via Neural API

$ ./neural-api capabilities list
  • validator: validate, test, health_check
  • handshake-orchestrator: stun_test, discovery_test
  • local-test-orchestrator: chaos_test, load_test
```

### 4. JSON-RPC Integration ✅

```
Before: Bash scripts with custom output
After:  Standard JSON-RPC protocol

All validation results:
  • Machine readable (JSON)
  • Standardized format
  • Parseable by other primals
  • Dashboard integration ready
```

### 5. Advanced Testing Features ✅

```
New Capabilities:
  ✅ Chaos testing (random failures)
  ✅ Load testing (stress testing)
  ✅ Continuous monitoring
  ✅ Automated regression testing
  ✅ CI/CD integration
  ✅ Health monitoring
```

---

## Implementation Strategy

### Quick Wins (Phase 1)

Start with **Validator Primal** - simplest to implement:

1. **Week 1**: Create `crates/validator/` crate
2. **Week 1**: Implement method validation logic
3. **Week 1**: Add health checking
4. **Week 2**: Neural API integration
5. **Week 2**: Documentation + tests
6. **Release**: v8.20.0 with Validator primal

### Medium Term (Phase 2-3)

Continue with orchestrators:

1. **Week 3-4**: Handshake Orchestrator primal
2. **Week 5-6**: Local Test Orchestrator primal
3. **Release**: v8.22.0 with all orchestrators

### Long Term (Phase 4)

Full biomeOS integration:

1. **Week 7-8**: Neural API deep integration
2. **Week 9**: CI/CD automation
3. **Week 10**: Dashboard + monitoring
4. **Release**: v8.23.0 - Complete ecosystem

---

## Migration Path

### Parallel Operation Period

Keep bash scripts during transition:

```
Phase 1-2: Both bash scripts AND Rust primals exist
  • Bash scripts deprecated but functional
  • Rust primals validated in production
  • Users can choose either

Phase 3: Bash scripts marked for removal
  • Warning when using bash scripts
  • Documentation updated to use primals
  • 30-day deprecation notice

Phase 4: Bash scripts removed
  • Only Rust primals remain
  • TRUE ecoBin achieved
  • 100% Pure Rust ecosystem
```

### Compatibility

Ensure smooth transition:

1. **Same API** - Rust primals accept same inputs
2. **Same Output** - JSON format matches bash output
3. **Same Behavior** - Test logic unchanged
4. **Better Features** - Additional capabilities

---

## Example: Validator Primal Sketch

```rust
// crates/validator/src/main.rs
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Register with Neural API
    register_capabilities(&[
        "validate",
        "test",
        "health_check",
    ]).await?;
    
    // Start validator primal
    let validator = ValidatorPrimal::new(config)?;
    let socket = "/run/user/1000/biomeos/validator-nat0.sock";
    
    // Serve JSON-RPC
    let listener = UnixListener::bind(socket)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let validator = validator.clone();
        
        tokio::spawn(async move {
            handle_connection(stream, validator).await
        });
    }
}

// crates/validator/src/validator.rs
pub struct ValidatorPrimal {
    targets: Vec<ValidationTarget>,
}

impl JsonRpcHandler for ValidatorPrimal {
    async fn handle(&self, method: &str, params: Value) -> Result<Value> {
        match method {
            "validate.all" => self.validate_all().await,
            "validate.primal" => self.validate_primal(params).await,
            "health.check" => self.health_check().await,
            "history.get" => self.get_history().await,
            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

impl ValidatorPrimal {
    async fn validate_all(&self) -> Result<Value> {
        let mut results = Vec::new();
        
        for target in &self.targets {
            let report = self.validate_target(target).await?;
            results.push(report);
        }
        
        Ok(json!({
            "timestamp": Utc::now(),
            "total": results.len(),
            "passed": results.iter().filter(|r| r.passed).count(),
            "failed": results.iter().filter(|r| !r.passed).count(),
            "results": results,
        }))
    }
}
```

---

## Success Metrics

### Code Quality

- ✅ 100% Pure Rust (zero bash)
- ✅ 90%+ test coverage
- ✅ Zero unsafe code
- ✅ Clippy pedantic compliant
- ✅ Full documentation

### Functionality

- ✅ All bash script features preserved
- ✅ New features added (chaos, load, monitoring)
- ✅ Performance improved (Rust vs bash)
- ✅ Better error handling

### Integration

- ✅ Neural API orchestration
- ✅ Capability discovery
- ✅ JSON-RPC standardized
- ✅ Dashboard integration ready

### Ecosystem

- ✅ TRUE ecoBin compliant
- ✅ Fellow primals (not tools)
- ✅ Runtime discovery
- ✅ Orchestrated deployment

---

## Timeline

| Phase | Duration | Deliverable | Version |
|-------|----------|-------------|---------|
| Phase 1 | 2 weeks | Validator Primal | v8.20.0 |
| Phase 2 | 2 weeks | Handshake Orchestrator | v8.21.0 |
| Phase 3 | 2 weeks | Local Test Orchestrator | v8.22.0 |
| Phase 4 | 2 weeks | Full biomeOS Integration | v8.23.0 |
| **Total** | **8 weeks** | **Complete Ecosystem** | **v8.23.0** |

---

## Conclusion

Evolving bash scripts to Rust primals achieves:

1. 🦀 **TRUE ecoBin** - 100% Pure Rust, zero external dependencies
2. 🌐 **biomeOS Native** - Orchestrated by Neural API as fellow primals
3. 🎯 **Capability Based** - Discoverable, composable, testable
4. 🚀 **Advanced Features** - Chaos testing, load testing, monitoring
5. 🏆 **Best Practices** - JSON-RPC, runtime discovery, clean architecture

**Result**: A **fully Rust ecosystem** where biomeOS orchestrates **all** components, including testing and validation, as **fellow primals**!

---

**Status**: 📋 **EVOLUTION PLAN READY**

Next: Implement Phase 1 - Validator Primal (v8.20.0)

---

**Generated**: January 29, 2026  
**Current**: v8.19.0  
**Target**: v8.23.0 (Complete Pure Rust Ecosystem)  
**Timeline**: 8 weeks to full evolution

