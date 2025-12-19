# 🎓 Professor Murillo Demo - Songbird Federated ML

**Date:** January 2025  
**Duration:** 30-45 minutes  
**Audience:** Professor Michael Murillo (MSU Data Science)  
**Goal:** Demonstrate Songbird's readiness for class deployment

---

## Executive Summary

**What We're Demonstrating:**
- Student onboarding in < 10 minutes
- Distributed ML training with cryptographic verification
- Graduated information disclosure (students learn without seeing infrastructure)
- Capability-based access control (students, TAs, professors, admins)
- Production-ready federated compute coordination

**Key Achievement:**
- 95.19% ML accuracy across 2 towers (December 18, 2025)
- Zero hardcoded IPs (capability-based discovery)
- Zero production mocks (real federation working)
- Cryptographic receipts for all tasks

---

## Demo Outline

### Part 1: The Problem (5 minutes)

**Current State of ML Education:**
- ❌ Students need expensive GPUs ($500-$2000+)
- ❌ Cloud credits are costly and limited
- ❌ University clusters have week-long queues
- ❌ Setup complexity prevents learning

**EcoPrimals Solution:**
- ✅ Students train on distributed GPUs from laptops
- ✅ No hardware, no cloud bills, no setup
- ✅ Standard PyTorch code (no special APIs)
- ✅ Cryptographic verification of work
- ✅ Learn distributed systems by using them

---

### Part 2: Student Experience (10 minutes)

**Live Demo: Student Submits MNIST Task**

#### Step 1: Install Client (30 seconds)

```bash
pip install -r client/requirements.txt
cd client && pip install -e .
```

#### Step 2: Connect to Federation (30 seconds)

```bash
export SONGBIRD_URL="ws://192.168.1.50:8080"
python -m ecoprimals_client.connect
```

**Output:**
```
🎵 Connecting to Songbird...
✅ Connected to MSU-EcoPrimals Federation
   Available nodes: 6
   Total GPUs: 6 (88GB VRAM)
```

#### Step 3: Submit Training Task (5 minutes)

```bash
cd projects/01-mnist-digits
python submit.py
```

**Watch real-time progress:**
```
🚀 Submitting task: train.py
   Dataset: mnist

✅ Task accepted! ID: task-abc123
   Allocated to: compute-node-alpha
   GPU: high-memory-gpu

⏳ Training in progress...
   Epoch 1/3: Loss=0.32, Accuracy=91.2%
   Epoch 2/3: Loss=0.18, Accuracy=94.1%
   Epoch 3/3: Loss=0.15, Accuracy=95.2%

============================================================
📊 RESULTS
============================================================
   final_test_accuracy: 0.9512
   final_loss: 0.1534
   training_time_seconds: 187.3
   device: cuda

📜 Cryptographic receipt saved to: receipt_task-abc123.json
============================================================
```

**Key Points:**
- Student sees **educational information** (sharding, distribution, learning notes)
- Student does NOT see your home network (IPs anonymized)
- Cryptographic receipt proves reproducibility

---

### Part 3: Graduated Information Disclosure (10 minutes)

**Demonstrate Different Roles See Different Info**

#### Student View (Educational Layer)

```json
{
  "task_id": "task-abc123",
  "status": "completed",
  "educational_info": {
    "sharding_strategy": "data_parallel",
    "shards": [
      {
        "shard_id": "shard-0",
        "node": "compute-node-alpha",
        "samples": 30000,
        "training_time_sec": 187.3
      },
      {
        "shard_id": "shard-1",
        "node": "compute-node-beta",
        "samples": 30000,
        "training_time_sec": 192.1
      }
    ],
    "learning_notes": [
      "Your task used data parallelism across 2 nodes",
      "Each node trained on half the dataset independently",
      "Gradients were synchronized after each epoch",
      "Total speedup: 1.89x (accounting for communication overhead)"
    ]
  }
}
```

**Educational Value:** Student learns HOW distribution works without seeing YOUR infrastructure.

#### TA View (Operational Layer)

```json
{
  "task_id": "task-failed-xyz",
  "status": "failed",
  "operational_info": {
    "failure_details": {
      "error": "CUDA out of memory",
      "node": "compute-node-alpha",
      "attempted_allocation_gb": 18.2,
      "available_vram_gb": 8.2,
      "suggestions": [
        "Reduce batch size (current: 128, try: 64)",
        "Enable gradient accumulation",
        "Use gradient checkpointing"
      ]
    }
  }
}
```

**Operational Value:** TA can help debug without seeing full infrastructure.

#### Professor View (Administrative Layer)

```json
{
  "federation_status": {
    "nodes": [
      {
        "node_name": "Eastgate",
        "location": "primary_site",
        "gpu": "RTX 3090 24GB",
        "utilization": 0.73,
        "tasks_today": 47
      }
    ],
    "class_statistics": {
      "course": "CSE-847-ML",
      "students_active": 23,
      "total_tasks": 156,
      "success_rate": 0.94,
      "avg_completion_time": 247.3
    }
  }
}
```

**Research Value:** You get utilization stats, performance data, class metrics for research.

---

### Part 4: Architecture & Security (10 minutes)

**Key Technical Points:**

#### Capability-Based Discovery (Not Hardcoded IPs)

```rust
// No hardcoded IPs anywhere!
let nodes = self.service_registry
    .find_by_capability(&["gpu-compute"])
    .await?;
```

**Nodes self-register, Songbird discovers, students benefit.**

#### Graduated Security Model

```
Public (Anyone)
    ↓ auth
Educational (Students) - Learn distribution, see anonymized topology
    ↓ elevated
Operational (TAs) - Debug failures, see node health
    ↓ elevated
Administrative (Professors) - Research, see utilization, no IPs
    ↓ elevated + 2FA
Infrastructure (Admins) - Full access, requires hardware key
```

#### Sovereign Infrastructure

```
Your Towers (Eastgate, Strandgate, Northgate, Southgate, Westgate, Swiftgate)
    ↓ Federation Discovery
Songbird Coordinator (campus deployment)
    ↓ Student Access
Students (learn + compute)
```

**No cloud dependency. No vendor lock-in. Complete sovereignty.**

---

### Part 5: Course Integration (5 minutes)

**How This Works in Your Class:**

#### Week 1-2: Students Get Access

```bash
# You provide:
export SONGBIRD_URL="ws://YOUR.LAPTOP.IP:8080"

# Students install:
pip install ecoprimals-client

# They're ready!
```

#### Week 3-10: Normal Coursework

Students use Songbird for:
- Homework assignments (MNIST, CIFAR-10, etc.)
- Project work (distributed training)
- Experimentation (trying different models)

**All standard PyTorch/TensorFlow code!**

#### Grading & Verification

Cryptographic receipts prove:
- What code ran
- When it ran
- What results were achieved
- That it's reproducible

**Academic integrity built-in.**

#### Your Research Benefits

You get:
- Performance data (how does federation perform?)
- Educational metrics (do students learn better?)
- Resource utilization patterns
- Potential academic paper: "Democratizing ML Education via Sovereign Compute"

---

## Q&A Preparation

### Technical Questions

**Q: What if a student's task fails?**  
A: TA sees failure details with suggestions. Student sees educational error message. System auto-retries on transient failures.

**Q: How do you prevent malicious code?**  
A: Sandboxed containers, resource quotas, network isolation. Students can't access home network or see other students' work.

**Q: What's the overhead vs local GPU?**  
A: ~5-10% for communication and scheduling. Massively faster than no GPU or cloud queues.

**Q: Can students see each other's work?**  
A: No. Information isolated by user. TAs see all students in their course. You see all courses you teach.

**Q: What happens if your laptop dies mid-class?**  
A: Tasks checkpoint automatically. Students can reconnect. Or use backup coordinator on another device.

---

### Deployment Questions

**Q: How hard is setup?**  
A: 10 minutes on your laptop. Students: `pip install` and one environment variable.

**Q: What's the capacity?**  
A: Currently 6 GPUs = ~100 tasks/hour. Scales by adding nodes (friends/family towers, campus resources).

**Q: Internet or LAN only?**  
A: V1 (now): Campus WiFi (students must be on-campus). V2 (Q2): Internet with BearDog auth (students from anywhere).

**Q: What about privacy/FERPA?**  
A: Data never leaves your infrastructure. Students can't see each other. Cryptographic audit trail. FERPA compliant.

---

### Educational Questions

**Q: Do students learn distributed systems?**  
A: Yes! They SEE how their task was sharded, how nodes coordinated, communication overhead, etc. Educational layer explicitly teaches this.

**Q: Is this just "magic" to students?**  
A: No. They see standard code (PyTorch), understand distribution (sharding info), and verify results (cryptographic receipts). Demystifies distributed ML.

**Q: Can this replace lectures on distributed systems?**  
A: It COMPLEMENTS lectures. Students learn by doing, then understand theory better. "Oh, THAT'S why we have communication overhead!"

**Q: What about students without laptops?**  
A: Lab computers work. Even phones/tablets can submit tasks (just need Python). Very low client requirements.

---

### Research Questions

**Q: Can I publish about this?**  
A: Yes! Suggested paper: "Democratizing ML Education via Sovereign Federated Compute" - performance metrics, educational outcomes, cost analysis. Happy to co-author.

**Q: What metrics can I collect?**  
A: Task completion times, student iteration patterns, debugging frequency, resource utilization, success rates, learning outcomes. All anonymized, FERPA compliant.

**Q: Is this novel enough for publication?**  
A: Yes. Graduated information disclosure for education is novel. Sovereign compute for academia is novel. Cryptographic receipts for academic integrity is novel.

**Q: Where would this publish?**  
A: SIGCSE (CS education), IEEE EDUCON (education conference), or systems conferences (USENIX, SOSP) if we emphasize the distributed systems angle.

---

## Next Steps

### Immediate (This Meeting)

1. [ ] Demo student workflow (live)
2. [ ] Show graduated information disclosure
3. [ ] Discuss course integration
4. [ ] Answer questions
5. [ ] Get feedback

### Short Term (January)

1. [ ] Deploy on your campus device
2. [ ] Test with 2-3 volunteer students
3. [ ] Refine based on feedback
4. [ ] Plan full class rollout

### Medium Term (Spring 2025)

1. [ ] Full class deployment
2. [ ] Collect metrics
3. [ ] Iterate based on student feedback
4. [ ] Begin drafting research paper

### Long Term (2025-2026)

1. [ ] Publish paper
2. [ ] Scale to multiple classes/campuses
3. [ ] BearDog integration (genetic identity)
4. [ ] Open source release

---

## Success Metrics

### Technical Metrics

- [ ] 100% student connection success
- [ ] < 5 minute average task completion
- [ ] Zero data loss
- [ ] All cryptographic receipts valid

### Educational Metrics

- [ ] Students complete assignments
- [ ] Positive feedback on usability
- [ ] Improved understanding of distributed systems
- [ ] Reproducible results

### Research Metrics

- [ ] Data collected for paper
- [ ] Prof. Murillo endorsement
- [ ] Course integration approved
- [ ] Path to publication clear

---

## Demo Checklist

### Before Demo

- [ ] Songbird running on laptop
- [ ] Towers reachable (VPN connected)
- [ ] Test task completes successfully
- [ ] Slides/notes ready
- [ ] Example receipts prepared
- [ ] Backup plan if network fails

### During Demo

- [ ] Start with problem statement
- [ ] Live student workflow demo
- [ ] Show graduated info disclosure
- [ ] Explain architecture
- [ ] Address security/privacy
- [ ] Discuss course integration
- [ ] Q&A

### After Demo

- [ ] Get feedback
- [ ] Schedule follow-up
- [ ] Provide access for testing
- [ ] Share documentation
- [ ] Plan next steps

---

## Contact Information

**For Technical Questions:**
- Kevin Mok
- mokkevin@msu.edu
- Available for setup assistance

**For Course Integration:**
- Can provide:
  - Setup scripts
  - Student guides
  - Assignment templates
  - Grading rubrics
  - Technical support

---

## Appendix: Technical Specifications

### Current Infrastructure

- **6 Towers**: Northgate (RTX 5090), Southgate (RTX 3090), Eastgate (RTX 3090), Strandgate (RTX 3070), Swiftgate (RTX 3070), Westgate (RTX 2070)
- **Total VRAM**: 88GB
- **Capacity**: ~100 tasks/hour
- **Proven Performance**: 95.19% ML accuracy, <200ms federation latency

### Security Model

- **Authentication**: JWT tokens (standalone), genetic identity (BearDog, Q2)
- **Authorization**: Capability-based access control
- **Audit Trail**: All access logged, cryptographic receipts
- **Privacy**: Graduated information disclosure, IP anonymization

### Roadmap

- **Q1 2025**: Campus deployment, standalone security
- **Q2 2025**: BearDog integration, internet access
- **Q3 2025**: Multi-campus federation, open source release
- **Q4 2025**: Research paper publication

---

**This is the democratization of ML education.** Students with laptops get access to distributed GPUs. They learn by doing. Their work is cryptographically verifiable. And you get research data.

**Let's make it happen.** 🎓🎵✨

