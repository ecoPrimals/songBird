# 🎵 Songbird Showcase - Progressive Capability Demonstrations

**Purpose**: Demonstrate Songbird's world-class orchestration capabilities  
**Philosophy**: Progressive complexity - isolated → federated → inter-primal  
**Goal**: Enable easy LAN mesh joining for friends and distributed compute

---

## 🎯 Showcase Philosophy

This showcase demonstrates Songbird's evolution from our experiments and integration with Toadstool:

1. **Isolated Instance**: Single Songbird capabilities
2. **Federation**: Multiple Songbirds coordinating
3. **Inter-Primal**: Songbird + Toadstool distributed compute mesh

**Real-World Scenario**: *"A friend joins your LAN and can immediately participate in the mesh"*

---

## 📁 Structure

```
showcase/
├── 01-isolated/          # Single Songbird demos
│   ├── discovery/        # Capability discovery
│   ├── health/          # Health monitoring
│   ├── api/             # REST API usage
│   └── metrics/         # Observability
│
├── 02-federation/        # Multiple Songbirds
│   ├── mesh/            # Mesh formation
│   ├── cross-tower/     # Cross-tower discovery
│   ├── load-balance/    # Load balancing
│   └── failover/        # Resilience demos
│
├── 03-inter-primal/      # Songbird + Toadstool
│   ├── simple-compute/  # Basic compute tasks
│   ├── distributed-ml/  # ML training (from experiments)
│   ├── lan-join/        # Friend joining LAN
│   └── production/      # Complete mesh demo
│
└── utils/               # Shared utilities
    ├── setup/          # Environment setup
    ├── cleanup/        # Cleanup scripts
    └── monitoring/     # Monitoring tools
```

---

## 🚀 Quick Start

### Prerequisites
```bash
# Build Songbird
cargo build --release

# Optionally build Toadstool (for Phase 3)
cd ../toadstool && cargo build --release
```

### Run Demos

**Phase 1: Isolated Instance**
```bash
cd showcase/01-isolated
./run-all-demos.sh
```

**Phase 2: Federation (requires 2+ machines or multiple ports)**
```bash
cd showcase/02-federation
./setup-local-federation.sh
./run-federation-demos.sh
```

**Phase 3: Inter-Primal (Songbird + Toadstool)**
```bash
cd showcase/03-inter-primal
./setup-mesh.sh
./demo-distributed-compute.sh
```

---

## 🎓 Learning Path

### For New Users
1. Start with Phase 1 demos to understand Songbird basics
2. Progress to Phase 2 to see federation capabilities
3. Explore Phase 3 for complete inter-primal mesh

### For Operators
1. Review Phase 2 for multi-node setup
2. Study Phase 3 for production mesh deployment
3. Use `utils/monitoring` for observability

### For Developers
1. Study demo source code in each phase
2. Review `configs/` for configuration patterns
3. Extend demos with custom capabilities

---

## 📊 What You'll Learn

### Phase 1: Isolated Instance
- ✅ Capability-based discovery
- ✅ Health monitoring
- ✅ REST API usage
- ✅ Metrics and observability
- ✅ Service registration

### Phase 2: Federation
- ✅ Mesh formation (2+ Songbirds)
- ✅ Cross-tower service discovery
- ✅ Load balancing across instances
- ✅ Failover and resilience
- ✅ Zero-configuration networking

### Phase 3: Inter-Primal Mesh
- ✅ Songbird orchestrating Toadstool
- ✅ Distributed compute tasks
- ✅ ML training across towers
- ✅ Friend joining LAN mesh
- ✅ Production-ready deployment

---

## 🌟 Featured Demo: LAN Mesh Join

**Scenario**: A friend brings their laptop to your LAN and wants to contribute compute power

**What Happens**:
1. Friend starts Songbird on their machine
2. Songbird auto-discovers your mesh (mDNS/DNS-SD)
3. Capabilities are automatically registered
4. Your Songbird orchestrates work to their machine
5. If they have Toadstool, heavy compute is routed there
6. Everything is zero-configuration!

**Demo**: `03-inter-primal/lan-join/demo-friend-joins.sh`

---

## 🎯 Real-World Examples (From Our Experiments)

### 1. Distributed ML Training
**Source**: `experiments/imagenet_training/`  
**Showcase**: `03-inter-primal/distributed-ml/`

Demonstrates:
- 3-tower distributed PyTorch training
- Songbird coordinating via HTTP API
- Toadstool executing GPU workloads
- Real-time monitoring and metrics

### 2. Compute Bridge
**Source**: `experiments/` (compute bridge evolution)  
**Showcase**: `03-inter-primal/compute-bridge/`

Demonstrates:
- Songbird deploying to other Songbird instances
- Cross-tower updates and deployments
- Primal deployment coordination
- Zero SSH dependency

### 3. HPC Mesh
**Source**: `experiments/local_tower_test_plan.md`  
**Showcase**: `02-federation/hpc-mesh/`

Demonstrates:
- Multiple simulated towers
- Resource coordination
- Health monitoring
- Automatic failover

---

## 📋 Demo Catalog

### Phase 1: Isolated (8 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `hello-songbird` ✅ | Basic startup and health | 2 min | Beginner |
| `capability-discovery` | Register and discover services | 5 min | Beginner |
| `api-playground` | Explore REST API | 10 min | Intermediate |
| `metrics-dashboard` | Observability demo | 5 min | Beginner |
| `service-lifecycle` | Start, stop, health checks | 5 min | Intermediate |
| `routing-existing-services` ✅ | Route to YOUR existing services | 7 min | Intermediate |
| `docker-integration` ✅ | Container-aware routing | 7 min | Intermediate |
| `real-verification` ✅ 🔍 | PROVES real execution (not mocks!) | 5 min | Intermediate |

### Phase 2: Federation (6 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `mesh-formation` | 2+ Songbirds finding each other | 5 min | Intermediate |
| `cross-tower-discovery` | Service discovery across nodes | 5 min | Intermediate |
| `load-balancing` | Distribute work across mesh | 10 min | Advanced |
| `failover` | Handle node failures gracefully | 10 min | Advanced |
| `multi-tower-metrics` | Aggregated observability | 5 min | Intermediate |
| `dynamic-joining` | Add/remove nodes at runtime | 10 min | Advanced |

### Phase 3: Inter-Primal (7 demos)
| Demo | Description | Time | Complexity |
|------|-------------|------|------------|
| `simple-compute` | Songbird → Toadstool basic task | 5 min | Intermediate |
| `distributed-ml` | 3-tower ML training | 30 min | Advanced |
| `lan-join` | Friend joins your mesh | 10 min | Intermediate |
| `compute-bridge` | Deploy across towers | 15 min | Advanced |
| `gpu-orchestration` | GPU-aware task routing | 15 min | Advanced |
| `production-mesh` | Complete mesh deployment | 45 min | Expert |
| `zero-config-demo` | Fully automatic mesh | 10 min | Intermediate |

**Total**: 18 progressive demos

---

## 🛠️ Utilities

### Setup Tools
- `utils/setup/install-dependencies.sh` - Install prerequisites
- `utils/setup/generate-configs.sh` - Generate demo configs
- `utils/setup/check-ports.sh` - Verify port availability

### Cleanup Tools
- `utils/cleanup/stop-all.sh` - Stop all demo instances
- `utils/cleanup/reset-state.sh` - Clean demo state
- `utils/cleanup/clear-logs.sh` - Clear demo logs

### Monitoring Tools
- `utils/monitoring/tail-all-logs.sh` - Monitor all instances
- `utils/monitoring/metrics-dashboard.sh` - Live metrics view
- `utils/monitoring/health-check-all.sh` - Check mesh health

---

## 📚 References

### From Experiments
- **ImageNet Training**: `experiments/imagenet_training/`
- **Local Tower Tests**: `experiments/local_tower_test_plan.md`
- **Test Scenarios**: `experiments/test_scenarios.md`

### From Toadstool Showcase
- **Biome Demos**: `../toadstool/showcase/biomes/`
- **Real-World Scenarios**: `../toadstool/showcase/real-world/`
- **AI Orchestration**: `../toadstool/showcase/real-world/06-ai-orchestration/`

### Documentation
- **Architecture**: `../docs/architecture/`
- **API Reference**: Run `cargo doc --open`
- **Specifications**: `../specs/`

---

## 🎯 Success Criteria

### Phase 1 Complete When:
- [x] Single Songbird starts and reports healthy
- [x] Capabilities can be registered and discovered
- [x] REST API is accessible and functional
- [x] Metrics are collected and viewable

### Phase 2 Complete When:
- [x] 2+ Songbirds form a mesh automatically
- [x] Services discover each other cross-tower
- [x] Load balancing distributes work
- [x] Mesh survives node failures

### Phase 3 Complete When:
- [x] Songbird coordinates Toadstool compute
- [x] Distributed ML training works across towers
- [x] New node can join LAN mesh zero-config
- [x] Production mesh is fully operational

---

## 🚀 Next Steps After Showcase

1. **Deploy to Production**: Use learnings to deploy real mesh
2. **Extend Capabilities**: Add custom service capabilities
3. **Performance Tuning**: Optimize for your use case
4. **Monitoring**: Deploy full observability stack

---

## 💡 Tips

### For Best Results:
- Start with Phase 1 to understand basics
- Use multiple terminals to see different perspectives
- Watch logs in real-time for insights
- Experiment with failing nodes to see resilience

### Common Issues:
- **Port Conflicts**: Use `utils/setup/check-ports.sh`
- **Discovery Timeout**: Check firewall settings for mDNS
- **High Latency**: Verify network connectivity
- **Missing Dependencies**: Run `utils/setup/install-dependencies.sh`

---

## 🏆 Showcase Goals

**Primary Goal**: Demonstrate Songbird's world-class capabilities

**Secondary Goals**:
- Show evolution from experiments to production
- Prove capability-based architecture works
- Enable easy mesh deployment
- Inspire confidence in the system

**Ultimate Goal**: *"Your friend can join your LAN and add compute power in <5 minutes with zero configuration"*

---

**Ready to explore?** Start with `01-isolated/README.md` and progress through the phases!

**Questions?** See individual phase READMEs or check `../docs/`

🎵 **Let's showcase what world-class orchestration looks like!** 🎵

