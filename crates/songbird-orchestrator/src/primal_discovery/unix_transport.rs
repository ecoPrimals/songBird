// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Line-delimited JSON-RPC over Unix stream sockets (`health.liveness`, `capabilities.list`).

use std::io::{Read, Write};
use std::path::Path;

use super::parse::parse_capabilities_result;

/// `health.liveness` then `capabilities.list` / `capability.list`; returns flat token list.
pub(super) fn probe_capabilities_list(path: &Path) -> Option<Vec<String>> {
    let mut stream = std::os::unix::net::UnixStream::connect(path).ok()?;
    stream.set_read_timeout(Some(songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT)).ok()?;

    let liveness_ok = jsonrpc_request_response(&mut stream, "health.liveness", 1).is_ok()
        || jsonrpc_request_response_raw(&mut stream, "ping", 11).is_ok();
    if !liveness_ok {
        return None;
    }

    let caps_resp = jsonrpc_request_response(&mut stream, "capabilities.list", 2)
        .or_else(|_| jsonrpc_request_response(&mut stream, "capability.list", 3))
        .ok()?;

    parse_capabilities_result(&caps_resp)
}

fn jsonrpc_request_response_raw(
    stream: &mut std::os::unix::net::UnixStream,
    method: &str,
    id: i64,
) -> Result<serde_json::Value, std::io::Error> {
    jsonrpc_request_response_inner(stream, method, id)
}

fn jsonrpc_request_response(
    stream: &mut std::os::unix::net::UnixStream,
    method: &str,
    id: i64,
) -> Result<serde_json::Value, std::io::Error> {
    let method = songbird_types::normalize_json_rpc_method_name(method);
    jsonrpc_request_response_inner(stream, method, id)
}

fn jsonrpc_request_response_inner(
    stream: &mut std::os::unix::net::UnixStream,
    method: &str,
    id: i64,
) -> Result<serde_json::Value, std::io::Error> {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": serde_json::json!({}),
        "id": id,
    });
    let mut bytes = serde_json::to_vec(&req)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    bytes.push(b'\n');
    stream.write_all(&bytes)?;
    let line = read_line(stream)?;
    let v: serde_json::Value = serde_json::from_str(line.trim())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if v.get("error").is_some() {
        return Err(std::io::Error::other("jsonrpc error"));
    }
    Ok(v)
}

fn read_line(stream: &mut std::os::unix::net::UnixStream) -> Result<String, std::io::Error> {
    let mut buf = Vec::new();
    let mut one = [0u8; 1];
    loop {
        match stream.read(&mut one) {
            Ok(0) => {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read"));
            }
            Ok(_) => {
                if one[0] == b'\n' {
                    break;
                }
                buf.push(one[0]);
            }
            Err(e) => return Err(e),
        }
    }
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::probe_capabilities_list;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn run_line_server(sock_path: PathBuf, tx: mpsc::Sender<()>) {
        thread::spawn(move || {
            let _ = std::fs::remove_file(&sock_path);
            let listener = UnixListener::bind(&sock_path).expect("bind");
            tx.send(()).expect("ready");
            let (mut stream, _) = listener.accept().expect("accept");
            stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();

            let mut line1 = String::new();
            let mut buf = [0u8; 1];
            while !line1.ends_with('\n') && line1.len() < 65536 {
                stream.read_exact(&mut buf).expect("read");
                line1.push(buf[0] as char);
            }

            writeln!(stream, "{}", serde_json::json!({"jsonrpc":"2.0","id":1,"result":true}))
                .expect("write1");

            let mut line2 = String::new();
            while !line2.ends_with('\n') && line2.len() < 65536 {
                stream.read_exact(&mut buf).expect("read2");
                line2.push(buf[0] as char);
            }

            writeln!(
                stream,
                "{}",
                serde_json::json!({"jsonrpc":"2.0","id":2,"result":["alpha","beta"]})
            )
            .expect("write2");
        });
    }

    #[test]
    fn probe_capabilities_list_parses_flat_array() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("probe.sock");
        let (tx, rx) = mpsc::channel();
        run_line_server(sock.clone(), tx);
        rx.recv_timeout(Duration::from_secs(2)).expect("server ready");
        thread::sleep(Duration::from_millis(20));

        let caps = probe_capabilities_list(&sock);
        assert_eq!(caps, Some(vec!["alpha".into(), "beta".into()]));
    }

    #[test]
    fn probe_capabilities_list_missing_socket_returns_none() {
        let caps = probe_capabilities_list(Path::new("/no/such/socket/path/primal.sock"));
        assert!(caps.is_none());
    }
}
