# 🚀 Primal Sovereignty Quick Start

**TL;DR**: Songbird works alone. BearDog makes it better. Neither is required for the other.

---

## ⚡ 30-Second Overview

```yaml
The Model:
  • Each primal is fully functional alone (sovereign)
  • Multiple primals create network effects (better together)
  • If a primal goes down, others continue (graceful degradation)
  
For Songbird Remote Execution:
  • Tier 1: Songbird alone → Works perfectly for LAN
  • Tier 2: Songbird + BearDog → Enhanced security
  • Tier 3: All primals → Maximum security
```

---

## 🎯 Quick Examples

### Example 1: LAN Development (Just Songbird)

```bash
# Start Songbird execution agent
cargo run -p songbird-execution-agent -- \
  --port 8080 \
  --enable-auth \
  --token "secret123"

# ✅ Result: Fully functional
# - Token authentication
# - Command validation  
# - Background jobs
# - Resource limits
# - Appropriate for LAN use
```

**Security Tier**: 1 (Sovereign)  
**Dependencies**: None  
**Status**: ✅ Production-ready for LAN

---

### Example 2: Production (Songbird + BearDog)

```bash
# Terminal 1: Start BearDog
cd ../beardog && cargo run --release

# Terminal 2: Start Songbird with BearDog discovery
export BEARDOG_SECURITY_ENDPOINT="http://localhost:8443"
cargo run -p songbird-execution-agent -- \
  --port 8080 \
  --enable-beardog-discovery

# ✅ Result: Enhanced security
# - All Tier 1 features
# - HSM-backed authentication (BearDog)
# - Enhanced threat assessment (BearDog)
# - Cryptographic audit trails (BearDog)
```

**Security Tier**: 2 (Network Effect)  
**Dependencies**: None (BearDog is optional enhancement)  
**Status**: ✅ Production-ready

---

### Example 3: BearDog Failure (Graceful Degradation)

```bash
# Scenario: BearDog running, then crashes
# Songbird automatically:

1. Detects BearDog timeout
   └─> Log: "⚠️ BearDog unavailable, falling back to sovereign"

2. Falls back to Tier 1
   └─> Continues with token auth

3. No service interruption
   └─> All requests continue processing

4. Periodically tries to rediscover BearDog
   └─> Every 5 minutes

5. When BearDog returns:
   └─> Log: "✅ BearDog rediscovered - enhanced security enabled"
   └─> Upgrades back to Tier 2
```

**Impact**: Zero downtime, reduced security features temporarily

---

## 🏗️ The Three Tiers

### Tier 1: Sovereign (Songbird Alone)

```yaml
What It Provides:
  ✅ Token-based authentication
  ✅ Dangerous command blocking (rm -rf /, fork bombs, etc.)
  ✅ Resource limits (timeout, concurrency)
  ✅ Local audit logging
  ✅ Background job management

When To Use:
  • LAN deployments
  • Development environments
  • Staging environments
  • Disconnected operation
  • When BearDog unavailable

Trust Level: 0.8 (good)
Dependencies: 0 (none)
Availability: 100% (always)
```

### Tier 2: Network Effect (Songbird + BearDog)

```yaml
What It Adds:
  ✅ All Tier 1 features +
  ✅ HSM-backed authentication
  ✅ Enhanced threat assessment
  ✅ Cryptographic audit trails
  ✅ Hardware security module integration

When To Use:
  • Production deployments
  • Internet-facing services
  • Higher security requirements
  • Compliance environments

Trust Level: 0.95 (excellent)
Dependencies: 0 (BearDog is optional)
Availability: 99.9% (typical)
Fallback: Tier 1 if BearDog unavailable
```

### Tier 3: Federation (All Primals)

```yaml
What It Adds:
  ✅ All Tier 2 features +
  ✅ Distributed audit trails (Squirrel)
  ✅ ML-based anomaly detection (ToadStool)
  ✅ Multi-primal threat correlation
  ✅ Comprehensive compliance automation

When To Use:
  • Maximum security requirements
  • Compliance-heavy environments
  • Multi-datacenter deployments
  • Financial/healthcare/government

Trust Level: 0.99 (maximum)
Dependencies: 0 (all primals optional)
Availability: 99.99% (no single point of failure)
Fallback: Tier 2 or Tier 1 as primals unavailable
```

---

## 🔧 Configuration

### Minimal (Tier 1)

```toml
# config.toml
[agent]
port = 8080
enable_auth = true
auth_tokens = ["secret123", "secret456"]
max_concurrent_jobs = 10
```

```bash
cargo run -p songbird-execution-agent -- --config config.toml
```

**Result**: Sovereign security, fully functional

---

### With BearDog Discovery (Tier 2)

```toml
# config.toml
[agent]
port = 8080
enable_auth = true
auth_tokens = ["secret123", "secret456"]
max_concurrent_jobs = 10

[beardog]
enable_discovery = true
endpoint = "http://beardog:8443"  # Optional, can auto-discover
```

```bash
cargo run -p songbird-execution-agent -- --config config.toml
```

**Result**: Enhanced security when BearDog available, falls back to Tier 1 if not

---

### Full Federation (Tier 3)

```toml
# config.toml
[agent]
port = 8080
enable_auth = true
auth_tokens = ["secret123", "secret456"]
max_concurrent_jobs = 10

[primal_discovery]
enabled = true

[beardog]
enable_discovery = true

[squirrel]
enable_discovery = true

[toadstool]
enable_discovery = true
```

```bash
cargo run -p songbird-execution-agent -- --config config.toml
```

**Result**: Maximum security, gracefully uses available primals

---

## 🧪 Testing the Model

### Test 1: Sovereignty (Works Alone)

```bash
# Start just Songbird
cargo run -p songbird-execution-agent -- --port 8080 --token secret123

# In another terminal
curl -X POST http://localhost:8080/execute \
  -H "Authorization: Bearer secret123" \
  -H "Content-Type: application/json" \
  -d '{"command": "echo hello", "background": false}'

# ✅ Expected: Success with Tier 1 security
```

### Test 2: Network Effect (Enhanced)

```bash
# Start BearDog first
cd ../beardog && cargo run &

# Start Songbird with discovery
export BEARDOG_SECURITY_ENDPOINT="http://localhost:8443"
cargo run -p songbird-execution-agent -- --port 8080 --enable-beardog-discovery

# Check logs:
# ✅ Expected: "BearDog discovered - enhanced security enabled"

# Make request (same as Test 1)
# ✅ Expected: Success with Tier 2 security
```

### Test 3: Graceful Degradation

```bash
# With Songbird running in Tier 2 (from Test 2)

# Kill BearDog
pkill -f beardog

# Check Songbird logs:
# ✅ Expected: "BearDog unavailable, falling back to sovereign"

# Make request (same as Test 1)
# ✅ Expected: Still succeeds, now using Tier 1

# Restart BearDog
cd ../beardog && cargo run &

# Wait ~5 minutes for rediscovery, or:
curl http://localhost:8080/admin/rediscover

# Check logs:
# ✅ Expected: "BearDog rediscovered - enhanced security enabled"
```

---

## 📊 Quick Decision Guide

```
┌─────────────────────────────────────────┐
│ What's your deployment scenario?        │
└─────────────────────────────────────────┘
              │
              ├─> LAN only, dev/staging
              │   └─> Use Tier 1 (Songbird alone)
              │       ✅ Simple, reliable, appropriate
              │
              ├─> Production, internet-facing
              │   └─> Use Tier 2 (Songbird + BearDog)
              │       ✅ Enhanced security, HSM-backed
              │
              └─> Maximum security, compliance
                  └─> Use Tier 3 (All primals)
                      ✅ ML detection, distributed audit
```

---

## 🎯 Key Principles

### 1. Sovereignty First

```yaml
Every Primal:
  ✅ Is fully functional alone
  ✅ Has zero required dependencies
  ✅ Can operate disconnected
  ✅ Provides complete core features
```

### 2. Network Effects Second

```yaml
Multiple Primals:
  ✅ Discover each other via capability
  ✅ Cooperate for enhanced features
  ✅ Each adds optional value
  ✅ Gracefully degrade if unavailable
```

### 3. Never Block

```yaml
If Another Primal Unavailable:
  ✅ Continue with reduced features
  ✅ Fall back to sovereign capabilities
  ✅ Log degradation for monitoring
  ✅ Periodically try to rediscover
  ✅ Zero service interruption
```

---

## 📚 More Information

- **Full Architecture**: `PRIMAL_SOVEREIGNTY_SECURITY_ARCHITECTURE.md`
- **Implementation Summary**: `PRIMAL_SOVEREIGNTY_IMPLEMENTATION_SUMMARY.md`
- **Correction Report**: `SECURITY_ARCHITECTURE_CORRECTION.md`
- **Remote Execution API**: `specs/REMOTE_EXECUTION_API_SPEC.md`

---

## 💡 Remember

```
Each primal knows itself and is sovereign.

If BearDog goes down → Songbird continues normally
If Songbird goes down → BearDog continues normally
If both go down → Squirrel continues normally

Together we're better. Apart we're functional.

This is the ecoPrimals way.
```

---

**Start Here**: Use Tier 1 for LAN, Tier 2 for production, Tier 3 for maximum security.

**Key Insight**: It's not a dependency graph, it's a network effect graph.

🎉 **Now go build something sovereign!**

