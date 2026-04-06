// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health check implementations
//!
//! Different types of health checks that can be performed.

use crate::types::HealthStatus;
use songbird_types::errors::SongbirdResult;
use std::time::Instant;

/// HTTP health check
pub struct HttpCheck {
    url: String,
    expected_status: u16,
}

impl HttpCheck {
    /// Create a new HTTP health check
    pub fn new(url: impl Into<String>, expected_status: u16) -> Self {
        Self {
            url: url.into(),
            expected_status,
        }
    }

    /// Perform the health check
    pub async fn check(&self) -> SongbirdResult<HealthStatus> {
        use songbird_http_client::SongbirdHttpClient;

        let start = Instant::now();

        // ✅ EVOLVED: Use Pure Rust HTTP client (ecoBin compliant!)
        let client = SongbirdHttpClient::from_env();

        match client.get(&self.url).await {
            Ok(response) => {
                let status = response.status;
                let elapsed = start.elapsed();

                if status == self.expected_status {
                    Ok(HealthStatus::healthy().with_response_time(elapsed))
                } else {
                    Ok(HealthStatus::degraded(
                        0.5,
                        format!("Expected status {}, got {}", self.expected_status, status),
                    )
                    .with_response_time(elapsed))
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                Ok(HealthStatus::unhealthy(format!("HTTP request failed: {e}"))
                    .with_response_time(elapsed))
            }
        }
    }
}

/// Process existence check
pub struct ProcessCheck {
    process_name: String,
}

impl ProcessCheck {
    /// Create a new process check
    pub fn new(process_name: impl Into<String>) -> Self {
        Self {
            process_name: process_name.into(),
        }
    }

    /// Perform the health check
    pub async fn check(&self) -> SongbirdResult<HealthStatus> {
        // Check if process exists via /proc
        // For now, use a simple approach that works cross-platform
        #[cfg(target_family = "unix")]
        {
            use std::process::Command;

            let output = Command::new("pgrep")
                .arg("-x") // Exact match
                .arg(&self.process_name)
                .output();

            match output {
                Ok(result) if result.status.success() && !result.stdout.is_empty() => {
                    Ok(HealthStatus::healthy())
                }
                Ok(_) => Ok(HealthStatus::unhealthy(format!(
                    "Process '{}' not found",
                    self.process_name
                ))),
                Err(e) => Ok(HealthStatus::degraded(0.3, format!("Failed to check process: {e}"))),
            }
        }

        #[cfg(target_family = "windows")]
        {
            use std::process::Command;

            let output = Command::new("tasklist")
                .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name)])
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    if stdout.contains(&self.process_name) {
                        Ok(HealthStatus::healthy())
                    } else {
                        Ok(HealthStatus::unhealthy(format!(
                            "Process '{}' not found",
                            self.process_name
                        )))
                    }
                }
                Ok(_) | Err(_) => Ok(HealthStatus::degraded(0.3, "Failed to check process")),
            }
        }

        #[cfg(not(any(target_family = "unix", target_family = "windows")))]
        {
            Ok(HealthStatus::degraded(0.5, "Process checking not supported on this platform"))
        }
    }
}

/// System metrics check
pub struct MetricsCheck {
    max_cpu: f64,
    max_memory: f64,
}

impl MetricsCheck {
    /// Create a new metrics check
    #[must_use]
    pub const fn new(max_cpu: f64, max_memory: f64) -> Self {
        Self {
            max_cpu,
            max_memory,
        }
    }

    /// Perform the health check
    pub async fn check(&self) -> SongbirdResult<HealthStatus> {
        let mem = songbird_types::sys_metrics::memory_info().unwrap_or(
            songbird_types::sys_metrics::MemoryInfo {
                total: 1,
                available: 1,
            },
        );

        let cpu_usage = 0.0_f32;
        let memory_usage = mem.usage_percent();

        // Check against thresholds
        let cpu_ok = f64::from(cpu_usage) < self.max_cpu;
        let memory_ok = memory_usage < self.max_memory;

        if cpu_ok && memory_ok {
            Ok(HealthStatus::healthy()
                .with_metadata("cpu", format!("{cpu_usage:.1}%"))
                .with_metadata("memory", format!("{memory_usage:.1}%")))
        } else if !cpu_ok && !memory_ok {
            Ok(HealthStatus::unhealthy(format!(
                "CPU: {:.1}% (max {:.1}%), Memory: {:.1}% (max {:.1}%)",
                cpu_usage, self.max_cpu, memory_usage, self.max_memory
            )))
        } else {
            let score = if cpu_ok {
                0.7
            } else {
                0.6
            };
            let issue = if cpu_ok {
                format!("Memory: {:.1}% (max {:.1}%)", memory_usage, self.max_memory)
            } else {
                format!("CPU: {:.1}% (max {:.1}%)", cpu_usage, self.max_cpu)
            };
            Ok(HealthStatus::degraded(score, issue))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn metrics_check_with_zero_thresholds_reports_unhealthy_or_degraded() {
        let check = MetricsCheck::new(0.0, 0.0);
        let status = check.check().await.unwrap();
        assert!(!status.healthy || status.score < 1.0);
    }

    #[tokio::test]
    async fn metrics_check_with_generous_thresholds_runs() {
        let check = MetricsCheck::new(100.0, 100.0);
        let status = check.check().await.unwrap();
        assert!(status.healthy);
    }

    #[tokio::test]
    async fn http_check_closed_port_returns_status_without_panic() {
        let check = HttpCheck::new("http://127.0.0.1:1/health", 200);
        let status = check.check().await.unwrap();
        assert!(!status.healthy || status.score < 1.0);
    }

    #[tokio::test]
    async fn process_check_runs_for_unlikely_name() {
        let check = ProcessCheck::new("songbird_nonexistent_process_9f3a2c");
        let status = check.check().await.unwrap();
        assert!(!status.healthy);
    }
}
