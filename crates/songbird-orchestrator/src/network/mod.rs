// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network Management
//!
//! Intelligent network interface management, binding strategies, and endpoint abstraction.

pub mod binding;
pub mod connectivity_test;
pub(crate) mod route_detect;
pub mod sovereign_socket;

// Re-export commonly used types
pub use connectivity_test::{ConnectivityRemediator, ConnectivityTestResult, ConnectivityTester};
pub use sovereign_socket::{SovereignBinder, SovereignSocket};

pub use binding::NetworkBindingStrategy;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::SocketAddr;
    use std::time::Duration;

    #[test]
    fn network_binding_strategy_variants_distinct() {
        assert_ne!(NetworkBindingStrategy::IPv4All, NetworkBindingStrategy::IPv6All);
        assert_ne!(NetworkBindingStrategy::DualStack, NetworkBindingStrategy::IPv4All);
    }

    #[test]
    fn connectivity_test_result_manual_construct() {
        let addr: SocketAddr = "127.0.0.1:9".parse().unwrap();
        let r = ConnectivityTestResult {
            target: addr,
            tcp_reachable: false,
            https_reachable: false,
            rtt_ms: None,
            error: Some(String::from("unit")),
        };
        assert_eq!(r.target, addr);
        assert!(!r.tcp_reachable);
        assert_eq!(r.error.as_deref(), Some("unit"));
    }

    #[test]
    fn network_binding_and_primary_addr_agree_for_ipv4() {
        let s = NetworkBindingStrategy::IPv4All;
        let primary = s.primary_socket_addr(4321);
        assert_eq!(s.to_socket_addrs(4321), vec![primary]);
    }

    #[tokio::test]
    async fn connectivity_tester_default_matches_new_for_unreachable_port() {
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap();
        let a = ConnectivityTester::with_timeout(Duration::from_millis(200));
        let b = ConnectivityTester::default();
        let ra = a.test_tcp_connectivity(addr).await.unwrap();
        let rb = b.test_tcp_connectivity(addr).await.unwrap();
        assert!(!ra.tcp_reachable);
        assert!(!rb.tcp_reachable);
        assert!(ra.error.is_some());
        assert!(rb.error.is_some());
    }

    #[tokio::test]
    async fn reexported_sovereign_bind_ephemeral_succeeds() {
        let result = SovereignBinder::bind_sovereign(0).await;
        assert!(result.is_ok(), "bind_sovereign(0) should pick an ephemeral port");
        let (_listener, addr) = result.unwrap();
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn reexported_connectivity_tester_tcp_loopback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { while listener.accept().await.is_ok() {} });

        let tester = ConnectivityTester::with_timeout(Duration::from_secs(2));
        let res = tester.test_tcp_connectivity(addr).await.unwrap();
        assert!(res.tcp_reachable);
        assert!(res.error.is_none());
    }
}
