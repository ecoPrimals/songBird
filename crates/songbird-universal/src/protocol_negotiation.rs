// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! G65 Protocol Negotiation — single-socket protocol selection for songBird.
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc at connection time.
//! Phase 3 cephalization: a single socket serves both protocols, eliminating the
//! dual-socket pattern (`.sock` + `.tarpc.sock`).
//!
//! ## Wire Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! [Connection proceeds with selected protocol]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the client doesn't send a `PROTOCOLS:` line, the server assumes JSON-RPC.
//! Legacy clients (Phase 1/2) continue to work without modification.
//!
//! ## Reference
//!
//! sourDough `d3d125f` — G65 reference implementation. See `specs/PROTOCOL_NEGOTIATION_SPEC.md`.

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

/// RPC protocol variants supported by the ecoPrimals ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 — text-based, human-readable, backward-compatible default.
    #[default]
    JsonRpc,
    /// tarpc — binary, type-safe, high-performance intra-gate protocol.
    Tarpc,
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

impl IpcProtocol {
    /// Wire name used in the `PROTOCOLS:` / `PROTOCOL:` lines.
    #[must_use]
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }

    /// Parse a protocol from its wire name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" | "binary" => Some(Self::Tarpc),
            _ => None,
        }
    }

    /// All protocols this build of songBird supports (tarpc preferred).
    #[must_use]
    pub fn all_supported() -> Vec<Self> {
        vec![Self::Tarpc, Self::JsonRpc]
    }
}

/// Client's protocol negotiation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationRequest {
    /// Protocols the client supports, in preference order.
    pub supported: Vec<IpcProtocol>,
}

impl NegotiationRequest {
    /// Create a request listing the given protocols.
    #[must_use]
    pub const fn new(supported: Vec<IpcProtocol>) -> Self {
        Self { supported }
    }

    /// Request preferring tarpc, falling back to JSON-RPC.
    #[must_use]
    pub fn prefer_tarpc() -> Self {
        Self {
            supported: vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        }
    }

    /// Serialize to wire format: `"PROTOCOLS: tarpc,jsonrpc\n"`
    #[must_use]
    pub fn to_wire(&self) -> String {
        let names: Vec<&str> = self.supported.iter().map(IpcProtocol::wire_name).collect();
        format!("PROTOCOLS: {}\n", names.join(","))
    }

    /// Parse from wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the line doesn't start with `PROTOCOLS: ` or has no valid protocols.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let body = trimmed
            .strip_prefix("PROTOCOLS: ")
            .ok_or(NegotiationError::InvalidRequest)?;

        let supported: Vec<IpcProtocol> = body
            .split(',')
            .filter_map(|s| IpcProtocol::parse(s.trim()))
            .collect();

        if supported.is_empty() {
            return Err(NegotiationError::NoValidProtocols);
        }

        Ok(Self { supported })
    }
}

/// Server's protocol selection response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationResponse {
    /// The protocol the server selected.
    pub selected: IpcProtocol,
}

impl NegotiationResponse {
    /// Create a response selecting the given protocol.
    #[must_use]
    pub const fn new(selected: IpcProtocol) -> Self {
        Self { selected }
    }

    /// Serialize to wire format: `"PROTOCOL: tarpc\n"`
    #[must_use]
    pub fn to_wire(&self) -> String {
        format!("PROTOCOL: {}\n", self.selected.wire_name())
    }

    /// Parse from wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the line doesn't match the expected format.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let name = trimmed
            .strip_prefix("PROTOCOL: ")
            .ok_or(NegotiationError::InvalidResponse)?;

        let selected = IpcProtocol::parse(name).ok_or(NegotiationError::UnknownProtocol)?;

        Ok(Self { selected })
    }
}

/// Errors during protocol negotiation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NegotiationError {
    /// Line does not start with `PROTOCOLS: `.
    #[error("invalid negotiation request (expected PROTOCOLS: ...)")]
    InvalidRequest,
    /// Line does not start with `PROTOCOL: `.
    #[error("invalid negotiation response (expected PROTOCOL: ...)")]
    InvalidResponse,
    /// None of the listed protocols are recognized.
    #[error("no valid protocols in request")]
    NoValidProtocols,
    /// Protocol name not recognized.
    #[error("unknown protocol name")]
    UnknownProtocol,
    /// I/O error during negotiation.
    #[error("negotiation I/O error: {0}")]
    Io(String),
    /// Timeout waiting for negotiation.
    #[error("negotiation timed out")]
    Timeout,
}

/// Select the best protocol: first from `client_prefs` that `server_supports` also contains.
///
/// Falls back to `JsonRpc` if no intersection (JSON-RPC is always implicitly supported).
#[must_use]
pub fn select_protocol(
    client_prefs: &[IpcProtocol],
    server_supports: &[IpcProtocol],
) -> IpcProtocol {
    for proto in client_prefs {
        if server_supports.contains(proto) {
            return *proto;
        }
    }
    IpcProtocol::JsonRpc
}

/// Client-side negotiation: send preferences, receive server's selection.
///
/// # Errors
///
/// Returns `NegotiationError` on I/O failure or invalid response.
pub async fn negotiate_client<T>(
    transport: &mut T,
    supported: &[IpcProtocol],
) -> Result<IpcProtocol, NegotiationError>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let request = NegotiationRequest::new(supported.to_vec());
    let wire = request.to_wire();

    tracing::debug!("G65 client sending: {:?}", wire.trim());
    transport
        .write_all(wire.as_bytes())
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;
    transport
        .flush()
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    let mut reader = BufReader::new(transport);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    if line.is_empty() {
        return Err(NegotiationError::Io("server closed connection".into()));
    }

    let response = NegotiationResponse::from_wire(&line)?;
    tracing::info!("G65 negotiated protocol: {}", response.selected);
    Ok(response.selected)
}

/// Server-side negotiation: given a first line already read, select best protocol and respond.
///
/// Returns the selected protocol. Writes the `PROTOCOL:` response line to the writer.
///
/// # Errors
///
/// Returns `NegotiationError` on I/O failure or malformed request.
pub async fn negotiate_server_from_line<W>(
    first_line: &str,
    writer: &mut W,
    server_supported: &[IpcProtocol],
) -> Result<IpcProtocol, NegotiationError>
where
    W: AsyncWrite + Unpin,
{
    let request = NegotiationRequest::from_wire(first_line)?;
    let selected = select_protocol(&request.supported, server_supported);

    let response = NegotiationResponse::new(selected);
    writer
        .write_all(response.to_wire().as_bytes())
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    tracing::info!("G65 server selected: {selected}");
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn ipc_protocol_wire_names() {
        assert_eq!(IpcProtocol::JsonRpc.wire_name(), "jsonrpc");
        assert_eq!(IpcProtocol::Tarpc.wire_name(), "tarpc");
    }

    #[test]
    fn ipc_protocol_parse_variants() {
        assert_eq!(IpcProtocol::parse("jsonrpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::parse("json-rpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::parse("json_rpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::parse("tarpc"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::parse("binary"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::parse("TARPC"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::parse("unknown"), None);
    }

    #[test]
    fn ipc_protocol_display() {
        assert_eq!(format!("{}", IpcProtocol::Tarpc), "tarpc");
        assert_eq!(format!("{}", IpcProtocol::JsonRpc), "jsonrpc");
    }

    #[test]
    fn negotiation_request_wire_roundtrip() {
        let req = NegotiationRequest::prefer_tarpc();
        let wire = req.to_wire();
        assert_eq!(wire, "PROTOCOLS: tarpc,jsonrpc\n");

        let parsed = NegotiationRequest::from_wire(&wire).unwrap();
        assert_eq!(parsed, req);
    }

    #[test]
    fn negotiation_request_from_wire_invalid() {
        assert!(NegotiationRequest::from_wire("HELLO").is_err());
        assert!(NegotiationRequest::from_wire("PROTOCOLS: ").is_err());
        assert!(NegotiationRequest::from_wire("PROTOCOLS: unknown,garbage").is_err());
    }

    #[test]
    fn negotiation_response_wire_roundtrip() {
        let resp = NegotiationResponse::new(IpcProtocol::Tarpc);
        let wire = resp.to_wire();
        assert_eq!(wire, "PROTOCOL: tarpc\n");

        let parsed = NegotiationResponse::from_wire(&wire).unwrap();
        assert_eq!(parsed, resp);
    }

    #[test]
    fn negotiation_response_from_wire_invalid() {
        assert!(NegotiationResponse::from_wire("HELLO").is_err());
        assert!(NegotiationResponse::from_wire("PROTOCOL: unknown").is_err());
    }

    #[test]
    fn select_protocol_tarpc_preferred() {
        let client = [IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = IpcProtocol::all_supported();
        assert_eq!(select_protocol(&client, &server), IpcProtocol::Tarpc);
    }

    #[test]
    fn select_protocol_jsonrpc_only() {
        let client = [IpcProtocol::JsonRpc];
        let server = IpcProtocol::all_supported();
        assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn select_protocol_no_overlap_defaults_jsonrpc() {
        let client = [IpcProtocol::Tarpc];
        let server = [IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(&client, &server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn select_protocol_empty_client_defaults_jsonrpc() {
        let server = IpcProtocol::all_supported();
        assert_eq!(select_protocol(&[], &server), IpcProtocol::JsonRpc);
    }

    #[tokio::test]
    async fn negotiate_client_server_roundtrip_tarpc() {
        let (mut client_stream, mut server_stream) = duplex(1024);

        let server_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(&mut server_stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request = NegotiationRequest::from_wire(&line).unwrap();
            let selected = select_protocol(&request.supported, &IpcProtocol::all_supported());
            let response = NegotiationResponse::new(selected);
            reader
                .get_mut()
                .write_all(response.to_wire().as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();
            selected
        });

        let client_result = negotiate_client(
            &mut client_stream,
            &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        )
        .await
        .unwrap();

        let server_result = server_handle.await.unwrap();

        assert_eq!(client_result, IpcProtocol::Tarpc);
        assert_eq!(server_result, IpcProtocol::Tarpc);
    }

    #[tokio::test]
    async fn negotiate_client_server_roundtrip_jsonrpc_only() {
        let (mut client_stream, mut server_stream) = duplex(1024);

        let server_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(&mut server_stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request = NegotiationRequest::from_wire(&line).unwrap();
            let selected = select_protocol(&request.supported, &[IpcProtocol::JsonRpc]);
            let response = NegotiationResponse::new(selected);
            reader
                .get_mut()
                .write_all(response.to_wire().as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();
            selected
        });

        let client_result =
            negotiate_client(&mut client_stream, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
                .await
                .unwrap();

        let server_result = server_handle.await.unwrap();

        assert_eq!(client_result, IpcProtocol::JsonRpc);
        assert_eq!(server_result, IpcProtocol::JsonRpc);
    }

    #[tokio::test]
    async fn negotiate_server_from_line_tarpc() {
        let (mut _reader, mut writer) = duplex(1024);
        let result = negotiate_server_from_line(
            "PROTOCOLS: tarpc,jsonrpc\n",
            &mut writer,
            &IpcProtocol::all_supported(),
        )
        .await
        .unwrap();
        assert_eq!(result, IpcProtocol::Tarpc);
    }

    #[tokio::test]
    async fn negotiate_server_from_line_jsonrpc_fallback() {
        let (mut _reader, mut writer) = duplex(1024);
        let result = negotiate_server_from_line(
            "PROTOCOLS: jsonrpc\n",
            &mut writer,
            &IpcProtocol::all_supported(),
        )
        .await
        .unwrap();
        assert_eq!(result, IpcProtocol::JsonRpc);
    }

    #[test]
    fn error_display() {
        let e = NegotiationError::InvalidRequest;
        assert!(e.to_string().contains("PROTOCOLS:"));
        let e = NegotiationError::Timeout;
        assert!(e.to_string().contains("timed out"));
    }

    #[test]
    fn ipc_protocol_default_is_jsonrpc() {
        assert_eq!(IpcProtocol::default(), IpcProtocol::JsonRpc);
    }

    #[test]
    fn all_supported_contains_both() {
        let all = IpcProtocol::all_supported();
        assert!(all.contains(&IpcProtocol::Tarpc));
        assert!(all.contains(&IpcProtocol::JsonRpc));
    }
}
