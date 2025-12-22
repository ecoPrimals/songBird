# Primal Responsibility Separation Specification

**Version**: 1.0.0  
**Date**: December 21, 2025  
**Status**: 🟡 **IN PROGRESS** - Infrastructure Ready (BTSP)  
**Scope**: Inter-Primal Security Architecture

---

## 🎯 Executive Summary

This specification defines the clear separation of responsibilities between **Songbird** (orchestration) and **BearDog** (security) for internet deployment, roaming devices, and federated trust.

### Core Principle

> **"Each Primal Has Its Focus"**  
> Songbird orchestrates. BearDog secures. They collaborate via BTSP at runtime.

---

## 🏗️ Architectural Boundaries

### 🎵 Songbird - Orchestration Primal

**Core Question**: *"Who does what and where?"*

#### Responsibilities

**✅ Songbird SHOULD Handle:**
- **Service Discovery** - Finding nodes and capabilities (UDP broadcast, rendezvous, mDNS)
- **Task Routing** - Directing workloads to capable nodes
- **Port Management** - Universal Port Authority (dynamic allocation)
- **Federation Coordination** - Maintaining node registry and heartbeats
- **Capability Matching** - "What can do X?"
- **Load Distribution** - Balancing work across nodes
- **Connection Management** - Establishing, migrating, tracking connections
- **Protocol Negotiation** - HTTP, HTTPS, tarpc, JSON-RPC, WebSocket, BTSP
- **NAT Traversal Coordination** - Orchestrating STUN/TURN/ICE workflows
- **Rendezvous Coordination** - Managing discovery for internet deployment

**❌ Songbird SHOULD NOT Handle:**
- Certificate generation/management → BearDog
- Trust decisions → BearDog
- Encryption/decryption → BearDog
- Key management → BearDog
- Authentication logic → BearDog
- Genetic cryptography → BearDog

#### Current State

**Already Implemented:**
```rust
// Songbird has BTSP interface ready
use songbird_network_federation::btsp::{
    BtspProviderFactory,
    BtspConfig,
    BtspProvider,
};

// Local implementation for testing (AES-256-GCM)
let provider = LocalBtspProvider::new();

// Runtime discovery of BearDog (not hardcoded)
let factory = BtspProviderFactory::new(config);
let provider = factory.create_provider().await?; // Discovers BearDog or falls back
```

**Location**: `crates/songbird-network-federation/src/btsp/`

---

### 🐻 BearDog - Security Primal

**Core Question**: *"Can I trust this? Is this secure?"*

#### Responsibilities

**✅ BearDog SHOULD Handle:**
- **Certificate Management** - Issue, verify, rotate, revoke
- **Mutual TLS (mTLS)** - Both sides verify each other
- **Trust Verification** - TOFU (Trust On First Use), pinning, chains
- **Encryption/Decryption** - AES-256-GCM, genetic cryptography
- **Key Generation & Storage** - RSA-PSS, Ed25519, ECDSA
- **Authentication** - "Who is this node?"
- **Authorization** - "What can they do?"
- **Audit Logging** - "Who did what when?"
- **HSM Integration** - YubiKey, SoloKeys, StrongBox, Secure Enclave
- **Entropy Hierarchy** - Real human entropy (never simulated)
- **Genetic Cryptography** - Key lineage tracking, cross-party renewal

**❌ BearDog SHOULD NOT Handle:**
- Task routing → Songbird
- Service discovery → Songbird
- Port management → Songbird
- Federation topology → Songbird
- Capability matching → Songbird

#### Current State

**Already Implemented:**
- ✅ Universal HSM support
- ✅ Entropy hierarchy
- ✅ Modern cryptography (RSA-PSS, Ed25519, ECDSA)
- ✅ Capability-based discovery
- ✅ Zero hardcoding
- ✅ Production-ready (Grade A, 96/100)

**Location**: `/home/eastgate/Development/ecoPrimals/beardog/`

**BTSP Ready**: BearDog can provide secure tunnels via the BTSP interface

---

## 🔗 Collaboration Pattern: BTSP Interface

### Current Implementation (Songbird Side)

```rust
// File: crates/songbird-network-federation/src/btsp/provider.rs

#[async_trait]
pub trait BtspProvider: Send + Sync {
    /// Establish secure tunnel with peer
    async fn establish_tunnel(
        &self,
        peer: &PeerInfo,
    ) -> Result<TunnelHandle>;
    
    /// Encrypt data through tunnel
    async fn encrypt(
        &self,
        data: &[u8],
        context: &SecurityContext,
    ) -> Result<Vec<u8>>;
    
    /// Decrypt data from tunnel
    async fn decrypt(
        &self,
        data: &[u8],
        context: &SecurityContext,
    ) -> Result<Vec<u8>>;
    
    /// Check tunnel status
    async fn tunnel_status(
        &self,
        handle: &TunnelHandle,
    ) -> Result<TunnelStatus>;
    
    /// Close tunnel gracefully
    async fn close_tunnel(
        &self,
        handle: &TunnelHandle,
    ) -> Result<()>;
}

/// Factory discovers BearDog at runtime
pub struct BtspProviderFactory {
    config: BtspConfig,
}

impl BtspProviderFactory {
    /// Create provider based on runtime discovery
    pub async fn create_provider(&self) -> Result<Arc<dyn BtspProvider>> {
        // 1. Try to discover BearDog via capability system
        if let Ok(beardog) = self.discover_beardog().await {
            info!("✅ BearDog BTSP provider discovered");
            return Ok(beardog);
        }
        
        // 2. Graceful fallback to local implementation
        if self.config.local_fallback {
            warn!("⚠️ BearDog unavailable, using local BTSP");
            return Ok(Arc::new(LocalBtspProvider::new()));
        }
        
        Err(SongbirdError::configuration("BTSP not available"))
    }
}
```

### Required Implementation (BearDog Side)

**File**: `crates/beardog-tunnel/src/btsp_provider.rs` (to be created)

```rust
/// BearDog's implementation of BTSP provider
pub struct BeardogBtspProvider {
    hsm: Arc<UniversalHsm>,
    entropy: Arc<EntropyHierarchy>,
    crypto: Arc<CryptoEngine>,
}

#[async_trait]
impl BtspProvider for BeardogBtspProvider {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> Result<TunnelHandle> {
        // 1. Generate ephemeral keys using entropy hierarchy
        let keypair = self.entropy.generate_keypair().await?;
        
        // 2. Perform TOFU (Trust On First Use) or verify known peer
        let trust = self.verify_peer_trust(peer).await?;
        
        // 3. Establish mTLS connection
        let tls_conn = self.establish_mtls(peer, &keypair).await?;
        
        // 4. Create genetic crypto tunnel
        let tunnel = self.crypto.create_tunnel(tls_conn, trust).await?;
        
        Ok(TunnelHandle {
            id: tunnel.id(),
            peer_id: peer.id.clone(),
            established_at: Utc::now(),
        })
    }
    
    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>> {
        // Use genetic cryptography with key lineage
        self.crypto.encrypt_with_lineage(data, context).await
    }
    
    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>> {
        // Verify lineage and decrypt
        self.crypto.decrypt_with_lineage(data, context).await
    }
    
    async fn tunnel_status(&self, handle: &TunnelHandle) -> Result<TunnelStatus> {
        // Check tunnel health, key rotation status
        self.crypto.check_tunnel_status(handle).await
    }
    
    async fn close_tunnel(&self, handle: &TunnelHandle) -> Result<()> {
        // Securely close tunnel, zeroize keys
        self.crypto.close_tunnel_secure(handle).await
    }
}

/// Register with Songbird's UPA
pub async fn register_btsp_with_songbird() -> Result<()> {
    // Use songbird-primal-sdk for registration
    let orchestrator = discover_orchestrators().await?
        .into_iter()
        .find(|o| o.has_capability("orchestration"))
        .ok_or("No orchestrator found")?;
    
    let registration = register_with_orchestrator(
        &orchestrator,
        ServiceInfo {
            primal_name: "beardog".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                Capability {
                    name: "security".to_string(),
                    version: "1.0.0".to_string(),
                    protocols: vec!["btsp".to_string(), "https".to_string()],
                },
                Capability {
                    name: "encryption".to_string(),
                    version: "1.0.0".to_string(),
                    protocols: vec!["btsp".to_string()],
                },
                Capability {
                    name: "authentication".to_string(),
                    version: "1.0.0".to_string(),
                    protocols: vec!["https".to_string()],
                },
            ],
            metadata: Default::default(),
        },
    ).await?;
    
    info!("✅ BearDog registered with Songbird: {}", registration.id);
    Ok(())
}
```

---

## 🚀 Internet Deployment: Responsibility Split

### Phase 1: Rendezvous Server

#### Songbird Responsibilities
```rust
impl Songbird {
    /// Discover available rendezvous servers
    async fn find_rendezvous_servers(&self) -> Vec<RendezvousServer> {
        // Check env vars, well-known endpoints, DNS
        // Orchestration logic only
    }
    
    /// Register presence with rendezvous
    async fn register_with_rendezvous(
        &self,
        server: &RendezvousServer,
        security: &dyn BtspProvider, // BearDog does security
    ) -> Result<RendezvousToken> {
        // 1. Ask BearDog to secure the connection
        let secure_channel = security
            .establish_tunnel(&server.peer_info())
            .await?;
        
        // 2. Songbird does the registration
        self.send_registration(secure_channel, &self.node_info)
            .await
    }
    
    /// Coordinate peer-to-peer connection via rendezvous
    async fn connect_via_rendezvous(
        &self,
        peer_id: &NodeId,
        security: &dyn BtspProvider,
    ) -> Result<SecureConnection> {
        // 1. Songbird coordinates
        let coordination = self.negotiate_connection(peer_id).await?;
        
        // 2. BearDog secures
        let secure_conn = security
            .establish_tunnel(&coordination.endpoint)
            .await?;
        
        Ok(secure_conn)
    }
}
```

#### BearDog Responsibilities
```rust
impl BeardogBtspProvider {
    /// Secure rendezvous registration
    async fn secure_rendezvous_registration(
        &self,
        rendezvous: &RendezvousServer,
        node_id: &NodeId,
    ) -> Result<SignedRegistration> {
        // 1. Generate proof of identity
        let proof = self.hsm.sign_registration(node_id)?;
        
        // 2. Establish encrypted channel with rendezvous
        let channel = self.establish_mtls(&rendezvous.endpoint).await?;
        
        // 3. Send authenticated registration
        let registration = SignedRegistration {
            node_id: node_id.clone(),
            public_key: self.hsm.get_public_key()?,
            signature: proof,
            timestamp: Utc::now(),
        };
        
        Ok(registration)
    }
    
    /// Verify peer through rendezvous
    async fn verify_peer_via_rendezvous(
        &self,
        peer_id: &NodeId,
        rendezvous_token: &Token,
    ) -> Result<TrustLevel> {
        // Cryptographic verification
        let peer_pubkey = self.fetch_peer_pubkey(peer_id, rendezvous_token).await?;
        let trust = self.verify_pubkey_trust(peer_pubkey)?;
        
        Ok(trust)
    }
}
```

### Phase 2: NAT Traversal

#### Songbird Responsibilities
```rust
impl Songbird {
    /// Coordinate NAT traversal (orchestration)
    async fn coordinate_nat_traversal(
        &self,
        peer_id: &NodeId,
        security: &dyn BtspProvider,
    ) -> Result<Connection> {
        // 1. Discover STUN servers
        let stun_servers = self.discover_stun_servers().await?;
        
        // 2. Get public endpoint
        let public_endpoint = self.query_stun(&stun_servers).await?;
        
        // 3. Exchange candidates with peer via rendezvous
        let peer_candidates = self.exchange_ice_candidates(peer_id).await?;
        
        // 4. Try direct connection
        if let Ok(direct) = self.try_direct_connection(&peer_candidates).await {
            // Ask BearDog to secure it
            return security.establish_tunnel(&direct).await;
        }
        
        // 5. Fall back to TURN relay
        let relay = self.establish_turn_relay(&peer_candidates).await?;
        security.establish_tunnel(&relay).await
    }
}
```

#### BearDog Responsibilities
```rust
impl BeardogBtspProvider {
    /// Secure NAT traversal
    async fn secure_nat_traversal(
        &self,
        stun_server: &StunServer,
    ) -> Result<SecureNatSession> {
        // 1. Verify STUN server (don't trust blindly)
        self.verify_stun_server_trust(stun_server).await?;
        
        // 2. Secure the STUN query (prevent injection)
        let secure_query = self.encrypt_stun_query().await?;
        
        // 3. Verify STUN response signature
        let response = self.send_secure_stun_query(secure_query).await?;
        self.verify_stun_response(&response)?;
        
        Ok(SecureNatSession {
            public_endpoint: response.public_endpoint,
            verified: true,
        })
    }
    
    /// Secure TURN relay
    async fn secure_turn_relay(
        &self,
        turn_server: &TurnServer,
        peer_pubkey: &PublicKey,
    ) -> Result<SecureTurnSession> {
        // End-to-end encryption through relay
        // Even if TURN server compromised, data is encrypted
        let e2e_keys = self.negotiate_e2e_keys(peer_pubkey).await?;
        
        Ok(SecureTurnSession {
            relay: turn_server.clone(),
            encryption: e2e_keys,
        })
    }
}
```

### Phase 3: Mobile/Roaming Support

#### Songbird Responsibilities
```rust
impl Songbird {
    /// Handle connection migration (orchestration)
    async fn migrate_connection(
        &self,
        old_endpoint: &Endpoint,
        new_endpoint: &Endpoint,
        security: &dyn BtspProvider,
    ) -> Result<Connection> {
        // 1. Detect network change
        let network_change = self.detect_network_change().await?;
        
        // 2. Re-register with rendezvous
        self.update_rendezvous_registration(&new_endpoint).await?;
        
        // 3. Notify peers of migration
        self.notify_peers_of_migration(&new_endpoint).await?;
        
        // 4. Ask BearDog to re-establish secure tunnels
        for peer in self.active_peers() {
            let new_tunnel = security
                .establish_tunnel(&peer.info)
                .await?;
            self.replace_connection(peer.id, new_tunnel)?;
        }
        
        Ok(())
    }
}
```

#### BearDog Responsibilities
```rust
impl BeardogBtspProvider {
    /// Maintain trust through roaming
    async fn migrate_trust(
        &self,
        peer_id: &NodeId,
        old_tunnel: &TunnelHandle,
        new_endpoint: &Endpoint,
    ) -> Result<TunnelHandle> {
        // 1. Preserve trust relationship
        let trust = self.get_tunnel_trust(old_tunnel)?;
        
        // 2. Re-establish tunnel at new endpoint
        let new_tunnel = self.establish_tunnel(&PeerInfo {
            id: peer_id.clone(),
            endpoint: new_endpoint.clone(),
        }).await?;
        
        // 3. Transfer trust (don't re-verify)
        self.transfer_trust(old_tunnel, &new_tunnel, trust)?;
        
        // 4. Close old tunnel securely
        self.close_tunnel(old_tunnel).await?;
        
        Ok(new_tunnel)
    }
}
```

---

## 📋 Implementation Checklist

### Phase 1: BTSP Integration ✅ (Infrastructure Ready)

**Songbird Side:**
- [x] BTSP provider trait (`BtspProvider`)
- [x] Local implementation for testing (`LocalBtspProvider`)
- [x] Factory for runtime discovery (`BtspProviderFactory`)
- [x] Configuration system (`BtspConfig`)
- [x] Tunnel types (`TunnelHandle`, `TunnelStatus`, `SecurityContext`)

**BearDog Side:**
- [ ] Implement `BtspProvider` trait in BearDog
- [ ] Register with Songbird's UPA
- [ ] Expose BTSP endpoints (HTTP API or direct)
- [ ] Integration tests (Songbird ↔ BearDog)

### Phase 2: Rendezvous Server (Internet Discovery)

**Songbird:**
- [ ] Rendezvous server discovery
- [ ] Registration protocol
- [ ] Peer-to-peer coordination via rendezvous
- [ ] Privacy-preserving presence (no IP exposure)

**BearDog:**
- [ ] Signed registration with rendezvous
- [ ] Peer verification via rendezvous
- [ ] Certificate pinning for rendezvous trust
- [ ] Audit logging for rendezvous activity

### Phase 3: NAT Traversal

**Songbird:**
- [ ] STUN server discovery
- [ ] ICE candidate exchange
- [ ] TURN relay fallback
- [ ] Connection quality monitoring

**BearDog:**
- [ ] Secure STUN queries (prevent injection)
- [ ] Verify STUN responses
- [ ] End-to-end encryption through TURN
- [ ] Trust verification for NAT endpoints

### Phase 4: Mobile/Roaming

**Songbird:**
- [ ] Network change detection
- [ ] Connection migration orchestration
- [ ] Peer notification of migrations
- [ ] Automatic reconnection

**BearDog:**
- [ ] Trust persistence across migrations
- [ ] Secure tunnel re-establishment
- [ ] Key rotation without re-verification
- [ ] Audit trail for migrations

---

## 🧪 Testing Strategy

### Unit Tests

**Songbird:**
```bash
# Test BTSP factory with mock BearDog
cargo test --package songbird-network-federation btsp

# Test rendezvous coordination
cargo test --package songbird-orchestrator rendezvous
```

**BearDog:**
```bash
# Test BTSP provider implementation
cargo test --package beardog-tunnel btsp_provider

# Test trust verification
cargo test --package beardog-security trust
```

### Integration Tests

**Local (Same Machine):**
```bash
# Songbird discovers local BearDog
cd showcase/10-inter-primal-foundation
./04-btsp-local-integration.sh
```

**LAN Federation:**
```bash
# BearDog on each tower, BTSP between Songbirds
cd showcase/11-federation-upa
./03-btsp-federation-test.sh
```

**Internet Simulation:**
```bash
# NAT traversal in docker-compose
cd showcase/12-internet-deployment
./01-rendezvous-simulation.sh
```

### E2E Tests

**Roaming Device:**
```bash
# Mobile device changes networks, maintains trust
./showcase/12-internet-deployment/02-roaming-test.sh
```

---

## 📖 Documentation Requirements

### For Developers

- [ ] `docs/BTSP_INTEGRATION_GUIDE.md` - How to wire BearDog
- [ ] `docs/RENDEZVOUS_PROTOCOL.md` - Internet discovery
- [ ] `docs/NAT_TRAVERSAL_GUIDE.md` - Punching through firewalls
- [ ] `docs/ROAMING_SUPPORT.md` - Mobile device migration

### For Users

- [ ] `docs/INTERNET_DEPLOYMENT_QUICKSTART.md` - Deploy on internet
- [ ] `docs/MOBILE_SETUP.md` - Configure roaming devices
- [ ] `docs/TROUBLESHOOTING_NAT.md` - Fix connectivity issues

---

## 🎯 Success Criteria

### Phase 1: BTSP Integration
- ✅ Songbird can discover BearDog at runtime
- ✅ Songbird falls back gracefully if BearDog unavailable
- [ ] BearDog implements `BtspProvider` trait
- [ ] End-to-end test: Songbird → BearDog → encrypted tunnel

### Phase 2: Rendezvous Server
- [ ] Songbird can register with rendezvous
- [ ] Peers can discover each other via rendezvous
- [ ] No IP addresses exposed in public discovery
- [ ] BearDog verifies all rendezvous interactions

### Phase 3: NAT Traversal
- [ ] STUN successfully determines public endpoint
- [ ] Direct peer-to-peer connection when possible
- [ ] TURN relay fallback when NAT blocks
- [ ] BearDog secures all NAT traversal steps

### Phase 4: Mobile/Roaming
- [ ] Device roams between WiFi networks
- [ ] Connections migrate automatically
- [ ] Trust maintained across migrations
- [ ] No interruption to active tasks

---

## 🔗 Related Specifications

- `specs/UNIVERSAL_PRIMAL_ADAPTER_SPECIFICATION.md` - Capability-based discovery
- `specs/STANDALONE_NETWORK_EFFECTS_ARCHITECTURE_SPEC.md` - Primal collaboration
- `specs/PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md` - Self-knowledge principle
- `docs/BTSP_INTERFACE_GUIDE.md` - BTSP technical details
- `docs/PRIMAL_RESPONSIBILITY_SEPARATION.md` - Architecture discussion
- `docs/PRIVACY_FIRST_FEDERATION.md` - Privacy principles

---

## 📝 Notes

### Why This Matters

**Architectural Clarity:**
- Each primal has a clear, focused responsibility
- No scope creep or responsibility bleed
- Easy to reason about and maintain

**Security Excellence:**
- BearDog is production-ready (Grade A, 96/100)
- Songbird doesn't need to become a security expert
- Leverage BearDog's entropy hierarchy, HSM support, genetic crypto

**Internet-Ready:**
- Privacy-first architecture already in place
- BTSP infrastructure exists
- Just need to wire the connections

### Current Blockers

1. **BearDog BTSP Implementation** - BearDog needs to implement `BtspProvider` trait
2. **Registration** - BearDog needs to register with Songbird's UPA
3. **Testing** - Need end-to-end tests with real BearDog

### Next Steps

1. **Coordinate with BearDog team** - Share this spec
2. **Create BearDog showcase** - `beardog/showcase/01-btsp-provider/`
3. **Integration test** - `songbird/showcase/10-inter-primal-foundation/04-btsp-integration.sh`
4. **Document success** - Update this spec to ✅ when complete

---

*ecoPrimals - Each Primal Knows Its Domain*

