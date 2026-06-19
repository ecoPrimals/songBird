// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
//! Health Monitoring Module
//!
//! Comprehensive health monitoring system

#![expect(async_fn_in_trait, reason = "native async HealthMonitor trait")]

use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Health monitor trait for implementing custom health monitoring
pub trait HealthMonitor: Send + Sync {
    /// Get overall health status
    async fn get_health_status(&self) -> Result<HealthStatusDetails>;

    /// Get detailed health information
    async fn get_detailed_health(&self) -> Result<Vec<HealthCheckResult>>;

    /// Set health check thresholds
    async fn set_health_thresholds(&self, thresholds: HealthThresholds) -> Result<()>;
}

/// Coarse tri-state outcome for a single probe or rollup (maps to HTTP-style readiness).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// All checks within configured limits.
    Healthy,
    /// Some checks are slow or soft-failing; service may still accept traffic.
    Degraded,
    /// Hard failure or breach of thresholds; callers should treat as down.
    Unhealthy,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

/// Single named probe outcome, suitable for aggregation into [`HealthStatusDetails`].
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Logical name of the check (e.g. `"database"` or `"disk"`).
    pub name: String,
    /// Outcome for this probe.
    pub status: HealthStatus,
    /// Human-readable detail or error text for operators.
    pub message: String,
    /// Round-trip latency of the probe in milliseconds.
    pub response_time_ms: u64,
}

/// Rollup view combining [`HealthState`], score, and optional JSON metadata for dashboards.
#[derive(Debug, Clone)]
pub struct HealthStatusDetails {
    /// Discrete lifecycle state (includes maintenance vs. unknown).
    pub state: HealthState,
    /// Normalized score in `0.0..=1.0` for sorting or UI gauges.
    pub score: f64,
    /// Count of checks that passed in the last evaluation.
    pub checks_passed: u32,
    /// Count of checks that failed in the last evaluation.
    pub checks_failed: u32,
    /// Wall-clock time of the last successful rollup.
    pub last_updated: std::time::SystemTime,
    /// Arbitrary structured fields (versions, build info, dependency snippets).
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Richer lifecycle state than [`HealthStatus`], including maintenance and unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthState {
    /// Fully within SLO; no action required.
    Healthy,
    /// Elevated risk or partial outage; may still serve degraded traffic.
    Degraded,
    /// Failing checks; do not route new work here without mitigation.
    Unhealthy,
    /// Catastrophic failure or data-loss risk; escalate immediately.
    Critical,
    /// Insufficient data to classify (startup, probes not yet run).
    Unknown,
    /// Intentionally taken out of rotation (drain, upgrade window).
    Maintenance,
}

/// Immutable snapshot of a past evaluation for auditing or trend charts.
pub struct HealthRecord {
    /// When this rollup was recorded.
    pub timestamp: std::time::SystemTime,
    /// Aggregate [`HealthState`] at that time.
    pub status: HealthState,
    /// Individual probe results included in the rollup.
    pub checks: Vec<HealthCheckResult>,
    /// Optional end-to-end latency of the full evaluation pass.
    pub response_time: Option<std::time::Duration>,
}

/// Tunable limits passed to [`HealthMonitor::set_health_thresholds`] for SLO-driven alerts.
#[allow(
    clippy::struct_field_names,
    reason = "threshold field names match external health check schema"
)]
pub struct HealthThresholds {
    /// Maximum acceptable probe latency before marking degraded.
    pub response_time_threshold: std::time::Duration,
    /// Fraction of failed requests (0.0–1.0) that flips status to unhealthy.
    pub error_rate_threshold: f64,
    /// CPU utilization ratio (0.0–1.0) that triggers alerts.
    pub cpu_threshold: f64,
    /// Memory utilization ratio (0.0–1.0) that triggers alerts.
    pub memory_threshold: f64,
    /// Disk utilization ratio (0.0–1.0) that triggers alerts.
    pub disk_threshold: f64,
    /// Consecutive probe failures before escalating to unhealthy.
    pub failure_count_threshold: u32,
}

/// Owns a list of [`HealthProbe`] entries and runs them via [`check_all`](Self::check_all).
pub struct HealthChecker {
    checks: Vec<HealthProbe>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    /// Builds an empty runner; register probes with [`add_check`](Self::add_check).
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Registers a probe; order is preserved when running [`check_all`](Self::check_all).
    pub fn add_check(&mut self, check: HealthProbe) {
        self.checks.push(check);
    }

    /// Runs every registered probe sequentially and collects [`HealthCheckResult`] rows.
    #[must_use]
    pub fn check_all(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        for check in &self.checks {
            match check.run() {
                Ok(result) => results.push(result),
                Err(err) => results.push(HealthCheckResult {
                    name: "Unknown".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: format!("Check failed: {err:?}"),
                    response_time_ms: 0,
                }),
            }
        }

        results
    }
}

/// Statically dispatched health probe for runtime subsystem checks.
#[derive(Clone)]
pub enum HealthProbe {
    /// Verify a TCP endpoint accepts connections within a timeout.
    TcpConnect {
        /// Display name for this probe row.
        name: String,
        /// Target address to connect to.
        addr: std::net::SocketAddr,
        /// Maximum time to wait for connection.
        timeout: std::time::Duration,
    },
    /// Verify a Unix domain socket exists and is connectable.
    #[cfg(unix)]
    UnixSocket {
        /// Display name for this probe row.
        name: String,
        /// Socket path to verify.
        path: std::path::PathBuf,
    },
    /// Check filesystem accessibility (verifies a path is reachable and writable).
    FilesystemAccess {
        /// Display name for this probe row.
        name: String,
        /// Filesystem path to verify accessibility on.
        path: std::path::PathBuf,
    },
    /// Custom probe via a user-supplied closure (captures arbitrary logic).
    Custom {
        /// Display name for this probe row.
        name: String,
        /// Closure that returns `Ok(message)` on success or `Err(message)` on failure.
        check: std::sync::Arc<dyn Fn() -> std::result::Result<String, String> + Send + Sync>,
    },
    /// Passthrough probe for capability-discovered services: calls a JSON-RPC
    /// `health.check` on the given Unix socket.
    #[cfg(unix)]
    JsonRpcHealth {
        /// Display name for this probe row.
        name: String,
        /// Unix socket path of the service to health-check.
        socket_path: std::path::PathBuf,
    },
}

impl HealthProbe {
    /// Run this probe and return a [`HealthCheckResult`] row.
    ///
    /// # Errors
    ///
    /// Returns the probe result; internal failures are captured as `Unhealthy`.
    pub fn run(&self) -> Result<HealthCheckResult> {
        Ok(match self {
            Self::TcpConnect {
                name,
                addr,
                timeout,
            } => Self::timed_probe(name, || {
                std::net::TcpStream::connect_timeout(addr, *timeout)
                    .map(|_| "connected".to_string())
                    .map_err(|e| format!("tcp connect failed: {e}"))
            }),
            #[cfg(unix)]
            Self::UnixSocket {
                name,
                path,
            } => {
                let p = path.clone();
                Self::timed_probe(name, || {
                    std::os::unix::net::UnixStream::connect(&p)
                        .map(|_| "socket reachable".to_string())
                        .map_err(|e| format!("socket unreachable: {e}"))
                })
            }
            Self::FilesystemAccess {
                name,
                path,
            } => {
                let p = path.clone();
                Self::timed_probe(name, || {
                    let accessible = p.exists()
                        && p.metadata().map(|m| !m.permissions().readonly()).unwrap_or(false);
                    if accessible {
                        Ok("path accessible and writable".to_string())
                    } else {
                        Err(format!("path not accessible: {}", p.display()))
                    }
                })
            }
            Self::Custom {
                name,
                check,
            } => Self::timed_probe(name, || check()),
            #[cfg(unix)]
            Self::JsonRpcHealth {
                name,
                socket_path,
            } => {
                let sp = socket_path.clone();
                Self::timed_probe(name, || Self::probe_json_rpc_health(&sp))
            }
        })
    }

    fn timed_probe(
        name: &str,
        f: impl FnOnce() -> std::result::Result<String, String>,
    ) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
        match result {
            Ok(msg) => HealthCheckResult {
                name: name.to_owned(),
                status: HealthStatus::Healthy,
                message: format!("{msg} ({elapsed}ms)"),
                response_time_ms: elapsed,
            },
            Err(msg) => HealthCheckResult {
                name: name.to_owned(),
                status: HealthStatus::Unhealthy,
                message: msg,
                response_time_ms: elapsed,
            },
        }
    }

    #[cfg(unix)]
    fn probe_json_rpc_health(socket_path: &std::path::Path) -> std::result::Result<String, String> {
        use std::io::{Read, Write};
        let mut stream = std::os::unix::net::UnixStream::connect(socket_path)
            .map_err(|e| format!("connect: {e}"))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
        let req = r#"{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}"#;
        stream.write_all(format!("{req}\n").as_bytes()).map_err(|e| format!("write: {e}"))?;
        let mut buf = vec![0u8; 2048];
        let n = stream.read(&mut buf).map_err(|e| format!("read: {e}"))?;
        let resp: serde_json::Value =
            serde_json::from_slice(&buf[..n]).map_err(|e| format!("parse: {e}"))?;
        if let Some(status) = resp["result"]["status"].as_str() {
            Ok(status.to_string())
        } else if let Some(err) = resp["error"]["message"].as_str() {
            Err(err.to_string())
        } else {
            Ok("ok".to_string())
        }
    }
}

#[cfg(test)]
mod probe_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::sync::Arc;

    #[test]
    fn tcp_connect_probe_healthy() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let probe = HealthProbe::TcpConnect {
            name: "test-tcp".to_string(),
            addr,
            timeout: std::time::Duration::from_secs(1),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.contains("connected"));
    }

    #[test]
    fn tcp_connect_probe_unhealthy() {
        let addr: std::net::SocketAddr = "127.0.0.1:1".parse().unwrap();

        let probe = HealthProbe::TcpConnect {
            name: "test-tcp-fail".to_string(),
            addr,
            timeout: std::time::Duration::from_millis(50),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.message.contains("tcp connect failed"));
    }

    #[cfg(unix)]
    #[test]
    fn unix_socket_probe_healthy() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&sock_path).unwrap();

        let probe = HealthProbe::UnixSocket {
            name: "test-uds".to_string(),
            path: sock_path,
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.contains("socket reachable"));
    }

    #[cfg(unix)]
    #[test]
    fn unix_socket_probe_unhealthy() {
        let probe = HealthProbe::UnixSocket {
            name: "test-uds-fail".to_string(),
            path: std::path::PathBuf::from("/tmp/nonexistent-songbird-test-probe.sock"),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.message.contains("socket unreachable"));
    }

    #[test]
    fn filesystem_access_probe_healthy() {
        let dir = tempfile::tempdir().unwrap();
        let probe = HealthProbe::FilesystemAccess {
            name: "test-fs".to_string(),
            path: dir.path().to_path_buf(),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[test]
    fn filesystem_access_probe_missing_path() {
        let probe = HealthProbe::FilesystemAccess {
            name: "test-fs-missing".to_string(),
            path: std::path::PathBuf::from("/nonexistent/songbird/test/path"),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn custom_probe_healthy() {
        let probe = HealthProbe::Custom {
            name: "custom-ok".to_string(),
            check: Arc::new(|| Ok("all good".to_string())),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.contains("all good"));
    }

    #[test]
    fn custom_probe_unhealthy() {
        let probe = HealthProbe::Custom {
            name: "custom-fail".to_string(),
            check: Arc::new(|| Err("something broke".to_string())),
        };

        let result = probe.run().unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert_eq!(result.message, "something broke");
    }

    #[test]
    fn health_checker_aggregates_all_probes() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let mut checker = HealthChecker::new();
        checker.add_check(HealthProbe::TcpConnect {
            name: "tcp".to_string(),
            addr,
            timeout: std::time::Duration::from_secs(1),
        });
        checker.add_check(HealthProbe::Custom {
            name: "custom".to_string(),
            check: Arc::new(|| Ok("ok".to_string())),
        });

        let results = checker.check_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == HealthStatus::Healthy));
    }

    #[test]
    fn health_checker_captures_failures() {
        let mut checker = HealthChecker::new();
        checker.add_check(HealthProbe::Custom {
            name: "pass".to_string(),
            check: Arc::new(|| Ok("fine".to_string())),
        });
        checker.add_check(HealthProbe::Custom {
            name: "fail".to_string(),
            check: Arc::new(|| Err("broken".to_string())),
        });

        let results = checker.check_all();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, HealthStatus::Healthy);
        assert_eq!(results[1].status, HealthStatus::Unhealthy);
    }

    #[test]
    fn probe_measures_response_time() {
        let probe = HealthProbe::Custom {
            name: "timed".to_string(),
            check: Arc::new(|| {
                std::thread::sleep(std::time::Duration::from_millis(10));
                Ok("done".to_string())
            }),
        };

        let result = probe.run().unwrap();
        assert!(result.response_time_ms >= 10);
    }
}
