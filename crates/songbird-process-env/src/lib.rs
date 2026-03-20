// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![warn(missing_docs)]
#![deny(unsafe_code)]

//! Safe façade for mutating the process environment.
//!
//! Rust 2024 marks [`std::env::set_var`] and [`std::env::remove_var`] as `unsafe`. The safety
//! contract is unchanged from earlier editions: callers must ensure no other thread reads the
//! environment concurrently while these run (see standard library documentation).
//!
//! ## Serialization of this crate’s callers
//!
//! A process-wide [`parking_lot::Mutex`] serializes [`set_var`] and [`remove_var`] so
//! that code using only this façade does not race with itself. That does **not** remove the
//! standard library’s contract: other threads (or dependencies) may still read the environment
//! without holding this mutex.
//!
//! ## Why not `Mutex` or another “safe” wrapper?
//!
//! The unsafety is not about atomicity of a single update—it is a **process-wide** contract:
//! concurrent readers (any `std::env::var` / `vars` on any thread) while the environment is
//! mutating is undefined behavior. A `Mutex` around these two functions only serializes *our*
//! calls; it does not stop other code (including dependencies) from reading the environment
//! without holding that mutex, so it cannot make the operations sound in the general case.
//! There is no stable, fully safe replacement in `std` for mutating the process environment;
//! these wrappers document the contract and centralize the `unsafe` boundary.

use parking_lot::Mutex;

static ENV_MUTATION: Mutex<()> = Mutex::new(());

/// Sets an environment variable for the current process.
///
/// # Safety contract
///
/// Same as [`std::env::set_var`]: no other thread may read the environment concurrently.
#[expect(unsafe_code, reason = "delegates to std::env::set_var; safety contract documented above")]
pub fn set_var(key: impl AsRef<str>, value: impl AsRef<str>) {
    let _guard = ENV_MUTATION.lock();
    // SAFETY: Delegates to `std::env::set_var`. Callers must satisfy the same invariant as the
    // standard library: no concurrent environment reads on any thread while this runs. This
    // crate cannot enforce that; callers must only invoke from startup or other points where
    // no other threads read `std::env` (or use a process-wide convention that matches). The
    // mutex above serializes other uses of this façade only.
    unsafe {
        std::env::set_var(key.as_ref(), value.as_ref());
    }
}

/// Removes an environment variable from the current process.
///
/// # Safety contract
///
/// Same as [`std::env::remove_var`]: no other thread may read the environment concurrently.
#[expect(
    unsafe_code,
    reason = "delegates to std::env::remove_var; safety contract documented above"
)]
pub fn remove_var(key: impl AsRef<str>) {
    let _guard = ENV_MUTATION.lock();
    // SAFETY: Delegates to `std::env::remove_var`. Same concurrent-read prohibition as
    // [`set_var`]; see that function’s SAFETY comment.
    unsafe {
        std::env::remove_var(key.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::{remove_var, set_var};

    #[test]
    fn set_remove_roundtrip() {
        const KEY: &str = "__SONGBIRD_PROCESS_ENV_TEST__";
        set_var(KEY, "hello");
        assert_eq!(std::env::var(KEY).unwrap(), "hello");
        remove_var(KEY);
        assert!(std::env::var(KEY).is_err());
    }
}
