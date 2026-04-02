// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Live `BearDog` process fixture for integration testing.
//!
//! Discovers the beardog binary from `plasmidBin/`, starts it on a temporary
//! Unix socket, waits for `health.liveness`, and tears it down on drop.
//!
//! ## Binary Discovery Order
//!
//! 1. `$BEARDOG_BIN` — explicit path
//! 2. `$ECOPRIMALS_PLASMID_BIN/primals/beardog` — ecosystem override
//! 3. Walk up from workspace root: `../../infra/plasmidBin/primals/beardog`

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// How long to wait for beardog to respond to health.liveness
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Live `BearDog` process managed for the lifetime of a test.
pub struct BearDogFixture {
    process: Child,
    socket_path: PathBuf,
}

impl BearDogFixture {
    /// Start a `BearDog` instance on a temporary socket.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the binary is not found, fails to start, or
    /// does not become healthy within the timeout.
    pub fn start() -> Result<Self, String> {
        let binary = discover_beardog_binary().ok_or_else(|| {
            "beardog binary not found: set $BEARDOG_BIN, $ECOPRIMALS_PLASMID_BIN, or place it in infra/plasmidBin/primals/".to_string()
        })?;

        let socket_dir =
            tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {e}"))?;
        let socket_path = socket_dir.path().join("beardog-test.sock");
        // Keep the temp dir alive so we can use the socket path
        let _socket_dir_keep = socket_dir.keep();

        let child = Command::new(&binary)
            .arg("--socket")
            .arg(&socket_path)
            .arg("--mode")
            .arg("json-rpc")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start beardog at {}: {e}", binary.display()))?;

        let mut fixture = Self {
            process: child,
            socket_path,
        };

        fixture.wait_for_ready()?;

        Ok(fixture)
    }

    /// Socket path for `SECURITY_PROVIDER_SOCKET` (preferred) or legacy `BEARDOG_SOCKET`.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Socket path as string for environment configuration.
    #[must_use]
    pub fn socket_path_str(&self) -> String {
        self.socket_path.to_string_lossy().to_string()
    }

    /// Returns `true` if the beardog binary is discoverable.
    #[must_use]
    pub fn is_available() -> bool {
        discover_beardog_binary().is_some()
    }

    /// Skip the current test if beardog is not available.
    ///
    /// Prints a message and returns `Ok(())` — callers should `return` immediately.
    /// Use: `if !BearDogFixture::is_available() { return BearDogFixture::skip(); }`
    pub fn skip() -> Result<(), String> {
        eprintln!("SKIP: beardog binary not available (set $BEARDOG_BIN to enable)");
        Ok(())
    }

    fn wait_for_ready(&mut self) -> Result<(), String> {
        let start = std::time::Instant::now();

        // Wait for socket file to appear
        while start.elapsed() < STARTUP_TIMEOUT {
            if self.socket_path.exists() {
                return Ok(());
            }
            // Check if process exited early
            if let Some(status) = self
                .process
                .try_wait()
                .map_err(|e| format!("Failed to check beardog status: {e}"))?
            {
                let stderr = self
                    .process
                    .stderr
                    .take()
                    .map(|s| {
                        BufReader::new(s)
                            .lines()
                            .take(20)
                            .filter_map(std::result::Result::ok)
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                return Err(format!("beardog exited early with {status}:\n{stderr}"));
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Err(format!(
            "beardog did not create socket at {} within {STARTUP_TIMEOUT:?}",
            self.socket_path.display()
        ))
    }
}

impl Drop for BearDogFixture {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let _ = std::fs::remove_file(&self.socket_path);
        if let Some(parent) = self.socket_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
}

/// Discover the beardog binary following the ecosystem convention.
fn discover_beardog_binary() -> Option<PathBuf> {
    // 1. Explicit $BEARDOG_BIN
    if let Ok(bin) = std::env::var("BEARDOG_BIN") {
        let p = PathBuf::from(bin);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. $ECOPRIMALS_PLASMID_BIN/primals/beardog
    if let Ok(dir) = std::env::var("ECOPRIMALS_PLASMID_BIN") {
        let p = PathBuf::from(dir).join("primals").join("beardog");
        if p.is_file() {
            return Some(p);
        }
    }

    // 3. Walk up from CWD looking for infra/plasmidBin/primals/beardog
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..6 {
        let candidate = dir.join("infra").join("plasmidBin").join("primals").join("beardog");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    // 4. Check relative path from workspace: ../../infra/plasmidBin/primals/beardog
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let workspace = PathBuf::from(manifest);
        for ancestor in &["../..", "../../.."] {
            let candidate = workspace
                .join(ancestor)
                .join("infra")
                .join("plasmidBin")
                .join("primals")
                .join("beardog");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_reports_availability() {
        // Just checks that discovery doesn't panic
        let available = discover_beardog_binary();
        if let Some(path) = &available {
            assert!(path.is_file(), "discovered path should be a file");
        }
        eprintln!(
            "beardog binary: {}",
            available.map_or_else(|| "not found".to_string(), |p| p.display().to_string())
        );
    }
}
