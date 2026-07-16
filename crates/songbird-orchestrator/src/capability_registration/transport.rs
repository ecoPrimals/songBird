// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Platform-agnostic IPC connection to the Neural API.

use songbird_types::IpcStream;

pub(super) async fn connect_platform(path: &str) -> std::io::Result<IpcStream> {
    IpcStream::connect(path).await
}
