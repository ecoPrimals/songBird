# Multi-Path Transport Architecture - Deep Debt Evolution

## 🐛 Problem Identified (Dec 20, 2025)

**Current Behavior:**
- Eastgate has 2 network interfaces:
  - Ethernet: 192.168.1.144
  - WiFi: 192.168.1.185
- Songbird broadcasts discovery from BOTH interfaces
- Other towers see TWO separate nodes
- But it's the SAME physical tower!

**Impact:**
- Duplicate node entries in federation
- Wastes coordination resources
- Confuses federation topology
- Prevents path multiplexing benefits

## 🎯 Vision: Multi-Path Transport Layer

### Concept

**One Node, Multiple Paths:**
```
Tower: eastgate (single logical node)
  ├─ Path 1: Ethernet (192.168.1.144)
  ├─ Path 2: WiFi (192.168.1.185)
  ├─ Path 3: Bluetooth (future)
  ├─ Path 4: LoRa/Radio (future)
  └─ Path 5: Smoke signals (if desperate 😄)
```

**Similar to Protocol Multiplexing:**
```
Node: eastgate
  └─ Transport Paths: [Ethernet, WiFi, ...]
     ├─ Protocol 1: HTTPS
     ├─ Protocol 2: tarpc
     └─ Protocol 3: WebSocket
```

### Benefits

1. **Redundancy**
   - If Ethernet fails, use WiFi
   - Automatic failover
   - No manual intervention

2. **Performance**
   - Aggregate bandwidth (Ethernet + WiFi)
   - Load balancing across paths
   - Choose fastest path per request

3. **Capability-Based**
   - Discover available interfaces automatically
   - Use what's available
   - Degrade gracefully

4. **Sovereignty**
   - Self-manage network topology
   - No external configuration
   - Adapt to environment

## 📐 Architecture Design

### 1. Network Interface Discovery

```rust
pub struct NetworkInterface {
    pub interface_name: String,      // "eth0", "wlp0s20f3"
    pub interface_type: InterfaceType, // Ethernet, WiFi, Bluetooth, etc.
    pub ip_addresses: Vec<IpAddr>,
    pub capabilities: InterfaceCapabilities,
    pub metrics: InterfaceMetrics,
}

pub enum InterfaceType {
    Ethernet,
    WiFi,
    Bluetooth,
    LoRa,
    Radio,
    // Future: Pollen, Smoke Signals, Carrier Pigeons 🕊️
}

pub struct InterfaceCapabilities {
    pub bandwidth: Option<u64>,
    pub latency: Option<Duration>,
    pub reliability: f64,  // 0.0 - 1.0
    pub cost: f64,         // Energy/monetary cost
}

pub struct NetworkInterfaceManager {
    interfaces: Arc<RwLock<Vec<NetworkInterface>>>,
}

impl NetworkInterfaceManager {
    /// Discover all available network interfaces
    pub async fn discover_interfaces() -> Result<Vec<NetworkInterface>> {
        // Use netlink, if_addrs, or similar to enumerate
        // Detect capabilities per interface
        // Measure metrics (bandwidth, latency)
    }
    
    /// Monitor interfaces for changes
    pub async fn monitor_interfaces(&self) {
        // Watch for interface up/down
        // Update capabilities on change
        // Notify transport layer
    }
}
```

### 2. Node Identity Layer

```rust
pub struct NodeIdentity {
    pub node_id: Uuid,                 // SINGLE stable ID (not session ID!)
    pub node_name: String,             // "eastgate"
    pub transport_paths: Vec<TransportPath>,
    pub protocols: Vec<Protocol>,
}

pub struct TransportPath {
    pub interface: NetworkInterface,
    pub endpoint: SocketAddr,
    pub status: PathStatus,
    pub metrics: PathMetrics,
}

pub enum PathStatus {
    Active,
    Standby,
    Degraded,
    Failed,
}

pub struct PathMetrics {
    pub latency: Duration,
    pub bandwidth: u64,
    pub packet_loss: f64,
    pub last_check: DateTime<Utc>,
}
```

### 3. Discovery Protocol Evolution

**Current (v2.1):**
```json
{
  "version": "2.1",
  "session_id": "rotating-anonymous-id",
  "capabilities": ["orchestration", "federation"],
  "protocols": ["https"],
  "port": 8080
}
```

**Evolved (v3.0 - Multi-Path):**
```json
{
  "version": "3.0",
  "node_id": "stable-node-uuid",
  "node_name": "eastgate",
  "capabilities": ["orchestration", "federation"],
  "transport_paths": [
    {
      "interface_type": "ethernet",
      "endpoint": "192.168.1.144:8080",
      "protocols": ["https", "tarpc"],
      "metrics": {
        "latency_ms": 1,
        "bandwidth_mbps": 1000,
        "reliability": 0.99
      }
    },
    {
      "interface_type": "wifi",
      "endpoint": "192.168.1.185:8080",
      "protocols": ["https", "tarpc"],
      "metrics": {
        "latency_ms": 5,
        "bandwidth_mbps": 300,
        "reliability": 0.95
      }
    }
  ],
  "preferred_path": "ethernet"
}
```

### 4. Path Selection & Multiplexing

```rust
pub struct PathSelector {
    policy: PathSelectionPolicy,
}

pub enum PathSelectionPolicy {
    FastestPath,           // Choose lowest latency
    HighestBandwidth,      // Choose highest throughput
    MostReliable,          // Choose best packet loss
    LoadBalance,           // Distribute across paths
    Aggregate,             // Use all paths simultaneously
    CostEfficient,         // Minimize energy/cost
}

impl PathSelector {
    /// Select best path(s) for a request
    pub fn select_path(
        &self,
        available_paths: &[TransportPath],
        request_type: RequestType,
    ) -> Vec<TransportPath> {
        match self.policy {
            PathSelectionPolicy::FastestPath => {
                // Choose single path with lowest latency
            }
            PathSelectionPolicy::Aggregate => {
                // Use all active paths
                // Split request across paths
                // Reassemble responses
            }
            // ... other policies
        }
    }
}

pub enum RequestType {
    SmallLatencySensitive,  // Use fastest path
    LargeBulkTransfer,      // Use aggregate bandwidth
    Critical,               // Use most reliable path
    BestEffort,            // Use whatever's available
}
```

### 5. Multi-Path Transport

```rust
pub struct MultiPathTransport {
    node_identity: NodeIdentity,
    path_selector: PathSelector,
    active_connections: HashMap<TransportPath, Connection>,
}

impl MultiPathTransport {
    /// Send request using selected path(s)
    pub async fn send<T>(&self, request: T, policy: PathSelectionPolicy) -> Result<Response> {
        let paths = self.path_selector.select_path(&self.node_identity.transport_paths, policy);
        
        match paths.len() {
            1 => {
                // Single path - simple send
                self.send_via_path(&paths[0], request).await
            }
            _ => {
                // Multiple paths - multiplex
                self.send_multipath(paths, request).await
            }
        }
    }
    
    /// Send via multiple paths simultaneously
    async fn send_multipath<T>(&self, paths: Vec<TransportPath>, request: T) -> Result<Response> {
        // Split request into chunks
        // Send chunks via different paths
        // Reassemble response
        // Use FEC (Forward Error Correction) for reliability
    }
    
    /// Automatic failover if path fails
    pub async fn send_with_failover<T>(&self, request: T) -> Result<Response> {
        let paths = self.node_identity.transport_paths.clone();
        
        for path in paths {
            match self.send_via_path(&path, request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Path {} failed: {}, trying next", path.interface.interface_name, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("All paths failed"))
    }
}
```

## 🔧 Implementation Plan

### Phase 1: Network Interface Discovery
- [ ] Enumerate network interfaces (netlink/if_addrs)
- [ ] Detect interface types (Ethernet, WiFi, etc.)
- [ ] Measure interface metrics (latency, bandwidth)
- [ ] Monitor interface changes (up/down events)

### Phase 2: Node Identity Coalescence
- [ ] Generate stable node ID (not session ID)
- [ ] Aggregate interfaces under single node
- [ ] Update discovery protocol to v3.0
- [ ] Broadcast all paths in single message

### Phase 3: Path Selection
- [ ] Implement path selection policies
- [ ] Add request type classification
- [ ] Choose optimal path per request
- [ ] Load balance across paths

### Phase 4: Multi-Path Transport
- [ ] Implement path multiplexing
- [ ] Add automatic failover
- [ ] Aggregate bandwidth across paths
- [ ] Add Forward Error Correction (FEC)

### Phase 5: Testing & Optimization
- [ ] Test failover (disable Ethernet, verify WiFi takeover)
- [ ] Test aggregation (measure combined bandwidth)
- [ ] Test path selection (verify policies work)
- [ ] Benchmark performance

## 📊 Expected Impact

### Before (Current)
```
Eastgate:
  - Appears as 2 nodes (Ethernet + WiFi)
  - No path redundancy
  - No bandwidth aggregation
  - Manual failover only

Federation:
  - 21 nodes (including duplicates)
  - Confused topology
  - Wasted resources
```

### After (Multi-Path)
```
Eastgate:
  - Appears as 1 node (unified identity)
  - 2 transport paths (Ethernet + WiFi)
  - Automatic failover
  - Optional bandwidth aggregation

Federation:
  - ~12 nodes (no duplicates)
  - Clear topology
  - Efficient resource use
  - Robust connectivity
```

## 🎓 Analogies & Examples

### Similar to MPTCP (Multipath TCP)
- Industry standard for multi-path transport
- Used by Apple for Siri (WiFi + Cellular)
- We're building similar concept for Songbird

### Similar to SCTP (Stream Control Transmission Protocol)
- Multi-homing support
- Path failover
- Used in telecom systems

### Similar to QUIC
- Connection migration
- Path validation
- Used by HTTP/3

### Our Twist: Capability-Based
- Not just TCP/IP paths
- ANY transport medium (Bluetooth, LoRa, etc.)
- Discover and use what's available
- Sovereignty through self-management

## 🚀 Quick Win: Identity Coalescence

**Immediate Fix (without full multi-path):**

```rust
// In discovery broadcaster:
pub struct NodeAnnouncement {
    pub node_id: Uuid,  // STABLE ID (from machine-id or generated once)
    pub node_name: String,
    pub endpoints: Vec<Endpoint>,  // All interfaces
}

pub struct Endpoint {
    pub interface_type: String,  // "ethernet", "wifi"
    pub address: SocketAddr,
    pub protocols: Vec<String>,
}

// When receiving discovery:
pub fn process_announcement(announcement: NodeAnnouncement) {
    // Group by node_id (not by endpoint!)
    if let Some(existing_node) = federation.get_node(&announcement.node_id) {
        // Update endpoints for existing node
        existing_node.endpoints.extend(announcement.endpoints);
    } else {
        // Register new node with all endpoints
        federation.register_node(announcement);
    }
}
```

This simple change would:
- ✅ Recognize eastgate as 1 node (not 2)
- ✅ Track both Ethernet + WiFi endpoints
- ✅ Enable basic failover (try each endpoint)
- ✅ Reduce duplicate nodes immediately

## 📚 References & Inspiration

- **MPTCP**: RFC 8684
- **SCTP**: RFC 4960
- **QUIC**: RFC 9000
- **BMC3/RDMA**: Multi-rail networking
- **Cellular/WiFi Handoff**: Seamless connectivity

## 💡 Future Extensions

### Transport Diversity
- **Bluetooth**: Local mesh, low power
- **LoRa**: Long range, IoT
- **Radio**: Ham radio, emergency comms
- **Acoustic**: Underwater, sonar
- **Optical**: Free-space laser, line-of-sight
- **Pollen**: Bio-inspired, organic networks 🌸
- **Smoke Signals**: Ultimate fallback 🔥💨

### Exotic Protocols
- **Delay Tolerant Networking (DTN)**: Mars missions
- **Store and Forward**: Intermittent connectivity
- **Opportunistic Networks**: Mobile ad-hoc
- **Named Data Networking (NDN)**: Content-centric

### Intelligence
- **ML-based Path Selection**: Learn optimal paths
- **Predictive Failover**: Fail before failure
- **Adaptive Multiplexing**: Adjust to conditions

## 🏆 Philosophy Alignment

**Sovereignty:**
- Self-discover network capabilities
- Self-manage path selection
- No external configuration

**Capability-Based:**
- Use what's available
- Discover at runtime
- Degrade gracefully

**Fail-Safe:**
- Automatic failover
- Redundant paths
- Never lose connectivity

**User-Centric:**
- Transparent operation
- No manual configuration
- "It just works"

---

**Priority:** P1 (High - improves federation accuracy)  
**Complexity:** Medium (network enumeration + identity tracking)  
**Impact:** High (enables multi-path, reduces duplicates)  
**Philosophy:** ✅ Sovereignty + Capability-Based + Fail-Safe  

**Next Steps:**
1. Implement stable node ID generation
2. Aggregate network interfaces in discovery
3. Update federation to recognize node identity
4. Test with Ethernet + WiFi on eastgate

---

**Status:** Documented (Dec 20, 2025)  
**Discovered By:** User observation (multiple IPs = same tower)  
**Related:** Session TTL cleanup, Discovery verification  
**Grade:** A+ observation - deep architectural insight! 🏆

