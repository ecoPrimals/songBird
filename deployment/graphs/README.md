# 🎯 neuralAPI Deployment Graphs

**Declarative, Graph-Based Orchestration**

This directory contains TOML-based deployment graphs for biomeOS neuralAPI graph orchestration. These graphs provide atomic, reversible, auditable deployments.

---

## 📋 Overview

neuralAPI graphs replace imperative shell scripts with declarative, graph-based orchestration:

- ✅ **Declarative**: Describe *what* to deploy, not *how*
- ✅ **Atomic**: All-or-nothing deployment
- ✅ **Reversible**: Automatic rollback on failure
- ✅ **Auditable**: Full execution trace
- ✅ **Parallel**: Independent nodes run concurrently
- ✅ **Type-Safe**: Validated inputs/outputs

---

## 🚀 Quick Start

### Deploy Single Songbird Instance

```bash
biomeos deploy --graph deployment/graphs/songbird_deploy.toml
```

### Deploy TOWER (BearDog + Songbird)

```bash
biomeos deploy --graph deployment/graphs/tower_genome.toml
```

### Cross-Platform Deployment (USB + Android)

```bash
biomeos deploy --graph deployment/graphs/cross_platform_deploy.toml \
  --usb-device /dev/sdb1 \
  --android-device adb
```

---

## 📊 Available Graphs

### 1. `songbird_deploy.toml` - Single Instance Deployment

**Purpose**: Deploy a single Songbird instance with full health checks and rollback.

**Phases**:
1. Platform detection (arch + OS)
2. genomeBin integrity verification
3. Deployment & extraction
4. Health checks (version, socket, discovery)
5. Capability registration
6. Success notification

**Rollback Triggers**:
- Deployment failure
- Health check failure
- Capability registration failure

**Duration**: ~30 seconds

**Example**:
```bash
biomeos deploy --graph songbird_deploy.toml \
  --install-dir /opt/biomeos \
  --family-id production-01
```

---

### 2. `tower_genome.toml` - TOWER Foundation Deployment

**Purpose**: Deploy the complete TOWER (BearDog + Songbird) with cross-primal capability wiring.

**Phases**:
1. **Phase 1**: Deploy BearDog (security foundation)
   - Platform detection
   - BearDog deployment
   - Health check
2. **Phase 2**: Deploy Songbird (discovery foundation)
   - Songbird deployment (after BearDog healthy)
   - Health check
3. **Phase 3**: TOWER validation (atomic)
   - Both primals must be healthy
4. **Phase 4**: Cross-primal capability wiring
   - Songbird → BearDog: crypto.sign, crypto.verify, auth.jwt_provision
   - BearDog → Songbird: discovery.find_primal
5. **Phase 5**: TOWER registration
   - Register as operational in neuralAPI

**Rollback Triggers**:
- BearDog deployment/health failure → rollback BearDog
- Songbird deployment/health failure → rollback Songbird
- Wiring failure → rollback entire TOWER

**Duration**: ~60 seconds

**Example**:
```bash
biomeos deploy --graph tower_genome.toml --family-id tower-prod
```

**Capabilities Provided**:
- `security.crypto` (BearDog)
- `security.auth` (BearDog)
- `discovery.mdns` (Songbird)
- `discovery.stun` (Songbird)
- `discovery.dark_forest` (Songbird)
- `federation.peer_management` (Songbird)

---

### 3. `cross_platform_deploy.toml` - Parallel Multi-Platform Deployment

**Purpose**: Deploy Songbird to USB + Android simultaneously, establish federation.

**Phases**:
1. **Phase 1**: Parallel deployment
   - USB Live Spore (musl binary)
   - Android device via ADB (aarch64 binary)
2. **Phase 2**: Parallel health checks
   - USB instance validation
   - Android instance validation
3. **Phase 3**: Cross-platform federation
   - mDNS/STUN peer discovery
   - Dark Forest handshake
4. **Phase 4**: Federation validation
   - USB → Android RPC ping
   - Android → USB RPC ping

**Rollback Triggers**:
- USB deployment/health failure → rollback USB
- Android deployment/health failure → rollback Android
- Handshake failure → teardown federation, rollback both

**Duration**: ~90 seconds

**Example**:
```bash
# USB drive at /dev/sdb1, Android via ADB
biomeos deploy --graph cross_platform_deploy.toml \
  --usb-device /dev/sdb1 \
  --android-device adb
```

**Result**: Two Songbird instances federated via Dark Forest protocol!

---

## 🎯 Graph Anatomy

### Graph Metadata

```toml
[graph]
id = "unique_graph_id"
version = "1.0"
description = "What this graph does"
author = "Team Name"
created = "2026-01-31"
```

### Node Definition

```toml
[[nodes]]
id = "node_unique_id"
type = "node.type"  # genome.deploy, health.check_primal, etc.
description = "What this node does"
depends_on = ["prerequisite_node_id"]  # Optional

[nodes.config]
# Node-specific configuration
key = "value"

[nodes.outputs]
# Expected outputs (for downstream nodes)
output_name = "type"
```

### Edge Definition (Error Handling)

```toml
[[edges]]
from = "source_node"
to = "error_handler_node"
condition = "on_error"
```

### Execution Settings

```toml
[execution]
mode = "sequential"  # or "parallel_where_possible"
timeout_seconds = 120
retry_on_failure = false  # Prefer rollback
parallel_where_possible = true

[observability]
log_level = "info"
trace_execution = true
collect_metrics = true
```

---

## 🔧 Node Types Reference

### Deployment Nodes

**`genome.deploy`**: Deploy genomeBin
```toml
[nodes.config]
genome = "songbird.genome"
target = "auto"  # auto, usb, android, linux, darwin
install_dir = "/opt/biomeos"
family_id = "default"
deploy_mode = "systemd"  # systemd, usb, android, manual
```

**`genome.rollback`**: Rollback genomeBin deployment
```toml
[nodes.config]
primal = "songbird"
family_id = "default"  # Optional
actions = ["stop_service", "remove_binaries", "clean_runtime_dirs"]
```

### Health Check Nodes

**`health.check_primal`**: Check single primal health
```toml
[nodes.config]
primal = "songbird"
timeout_ms = 5000
checks = ["version", "socket", "discovery"]
```

**`health.check_atomic`**: Atomic multi-primal health check
```toml
[nodes.config]
primals = ["beardog", "songbird"]
require_all = true
timeout_ms = 10000
```

### Capability Nodes

**`capability.register`**: Register primal capabilities
```toml
[nodes.config]
primal = "songbird"
capabilities = ["discovery.mdns", "discovery.stun"]
```

**`capability.wire_tower`**: Wire cross-primal capabilities
```toml
[nodes.config]
tower_primals = ["beardog", "songbird"]
wiring = [
    {
        from = "songbird",
        to = "beardog",
        capability = "crypto.sign",
        protocol = "ipc"
    },
]
```

### Federation Nodes

**`federation.handshake`**: Establish peer handshake
```toml
[nodes.config]
nodes = [
    { family_id = "node1", endpoints = ["..."] },
    { family_id = "node2", endpoints = ["..."] },
]
protocol = "dark_forest"
trust_level = "limited"  # full, limited, federated
```

**`federation.teardown`**: Teardown federation
```toml
[nodes.config]
family_ids = ["node1", "node2"]
```

### Utility Nodes

**`platform.detect`**: Detect platform & architecture
```toml
[nodes.config]
detect_arch = true
detect_os = true
detect_android = true
```

**`file.verify`**: Verify file integrity
```toml
[nodes.config]
file_path = "songbird.genome"
verify_checksum = true
verify_executable = true
```

**`notification.send`**: Send notification
```toml
[nodes.config]
message = "Deployment successful!"
level = "info"  # info, warn, error
channels = ["log", "stdout"]
```

---

## 🎓 Deep Debt Solution

### Before: Imperative Shell Scripts

```bash
# deploy.sh - Manual, error-prone
set -e
deploy_beardog || { echo "Failed"; exit 1; }
check_beardog || { echo "Failed"; rollback_beardog; exit 1; }
deploy_songbird || { echo "Failed"; rollback_beardog; exit 1; }
check_songbird || { echo "Failed"; rollback_all; exit 1; }
wire_capabilities || { echo "Failed"; rollback_all; exit 1; }
echo "Success"
```

**Problems**:
- ❌ Imperative: Must specify *how* to deploy
- ❌ Manual error handling
- ❌ No parallelization
- ❌ No type safety
- ❌ Hard to audit

### After: Declarative neuralAPI Graphs

```toml
# tower_genome.toml - Declarative, type-safe
[[nodes]]
id = "deploy_beardog"
type = "genome.deploy"
config = { genome = "beardog.genome" }

[[nodes]]
id = "deploy_songbird"
type = "genome.deploy"
depends_on = ["deploy_beardog"]
config = { genome = "songbird.genome" }

# Automatic error handling via edges!
[[edges]]
from = "deploy_songbird"
to = "rollback_songbird"
condition = "on_error"
```

**Benefits**:
- ✅ Declarative: Describe *what* to deploy
- ✅ Automatic error handling
- ✅ Automatic parallelization
- ✅ Type-safe inputs/outputs
- ✅ Full execution trace
- ✅ Atomic rollback

---

## 🧪 Testing Graphs

### Dry Run (Validation Only)

```bash
# Validate graph syntax and dependencies
biomeos deploy --graph songbird_deploy.toml --dry-run
```

### Execution Trace

```bash
# Run with full tracing
biomeos deploy --graph tower_genome.toml --trace

# Output: Execution trace with timing, dependencies, outputs
```

### Rollback Testing

```bash
# Simulate failure at specific node
biomeos deploy --graph songbird_deploy.toml \
  --simulate-failure health_check
  
# Should trigger rollback
```

---

## 📚 Creating Custom Graphs

### Step 1: Define Graph Metadata

```toml
[graph]
id = "my_custom_graph"
version = "1.0"
description = "My custom deployment"
author = "Your Name"
created = "2026-01-31"
```

### Step 2: Add Deployment Nodes

```toml
[[nodes]]
id = "deploy_primal"
type = "genome.deploy"
config = { genome = "primal.genome", target = "auto" }
```

### Step 3: Add Health Checks

```toml
[[nodes]]
id = "check_health"
type = "health.check_primal"
depends_on = ["deploy_primal"]
config = { primal = "primal", checks = ["version", "socket"] }
```

### Step 4: Add Error Handling

```toml
[[edges]]
from = "deploy_primal"
to = "rollback_primal"
condition = "on_error"

[[nodes]]
id = "rollback_primal"
type = "genome.rollback"
config = { primal = "primal" }
```

### Step 5: Configure Execution

```toml
[execution]
mode = "sequential"
timeout_seconds = 120
parallel_where_possible = true

[observability]
log_level = "info"
trace_execution = true
```

---

## 🎯 Best Practices

### 1. Dependency Ordering
- Deploy security foundation first (BearDog)
- Deploy discovery second (Songbird)
- Wire capabilities after both healthy

### 2. Health Checks
- Always health check after deployment
- Use atomic checks for multi-primal
- Set realistic timeouts (5-10 seconds)

### 3. Error Handling
- Add rollback edge for each critical node
- Rollback in reverse dependency order
- Teardown federation before rolling back nodes

### 4. Parallelization
- Mark independent deployments as parallel
- Use `parallel_groups` for hints
- Be conservative with concurrency

### 5. Observability
- Enable tracing for debugging
- Collect metrics for performance analysis
- Use appropriate log levels

---

## 🚧 Limitations & Future Work

### Current Limitations

1. **Requires biomeOS neuralAPI**: Graphs can't execute standalone
2. **No graph composition**: Can't include graphs in graphs (yet)
3. **Limited node types**: More node types coming in Phase 2
4. **No conditional branching**: Only error-based branching

### Future Enhancements (Phase 2)

1. **Graph composition**: Import and reuse sub-graphs
2. **Conditional nodes**: `if`/`else` branching based on outputs
3. **Loop nodes**: Deploy N instances dynamically
4. **Remote execution**: Deploy to remote machines
5. **Graph templates**: Parameterized graphs
6. **Visual graph editor**: GUI for graph creation

---

## 📊 Success Metrics

neuralAPI graphs are successful when:
- ✅ 3 reference graphs created (single, TOWER, cross-platform)
- ✅ Declarative deployment patterns documented
- ✅ Error handling with rollback works
- ✅ Integration with biomeOS neuralAPI (when available)
- ✅ Atomic TOWER deployment works
- ✅ Cross-platform parallel deployment works
- ✅ Full execution traces available
- ✅ Graphs are type-safe and validated

---

## 📚 Related Documentation

- `../genome/README.md` - genomeBin self-extracting wrapper
- `../systemd/README.md` - systemd service integration
- `../../GENOMEBIN_WEEK3_EXECUTION_PLAN_JAN_31_2026.md` - Week 3 plan
- biomeOS neuralAPI documentation (external)

---

**Created**: January 31, 2026 (Evening)  
**Status**: ✅ 3 reference graphs complete, ready for biomeOS integration  
**Next**: Async/concurrent evolution, platform optimizations
