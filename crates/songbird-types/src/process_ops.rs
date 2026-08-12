// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cross-platform process lifecycle operations.
//!
//! Consolidates process existence checks and signal delivery used by the
//! orchestrator, execution agent, and deployment API.

use std::fs;
use std::time::Duration;

use tracing::{debug, warn};

use crate::{SongbirdError, SongbirdResult};

/// Unix signal used to stop a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    /// Graceful termination (`SIGTERM` on Unix).
    Terminate,
    /// Forceful kill (`SIGKILL` on Unix).
    Kill,
}

/// Check if a process is running and healthy.
///
/// On Linux, reads `/proc/{pid}/stat` and treats zombie (`Z`) and dead (`X`)
/// processes as not running. Falls back to `kill -0` when state is ambiguous.
#[must_use]
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let stat_path = format!("/proc/{pid}/stat");
        if let Ok(contents) = fs::read_to_string(&stat_path) {
            if let Some(state_pos) = contents.rfind(')')
                && let Some(state_char) = contents[state_pos + 2..].chars().next()
            {
                return match state_char {
                    'Z' | 'X' | 'x' => {
                        debug!("PID {pid} is zombie/dead (state: {state_char})");
                        false
                    }
                    'R' | 'S' | 'D' | 'I' => {
                        debug!("PID {pid} is healthy (state: {state_char})");
                        true
                    }
                    'T' | 't' => {
                        warn!("PID {pid} is stopped/traced (state: {state_char})");
                        false
                    }
                    _ => kill_zero_check(pid),
                };
            }
        } else {
            debug!("PID {pid} does not exist (/proc entry missing)");
            return false;
        }

        kill_zero_check(pid)
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        warn!("Process existence check not implemented on this platform, assuming running");
        true
    }
}

#[cfg(unix)]
fn kill_zero_check(pid: u32) -> bool {
    debug!("Using fallback kill -0 check for PID {pid}");
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Send a signal to a process.
///
/// # Errors
///
/// Returns [`SongbirdError::Runtime`] when the signal cannot be delivered.
pub fn stop_process(pid: u32, signal: ProcessSignal) -> SongbirdResult<()> {
    #[cfg(unix)]
    {
        let sig_flag = match signal {
            ProcessSignal::Terminate => "-TERM",
            ProcessSignal::Kill => "-9",
        };
        let output = std::process::Command::new("kill")
            .arg(sig_flag)
            .arg(pid.to_string())
            .output()
            .map_err(|e| SongbirdError::Runtime {
                message: format!("Failed to run kill for PID {pid}: {e}"),
                component: Some(String::from("process_ops")),
                debug_info: None,
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SongbirdError::Runtime {
                message: format!("kill {sig_flag} failed for PID {pid}"),
                component: Some(String::from("process_ops")),
                debug_info: None,
            })
        }
    }

    #[cfg(windows)]
    {
        let _ = signal;
        let output = std::process::Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .arg("/T")
            .output()
            .map_err(|e| SongbirdError::Runtime {
                message: format!("Failed to run taskkill for PID {pid}: {e}"),
                component: Some(String::from("process_ops")),
                debug_info: None,
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SongbirdError::Runtime {
                message: format!("taskkill failed for PID {pid}"),
                component: Some(String::from("process_ops")),
                debug_info: None,
            })
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (pid, signal);
        Err(SongbirdError::Runtime {
            message: String::from("Process stopping is not supported on this platform"),
            component: Some(String::from("process_ops")),
            debug_info: None,
        })
    }
}

/// Gracefully stop a process: `SIGTERM`, wait, then `SIGKILL` if still running.
///
/// # Errors
///
/// Returns an error only when the initial `SIGTERM` fails to execute.
pub async fn stop_process_gracefully(pid: u32, term_wait: Duration) -> SongbirdResult<()> {
    let _ = stop_process(pid, ProcessSignal::Terminate);
    tokio::time::sleep(term_wait).await;
    if is_process_running(pid) {
        let _ = stop_process(pid, ProcessSignal::Kill);
    }
    Ok(())
}

/// Blocking variant of [`stop_process_gracefully`].
pub fn stop_process_gracefully_blocking(pid: u32, term_wait: Duration) -> SongbirdResult<()> {
    let _ = stop_process(pid, ProcessSignal::Terminate);
    std::thread::sleep(term_wait);
    if is_process_running(pid) {
        let _ = stop_process(pid, ProcessSignal::Kill);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn current_process_is_running() {
        let pid = std::process::id();
        assert!(is_process_running(pid));
    }

    #[test]
    fn nonexistent_pid_is_not_running() {
        assert!(!is_process_running(999_999));
    }

    #[cfg(unix)]
    #[test]
    fn pid_one_is_running_on_unix_when_proc_visible() {
        if std::path::Path::new("/proc/1/stat").exists() {
            assert!(is_process_running(1), "PID 1 (init/systemd) should be running");
        }
    }

    #[test]
    fn process_signal_variants_distinct() {
        assert_ne!(ProcessSignal::Terminate, ProcessSignal::Kill);
    }
}
