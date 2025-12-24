# 🐻 BearDog v0.9.0 Integration Guide - December 24, 2025

**Status**: 🟢 **Ready for Integration**  
**BearDog Release**: v0.9.0-integration-dec23  
**Songbird Lineage Relay**: v0.1.0 (Ready)

---

## 🎯 Quick Start

### **BearDog v0.9.0 Availability**

**Local Testing**:
```bash
../phase2/phase1bins/beardog-v0.9.0-dec23 --version
```

**GitHub Release** (for staging):
```
https://github.com/ecoPrimals/bearDog/releases/tag/v0.9.0-integration-dec23
```

---

## 🔄 Integration Status

### **Songbird Side** ✅
- ✅ Lineage relay system implemented (`songbird-lineage-relay`)
- ✅ BirdSong broadcast protocol ready
- ✅ Universal Coordinator integration complete
- ✅ Mock implementations for testing
- ✅ 18/18 tests passing

### **BearDog v0.9.0** 🟢
- 🟢 Available for integration
- ⏳ Check release notes for Genesis lineage support
- ⏳ Verify BirdSong crypto capabilities
- ⏳ Test relay authorization API

---

## 🚀 Integration Steps

### **Phase 1: Local Testing** (Day 1)

#### 1.1 Verify BearDog Installation
```bash
# Check version
../phase2/phase1bins/beardog-v0.9.0-dec23 --version

# Expected output:
# beardog v0.9.0-integration-dec23
```

#### 1.2 Check Available Features
```bash
# List capabilities
../phase2/phase1bins/beardog-v0.9.0-dec23 --help

# Look for:
# - Genesis lineage commands
# - BirdSong encryption
# - Relay authorization
```

#### 1.3 Test Basic Lineage Operations
```bash
# Test lineage signing (if available)
beardog genesis sign-lineage \
  --parent node-1 \
  --child node-2

# Test lineage verification
beardog lineage verify \
  --ancestor node-1 \
  --descendant node-2
```

---

### **Phase 2: Songbird Integration** (Day 2-3)

#### 2.1 Update Configuration

**Environment Variables**:
```bash
# Point Songbird to BearDog
export BEARDOG_ENDPOINT="http://localhost:8443"
export BEARDOG_VERSION="v0.9.0"

# Enable lineage relay
export ENABLE_LINEAGE_RELAY=true

# BirdSong configuration
export BIRDSONG_BIND="0.0.0.0:42424"
export BIRDSONG_BROADCAST="255.255.255.255:42424"
```

#### 2.2 Replace Mock Implementations

**Before** (Mock):
```rust
use songbird_lineage_relay::beardog::{MockLineageProvider, MockBirdSongCrypto};

let lineage = Arc::new(MockLineageProvider::new());
let crypto = Arc::new(MockBirdSongCrypto::new(lineage.clone(), my_id));
```

**After** (Real BearDog):
```rust
use songbird_lineage_relay::beardog_client::{BeardogLineageProvider, BeardogBirdSongCrypto};

// Connect to real BearDog
let beardog_client = BeardogClient::connect("http://localhost:8443").await?;

let lineage = Arc::new(BeardogLineageProvider::new(beardog_client.clone()));
let crypto = Arc::new(BeardogBirdSongCrypto::new(beardog_client, my_id));
```

#### 2.3 Test Genesis Lineage

**Create Test Lineage**:
```rust
// Use BearDog to establish lineage
let parent = NodeId::from("integration-test-parent");
let child = NodeId::from("integration-test-child");

// BearDog signs the lineage
let lineage_proof = beardog_client
    .sign_birth(parent, child)
    .await?;

println!("✅ Lineage established: {} → {}", parent, child);
```

#### 2.4 Test BirdSong Encryption

**Encrypt for Lineage**:
```rust
let message = b"test relay request";

// Encrypt using BearDog genetic crypto
let encrypted = crypto
    .encrypt_for_lineage(message, LineageHint::DirectAncestors)
    .await?;

// Only ancestors can decrypt
let decrypted = crypto
    .decrypt_birdsong(&encrypted, &sender_id)
    .await?;

println!("✅ BirdSong encryption working");
```

---

### **Phase 3: End-to-End Testing** (Day 4-5)

#### 3.1 Three-Node Test Setup

**Scenario**: Grandparent → Parent → Child (child behind NAT)

**Node Setup**:
```bash
# Terminal 1: Grandparent (relay provider)
ROLE=grandparent \
MY_NODE_ID=grandparent-1 \
RELAY_ADDRESS=0.0.0.0:9000 \
cargo run --release --bin songbird-orchestrator

# Terminal 2: Parent (relay provider)
ROLE=parent \
MY_NODE_ID=parent-1 \
RELAY_ADDRESS=0.0.0.0:9001 \
cargo run --release --bin songbird-orchestrator

# Terminal 3: Child (relay requester - simulated NAT)
ROLE=child \
MY_NODE_ID=child-1 \
SIMULATE_NAT=true \
cargo run --release --bin songbird-orchestrator
```

#### 3.2 Establish Genesis Lineage

```bash
# Use BearDog to create lineage
beardog genesis establish \
  --parent grandparent-1 \
  --child parent-1

beardog genesis establish \
  --parent parent-1 \
  --child child-1

# Verify lineage graph
beardog lineage query --node child-1
# Expected: child-1 → parent-1 → grandparent-1
```

#### 3.3 Test Relay Request

```bash
# Child requests relay (from child terminal)
songbird-cli relay request \
  --target peer-outside-nat \
  --address 192.168.1.100:8080

# Expected flow:
# 1. Child broadcasts relay request (BirdSong)
# 2. Parent/grandparent decrypt (are ancestors)
# 3. Parent offers relay
# 4. Connection established through parent
```

#### 3.4 Verify Privacy

```bash
# Unrelated node cannot decrypt
ROLE=unrelated \
MY_NODE_ID=unrelated-1 \
cargo run --release --bin songbird-orchestrator

# Should see: BirdSong noise (cannot decrypt)
# Should NOT see: Relay request details
```

---

### **Phase 4: Staging Deployment** (Week 2)

#### 4.1 Deploy BearDog v0.9.0 to Staging

```bash
# Download from GitHub release
wget https://github.com/ecoPrimals/bearDog/releases/download/v0.9.0-integration-dec23/beardog-v0.9.0-linux-x86_64.tar.gz

# Extract
tar -xzf beardog-v0.9.0-linux-x86_64.tar.gz

# Deploy to staging
scp beardog staging:/opt/beardog/bin/beardog-v0.9.0
ssh staging "systemctl restart beardog"
```

#### 4.2 Deploy Songbird with Lineage Relay

```bash
# Build Songbird with lineage relay
cargo build --release --features lineage-relay

# Deploy to staging
scp target/release/songbird-orchestrator staging:/opt/songbird/bin/
ssh staging "systemctl restart songbird"
```

#### 4.3 Staging Validation

```bash
# Health check
curl http://staging:8080/health

# Check lineage relay status
curl http://staging:8080/api/lineage-relay/status

# Expected response:
# {
#   "status": "operational",
#   "beardog_version": "v0.9.0",
#   "active_relays": 0,
#   "lineage_relay_enabled": true
# }
```

---

## 🧪 Test Scenarios

### **Scenario 1: Direct Connection (No Relay Needed)**

**Setup**: Both nodes have public IPs

**Expected**:
```
✅ Direct connection attempt: SUCCESS
⏩ Relay NOT used
Connection type: Direct
```

### **Scenario 2: NAT Traversal via Parent Relay**

**Setup**: Child behind NAT, parent has public IP

**Expected**:
```
❌ Direct connection attempt: FAILED
📡 Requesting relay from ancestors...
✅ Parent relay offer received
✅ Connection established through parent
Connection type: Relayed (masked)
```

### **Scenario 3: Privacy Preservation**

**Setup**: Unrelated node observes traffic

**Expected**:
```
👁️ Unrelated node sees: Encrypted BirdSong (noise)
❌ Cannot decrypt relay request
❌ Cannot see relay metadata
Privacy: ✅ PRESERVED
```

### **Scenario 4: Relay Authorization Denied**

**Setup**: Non-ancestor tries to relay

**Expected**:
```
❌ Relay authorization: DENIED
Reason: Not in lineage
Child cannot relay for parent: ✅ CORRECT
```

---

## 🔍 Verification Checklist

### **BearDog Integration**
- [ ] BearDog v0.9.0 running
- [ ] Genesis lineage API working
- [ ] BirdSong encryption functional
- [ ] Relay authorization operational
- [ ] Lineage graph queries working

### **Songbird Integration**
- [ ] Lineage relay coordinator started
- [ ] BirdSong broadcaster operational
- [ ] Relay discovery working
- [ ] Connection sessions functional
- [ ] Universal Coordinator integration active

### **End-to-End**
- [ ] Genesis ceremony creates lineage
- [ ] Ancestors can decrypt descendant BirdSong
- [ ] Non-family sees only noise
- [ ] Relay authorization based on lineage
- [ ] Direct connection attempted first
- [ ] Relay fallback works
- [ ] Privacy masking active

---

## 🐛 Troubleshooting

### **Issue**: BearDog connection refused
```bash
# Check BearDog status
systemctl status beardog

# Check endpoint
curl http://localhost:8443/health

# Verify environment variable
echo $BEARDOG_ENDPOINT
```

### **Issue**: Lineage verification fails
```bash
# Query lineage graph
beardog lineage query --node <node-id>

# Check parent-child relationship
beardog lineage verify --parent <parent> --child <child>

# Re-establish if needed
beardog genesis establish --parent <parent> --child <child>
```

### **Issue**: BirdSong decryption fails
```bash
# Check crypto keys
beardog keys list

# Verify sender is in lineage
beardog lineage is-ancestor --ancestor <me> --descendant <sender>

# Test encryption/decryption
beardog birdsong test --message "test" --hint DirectAncestors
```

### **Issue**: Relay authorization denied
```bash
# Verify lineage relationship
beardog lineage verify --ancestor <relay> --descendant <requester>

# Check relay authority
beardog relay authorize --relay <node> --requester <node>

# Expected: authorized=true if ancestor
```

---

## 📊 Monitoring

### **Metrics to Watch**

**BearDog**:
```
beardog_lineage_verifications_total
beardog_birdsong_encryptions_total
beardog_relay_authorizations_total
beardog_relay_authorizations_denied_total
```

**Songbird**:
```
songbird_lineage_relay_requests_total
songbird_direct_connection_attempts_total
songbird_direct_connection_successes_total
songbird_relay_connections_total
songbird_birdsong_broadcasts_total
songbird_birdsong_received_total
songbird_birdsong_decrypted_total (family)
songbird_birdsong_noise_total (non-family)
```

### **Logging**

**Enable Debug Logging**:
```bash
export RUST_LOG=songbird_lineage_relay=debug,beardog=debug
```

**Watch for**:
- Genesis lineage establishment
- BirdSong encryption/decryption
- Relay requests and offers
- Authorization checks
- Privacy masking events

---

## 🎯 Success Criteria

### **Phase 1: Local Testing** ✅
- BearDog v0.9.0 running locally
- Basic lineage operations working
- Songbird can connect to BearDog

### **Phase 2: Integration** ✅
- Real BearDog replacing mocks
- Genesis lineage functional
- BirdSong encryption operational

### **Phase 3: E2E Testing** ✅
- Three-node relay scenario working
- Privacy preserved for non-family
- Relay authorization based on lineage

### **Phase 4: Staging** ✅
- BearDog v0.9.0 deployed
- Songbird with lineage relay deployed
- Health checks passing
- Ready for production

---

## 📚 References

### **Documentation**
- [BEARDOG_LINEAGE_RELAY_HANDOFF.md](BEARDOG_LINEAGE_RELAY_HANDOFF.md) - BearDog API spec
- [NAT_TRAVERSAL_VIA_LINEAGE.md](NAT_TRAVERSAL_VIA_LINEAGE.md) - Integration architecture
- [LINEAGE_RELAY_COMPLETE_DEC_24.md](LINEAGE_RELAY_COMPLETE_DEC_24.md) - Implementation summary

### **Code**
- `crates/songbird-lineage-relay/` - Songbird implementation
- `crates/songbird-lineage-relay/tests/` - Integration test examples
- `crates/songbird-lineage-relay/src/beardog.rs` - Mock implementations (reference)

### **BearDog**
- GitHub Release: https://github.com/ecoPrimals/bearDog/releases/tag/v0.9.0-integration-dec23
- Local Binary: `../phase2/phase1bins/beardog-v0.9.0-dec23`

---

## 🚀 Next Steps

### **Immediate (This Week)**
1. Download BearDog v0.9.0
2. Test basic lineage operations
3. Replace mocks with real BearDog
4. Run integration tests

### **Short-Term (Next Week)**
1. Three-node relay testing
2. Privacy verification
3. Performance testing
4. Staging deployment

### **Long-Term (Q1 2025)**
1. Production deployment
2. Hardware root of trust (SoloKey)
3. Multi-relay support
4. Advanced privacy features

---

**Status**: 🟢 **Ready for Integration**  
**Timeline**: Start integration immediately  
**Support**: #songbird-lineage-relay on Slack

🐻 **BearDog v0.9.0** + 🌳 **Songbird Lineage Relay** = 🧬 **Genetic Lineage Connectivity**

---

**Last Updated**: December 24, 2025  
**BearDog Version**: v0.9.0-integration-dec23  
**Songbird Version**: With lineage-relay v0.1.0

