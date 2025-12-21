# Showcase 09: Local Compute Orchestration

**Status:** ✅ Complete  
**Goal:** Demonstrate Songbird as a local compute orchestrator  
**Purpose:** Foundation for compute before inter-primal integration

## Overview

This showcase demonstrates Songbird's fundamental capability: **managing compute tasks on a single tower**. Even before inter-primal integration, Songbird provides a unified API for task execution, resource management, and lifecycle control.

## Demos

### 01. Spawn Simple Task ✅
**File:** `01-spawn-simple-task.sh`  
**Purpose:** Basic task execution  
**Status:** Complete

**What it demonstrates:**
- Submit a simple command to Songbird
- Songbird executes locally
- Return output to user
- Timeout management

**Run:**
```bash
./01-spawn-simple-task.sh
```

### 02. Spawn Python Task ✅
**File:** `02-spawn-python-task.sh`  
**Purpose:** Python runtime execution  
**Status:** Complete

**What it demonstrates:**
- Execute Python code via Songbird
- Import standard library modules
- Process data and return JSON
- Multi-line script support

**Run:**
```bash
./02-spawn-python-task.sh
```

**Features shown:**
- Python 3 runtime
- Import statements (`sys`, `json`, `math`, `datetime`)
- Data processing
- JSON output parsing

### 03. Spawn Concurrent Tasks ✅
**File:** `03-spawn-concurrent-tasks.sh`  
**Purpose:** Parallel execution  
**Status:** Complete

**What it demonstrates:**
- Submit multiple tasks simultaneously
- Non-blocking task submission
- Parallel execution (not sequential)
- Result collection

**Run:**
```bash
./03-spawn-concurrent-tasks.sh
```

**Key insight:** 5 tasks with 1-3s delays complete in ~3-4s total (parallel), not ~10-15s (sequential)

### 04. Resource Monitoring ✅
**File:** `04-resource-monitoring.sh`  
**Purpose:** System and service monitoring  
**Status:** Complete

**What it demonstrates:**
- System resource tracking
- Service registry statistics
- Port allocation monitoring
- Health status endpoints

**Run:**
```bash
./04-resource-monitoring.sh
```

## Architecture Pattern

```
USER
  ↓
SONGBIRD (Local Orchestrator)
  ↓
LOCAL SYSTEM (bash, python, etc.)
```

This is the **foundation**. Next step:

```
USER
  ↓
SONGBIRD (Universal Orchestrator)
  ↓ (via service registry)
TOADSTOOL (Compute Primal) ← Showcase 10
```

## Key Capabilities

✅ **Task Submission**: Simple command execution  
✅ **Runtime Support**: Bash, Python (extensible)  
✅ **Timeout Management**: Prevent hanging tasks  
✅ **Parallel Execution**: Concurrent task orchestration  
✅ **Resource Monitoring**: System and service stats  

## Running All Demos

```bash
cd showcase/09-local-compute

# Make all executable
chmod +x *.sh

# Run in sequence
./01-spawn-simple-task.sh
./02-spawn-python-task.sh
./03-spawn-concurrent-tasks.sh
./04-resource-monitoring.sh
```

## Next Steps

After running these demos:

1. **Explore Inter-Primal Integration:**
   ```bash
   cd ../10-inter-primal-foundation
   ./02-toadstool-live-integration.sh
   ```

2. **Compare Local vs Distributed:**
   - Local: Songbird executes directly
   - Distributed: Songbird routes to Toadstool
   - Same API, different backends!

---

*Local Compute Foundation - December 20, 2025*  
*Building Blocks for Distributed Orchestration*
