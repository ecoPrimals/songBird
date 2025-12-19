# 🎭 Quick Start: Inter-Primal Orchestration

**Time**: 5 minutes  
**Goal**: See how Songbird discovers and coordinates other primals

---

## 🚀 Fastest Demo

```bash
# Terminal 1: Start Songbird
cd /home/eastgate/Development/ecoPrimals/songbird
cargo run --release --bin songbird-orchestrator

# Terminal 2: Run discovery demo
cd showcase/03-inter-primal
./demos/01-discover-ecosystem.sh
```

**What you'll see**:
- How Songbird discovers primals (Squirrel, Toadstool, BearDog)
- Capability mapping across the ecosystem
- Intelligent routing decisions
- Songbird's role as orchestrator

---

## 🎯 What Makes This Different?

### Toadstool's Showcase
> "Here's how I execute GPU workloads"

Focus: **What Toadstool does**

### Squirrel's Showcase
> "Here's how I route AI requests to providers"

Focus: **What Squirrel does**

### Songbird's Showcase
> "Here's how I discover Toadstool's GPU, Squirrel's AI, coordinate them together, and make the ecosystem work as one"

Focus: **How Songbird orchestrates everything**

---

## 📋 Current Demos

### 01: Discover Ecosystem ✅
**Status**: Ready  
**File**: `demos/01-discover-ecosystem.sh`  
**Shows**: Service discovery, capability mapping, routing logic

```bash
./demos/01-discover-ecosystem.sh
```

### 02: Route to Primal 🚧
**Status**: Coming Soon  
**Shows**: Real workload routing to discovered primal

### 03: Multi-Primal Workflow 🚧
**Status**: Coming Soon  
**Shows**: Coordinating multiple primals for complex task

### 04: Federation + Primals 🚧
**Status**: Coming Soon  
**Shows**: Primals across multiple towers

---

## 🎭 Songbird's Perspective

When Toadstool showcases distributed ML training, they show:
- "Here's 3 GPUs training a model"

When Songbird showcases the same thing, we show:
- "Here's how I discovered 3 towers with GPUs"
- "Here's how I coordinated the distributed setup"
- "Here's how I handled Tower B's network failure"
- "Here's how I rebalanced to remaining 2 GPUs"
- "Here's how I aggregated the final model"
- **All automatically**

**Both are valid! Different perspectives on the same workflow.**

---

## 🔮 Vision: Complete Ecosystem

### The Dream Scenario

```
Friend arrives with laptop
  ↓
Runs: ./join-mesh.sh
  ↓
Songbird discovers:
  • Friend's CPU (8 cores)
  • Friend's Squirrel (AI capability)
  ↓
Automatically:
  • Joins federation
  • Registers capabilities
  • Starts receiving work
  • Contributes to mesh
  ↓
Zero configuration
Zero hardcoding
Pure sovereignty
```

**That's what we're building towards.**

---

## 📚 Learn More

- **Full Plan**: `SONGBIRD_SHOWCASE_EVOLUTION.md`
- **Detailed README**: `README.md`
- **Other Showcases**: `../00_SHOWCASE_INDEX.md`

---

*Last Updated: December 17, 2025*  
*Status: Phase 1 (Discovery) Complete, Phase 2 (Routing) Next*

