// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Lineage proof — family membership a peer can prove, not only announce.
//!
//! A family name in a discovery tag is a claim anyone on the network can make.
//! This module turns the claim into a proof built from two things the security
//! provider already has:
//!
//! - a **lineage certificate**: the family's signing key signs a device's public
//!   key (`genetic.sign_lineage_certificate`, made once at enrolment), and
//! - a **device key that never leaves the security provider**
//!   (`crypto.sign_ed25519` with `key_id = "lineage"`).
//!
//! A proof is the certificate plus a fresh signature by the device key. The
//! verifier needs no secret: only the family's public key (the *anchor*).
//!
//! ```text
//! message = "songbird-lineage-proof-v1\n" kind "\n" aud "\n" ts "\n" nonce "\n" binding
//! ```
//!
//! - `kind`: `request` (rides with a call) or `challenge` (answers `lineage.prove`)
//! - `aud`: the address the verifier listens on — as dialled by the prover of a
//!   request, as accepted by the prover of a challenge. A proof relayed through
//!   another host carries the wrong address and fails.
//! - `binding`: what the proof is for — a hash of the call, or the challenger's nonce.
//!
//! # Configuration
//!
//! | Variable | Meaning |
//! |---|---|
//! | `SONGBIRD_LINEAGE_ANCHOR` | the family's public key (base64), or a file holding it |
//! | `SONGBIRD_LINEAGE_CERT` | file with this device's lineage certificate (JSON) |
//! | `SONGBIRD_LINEAGE_GATE` | `enforce`, `observe` or `off`; default `enforce` when an anchor is set, else `off` |
//!
//! Neither the anchor nor the certificate is a secret.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// JSON-RPC `params` field that carries a request proof. Removed before dispatch.
pub const PROOF_PARAM: &str = "_lineage_proof";
/// HTTP header that carries a request proof (base64 of the proof JSON) on REST paths.
pub const PROOF_HEADER: &str = "x-lineage-proof";
/// Public method: prove family membership to a challenger.
pub const METHOD_PROVE: &str = "lineage.prove";
/// Public method: what this node's gate is set to.
pub const METHOD_STATUS: &str = "lineage.status";
/// JSON-RPC method by which a device that holds the right signs a certificate for another
/// device. Served to callers on the device itself only.
pub const METHOD_ENROLL: &str = "lineage.enroll";

const DOMAIN: &str = "songbird-lineage-proof-v1";
const CERT_DOMAIN_V2: &str = "songbird-lineage-cert-v2";
/// Most certificates between a device and the family's key, the device's own included.
const MAX_CHAIN: usize = 4;
const MAX_NAME_LEN: usize = 64;
const KEY_ID: &str = "lineage";
const KEY_PURPOSE: &str = "family-proof";
/// How far a proof's clock may be from ours, either way.
const CLOCK_WINDOW_SECS: u64 = 300;
const CERT_CACHE_TTL: Duration = Duration::from_secs(600);
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(5);
const MIN_NONCE_LEN: usize = 16;
const MAX_NONCE_LEN: usize = 128;
const CACHE_PRUNE_AT: usize = 2048;

/// Methods a caller from the network may use without a proof.
const PUBLIC_METHODS: &[&str] = &[
    "health",
    "ping",
    "capabilities",
    "system.capabilities",
    "capabilities.list",
    "capability.list",
    "identity.get",
    "lifecycle.status",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
    METHOD_PROVE,
    METHOD_STATUS,
];

/// Whether `method` is open to callers from the network without a proof.
#[must_use]
pub fn is_public_method(method: &str) -> bool {
    PUBLIC_METHODS.contains(&method) || method.starts_with("health.")
}

/// What the gate does with a caller from the network that brings no proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateMode {
    /// No gate: every caller is served (the behaviour before this module).
    Off,
    /// Serve every caller, but log each call that enforcement would refuse.
    Observe,
    /// Refuse protected calls from the network without a valid proof.
    Enforce,
}

impl GateMode {
    /// Label for logs and `lineage.status`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Observe => "observe",
            Self::Enforce => "enforce",
        }
    }
}

/// What a proof is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofKind {
    /// Rides with a call; bound to the call's content.
    Request,
    /// Answers `lineage.prove`; bound to the challenger's nonce.
    Challenge,
}

impl ProofKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Challenge => "challenge",
        }
    }
}

/// Gate settings, read once from the environment.
#[derive(Debug, Clone)]
pub struct LineageConfig {
    /// What to do with unproven callers from the network.
    pub mode: GateMode,
    /// Our family name (`SONGBIRD_FAMILY_ID`, else `FAMILY_ID`).
    pub family_id: String,
    /// The family's public key, 32 bytes.
    pub anchor: Option<Vec<u8>>,
    /// This device's lineage certificate.
    pub certificate: Option<Value>,
    /// The certificates above ours, nearest first, when a device and not the family's key
    /// signed ours. Empty when the family's key signed it.
    pub chain: Vec<Value>,
    /// A second certificate for this device's key that carries the right to enroll others,
    /// with the certificates above it. Kept apart from `certificate` so that the right can
    /// end without the device itself leaving the family.
    pub enroller: Option<(Value, Vec<Value>)>,
}

impl LineageConfig {
    /// Read the settings from the process environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_lookup(
            |k| songbird_process_env::var(k).ok(),
            |p| std::fs::read_to_string(p).ok(),
        )
    }

    /// Read the settings through the given environment and file readers.
    #[must_use]
    pub fn from_lookup(
        get: impl Fn(&str) -> Option<String>,
        read: impl Fn(&str) -> Option<String>,
    ) -> Self {
        let non_empty = |k: &str| get(k).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());

        let family_id =
            non_empty("SONGBIRD_FAMILY_ID").or_else(|| non_empty("FAMILY_ID")).unwrap_or_default();

        let anchor = non_empty("SONGBIRD_LINEAGE_ANCHOR").and_then(|v| {
            let text = if v.starts_with('/') {
                read(&v)?
            } else {
                v
            };
            let bytes = B64.decode(text.trim()).ok()?;
            (bytes.len() == 32).then_some(bytes)
        });

        // The file holds one certificate, or `{"certificate": .., "chain": [..]}` as
        // `lineage.enroll` returns it.
        let bundle = |key: &str| -> Option<(Value, Vec<Value>)> {
            let v = non_empty(key)
                .and_then(|p| read(&p))
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())?;
            let chain = v
                .get("chain")
                .and_then(Value::as_array)
                .map(|certs| certs.iter().filter(|c| c.is_object()).cloned().collect())
                .unwrap_or_default();
            let certificate = v.get("certificate").filter(|c| c.is_object()).cloned().unwrap_or(v);
            certificate.is_object().then_some((certificate, chain))
        };
        let (certificate, chain) = bundle("SONGBIRD_LINEAGE_CERT")
            .map_or((None, Vec::new()), |(c, chain)| (Some(c), chain));
        let enroller = bundle("SONGBIRD_LINEAGE_ENROLLER_CERT");

        let mode = match non_empty("SONGBIRD_LINEAGE_GATE").map(|v| v.to_lowercase()).as_deref() {
            Some("enforce" | "enforced" | "strict" | "on") => GateMode::Enforce,
            Some("observe" | "permissive" | "log") => GateMode::Observe,
            Some("off" | "disabled" | "false") => GateMode::Off,
            _ if anchor.is_some() => GateMode::Enforce,
            _ => GateMode::Off,
        };

        Self {
            mode,
            family_id,
            anchor,
            certificate,
            chain,
            enroller,
        }
    }
}

/// The three operations a proof needs from the security provider.
pub trait LineageCrypto: Send + Sync {
    /// Sign `message` with this device's lineage key. Returns `(signature, public_key)`, base64.
    fn sign(&self, message: &[u8])
    -> impl Future<Output = Result<(String, String), String>> + Send;

    /// Check an Ed25519 signature (base64 signature and public key).
    fn verify_signature(
        &self,
        message: &[u8],
        signature: &str,
        public_key: &str,
    ) -> impl Future<Output = Result<bool, String>> + Send;

    /// Check that `certificate` carries a valid signature by its parent key for `family_id`.
    fn verify_certificate(
        &self,
        certificate: &Value,
        family_id: &str,
    ) -> impl Future<Output = Result<bool, String>> + Send;
}

/// [`LineageCrypto`] served by the security provider over its local socket.
#[derive(Debug, Clone)]
pub struct ProviderCrypto {
    socket: String,
}

impl ProviderCrypto {
    /// Use the security provider found in the environment.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            socket: songbird_crypto_provider::socket_discovery::discover_security_socket(),
        }
    }

    /// Use the security provider at `socket` (a path, `@name`, or `host:port`).
    #[must_use]
    pub fn at(socket: impl Into<String>) -> Self {
        Self {
            socket: socket.into(),
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let target = self.socket.trim_start_matches("tcp://").trim_start_matches("unix://");
        let io = async {
            let stream = if let Ok(addr) = target.parse::<SocketAddr>() {
                songbird_types::IpcStream::connect_tcp(&addr.ip().to_string(), addr.port()).await
            } else {
                songbird_types::IpcStream::connect(target).await
            }
            .map_err(|e| format!("security provider at {target}: {e}"))?;

            let mut line =
                json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1}).to_string();
            line.push('\n');

            let (reader, mut writer) = tokio::io::split(stream);
            writer.write_all(line.as_bytes()).await.map_err(|e| format!("write: {e}"))?;
            writer.flush().await.map_err(|e| format!("flush: {e}"))?;

            let mut response = String::new();
            BufReader::new(reader)
                .read_line(&mut response)
                .await
                .map_err(|e| format!("read: {e}"))?;
            Ok::<String, String>(response)
        };

        let response = tokio::time::timeout(PROVIDER_TIMEOUT, io)
            .await
            .map_err(|_| format!("security provider did not answer {method} in time"))??;

        let parsed: Value = serde_json::from_str(&response)
            .map_err(|e| format!("security provider answer to {method}: {e}"))?;
        if let Some(error) = parsed.get("error").filter(|e| !e.is_null()) {
            let msg = error.get("message").and_then(Value::as_str).unwrap_or("error");
            return Err(format!("security provider refused {method}: {msg}"));
        }
        parsed.get("result").cloned().ok_or_else(|| format!("no result from {method}"))
    }

    /// This device's lineage public key (base64), without signing anything.
    ///
    /// # Errors
    ///
    /// Returns an error if the security provider cannot be reached.
    pub async fn public_key(&self) -> Result<String, String> {
        let result = self
            .call("crypto.public_key", json!({"key_id": KEY_ID, "purpose": KEY_PURPOSE}))
            .await?;
        result
            .get("public_key")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| String::from("security provider returned no public_key"))
    }
}

impl LineageCrypto for ProviderCrypto {
    async fn sign(&self, message: &[u8]) -> Result<(String, String), String> {
        let result = self
            .call(
                "crypto.sign_ed25519",
                json!({"message": B64.encode(message), "key_id": KEY_ID, "purpose": KEY_PURPOSE}),
            )
            .await?;
        let field = |k: &str| {
            result
                .get(k)
                .and_then(Value::as_str)
                .map(String::from)
                .ok_or_else(|| format!("security provider returned no {k}"))
        };
        Ok((field("signature")?, field("public_key")?))
    }

    async fn verify_signature(
        &self,
        message: &[u8],
        signature: &str,
        public_key: &str,
    ) -> Result<bool, String> {
        let result = self
            .call(
                "crypto.verify_ed25519",
                json!({"message": B64.encode(message), "signature": signature, "public_key": public_key}),
            )
            .await?;
        Ok(result.get("valid").and_then(Value::as_bool).unwrap_or(false))
    }

    async fn verify_certificate(
        &self,
        certificate: &Value,
        family_id: &str,
    ) -> Result<bool, String> {
        let result = self
            .call(
                "genetic.verify_lineage_certificate",
                json!({"certificate": certificate, "expected_family_id": family_id}),
            )
            .await?;
        Ok(result.get("valid").and_then(Value::as_bool).unwrap_or(false))
    }
}

/// A peer whose proof held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPeer {
    /// Device name from the certificate.
    pub device_id: String,
    /// Family name from the certificate (equal to ours).
    pub family_id: String,
    /// The device's lineage public key (base64).
    pub public_key: String,
}

/// What the gate decided about one call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Serve it: the gate is off, the caller is local, or the method is public.
    Allow,
    /// Serve it: the caller proved family membership.
    AllowFamily(VerifiedPeer),
    /// Serve it, but enforcement would have refused it (observe mode). Carries the reason.
    Observed(String),
    /// Refuse it. Carries the reason.
    Deny(String),
}

impl Decision {
    /// Whether the call goes ahead.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        !matches!(self, Self::Deny(_))
    }
}

/// Makes and checks lineage proofs, and decides which calls from the network are served.
#[derive(Debug)]
pub struct LineageGate<C> {
    config: LineageConfig,
    crypto: C,
    seen: Mutex<HashMap<String, Instant>>,
    good_certs: Mutex<HashMap<String, Instant>>,
}

impl<C: LineageCrypto> LineageGate<C> {
    /// Build a gate from settings and a crypto backend.
    #[must_use]
    pub fn new(config: LineageConfig, crypto: C) -> Self {
        Self {
            config,
            crypto,
            seen: Mutex::new(HashMap::new()),
            good_certs: Mutex::new(HashMap::new()),
        }
    }

    /// The gate's mode.
    #[must_use]
    pub const fn mode(&self) -> GateMode {
        self.config.mode
    }

    /// The gate's settings.
    #[must_use]
    pub const fn config(&self) -> &LineageConfig {
        &self.config
    }

    /// The crypto backend.
    #[must_use]
    pub const fn crypto(&self) -> &C {
        &self.crypto
    }

    /// Whether this node can prove its own membership (it has a certificate).
    #[must_use]
    pub const fn can_prove(&self) -> bool {
        self.config.certificate.is_some()
    }

    /// Whether this node can check a peer's membership (it has the family's public key).
    #[must_use]
    pub fn can_verify(&self) -> bool {
        self.config.anchor.is_some() && !self.config.family_id.is_empty()
    }

    /// Answer for `lineage.status`. Nothing in it is secret.
    #[must_use]
    pub fn status(&self) -> Value {
        json!({
            "gate": self.config.mode.as_str(),
            "family_id": self.config.family_id,
            "can_prove": self.can_prove(),
            "can_verify": self.can_verify(),
            "anchor_fingerprint": self.config.anchor.as_deref().map(|a| hex::encode(&Sha256::digest(a)[..8])),
            "device_id": self.config.certificate.as_ref().and_then(|c| c.get("child_device_id")).cloned(),
            "depth": self.config.certificate.as_ref().map(|c| cert_u64(c, "depth")),
            "enroll": self.enroller().map_or(0, |(c, _)| cert_u64(c, "enroll")),
            "enroll_expires_at": self.enroller().map(|(c, _)| cert_u64(c, "expires_at")).filter(|e| *e != 0),
            "expires_at": self.config.certificate.as_ref().map(|c| cert_u64(c, "expires_at")).filter(|e| *e != 0),
        })
    }

    /// Make a proof of our membership.
    ///
    /// # Errors
    ///
    /// Returns an error if no certificate is configured, the security provider
    /// cannot sign, or the certificate was issued for a different key.
    pub async fn make_proof(
        &self,
        kind: ProofKind,
        aud: &str,
        binding: &str,
    ) -> Result<Value, String> {
        let certificate = self
            .config
            .certificate
            .as_ref()
            .ok_or("no lineage certificate on this node (SONGBIRD_LINEAGE_CERT)")?;

        let ts = now_secs();
        let nonce = new_nonce();
        let message = proof_message(kind, aud, ts, &nonce, binding);
        let (signature, public_key) = self.crypto.sign(&message).await?;

        if certificate.get("child_public_key").and_then(Value::as_str) != Some(public_key.as_str())
        {
            return Err(String::from(
                "the lineage certificate was issued for a different key than the security provider signs with",
            ));
        }

        let mut proof = json!({
            "v": 1,
            "kind": kind.as_str(),
            "aud": aud,
            "ts": ts,
            "nonce": nonce,
            "signature": signature,
            "certificate": certificate,
        });
        if !self.config.chain.is_empty() {
            proof["chain"] = Value::Array(self.config.chain.clone());
        }
        Ok(proof)
    }

    /// Check a peer's proof.
    ///
    /// `aud` is the address the proof must name (ours for a request, the one we
    /// dialled for a challenge); `binding` is what the proof must be for.
    ///
    /// # Errors
    ///
    /// Returns the reason the proof does not hold.
    pub async fn verify_proof(
        &self,
        proof: &Value,
        kind: ProofKind,
        aud: &str,
        binding: &str,
    ) -> Result<VerifiedPeer, String> {
        let anchor = self
            .config
            .anchor
            .as_deref()
            .ok_or("no family anchor on this node (SONGBIRD_LINEAGE_ANCHOR)")?;
        if self.config.family_id.is_empty() {
            return Err(String::from("this node has no family"));
        }

        let text = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(String::from);

        if proof.get("v").and_then(Value::as_u64) != Some(1) {
            return Err(String::from("unknown proof version"));
        }
        if text(proof, "kind").as_deref() != Some(kind.as_str()) {
            return Err(String::from("proof is of the wrong kind"));
        }
        if text(proof, "aud").as_deref() != Some(aud) {
            return Err(String::from("proof was made for another address"));
        }
        let ts = proof.get("ts").and_then(Value::as_u64).ok_or("proof has no time")?;
        if now_secs().abs_diff(ts) > CLOCK_WINDOW_SECS {
            return Err(String::from("proof is too old or from the future"));
        }
        let nonce =
            text(proof, "nonce").filter(|n| valid_nonce(n)).ok_or("proof has no usable nonce")?;
        let signature = text(proof, "signature").ok_or("proof has no signature")?;

        let certificate =
            proof.get("certificate").filter(|c| c.is_object()).ok_or("proof has no certificate")?;
        let chain: Vec<Value> = match proof.get("chain") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(certs)) if certs.iter().all(Value::is_object) => certs.clone(),
            Some(_) => return Err(String::from("proof has a malformed chain")),
        };
        let family_id = text(certificate, "family_id").unwrap_or_default();
        let public_key =
            text(certificate, "child_public_key").ok_or("certificate has no device key")?;
        let device_id = text(certificate, "child_device_id").unwrap_or_default();

        self.check_lineage(certificate, &chain, anchor).await?;

        let message = proof_message(kind, aud, ts, &nonce, binding);
        if !self.crypto.verify_signature(&message, &signature, &public_key).await? {
            return Err(String::from("proof signature does not hold"));
        }

        // A proof is good once. (A challenge is bound to our own fresh nonce, so this only bites requests.)
        let replay_key = format!("{public_key}|{nonce}");
        {
            let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
            if seen.contains_key(&replay_key) {
                return Err(String::from("proof was already used"));
            }
            let keep = Duration::from_secs(2 * CLOCK_WINDOW_SECS + 60);
            if seen.len() >= CACHE_PRUNE_AT {
                seen.retain(|_, at| at.elapsed() < keep);
            }
            seen.insert(replay_key, Instant::now());
        }

        Ok(VerifiedPeer {
            device_id,
            family_id,
            public_key,
        })
    }

    /// Check that `leaf`, with the certificates above it (nearest first), leads to the
    /// family's key, and that every device on the way had the right to sign the next.
    async fn check_lineage(
        &self,
        leaf: &Value,
        chain: &[Value],
        anchor: &[u8],
    ) -> Result<(), String> {
        let certs: Vec<&Value> = std::iter::once(leaf).chain(chain.iter()).collect();
        if certs.len() > MAX_CHAIN {
            return Err(String::from("certificate chain is too long"));
        }
        let now = now_secs();
        let top = certs.len() - 1;

        for (i, cert) in certs.iter().enumerate() {
            if cert_text(cert, "family_id") != self.config.family_id {
                return Err(String::from("certificate is for another family"));
            }
            let version = cert_u64(cert, "version");
            if i == top {
                let parent = B64
                    .decode(cert_text(cert, "parent_public_key"))
                    .map_err(|_| "certificate has no parent key")?;
                if parent != anchor {
                    return Err(String::from("certificate was not signed by this family's key"));
                }
                if version == 2 && cert_u64(cert, "depth") != 1 {
                    return Err(String::from("certificate chain has the wrong depth"));
                }
            } else {
                let above = certs[i + 1];
                if version != 2 {
                    return Err(String::from("a certificate signed by a device must be version 2"));
                }
                if cert_u64(above, "version") != 2 || cert_u64(above, "enroll") == 0 {
                    return Err(String::from("the signing device has no right to enroll"));
                }
                if cert_text(cert, "parent_public_key") != cert_text(above, "child_public_key") {
                    return Err(String::from("certificate chain is broken"));
                }
                if cert_u64(cert, "depth") != cert_u64(above, "depth") + 1 {
                    return Err(String::from("certificate chain has the wrong depth"));
                }
                if cert_u64(cert, "enroll") >= cert_u64(above, "enroll") {
                    return Err(String::from("certificate grants more than its signer may"));
                }
            }
            match version {
                1 => {}
                2 => {
                    let expires = cert_u64(cert, "expires_at");
                    if expires != 0 && now >= expires {
                        return Err(String::from("certificate has expired"));
                    }
                    if cert_u64(cert, "issued_at") > now + CLOCK_WINDOW_SECS {
                        return Err(String::from("certificate is from the future"));
                    }
                }
                _ => return Err(String::from("unknown certificate version")),
            }
        }

        // Signatures are the costly part; a chain that held is remembered for a while.
        // Expiry and rights above are checked every time.
        let all = Value::Array(certs.iter().map(|c| (*c).clone()).collect());
        let cert_id = hex::encode(Sha256::digest(canonical_json(&all).as_bytes()));
        if !self.cert_is_cached(&cert_id) {
            for cert in &certs {
                let holds = if cert_u64(cert, "version") == 2 {
                    let message = cert_v2_message(cert)?;
                    self.crypto
                        .verify_signature(
                            &message,
                            cert_text(cert, "parent_signature"),
                            cert_text(cert, "parent_public_key"),
                        )
                        .await?
                } else {
                    self.crypto.verify_certificate(cert, &self.config.family_id).await?
                };
                if !holds {
                    return Err(String::from("certificate signature does not hold"));
                }
            }
            remember(&self.good_certs, cert_id, CERT_CACHE_TTL);
        }
        Ok(())
    }

    /// Sign a certificate for another device (`lineage.enroll`).
    ///
    /// Only a device whose own certificate carries the right (`enroll` of 1 or more, given
    /// by the family's key or by a device above it) can do this. The new certificate can
    /// carry less of that right than ours, never as much, and cannot outlive ours.
    ///
    /// Returns `{"certificate": .., "chain": [..]}`, the file the new device needs.
    ///
    /// # Errors
    ///
    /// Returns the reason no certificate was made.
    pub async fn enroll(&self, params: &Value) -> Result<Value, String> {
        let (own, above) =
            self.enroller().ok_or("this node's certificate carries no right to enroll")?;
        let own_enroll = cert_u64(own, "enroll");
        if above.len() + 2 > MAX_CHAIN {
            return Err(String::from("the chain would be too long"));
        }
        let now = now_secs();
        let own_expires = cert_u64(own, "expires_at");
        if own_expires != 0 && now >= own_expires {
            return Err(String::from("this node's right to enroll has ended"));
        }

        let text = |k: &str| params.get(k).and_then(Value::as_str);
        let child_id = text("child_device_id")
            .filter(|n| valid_name(n))
            .ok_or("lineage.enroll needs child_device_id (letters, digits, - _ .)")?;
        let child_key = text("child_public_key")
            .filter(|k| valid_key(k))
            .ok_or("lineage.enroll needs child_public_key (base64 of 32 bytes)")?;
        let own_key = cert_text(own, "child_public_key");
        if child_key == own_key {
            return Err(String::from("a device cannot enroll itself"));
        }
        let enroll = params.get("enroll").and_then(Value::as_u64).unwrap_or(0);
        if enroll >= own_enroll {
            return Err(String::from("the new certificate cannot carry as much right as ours"));
        }
        let asked = params.get("expires_at").and_then(Value::as_u64).unwrap_or(0);
        let expires_at = match (own_expires, asked) {
            (0, asked) => asked,
            (own, 0) => own,
            (own, asked) => own.min(asked),
        };
        if expires_at != 0 && expires_at <= now {
            return Err(String::from("expires_at is in the past"));
        }

        let mut certificate = json!({
            "version": 2,
            "family_id": self.config.family_id,
            "parent_device_id": cert_text(own, "child_device_id"),
            "parent_public_key": own_key,
            "child_device_id": child_id,
            "child_public_key": child_key,
            "issued_at": now,
            "expires_at": expires_at,
            "depth": cert_u64(own, "depth") + 1,
            "enroll": enroll,
        });
        let message = cert_v2_message(&certificate)?;
        let (signature, public_key) = self.crypto.sign(&message).await?;
        if public_key != own_key {
            return Err(String::from(
                "the lineage certificate was issued for a different key than the security provider signs with",
            ));
        }
        certificate["parent_signature"] = Value::String(signature);

        let mut chain = vec![own.clone()];
        chain.extend(above.iter().cloned());
        Ok(json!({"certificate": certificate, "chain": chain}))
    }

    /// The certificate under which this node may enroll others, with the certificates above
    /// it: the separate enroller certificate when there is one, else our own if it carries
    /// the right.
    fn enroller(&self) -> Option<(&Value, &[Value])> {
        let carries = |c: &Value| cert_u64(c, "version") == 2 && cert_u64(c, "enroll") > 0;
        if let Some((cert, above)) = self.config.enroller.as_ref().filter(|(c, _)| carries(c)) {
            return Some((cert, above.as_slice()));
        }
        let own = self.config.certificate.as_ref().filter(|c| carries(c))?;
        Some((own, self.config.chain.as_slice()))
    }

    fn cert_is_cached(&self, cert_id: &str) -> bool {
        let certs = self.good_certs.lock().unwrap_or_else(PoisonError::into_inner);
        certs.get(cert_id).is_some_and(|at| at.elapsed() < CERT_CACHE_TTL)
    }

    /// Add a request proof to a JSON-RPC request (`{"method": .., "params": ..}`) bound for `aud`.
    ///
    /// Does nothing when this node has no certificate, so a node outside any
    /// family keeps talking to peers that do not ask for proof.
    ///
    /// # Errors
    ///
    /// Returns an error if the request has positional params or the proof cannot be made.
    pub async fn attach_to_rpc(&self, request: &mut Value, aud: &str) -> Result<(), String> {
        if !self.can_prove() {
            return Ok(());
        }
        let method = request.get("method").and_then(Value::as_str).unwrap_or_default().to_string();
        if is_public_method(&method) {
            return Ok(());
        }
        let params = request.get("params").cloned().unwrap_or(Value::Null);
        let mut params = match params {
            Value::Null => json!({}),
            Value::Object(_) => params,
            _ => return Err(String::from("a proof needs named params")),
        };
        if let Some(map) = params.as_object_mut() {
            map.remove(PROOF_PARAM);
        }
        let proof =
            self.make_proof(ProofKind::Request, aud, &rpc_binding(&method, &params)).await?;
        if let Some(map) = params.as_object_mut() {
            map.insert(PROOF_PARAM.to_string(), proof);
        }
        if let Some(req) = request.as_object_mut() {
            req.insert(String::from("params"), params);
        }
        Ok(())
    }

    /// Make the header value that proves an HTTP request to `aud`.
    ///
    /// # Errors
    ///
    /// Returns an error if the proof cannot be made.
    pub async fn http_header(
        &self,
        aud: &str,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<String, String> {
        let proof =
            self.make_proof(ProofKind::Request, aud, &http_binding(method, path, body)).await?;
        Ok(B64.encode(proof.to_string()))
    }

    /// Decide one JSON-RPC call that arrived from `remote` on our address `local`.
    ///
    /// The proof, if any, is taken out of `params` whatever the outcome, so no
    /// handler ever sees it.
    pub async fn authorize_rpc(
        &self,
        remote: SocketAddr,
        local: SocketAddr,
        method: &str,
        params: &mut Value,
    ) -> Decision {
        let proof = params.as_object_mut().and_then(|m| m.remove(PROOF_PARAM));

        if self.config.mode == GateMode::Off || is_local(remote) || is_public_method(method) {
            return Decision::Allow;
        }

        let outcome = match proof {
            Some(proof) => {
                let binding_params = if params.is_null() {
                    json!({})
                } else {
                    params.clone()
                };
                self.verify_proof(
                    &proof,
                    ProofKind::Request,
                    &addr_text(local),
                    &rpc_binding(method, &binding_params),
                )
                .await
            }
            None => {
                Err(String::from("this method needs a lineage proof from callers on the network"))
            }
        };
        self.settle(outcome)
    }

    /// Decide one HTTP request (other than JSON-RPC) that arrived from `remote` on `local`.
    pub async fn authorize_http(
        &self,
        remote: SocketAddr,
        local: SocketAddr,
        header: Option<&str>,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Decision {
        if self.config.mode == GateMode::Off || is_local(remote) {
            return Decision::Allow;
        }
        let outcome = match header.map(|h| {
            B64.decode(h.trim()).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        }) {
            Some(Some(proof)) => {
                self.verify_proof(
                    &proof,
                    ProofKind::Request,
                    &addr_text(local),
                    &http_binding(method, path, body),
                )
                .await
            }
            Some(None) => Err(String::from("the lineage proof header cannot be read")),
            None => {
                Err(String::from("this path needs a lineage proof from callers on the network"))
            }
        };
        self.settle(outcome)
    }

    fn settle(&self, outcome: Result<VerifiedPeer, String>) -> Decision {
        match (outcome, self.config.mode) {
            (Ok(peer), _) => Decision::AllowFamily(peer),
            (Err(reason), GateMode::Enforce) => Decision::Deny(reason),
            (Err(reason), _) => Decision::Observed(reason),
        }
    }

    /// Answer `lineage.prove`: sign the challenger's nonce, naming the address it reached us on.
    ///
    /// # Errors
    ///
    /// Returns an error if the nonce is unusable or the proof cannot be made.
    pub async fn answer_prove(&self, params: &Value, local: SocketAddr) -> Result<Value, String> {
        let nonce = params
            .get("nonce")
            .and_then(Value::as_str)
            .filter(|n| valid_nonce(n))
            .ok_or("lineage.prove needs a nonce of 16 to 128 letters or digits")?;
        let proof = self.make_proof(ProofKind::Challenge, &addr_text(local), nonce).await?;
        Ok(json!({ "proof": proof }))
    }

    /// Check the answer a peer at `dialled` gave to our `lineage.prove` challenge `nonce`.
    ///
    /// # Errors
    ///
    /// Returns the reason the answer does not prove membership.
    pub async fn check_prove_answer(
        &self,
        answer: &Value,
        dialled: SocketAddr,
        nonce: &str,
    ) -> Result<VerifiedPeer, String> {
        let proof = answer.get("proof").ok_or("the peer sent no proof")?;
        self.verify_proof(proof, ProofKind::Challenge, &addr_text(dialled), nonce).await
    }
}

/// The process-wide gate, built from the environment on first use.
pub fn gate() -> &'static Arc<LineageGate<ProviderCrypto>> {
    static GATE: OnceLock<Arc<LineageGate<ProviderCrypto>>> = OnceLock::new();
    GATE.get_or_init(|| {
        Arc::new(LineageGate::new(LineageConfig::from_env(), ProviderCrypto::from_env()))
    })
}

/// A fresh random nonce (32 hex characters).
#[must_use]
pub fn new_nonce() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        // No OS randomness: fall back to a per-process generator so proofs still differ.
        for b in &mut bytes {
            *b = fastrand::u8(..);
        }
    }
    hex::encode(bytes)
}

/// An address as proofs name it: `ip:port`, with an IPv4 address never in its IPv6-mapped form.
#[must_use]
pub fn addr_text(addr: SocketAddr) -> String {
    SocketAddr::new(addr.ip().to_canonical(), addr.port()).to_string()
}

/// Whether a caller is on this host.
#[must_use]
pub fn is_local(remote: SocketAddr) -> bool {
    remote.ip().to_canonical().is_loopback()
}

/// What a request proof on a JSON-RPC call is bound to.
#[must_use]
pub fn rpc_binding(method: &str, params: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(method.as_bytes());
    hasher.update(b"\n");
    hasher.update(canonical_json(params).as_bytes());
    hex::encode(hasher.finalize())
}

/// What a request proof on an HTTP request is bound to.
#[must_use]
pub fn http_binding(method: &str, path: &str, body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(method.to_ascii_uppercase().as_bytes());
    hasher.update(b"\n");
    hasher.update(path.as_bytes());
    hasher.update(b"\n");
    hasher.update(hex::encode(Sha256::digest(body)).as_bytes());
    hex::encode(hasher.finalize())
}

/// JSON with object keys in sorted order at every depth, so both ends hash the same text.
#[must_use]
pub fn canonical_json(value: &Value) -> String {
    fn write(value: &Value, out: &mut String) {
        match value {
            Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort_unstable();
                out.push('{');
                for (i, key) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&Value::String(key.clone()).to_string());
                    out.push(':');
                    if let Some(v) = map.get(key) {
                        write(v, out);
                    }
                }
                out.push('}');
            }
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(item, out);
                }
                out.push(']');
            }
            other => out.push_str(&other.to_string()),
        }
    }
    let mut out = String::new();
    write(value, &mut out);
    out
}

fn proof_message(kind: ProofKind, aud: &str, ts: u64, nonce: &str, binding: &str) -> Vec<u8> {
    format!("{DOMAIN}\n{}\n{aud}\n{ts}\n{nonce}\n{binding}", kind.as_str()).into_bytes()
}

fn cert_u64(cert: &Value, key: &str) -> u64 {
    cert.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn cert_text<'a>(cert: &'a Value, key: &str) -> &'a str {
    cert.get(key).and_then(Value::as_str).unwrap_or_default()
}

/// A family or device name that can stand in a signed line of text.
fn valid_name(name: &str) -> bool {
    (1..=MAX_NAME_LEN).contains(&name.len())
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

fn valid_key(key: &str) -> bool {
    B64.decode(key).is_ok_and(|bytes| bytes.len() == 32)
}

/// The text a version 2 certificate's signature covers: every field that says who was
/// enrolled, by whom, until when, and how much further the right to enroll goes.
///
/// ```text
/// "songbird-lineage-cert-v2\n" family "\n" parent_id "\n" parent_key "\n" child_id "\n"
///     child_key "\n" issued_at "\n" expires_at "\n" depth "\n" enroll
/// ```
///
/// # Errors
///
/// Returns an error for a name or key that cannot be signed unambiguously.
pub fn cert_v2_message(cert: &Value) -> Result<Vec<u8>, String> {
    let family = cert_text(cert, "family_id");
    let parent_id = cert_text(cert, "parent_device_id");
    let child_id = cert_text(cert, "child_device_id");
    let parent_key = cert_text(cert, "parent_public_key");
    let child_key = cert_text(cert, "child_public_key");
    if !(valid_name(family) && valid_name(parent_id) && valid_name(child_id)) {
        return Err(String::from("certificate has a name that cannot be signed"));
    }
    if !(valid_key(parent_key) && valid_key(child_key)) {
        return Err(String::from("certificate has a malformed key"));
    }
    Ok(format!(
        "{CERT_DOMAIN_V2}\n{family}\n{parent_id}\n{parent_key}\n{child_id}\n{child_key}\n{}\n{}\n{}\n{}",
        cert_u64(cert, "issued_at"),
        cert_u64(cert, "expires_at"),
        cert_u64(cert, "depth"),
        cert_u64(cert, "enroll"),
    )
    .into_bytes())
}

fn valid_nonce(nonce: &str) -> bool {
    (MIN_NONCE_LEN..=MAX_NONCE_LEN).contains(&nonce.len())
        && nonce.bytes().all(|b| b.is_ascii_alphanumeric())
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn remember(cache: &Mutex<HashMap<String, Instant>>, key: String, keep: Duration) {
    let mut cache = cache.lock().unwrap_or_else(PoisonError::into_inner);
    if cache.len() >= CACHE_PRUNE_AT {
        cache.retain(|_, at| at.elapsed() < keep);
    }
    cache.insert(key, Instant::now());
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    /// Stand-in for the security provider: a "signature" is a hash of key and message, and
    /// a certificate is "signed" when its `parent_signature` is the hash of its other fields
    /// under the parent's secret. Enough to exercise every decision the gate makes.
    #[derive(Debug, Clone)]
    struct FakeCrypto {
        device_secret: String,
    }

    fn fake_public(secret: &str) -> String {
        B64.encode(Sha256::digest(format!("public:{secret}").as_bytes()))
    }

    fn fake_sign(secret: &str, message: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(fake_public(secret).as_bytes());
        h.update(message);
        B64.encode(h.finalize())
    }

    fn fake_cert(root_secret: &str, device_secret: &str, device: &str, family: &str) -> Value {
        let mut cert = json!({
            "version": 1,
            "parent_device_id": "family-root",
            "child_device_id": device,
            "child_public_key": fake_public(device_secret),
            "family_id": family,
            "issued_at": 1,
            "depth": 1,
            "parent_public_key": fake_public(root_secret),
        });
        let body = canonical_json(&cert);
        cert["parent_signature"] = Value::String(fake_sign(root_secret, body.as_bytes()));
        cert
    }

    impl LineageCrypto for FakeCrypto {
        async fn sign(&self, message: &[u8]) -> Result<(String, String), String> {
            Ok((fake_sign(&self.device_secret, message), fake_public(&self.device_secret)))
        }

        async fn verify_signature(
            &self,
            message: &[u8],
            signature: &str,
            public_key: &str,
        ) -> Result<bool, String> {
            let mut h = Sha256::new();
            h.update(public_key.as_bytes());
            h.update(message);
            Ok(B64.encode(h.finalize()) == signature)
        }

        async fn verify_certificate(
            &self,
            certificate: &Value,
            family_id: &str,
        ) -> Result<bool, String> {
            let mut body = certificate.clone();
            let signature = body.as_object_mut().and_then(|m| m.remove("parent_signature"));
            let parent = certificate["parent_public_key"].as_str().unwrap_or_default();
            let mut h = Sha256::new();
            h.update(parent.as_bytes());
            h.update(canonical_json(&body).as_bytes());
            Ok(certificate["family_id"] == family_id
                && signature.and_then(|s| s.as_str().map(String::from))
                    == Some(B64.encode(h.finalize())))
        }
    }

    const FAMILY: &str = "test-family";

    fn node(
        root: &str,
        device_secret: &str,
        device: &str,
        mode: GateMode,
    ) -> LineageGate<FakeCrypto> {
        LineageGate::new(
            LineageConfig {
                mode,
                family_id: FAMILY.to_string(),
                anchor: Some(B64.decode(fake_public(root)).unwrap()),
                certificate: Some(fake_cert(root, device_secret, device, FAMILY)),
                chain: Vec::new(),
                enroller: None,
            },
            FakeCrypto {
                device_secret: device_secret.to_string(),
            },
        )
    }

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    /// A version 2 certificate signed by `parent_secret` (the root's, or a device's).
    fn cert_v2(
        parent: (&str, &str),
        child: (&str, &str),
        depth: u64,
        enroll: u64,
        expires_at: u64,
    ) -> Value {
        let mut cert = json!({
            "version": 2,
            "family_id": FAMILY,
            "parent_device_id": parent.1,
            "parent_public_key": fake_public(parent.0),
            "child_device_id": child.1,
            "child_public_key": fake_public(child.0),
            "issued_at": now_secs(),
            "expires_at": expires_at,
            "depth": depth,
            "enroll": enroll,
        });
        let message = cert_v2_message(&cert).unwrap();
        cert["parent_signature"] = Value::String(fake_sign(parent.0, &message));
        cert
    }

    const ROOT: (&str, &str) = ("root", "family-root");

    fn node_with(
        device_secret: &str,
        certificate: Value,
        chain: Vec<Value>,
    ) -> LineageGate<FakeCrypto> {
        LineageGate::new(
            LineageConfig {
                mode: GateMode::Enforce,
                family_id: FAMILY.to_string(),
                anchor: Some(B64.decode(fake_public("root")).unwrap()),
                certificate: Some(certificate),
                chain,
                enroller: None,
            },
            FakeCrypto {
                device_secret: device_secret.to_string(),
            },
        )
    }

    fn from_bundle(device_secret: &str, bundle: &Value) -> LineageGate<FakeCrypto> {
        node_with(
            device_secret,
            bundle["certificate"].clone(),
            bundle["chain"].as_array().unwrap().clone(),
        )
    }

    async fn ask(server: &LineageGate<FakeCrypto>, caller: &LineageGate<FakeCrypto>) -> Decision {
        let request = proven_request(caller, "mesh.status", json!({})).await;
        let mut params = request["params"].clone();
        server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut params).await
    }

    fn denied(reason: &str) -> Decision {
        Decision::Deny(String::from(reason))
    }

    #[tokio::test]
    async fn a_device_enrolled_by_a_device_with_the_right_is_served() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0), vec![]);

        let bundle = phone
            .enroll(&json!({"child_device_id": "laptop", "child_public_key": fake_public("lap")}))
            .await
            .unwrap();
        assert_eq!(bundle["certificate"]["depth"], 2);
        assert_eq!(bundle["certificate"]["enroll"], 0);
        assert_eq!(bundle["chain"].as_array().unwrap().len(), 1);

        let laptop = from_bundle("lap", &bundle);
        match ask(&server, &laptop).await {
            Decision::AllowFamily(peer) => assert_eq!(peer.device_id, "laptop"),
            other => panic!("expected family, got {other:?}"),
        }
        // The phone itself, with its version 2 certificate, is served too.
        assert!(matches!(ask(&server, &phone).await, Decision::AllowFamily(_)));
        // And the laptop's door answers a challenge with its chain.
        let answer =
            laptop.answer_prove(&json!({"nonce": "n".repeat(32)}), addr(SERVER)).await.unwrap();
        let peer = server.check_prove_answer(&answer, addr(SERVER), &"n".repeat(32)).await.unwrap();
        assert_eq!(peer.device_id, "laptop");
    }

    #[tokio::test]
    async fn the_right_to_enroll_can_end_without_the_device_leaving_the_family() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let request = json!({"child_device_id": "laptop", "child_public_key": fake_public("lap")});
        // The phone keeps its own certificate (version 1, no end) and holds the right apart.
        let phone_with = |grant: Value| {
            let mut phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);
            phone.config.enroller = Some((grant, Vec::new()));
            phone
        };

        let phone = phone_with(cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, now_secs() + 600));
        let bundle = phone.enroll(&request).await.unwrap();
        // The laptop's chain runs through the grant, not through the phone's own certificate.
        assert_eq!(bundle["chain"][0]["version"], 2);
        assert!(matches!(
            ask(&server, &from_bundle("lap", &bundle)).await,
            Decision::AllowFamily(_)
        ));
        assert_eq!(phone.status()["enroll"], 1);

        // The right has ended: the phone signs nothing new, and is still family itself.
        let phone = phone_with(cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, now_secs() - 10));
        assert_eq!(
            phone.enroll(&request).await.unwrap_err(),
            "this node's right to enroll has ended"
        );
        assert!(matches!(ask(&server, &phone).await, Decision::AllowFamily(_)));

        // A grant made out to another device's key is of no use to this one.
        let phone = phone_with(cert_v2(ROOT, ("other", "otherGate"), 1, 1, 0));
        assert!(phone.enroll(&request).await.is_err());
    }

    #[tokio::test]
    async fn a_device_without_the_right_cannot_enroll() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let child = json!({"child_device_id": "laptop", "child_public_key": fake_public("lap")});

        // A version 1 certificate, as every device has today: no right.
        let v1 = node("root", "pixel", "grapheneGate", GateMode::Enforce);
        assert_eq!(
            v1.enroll(&child).await.unwrap_err(),
            "this node's certificate carries no right to enroll"
        );
        // A version 2 certificate with enroll 0: no right.
        let plain = node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 0, 0), vec![]);
        assert_eq!(
            plain.enroll(&child).await.unwrap_err(),
            "this node's certificate carries no right to enroll"
        );

        // Signing one by hand does not help: the chain says the signer had no right.
        let forged = cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 2, 0, 0);
        let under_v1 = node_with(
            "lap",
            forged.clone(),
            vec![fake_cert("root", "pixel", "grapheneGate", FAMILY)],
        );
        assert_eq!(
            ask(&server, &under_v1).await,
            denied("the signing device has no right to enroll")
        );
        let under_plain =
            node_with("lap", forged, vec![cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 0, 0)]);
        assert_eq!(
            ask(&server, &under_plain).await,
            denied("the signing device has no right to enroll")
        );
    }

    #[tokio::test]
    async fn the_right_does_not_pass_further_than_it_was_given() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0), vec![]);

        // The phone may enroll, but may not hand the right on.
        let err = phone
            .enroll(&json!({"child_device_id": "laptop", "child_public_key": fake_public("lap"), "enroll": 1}))
            .await
            .unwrap_err();
        assert_eq!(err, "the new certificate cannot carry as much right as ours");

        // A laptop enrolled by the phone cannot enroll a tablet, whatever it signs.
        let bundle = phone
            .enroll(&json!({"child_device_id": "laptop", "child_public_key": fake_public("lap")}))
            .await
            .unwrap();
        let laptop = from_bundle("lap", &bundle);
        assert!(
            laptop
                .enroll(
                    &json!({"child_device_id": "tablet", "child_public_key": fake_public("tab")})
                )
                .await
                .is_err()
        );
        let mut chain = vec![bundle["certificate"].clone()];
        chain.extend(bundle["chain"].as_array().unwrap().iter().cloned());
        let tablet =
            node_with("tab", cert_v2(("lap", "laptop"), ("tab", "tablet"), 3, 0, 0), chain);
        assert_eq!(
            ask(&server, &tablet).await,
            denied("the signing device has no right to enroll")
        );

        // A certificate that claims more than its signer had is refused.
        let greedy = node_with(
            "lap",
            cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 2, 1, 0),
            vec![cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0)],
        );
        assert_eq!(
            ask(&server, &greedy).await,
            denied("certificate grants more than its signer may")
        );
    }

    #[tokio::test]
    async fn an_expired_enroller_takes_its_children_with_it() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let past = now_secs() - 10;
        let child = cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 2, 0, 0);

        let laptop = node_with(
            "lap",
            child.clone(),
            vec![cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, past)],
        );
        assert_eq!(ask(&server, &laptop).await, denied("certificate has expired"));

        // The same child under an enroller that is still good is served ...
        let laptop = node_with(
            "lap",
            child,
            vec![cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, now_secs() + 600)],
        );
        assert!(matches!(ask(&server, &laptop).await, Decision::AllowFamily(_)));

        // ... and an expired enroller signs nothing new, and a child cannot outlive it.
        let expired =
            node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, past), vec![]);
        let request = json!({"child_device_id": "laptop", "child_public_key": fake_public("lap")});
        assert_eq!(
            expired.enroll(&request).await.unwrap_err(),
            "this node's right to enroll has ended"
        );
        let soon = now_secs() + 600;
        let phone =
            node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, soon), vec![]);
        let mut long = request.clone();
        long["expires_at"] = json!(soon + 86_400);
        assert_eq!(phone.enroll(&long).await.unwrap()["certificate"]["expires_at"], soon);
        assert_eq!(phone.enroll(&request).await.unwrap()["certificate"]["expires_at"], soon);
    }

    #[tokio::test]
    async fn a_chain_that_was_touched_or_does_not_reach_the_family_is_refused() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let child = cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 2, 0, 0);

        // The enroller's certificate said enroll 0; someone edits it to 1.
        let mut edited = cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 0, 0);
        edited["enroll"] = json!(1);
        let laptop = node_with("lap", child.clone(), vec![edited]);
        assert_eq!(ask(&server, &laptop).await, denied("certificate signature does not hold"));

        // The expiry is signed too.
        let mut extended = cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, now_secs() + 5);
        extended["expires_at"] = json!(now_secs() + 86_400);
        let laptop = node_with("lap", child.clone(), vec![extended]);
        assert_eq!(ask(&server, &laptop).await, denied("certificate signature does not hold"));

        // A chain that ends at another root.
        let other = ("someone-elses-root", "family-root");
        let laptop = node_with(
            "lap",
            child.clone(),
            vec![cert_v2(other, ("pixel", "grapheneGate"), 1, 1, 0)],
        );
        assert_eq!(
            ask(&server, &laptop).await,
            denied("certificate was not signed by this family's key")
        );

        // A chain whose links do not join: signed by a device other than the one above.
        let laptop =
            node_with("lap", child.clone(), vec![cert_v2(ROOT, ("other", "otherGate"), 1, 1, 0)]);
        assert_eq!(ask(&server, &laptop).await, denied("certificate chain is broken"));

        // The wrong depth, and a chain longer than allowed.
        let laptop = node_with(
            "lap",
            cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 5, 0, 0),
            vec![cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0)],
        );
        assert_eq!(ask(&server, &laptop).await, denied("certificate chain has the wrong depth"));
        let top = cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 9, 0);
        let laptop = node_with("lap", child, vec![top.clone(), top.clone(), top.clone(), top]);
        assert_eq!(ask(&server, &laptop).await, denied("certificate chain is too long"));
    }

    #[tokio::test]
    async fn enroll_refuses_names_and_keys_it_cannot_sign() {
        let phone = node_with("pixel", cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0), vec![]);
        for bad in [
            json!({"child_device_id": "two\nlines", "child_public_key": fake_public("lap")}),
            json!({"child_device_id": "", "child_public_key": fake_public("lap")}),
            json!({"child_device_id": "laptop", "child_public_key": "not-a-key"}),
            json!({"child_device_id": "laptop"}),
            json!({"child_device_id": "grapheneGate", "child_public_key": fake_public("pixel")}),
            json!({"child_device_id": "laptop", "child_public_key": fake_public("lap"), "expires_at": 5}),
        ] {
            assert!(phone.enroll(&bad).await.is_err(), "{bad}");
        }
    }

    #[test]
    fn the_certificate_file_may_hold_a_certificate_with_its_chain() {
        let top = cert_v2(ROOT, ("pixel", "grapheneGate"), 1, 1, 0);
        let leaf = cert_v2(("pixel", "grapheneGate"), ("lap", "laptop"), 2, 0, 0);
        let file = json!({"certificate": leaf, "chain": [top]}).to_string();
        let config = LineageConfig::from_lookup(
            |k| match k {
                "SONGBIRD_FAMILY_ID" => Some(String::from(FAMILY)),
                "SONGBIRD_LINEAGE_CERT" => Some(String::from("/cert.json")),
                _ => None,
            },
            |_| Some(file.clone()),
        );
        assert_eq!(config.certificate.unwrap()["child_device_id"], "laptop");
        assert_eq!(config.chain.len(), 1);
        assert_eq!(config.chain[0]["child_device_id"], "grapheneGate");
    }

    const SERVER: &str = "192.168.4.244:7700";
    const CALLER: &str = "192.168.4.50:40000";

    async fn proven_request(from: &LineageGate<FakeCrypto>, method: &str, params: Value) -> Value {
        let mut request = json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1});
        from.attach_to_rpc(&mut request, SERVER).await.unwrap();
        request
    }

    #[tokio::test]
    async fn a_family_member_with_a_proof_is_served() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);

        let request = proven_request(&phone, "mesh.status", json!({"verbose": true})).await;
        let mut params = request["params"].clone();
        let decision =
            server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut params).await;

        match decision {
            Decision::AllowFamily(peer) => {
                assert_eq!(peer.device_id, "grapheneGate");
                assert_eq!(peer.family_id, FAMILY);
            }
            other => panic!("expected family, got {other:?}"),
        }
        assert_eq!(params, json!({"verbose": true}), "the proof never reaches a handler");
    }

    #[tokio::test]
    async fn the_family_name_alone_opens_nothing() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let mut params = json!({"family_id": FAMILY, "tags": [format!("crypto:family:{FAMILY}")]});
        let decision = server
            .authorize_rpc(addr(CALLER), addr(SERVER), "birdsong.get_lineage", &mut params)
            .await;
        assert!(matches!(decision, Decision::Deny(_)), "{decision:?}");
    }

    #[tokio::test]
    async fn a_certificate_from_another_root_is_refused_even_with_our_family_name() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let stranger = node("someone-elses-root", "x", "impostor", GateMode::Enforce);

        let request = proven_request(&stranger, "mesh.status", json!({})).await;
        let mut params = request["params"].clone();
        let decision =
            server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut params).await;
        assert_eq!(
            decision,
            Decision::Deny(String::from("certificate was not signed by this family's key"))
        );
    }

    #[tokio::test]
    async fn a_stolen_certificate_without_the_device_key_is_refused() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);
        // The thief has the phone's certificate (it is public) but signs with its own key.
        let thief = LineageGate::new(
            phone.config().clone(),
            FakeCrypto {
                device_secret: String::from("thief"),
            },
        );
        assert!(
            thief.make_proof(ProofKind::Request, SERVER, "x").await.is_err(),
            "an honest node notices"
        );

        // A dishonest one forges the proof by hand.
        let ts = now_secs();
        let nonce = new_nonce();
        let binding = rpc_binding("mesh.status", &json!({}));
        let message = proof_message(ProofKind::Request, SERVER, ts, &nonce, &binding);
        let forged = json!({"v": 1, "kind": "request", "aud": SERVER, "ts": ts, "nonce": nonce,
            "signature": fake_sign("thief", &message), "certificate": phone.config().certificate});
        let mut params = json!({ PROOF_PARAM: forged });
        let decision =
            server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut params).await;
        assert_eq!(decision, Decision::Deny(String::from("proof signature does not hold")));
    }

    #[tokio::test]
    async fn a_proof_is_bound_to_its_call_and_good_once() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);
        let request = proven_request(&phone, "mesh.status", json!({"a": 1})).await;
        let proof = request["params"][PROOF_PARAM].clone();

        // Same proof, other params.
        let mut other = json!({"a": 2, PROOF_PARAM: proof.clone()});
        let d = server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut other).await;
        assert_eq!(d, Decision::Deny(String::from("proof signature does not hold")));

        // Same proof, other method.
        let mut other = json!({"a": 1, PROOF_PARAM: proof.clone()});
        let d = server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.shutdown", &mut other).await;
        assert!(matches!(d, Decision::Deny(_)));

        // Same proof, arriving at another address of the family.
        let mut same = request["params"].clone();
        let d = server
            .authorize_rpc(addr(CALLER), addr("192.168.4.147:7700"), "mesh.status", &mut same)
            .await;
        assert_eq!(d, Decision::Deny(String::from("proof was made for another address")));

        // The call as sent: served once, refused the second time.
        let mut first = request["params"].clone();
        assert!(matches!(
            server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut first).await,
            Decision::AllowFamily(_)
        ));
        let mut again = request["params"].clone();
        assert_eq!(
            server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut again).await,
            Decision::Deny(String::from("proof was already used"))
        );
    }

    #[tokio::test]
    async fn an_old_proof_is_refused() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let ts = now_secs() - CLOCK_WINDOW_SECS - 5;
        let nonce = new_nonce();
        let binding = rpc_binding("mesh.status", &json!({}));
        let message = proof_message(ProofKind::Request, SERVER, ts, &nonce, &binding);
        let old = json!({"v": 1, "kind": "request", "aud": SERVER, "ts": ts, "nonce": nonce,
            "signature": fake_sign("pixel", &message),
            "certificate": fake_cert("root", "pixel", "grapheneGate", FAMILY)});
        let mut params = json!({ PROOF_PARAM: old });
        let d = server.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut params).await;
        assert_eq!(d, Decision::Deny(String::from("proof is too old or from the future")));
    }

    #[tokio::test]
    async fn local_callers_and_public_methods_need_no_proof() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let mut p = json!({});
        assert_eq!(
            server
                .authorize_rpc(
                    addr("127.0.0.1:5000"),
                    addr("127.0.0.1:7700"),
                    "mesh.status",
                    &mut p
                )
                .await,
            Decision::Allow
        );
        assert_eq!(
            server
                .authorize_rpc(
                    addr("[::ffff:127.0.0.1]:5000"),
                    addr("[::1]:7700"),
                    "mesh.status",
                    &mut p
                )
                .await,
            Decision::Allow
        );
        for method in [
            "health",
            "health.liveness",
            "ping",
            "system.capabilities",
            METHOD_PROVE,
            METHOD_STATUS,
        ] {
            assert_eq!(
                server.authorize_rpc(addr(CALLER), addr(SERVER), method, &mut p).await,
                Decision::Allow
            );
        }
        assert!(!is_public_method("healthy.not"));
        assert!(!is_public_method("birdsong.decrypt"));
    }

    #[tokio::test]
    async fn observe_serves_and_names_what_enforce_would_refuse_and_off_serves_all() {
        let observing = node("root", "east", "eastGate", GateMode::Observe);
        let mut p = json!({});
        assert!(matches!(
            observing.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut p).await,
            Decision::Observed(_)
        ));
        let off = node("root", "east", "eastGate", GateMode::Off);
        let mut p = json!({ PROOF_PARAM: "junk" });
        assert_eq!(
            off.authorize_rpc(addr(CALLER), addr(SERVER), "mesh.status", &mut p).await,
            Decision::Allow
        );
        assert_eq!(p, json!({}), "the proof field is removed even when the gate is off");
    }

    #[tokio::test]
    async fn a_challenge_is_answered_and_a_relayed_answer_fails() {
        let east = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);
        let nonce = new_nonce();

        // eastGate dials the phone at .185 and the phone answers from the socket it accepted on.
        let answer = phone
            .answer_prove(&json!({"nonce": nonce}), addr("[::ffff:192.168.1.185]:7700"))
            .await
            .unwrap();
        let peer =
            east.check_prove_answer(&answer, addr("192.168.1.185:7700"), &nonce).await.unwrap();
        assert_eq!(peer.device_id, "grapheneGate");

        // A host in the middle at .66 passes the challenge on to the phone and the answer back.
        let relayed = east.check_prove_answer(&answer, addr("192.168.1.66:7700"), &nonce).await;
        assert_eq!(relayed, Err(String::from("proof was made for another address")));

        // An answer to someone else's nonce is no answer to ours.
        let other =
            east.check_prove_answer(&answer, addr("192.168.1.185:7700"), &new_nonce()).await;
        assert_eq!(other, Err(String::from("proof signature does not hold")));

        // A challenge answer cannot be passed off as a request proof.
        let mut params = json!({ PROOF_PARAM: answer["proof"] });
        let d = east
            .authorize_rpc(addr(CALLER), addr("192.168.1.185:7700"), "mesh.status", &mut params)
            .await;
        assert_eq!(d, Decision::Deny(String::from("proof is of the wrong kind")));

        assert!(phone.answer_prove(&json!({"nonce": "short"}), addr(SERVER)).await.is_err());
        assert!(
            phone
                .answer_prove(&json!({"nonce": "has spaces in it, and\nnewlines"}), addr(SERVER))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn an_http_request_is_served_with_a_header_proof_and_refused_without() {
        let server = node("root", "east", "eastGate", GateMode::Enforce);
        let phone = node("root", "pixel", "grapheneGate", GateMode::Enforce);
        let body = br#"{"node":"grapheneGate"}"#;
        let header = phone.http_header(SERVER, "post", "/api/federation/join", body).await.unwrap();

        let ok = server
            .authorize_http(
                addr(CALLER),
                addr(SERVER),
                Some(&header),
                "POST",
                "/api/federation/join",
                body,
            )
            .await;
        assert!(matches!(ok, Decision::AllowFamily(_)), "{ok:?}");

        let header = phone.http_header(SERVER, "POST", "/api/federation/join", body).await.unwrap();
        let other_body = server
            .authorize_http(
                addr(CALLER),
                addr(SERVER),
                Some(&header),
                "POST",
                "/api/federation/join",
                b"{}",
            )
            .await;
        assert!(matches!(other_body, Decision::Deny(_)));

        let none = server
            .authorize_http(addr(CALLER), addr(SERVER), None, "GET", "/api/federation/nodes", b"")
            .await;
        assert!(matches!(none, Decision::Deny(_)));
        let junk =
            server.authorize_http(addr(CALLER), addr(SERVER), Some("%%%"), "GET", "/x", b"").await;
        assert_eq!(junk, Decision::Deny(String::from("the lineage proof header cannot be read")));
        let local =
            server.authorize_http(addr("127.0.0.1:1"), addr(SERVER), None, "GET", "/x", b"").await;
        assert_eq!(local, Decision::Allow);
    }

    #[tokio::test]
    async fn a_node_without_a_certificate_sends_calls_unchanged_and_cannot_answer_a_challenge() {
        let bare = LineageGate::new(
            LineageConfig {
                mode: GateMode::Off,
                family_id: String::new(),
                anchor: None,
                certificate: None,
                chain: Vec::new(),
                enroller: None,
            },
            FakeCrypto {
                device_secret: String::from("bare"),
            },
        );
        let mut request = json!({"method": "mesh.status", "params": {"a": 1}});
        bare.attach_to_rpc(&mut request, SERVER).await.unwrap();
        assert_eq!(request, json!({"method": "mesh.status", "params": {"a": 1}}));
        assert!(bare.answer_prove(&json!({"nonce": new_nonce()}), addr(SERVER)).await.is_err());
        assert!(!bare.can_prove());
        assert!(!bare.can_verify());
    }

    #[test]
    fn settings_are_read_from_the_environment() {
        let anchor = fake_public("root");
        let cert = fake_cert("root", "east", "eastGate", FAMILY);
        let cert_file = json!({"certificate": cert, "certificate_id": "abc"}).to_string();
        let env = |pairs: Vec<(&'static str, String)>| {
            move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone())
        };
        let read = |p: &str| (p == "/cert.json").then(|| cert_file.clone());

        let configured = LineageConfig::from_lookup(
            env(vec![
                ("FAMILY_ID", FAMILY.to_string()),
                ("SONGBIRD_LINEAGE_ANCHOR", anchor.clone()),
                ("SONGBIRD_LINEAGE_CERT", String::from("/cert.json")),
            ]),
            read,
        );
        assert_eq!(configured.mode, GateMode::Enforce, "an anchor turns the gate on");
        assert_eq!(configured.family_id, FAMILY);
        assert_eq!(configured.certificate.as_ref().unwrap()["child_device_id"], "eastGate");

        let unconfigured =
            LineageConfig::from_lookup(env(vec![("FAMILY_ID", FAMILY.to_string())]), read);
        assert_eq!(unconfigured.mode, GateMode::Off, "no anchor: as before");
        assert!(unconfigured.anchor.is_none());

        let observing = LineageConfig::from_lookup(
            env(vec![
                ("SONGBIRD_LINEAGE_ANCHOR", anchor),
                ("SONGBIRD_LINEAGE_GATE", String::from("observe")),
            ]),
            read,
        );
        assert_eq!(observing.mode, GateMode::Observe);

        let closed = LineageConfig::from_lookup(
            env(vec![("SONGBIRD_LINEAGE_GATE", String::from("enforce"))]),
            read,
        );
        assert_eq!(
            closed.mode,
            GateMode::Enforce,
            "enforce without an anchor: the network gets public methods only"
        );

        let bad_anchor = LineageConfig::from_lookup(
            env(vec![("SONGBIRD_LINEAGE_ANCHOR", String::from("not-a-key"))]),
            read,
        );
        assert!(bad_anchor.anchor.is_none());
        assert_eq!(bad_anchor.mode, GateMode::Off);
    }

    #[test]
    fn canonical_json_sorts_keys_at_every_depth() {
        let a: Value =
            serde_json::from_str(r#"{"b":{"y":1,"x":[{"k":2,"j":"q\"uote"}]},"a":null}"#).unwrap();
        let b: Value =
            serde_json::from_str(r#"{"a":null,"b":{"x":[{"j":"q\"uote","k":2}],"y":1}}"#).unwrap();
        assert_eq!(canonical_json(&a), canonical_json(&b));
        assert_eq!(canonical_json(&a), r#"{"a":null,"b":{"x":[{"j":"q\"uote","k":2}],"y":1}}"#);
        assert_eq!(rpc_binding("m", &a), rpc_binding("m", &b));
        assert_ne!(rpc_binding("m", &a), rpc_binding("n", &a));
    }

    #[test]
    fn addresses_are_named_the_same_from_both_ends() {
        assert_eq!(addr_text(addr("[::ffff:192.168.4.244]:7700")), "192.168.4.244:7700");
        assert_eq!(addr_text(addr("192.168.4.244:7700")), "192.168.4.244:7700");
        assert!(is_local(addr("[::1]:1")));
        assert!(!is_local(addr("192.168.4.50:1")));
    }
}
