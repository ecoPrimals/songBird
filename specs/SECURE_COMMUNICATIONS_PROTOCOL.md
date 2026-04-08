# 🔒 Secure Communications Protocol - Songbird

**Version**: 1.0  
**Date**: January 27, 2026  
**Status**: ACTIVE - Security Policy Specification  
**Authority**: Security Provider delegates transport security decisions

---

## 🎯 Overview

This specification defines how Songbird establishes secure communications across all transport layers, with security policy delegated to Security Provider's crypto authority.

### Core Principle

**"Security Provider Decides What Goes Where"**

Songbird provides the transport mechanisms (TLS 1.3, TLS 1.2, TLS 1.0, plaintext), but **Security Provider determines**:
- Which data can be transmitted over which transport
- Minimum TLS version for different data classifications
- Whether plaintext is acceptable for specific operations
- Per-connection security policy enforcement

---

## 🏗️ Architecture: Transport Layer Abstraction

### Tower Atomic Pattern (Crypto Delegation)

```
Application Layer (Songbird, other primals)
         ↓
Security Policy Layer (Security Provider)
         ↓ [Policy Decision]
         ↓
Transport Layer (Songbird)
    ├── TLS 1.3 (maximum security)
    ├── TLS 1.2 (legacy compatibility)
    ├── TLS 1.0 (ancient systems)
    └── Plaintext (local/dev only)
```

**Security Provider's Role**:
1. Classify data sensitivity
2. Determine minimum acceptable transport security
3. Approve or reject connection based on policy
4. Audit security decisions

**Songbird's Role**:
1. Implement transport protocols (TLS 1.x)
2. Negotiate highest available version
3. Execute crypto operations via Security Provider
4. Report connection security to Security Provider for policy check

---

## 🔐 Security Policy Framework

### Data Classification (Security Provider Managed)

**Sensitivity Levels**:

```rust
pub enum DataClassification {
    /// Public data - no encryption required
    Public,
    
    /// Internal data - encryption preferred
    Internal,
    
    /// Confidential - encryption required (TLS 1.2+)
    Confidential,
    
    /// Secret - strong encryption required (TLS 1.3 only)
    Secret,
    
    /// Top Secret - TLS 1.3 + additional controls
    TopSecret,
}
```

### Transport Security Matrix (Security Provider Policy)

| Data Classification | Minimum TLS | Allowed Transports | Fallback Allowed |
|---------------------|-------------|-------------------|------------------|
| **Public** | None | Any | Yes |
| **Internal** | TLS 1.0 | TLS 1.0+ | Yes |
| **Confidential** | TLS 1.2 | TLS 1.2+ | Limited |
| **Secret** | TLS 1.3 | TLS 1.3 only | No |
| **Top Secret** | TLS 1.3 | TLS 1.3 + mTLS | No |

**Security Provider enforces these policies at runtime.**

---

## 🔄 Protocol Negotiation Flow

### Phase 1: Transport Capability Discovery

**Songbird** queries available transports:
```rust
// Songbird discovers what it can offer
let available_transports = vec![
    Transport::Tls13,
    Transport::Tls12,
    Transport::Tls10, // if enabled
    Transport::Plaintext, // if local/dev
];
```

### Phase 2: Data Classification

**Application** sends data with classification:
```rust
let request = SecureRequest {
    data: payload,
    classification: DataClassification::Confidential,
    destination: "api.example.com:443",
};
```

### Phase 3: Security Provider Policy Check

**Songbird** asks Security Provider for policy:
```json
{
  "jsonrpc": "2.0",
  "method": "security.transport.validate",
  "params": {
    "data_classification": "Confidential",
    "destination": "api.example.com:443",
    "available_transports": ["tls_1_3", "tls_1_2"],
    "connection_metadata": {
      "is_local": false,
      "is_dev": false,
      "user_override": null
    }
  },
  "id": 1
}
```

**Security Provider** responds with policy decision:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "approved": true,
    "required_transport": "tls_1_2_or_higher",
    "allowed_ciphers": [
      "TLS_AES_128_GCM_SHA256",
      "TLS_AES_256_GCM_SHA384",
      "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
    ],
    "policy_reason": "Confidential data requires TLS 1.2+"
  },
  "id": 1
}
```

### Phase 4: Transport Establishment

**Songbird** negotiates connection:
1. Attempts TLS 1.3 (preferred)
2. Falls back to TLS 1.2 if server doesn't support 1.3
3. Validates negotiated version meets Security Provider's requirement
4. Proceeds if approved, fails if policy violated

### Phase 5: Policy Enforcement

**Songbird** reports negotiated security:
```json
{
  "jsonrpc": "2.0",
  "method": "security.transport.report",
  "params": {
    "connection_id": "conn-12345",
    "negotiated_version": "tls_1_2",
    "cipher_suite": "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "data_classification": "Confidential"
  },
  "id": 2
}
```

**Security Provider** confirms or rejects:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "approved": true,
    "audit_logged": true,
    "connection_token": "bearer-xyz..."
  },
  "id": 2
}
```

---

## 🛡️ Security Guarantees

### Per-Connection Security

**TLS 1.3 Connections**:
- ✅ Perfect Forward Secrecy (mandatory)
- ✅ Encrypted handshake
- ✅ Modern AEAD ciphers only
- ✅ No weak cipher suites
- ✅ Downgrade attack protection

**TLS 1.2 Connections**:
- ✅ Perfect Forward Secrecy (ECDHE required)
- ✅ AEAD ciphers only (AES-GCM)
- ✅ No RSA key exchange
- ✅ No CBC mode
- ✅ Downgrade detection

**TLS 1.0 Connections** (ancient systems only):
- ⚠️ Limited security (legacy compatibility)
- ✅ Best available for ancient systems
- ⚠️ CBC mode may be used
- ⚠️ Weaker than TLS 1.2/1.3
- ✅ Still encrypted (better than plaintext)

**Plaintext Connections** (local/dev only):
- ❌ No encryption
- ❌ No authentication
- ✅ Fast for local development
- ⚠️ NEVER for sensitive data
- ⚠️ NEVER over untrusted networks

### Security Provider Policy Enforcement

**At Connection Time**:
1. Data classification determined
2. Transport capability assessed
3. Policy checked (Security Provider query)
4. Connection approved/rejected
5. Audit logged

**During Data Transfer**:
1. Connection security monitored
2. Policy compliance checked
3. Violations logged
4. Connection terminated if policy violated

---

## 📋 Security Provider Security Methods

### Required Security Provider JSON-RPC Methods

**1. Transport Validation**
```json
{
  "method": "security.transport.validate",
  "params": {
    "data_classification": "string",
    "destination": "string",
    "available_transports": ["string"],
    "connection_metadata": {}
  }
}
```

**2. Connection Approval**
```json
{
  "method": "security.transport.approve",
  "params": {
    "connection_id": "string",
    "negotiated_security": {
      "tls_version": "string",
      "cipher_suite": "string",
      "certificate_validation": "boolean"
    },
    "data_classification": "string"
  }
}
```

**3. Policy Query**
```json
{
  "method": "security.policy.get",
  "params": {
    "data_classification": "string",
    "context": "string"
  }
}
```

**4. Audit Logging**
```json
{
  "method": "security.audit.log",
  "params": {
    "event_type": "string",
    "connection_id": "string",
    "security_details": {},
    "policy_decision": {}
  }
}
```

---

## 🎯 Use Cases

### Use Case 1: Modern API (TLS 1.3)

**Scenario**: Songbird connects to OpenAI API

```rust
// Application sends confidential data
let response = http_client.post("https://api.openai.com/v1/chat/completions")
    .classification(DataClassification::Secret)
    .body(prompt)
    .send().await?;

// Internal flow:
// 1. Songbird: Offers TLS 1.3
// 2. Security Provider: Requires TLS 1.3 for Secret data
// 3. Songbird: Negotiates TLS 1.3 with server
// 4. Security Provider: Approves (policy met)
// 5. Songbird: Sends data
```

**Result**: ✅ TLS 1.3 connection, maximum security

### Use Case 2: Legacy Bank (TLS 1.2)

**Scenario**: Songbird connects to legacy banking API

```rust
// Application sends confidential financial data
let response = http_client.post("https://legacy-bank.com/api/transfer")
    .classification(DataClassification::Confidential)
    .body(transaction)
    .send().await?;

// Internal flow:
// 1. Songbird: Attempts TLS 1.3, server only supports 1.2
// 2. Songbird: Negotiates TLS 1.2 (ECDHE+AES-GCM)
// 3. Security Provider: Checks policy (Confidential allows TLS 1.2)
// 4. Security Provider: Approves (policy met)
// 5. Songbird: Sends data
```

**Result**: ✅ TLS 1.2 connection, best available for legacy system

### Use Case 3: Ancient Embedded Device (TLS 1.0)

**Scenario**: Songbird connects to factory IoT device

```rust
// Application sends internal telemetry
let response = http_client.get("https://factory-iot.local/status")
    .classification(DataClassification::Internal)
    .send().await?;

// Internal flow:
// 1. Songbird: Attempts TLS 1.3, 1.2, device only supports 1.0
// 2. Songbird: Negotiates TLS 1.0
// 3. Security Provider: Checks policy (Internal allows TLS 1.0)
// 4. Security Provider: Approves with warning (weak security logged)
// 5. Songbird: Sends data
```

**Result**: ⚠️ TLS 1.0 connection, logged for audit

### Use Case 4: Policy Violation (Blocked)

**Scenario**: Attempt to send Secret data over TLS 1.2

```rust
// Application tries to send secret data
let response = http_client.post("https://old-server.com/api")
    .classification(DataClassification::Secret)
    .body(secret_data)
    .send().await?;

// Internal flow:
// 1. Songbird: Negotiates TLS 1.2 (server doesn't support 1.3)
// 2. Security Provider: Checks policy (Secret requires TLS 1.3)
// 3. Security Provider: REJECTS (policy violated)
// 4. Songbird: Returns error
```

**Result**: ❌ Connection blocked, error returned to application

### Use Case 5: Local Development (Plaintext)

**Scenario**: Local testing without TLS

```rust
// Application in dev mode
let response = http_client.get("http://localhost:8080/test")
    .classification(DataClassification::Public)
    .allow_plaintext() // explicit opt-in
    .send().await?;

// Internal flow:
// 1. Songbird: Plaintext connection (local)
// 2. Security Provider: Checks policy (Public allows plaintext on localhost)
// 3. Security Provider: Approves (dev mode + public data)
// 4. Songbird: Sends data (no encryption)
```

**Result**: ✅ Plaintext connection, dev mode approved

---

## 🔧 Configuration

### Security Provider Policy Configuration

**Default Policy** (secure by default):
```toml
[security.transport.policy]

# Public data
public.min_tls_version = "none"
public.allow_plaintext = true
public.allow_localhost_plaintext = true

# Internal data
internal.min_tls_version = "tls_1_0"
internal.allow_plaintext = false
internal.allow_localhost_plaintext = true

# Confidential data
confidential.min_tls_version = "tls_1_2"
confidential.allow_plaintext = false
confidential.allow_localhost_plaintext = false
confidential.require_ecdhe = true
confidential.require_aead = true

# Secret data
secret.min_tls_version = "tls_1_3"
secret.allow_plaintext = false
secret.allow_fallback = false
secret.require_certificate_validation = true

# Top Secret data
top_secret.min_tls_version = "tls_1_3"
top_secret.require_mutual_tls = true
top_secret.require_certificate_pinning = true
top_secret.audit_all_connections = true
```

### Songbird Transport Configuration

**Available Transports**:
```toml
[transport.tls]

# TLS version support
enabled_versions = ["1.3", "1.2"] # default
# enabled_versions = ["1.3", "1.2", "1.0"] # ancient systems mode

# Cipher suites (Security Provider enforces, Songbird offers)
tls_1_3_ciphers = [
    "TLS_AES_128_GCM_SHA256",
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256"
]

tls_1_2_ciphers = [
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384"
]

# Certificate validation
validate_certificates = true
validate_hostnames = true
allow_self_signed_dev = true # only if dev mode
```

---

## 📊 Audit & Compliance

### Security Events Logged

**Security Provider Audit Log**:
1. **Policy Decisions**:
   - Data classification determined
   - Transport requirement specified
   - Approval/rejection decision
   - Policy rationale

2. **Connection Events**:
   - TLS version negotiated
   - Cipher suite selected
   - Certificate validation result
   - Policy compliance status

3. **Security Violations**:
   - Policy violations attempted
   - Weak cipher rejection
   - Version downgrade detection
   - Certificate validation failures

4. **Data Transfer**:
   - Data classification
   - Bytes transferred
   - Connection duration
   - Security level maintained

### Compliance Reporting

**Security Provider provides**:
- Real-time security dashboard
- Policy compliance reports
- Violation alerts
- Audit trail for compliance

---

## 🎯 Security Levels Summary

| Level | TLS Version | Cipher Requirements | Use Case |
|-------|-------------|---------------------|----------|
| **Maximum** | TLS 1.3 only | AEAD, PFS | Secret data, modern APIs |
| **High** | TLS 1.2+ | ECDHE+AEAD | Confidential, legacy compatibility |
| **Medium** | TLS 1.0+ | Encryption required | Internal, ancient systems |
| **Low** | TLS optional | Any encryption | Public data, dev/test |
| **None** | Plaintext OK | N/A | Public data, localhost only |

**Security Provider determines which level applies to each connection.**

---

## ✅ Implementation Status

### Current (v8.11.0)
- ✅ TLS 1.3 support
- ✅ Security Provider crypto delegation
- ✅ Tower Atomic pattern
- ⏳ Policy framework (design phase)
- ⏳ TLS 1.2 support (planned)

### Planned
- [ ] Security Provider security.transport.* methods
- [ ] Data classification API
- [ ] Policy enforcement framework
- [ ] TLS 1.2 + 1.0 support
- [ ] Audit logging integration

### Timeline
- **Q1 2026**: Policy framework + TLS 1.2
- **Q2 2026**: Full implementation + audit
- **Q3 2026**: Production hardening

---

## 📚 Related Specifications

- **SONGBIRD_TLS_13_COMPLETE.md** - Current TLS 1.3 implementation
- **TARPC_JSON_RPC_PROTOCOL_SPEC.md** - RPC protocol
- **Archived security provider integration spec** ([fossil record copy](../../../infra/wateringHole/fossilRecord/consolidated-apr2026/SONGBIRD_BEARDOG_INTEGRATION.md)) — Security Provider integration
- **SONGBIRD_ACCESS_CONTROL.md** - Access control framework

---

## 🎯 Key Takeaways

1. **Security Provider Decides Security** - Policy authority delegated to crypto primal
2. **Songbird Implements Transports** - TLS 1.0/1.2/1.3 + plaintext
3. **Version Negotiation** - Always try highest, fall back if needed
4. **Policy Enforcement** - Security Provider approves/rejects based on data classification
5. **Audit Everything** - All security decisions logged
6. **Flexibility** - Support ancient to modern systems
7. **Security First** - No connection if policy violated

**Bottom Line**: Songbird provides the pipes, Security Provider controls what flows through them.

---

**Version**: 1.0  
**Last Updated**: January 27, 2026  
**Status**: ACTIVE  
**Authority**: Security Provider Security Team

