# 🌐 Multi-Federation Evolution - December 19, 2025

**Vision:** Enable participation in multiple federations simultaneously with different trust boundaries, capabilities, and resource allocations.

---

## 🎯 Real-World Use Cases

### Scenario: Alice's Multi-Federation Life

**Family Federation:**
- Trust: High (hardware-verified)
- Capabilities: Full (storage, compute, media streaming)
- Resources: 50% allocation
- Nodes: Family members' devices

**School Federation:**
- Trust: Medium (identity-verified)
- Capabilities: Limited (compute for assignments, collaborative editing)
- Resources: 30% allocation
- Nodes: Classmates, professors
- Restrictions: No personal data access

**Work Federation:**
- Trust: Capability-verified
- Capabilities: Work-specific (code review, CI/CD, documentation)
- Resources: 20% allocation
- Nodes: Coworkers, contractors
- Restrictions: Strict data isolation, audit logging

**Overlap:**
- Some family members are also in school federation (siblings)
- Some work colleagues are friends (social overlap)
- But boundaries and permissions differ per context

---

## 🏗️ Architecture Evolution

### Current: Single Federation

```rust
pub struct FederationState {
    federation_id: Uuid,
    nodes: HashMap<NodeId, NodeRegistration>,
    // Single, monolithic federation
}
```

**Limitations:**
- Can only join one federation
- All nodes have same trust/capabilities
- No context-aware resource allocation

### Evolved: Multi-Federation with Contexts

```rust
pub struct MultiFederationState {
    /// All federations this node participates in
    federations: Arc<RwLock<HashMap<FederationId, FederationContext>>>,
    
    /// Node's identity across federations
    node_identity: NodeIdentity,
    
    /// Global resource manager (allocates across federations)
    resource_manager: Arc<FederationResourceManager>,
    
    /// Discovery → Federation routing
    discovery_router: Arc<DiscoveryRouter>,
}

pub struct FederationContext {
    /// Unique federation identifier
    federation_id: FederationId,
    
    /// Human-readable name
    federation_name: String,
    
    /// Nodes in this federation
    nodes: Arc<RwLock<HashMap<NodeId, NodeRegistration>>>,
    
    /// Trust policy for this federation
    trust_policy: TrustPolicy,
    
    /// What capabilities we expose in this federation
    exposed_capabilities: Vec<String>,
    
    /// Resource limits for this federation
    resource_quota: ResourceQuota,
    
    /// Data isolation rules
    data_policy: DataPolicy,
    
    /// Auto-join rules (discovery → this federation)
    auto_join_policy: AutoJoinPolicy,
}

pub struct FederationId(Uuid);

pub struct NodeIdentity {
    /// Core node ID (same across federations)
    node_id: Uuid,
    
    /// Per-federation personas (different names/capabilities)
    personas: HashMap<FederationId, Persona>,
}

pub struct Persona {
    /// Display name in this federation
    name: String,
    
    /// Capabilities advertised in this federation
    capabilities: Vec<String>,
    
    /// Trust level required from peers
    required_trust: TrustLevel,
}
```

---

## 🔄 Discovery → Federation Bridge (Multi-Context)

### Smart Routing Logic

```rust
pub struct DiscoveryRouter {
    /// Rules for routing discovered peers to federations
    routing_rules: Vec<RoutingRule>,
    
    /// Fallback federation (if no rules match)
    default_federation: Option<FederationId>,
}

pub struct RoutingRule {
    /// Match criteria
    matcher: RoutingMatcher,
    
    /// Target federation
    target_federation: FederationId,
    
    /// Priority (higher = checked first)
    priority: u32,
}

pub enum RoutingMatcher {
    /// Match by IP subnet (e.g., family = 192.168.1.0/24)
    IpSubnet(IpNetwork),
    
    /// Match by capabilities (e.g., has "ml-training" → work)
    HasCapability(String),
    
    /// Match by session ID pattern (if we pre-share)
    SessionPattern(Regex),
    
    /// Match by time (e.g., 9am-5pm = work, else = family)
    TimeRange(TimeRange),
    
    /// Match by DNS/hostname resolution
    HostnamePattern(Regex),
    
    /// Custom predicate
    Custom(Box<dyn Fn(&DiscoveredPeer) -> bool + Send + Sync>),
}

impl DiscoveryRouter {
    /// Route a discovered peer to appropriate federation(s)
    pub async fn route(&self, peer: &DiscoveredPeer) -> Vec<FederationId> {
        let mut matches = Vec::new();
        
        // Check all rules by priority
        for rule in self.routing_rules.iter().sorted_by_key(|r| r.priority).rev() {
            if rule.matcher.matches(peer) {
                matches.push(rule.target_federation.clone());
            }
        }
        
        // If no matches, use default
        if matches.is_empty() {
            if let Some(default) = &self.default_federation {
                matches.push(default.clone());
            }
        }
        
        matches
    }
}
```

---

## 📊 Resource Management

### Federation Resource Quotas

```rust
pub struct ResourceQuota {
    /// Max CPU cores allocated to this federation
    max_cpu_cores: Option<u32>,
    
    /// Max memory (GB) allocated to this federation
    max_memory_gb: Option<u32>,
    
    /// Max storage (GB) for this federation's data
    max_storage_gb: Option<u32>,
    
    /// Max concurrent tasks from this federation
    max_concurrent_tasks: Option<u32>,
    
    /// Priority (0-100, higher = more resources under contention)
    priority: u8,
}

pub struct FederationResourceManager {
    /// Total available resources
    total_resources: ComputeResources,
    
    /// Resources allocated per federation
    allocations: Arc<RwLock<HashMap<FederationId, ResourceAllocation>>>,
}

impl FederationResourceManager {
    /// Allocate resources for a federation
    pub async fn allocate(
        &self,
        federation_id: &FederationId,
        requested: &ResourceQuota,
    ) -> Result<ResourceAllocation> {
        let mut allocations = self.allocations.write().await;
        
        // Check if request fits within available resources
        let available = self.calculate_available(&allocations).await;
        
        let allocation = ResourceAllocation {
            cpu_cores: min(requested.max_cpu_cores, available.cpu_cores),
            memory_gb: min(requested.max_memory_gb, available.memory_gb),
            storage_gb: min(requested.max_storage_gb, available.storage_gb),
            granted_at: Utc::now(),
        };
        
        allocations.insert(federation_id.clone(), allocation.clone());
        Ok(allocation)
    }
    
    /// Rebalance resources when new federation joins or resources change
    pub async fn rebalance(&self) {
        // Implement priority-based resource redistribution
        // High-priority federations get resources first
        // Low-priority federations share remainder
    }
}
```

---

## 🔒 Per-Federation Data Isolation

### Data Policy

```rust
pub struct DataPolicy {
    /// What data can be accessed by this federation's nodes
    accessible_paths: Vec<PathBuf>,
    
    /// Data that must be kept isolated
    isolated_paths: Vec<PathBuf>,
    
    /// Encryption requirements
    encryption_required: bool,
    
    /// Audit logging enabled
    audit_logging: bool,
    
    /// Data retention policy
    retention_policy: RetentionPolicy,
}

pub enum RetentionPolicy {
    /// Keep data forever
    Permanent,
    
    /// Delete after duration
    TimeLimited(Duration),
    
    /// Delete when federation dissolved
    SessionOnly,
}

impl FederationContext {
    /// Check if a file path is accessible by this federation
    pub fn can_access_path(&self, path: &Path) -> bool {
        // Check if path is in accessible_paths
        // AND not in isolated_paths
        self.data_policy.accessible_paths.iter().any(|p| path.starts_with(p))
            && !self.data_policy.isolated_paths.iter().any(|p| path.starts_with(p))
    }
}
```

---

## 🤝 Auto-Join Policies

### Graduated Auto-Join

```rust
pub struct AutoJoinPolicy {
    /// Enable auto-join for this federation
    enabled: bool,
    
    /// Minimum trust level required for auto-join
    min_trust_level: TrustLevel,
    
    /// Required capabilities (peer must have ALL)
    required_capabilities: Vec<String>,
    
    /// Forbidden capabilities (peer must have NONE)
    forbidden_capabilities: Vec<String>,
    
    /// Max nodes in this federation (capacity limit)
    max_nodes: Option<usize>,
    
    /// Allowlist (only these IPs can auto-join)
    ip_allowlist: Option<Vec<IpNetwork>>,
    
    /// Denylist (these IPs cannot auto-join)
    ip_denylist: Vec<IpNetwork>,
    
    /// Require manual approval
    require_approval: bool,
}

impl AutoJoinPolicy {
    /// Check if a discovered peer should auto-join
    pub fn should_auto_join(&self, peer: &DiscoveredPeer, current_nodes: usize) -> bool {
        if !self.enabled {
            return false;
        }
        
        // Check capacity
        if let Some(max) = self.max_nodes {
            if current_nodes >= max {
                return false;
            }
        }
        
        // Check IP allowlist/denylist
        if let Some(ref allowlist) = self.ip_allowlist {
            if !allowlist.iter().any(|net| net.contains(&peer.address.ip())) {
                return false;
            }
        }
        
        if self.ip_denylist.iter().any(|net| net.contains(&peer.address.ip())) {
            return false;
        }
        
        // Check capabilities
        if !self.required_capabilities.iter().all(|cap| peer.capabilities.contains(cap)) {
            return false;
        }
        
        if self.forbidden_capabilities.iter().any(|cap| peer.capabilities.contains(cap)) {
            return false;
        }
        
        // If require approval, don't auto-join
        if self.require_approval {
            return false;
        }
        
        true
    }
}
```

---

## 🎨 Configuration Example

### Multi-Federation Config

```toml
# Family Federation
[[federations]]
name = "family"
federation_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
enabled = true

[federations.trust_policy]
default_level = "hardware_verified"
allow_anonymous = false

[federations.capabilities]
expose = ["storage", "media_streaming", "backup", "home_automation"]
hide = ["work_docs", "school_assignments"]

[federations.resources]
max_cpu_cores = 12  # 50% of 24 cores
max_memory_gb = 16   # 50% of 32 GB
max_storage_gb = 900  # 50% of 1800 GB
priority = 90  # High priority

[federations.data_policy]
accessible_paths = ["/home/user/family", "/media/photos", "/media/videos"]
isolated_paths = ["/home/user/work", "/home/user/school"]
encryption_required = false
audit_logging = false
retention_policy = "permanent"

[federations.auto_join]
enabled = true
min_trust_level = "identity_verified"
ip_allowlist = ["192.168.1.0/24"]  # Home network only
require_approval = false

# School Federation
[[federations]]
name = "school"
federation_id = "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy"
enabled = true

[federations.trust_policy]
default_level = "identity_verified"
allow_anonymous = false

[federations.capabilities]
expose = ["compute", "collaborative_editing", "storage"]
hide = ["family_photos", "work_docs", "personal_finance"]

[federations.resources]
max_cpu_cores = 7  # 30% of 24 cores
max_memory_gb = 10  # 30% of 32 GB
max_storage_gb = 540  # 30% of 1800 GB
priority = 70  # Medium-high priority

[federations.data_policy]
accessible_paths = ["/home/user/school", "/home/user/documents/shared"]
isolated_paths = ["/home/user/family", "/home/user/work"]
encryption_required = true
audit_logging = true
retention_policy = { time_limited = "90d" }  # Delete after semester

[federations.auto_join]
enabled = true
min_trust_level = "identity_verified"
required_capabilities = ["academic"]
ip_allowlist = ["10.0.0.0/8"]  # University network
require_approval = false

# Work Federation
[[federations]]
name = "work"
federation_id = "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz"
enabled = true

[federations.trust_policy]
default_level = "capability_verified"
allow_anonymous = false

[federations.capabilities]
expose = ["code_review", "ci_cd", "documentation"]
hide = ["family_photos", "school_assignments", "personal_docs"]

[federations.resources]
max_cpu_cores = 5  # 20% of 24 cores
max_memory_gb = 6   # 20% of 32 GB
max_storage_gb = 360  # 20% of 1800 GB
priority = 50  # Medium priority

[federations.data_policy]
accessible_paths = ["/home/user/work"]
isolated_paths = ["/home/user/family", "/home/user/school", "/home/user/personal"]
encryption_required = true
audit_logging = true
retention_policy = { time_limited = "365d" }  # 1 year retention

[federations.auto_join]
enabled = true
min_trust_level = "capability_verified"
required_capabilities = ["professional"]
forbidden_capabilities = ["personal", "social"]
ip_allowlist = ["172.16.0.0/12"]  # Corporate network
require_approval = true  # Always require approval for work

# Discovery Routing Rules
[[discovery.routing_rules]]
priority = 100
matcher = { ip_subnet = "192.168.1.0/24" }
target_federation = "family"

[[discovery.routing_rules]]
priority = 90
matcher = { ip_subnet = "10.0.0.0/8" }
target_federation = "school"

[[discovery.routing_rules]]
priority = 80
matcher = { ip_subnet = "172.16.0.0/12" }
target_federation = "work"

[[discovery.routing_rules]]
priority = 50
matcher = { has_capability = "academic" }
target_federation = "school"

[[discovery.routing_rules]]
priority = 40
matcher = { has_capability = "professional" }
target_federation = "work"

# Default federation (if no rules match)
[discovery]
default_federation = "family"
```

---

## 🔄 Discovery → Federation Bridge Implementation

### Enhanced Bridge with Multi-Federation

```rust
// In crates/songbird-orchestrator/src/app/mod.rs

/// Start the discovery → federation bridge with multi-federation support
async fn start_discovery_federation_bridge(&self) -> Result<()> {
    if let Some(ref listener) = self.discovery_listener {
        let listener_clone = Arc::clone(listener);
        let multi_federation_state = Arc::clone(&self.multi_federation_state);
        let discovery_router = Arc::clone(&self.discovery_router);
        let trust_manager = Arc::clone(&self.trust_manager);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            info!("🌉 Discovery → Federation bridge started (multi-federation)");
            
            loop {
                interval.tick().await;
                
                // Get all discovered peers
                let peers = listener_clone.get_peers().await;
                
                for peer in peers {
                    // Route peer to appropriate federation(s)
                    let target_federations = discovery_router.route(&peer).await;
                    
                    for federation_id in target_federations {
                        // Get federation context
                        let federation = match multi_federation_state
                            .get_federation(&federation_id)
                            .await
                        {
                            Some(f) => f,
                            None => {
                                warn!("Federation {} not found, skipping peer", federation_id);
                                continue;
                            }
                        };
                        
                        // Check auto-join policy
                        let current_nodes = federation.nodes.read().await.len();
                        if !federation.auto_join_policy.should_auto_join(&peer, current_nodes) {
                            debug!(
                                "Peer {} does not meet auto-join policy for federation {}",
                                peer.session_id, federation.federation_name
                            );
                            continue;
                        }
                        
                        // Establish anonymous trust first
                        let trust_session = match trust_manager
                            .establish_anonymous_trust(&peer.session_id, &peer.capabilities)
                            .await
                        {
                            Ok(session) => session,
                            Err(e) => {
                                warn!("Failed to establish trust with {}: {}", peer.session_id, e);
                                continue;
                            }
                        };
                        
                        // Check if trust level meets minimum
                        if trust_session.trust_level < federation.trust_policy.min_trust_level {
                            debug!(
                                "Peer {} trust level {:?} below minimum {:?} for federation {}",
                                peer.session_id,
                                trust_session.trust_level,
                                federation.trust_policy.min_trust_level,
                                federation.federation_name
                            );
                            continue;
                        }
                        
                        // Get HTTPS endpoint
                        let endpoint = peer.https_endpoint();
                        
                        // Check if already joined
                        if federation.nodes.read().await.contains_key(&peer.session_id) {
                            continue;
                        }
                        
                        // Try to join federation
                        match federation.try_join(&peer, &endpoint).await {
                            Ok(_) => {
                                info!(
                                    "✅ Auto-joined peer {} to federation '{}' ({})",
                                    peer.session_id, federation.federation_name, endpoint
                                );
                                
                                // Log to audit trail if required
                                if federation.data_policy.audit_logging {
                                    audit_log::record_federation_join(
                                        &federation_id,
                                        &peer.session_id,
                                        &endpoint,
                                        &trust_session.trust_level,
                                    ).await;
                                }
                            }
                            Err(e) => {
                                debug!("Failed to auto-join {}: {}", endpoint, e);
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    } else {
        Ok(())
    }
}
```

---

## 🎯 Benefits

### 1. Context-Aware Participation
- Different capabilities per federation
- Graduated disclosure based on context
- Privacy by default

### 2. Resource Fairness
- Prevent single federation from monopolizing resources
- Priority-based allocation
- Dynamic rebalancing

### 3. Security & Privacy
- Data isolation per federation
- Per-federation encryption policies
- Audit trails for sensitive federations

### 4. Flexibility
- Join/leave federations independently
- Different trust levels per federation
- Overlap allowed (same node in multiple)

### 5. Real-World Alignment
- Matches how humans actually organize
- Family, school, work boundaries respected
- Social overlap handled gracefully

---

## 🚀 Implementation Plan

### Phase 1: Core Multi-Federation (Tonight)
1. Create `MultiFederationState` structure
2. Implement `FederationContext` with policies
3. Add basic routing rules (IP subnet)
4. Bridge discovery → federation with routing

### Phase 2: Resource Management (Tomorrow)
1. Implement `FederationResourceManager`
2. Add resource quotas and priorities
3. Dynamic rebalancing

### Phase 3: Advanced Policies (This Week)
1. Time-based routing
2. Capability-based routing
3. Custom predicates
4. Approval workflows

### Phase 4: UI & Monitoring (Next Week)
1. Federation management UI
2. Resource usage dashboard
3. Trust level visualization
4. Audit trail viewer

---

**This evolution transforms Songbird from a single-federation system to a context-aware, multi-federation platform that respects real-world social boundaries!** 🌐

