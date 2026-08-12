// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::ProcessManager;
use std::env;
use std::fs;
use std::process;

#[test]
fn test_default_pid_file_location() {
    let path = ProcessManager::default_pid_file().expect("default pid path");
    assert!(path.to_string_lossy().contains("songbird"));
    assert!(path.to_string_lossy().ends_with(".pid"));
}

#[test]
fn test_singleton_enforcement() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_test_{}.pid", process::id()));

    let _ = fs::remove_file(&pid_file);

    let manager = ProcessManager::with_pid_file(pid_file);

    let _guard1 = manager.acquire_lock().expect("first lock");

    let result = manager.acquire_lock();
    assert!(result.is_err());

    drop(_guard1);

    let _guard2 = manager.acquire_lock().expect("second lock");
}

#[test]
fn test_stale_pid_cleanup() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_stale_{}.pid", process::id()));

    fs::write(&pid_file, "999999").expect("write pid");

    let manager = ProcessManager::with_pid_file(pid_file);

    let _guard = manager.acquire_lock().expect("stale cleanup");
}

#[test]
fn test_process_running_check() {
    let manager = ProcessManager::new().expect("process manager");

    let current_pid = process::id();
    assert!(manager.is_process_running(current_pid));

    assert!(!manager.is_process_running(999999));
}

#[test]
#[cfg(unix)]
fn test_zombie_detection_logic() {
    let manager = ProcessManager::new().expect("process manager");

    let current_pid = process::id();
    assert!(
        manager.is_process_running(current_pid),
        "Current process should be detected as running"
    );

    assert!(
        !manager.is_process_running(999999),
        "Non-existent PID should not be detected as running"
    );

    let pid_1_exists = fs::read_to_string("/proc/1/stat").is_ok();
    if pid_1_exists {
        assert!(manager.is_process_running(1), "PID 1 (init/systemd) should be running");
    }
}

#[test]
#[cfg(unix)]
fn test_proc_stat_parsing() {
    let stat_running = "12345 (bash) R 1 12345 12345 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234567 8192 100 18446744073709551615";
    let state_pos = stat_running.rfind(')').expect("paren");
    let state = stat_running[state_pos + 2..].chars().next().expect("state");
    assert_eq!(state, 'R', "Should parse running state");

    let stat_sleeping = "12346 (sleep) S 1 12346 12346 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234568 8192 100 18446744073709551615";
    let state_pos = stat_sleeping.rfind(')').expect("paren");
    let state = stat_sleeping[state_pos + 2..].chars().next().expect("state");
    assert_eq!(state, 'S', "Should parse sleeping state");

    let stat_zombie = "12347 (defunct) Z 1 12347 12347 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 1234569 0 0 18446744073709551615";
    let state_pos = stat_zombie.rfind(')').expect("paren");
    let state = stat_zombie[state_pos + 2..].chars().next().expect("state");
    assert_eq!(state, 'Z', "Should parse zombie state");

    let stat_complex = "12348 (my (complex) name!) R 1 12348 12348 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234570 8192 100 18446744073709551615";
    let state_pos = stat_complex.rfind(')').expect("paren");
    let state = stat_complex[state_pos + 2..].chars().next().expect("state");
    assert_eq!(state, 'R', "Should handle complex process names");
}

#[test]
fn singleton_guard_drop_removes_pid_file() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_guard_drop_{}.pid", process::id()));
    let _ = fs::remove_file(&pid_file);

    let manager = ProcessManager::with_pid_file(pid_file.clone());
    let guard = manager.acquire_lock().expect("lock");
    assert!(pid_file.exists(), "PID file should exist while holding guard");
    drop(guard);
    assert!(!pid_file.exists(), "PID file should be removed on guard drop");
}

#[test]
fn test_zombie_allows_new_deployment() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_zombie_test_{}.pid", process::id()));

    let _ = fs::remove_file(&pid_file);

    fs::write(&pid_file, "999999").expect("write pid");

    let manager = ProcessManager::with_pid_file(pid_file);

    let result = manager.acquire_lock();
    assert!(result.is_ok(), "Should acquire lock even with 'zombie' PID file (treats as stale)");
}

#[test]
fn test_helpful_error_messages() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_error_test_{}.pid", process::id()));

    let _ = fs::remove_file(&pid_file);

    let manager = ProcessManager::with_pid_file(pid_file);

    let _guard1 = manager.acquire_lock().expect("first lock");

    let result = manager.acquire_lock();
    assert!(result.is_err());

    let error_msg = format!("{}", result.expect_err("duplicate lock should fail"));
    assert!(
        error_msg.contains("already running") || error_msg.contains("PID"),
        "Error should explain the conflict clearly"
    );
}

#[test]
fn acquire_lock_fails_when_pid_file_not_parseable() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_bad_pid_{}.pid", process::id()));
    let _ = fs::remove_file(&pid_file);
    fs::write(&pid_file, "not-a-valid-pid\n").expect("write bad pid");

    let manager = ProcessManager::with_pid_file(pid_file);
    let result = manager.acquire_lock();
    assert!(result.is_err(), "expected parse error from malformed PID file");
}

#[test]
fn acquire_lock_fails_when_pid_file_empty() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_empty_pid_{}.pid", process::id()));
    let _ = fs::remove_file(&pid_file);
    fs::write(&pid_file, "   \n").expect("write empty");

    let manager = ProcessManager::with_pid_file(pid_file);
    assert!(manager.acquire_lock().is_err());
}

#[test]
fn singleton_guard_debug_is_bounded() {
    let temp_dir = env::temp_dir();
    let pid_file = temp_dir.join(format!("songbird_dbg_{}.pid", process::id()));
    let _ = fs::remove_file(&pid_file);
    let manager = ProcessManager::with_pid_file(pid_file);
    let guard = manager.acquire_lock().expect("lock");
    let s = format!("{guard:?}");
    assert!(s.contains("SingletonGuard"));
    assert!(s.contains("pid_file"));
}
