// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![warn(missing_docs)]

//! Process environment overlay — zero `unsafe`, fully safe Rust.
//!
//! Rust 2024 classifies [`std::env::set_var`] and [`std::env::remove_var`] as `unsafe` because
//! the POSIX process environment is not thread-safe. This crate avoids those APIs entirely by
//! keeping all mutation in a `Mutex`-protected in-memory overlay.
//!
//! ## How it works
//!
//! - [`set_var`] and [`remove_var`] write to an in-memory `HashMap`, never touching the OS
//!   environment.
//! - [`var`], [`var_os`], and [`vars`] consult the overlay first, then fall back to
//!   [`std::env::var`] / [`std::env::var_os`] (which are safe read-only calls).
//!
//! ## Subprocess inheritance
//!
//! Values set **only** in the overlay are **not** visible to child processes spawned via
//! [`std::process::Command`]. Pass them explicitly with [`std::process::Command::env`].
//! Production deployments that rely on a real externally-set environment still work: unset keys
//! in the overlay defer to the OS environment.
//!
//! ## Thread safety
//!
//! All functions in this crate are safe to call from any thread at any time. The overlay is
//! guarded by a `std::sync::Mutex` (not `tokio::sync::Mutex`): the API is entirely synchronous,
//! and the mutex is always released before each function returns—no lock is held across a
//! caller’s `.await`. Short critical sections favor `std::sync::Mutex` (or `parking_lot::Mutex`)
//! over an async mutex.

use std::collections::HashMap;
use std::env::VarError;
use std::ffi::{OsStr, OsString};
use std::sync::{Mutex, OnceLock};

type Overlay = HashMap<String, Option<String>>;

fn overlay() -> &'static Mutex<Overlay> {
    static CELL: OnceLock<Mutex<Overlay>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn key_str(key: &OsStr) -> String {
    key.to_string_lossy().into_owned()
}

/// Set a value in the process environment overlay (thread-safe, zero `unsafe`).
///
/// Does **not** call [`std::env::set_var`].
///
/// # Panics
///
/// Panics if the internal overlay mutex is poisoned (unrecoverable).
#[expect(clippy::expect_used, reason = "mutex poisoning is unrecoverable for the env overlay")]
pub fn set_var(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
    let k = key_str(key.as_ref());
    let v = key_str(value.as_ref());
    overlay().lock().expect("process_env overlay mutex poisoned").insert(k, Some(v));
}

/// Mark a key as removed in the overlay (thread-safe, zero `unsafe`).
///
/// A removed key masks the OS value for [`var`] / [`var_os`] / [`vars`].
/// Does **not** call [`std::env::remove_var`].
///
/// # Panics
///
/// Panics if the internal overlay mutex is poisoned (unrecoverable).
#[expect(clippy::expect_used, reason = "mutex poisoning is unrecoverable for the env overlay")]
pub fn remove_var(key: impl AsRef<OsStr>) {
    let k = key_str(key.as_ref());
    overlay().lock().expect("process_env overlay mutex poisoned").insert(k, None);
}

/// Read an environment variable: overlay first, then [`std::env::var`].
///
/// # Errors
///
/// Returns [`VarError::NotPresent`] when the key is absent from both the overlay and the OS
/// environment. Returns [`VarError::NotUnicode`] when the OS value is not valid UTF-8.
#[inline]
pub fn var(key: impl AsRef<OsStr>) -> Result<String, VarError> {
    var_os(key.as_ref())
        .map_or(Err(VarError::NotPresent), |s| s.into_string().map_err(VarError::NotUnicode))
}

/// Alias for [`var`] — same semantics, matches `std::env::var` naming.
///
/// # Errors
///
/// Same as [`var`].
#[inline]
pub fn get_var(key: impl AsRef<OsStr>) -> Result<String, VarError> {
    var(key)
}

/// Read an environment variable as [`OsString`]: overlay first, then [`std::env::var_os`].
///
/// # Panics
///
/// Panics if the internal overlay mutex is poisoned (unrecoverable).
#[expect(clippy::expect_used, reason = "mutex poisoning is unrecoverable for the env overlay")]
pub fn var_os(key: impl AsRef<OsStr>) -> Option<OsString> {
    let k = key_str(key.as_ref());
    let guard = overlay().lock().expect("process_env overlay mutex poisoned");
    if let Some(opt) = guard.get(&k) {
        return opt.as_ref().map(OsString::from);
    }
    drop(guard);
    std::env::var_os(key)
}

/// Iterate all environment variables: OS vars merged with the overlay.
///
/// Overlay values win over OS values. Keys removed via [`remove_var`] are excluded.
///
/// # Panics
///
/// Panics if the internal overlay mutex is poisoned (unrecoverable).
#[expect(clippy::expect_used, reason = "mutex poisoning is unrecoverable for the env overlay")]
pub fn vars() -> impl Iterator<Item = (String, String)> {
    let snapshot: Overlay = overlay().lock().expect("process_env overlay mutex poisoned").clone();
    let mut combined: HashMap<String, String> = std::env::vars().collect();
    for (k, v_opt) in snapshot {
        match v_opt {
            Some(v) => {
                combined.insert(k, v);
            }
            None => {
                combined.remove(&k);
            }
        }
    }
    combined.into_iter()
}

/// Reset the overlay to empty state. Useful for test isolation.
///
/// After calling this, all reads fall through to the OS environment.
///
/// # Panics
///
/// Panics if the internal overlay mutex is poisoned (unrecoverable).
#[expect(clippy::expect_used, reason = "mutex poisoning is unrecoverable for the env overlay")]
pub fn reset_overlay() {
    overlay().lock().expect("process_env overlay mutex poisoned").clear();
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
#[allow(
    clippy::items_after_statements,
    reason = "test KEY constants declared after lock() guard for clarity"
)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::ffi::{OsStr, OsString};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn set_and_read_overlay() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_SET_READ__";
        set_var(KEY, "hello");
        assert_eq!(var(KEY).unwrap(), "hello");
        remove_var(KEY);
        assert!(var(KEY).is_err() || std::env::var(KEY).is_err());
    }

    #[test]
    fn remove_masks_os_value() {
        let _g = lock();
        const KEY: &str = "PATH";
        assert!(std::env::var(KEY).is_ok(), "PATH should be set by OS");
        remove_var(KEY);
        assert_eq!(var(KEY), Err(VarError::NotPresent));
        overlay().lock().unwrap().remove(KEY);
    }

    #[test]
    fn overlay_wins_over_os() {
        let _g = lock();
        const KEY: &str = "PATH";
        let original = std::env::var(KEY).unwrap();
        set_var(KEY, "overlay-value");
        assert_eq!(var(KEY).unwrap(), "overlay-value");
        overlay().lock().unwrap().remove(KEY);
        assert_eq!(var(KEY).unwrap(), original);
    }

    #[test]
    fn vars_merges_overlay() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_VARS__";
        set_var(KEY, "merged");
        let found = vars().any(|(k, v)| k == KEY && v == "merged");
        assert!(found, "overlay key should appear in vars()");
        remove_var(KEY);
    }

    #[test]
    fn reset_clears_overlay() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_RESET__";
        set_var(KEY, "before_reset");
        assert_eq!(var(KEY).unwrap(), "before_reset");
        reset_overlay();
        assert!(var(KEY).is_err());
    }

    #[test]
    fn get_var_alias() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_GETVAR__";
        set_var(KEY, "via_alias");
        assert_eq!(get_var(KEY).unwrap(), "via_alias");
        remove_var(KEY);
    }

    #[test]
    fn var_os_returns_os_string() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_VAR_OS__";
        set_var(KEY, "os-value");
        let val = var_os(KEY);
        assert_eq!(val.as_deref(), Some(OsStr::new("os-value")));
        remove_var(KEY);
    }

    #[test]
    fn var_os_returns_none_for_missing_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_NEVER_SET_12345__";
        overlay().lock().unwrap().remove(KEY);
        assert!(var_os(KEY).is_none());
    }

    #[test]
    fn var_os_returns_none_for_removed_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_VAR_OS_REMOVED__";
        set_var(KEY, "will-be-removed");
        remove_var(KEY);
        assert!(var_os(KEY).is_none());
    }

    #[test]
    fn vars_excludes_removed_keys() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_VARS_REMOVE__";
        set_var(KEY, "temporary");
        assert!(vars().any(|(k, _)| k == KEY));
        remove_var(KEY);
        assert!(!vars().any(|(k, _)| k == KEY));
    }

    #[test]
    fn last_set_wins() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_LAST_WINS__";
        set_var(KEY, "first");
        set_var(KEY, "second");
        set_var(KEY, "third");
        assert_eq!(var(KEY).unwrap(), "third");
        remove_var(KEY);
    }

    #[test]
    fn set_then_remove_then_set_restores() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_RESTORE__";
        set_var(KEY, "original");
        assert_eq!(var(KEY).unwrap(), "original");
        remove_var(KEY);
        assert!(var(KEY).is_err());
        set_var(KEY, "restored");
        assert_eq!(var(KEY).unwrap(), "restored");
        remove_var(KEY);
    }

    #[test]
    fn empty_string_value_is_valid() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_EMPTY__";
        set_var(KEY, "");
        assert_eq!(var(KEY).unwrap(), "");
        assert_eq!(var_os(KEY).as_deref(), Some(OsStr::new("")));
        remove_var(KEY);
    }

    #[test]
    fn unicode_key_and_value() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_ÜNÏCÖDÉ__";
        set_var(KEY, "Ñoño café ☕");
        assert_eq!(var(KEY).unwrap(), "Ñoño café ☕");
        remove_var(KEY);
    }

    #[test]
    fn reset_overlay_then_os_falls_through() {
        let _g = lock();
        const KEY: &str = "PATH";
        let os_path = std::env::var(KEY).unwrap();
        set_var(KEY, "overlay-path");
        assert_eq!(var(KEY).unwrap(), "overlay-path");
        reset_overlay();
        assert_eq!(var(KEY).unwrap(), os_path);
    }

    #[test]
    fn key_str_handles_regular_strings() {
        let s = key_str(OsStr::new("MY_KEY"));
        assert_eq!(s, "MY_KEY");
    }

    #[test]
    fn overlay_is_singleton() {
        let a = std::ptr::from_ref(overlay());
        let b = std::ptr::from_ref(overlay());
        assert_eq!(a, b, "overlay should return the same static instance");
    }

    #[test]
    fn not_present_error_for_unset_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_DEFINITELY_UNSET_98765__";
        overlay().lock().unwrap().remove(KEY);
        assert_eq!(var(KEY), Err(VarError::NotPresent));
        assert_eq!(get_var(KEY), Err(VarError::NotPresent));
    }

    #[test]
    fn var_unwrap_or_default_on_missing_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_UNWRAP_OR_DEFAULT__";
        overlay().lock().unwrap().remove(KEY);
        assert_eq!(
            var(KEY).unwrap_or_default(),
            String::new(),
            "callers can use Result::unwrap_or_default on var() when a default empty string is acceptable"
        );
    }

    #[test]
    fn vars_includes_os_vars() {
        let _g = lock();
        let has_path = vars().any(|(k, _)| k == "PATH");
        assert!(has_path, "vars() should include OS PATH");
    }

    #[test]
    fn multiple_keys_independent() {
        let _g = lock();
        const A: &str = "__SONGBIRD_PE_TEST_A__";
        const B: &str = "__SONGBIRD_PE_TEST_B__";
        set_var(A, "alpha");
        set_var(B, "beta");
        assert_eq!(var(A).unwrap(), "alpha");
        assert_eq!(var(B).unwrap(), "beta");
        remove_var(A);
        assert!(var(A).is_err());
        assert_eq!(var(B).unwrap(), "beta");
        remove_var(B);
    }

    #[test]
    fn var_with_equals_and_special_chars() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_SPECIAL__";
        set_var(KEY, "val=with=equals&special%chars");
        assert_eq!(var(KEY).unwrap(), "val=with=equals&special%chars");
        remove_var(KEY);
    }

    #[test]
    fn remove_var_idempotent() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_TEST_IDEMPOTENT__";
        remove_var(KEY);
        remove_var(KEY);
        assert!(var(KEY).is_err());
    }

    #[test]
    fn var_os_matches_std_env_when_key_not_in_overlay() {
        let _g = lock();
        const KEY: &str = "PATH";
        overlay().lock().expect("overlay lock").remove(KEY);
        let expected = std::env::var_os(KEY);
        assert_eq!(
            var_os(KEY),
            expected,
            "var_os should mirror std::env::var_os when overlay has no entry for this key"
        );
    }

    #[test]
    fn var_os_empty_overlay_value_round_trip() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_VAR_OS_EMPTY__";
        set_var(KEY, "");
        assert_eq!(
            var_os(KEY).as_deref(),
            Some(OsStr::new("")),
            "empty overlay value should be visible as empty OsStr"
        );
        remove_var(KEY);
    }

    #[test]
    fn get_var_always_matches_var() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_GET_EQ_VAR__";
        assert_eq!(get_var(KEY), var(KEY), "get_var must be an alias of var");
        set_var(KEY, "x");
        assert_eq!(get_var(KEY), var(KEY));
        remove_var(KEY);
        assert_eq!(get_var(KEY), var(KEY));
    }

    #[test]
    fn reset_overlay_is_idempotent() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_RESET_IDEMP__";
        set_var(KEY, "1");
        reset_overlay();
        reset_overlay();
        assert!(var(KEY).is_err(), "double reset should still leave overlay empty for this key");
    }

    #[test]
    fn vars_yields_no_duplicate_keys() {
        let _g = lock();
        let keys: HashSet<String> = vars().map(|(k, _)| k).collect();
        let count = vars().count();
        assert_eq!(keys.len(), count, "vars() iterator must not emit duplicate keys");
    }

    #[test]
    fn overlay_set_var_overrides_os_for_reads_only() {
        let _g = lock();
        const KEY: &str = "PATH";
        let os_snapshot = std::env::var_os(KEY).expect("PATH should exist in test environment");
        set_var(KEY, "overlay-only-read");
        assert_eq!(var_os(KEY).as_deref(), Some(OsStr::new("overlay-only-read")));
        overlay().lock().expect("overlay lock").remove(KEY);
        assert_eq!(
            var_os(KEY),
            Some(os_snapshot),
            "after removing overlay entry, OS value returns"
        );
    }

    #[test]
    fn remove_var_then_var_sees_os_again_for_path() {
        let _g = lock();
        const KEY: &str = "PATH";
        let original = std::env::var(KEY).expect("PATH present");
        set_var(KEY, "temporary-overlay");
        assert_eq!(var(KEY).unwrap(), "temporary-overlay");
        remove_var(KEY);
        assert_eq!(
            var(KEY),
            Err(VarError::NotPresent),
            "remove_var masks OS; var() should not see PATH until overlay entry cleared"
        );
        overlay().lock().expect("overlay lock").remove(KEY);
        assert_eq!(var(KEY).unwrap(), original, "clearing overlay None entry restores OS");
    }

    #[test]
    fn concurrent_independent_keys_many_threads() {
        let _g = lock();
        const N: usize = 64;
        let barrier = Arc::new(Barrier::new(N));
        let mut handles = Vec::with_capacity(N);
        for i in 0..N {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                let key = format!("__SONGBIRD_PE_CONC_IND_{i}__");
                b.wait();
                set_var(&key, format!("v{i}"));
                assert_eq!(
                    var(&key).expect("read after set in same thread"),
                    format!("v{i}"),
                    "thread {i} should read its own overlay value"
                );
                remove_var(&key);
                assert!(var(&key).is_err(), "after remove, key {i} should be absent");
            }));
        }
        for h in handles {
            h.join().expect("thread should not panic");
        }
    }

    #[test]
    fn concurrent_contention_single_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_CONC_SINGLE__";
        overlay().lock().expect("overlay lock").remove(KEY);
        let threads = 32;
        let barrier = Arc::new(Barrier::new(threads + 1));
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(threads);
        for _ in 0..threads {
            let b = Arc::clone(&barrier);
            let c = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                let n = c.fetch_add(1, Ordering::SeqCst);
                b.wait();
                set_var(KEY, format!("writer-{n}"));
                let _ = var(KEY);
            }));
        }
        barrier.wait();
        for h in handles {
            h.join().expect("thread join");
        }
        assert!(
            var(KEY).unwrap().starts_with("writer-"),
            "final value should be from one of the writers"
        );
        remove_var(KEY);
        assert!(var(KEY).is_err());
    }

    #[test]
    fn concurrent_readers_while_writer_updates() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_CONC_RW__";
        set_var(KEY, "start");
        let stop = Arc::new(AtomicUsize::new(0));
        let reader_stop = Arc::clone(&stop);
        let reader = thread::spawn(move || {
            while reader_stop.load(Ordering::Relaxed) < 1000 {
                let _ = var_os(KEY);
            }
        });
        for i in 0..500 {
            set_var(KEY, format!("iter-{i}"));
        }
        stop.store(1000, Ordering::Relaxed);
        reader.join().expect("reader join");
        assert_eq!(var(KEY).unwrap(), "iter-499");
        remove_var(KEY);
    }

    #[test]
    fn vars_overlay_wins_over_duplicate_os_key() {
        let _g = lock();
        const KEY: &str = "__SONGBIRD_PE_VARS_WIN__";
        set_var(KEY, "overlay-wins");
        let from_vars = vars().find(|(k, _)| k == KEY).map(|(_, v)| v);
        assert_eq!(
            from_vars.as_deref(),
            Some("overlay-wins"),
            "merged vars() must prefer overlay over any OS collision"
        );
        remove_var(KEY);
    }

    #[cfg(unix)]
    #[test]
    fn set_var_accepts_non_utf8_os_string_key_lossy() {
        let _g = lock();
        use std::os::unix::ffi::OsStringExt;
        let key = OsString::from_vec(vec![b'F', b'O', b'O', 0xFF]);
        set_var(&key, "bar");
        let k_str = key_str(key.as_os_str());
        assert_eq!(var(&k_str).unwrap(), "bar");
        remove_var(&k_str);
    }
}
