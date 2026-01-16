//! Network Connectivity Testing and Auto-Remediation
//!
//! This module provides comprehensive network connectivity testing to ensure
//! Songbird federation can establish connections across different network configurations.
//!
//! # Deep Debt Solution (Dec 20, 2025)
//!
//! Problem: HTTPS server binds successfully but isn't reachable from LAN peers.
//! - Ping works (ICMP) but TCP connections timeout
//! - Even with firewall disabled, connections fail
//! - No visibility into what's blocking the connection
//!
//! Solution: Proactive connectivity testing with auto-remediation
//! - Test both directions (inbound and outbound)
//! - Detect common issues (iptables, routing, MTU)
//! - Auto-configure where possible
//! - Clear diagnostics for manual intervention

use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Network connectivity test result
#[derive(Debug, Clone)]
pub struct ConnectivityTestResult {
    /// Target address tested
    pub target: SocketAddr,
    /// Whether TCP connection succeeded
    pub tcp_reachable: bool,
    /// Whether HTTPS handshake succeeded
    pub https_reachable: bool,
    /// Round-trip time in milliseconds (if successful)
    pub rtt_ms: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Network connectivity tester
pub struct ConnectivityTester {
    /// Timeout for connectivity tests
    test_timeout: Duration,
}

impl ConnectivityTester {
    /// Create a new connectivity tester
    #[must_use]
    pub fn new() -> Self {
        Self {
            test_timeout: Duration::from_secs(5),
        }
    }

    /// Create a connectivity tester with custom timeout
    #[must_use]
    pub fn with_timeout(test_timeout: Duration) -> Self {
        Self {
            test_timeout,
        }
    }

    /// Test TCP connectivity to a target
    pub async fn test_tcp_connectivity(
        &self,
        target: SocketAddr,
    ) -> Result<ConnectivityTestResult> {
        debug!("Testing TCP connectivity to {}", target);

        let start = std::time::Instant::now();

        match timeout(self.test_timeout, TcpStream::connect(target)).await {
            Ok(Ok(_stream)) => {
                let rtt_ms = start.elapsed().as_millis() as u64;
                info!("✅ TCP connection to {} succeeded ({}ms)", target, rtt_ms);

                Ok(ConnectivityTestResult {
                    target,
                    tcp_reachable: true,
                    https_reachable: false, // Will be tested separately
                    rtt_ms: Some(rtt_ms),
                    error: None,
                })
            }
            Ok(Err(e)) => {
                warn!("❌ TCP connection to {} failed: {}", target, e);
                Ok(ConnectivityTestResult {
                    target,
                    tcp_reachable: false,
                    https_reachable: false,
                    rtt_ms: None,
                    error: Some(e.to_string()),
                })
            }
            Err(_) => {
                warn!("❌ TCP connection to {} timed out after {:?}", target, self.test_timeout);
                Ok(ConnectivityTestResult {
                    target,
                    tcp_reachable: false,
                    https_reachable: false,
                    rtt_ms: None,
                    error: Some(format!("Timeout after {:?}", self.test_timeout)),
                })
            }
        }
    }

    /// Test HTTPS connectivity to a target
    pub async fn test_https_connectivity(
        &self,
        target: SocketAddr,
    ) -> Result<ConnectivityTestResult> {
        debug!("Testing HTTPS connectivity to {}", target);

        let start = std::time::Instant::now();

        // Build a permissive HTTPS client for testing (accepts self-signed certs)
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(self.test_timeout)
            .build()
            .map_err(|e| anyhow!("Failed to build HTTPS client: {}", e))?;

        let url = format!("https://{}/health", target);

        match client.get(&url).send().await {
            Ok(response) => {
                let rtt_ms = start.elapsed().as_millis() as u64;
                let status = response.status();

                if status.is_success() {
                    info!(
                        "✅ HTTPS connection to {} succeeded ({}ms, status: {})",
                        target, rtt_ms, status
                    );
                    Ok(ConnectivityTestResult {
                        target,
                        tcp_reachable: true,
                        https_reachable: true,
                        rtt_ms: Some(rtt_ms),
                        error: None,
                    })
                } else {
                    warn!("⚠️  HTTPS connection to {} returned status: {}", target, status);
                    Ok(ConnectivityTestResult {
                        target,
                        tcp_reachable: true,
                        https_reachable: false,
                        rtt_ms: Some(rtt_ms),
                        error: Some(format!("HTTP status: {}", status)),
                    })
                }
            }
            Err(e) => {
                warn!("❌ HTTPS connection to {} failed: {}", target, e);

                // Distinguish between TCP failure and HTTPS failure
                let tcp_reachable = !e.is_connect() && !e.is_timeout();

                Ok(ConnectivityTestResult {
                    target,
                    tcp_reachable,
                    https_reachable: false,
                    rtt_ms: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Run comprehensive connectivity tests
    pub async fn test_comprehensive(&self, target: SocketAddr) -> Result<ConnectivityTestResult> {
        info!("🔍 Running comprehensive connectivity test to {}", target);

        // First test TCP connectivity
        let tcp_result = self.test_tcp_connectivity(target).await?;

        if !tcp_result.tcp_reachable {
            warn!("TCP not reachable, skipping HTTPS test");
            return Ok(tcp_result);
        }

        // If TCP works, test HTTPS
        let https_result = self.test_https_connectivity(target).await?;

        Ok(https_result)
    }

    /// Diagnose connectivity issues
    pub async fn diagnose_connectivity_issues(&self, target: SocketAddr) -> Vec<String> {
        let mut diagnostics = Vec::new();

        info!("🔍 Diagnosing connectivity issues to {}", target);

        // Test TCP first
        let tcp_result = self.test_tcp_connectivity(target).await;

        match tcp_result {
            Ok(result) if !result.tcp_reachable => {
                diagnostics.push("❌ TCP connectivity failed".to_string());

                if let Some(error) = &result.error {
                    if error.contains("timeout") || error.contains("Timeout") {
                        diagnostics.push("  🔍 Possible causes:".to_string());
                        diagnostics
                            .push("     - Firewall blocking port (check iptables/ufw)".to_string());
                        diagnostics.push(
                            "     - Network isolation (VLANs, different subnets)".to_string(),
                        );
                        diagnostics.push("     - Router/switch filtering".to_string());
                        diagnostics.push(
                            "  💡 Try: sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT"
                                .to_string(),
                        );
                    } else if error.contains("refused") {
                        diagnostics
                            .push("  🔍 Connection refused - service not listening".to_string());
                        diagnostics.push("  💡 Check: ss -tlnp | grep 8080".to_string());
                    }
                }
            }
            Ok(result) if result.tcp_reachable => {
                diagnostics.push("✅ TCP connectivity working".to_string());

                // Now test HTTPS
                if let Ok(https_result) = self.test_https_connectivity(target).await {
                    if https_result.https_reachable {
                        diagnostics.push("✅ HTTPS connectivity working".to_string());
                    } else {
                        diagnostics.push("❌ HTTPS handshake failed".to_string());
                        diagnostics.push("  🔍 Possible causes:".to_string());
                        diagnostics.push("     - TLS certificate issues".to_string());
                        diagnostics.push("     - Protocol mismatch (HTTP vs HTTPS)".to_string());
                        diagnostics.push("  💡 Check server logs for TLS errors".to_string());
                    }
                }
            }
            Err(e) => {
                error!("Failed to run diagnostics: {}", e);
                diagnostics.push(format!("❌ Diagnostic error: {}", e));
            }
            _ => {}
        }

        diagnostics
    }
}

impl Default for ConnectivityTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-remediation for common connectivity issues
pub struct ConnectivityRemediator;

impl ConnectivityRemediator {
    /// Attempt to remediate connectivity issues
    ///
    /// # Errors
    ///
    /// Returns an error if remediation fails
    pub async fn attempt_remediation(target: SocketAddr) -> Result<Vec<String>> {
        let mut actions = Vec::new();

        info!("🔧 Attempting auto-remediation for connectivity to {}", target);

        // Check if we're running with sufficient privileges
        if !Self::has_network_admin_privileges() {
            warn!("⚠️  Insufficient network admin capabilities for auto-remediation");
            actions.push("❌ Songbird lacks CAP_NET_ADMIN capability".to_string());
            actions.push(String::new());
            actions.push("🦅 SELF-SOVEREIGN SOLUTION:".to_string());
            actions.push("   Run the network sovereignty setup (one-time):".to_string());
            actions.push("   sudo ./setup-network-sovereignty.sh".to_string());
            actions.push(String::new());
            actions.push("   This grants Songbird:".to_string());
            actions.push("   • CAP_NET_ADMIN: Manage its own firewall rules".to_string());
            actions.push("   • CAP_NET_BIND_SERVICE: Bind to any port".to_string());
            actions.push(String::new());
            actions.push("   After setup, Songbird will:".to_string());
            actions.push("   • Auto-configure firewall on new deployments".to_string());
            actions.push("   • Work without sudo or manual intervention".to_string());
            actions.push("   • Remain sovereign and self-managing".to_string());
            actions.push(String::new());
            actions.push("📝 Manual Alternative (not recommended):".to_string());
            actions.push(
                "   sudo iptables -I INPUT -p tcp --dport {} -j ACCEPT"
                    .to_string()
                    .replace("{}", &target.port().to_string()),
            );
            actions.push("   sudo iptables -I INPUT -p udp --dport 2300 -j ACCEPT".to_string());
            return Ok(actions);
        }

        // Attempt to add iptables rule (Linux only)
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = tokio::process::Command::new("iptables")
                .args([
                    "-C",
                    "INPUT",
                    "-p",
                    "tcp",
                    "--dport",
                    &target.port().to_string(),
                    "-j",
                    "ACCEPT",
                ])
                .output()
                .await
            {
                if output.status.success() {
                    actions.push(format!(
                        "ℹ️  iptables rule already exists for port {}",
                        target.port()
                    ));
                } else {
                    // Rule doesn't exist, try to add it
                    info!("Adding iptables rule for port {}", target.port());

                    match tokio::process::Command::new("iptables")
                        .args([
                            "-I",
                            "INPUT",
                            "-p",
                            "tcp",
                            "--dport",
                            &target.port().to_string(),
                            "-j",
                            "ACCEPT",
                        ])
                        .output()
                        .await
                    {
                        Ok(output) if output.status.success() => {
                            actions.push(format!(
                                "✅ Added iptables ACCEPT rule for port {}",
                                target.port()
                            ));
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            warn!("Failed to add iptables rule: {}", stderr);
                            actions.push(format!("❌ Failed to add iptables rule: {}", stderr));
                        }
                        Err(e) => {
                            warn!("Failed to run iptables: {}", e);
                            actions.push(format!("❌ Failed to run iptables: {}", e));
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            actions.push("ℹ️  Auto-remediation only supported on Linux".to_string());
        }

        Ok(actions)
    }

    /// Check if running with network admin privileges
    fn has_network_admin_privileges() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Check if the binary has CAP_NET_ADMIN capability
            // This allows network configuration without full root
            //
            // To grant this capability:
            //   sudo setcap 'cap_net_admin=+ep' /path/to/songbird-orchestrator
            //
            // Or run the setup-network-sovereignty.sh script

            // Try to execute a benign iptables command to test for capabilities
            // If it succeeds, we have CAP_NET_ADMIN
            std::process::Command::new("iptables")
                .args(["-C", "INPUT", "-j", "ACCEPT"])
                .output()
                .map(|output| output.status.success() || output.status.code() == Some(1)) // 1 = rule not found (but iptables works)
                .unwrap_or(false)
        }

        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connectivity_tester_creation() {
        let tester = ConnectivityTester::new();
        assert_eq!(tester.test_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_connectivity_tester_with_timeout() {
        let tester = ConnectivityTester::with_timeout(Duration::from_secs(10));
        assert_eq!(tester.test_timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_tcp_connectivity_to_localhost() {
        // Bind a test server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server in background
        tokio::spawn(async move {
            while let Ok((_stream, _)) = listener.accept().await {
                // Accept and drop connections
            }
        });

        // Test connectivity
        let tester = ConnectivityTester::new();
        let result = tester.test_tcp_connectivity(addr).await.unwrap();

        assert!(result.tcp_reachable);
        assert!(result.rtt_ms.is_some());
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_tcp_connectivity_to_unreachable() {
        // Try to connect to a port that's not listening
        let addr: SocketAddr = "127.0.0.1:1".parse().unwrap(); // Port 1 typically not listening

        let tester = ConnectivityTester::with_timeout(Duration::from_millis(100));
        let result = tester.test_tcp_connectivity(addr).await.unwrap();

        assert!(!result.tcp_reachable);
        assert!(result.error.is_some());
    }
}
