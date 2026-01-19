# 🔐 Phase 2: Hybrid Certificate Strategy

**Date**: January 19, 2026  
**Status**: Planning  
**Approach**: **Hybrid** - Standalone + BearDog Integration

---

## 🎯 STRATEGY: Best of Both Worlds

### **Philosophy**
1. **Songbird is secure by default and alone** ✅
   - Built-in Pure Rust cert generation (`ed25519-dalek`)
   - Zero external dependencies for basic operation
   - Works immediately, no BearDog required

2. **BearDog provides enhanced capabilities** ✅
   - Runtime discovery and delegation
   - Enhanced key management
   - Lineage tracking and attestation
   - HSM-backed certificates (when available)

---

## 📋 IMPLEMENTATION PLAN

### **Step 1: Built-in Pure Rust Cert Generation** (2-3 hours)

Create `crates/songbird-tls/src/cert/generator.rs`:

```rust
//! Pure Rust Certificate Generation (Standalone)
//! 
//! Songbird can generate self-signed certificates using ed25519-dalek
//! for standalone operation. When BearDog is available, it delegates
//! to BearDog for enhanced capabilities.

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, anyhow};

/// Certificate generation mode
#[derive(Debug, Clone)]
pub enum CertGenerationMode {
    /// Standalone: Use built-in ed25519-dalek
    Standalone,
    /// BearDog: Delegate to BearDog for enhanced capabilities
    BearDog { endpoint: String },
    /// Auto: Try BearDog, fallback to standalone
    Auto,
}

/// Certificate generator (hybrid approach)
pub struct CertificateGenerator {
    mode: CertGenerationMode,
    beardog_client: Option<BeardogCryptoClient>,
}

impl CertificateGenerator {
    /// Create a new generator with auto-discovery
    pub async fn new() -> Result<Self> {
        Self::with_mode(CertGenerationMode::Auto).await
    }

    /// Create with explicit mode
    pub async fn with_mode(mode: CertGenerationMode) -> Result<Self> {
        let beardog_client = match &mode {
            CertGenerationMode::BearDog { .. } | CertGenerationMode::Auto => {
                // Try to discover BearDog
                BeardogCryptoClient::new().await.ok()
            }
            CertGenerationMode::Standalone => None,
        };

        Ok(Self {
            mode,
            beardog_client,
        })
    }

    /// Generate a self-signed certificate
    pub async fn generate_self_signed(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<Certificate> {
        // Try BearDog first if available
        if let Some(ref client) = self.beardog_client {
            match self.generate_via_beardog(client, domain, validity_days).await {
                Ok(cert) => {
                    tracing::info!("✅ Generated certificate via BearDog: {}", domain);
                    return Ok(cert);
                }
                Err(e) => {
                    tracing::warn!("⚠️ BearDog cert generation failed: {}, falling back to standalone", e);
                }
            }
        }

        // Fallback to standalone
        self.generate_standalone(domain, validity_days)
    }

    /// Standalone generation using ed25519-dalek
    fn generate_standalone(&self, domain: &str, validity_days: u32) -> Result<Certificate> {
        tracing::info!("🔐 Generating standalone certificate: {}", domain);

        // Generate Ed25519 keypair
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Create certificate
        let not_before = Utc::now();
        let not_after = not_before + Duration::days(validity_days as i64);

        let cert = Certificate {
            version: 3,
            serial_number: Self::generate_serial(),
            signature_algorithm: SignatureAlgorithm::Ed25519,
            issuer: distinguished_name(&format!("CN={}", domain)),
            subject: distinguished_name(&format!("CN={}", domain)),
            not_before,
            not_after,
            public_key: verifying_key.to_bytes().to_vec(),
            extensions: vec![
                Extension::SubjectAlternativeName(vec![domain.to_string()]),
                Extension::KeyUsage(KeyUsage::DigitalSignature | KeyUsage::KeyEncipherment),
                Extension::ExtendedKeyUsage(vec![
                    ExtendedKeyUsage::ServerAuth,
                    ExtendedKeyUsage::ClientAuth,
                ]),
            ],
            signature: Vec::new(), // Will be filled below
        };

        // Sign the certificate
        let cert_bytes = cert.encode_for_signing()?;
        let signature = signing_key.sign(&cert_bytes);

        Ok(Certificate {
            signature: signature.to_bytes().to_vec(),
            ..cert
        })
    }

    /// Enhanced generation via BearDog
    async fn generate_via_beardog(
        &self,
        client: &BeardogCryptoClient,
        domain: &str,
        validity_days: u32,
    ) -> Result<Certificate> {
        tracing::info!("🐻 Generating certificate via BearDog: {}", domain);

        // Request BearDog to generate a certificate
        // BearDog will:
        // - Use HSM-backed keys (if available)
        // - Track lineage
        // - Provide attestation
        // - Enable key rotation
        let params = serde_json::json!({
            "domain": domain,
            "validity_days": validity_days,
            "key_type": "Ed25519",
            "usage": ["serverAuth", "clientAuth"],
        });

        let result = client.call_method("certificate.generate_self_signed", params).await?;
        
        // Parse certificate from BearDog's response
        let cert_data: Vec<u8> = serde_json::from_value(result["certificate"].clone())?;
        Certificate::decode(&cert_data)
    }

    /// Generate a random serial number
    fn generate_serial() -> Vec<u8> {
        use rand::RngCore;
        let mut serial = vec![0u8; 20];
        OsRng.fill_bytes(&mut serial);
        serial
    }
}

/// Helper: Create a distinguished name
fn distinguished_name(cn: &str) -> String {
    cn.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standalone_cert_generation() {
        let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone)
            .await
            .unwrap();

        let cert = generator
            .generate_self_signed("test.songbird.local", 365)
            .await
            .unwrap();

        assert_eq!(cert.signature_algorithm, SignatureAlgorithm::Ed25519);
        assert!(cert.signature.len() > 0);
    }

    #[tokio::test]
    async fn test_auto_mode_fallback() {
        // Auto mode should work even without BearDog
        let generator = CertificateGenerator::new().await.unwrap();
        
        let cert = generator
            .generate_self_signed("auto.songbird.local", 90)
            .await
            .unwrap();

        assert!(cert.signature.len() > 0);
    }
}
```

---

### **Step 2: Extend BearDog JSON-RPC Methods** (1-2 hours)

Add certificate generation to BearDog's JSON-RPC API:

**In `crates/songbird-tls/src/crypto.rs`** (extend `BeardogCryptoClient`):

```rust
impl BeardogCryptoClient {
    // ... existing methods ...

    /// Generate a self-signed certificate via BearDog
    pub async fn generate_certificate(
        &self,
        domain: &str,
        validity_days: u32,
    ) -> Result<Vec<u8>, TlsError> {
        let params = serde_json::json!({
            "domain": domain,
            "validity_days": validity_days,
            "key_type": "Ed25519",
            "usage": ["serverAuth", "clientAuth"],
        });

        let result = self.call_method("certificate.generate_self_signed", params).await?;
        
        serde_json::from_value(result["certificate"].clone())
            .map_err(|e| TlsError::CryptoOperation(format!("Certificate parse error: {}", e)))
    }

    /// Request certificate signing from BearDog
    pub async fn sign_certificate(
        &self,
        csr: &[u8],
        key_id: &str,
    ) -> Result<Vec<u8>, TlsError> {
        let params = serde_json::json!({
            "csr": base64::encode(csr),
            "key_id": key_id,
        });

        let result = self.call_method("certificate.sign_request", params).await?;
        
        let cert_b64: String = serde_json::from_value(result["certificate"].clone())
            .map_err(|e| TlsError::CryptoOperation(format!("Certificate parse error: {}", e)))?;
        
        base64::decode(&cert_b64)
            .map_err(|e| TlsError::CryptoOperation(format!("Base64 decode error: {}", e)))
    }
}
```

---

### **Step 3: Update Certificate Manager** (30 minutes)

**In `crates/songbird-orchestrator/src/app/cert_manager.rs`**:

```rust
use songbird_tls::cert::CertificateGenerator;

pub struct CertificateManager {
    generator: CertificateGenerator,
    // ... existing fields ...
}

impl CertificateManager {
    pub async fn new(node_id: &str) -> Result<Self> {
        // Auto-discover BearDog, fallback to standalone
        let generator = CertificateGenerator::new().await?;
        
        Ok(Self {
            generator,
            // ... existing initialization ...
        })
    }

    pub async fn ensure_certificate(&mut self) -> Result<()> {
        // Try to load existing cert
        if let Ok(cert) = self.load_certificate().await {
            if !cert.is_expired() {
                tracing::info!("✅ Using existing certificate");
                return Ok(());
            }
        }

        // Generate new certificate (hybrid mode)
        tracing::info!("🔐 Generating new certificate...");
        let cert = self.generator
            .generate_self_signed(&self.domain, 365)
            .await?;

        // Save certificate
        self.save_certificate(&cert).await?;
        
        tracing::info!("✅ Certificate ready: {}", self.domain);
        Ok(())
    }
}
```

---

### **Step 4: Remove `rcgen` Dependency** (5 minutes)

**In affected `Cargo.toml` files**:

```bash
# crates/songbird-network-federation/Cargo.toml
# DELETE: rcgen = "0.14"

# crates/songbird-network/Cargo.toml  
# DELETE: rcgen = "0.14"
```

**Add dependency**:
```toml
ed25519-dalek = "2.1"  # Pure Rust Ed25519
rand = "0.8"           # For key generation
chrono = "0.4"         # For certificate validity
```

---

## 🎯 BENEFITS OF HYBRID APPROACH

### **Standalone Benefits** ✅
1. **Zero External Dependencies**
   - Songbird works immediately
   - No BearDog required for basic TLS
   - Perfect for development/testing

2. **Pure Rust Security**
   - `ed25519-dalek` is 100% Pure Rust
   - No C dependencies
   - Modern, secure Ed25519 signatures

3. **Fast & Simple**
   - Instant certificate generation
   - No network calls for basic operation
   - Predictable behavior

---

### **BearDog Integration Benefits** ✅
1. **Enhanced Security**
   - HSM-backed certificates (when available)
   - Hardware security modules
   - Tamper-resistant key storage

2. **Lineage Tracking**
   - Certificate lineage and provenance
   - Audit trail
   - Compliance documentation

3. **Key Management**
   - Centralized key rotation
   - Key derivation hierarchies
   - Multi-tenant key isolation

4. **Attestation**
   - Cryptographic proof of certificate origin
   - Verifiable certificate generation
   - Trust chain validation

---

## 📊 COMPARISON: Before & After

### **Before (rcgen)**
```
rcgen v0.14.6
└── ring v0.17.14 (C code, ABANDONED)
```
- ❌ C dependencies
- ❌ Abandoned `ring` project
- ✅ Works standalone

### **After (Hybrid)**
```
Standalone:
  ed25519-dalek v2.1 (100% Pure Rust)
  └── No C dependencies! ✅

BearDog Enhanced:
  songbird-tls::crypto::BeardogCryptoClient
  └── JSON-RPC to BearDog
      └── HSM-backed, lineage-tracked ✅
```
- ✅ 100% Pure Rust
- ✅ Works standalone
- ✅ Enhanced when BearDog available
- ✅ Automatic fallback
- ✅ Best of both worlds!

---

## 🚀 IMPLEMENTATION TIMELINE

### **Session 1: Core Implementation** (2-3 hours)
- [ ] Create `cert/generator.rs` with hybrid logic
- [ ] Implement standalone Ed25519 generation
- [ ] Add auto-discovery and fallback
- [ ] Write unit tests

### **Session 2: BearDog Integration** (1-2 hours)
- [ ] Extend `BeardogCryptoClient` with cert methods
- [ ] Implement BearDog delegation
- [ ] Add integration tests
- [ ] Test with live BearDog instance

### **Session 3: Integration & Cleanup** (30-60 minutes)
- [ ] Update `CertificateManager` to use hybrid generator
- [ ] Remove `rcgen` dependencies
- [ ] Update documentation
- [ ] Test full flow (standalone + BearDog)

**Total Effort**: **3-6 hours**

---

## 🧪 TESTING STRATEGY

### **Test 1: Standalone Mode**
```rust
#[tokio::test]
async fn test_standalone_no_beardog() {
    let generator = CertificateGenerator::with_mode(CertGenerationMode::Standalone)
        .await
        .unwrap();

    let cert = generator.generate_self_signed("test.local", 365).await.unwrap();
    
    assert!(cert.signature.len() > 0);
    assert_eq!(cert.signature_algorithm, SignatureAlgorithm::Ed25519);
}
```

### **Test 2: BearDog Mode**
```rust
#[tokio::test]
async fn test_beardog_delegation() {
    // Requires running BearDog instance
    let generator = CertificateGenerator::with_mode(
        CertGenerationMode::BearDog { 
            endpoint: "unix:///tmp/beardog.sock".to_string() 
        }
    ).await.unwrap();

    let cert = generator.generate_self_signed("beardog.local", 365).await.unwrap();
    
    // Should have BearDog attestation in extensions
    assert!(cert.has_attestation());
}
```

### **Test 3: Auto Mode with Fallback**
```rust
#[tokio::test]
async fn test_auto_mode_fallback() {
    // Should work even without BearDog
    let generator = CertificateGenerator::new().await.unwrap();
    
    let cert = generator.generate_self_signed("auto.local", 90).await.unwrap();
    
    assert!(cert.signature.len() > 0);
}
```

---

## 📋 MIGRATION CHECKLIST

### **Phase 2a: Standalone Implementation** (2-3 hours)
- [ ] Create `songbird-tls/src/cert/generator.rs`
- [ ] Implement Ed25519-based cert generation
- [ ] Add `CertGenerationMode` enum
- [ ] Implement standalone mode
- [ ] Write unit tests
- [ ] Update `Cargo.toml` dependencies

### **Phase 2b: BearDog Integration** (1-2 hours)
- [ ] Extend `BeardogCryptoClient` with cert methods
- [ ] Implement BearDog delegation mode
- [ ] Add auto-discovery logic
- [ ] Implement fallback mechanism
- [ ] Write integration tests

### **Phase 2c: Integration & Cleanup** (30-60 minutes)
- [ ] Update `CertificateManager`
- [ ] Remove `rcgen` from all Cargo.toml files
- [ ] Test full flow
- [ ] Update documentation
- [ ] Verify zero `ring` dependencies from `rcgen`

---

## 🎯 SUCCESS CRITERIA

✅ **Songbird generates certificates standalone**  
✅ **Zero C dependencies from cert generation**  
✅ **Auto-discovers BearDog when available**  
✅ **Graceful fallback to standalone**  
✅ **All tests passing**  
✅ **Documentation updated**  
✅ **`rcgen` completely removed**

---

## 🎉 RESULT

**After Phase 2**:
- ✅ **Standalone**: Songbird is secure by default
- ✅ **Enhanced**: BearDog provides advanced capabilities
- ✅ **Flexible**: Auto-discovery with fallback
- ✅ **Pure Rust**: Zero C dependencies from cert generation
- ✅ **Production Ready**: Works in all scenarios

**ecoBin Progress**: ~98% → ~98.5% Pure Rust

---

🦀✨ **Best of both worlds: Independence + Collaboration!** ✨🦀

**Status**: Ready to implement  
**Effort**: 3-6 hours  
**Impact**: Removes `rcgen` → `ring` dependency  
**Philosophy**: Songbird stands alone, thrives together

