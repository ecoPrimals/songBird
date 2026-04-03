// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `security provider` Secure Tunnel Protocol (BTSP) Interface
//!
//! This module provides the interface for integrating with `security provider`'s genetic
//! cryptography and secure tunnel protocol. It includes:
//!
//! - Trait definitions for BTSP providers
//! - Local implementation for testing without `security provider`
//! - Integration points for real `security provider` connection
//!
//! ## Architecture
//!
//! The BTSP system is designed for sovereignty:
//! - Songbird has self-knowledge only
//! - Discovers `security provider` via capability-based discovery at runtime
//! - Gracefully degrades if `security provider` unavailable
//! - No hardcoded `security provider` dependencies
//!
//! ## Testing
//!
//! Local implementation allows testing federation with encryption without
//! requiring `security provider` to be running. When `security provider` is available, the real
//! provider is discovered and used automatically.

pub mod http_provider;
pub mod local;
pub mod provider;
pub mod tunnel;

pub use local::LocalBtspProvider;
pub use provider::{BtspConfig, BtspProvider};
pub use tunnel::{SecurityContext, Tunnel, TunnelHandle, TunnelStatus};

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::local::LocalBtspProvider;
    use super::provider::{BtspConfig, BtspProvider, DiscoveryMethod, PeerInfo};
    use super::tunnel::{SecurityContext, Tunnel, TunnelHandle, TunnelState};

    #[test]
    fn btsp_config_default_serde_roundtrip() {
        let c = BtspConfig::default();
        assert!(!c.enabled);
        let json = serde_json::to_string(&c).unwrap();
        let back: BtspConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.security_capability, back.security_capability);
        assert_eq!(c.local_fallback, back.local_fallback);
    }

    #[test]
    fn discovery_method_json_stable() {
        let m = DiscoveryMethod::Environment;
        let json = serde_json::to_string(&m).unwrap();
        let back: DiscoveryMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    #[test]
    fn peer_info_serde_roundtrip() {
        let p = PeerInfo {
            id: "p1".to_string(),
            endpoint: "http://localhost:1".to_string(),
            public_key: Some(vec![1, 2]),
            protocols: vec!["btsp".to_string()],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: PeerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(p.id, back.id);
        assert_eq!(p.protocols, back.protocols);
    }

    #[test]
    fn tunnel_handle_with_id_and_security_context_serde() {
        let h = TunnelHandle::with_id("tid".to_string());
        let ctx = SecurityContext {
            tunnel_id: h.id,
            peer_id: "peer".to_string(),
            nonce: Some(vec![0, 1]),
            aad: Some(b"aad".to_vec()),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let back: SecurityContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx.tunnel_id, back.tunnel_id);
    }

    #[test]
    fn tunnel_new_and_status_reflect_state() {
        let t = Tunnel::new("pid".into(), "ep".into(), vec![7, 8]);
        assert_eq!(t.state, TunnelState::Active);
        let st = t.status();
        assert_eq!(st.status, TunnelState::Active);
        assert_eq!(st.peer_id, "pid");
    }

    #[test]
    fn tunnel_state_serde_roundtrip() {
        let s = TunnelState::Degraded;
        let json = serde_json::to_string(&s).unwrap();
        let back: TunnelState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[tokio::test]
    async fn local_provider_establish_tunnel_and_encrypt_roundtrip() {
        let peer = PeerInfo {
            id: "peer-a".to_string(),
            endpoint: "http://127.0.0.1:9".to_string(),
            public_key: None,
            protocols: vec![],
        };
        let provider = LocalBtspProvider::default();
        let handle = provider.establish_tunnel(&peer).await.unwrap();
        let ctx = SecurityContext {
            tunnel_id: handle.id.clone(),
            peer_id: peer.id.clone(),
            nonce: None,
            aad: None,
        };
        let ciphertext = provider.encrypt(b"payload", &ctx).await.unwrap();
        let plain = provider.decrypt(&ciphertext, &ctx).await.unwrap();
        assert_eq!(plain, b"payload");
    }
}
