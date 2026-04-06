// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::IpcServiceHandler;
use serde::Serialize;
use serde_json::Value;

impl IpcServiceHandler {
    pub(super) fn parse_tcp_port(addr: &str) -> Result<u16, String> {
        addr.split(':')
            .next_back()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("Invalid TCP address: {addr}"))
    }

    pub(super) fn parse_local_tcp_endpoint(endpoint: &str) -> Option<u16> {
        let (host, port_str) = endpoint.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;
        let is_local = matches!(host, "127.0.0.1" | "0.0.0.0" | "localhost" | "::1" | "[::1]");
        is_local.then_some(port)
    }

    /// Serialize a handler result into a JSON-RPC response `Value`.
    pub(super) fn wrap_result<T: Serialize>(
        result: std::result::Result<T, impl std::fmt::Display>,
        context: &str,
    ) -> Result<Value, String> {
        let val = result.map_err(|e| format!("{context}: {e}"))?;
        serde_json::to_value(val).map_err(|e| format!("Serialization error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Copyright (c) 2024-2026 ecoPrimals

    use super::super::IpcServiceHandler;
    use serde::Serialize;
    use serde::ser::Error;
    use serde_json::json;

    #[test]
    fn parse_tcp_port_ipv4_happy() {
        assert_eq!(IpcServiceHandler::parse_tcp_port("127.0.0.1:8080"), Ok(8080));
        assert_eq!(IpcServiceHandler::parse_tcp_port("192.168.0.1:1"), Ok(1));
        assert_eq!(IpcServiceHandler::parse_tcp_port("0.0.0.0:0"), Ok(0));
        assert_eq!(IpcServiceHandler::parse_tcp_port("10.0.0.1:65535"), Ok(65535));
    }

    #[test]
    fn parse_tcp_port_bracketed_ipv6_happy() {
        assert_eq!(IpcServiceHandler::parse_tcp_port("[::1]:443"), Ok(443));
        assert_eq!(IpcServiceHandler::parse_tcp_port("[2001:db8::1]:22"), Ok(22));
    }

    #[test]
    fn parse_tcp_port_invalid() {
        let err = IpcServiceHandler::parse_tcp_port("127.0.0.1:not-a-port").unwrap_err();
        assert!(err.contains("Invalid TCP address"), "unexpected message: {err}");
        assert!(err.contains("127.0.0.1:not-a-port"));

        let err = IpcServiceHandler::parse_tcp_port("no-colon-here").unwrap_err();
        assert!(err.contains("Invalid TCP address"));

        let err = IpcServiceHandler::parse_tcp_port("localhost").unwrap_err();
        assert!(err.contains("Invalid TCP address"));

        let err = IpcServiceHandler::parse_tcp_port("host:").unwrap_err();
        assert!(err.contains("Invalid TCP address"));
    }

    #[test]
    fn parse_local_tcp_endpoint_local_hosts() {
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("127.0.0.1:9090"), Some(9090));
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("0.0.0.0:0"), Some(0));
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("localhost:3000"), Some(3000));
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("[::1]:9000"), Some(9000));
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("::1:9001"), Some(9001));
    }

    #[test]
    fn parse_local_tcp_endpoint_non_local() {
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("192.168.1.10:80"), None);
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("8.8.8.8:53"), None);
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("example.com:443"), None);
    }

    #[test]
    fn parse_local_tcp_endpoint_malformed() {
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("no-colon"), None);
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint(""), None);
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("127.0.0.1:bad"), None);
        assert_eq!(IpcServiceHandler::parse_local_tcp_endpoint("127.0.0.1:65536"), None);
    }

    /// Forces `serde_json::to_value` to fail without relying on float/JSON quirks.
    struct AlwaysFailsSerialize;

    impl Serialize for AlwaysFailsSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("forced serialization failure for tests"))
        }
    }

    #[test]
    fn wrap_result_ok_serializes() {
        let v = IpcServiceHandler::wrap_result(Ok::<_, String>(json!({"a": 1})), "ctx").unwrap();
        assert_eq!(v, json!({"a": 1}));
    }

    #[test]
    fn wrap_result_err_includes_context_and_display() {
        let err = IpcServiceHandler::wrap_result::<i32>(
            Err(std::io::Error::new(std::io::ErrorKind::Other, "boom")),
            "my_operation",
        )
        .unwrap_err();
        assert!(err.starts_with("my_operation:"), "got {err}");
        assert!(err.contains("boom"), "got {err}");
    }

    #[test]
    fn wrap_result_serialization_error_from_serializer() {
        let err =
            IpcServiceHandler::wrap_result(Ok::<_, String>(AlwaysFailsSerialize), "serialize_test")
                .unwrap_err();
        assert!(err.starts_with("Serialization error:"), "unexpected: {err}");
        assert!(err.contains("forced serialization failure"), "unexpected: {err}");
    }
}
