# DNS and mDNS Discovery Implementation Plan
## November 20, 2025

## 🎯 REQUIREMENTS

**Critical**: DNS and mDNS discovery are **REQUIRED** for Songbird to be live over internet with RPC fully useable.

---

## 📋 CURRENT STATUS

### What Exists (TODOs/Stubs)
```rust
// crates/songbird-discovery/src/discovery_comprehensive_tests.rs
// TODO: DNS-based discovery implementation
// TODO: mDNS discovery implementation
// TODO: etcd integration implementation
```

### What's Missing
1. **DNS-SD (DNS Service Discovery)** - RFC 6763
2. **mDNS (Multicast DNS)** - RFC 6762
3. **Integration with existing discovery system**

---

## 🏗️ ARCHITECTURE DESIGN

### DNS-SD Implementation

**Purpose**: Service discovery over standard DNS  
**Use Case**: Internet-wide discovery, production deployments  

```rust
/// DNS-based service discovery (RFC 6763)
pub struct DnsDiscovery {
    /// DNS resolver
    resolver: Arc<TrustDnsResolver>,
    
    /// Search domains
    search_domains: Vec<String>,
    
    /// Service type (e.g., "_songbird._tcp")
    service_type: String,
    
    /// Cache for discovered services
    cache: Arc<RwLock<HashMap<String, DiscoveredService>>>,
    
    /// Cache TTL
    cache_ttl: Duration,
}

impl DnsDiscovery {
    /// Discover services via DNS-SD
    pub async fn discover_services(&self) -> SongbirdResult<Vec<Service>> {
        // 1. Query PTR records for service enumeration
        // 2. Query SRV records for host/port
        // 3. Query TXT records for metadata
        // 4. Cache results with TTL
    }
    
    /// Register service in DNS (if DNS server supports dynamic updates)
    pub async fn register_service(&self, service: &Service) -> SongbirdResult<()> {
        // 1. Create SRV record
        // 2. Create TXT record with metadata
        // 3. Update DNS via RFC 2136 (Dynamic Updates)
    }
}
```

### mDNS Implementation

**Purpose**: Zero-configuration local network discovery  
**Use Case**: LAN/local networks, development, edge deployments  

```rust
/// mDNS-based service discovery (RFC 6762)
pub struct MdnsDiscovery {
    /// mDNS responder
    responder: Arc<MdnsResponder>,
    
    /// Service instance name
    instance_name: String,
    
    /// Service type (e.g., "_songbird._tcp.local")
    service_type: String,
    
    /// Port
    port: u16,
    
    /// TXT records (metadata)
    txt_records: HashMap<String, String>,
    
    /// Discovered services
    discovered: Arc<RwLock<HashMap<String, DiscoveredService>>>,
}

impl MdnsDiscovery {
    /// Start mDNS responder and browser
    pub async fn start(&mut self) -> SongbirdResult<()> {
        // 1. Start mDNS responder on port 5353
        // 2. Announce service
        // 3. Start browser for service type
        // 4. Handle queries and responses
    }
    
    /// Discover services on local network
    pub async fn discover_services(&self) -> SongbirdResult<Vec<Service>> {
        // 1. Send mDNS query
        // 2. Collect responses
        // 3. Parse SRV, TXT, A/AAAA records
    }
    
    /// Announce service on local network
    pub async fn announce_service(&self) -> SongbirdResult<()> {
        // 1. Send mDNS announcement
        // 2. Respond to queries
        // 3. Update on changes
    }
}
```

---

## 📦 DEPENDENCIES

### DNS-SD Dependencies
```toml
[dependencies]
# DNS resolution
trust-dns-resolver = "0.23"
trust-dns-proto = "0.23"

# For dynamic DNS updates (optional)
trust-dns-client = "0.23"
```

### mDNS Dependencies
```toml
[dependencies]
# mDNS implementation
mdns-sd = "0.10"  # Pure Rust mDNS library

# Alternative (more mature):
# libmdns = "0.7"  # Bindings to Avahi/Bonjour
```

---

## 🔧 IMPLEMENTATION PHASES

### Phase 1: DNS-SD Implementation (3-4 hours)

#### Step 1: Create DNS Discovery Module
**File**: `crates/songbird-discovery/src/dns_discovery.rs`

```rust
use trust_dns_resolver::{
    config::*,
    TokioAsyncResolver,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// DNS-based service discovery
pub struct DnsDiscovery {
    resolver: Arc<TokioAsyncResolver>,
    search_domains: Vec<String>,
    service_type: String,
    cache: Arc<RwLock<HashMap<String, CachedService>>>,
    cache_ttl: Duration,
}

#[derive(Clone, Debug)]
struct CachedService {
    service: Service,
    discovered_at: std::time::Instant,
}

impl DnsDiscovery {
    pub async fn new(service_type: String, search_domains: Vec<String>) -> SongbirdResult<Self> {
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        )?;
        
        Ok(Self {
            resolver: Arc::new(resolver),
            search_domains,
            service_type,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        })
    }
    
    pub async fn discover_services(&self) -> SongbirdResult<Vec<Service>> {
        let mut services = Vec::new();
        
        for domain in &self.search_domains {
            let query = format!("{}..{}", self.service_type, domain);
            
            // Query PTR records
            if let Ok(response) = self.resolver.lookup(&query, RecordType::PTR).await {
                for record in response.iter() {
                    if let Some(ptr) = record.as_ptr() {
                        // Query SRV record
                        if let Ok(srv_response) = self.resolver
                            .lookup(ptr.0.to_string(), RecordType::SRV)
                            .await 
                        {
                            // Parse SRV, query TXT, build Service
                            services.extend(self.parse_srv_records(srv_response).await?);
                        }
                    }
                }
            }
        }
        
        // Update cache
        self.update_cache(&services).await;
        
        Ok(services)
    }
}
```

#### Step 2: Integrate with Discovery System
**File**: `crates/songbird-discovery/src/discovery/core.rs`

```rust
pub enum DiscoveryBackend {
    Static(StaticDiscovery),
    Dns(DnsDiscovery),
    Mdns(MdnsDiscovery),
    Etcd(EtcdDiscovery),
    Composite(Vec<Box<dyn DiscoveryBackend>>),
}
```

### Phase 2: mDNS Implementation (3-4 hours)

#### Step 1: Create mDNS Discovery Module
**File**: `crates/songbird-discovery/src/mdns_discovery.rs`

```rust
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// mDNS-based service discovery
pub struct MdnsDiscovery {
    daemon: ServiceDaemon,
    instance_name: String,
    service_type: String,
    port: u16,
    txt_records: HashMap<String, String>,
    discovered: Arc<RwLock<HashMap<String, DiscoveredService>>>,
}

impl MdnsDiscovery {
    pub fn new(
        instance_name: String,
        service_type: String,
        port: u16,
    ) -> SongbirdResult<Self> {
        let daemon = ServiceDaemon::new()?;
        
        Ok(Self {
            daemon,
            instance_name,
            service_type,
            port,
            txt_records: HashMap::new(),
            discovered: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn start(&self) -> SongbirdResult<()> {
        // Register our service
        let service_info = ServiceInfo::new(
            &self.service_type,
            &self.instance_name,
            &format!("{}.local.", self.instance_name),
            "",  // IPv4 address (auto-detected)
            self.port,
            self.txt_records.clone(),
        )?;
        
        self.daemon.register(service_info)?;
        
        // Start browsing for services
        let receiver = self.daemon.browse(&self.service_type)?;
        
        // Spawn task to handle discoveries
        let discovered = Arc::clone(&self.discovered);
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Add to discovered services
                        let mut disc = discovered.write().await;
                        disc.insert(info.get_fullname().to_string(), info.into());
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        // Remove from discovered services
                        let mut disc = discovered.write().await;
                        disc.remove(&fullname);
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn discover_services(&self) -> SongbirdResult<Vec<Service>> {
        let discovered = self.discovered.read().await;
        Ok(discovered.values().map(|d| d.to_service()).collect())
    }
}
```

### Phase 3: Integration & Testing (2-3 hours)

#### Tests to Implement
```rust
#[tokio::test]
async fn test_dns_discovery_query() {
    let discovery = DnsDiscovery::new(
        "_songbird._tcp".to_string(),
        vec!["example.com".to_string()],
    ).await.unwrap();
    
    let services = discovery.discover_services().await.unwrap();
    assert!(!services.is_empty());
}

#[tokio::test]
async fn test_mdns_announcement_and_discovery() {
    let mut mdns1 = MdnsDiscovery::new(
        "songbird-1".to_string(),
        "_songbird._tcp".to_string(),
        8080,
    ).unwrap();
    
    mdns1.start().await.unwrap();
    
    let mut mdns2 = MdnsDiscovery::new(
        "songbird-2".to_string(),
        "_songbird._tcp".to_string(),
        8081,
    ).unwrap();
    
    mdns2.start().await.unwrap();
    
    // Wait for discovery
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let services = mdns1.discover_services().await.unwrap();
    assert!(services.iter().any(|s| s.name == "songbird-2"));
}
```

---

## 🚀 IMPLEMENTATION TIMELINE

| Phase | Task | Hours | Priority |
|-------|------|-------|----------|
| 1 | DNS-SD Core | 2h | P0 |
| 1 | DNS-SD Integration | 1h | P0 |
| 1 | DNS-SD Tests | 1h | P0 |
| 2 | mDNS Core | 2h | P0 |
| 2 | mDNS Integration | 1h | P0 |
| 2 | mDNS Tests | 1h | P0 |
| 3 | E2E Tests | 1h | P1 |
| 3 | Documentation | 1h | P1 |
| **TOTAL** | | **10h** | |

---

## 📝 ACCEPTANCE CRITERIA

### DNS-SD
- ✅ Can discover services via DNS PTR/SRV/TXT records
- ✅ Supports multiple search domains
- ✅ Caches results with TTL
- ✅ Integrates with existing discovery system
- ✅ Has comprehensive tests

### mDNS
- ✅ Can announce services on local network
- ✅ Can discover services on local network
- ✅ Responds to mDNS queries
- ✅ Auto-detects network interfaces
- ✅ Works without configuration (zero-conf)
- ✅ Has comprehensive tests

### Integration
- ✅ Works with existing UniversalDiscovery
- ✅ Can use multiple backends simultaneously
- ✅ Handles backend failures gracefully
- ✅ Supports priority/fallback mechanisms

---

## 🔗 RELATED FILES

### To Modify
- `crates/songbird-discovery/Cargo.toml` - Add dependencies
- `crates/songbird-discovery/src/lib.rs` - Export new modules
- `crates/songbird-discovery/src/discovery/core.rs` - Add backends
- `crates/songbird-discovery/src/discovery_comprehensive_tests.rs` - Remove TODOs

### To Create
- `crates/songbird-discovery/src/dns_discovery.rs` - New
- `crates/songbird-discovery/src/mdns_discovery.rs` - New
- `crates/songbird-discovery/tests/dns_discovery_tests.rs` - New
- `crates/songbird-discovery/tests/mdns_discovery_tests.rs` - New

---

## 🎯 NEXT STEPS

1. ✅ Add dependencies to Cargo.toml
2. ⏳ Implement DNS-SD module
3. ⏳ Implement mDNS module
4. ⏳ Integrate with discovery system
5. ⏳ Write comprehensive tests
6. ⏳ Update documentation
7. ⏳ Remove TODO comments

**Ready to proceed with implementation!**

---

**Document Version**: 1.0  
**Last Updated**: November 20, 2025  
**Status**: Ready for implementation

