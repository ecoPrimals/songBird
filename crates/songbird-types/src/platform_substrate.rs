// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! G68 Platform Substrate Abstraction — eliminate silicon deism beyond transport.
//!
//! Three abstraction layers:
//! - **L1 Links**: `platform_link()` — symlink on Unix, junction/hard-link on Windows
//! - **L2 Permissions**: `PlatformAccess` — POSIX mode bits on Unix, ACL-compatible on Windows
//! - **L3 Device backends**: trait-based (domain-specific per primal, not in songbird-types)
//!
//! Reference: sourDough `platform_substrate` module. See `specs/PLATFORM_SUBSTRATE_SPEC.md`.
//!
//! # Philosophy
//!
//! The test: "Does this primal do *less* on Windows, or the *same thing differently*?"
//! If less → silicon deism. If differently → platform abstraction.
//!
//! `#[cfg(unix)]` belongs in this module and the transport layer. Business logic
//! calls these functions and gets the right behavior on any platform.

use std::io;
use std::path::Path;

// ─── L1: Links ─────────────────────────────────────────────────────────────

/// Create a platform-appropriate link from `original` to `link`.
///
/// - **Unix**: Symbolic link (`std::os::unix::fs::symlink`).
/// - **Windows**: Symlink for files/dirs, falling back to hard link.
/// - **Other**: Hard link.
///
/// # Errors
///
/// Returns `io::Error` if link creation fails.
pub fn platform_link(original: &Path, link: &Path) -> io::Result<()> {
    platform_link_impl(original, link)
}

#[cfg(unix)]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    if original.is_dir() {
        std::os::windows::fs::symlink_dir(original, link)
    } else {
        std::os::windows::fs::symlink_file(original, link)
            .or_else(|_| std::fs::hard_link(original, link))
    }
}

#[cfg(not(any(unix, windows)))]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    std::fs::hard_link(original, link)
}

/// Check if a path is a symbolic link (all platforms).
#[must_use]
pub fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).map(|m| m.file_type().is_symlink()).unwrap_or(false)
}

// ─── L2: Permissions ───────────────────────────────────────────────────────

/// Platform-neutral access level for filesystem objects.
///
/// Maps to POSIX mode bits on Unix, and to equivalent access semantics
/// on Windows (where exact mode bits don't exist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAccess {
    /// Owner-only read+write (0o600 on Unix). Sockets, keys, secrets.
    OwnerReadWrite,
    /// Owner+group read+write (0o660 on Unix). Shared sockets.
    GroupReadWrite,
    /// Owner read+write+execute (0o700 on Unix). Private directories.
    OwnerFull,
    /// Owner r+w+x, group+other read+execute (0o755 on Unix). Binaries.
    PublicExecute,
    /// Owner read+write, group+other read (0o644 on Unix). Config files.
    PublicRead,
    /// No access except owner read (0o400 on Unix). Immutable secrets.
    Readonly,
}

impl PlatformAccess {
    /// Apply this access level to the file at `path`.
    ///
    /// On Unix, sets the file mode. On other platforms, best-effort
    /// (e.g., Windows sets readonly attribute for `Readonly`).
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if the permission change fails.
    pub fn apply(self, path: &Path) -> io::Result<()> {
        apply_access_impl(path, self)
    }
}

#[cfg(unix)]
fn apply_access_impl(path: &Path, access: PlatformAccess) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = match access {
        PlatformAccess::OwnerReadWrite => 0o600,
        PlatformAccess::GroupReadWrite => 0o660,
        PlatformAccess::OwnerFull => 0o700,
        PlatformAccess::PublicExecute => 0o755,
        PlatformAccess::PublicRead => 0o644,
        PlatformAccess::Readonly => 0o400,
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn apply_access_impl(path: &Path, access: PlatformAccess) -> io::Result<()> {
    let readonly = matches!(access, PlatformAccess::Readonly);
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_readonly(readonly);
    std::fs::set_permissions(path, perms)
}

/// Check if a path is a Unix socket (platform-aware).
///
/// Returns `true` only on Unix when the file type is a socket.
/// Always `false` on non-Unix (sockets don't exist as file types on Windows).
#[must_use]
pub fn is_unix_socket(path: &Path) -> bool {
    is_unix_socket_impl(path)
}

#[cfg(unix)]
fn is_unix_socket_impl(path: &Path) -> bool {
    use std::os::unix::fs::FileTypeExt;
    std::fs::metadata(path).map(|m| m.file_type().is_socket()).unwrap_or(false)
}

#[cfg(not(unix))]
fn is_unix_socket_impl(_path: &Path) -> bool {
    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn platform_access_apply_owner_rw() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello").unwrap();
        PlatformAccess::OwnerReadWrite.apply(&file).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn platform_access_apply_public_execute() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("binary");
        std::fs::write(&file, "#!/bin/sh").unwrap();
        PlatformAccess::PublicExecute.apply(&file).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }
    }

    #[test]
    fn platform_access_apply_group_rw() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("shared.sock");
        std::fs::write(&file, "").unwrap();
        PlatformAccess::GroupReadWrite.apply(&file).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o660);
        }
    }

    #[test]
    fn platform_link_creates_link() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("original.txt");
        let link = dir.path().join("link.txt");
        std::fs::write(&original, "content").unwrap();
        platform_link(&original, &link).unwrap();
        assert!(link.exists() || is_symlink(&link));
    }

    #[test]
    fn is_symlink_detects_links() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("real.txt");
        let link = dir.path().join("sym.txt");
        std::fs::write(&original, "data").unwrap();
        platform_link(&original, &link).unwrap();
        assert!(is_symlink(&link));
        assert!(!is_symlink(&original));
    }

    #[test]
    fn is_symlink_false_for_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path/12345")));
    }

    #[cfg(unix)]
    #[test]
    fn is_unix_socket_works() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let listener = std::os::unix::net::UnixListener::bind(&sock_path).unwrap();
        assert!(is_unix_socket(&sock_path));
        drop(listener);
    }

    #[test]
    fn is_unix_socket_false_for_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("not_a_socket.txt");
        std::fs::write(&file, "hello").unwrap();
        assert!(!is_unix_socket(&file));
    }

    #[test]
    fn platform_access_apply_owner_full() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("private");
        std::fs::create_dir(&subdir).unwrap();
        PlatformAccess::OwnerFull.apply(&subdir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&subdir).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
    }

    #[test]
    fn platform_access_apply_public_read() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.toml");
        std::fs::write(&file, "key=value").unwrap();
        PlatformAccess::PublicRead.apply(&file).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o644);
        }
    }

    #[test]
    fn platform_access_apply_readonly() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("secret.key");
        std::fs::write(&file, "material").unwrap();
        PlatformAccess::Readonly.apply(&file).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o400);
        }
    }

    #[test]
    fn platform_link_overwrites_existing_symlink_target() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("data.txt");
        let updated = dir.path().join("new.txt");
        let link = dir.path().join("link.txt");
        std::fs::write(&original, "v1").unwrap();
        std::fs::write(&updated, "v2").unwrap();
        platform_link(&original, &link).unwrap();
        std::fs::remove_file(&link).unwrap();
        platform_link(&updated, &link).unwrap();
        assert!(link.exists() || is_symlink(&link));
    }

    #[test]
    fn is_symlink_false_for_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_symlink(dir.path()));
    }
}
