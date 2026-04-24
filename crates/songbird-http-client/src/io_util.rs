// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared async I/O helpers for JSON-RPC socket communication.

use serde_json::Value;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::time::timeout;

/// Read a complete JSON-RPC response from a stream using chunked reads.
///
/// Many JSON-RPC servers (BearDog, Neural API) keep sockets open for
/// multiple requests, so `read_to_end()` would block forever waiting for
/// EOF.  This helper reads in chunks and returns as soon as a complete
/// JSON value has been received.
///
/// `timeout_per_chunk` applies to each individual `read()` call — it is
/// **not** a wall-clock deadline for the entire response.
pub async fn read_json_response<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    timeout_per_chunk: Duration,
) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 4096];

    loop {
        match timeout(timeout_per_chunk, stream.read(&mut temp_buf)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(n)) => {
                buffer.extend_from_slice(&temp_buf[..n]);
                if let Ok(s) = std::str::from_utf8(&buffer)
                    && serde_json::from_str::<Value>(s).is_ok()
                {
                    break;
                }
            }
            Ok(Err(e)) => return Err(format!("socket read error: {e}")),
            Err(_) => {
                if !buffer.is_empty()
                    && let Ok(s) = std::str::from_utf8(&buffer)
                    && serde_json::from_str::<Value>(s).is_ok()
                {
                    break;
                }
                return Err(format!(
                    "timeout reading response ({}s with no data)",
                    timeout_per_chunk.as_secs()
                ));
            }
        }
    }

    Ok(buffer)
}
