// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-agnostic IPC connection to the Neural API.

// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;

#[cfg(unix)]
pub(super) async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
pub(super) async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(address).await
}
