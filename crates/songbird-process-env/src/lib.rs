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
//! guarded by a `std::sync::Mutex`.

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
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[allow(
    clippy::items_after_statements,
    reason = "test KEY constants declared after lock() guard for clarity"
)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::sync::Mutex;

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
}
