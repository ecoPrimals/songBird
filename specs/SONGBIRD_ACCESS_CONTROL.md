# Songbird Access Control Specification

**Status:** Implemented (MethodGate + BTSP Phase 3, Waves 180–206)  
**Version:** 0.2.1  
**Dependencies:** None (Standalone), Enhanced by Security Provider  
**Note (May 2026):** This spec is a design fossil. The production implementation is `MethodGate` (see `crates/songbird-orchestrator/src/ipc/pure_rust_server/method_gate/`). Access control is enforced via `SONGBIRD_AUTH_MODE`, bearer token verification through BearDog's `auth.verify_ionic`, and transport-aware `CallerContext`. See `SECURITY.md` for the current posture.  

---

## Overview

Songbird provides **graduated information disclosure** and **capability-based access control** for federated compute coordination. It operates as a sovereign primal with built-in security, with optional integration with Security Provider for enhanced cryptographic guarantees.

---

## Core Principles

### 1. Fail-Safe Sovereignty

**Songbird must function independently:**
- ✅ Built-in access control (no external dependencies)
- ✅ Standalone authentication (tokens, API keys)
- ✅ Default-deny security posture
- ✅ Graceful degradation if Security Provider unavailable

**Enhanced by Security Provider:**
- 🔐 Genetic encryption for user identity
- 🔐 Hardware-backed key storage
- 🔐 Cryptographic capability delegation
- 🔐 Zero-knowledge proofs for access

### 2. Graduated Information Disclosure

**Information is NOT binary** (show all or nothing):
- Different roles see different detail levels
- Context-appropriate disclosure
- Educational value preserved
- Security maintained

### 3. Capability-Based Access

**Users have capabilities, not permissions:**
- "Can submit tasks" vs "is admin"
- Capabilities can be delegated
- Capabilities can be time-limited
- Capabilities compose (least privilege)

---

## Information Layers

### Layer 0: Public (Unauthenticated)

**Visible to anyone:**
```rust
pub struct PublicInfo {
    /// Task execution status
    pub status: ExecutionStatus,
    
    /// Completion time (if finished)
    pub completion_time: Option<Duration>,
    
    /// High-level metrics
    pub result_summary: Option<MetricsSummary>,
}

pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

pub struct MetricsSummary {
    pub success: bool,
    pub duration_sec: f64,
    // No detailed metrics, node info, or internal state
}
```

**Use Cases:**
- Public dashboards
- Status checks without authentication
- Federation health indicators

---

### Layer 1: Educational (Students, Learners)

**Visible to authenticated students:**
```rust
pub struct EducationalInfo {
    /// How the task was distributed
    pub sharding_strategy: ShardingInfo,
    
    /// Anonymized resource allocation
    pub resource_allocation: AnonymizedResourceMap,
    
    /// Anonymized topology
    pub node_topology: AnonymizedTopology,
    
    /// Task execution graph
    pub execution_flow: TaskGraph,
    
    /// Educational context
    pub learning_notes: Vec<String>,
}

pub struct ShardingInfo {
    pub strategy: String,  // "data_parallel", "model_parallel", etc.
    pub shard_count: usize,
    pub shards: Vec<ShardDetail>,
}

pub struct ShardDetail {
    pub shard_id: String,
    pub samples: usize,
    pub node: String,  // "compute-node-alpha" (anonymized)
    pub gpu: String,   // "high-memory-gpu" (generic)
    pub training_time_sec: f64,
}

pub struct AnonymizedTopology {
    pub nodes: Vec<AnonymousNode>,
    pub edges: Vec<Connection>,
}

pub struct AnonymousNode {
    pub node_id: String,  // "node-alpha", "node-beta"
    pub capabilities: Vec<String>,
    pub gpu_class: String,  // "high-memory", "efficient", "balanced"
    // NO: actual names, IPs, locations
}
```

**Use Cases:**
- Students learning distributed systems
- Understanding how federation works
- Analyzing distributed ML patterns
- Educational research

**Example Response:**
```json
{
  "task_id": "task-abc123",
  "status": "completed",
  "educational_info": {
    "sharding_strategy": "data_parallel",
    "shards": [
      {
        "shard_id": "shard-0",
        "samples": 30000,
        "node": "compute-node-alpha",
        "gpu": "high-memory-gpu",
        "training_time_sec": 187.3
      },
      {
        "shard_id": "shard-1",
        "samples": 30000,
        "node": "compute-node-beta",
        "gpu": "high-memory-gpu",
        "training_time_sec": 192.1
      }
    ],
    "topology": {
      "nodes": ["compute-node-alpha", "compute-node-beta"],
      "connection": "high_bandwidth_fabric"
    },
    "learning_notes": [
      "Your task used data parallelism across 2 nodes",
      "Each node trained on half the dataset independently",
      "Gradients were synchronized after each epoch",
      "Total speedup: 1.89x (accounting for communication overhead)"
    ]
  }
}
```

---

### Layer 2: Operational (TAs, Support Staff)

**Visible to teaching assistants and support:**
```rust
pub struct OperationalInfo {
    /// Node health at time of execution
    pub node_health: Vec<NodeHealthStatus>,
    
    /// Queue status when task submitted
    pub queue_status: QueueStatus,
    
    /// Detailed failure reasons
    pub failure_details: Option<FailureContext>,
    
    /// Performance metrics
    pub performance_metrics: PerformanceData,
    
    /// Resource constraints encountered
    pub constraints: Vec<ResourceConstraint>,
}

pub struct NodeHealthStatus {
    pub node_id: String,  // Still anonymized
    pub status: HealthStatus,
    pub gpu_utilization: f32,
    pub memory_utilization: f32,
    pub temperature_c: Option<f32>,
    pub last_heartbeat: DateTime<Utc>,
}

pub struct FailureContext {
    pub error_type: String,
    pub error_message: String,
    pub node: String,  // Anonymized
    pub timestamp: DateTime<Utc>,
    pub recovery_suggestions: Vec<String>,
    pub logs_excerpt: String,  // Last N lines
}
```

**Use Cases:**
- TAs helping students debug
- Support staff diagnosing issues
- Performance troubleshooting
- Resource optimization

**Example Response:**
```json
{
  "task_id": "task-failed-xyz",
  "status": "failed",
  "operational_info": {
    "failure_details": {
      "error_type": "OutOfMemory",
      "error_message": "CUDA out of memory: attempted to allocate 18.2 GB",
      "node": "compute-node-alpha",
      "suggestions": [
        "Reduce batch size (current: 128, try: 64)",
        "Enable gradient accumulation",
        "Use gradient checkpointing"
      ],
      "logs_excerpt": "..."
    },
    "node_health": {
      "node_id": "compute-node-alpha",
      "gpu_utilization": 0.98,
      "available_vram_gb": 8.2
    }
  }
}
```

---

### Layer 3: Administrative (Professors, PIs, Managers)

**Visible to course instructors and principal investigators:**
```rust
pub struct AdministrativeInfo {
    /// Node identities (names, not IPs)
    pub node_identities: Vec<NodeIdentity>,
    
    /// Detailed resource utilization
    pub resource_utilization: DetailedMetrics,
    
    /// User/class statistics
    pub statistics: AggregateStatistics,
    
    /// Quota management
    pub quota_status: QuotaStatus,
}

pub struct NodeIdentity {
    pub node_id: String,
    pub node_name: String,  // "Eastgate", "Strandgate" - actual names
    pub location: String,   // "primary_site", "secondary_site"
    pub capabilities: Vec<String>,
    pub hardware: HardwareSpec,
    pub utilization: UtilizationMetrics,
    // Still NO: internal IPs, system configs
}

pub struct HardwareSpec {
    pub gpu: String,  // "RTX 3090 24GB"
    pub cpu: String,  // "Intel i9-12900"
    pub ram_gb: usize,
}

pub struct AggregateStatistics {
    pub total_tasks_submitted: usize,
    pub average_completion_time_sec: f64,
    pub success_rate: f64,
    pub students_active: usize,
    pub gpu_hours_consumed: f64,
}
```

**Use Cases:**
- Professors managing courses
- PIs planning research workloads
- Resource allocation decisions
- Performance research
- Writing papers on distributed systems

**Example Response:**
```json
{
  "federation_status": {
    "nodes": [
      {
        "node_name": "Eastgate",
        "location": "primary_site",
        "gpu": "RTX 3090 24GB",
        "utilization": 0.73,
        "tasks_today": 47,
        "uptime_hours": 847
      },
      {
        "node_name": "Strandgate",
        "location": "secondary_site",
        "gpu": "RTX 3070 8GB",
        "utilization": 0.89,
        "tasks_today": 38,
        "uptime_hours": 723
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

---

### Layer 4: Infrastructure (System Administrators)

**Visible only to infrastructure owners:**
```rust
pub struct InfrastructureInfo {
    /// Full network topology with IPs
    pub full_topology: NetworkTopology,
    
    /// System logs
    pub system_logs: LogStream,
    
    /// Security events
    pub security_events: SecurityLog,
    
    /// Configuration details
    pub configuration: SystemConfig,
    
    /// Remote access information
    pub remote_access: RemoteAccessInfo,
}

pub struct NetworkTopology {
    pub nodes: Vec<NodeFull>,
    pub network_fabric: NetworkInfo,
    pub security_zones: Vec<SecurityZone>,
}

pub struct NodeFull {
    pub name: String,           // "Eastgate"
    pub internal_ip: SocketAddr,  // "192.0.2.10:8000"
    pub vpn_ip: Option<IpAddr>,   // "10.8.0.2"
    pub public_endpoint: Option<String>,
    pub ssh_access: SshInfo,
    pub hardware: DetailedHardware,
    pub services: Vec<Service>,
    pub configuration: NodeConfig,
}
```

**Use Cases:**
- Remote administration
- System maintenance
- Security monitoring
- Configuration management
- Incident response

**Requires:**
- Hardware authentication (SoloKey)
- 2FA
- VPN connection (for remote)
- Audit logging

---

## Role Definitions

### Anonymous (No Authentication)

```rust
pub struct AnonymousAccess {
    capabilities: vec![
        Capability::ViewPublicInfo,
    ],
}
```

---

### Student (Authenticated Learner)

```rust
pub struct StudentAccess {
    student_id: String,
    course_id: String,
    enrollment: Enrollment,
    
    capabilities: vec![
        Capability::ViewPublicInfo,
        Capability::ViewEducationalInfo,
        Capability::SubmitTask,
        Capability::ViewOwnTasks,
        Capability::CancelOwnTasks,
    ],
    
    quota: ResourceQuota {
        max_concurrent_tasks: 3,
        max_gpu_hours_per_week: 10,
        max_storage_gb: 5,
    },
}
```

**Token Example (Standalone):**
```json
{
  "token_type": "student",
  "student_id": "student-xyz",
  "course_id": "CSE-847",
  "issued_at": "2025-01-15T10:00:00Z",
  "expires_at": "2025-05-15T23:59:59Z",
  "capabilities": [
    "view_educational_info",
    "submit_task",
    "view_own_tasks"
  ],
  "signature": "..."
}
```

**Token Example (Security Provider-Enhanced):**
```json
{
  "token_type": "security_provider_student",
  "security_provider_identity": "genetics:abc123...",  // Genetic encryption
  "capabilities_delegated": "zkproof:xyz...",  // Zero-knowledge proof
  "hardware_bound": true,  // Bound to student's device
  "issued_by": "security_provider:professor-key",
  "signature": "ed25519:..."
}
```

---

### TA (Teaching Assistant)

```rust
pub struct TAAccess {
    ta_id: String,
    course_id: String,
    supervised_by: ProfessorId,
    
    capabilities: vec![
        Capability::ViewPublicInfo,
        Capability::ViewEducationalInfo,
        Capability::ViewOperationalInfo,
        Capability::SubmitTask,
        Capability::ViewAllStudentTasks,  // For their course
        Capability::AccessStudentLogs,
        Capability::ManageCourseQueue,
    ],
    
    quota: ResourceQuota {
        max_concurrent_tasks: 10,
        max_gpu_hours_per_week: 50,
    },
}
```

---

### Professor (Course Instructor / PI)

```rust
pub struct ProfessorAccess {
    professor_id: String,
    courses: Vec<String>,
    research_group: Option<String>,
    
    capabilities: vec![
        Capability::ViewPublicInfo,
        Capability::ViewEducationalInfo,
        Capability::ViewOperationalInfo,
        Capability::ViewAdministrativeInfo,
        Capability::SubmitTask,
        Capability::ViewAllTasks,  // For their courses
        Capability::ManageCourseUsers,
        Capability::ManageQuotas,
        Capability::ViewStatistics,
        Capability::ExportData,  // For research
    ],
    
    quota: ResourceQuota {
        max_concurrent_tasks: 50,
        max_gpu_hours_per_month: 500,
    },
}
```

---

### Admin (Infrastructure Owner)

```rust
pub struct AdminAccess {
    admin_id: String,
    access_level: AdminLevel,
    requires_2fa: bool,
    requires_hardware_key: bool,
    
    capabilities: vec![
        Capability::All,  // Full access
    ],
    
    // Additional security requirements
    authentication: AdminAuthentication {
        password: true,
        hardware_key: true,  // SoloKey required
        totp: true,
        vpn_required_for_infrastructure: true,
    },
}

pub enum AdminLevel {
    ReadOnly,     // View everything, change nothing
    Operational,  // Manage users, quotas, not infrastructure
    Full,         // Full control
}
```

---

### Remote Admin (Admin from Internet)

```rust
pub struct RemoteAdminAccess {
    admin_id: String,
    hardware_key: SoloKeyId,
    vpn_connection: VpnSession,
    
    // Start with limited access
    initial_capabilities: vec![
        Capability::ViewAdministrativeInfo,
    ],
    
    // Require hardware key interaction for sensitive operations
    elevated_capabilities: vec![
        Capability::ViewInfrastructureInfo,  // Requires key tap
        Capability::ManageConfiguration,      // Requires key tap
        Capability::RestartServices,          // Requires key tap
    ],
    
    session: RemoteSession {
        established_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        max_duration_minutes: 30,
        require_reauth_for_elevation: true,
    },
    
    audit: AuditLog {
        all_actions_logged: true,
        log_location: "append_only_log",
    },
}
```

---

### Federated Organization

```rust
pub struct FederatedOrgAccess {
    org_id: String,
    federation_agreement: AgreementId,
    
    capabilities: Vec<Capability>,  // Negotiated per agreement
    
    quota: ResourceQuota,
    sla: Option<ServiceLevelAgreement>,
    
    // Data sharing policy
    data_policy: DataSharingPolicy {
        results_public: bool,
        methods_public: bool,
        infrastructure_private: bool,
    },
}
```

---

## Capability System

### Core Capabilities

```rust
pub enum Capability {
    // Information Access
    ViewPublicInfo,
    ViewEducationalInfo,
    ViewOperationalInfo,
    ViewAdministrativeInfo,
    ViewInfrastructureInfo,
    
    // Task Management
    SubmitTask,
    ViewOwnTasks,
    ViewAllStudentTasks,
    ViewAllTasks,
    CancelOwnTasks,
    CancelAnyTask,
    
    // User Management
    ManageCourseUsers,
    ManageAllUsers,
    ManageQuotas,
    
    // System Management
    ManageNodes,
    ManageConfiguration,
    RestartServices,
    AccessSystemLogs,
    
    // Data Access
    ExportData,
    AccessRawLogs,
    AccessSecurityLogs,
    
    // Admin
    All,  // Wildcard (admin only)
}
```

### Capability Composition

```rust
impl Capability {
    /// Check if this capability implies another
    pub fn implies(&self, other: &Capability) -> bool {
        match (self, other) {
            (Capability::All, _) => true,
            (Capability::ViewInfrastructureInfo, Capability::ViewAdministrativeInfo) => true,
            (Capability::ViewAdministrativeInfo, Capability::ViewOperationalInfo) => true,
            (Capability::ViewOperationalInfo, Capability::ViewEducationalInfo) => true,
            (Capability::ViewEducationalInfo, Capability::ViewPublicInfo) => true,
            _ => self == other,
        }
    }
}
```

**Information hierarchy:**
```
Infrastructure ⊃ Administrative ⊃ Operational ⊃ Educational ⊃ Public
```

If you have `ViewInfrastructureInfo`, you implicitly have all lower layers.

---

## Authentication Modes

### Mode 1: Standalone (No Security Provider)

**JWT-based tokens:**
```rust
pub struct StandaloneToken {
    pub subject: String,  // user_id
    pub role: Role,
    pub capabilities: Vec<Capability>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: Ed25519Signature,  // Signed by Songbird
}
```

**Advantages:**
- ✅ No external dependencies
- ✅ Works immediately
- ✅ Simple to implement
- ✅ Fast verification

**Limitations:**
- ⚠️ Token theft = full compromise (until expiry)
- ⚠️ No hardware binding
- ⚠️ No genetic identity
- ⚠️ Revocation requires distributed state

---

### Mode 2: Security Provider-Enhanced

**Genetic identity + capability delegation:**
```rust
pub struct SecurityProviderToken {
    pub genetic_identity: GeneticIdentity,  // Security Provider genetic encryption
    pub capability_proof: ZkProof,          // Zero-knowledge capability proof
    pub hardware_binding: Option<DeviceId>, // Bound to specific device
    pub delegation_chain: Vec<DelegationStep>,
    pub signature: SecurityProviderSignature,
}

pub struct GeneticIdentity {
    pub identity_hash: Hash,  // Genetic encryption - can't be stolen
    pub entropy_level: EntropyLevel,
    pub verifiable: bool,
}
```

**Advantages:**
- 🔐 Identity can't be stolen (genetic encryption)
- 🔐 Hardware-bound (SoloKey, Titan M2, etc.)
- 🔐 Capability delegation with proofs
- 🔐 Instant revocation (genetic identity invalidation)
- 🔐 Audit trail baked in

**Enhanced Security:**
- Even if token is intercepted, genetic identity can't be forged
- Hardware key required for sensitive operations
- Zero-knowledge proofs for capability checks
- Full delegation chain for audit

---

## Implementation Architecture

### Access Control Layer

```rust
pub struct AccessControl {
    /// Token validator
    token_validator: TokenValidator,
    
    /// Role → Capability mapping
    role_capabilities: RoleCapabilityMap,
    
    /// Information layer builder
    info_builder: InformationLayerBuilder,
    
    /// Audit logger
    audit_log: AuditLog,
    
    /// Optional Security Provider integration
    security_provider: Option<SecurityProviderClient>,
}

impl AccessControl {
    pub async fn check_access(
        &self,
        token: &AccessToken,
        capability: &Capability,
    ) -> Result<bool> {
        // 1. Validate token
        let identity = self.token_validator.validate(token).await?;
        
        // 2. Get role capabilities
        let caps = self.role_capabilities.get(&identity.role)?;
        
        // 3. Check capability
        let has_capability = caps.iter().any(|c| c.implies(capability));
        
        // 4. If Security Provider available, verify genetic identity
        if let Some(security_provider) = &self.security_provider {
            if !security_provider.verify_genetic_identity(&identity).await? {
                return Ok(false);  // Genetic identity invalid
            }
        }
        
        // 5. Log access attempt
        self.audit_log.log(AccessAttempt {
            identity: identity.id.clone(),
            capability: capability.clone(),
            granted: has_capability,
            timestamp: Utc::now(),
        }).await?;
        
        Ok(has_capability)
    }
    
    pub async fn get_visible_info(
        &self,
        token: &AccessToken,
        task: &Task,
    ) -> Result<TaskInfo> {
        let identity = self.token_validator.validate(token).await?;
        
        // Build information layers based on capabilities
        let mut info = TaskInfo::new(task.id.clone());
        
        // Always add public layer
        info.add_layer(self.info_builder.build_public(task));
        
        // Add educational layer if capability present
        if self.check_access(token, &Capability::ViewEducationalInfo).await? {
            info.add_layer(self.info_builder.build_educational(task));
        }
        
        // Add operational layer if capability present
        if self.check_access(token, &Capability::ViewOperationalInfo).await? {
            info.add_layer(self.info_builder.build_operational(task));
        }
        
        // Add administrative layer if capability present
        if self.check_access(token, &Capability::ViewAdministrativeInfo).await? {
            info.add_layer(self.info_builder.build_administrative(task));
        }
        
        // Add infrastructure layer if capability present AND 2FA verified
        if self.check_access(token, &Capability::ViewInfrastructureInfo).await? {
            if self.verify_2fa(token).await? {
                info.add_layer(self.info_builder.build_infrastructure(task));
            }
        }
        
        Ok(info)
    }
}
```

---

## Configuration

```toml
[access_control]
# Authentication mode
mode = "standalone"  # or "security_provider-enhanced"

# Default access level
default_visibility = "public"

# Require 2FA for admin
require_2fa_for_admin = true

# Require hardware key for infrastructure access
require_hardware_key_for_infrastructure = true

[access_control.roles.student]
capabilities = [
    "view_public_info",
    "view_educational_info",
    "submit_task",
    "view_own_tasks",
    "cancel_own_tasks",
]
max_concurrent_tasks = 3
max_gpu_hours_per_week = 10

[access_control.roles.ta]
capabilities = [
    "view_public_info",
    "view_educational_info",
    "view_operational_info",
    "submit_task",
    "view_all_student_tasks",
    "access_student_logs",
]
max_concurrent_tasks = 10

[access_control.roles.professor]
capabilities = [
    "view_public_info",
    "view_educational_info",
    "view_operational_info",
    "view_administrative_info",
    "submit_task",
    "view_all_tasks",
    "manage_course_users",
    "manage_quotas",
    "export_data",
]
max_concurrent_tasks = 50

[access_control.roles.admin]
capabilities = ["all"]
require_hardware_key = true
require_2fa = true
require_vpn_for_infrastructure = true

[access_control.remote_admin]
session_timeout_minutes = 30
require_reauth_for_elevation = true
audit_all_actions = true

[information_disclosure]
# What gets anonymized
anonymize_ips = true
anonymize_node_names = false  # Show "Eastgate", hide "192.0.2.10"
show_sharding_to_students = true
show_topology_to_students = "anonymized"

[security_provider_integration]
# Optional Security Provider integration
enabled = false  # Set true when Security Provider available
genetic_identity_verification = true
hardware_key_binding = true
zero_knowledge_proofs = true
```

---

## Evolution Path

### Phase 1: Standalone (Q1 2025)
- ✅ Basic role-based access
- ✅ JWT tokens
- ✅ Information layer separation
- ✅ Audit logging

### Phase 2: Enhanced Standalone (Q1 2025)
- Hardware key support (SoloKey)
- 2FA for admin
- VPN requirement for infrastructure access
- Capability composition

### Phase 3: Security Provider Integration (Q2 2025)
- Genetic identity verification
- Hardware-bound tokens
- Zero-knowledge capability proofs
- Instant revocation

### Phase 4: Federation (Q2-Q3 2025)
- Cross-organization capability delegation
- Federated identity
- Distributed audit logs
- Multi-party authorization

---

## Security Guarantees

### Standalone Mode

**Guarantees:**
- ✅ Default-deny access control
- ✅ Audit trail of all access
- ✅ Information layer separation
- ✅ Capability-based authorization
- ✅ Token expiry enforcement

**Limitations:**
- ⚠️ Token theft = compromise until expiry
- ⚠️ No hardware binding (unless SoloKey added)
- ⚠️ Revocation requires distributed state

### Security Provider-Enhanced Mode

**Additional Guarantees:**
- 🔐 Identity theft impossible (genetic encryption)
- 🔐 Hardware-bound operations
- 🔐 Instant revocation (genetic identity)
- 🔐 Cryptographic audit trail
- 🔐 Zero-knowledge proofs

**Security Model:**
- Even if token stolen, can't forge genetic identity
- Even if device stolen, can't use without hardware key
- Even if both stolen, remote wipe invalidates genetic identity

---

## API Examples

### Get Task Info (Student)

```http
GET /api/tasks/task-abc123
Authorization: Bearer <student_token>
```

Response:
```json
{
  "task_id": "task-abc123",
  "status": "completed",
  "completion_time_sec": 187.3,
  "result_summary": {
    "success": true,
    "accuracy": 0.9512
  },
  "educational_info": {
    "sharding_strategy": "data_parallel",
    "shards": [
      {
        "shard_id": "shard-0",
        "node": "compute-node-alpha",
        "samples": 30000
      },
      {
        "shard_id": "shard-1",
        "node": "compute-node-beta",
        "samples": 30000
      }
    ]
  }
}
```

### Get Task Info (Admin)

```http
GET /api/tasks/task-abc123
Authorization: Bearer <admin_token>
X-Hardware-Key-Challenge: <solokey_response>
```

Response includes full infrastructure layer:
```json
{
  "task_id": "task-abc123",
  "status": "completed",
  "infrastructure_info": {
    "nodes": [
      {
        "name": "Eastgate",
        "internal_ip": "192.0.2.10:8000",
        "gpu": "RTX 3090",
        "utilization": 0.73
      },
      {
        "name": "Strandgate",
        "internal_ip": "192.0.2.134:8081",
        "gpu": "RTX 3070",
        "utilization": 0.89
      }
    ],
    "system_logs": "...",
    "configuration": {...}
  }
}
```

---

## References

- Security Provider Specification: `../security_provider/specs/GENETIC_ENCRYPTION.md` (when available)
- RhizoCrypt Specification: `../rhizocrypt/specs/DAG_CRYPTOGRAPHY.md`
- Federation Protocol: `SONGBIRD_FEDERATION.md`

---

**Status:** Design Complete  
**Next Steps:** Implementation Q1 2025, Security Provider integration Q2 2025

