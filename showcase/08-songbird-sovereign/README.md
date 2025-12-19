# 🎵 Showcase 08: Songbird as Sovereign Primal

**Purpose:** Demonstrate Songbird's evolution as a standalone sovereign primal with built-in access control and security.

**Key Concepts:**
- Fail-safe sovereignty (works independently)
- Graduated information disclosure
- Capability-based access control
- Enhanced by BearDog (but not dependent)

---

## What This Showcase Demonstrates

### 1. Songbird Standalone (Q1 2025)

**Without BearDog:**
- ✅ JWT-based authentication
- ✅ Role-based access control
- ✅ Information layer separation
- ✅ Audit logging
- ✅ Hardware key support (SoloKey)

**Capabilities:**
- Students submit tasks, see educational info
- TAs debug, see operational info
- Professors manage, see administrative info
- Admins configure, see infrastructure info

---

### 2. Songbird + BearDog (Q2 2025)

**With BearDog Integration:**
- 🔐 Genetic identity (theft-proof)
- 🔐 Hardware-bound tokens
- 🔐 Zero-knowledge capability proofs
- 🔐 Instant revocation

**Enhanced Capabilities:**
- Identity can't be stolen (genetic encryption)
- Tokens bound to devices (hardware)
- Capability delegation with proofs
- Cryptographic audit trail

---

## Directory Structure

```
showcase/08-songbird-sovereign/
├── README.md                    # This file
├── 01-standalone-security/      # Songbird without BearDog
│   ├── README.md
│   ├── config/
│   │   └── access-control.toml
│   ├── examples/
│   │   ├── student-access.sh
│   │   ├── ta-access.sh
│   │   ├── professor-access.sh
│   │   └── admin-access.sh
│   └── tokens/
│       ├── student-token.json
│       ├── ta-token.json
│       └── admin-token.json
│
├── 02-information-layers/       # Graduated disclosure examples
│   ├── README.md
│   ├── public-info.json         # What anonymous users see
│   ├── educational-info.json    # What students see
│   ├── operational-info.json    # What TAs see
│   ├── administrative-info.json # What professors see
│   └── infrastructure-info.json # What admins see
│
├── 03-beardog-enhanced/         # With BearDog integration
│   ├── README.md
│   ├── genetic-identity.md      # How genetic identity works
│   ├── hardware-binding.md      # SoloKey integration
│   └── examples/
│       ├── genetic-auth.sh
│       └── hardware-key-ops.sh
│
└── COMPARISON.md                # Standalone vs BearDog-enhanced
```

---

## Examples

### Example 1: Student Submits Task

**Standalone Mode:**
```bash
# Student gets JWT token from Songbird
TOKEN=$(curl -X POST http://songbird:8080/api/auth/login \
  -d '{"student_id": "student-xyz", "course_id": "CSE-847"}' \
  | jq -r '.token')

# Submit task with token
curl -X POST http://songbird:8080/api/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -d @task.json
```

**Response (Educational Layer):**
```json
{
  "task_id": "task-abc123",
  "status": "queued",
  "educational_info": {
    "queue_position": 3,
    "estimated_wait_minutes": 5,
    "will_use_data_parallelism": true,
    "estimated_shards": 2
  }
}
```

**BearDog-Enhanced Mode:**
```bash
# Student authenticates with genetic identity
GENETIC_TOKEN=$(beardog authenticate \
  --identity student-xyz \
  --hardware-key solokey \
  --course CSE-847)

# Submit task with genetic identity token
curl -X POST http://songbird:8080/api/tasks \
  -H "Authorization: BearDog $GENETIC_TOKEN" \
  -d @task.json
```

**Benefits:**
- Identity can't be stolen
- Token bound to student's device
- Instant revocation if compromised

---

### Example 2: TA Debugs Student Task

**Standalone Mode:**
```bash
# TA has elevated access
curl -X GET http://songbird:8080/api/tasks/task-abc123/debug \
  -H "Authorization: Bearer $TA_TOKEN"
```

**Response (Operational Layer):**
```json
{
  "task_id": "task-abc123",
  "status": "failed",
  "educational_info": {
    "sharding": {...}
  },
  "operational_info": {
    "failure_details": {
      "error": "CUDA out of memory",
      "node": "compute-node-alpha",
      "attempted_allocation_gb": 18,
      "available_vram_gb": 8,
      "suggestions": [
        "Reduce batch size",
        "Enable gradient checkpointing"
      ]
    },
    "logs_excerpt": "..."
  }
}
```

---

### Example 3: Admin Remote Access

**Standalone Mode:**
```bash
# Admin requires 2FA + hardware key
# First: VPN connection required
vpn-connect --profile songbird-admin

# Second: Get admin token
ADMIN_TOKEN=$(curl -X POST http://songbird:8080/api/auth/admin \
  -d '{"admin_id": "kevin", "password": "...", "totp": "123456"}' \
  | jq -r '.token')

# Third: Sensitive operations require hardware key challenge
CHALLENGE=$(curl -X POST http://songbird:8080/api/admin/challenge \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  | jq -r '.challenge')

# Fourth: Sign challenge with SoloKey
RESPONSE=$(solokey sign $CHALLENGE)

# Fifth: Access infrastructure info
curl -X GET http://songbird:8080/api/infrastructure \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "X-Hardware-Key-Response: $RESPONSE"
```

**Response (Infrastructure Layer):**
```json
{
  "nodes": [
    {
      "name": "Eastgate",
      "internal_ip": "192.168.1.144:8000",
      "gpu": "RTX 3090 24GB",
      "uptime_hours": 847,
      "temperature_c": 67
    }
  ],
  "system_logs": "...",
  "security_events": [...]
}
```

---

## Security Comparison

### Standalone Mode

**Strengths:**
- ✅ No external dependencies
- ✅ Works immediately
- ✅ Audit logging
- ✅ Role-based access
- ✅ Hardware key support

**Limitations:**
- ⚠️ Token theft = compromise
- ⚠️ Revocation requires state sync
- ⚠️ No genetic identity

**When to Use:**
- Testing and development
- Controlled environments (campus LAN)
- When BearDog not yet available

---

### BearDog-Enhanced Mode

**Strengths:**
- 🔐 Identity theft impossible
- 🔐 Hardware-bound operations
- 🔐 Instant revocation
- 🔐 Zero-knowledge proofs
- 🔐 Cryptographic audit trail

**Limitations:**
- Requires BearDog infrastructure
- More complex setup

**When to Use:**
- Production deployments
- Internet-facing services
- High-security environments
- Multi-organization federation

---

## Implementation Status

### Phase 1: Standalone (Q1 2025)

- [ ] JWT token system
- [ ] Role definitions
- [ ] Capability checking
- [ ] Information layer builders
- [ ] Audit logging
- [ ] SoloKey integration for admin

**Dependencies:** None  
**Timeline:** 4 weeks  
**Status:** Design complete, implementation pending

---

### Phase 2: BearDog Integration (Q2 2025)

- [ ] Genetic identity verification
- [ ] Hardware-bound tokens
- [ ] Zero-knowledge capability proofs
- [ ] Instant revocation system
- [ ] Enhanced audit trail

**Dependencies:** BearDog genetic encryption  
**Timeline:** 4 weeks (after BearDog available)  
**Status:** Planned

---

## Testing Scenarios

### Scenario 1: Student Workflow

1. Student authenticates (standalone: JWT, enhanced: genetic)
2. Submits MNIST training task
3. Views task status (sees educational info)
4. Gets results with receipt
5. Understands how distribution worked

**Educational Value:** Student learns distributed systems without seeing infrastructure

---

### Scenario 2: TA Support

1. Student task fails
2. TA authenticates (operational access)
3. Views failure details (sees operational info)
4. Identifies issue (OOM error)
5. Helps student fix code

**Operational Value:** TA can debug without seeing full infrastructure

---

### Scenario 3: Professor Research

1. Professor analyzes class performance
2. Views aggregate statistics (administrative info)
3. Sees node utilization patterns
4. Exports data for research paper
5. No student data exposed

**Research Value:** Professor gets insights without compromising privacy

---

### Scenario 4: Remote Admin

1. Admin needs to restart service (from home)
2. Connects via VPN
3. Authenticates with password + TOTP
4. Sensitive operation requires SoloKey tap
5. Operation executed and logged

**Security Value:** Full control maintained, hardware-backed, audited

---

## Configuration Examples

### `config/access-control-standalone.toml`

```toml
[access_control]
mode = "standalone"
default_visibility = "public"

[authentication]
token_type = "jwt"
token_expiry_hours = 24
require_2fa_for_admin = true
require_hardware_key_for_infrastructure = true

[roles.student]
capabilities = [
    "view_public_info",
    "view_educational_info",
    "submit_task",
    "view_own_tasks",
]
max_concurrent_tasks = 3
max_gpu_hours_per_week = 10

[roles.admin]
capabilities = ["all"]
require_hardware_key = true
require_vpn = true
session_timeout_minutes = 30

[information_disclosure]
anonymize_ips = true
anonymize_node_names = false
show_sharding_to_students = true
```

---

### `config/access-control-beardog.toml`

```toml
[access_control]
mode = "beardog-enhanced"
default_visibility = "public"

[authentication]
token_type = "beardog_genetic"
genetic_identity_required = true
hardware_binding_required = true
zero_knowledge_proofs = true

[beardog_integration]
genetic_verification_endpoint = "http://beardog:8081/verify"
hardware_key_types = ["solokey", "yubikey", "titan_m2"]
instant_revocation = true

# Rest same as standalone
[roles.student]
capabilities = [...]
```

---

## Next Steps

1. **Implement standalone access control** (Q1 2025)
2. **Test with Prof. Murillo's class** (Q1 2025)
3. **Collect feedback and refine** (Q1 2025)
4. **Integrate BearDog when available** (Q2 2025)
5. **Deploy internet-facing** (Q3 2025)

---

**Songbird: Sovereign coordination with graduated security.** 🎵🔐

From simple JWT tokens to genetic identity, from campus LANs to global internet, from single coordinator to federated mesh—all while maintaining fail-safe sovereignty.

