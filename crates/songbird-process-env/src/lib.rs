// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Safe façade for mutating the process environment.
//!
//! Rust 2024 marks [`std::env::set_var`] and [`std::env::remove_var`] as `unsafe`. The safety
//! contract is unchanged: callers must prevent concurrent reads of the environment from other
//! threads while these run (see standard library documentation).

/// Sets an environment variable for the current process.
///
/// # Safety contract
///
/// Same as [`std::env::set_var`]: no other thread may read the environment concurrently.
pub fn set_var(key: impl AsRef<str>, value: impl AsRef<str>) {
    // SAFETY: Forwarding to `std`; caller must satisfy `std::env::set_var` safety requirements.
    unsafe {
        std::env::set_var(key.as_ref(), value.as_ref());
    }
}

/// Removes an environment variable from the current process.
///
/// # Safety contract
///
/// Same as [`std::env::remove_var`]: no other thread may read the environment concurrently.
pub fn remove_var(key: impl AsRef<str>) {
    // SAFETY: Forwarding to `std`; caller must satisfy `std::env::remove_var` safety requirements.
    unsafe {
        std::env::remove_var(key.as_ref());
    }
}
