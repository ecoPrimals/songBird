# 🔒 Secure Federation Design - Zero Trust with Progressive Escalation

**Date:** December 19, 2025  
**Status:** Implementation Plan  
**Principle:** Secure by default, anonymous first, escalate with trust

---

## 🎯 Design Principles

### 1. Songbird Handles Complexity ✨
- **Developers:** Just start Songbird, it works
- **Auto-discovery:** Enabled by default
- **TLS:** Enabled by default (failsafe)
- **Port selection:** Automatic (no hardcoding)
- **Certificate generation:** Automatic (self-signed for LAN)

### 2. Secure Anonymous Discovery 🔒
```
Phase 1: ANONYMOUS Discovery
↓ Broadcast: "I'm a Songbird tower" (no identity)
↓ Response: "I'm a Songbird tower" (no identity)
↓ Establish: Anonymous TLS connection
↓ Exchange: Capabilities only (what I can do, not who I am)

Phase 2: CAPABILITY-BASED Trust
↓ Verify: Cryptographic proof of capabilities
↓ Establish: Role-based anonymous identity
↓ Grant: Access based on capabilities, not identity

Phase 3: IDENTITY Verification (Optional)
↓ Request: "Prove you are tower X"
↓ Verify: BearDog genetic identity / Hardware key
↓ Escalate: Full trust, internal IPs visible
↓ Grant: Admin-level access
```

### 3. Progressive Trust Escalation 📈
- **Level 0:** Anonymous (default) - Can discover, no data shared
- **Level 1:** Capability-verified - Can coordinate tasks
- **Level 2:** Role-verified - Can access service registry
- **Level 3:** Identity-verified - Can see infrastructure details
- **Level 4:** Hardware-verified - Full admin access (BearDog)

### 4. Developer Override 🛠️
```bash
# Default (automatic everything)
./songbird-orchestrator

# Override if needed
./songbird-orchestrator \
  --node-id custom-name \
  --port 8443 \
  --tls-cert /path/to/cert.pem \
  --discovery-port 2300 \
  --federation-endpoint https://specific-tower:8443
```

---

## 📊 Current vs Desired State

### CURRENT (Broken)
```
eastgate:  HTTP, hardcoded port 8000, no TLS ❌
strandgate: HTTP, hardcoded port 8090, no TLS ❌
westgate:  HTTPS, auto-port, TLS ✅ (CORRECT!)

Problem: Protocol mismatch, insecure, hardcoded
```

### DESIRED (Secure by Default)
```
eastgate:  HTTPS, auto-port, TLS, anonymous discovery ✅
strandgate: HTTPS, auto-port, TLS, anonymous discovery ✅
westgate:  HTTPS, auto-port, TLS, anonymous discovery ✅

Result: Auto-discover, secure connections, zero-trust
```

---

## 🔐 Secure Anonymous Discovery Protocol

### Step 1: Broadcast Discovery (Anonymous)
```json
// UDP broadcast on port 2300
{
  "type": "songbird_discovery",
  "version": "2.0",
  "timestamp": 1734654321,
  "capabilities": [
    "orchestration",
    "gpu-compute",
    "storage"
  ],
  "protocols": ["https", "tarpc-tls", "websocket-tls"],
  "anonymous_id": "a3f7b9c2d8e1...",  // Temporary session ID
  "public_key": "..."  // For secure channel establishment
  // NO: hostname, IP, identity, internal details
}
```

**What's Shared:** Capabilities, protocols, temporary ID  
**What's Hidden:** Identity, hostname, internal IPs, topology

---

### Step 2: TLS Handshake (Anonymous)
```
Tower A → Tower B: "I found you via discovery"
Tower B → Tower A: "Establish TLS connection"
↓
TLS handshake with self-signed certs
↓
Encrypted channel established (anonymous)
↓
Certificate NOT verified yet (zero trust)
```

**Result:** Encrypted channel, both parties anonymous

---

### Step 3: Capability Exchange (Verified)
```json
// Over TLS, exchange capabilities with proof
{
  "session_id": "a3f7b9c2d8e1...",
  "capabilities": {
    "gpu-compute": {
      "proof": "...",  // Cryptographic proof
      "resources": {
        "gpu_count": 2,
        "memory_gb": 32,
        "compute_capability": "8.6"
      }
    }
  },
  "trust_level": "anonymous",
  "accept_tasks": true,
  "share_topology": false
}
```

**What's Verified:** Capabilities with cryptographic proof  
**What's Granted:** Task coordination, no identity exposure

---

### Step 4: Progressive Escalation (On Demand)

**Scenario A: Student Task (Anonymous OK)**
```
Student submits ML task
↓
Coordinator: "I need GPU compute"
↓
Anonymous GPU tower: "I have GPUs" (capability-verified)
↓
Coordinator: Assigns task (no identity needed)
↓
GPU tower: Executes, returns results
↓
Trust Level: 1 (Capability-verified)
```

**Scenario B: Admin Operation (Identity Required)**
```
Admin requests infrastructure details
↓
Coordinator: "Prove your identity"
↓
Admin: Provides BearDog hardware key
↓
Coordinator: Verifies hardware key + genetic identity
↓
Grants: Full access, internal IPs visible
↓
Trust Level: 4 (Hardware-verified)
```

---

## 🚀 Implementation Plan

### Phase 1: Default Secure Configuration ✅

**File:** `crates/songbird-orchestrator/src/main.rs`

```rust
// Default configuration (secure by default)
#[derive(Parser, Debug)]
#[command(name = "songbird-orchestrator")]
struct Args {
    /// Node ID (auto-detected from hostname if not provided)
    #[arg(long, env = "SONGBIRD_NODE_ID")]
    node_id: Option<String>,
    
    /// Enable TLS (default: true, failsafe)
    #[arg(long, env = "SONGBIRD_TLS_ENABLED", default_value = "true")]
    tls_enabled: bool,
    
    /// Enable auto-discovery (default: true)
    #[arg(long, env = "SONGBIRD_ENABLE_DISCOVERY", default_value = "true")]
    enable_discovery: bool,
    
    /// Enable federation (default: true)
    #[arg(long, env = "SONGBIRD_ENABLE_FEDERATION", default_value = "true")]
    enable_federation: bool,
    
    /// Port (auto-selected if not provided)
    #[arg(long, env = "SONGBIRD_PORT")]
    port: Option<u16>,
    
    /// Anonymous discovery (default: true, secure)
    #[arg(long, env = "SONGBIRD_ANONYMOUS_DISCOVERY", default_value = "true")]
    anonymous_discovery: bool,
    
    /// Trust escalation enabled (default: true)
    #[arg(long, env = "SONGBIRD_TRUST_ESCALATION", default_value = "true")]
    trust_escalation: bool,
}

// Startup message
info!("🎵 Songbird Orchestrator");
info!("   TLS: {} (failsafe default)", if tls_enabled { "Enabled" } else { "Disabled" });
info!("   Auto-discovery: {} (secure anonymous)", if enable_discovery { "Enabled" } else { "Disabled" });
info!("   Federation: {}", if enable_federation { "Enabled" } else { "Disabled" });
info!("   Trust model: Progressive escalation (anonymous → capability → identity)");
info!("   🔒 SECURE BY DEFAULT - Zero trust with progressive escalation");
```

---

### Phase 2: Anonymous Discovery Protocol

**File:** `crates/songbird-discovery/src/anonymous_discovery.rs` (NEW)

```rust
/// Anonymous discovery message
#[derive(Serialize, Deserialize)]
pub struct AnonymousDiscoveryMessage {
    /// Protocol version
    version: String,
    
    /// Temporary session ID (rotates every hour)
    session_id: String,
    
    /// Capabilities (what I can do)
    capabilities: Vec<Capability>,
    
    /// Supported protocols
    protocols: Vec<Protocol>,
    
    /// Public key for secure channel
    public_key: PublicKey,
    
    /// Trust level accepted (anonymous, capability, identity)
    max_trust_level: TrustLevel,
    
    // NO identity fields:
    // - NO hostname
    // - NO IP address
    // - NO node_id
    // - NO internal topology
}

/// Trust levels for progressive escalation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// No trust, anonymous only
    Anonymous = 0,
    
    /// Capability-verified (can coordinate tasks)
    CapabilityVerified = 1,
    
    /// Role-verified (can access registry)
    RoleVerified = 2,
    
    /// Identity-verified (can see infrastructure)
    IdentityVerified = 3,
    
    /// Hardware-verified (full admin access, BearDog)
    HardwareVerified = 4,
}

impl AnonymousDiscoveryMessage {
    /// Create anonymous discovery message
    pub fn new(capabilities: Vec<Capability>) -> Self {
        Self {
            version: "2.0".to_string(),
            session_id: Self::generate_session_id(),
            capabilities,
            protocols: vec![
                Protocol::HttpsTls,
                Protocol::TarpcTls,
                Protocol::WebSocketTls,
            ],
            public_key: Self::generate_ephemeral_keypair(),
            max_trust_level: TrustLevel::Anonymous,
        }
    }
    
    /// Generate rotating session ID (prevents tracking)
    fn generate_session_id() -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_le_bytes());
        hasher.update(uuid::Uuid::new_v4().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

---

### Phase 3: Trust Escalation Engine

**File:** `crates/songbird-orchestrator/src/trust/escalation.rs` (NEW)

```rust
/// Trust escalation manager
pub struct TrustEscalationManager {
    /// Current trust relationships
    trust_store: Arc<RwLock<HashMap<SessionId, TrustRelationship>>>,
    
    /// BearDog integration for hardware verification
    beardog_client: Option<Arc<BearDogClient>>,
}

/// Trust relationship between towers
pub struct TrustRelationship {
    /// Remote session ID (anonymous)
    session_id: SessionId,
    
    /// Current trust level
    trust_level: TrustLevel,
    
    /// Verified capabilities
    verified_capabilities: Vec<Capability>,
    
    /// Identity (only if escalated to IdentityVerified)
    identity: Option<TowerIdentity>,
    
    /// Hardware attestation (only if escalated to HardwareVerified)
    hardware_proof: Option<HardwareAttestation>,
    
    /// Established timestamp
    established_at: SystemTime,
    
    /// Last verified timestamp
    last_verified_at: SystemTime,
}

impl TrustEscalationManager {
    /// Establish initial anonymous trust
    pub async fn establish_anonymous(&self, session_id: SessionId, capabilities: Vec<Capability>) -> Result<()> {
        let relationship = TrustRelationship {
            session_id: session_id.clone(),
            trust_level: TrustLevel::Anonymous,
            verified_capabilities: Vec::new(),
            identity: None,
            hardware_proof: None,
            established_at: SystemTime::now(),
            last_verified_at: SystemTime::now(),
        };
        
        self.trust_store.write().await.insert(session_id, relationship);
        
        info!("✅ Anonymous trust established (Level 0)");
        Ok(())
    }
    
    /// Escalate to capability-verified
    pub async fn verify_capabilities(&self, session_id: &SessionId, proof: CapabilityProof) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship = store.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        // Verify cryptographic proof of capabilities
        if proof.verify()? {
            relationship.trust_level = TrustLevel::CapabilityVerified;
            relationship.verified_capabilities = proof.capabilities;
            relationship.last_verified_at = SystemTime::now();
            
            info!("✅ Trust escalated to Capability-Verified (Level 1)");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Capability proof verification failed"))
        }
    }
    
    /// Escalate to identity-verified
    pub async fn verify_identity(&self, session_id: &SessionId, identity_proof: IdentityProof) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship = store.get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        // Verify identity proof (JWT, certificate, etc.)
        if identity_proof.verify()? {
            relationship.trust_level = TrustLevel::IdentityVerified;
            relationship.identity = Some(identity_proof.identity);
            relationship.last_verified_at = SystemTime::now();
            
            info!("✅ Trust escalated to Identity-Verified (Level 3)");
            info!("   Identity: {}", identity_proof.identity.node_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Identity proof verification failed"))
        }
    }
    
    /// Escalate to hardware-verified (requires BearDog)
    pub async fn verify_hardware(&self, session_id: &SessionId, hardware_proof: HardwareAttestation) -> Result<()> {
        let beardog = self.beardog_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("BearDog integration not configured"))?;
        
        // Verify hardware key via BearDog
        if beardog.verify_hardware_key(&hardware_proof.hardware_key).await? {
            let mut store = self.trust_store.write().await;
            let relationship = store.get_mut(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            
            relationship.trust_level = TrustLevel::HardwareVerified;
            relationship.hardware_proof = Some(hardware_proof);
            relationship.last_verified_at = SystemTime::now();
            
            info!("🔒 Trust escalated to Hardware-Verified (Level 4 - ADMIN)");
            info!("   Hardware Key: {}", hardware_proof.hardware_key);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Hardware attestation failed"))
        }
    }
    
    /// Check if operation is allowed at current trust level
    pub async fn check_permission(&self, session_id: &SessionId, required_level: TrustLevel) -> Result<bool> {
        let store = self.trust_store.read().await;
        let relationship = store.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        
        Ok(relationship.trust_level >= required_level)
    }
}
```

---

### Phase 4: Graduated Information Disclosure

**File:** `crates/songbird-orchestrator/src/access_control/graduated_disclosure.rs`

```rust
/// Information disclosure based on trust level
pub struct GraduatedDisclosure {
    trust_manager: Arc<TrustEscalationManager>,
}

impl GraduatedDisclosure {
    /// Get tower information based on trust level
    pub async fn get_tower_info(&self, session_id: &SessionId, tower_id: &str) -> Result<TowerInfo> {
        let trust_level = self.trust_manager.get_trust_level(session_id).await?;
        
        match trust_level {
            TrustLevel::Anonymous => {
                // Share only capabilities, no identity
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    // NO: identity, hostname, IP, topology
                    ..Default::default()
                })
            }
            
            TrustLevel::CapabilityVerified => {
                // Share capabilities + role
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    // NO: hostname, IP, topology
                    ..Default::default()
                })
            }
            
            TrustLevel::RoleVerified => {
                // Share capabilities + role + service registry
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    // NO: hostname, IP (still anonymous)
                    ..Default::default()
                })
            }
            
            TrustLevel::IdentityVerified => {
                // Share capabilities + identity + hostname
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    identity: Some(self.get_identity(tower_id).await?),
                    hostname: Some(self.get_hostname(tower_id).await?),
                    // NO: internal IP (not yet)
                    ..Default::default()
                })
            }
            
            TrustLevel::HardwareVerified => {
                // Share EVERYTHING (full admin)
                Ok(TowerInfo {
                    capabilities: self.get_capabilities(tower_id).await?,
                    protocols: self.get_protocols(tower_id).await?,
                    role: Some(self.get_role(tower_id).await?),
                    services: Some(self.get_services(tower_id).await?),
                    identity: Some(self.get_identity(tower_id).await?),
                    hostname: Some(self.get_hostname(tower_id).await?),
                    internal_ip: Some(self.get_internal_ip(tower_id).await?),
                    topology: Some(self.get_topology(tower_id).await?),
                    config: Some(self.get_config(tower_id).await?),
                    // FULL ACCESS
                })
            }
        }
    }
}
```

---

## 📦 Deployment Configuration

### Default Configuration (Secure by Default)

**File:** `config/production-secure.toml`

```toml
[server]
# Auto-detect node ID from hostname
# node_id = "auto"  # Commented out = auto-detect

# TLS enabled by default (failsafe)
tls_enabled = true

# Auto-select port (finds available port)
# port = auto  # Commented out = auto-select

# Discovery enabled by default
enable_discovery = true

# Federation enabled by default
enable_federation = true

[discovery]
# Anonymous discovery (secure by default)
anonymous = true

# Discovery port (UDP broadcast)
port = 2300

# Broadcast addresses (local network)
broadcast_addresses = ["255.255.255.255:2300", "192.168.1.255:2300"]

[federation]
# Trust escalation enabled
trust_escalation = true

# Initial trust level (anonymous)
initial_trust_level = "anonymous"

# Allow capability escalation
allow_capability_escalation = true

# Allow identity escalation
allow_identity_escalation = true

# Require hardware key for admin
require_hardware_for_admin = true

[tls]
# Auto-generate certificates if not found
auto_generate = true

# Certificate paths
cert_path = "certs/songbird.crt"
key_path = "certs/songbird.key"

# Auto-detect SANs (hostname + local IP)
auto_sans = true

# Additional SANs (optional)
# sans = ["localhost", "127.0.0.1"]

[trust]
# Progressive escalation timeouts
anonymous_timeout = "1h"      # Anonymous sessions expire after 1 hour
capability_timeout = "24h"    # Capability sessions expire after 24 hours
identity_timeout = "7d"       # Identity sessions expire after 7 days
hardware_timeout = "never"    # Hardware sessions never expire
```

---

### Developer Override (If Needed)

```bash
# Minimal (all defaults)
./songbird-orchestrator

# Override specific settings
./songbird-orchestrator \
  --node-id my-custom-name \
  --port 9000 \
  --tls-cert /custom/cert.pem

# Disable TLS (NOT RECOMMENDED, local dev only)
SONGBIRD_TLS_ENABLED=false ./songbird-orchestrator

# Use specific federation endpoint
SONGBIRD_FEDERATION_ENDPOINTS="https://specific-tower:8443" \
  ./songbird-orchestrator
```

---

## 🚀 Deployment to Strandgate

### Automated Deployment Script

**File:** `deploy_strandgate_secure.sh`

```bash
#!/bin/bash
# Deploy Songbird to Strandgate with Secure Defaults

STRANDGATE_HOST="strandgate.local"
STRANDGATE_USER="strandgate"
SONGBIRD_DIR="/home/strandgate/Development/ecoPrimals/songbird"

echo "🚀 Deploying Songbird to Strandgate (Secure by Default)"
echo "========================================================"

# 1. Stop old processes on strandgate
echo "1️⃣  Stopping old processes on strandgate..."
ssh $STRANDGATE_USER@$STRANDGATE_HOST "pkill -f 'songbird-orchestrator|tarpc-server'" || true

# 2. Sync code to strandgate
echo "2️⃣  Syncing code to strandgate..."
rsync -avz --exclude 'target' --exclude '.git' \
  ./ $STRANDGATE_USER@$STRANDGATE_HOST:$SONGBIRD_DIR/

# 3. Build on strandgate
echo "3️⃣  Building on strandgate..."
ssh $STRANDGATE_USER@$STRANDGATE_HOST "cd $SONGBIRD_DIR && cargo build --release"

# 4. Deploy with secure defaults
echo "4️⃣  Starting Songbird with secure defaults..."
ssh $STRANDGATE_USER@$STRANDGATE_HOST "cd $SONGBIRD_DIR && \
  mkdir -p certs && \
  export SONGBIRD_NODE_ID=strandgate && \
  export SONGBIRD_TLS_ENABLED=true && \
  export SONGBIRD_ENABLE_DISCOVERY=true && \
  export SONGBIRD_ENABLE_FEDERATION=true && \
  export SONGBIRD_ANONYMOUS_DISCOVERY=true && \
  nohup ./target/release/songbird-orchestrator > /tmp/strandgate-secure.log 2>&1 &"

# 5. Verify
echo "5️⃣  Verifying deployment..."
sleep 3
ssh $STRANDGATE_USER@$STRANDGATE_HOST "curl -k https://localhost:8443/health" || \
  echo "⚠️  May need more time to start, check logs on strandgate"

echo ""
echo "✅ Deployment complete!"
echo "   Logs: ssh $STRANDGATE_USER@$STRANDGATE_HOST 'tail -f /tmp/strandgate-secure.log'"
echo "   Health: curl -k https://strandgate.local:8443/health"
```

---

## ✅ Success Criteria

### After Deployment, All Towers Should:

1. **Auto-discover each other** via anonymous UDP broadcast
2. **Establish TLS** connections automatically
3. **Share capabilities** anonymously (no identity)
4. **Coordinate tasks** at capability-verified trust level
5. **Escalate trust** only when needed (admin operations)
6. **Zero hardcoded** ports or endpoints
7. **Fully encrypted** all connections (TLS)
8. **Sovereign** - Each tower maintains autonomy

### Verification Commands:

```bash
# On any tower
curl -k https://localhost:8443/health

# Should see:
{
  "status": "healthy",
  "tls": true,
  "discovery": "anonymous",
  "trust_model": "progressive-escalation",
  "discovered_towers": 3,
  "trust_levels": {
    "anonymous": 3,
    "capability_verified": 0,
    "identity_verified": 0,
    "hardware_verified": 0
  }
}
```

---

**Status:** 🚧 **DESIGN COMPLETE - READY FOR IMPLEMENTATION**  
**Principle:** Secure by default, anonymous first, escalate with trust  
**Result:** Zero-trust federation with progressive escalation

**🔒 Songbird handles complexity, developers just start it, security happens automatically!** ✨

