# 🎵 Phase 1: Isolated Songbird Instance

**Goal**: Demonstrate single Songbird capabilities  
**Prerequisites**: None (runs on single machine)  
**Time**: 15-30 minutes  
**Complexity**: Beginner-Intermediate

---

## 🎯 What You'll Learn

- Starting and configuring Songbird
- Capability-based service discovery
- REST API usage
- Health monitoring
- Metrics and observability
- Service lifecycle management

---

## 📁 Demos

### 1. Hello Songbird (2 minutes)
**File**: `demos/01-hello-songbird.sh`

The simplest demo - start Songbird and verify it's healthy.

```bash
./demos/01-hello-songbird.sh
```

**What it shows**:
- Songbird startup
- Health check endpoint
- Basic configuration
- Log output

---

### 2. Capability Discovery (5 minutes)
**File**: `demos/02-capability-discovery.sh`

Demonstrates Songbird's capability-based architecture.

```bash
./demos/02-capability-discovery.sh
```

**What it shows**:
- Registering services with capabilities
- Discovering services by capability (not by name!)
- Capability-based routing
- Dynamic service updates

**Key Insight**: Services advertise WHAT THEY CAN DO, not WHO THEY ARE

---

### 3. API Playground (10 minutes)
**File**: `demos/03-api-playground.sh`

Explore Songbird's REST API interactively.

```bash
./demos/03-api-playground.sh
```

**Endpoints Demonstrated**:
- `GET /health` - Health status
- `GET /api/v1/discovery` - Service discovery
- `POST /api/v1/capabilities/register` - Register service
- `GET /api/v1/capabilities/query` - Query capabilities
- `GET /api/v1/metrics` - Metrics endpoint
- `POST /api/v1/compute/task` - Submit compute task

---

### 4. Metrics Dashboard (5 minutes)
**File**: `demos/04-metrics-dashboard.sh`

Live metrics and observability.

```bash
./demos/04-metrics-dashboard.sh
```

**What it shows**:
- Real-time metrics collection
- Request counts and latencies
- Service health status
- Resource usage
- Simple ASCII dashboard

---

### 5. Service Lifecycle (5 minutes)
**File**: `demos/05-service-lifecycle.sh`

Service registration, updates, and removal.

```bash
./demos/05-service-lifecycle.sh
```

**What it shows**:
- Service registration
- Capability updates
- Health check evolution
- Graceful removal
- Discovery consistency

---

### 6. Routing Existing Services (7 minutes) ⭐
**File**: `demos/06-routing-existing-services.sh`

**Songbird as standalone router for YOUR existing services.**

```bash
./demos/06-routing-existing-services.sh
```

**What it shows**:
- Discover running services (Redis, PostgreSQL, MySQL, MongoDB, etc.)
- Route to existing systems WITHOUT changing them
- Automatic failover and load balancing
- Secrets management integration
- Real-world routing scenarios

**Key Insight**: Deploy Songbird in front of existing services for instant intelligent routing!

---

### 7. Docker Integration (7 minutes) 🐳
**File**: `demos/07-docker-integration.sh`

**Songbird routing to Docker containers.**

```bash
./demos/07-docker-integration.sh
```

**What it shows**:
- Docker container discovery
- Automatic capability detection
- Multi-container load balancing
- Blue-green deployments
- docker-compose integration

**Key Insight**: Container-aware routing without Kubernetes complexity!

---

### 8. Real Execution Verification (5 minutes) 🔍
**File**: `demos/08-real-verification.sh`

**PROVES these demos run real Songbird, not mock data.**

```bash
./demos/08-real-verification.sh
```

**10-Point Verification**:
- Real binary exists (14MB ELF executable)
- Real process created (PID captured)
- Real port listening (lsof verification)
- Real HTTP responses (curl with status codes)
- Real API endpoints working
- Real log output
- Real resource usage (memory, CPU)
- Real network connections
- Interactive verification commands
- Clean shutdown verification

**Result**: ✅ 100% Real Execution Verified!

---

## 🚀 Quick Start

### Run All Demos
```bash
./run-all-demos.sh
```

### Run Individual Demo
```bash
./demos/01-hello-songbird.sh
```

### With Custom Config
```bash
SONGBIRD_PORT=8080 ./demos/01-hello-songbird.sh
```

---

## 📊 Expected Output

### Hello Songbird
```
🎵 Songbird Starting...
✅ Songbird healthy on http://localhost:8000
📊 Status: {"status":"healthy","uptime_seconds":0}
```

### Capability Discovery
```
📝 Registering compute service...
✅ Service registered with capabilities: ["compute_light", "http"]

🔍 Discovering compute services...
✅ Found 1 service(s):
   - compute-service-1 (capabilities: compute_light, http)

🎯 Querying for "compute_light" capability...
✅ Matched service: compute-service-1
```

---

## 🛠️ Configuration

### Port Configuration
```bash
# Default
SONGBIRD_PORT=8000

# Custom
SONGBIRD_PORT=8080 ./demos/01-hello-songbird.sh
```

### Log Level
```bash
RUST_LOG=debug ./demos/01-hello-songbird.sh
```

### Config File
```bash
SONGBIRD_CONFIG=./configs/custom.toml ./demos/01-hello-songbird.sh
```

---

## 📁 Directory Structure

```
01-isolated/
├── README.md           # This file
├── run-all-demos.sh   # Run all demos sequentially
├── demos/             # Individual demo scripts
│   ├── 01-hello-songbird.sh
│   ├── 02-capability-discovery.sh
│   ├── 03-api-playground.sh
│   ├── 04-metrics-dashboard.sh
│   └── 05-service-lifecycle.sh
├── configs/           # Demo configurations
│   ├── default.toml
│   ├── compute-service.toml
│   └── storage-service.toml
└── scripts/          # Helper scripts
    ├── start-songbird.sh
    ├── stop-songbird.sh
    └── check-health.sh
```

---

## 🎓 Learning Objectives

After completing Phase 1, you should understand:

- ✅ How to start and configure Songbird
- ✅ Capability-based architecture (services discovered by WHAT they do)
- ✅ REST API endpoints and usage
- ✅ Health monitoring and metrics
- ✅ Service registration and discovery
- ✅ Basic troubleshooting

---

## 🔧 Troubleshooting

### Port Already in Use
```bash
# Check what's using the port
lsof -i :8000

# Use different port
SONGBIRD_PORT=8080 ./demos/01-hello-songbird.sh
```

### Songbird Won't Start
```bash
# Check logs
tail -f /tmp/songbird.log

# Verify binary
cargo build --release && ls -la target/release/songbird-orchestrator
```

### Discovery Not Working
```bash
# Check discovery endpoint
curl http://localhost:8000/api/v1/discovery

# Verify service registration
curl http://localhost:8000/api/v1/capabilities/list
```

---

## 💡 Key Concepts

### Capability-Based Discovery
**Traditional**: "Find me the AUTH service"  
**Songbird**: "Find me a service that can AUTHENTICATE"

**Why Better?**:
- Multiple providers can satisfy
- Load balancing automatic
- Failover built-in
- No hardcoded dependencies

### Zero Primal Coupling
Services never know WHO will fulfill their needs, only WHAT capabilities they need.

Example:
```bash
# Service asks for "storage" capability
# Songbird finds: squirrel, s3-adapter, local-storage
# Songbird picks best based on health, load, latency
```

---

## 📚 Next Steps

After mastering Phase 1:

1. **Proceed to Phase 2**: `../02-federation/README.md`
   - Multiple Songbirds forming a mesh
   - Cross-tower discovery
   - Load balancing

2. **Read Architecture Docs**: `../../docs/architecture/`
   - Understand design decisions
   - Learn advanced patterns

3. **Explore API**: `cargo doc --open`
   - Complete API reference
   - Implementation details

---

## 🎯 Success Criteria

Phase 1 is complete when you can:

- [ ] Start Songbird successfully
- [ ] Query health endpoint
- [ ] Register a service with capabilities
- [ ] Discover services by capability
- [ ] View metrics
- [ ] Understand capability-based vs name-based discovery

---

**Ready?** Start with `./run-all-demos.sh` or go demo-by-demo!

🎵 **Welcome to the world of capability-based orchestration!** 🎵

