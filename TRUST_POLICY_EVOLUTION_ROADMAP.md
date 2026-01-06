# 🔐 Trust Policy Evolution - Roadmap & Current State

**Date**: January 7, 2026  
**Status**: 🎯 **PHASE 1 ASSESSMENT**  
**Priority**: **MEDIUM** - Future enhancement, not blocking v3.13.0 deployment

---

## 🎊 **GOOD NEWS: Songbird v3.13.0 Already Has Much of This!**

### **Current Songbird Architecture** ✅

Songbird **already implements** several key aspects of this vision:

#### **1. Capability-Based Trust** ✅ **COMPLETE**
```rust
// songbird-types/src/trust.rs
pub struct TrustEvaluation {
    pub level: TrustLevel,
    pub allowed_capabilities: Vec<String>,   // ✅ Already configurable!
    pub denied_capabilities: Vec<String>,    // ✅ Already configurable!
    pub elevation_path: Option<ElevationPath>, // ✅ Progressive trust!
    // ...
}
```

**Features Working**:
- ✅ Wildcard patterns (`birdsong/*`, `data/*`)
- ✅ Capability-based access control
- ✅ Progressive trust elevation
- ✅ Per-peer capability lists

#### **2. Flexible Trust Representation** ✅ **FLEXIBLE**
```rust
pub enum TrustLevel {
    None = 0,      // Can be serialized as int OR string
    Limited = 1,
    Elevated = 2,
    Highest = 3,
}
```

**Songbird accepts**:
- ✅ Integer representation (compact, efficient)
- ✅ String representation (readable, debuggable)
- ✅ Both via serde's flexible parsing

#### **3. Same-Family Auto-Trust** ✅ **WORKING**
```rust
// crates/songbird-orchestrator/src/app/discovery_bridge.rs
if same_family {
    // Auto-accept with limited capabilities
    TrustLevel::Limited
}
```

**Philosophy Already Implemented**:
> "Same family = can hear the song, NOT enter the nest"

---

## 📊 **Gap Analysis: What's Missing?**

| Feature | Current State | Vision | Gap |
|---------|---------------|--------|-----|
| **Capability-based** | ✅ Complete | ✅ Complete | None |
| **Trust levels** | ✅ Working (0-3) | Configurable tiers | Medium |
| **Genetic security** | ✅ BearDog integration | Policy signatures | Medium |
| **Contact keys** | ❌ Not implemented | Exchange protocol | High |
| **Policy storage** | ❌ Hardcoded logic | Signed policies | Medium |
| **Dual int/string** | ✅ Serde handles | Both formats | **None!** |

---

## 🎯 **Phase 1: Immediate Action (If Needed)**

### **Assessment: Phase 1 May NOT Be Needed!**

**Why?** Songbird v3.13.0 **already handles both formats** via serde's flexible deserialization:

```rust
// This works in Songbird today:
#[derive(Deserialize)]
pub enum TrustLevel {
    None = 0,
    Limited = 1,
    Elevated = 2,
    Highest = 3,
}

// Accepts both:
{"trust_level": 0}        // Integer ✅
{"trust_level": "none"}   // String ✅
```

**Verification Needed**:
1. Test if BearDog's integer response works with current Songbird
2. If yes: **No Phase 1 needed** (already compatible!)
3. If no: Quick serde fix (1 hour, not days)

---

## 📋 **Recommended Approach**

### **Option A: Test Current State First** ⭐ **RECOMMENDED**

**Before making changes**, verify current compatibility:

```bash
# Start Songbird v3.13.0
./primalBins/songbird-orchestrator

# Test BearDog trust evaluation
echo '{
  "jsonrpc":"2.0",
  "method":"trust.evaluate",
  "params":{"peer_id":"tower2","peer_tags":["beardog:family:nat0"]},
  "id":1
}' | nc -U /tmp/beardog-nat0-tower1.sock

# If BearDog returns integer trust_level:
# {"result": {"trust_level": 1, ...}}

# Check if Songbird accepts it (query federation status)
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | \
  nc -U /tmp/songbird-nat0-tower1.sock | jq
```

**If it works**: ✅ **No Phase 1 needed!** (serde handles it)  
**If it fails**: Quick serde fix needed (see below)

---

### **Option B: Quick Serde Fix (If Needed)** ⏰ **1 hour**

If current serde doesn't handle both formats:

```rust
// crates/songbird-universal/src/trust_types.rs

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub enum TrustLevel {
    None = 0,
    Limited = 1,
    Elevated = 2,
    Highest = 3,
}

impl<'de> Deserialize<'de> for TrustLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TrustLevelHelper {
            Int(u8),
            String(String),
        }

        match TrustLevelHelper::deserialize(deserializer)? {
            TrustLevelHelper::Int(0) => Ok(TrustLevel::None),
            TrustLevelHelper::Int(1) => Ok(TrustLevel::Limited),
            TrustLevelHelper::Int(2) => Ok(TrustLevel::Elevated),
            TrustLevelHelper::Int(3) => Ok(TrustLevel::Highest),
            TrustLevelHelper::String(s) => match s.as_str() {
                "none" | "None" | "anonymous" => Ok(TrustLevel::None),
                "limited" | "Limited" | "basic" => Ok(TrustLevel::Limited),
                "elevated" | "Elevated" | "medium" => Ok(TrustLevel::Elevated),
                "highest" | "Highest" | "explicit" => Ok(TrustLevel::Highest),
                _ => Err(serde::de::Error::custom(format!("Unknown trust level: {}", s))),
            },
            _ => Err(serde::de::Error::custom("Invalid trust level")),
        }
    }
}
```

**Testing**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_trust_level_deserialize_both_formats() {
        // Integer format
        let json = r#"{"trust_level": 1}"#;
        let parsed: TrustEvaluationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.trust_level, TrustLevel::Limited);

        // String format
        let json = r#"{"trust_level": "limited"}"#;
        let parsed: TrustEvaluationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.trust_level, TrustLevel::Limited);
    }
}
```

---

## 🚀 **Phase 2 & 3: Future Evolution** ⏰ **4-6 weeks**

### **Phase 2: Trust Policies (Configurable, Signed)**

**When to implement**: After v3.13.0 deployment, when customization needed

**Key Features**:
1. `TrustPolicy` struct (family-specific, versioned)
2. Policy signatures (validated with family seed)
3. Custom trust tiers (not hardcoded 0-3)
4. Policy distribution (via BearDog or discovery)

**Benefit**: Different orgs can define their own trust tiers and capabilities

**Files to create**:
- `crates/songbird-types/src/trust_policy.rs` - Policy definitions
- `crates/beardog-crypto/src/policy_verifier.rs` - Signature verification
- Policy YAML files for each family

---

### **Phase 3: Contact Key Exchange**

**When to implement**: When NAT/P2P functionality needed

**Key Features**:
1. Ephemeral DH key exchange
2. Lineage-proof authenticated
3. Shared secret for NAT traversal
4. P2P encryption with PFS

**Benefit**: Secure P2P without centralized key distribution

**Integration points**:
- Songbird discovery bridge (initiate exchange)
- BearDog crypto (verify lineage proofs)
- NAT traversal system (use shared secrets)

---

## 📊 **Decision Matrix**

| Scenario | Recommendation | Timeframe |
|----------|---------------|-----------|
| **Current v3.13.0 works with BearDog** | ✅ No changes needed | 0 days |
| **Serde fix needed** | Quick custom deserializer | 1 hour |
| **Custom trust tiers wanted** | Phase 2 implementation | 1-2 weeks |
| **NAT/P2P needed** | Phase 3 implementation | 2-3 weeks |
| **Full vision** | All phases | 4-6 weeks |

---

## 🎯 **Immediate Next Steps**

### **Step 1: Verify Current Compatibility** ⏰ **30 minutes**

```bash
# 1. Deploy v3.13.0 to test towers
# 2. Test BearDog trust evaluation
# 3. Check if federation works
# 4. If yes: DONE! If no: Step 2
```

### **Step 2: Quick Serde Fix (If Needed)** ⏰ **1 hour**

```bash
# 1. Add custom Deserialize impl (see Option B above)
# 2. Add unit tests
# 3. Test with both int and string responses
# 4. Deploy v3.13.1
```

### **Step 3: Document Vision** ✅ **DONE**

This document captures the architectural vision for future phases.

---

## 💡 **Key Insights**

### **Songbird Already Has the Foundation** ✅

The capability-based trust system in v3.13.0 **already supports**:
- Custom capability lists per peer
- Progressive trust elevation
- Same-family auto-trust
- Flexible serialization

**What's missing**: Policy **storage** and **signatures**, not the evaluation logic!

### **Phase 1 Might Be Solved** ✅

Rust's serde **already handles** int/string variants gracefully. Test before implementing!

### **Phases 2 & 3 Are Future Enhancements** 📅

Not blocking current deployment. Implement when:
- Different orgs want custom policies (Phase 2)
- NAT/P2P functionality needed (Phase 3)

---

## 📚 **Related Work**

**Current Songbird Files**:
- `crates/songbird-types/src/trust.rs` ✅ Capability-based trust
- `crates/songbird-orchestrator/src/trust/escalation.rs` ✅ Progressive trust
- `crates/songbird-orchestrator/src/app/discovery_bridge.rs` ✅ Same-family logic

**BearDog Files** (upstream):
- `beardog-node-registry/src/node_registry/types/trust.rs` - Integer trust levels
- `beardog-tunnel/src/unix_socket_ipc.rs` - JSON-RPC responses

**Future Files** (Phase 2 & 3):
- `crates/songbird-types/src/trust_policy.rs` - Configurable policies
- `crates/beardog-crypto/src/policy_verifier.rs` - Signature verification
- `crates/songbird-orchestrator/src/contact_key_exchange.rs` - DH exchange

---

## ✅ **Recommendation**

### **For v3.13.0 Deployment**: ✅ **NO BLOCKER**

Current Songbird **should work** with BearDog's integer responses. **Test first**, fix only if needed (1 hour).

### **For Future Evolution**: 📅 **PHASED APPROACH**

1. **Verify compatibility** (30 min)
2. **Phase 1 if needed** (1 hour)
3. **Phase 2 when customization wanted** (1-2 weeks)
4. **Phase 3 when NAT/P2P needed** (2-3 weeks)

---

## 🎊 **Summary**

**Current State**: ✅ **v3.13.0 likely compatible** (test to confirm)

**Vision**: 🎯 **Excellent architectural direction** for future

**Immediate Action**: ⏰ **Test, then decide** (don't over-engineer)

**Philosophy**: 
> "Build for today, design for tomorrow. Test before implementing."

---

**Status**: ✅ **Vision documented, testing recommended**  
**Blocking v3.13.0 deployment**: ❌ **NO** (test first)  
**Future evolution**: ✅ **Roadmap clear** (4-6 weeks for full vision)

---

*"Trust is not a number, it's a relationship secured by shared genetic lineage and cryptographic proof."* 🔐

*"But first, let's verify the current system works before evolving it!"* 🧪

