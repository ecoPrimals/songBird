// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP response parsing
//!
//! Handles parsing of raw HTTP response bytes into structured `HttpResponse` objects.

use crate::error::{Error, Result};
use crate::types::HttpResponse;
use std::collections::HashMap;
use tracing::debug;

/// HTTP response parser
///
/// Parses raw HTTP response bytes into structured format with status, headers, and body.
pub struct ResponseParser;

impl ResponseParser {
    /// Parse raw HTTP response bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Raw HTTP response bytes
    ///
    /// # Returns
    ///
    /// Parsed `HttpResponse` with status code, headers, and JSON body
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Response is empty
    /// - Status line is malformed
    /// - Headers cannot be parsed
    pub fn parse(data: &[u8]) -> Result<HttpResponse> {
        // Debug: Log first bytes to understand any corruption
        debug!("📝 parse_http_response: {} bytes total", data.len());
        debug!("📝 First 50 bytes (hex): {:?}", &data[..std::cmp::min(50, data.len())]);
        debug!(
            "📝 First 50 bytes (str): {:?}",
            String::from_utf8_lossy(&data[..std::cmp::min(50, data.len())])
        );

        let response = String::from_utf8_lossy(data);
        let mut lines = response.lines();

        // Status line
        let status_line =
            lines.next().ok_or_else(|| Error::InvalidResponse("Empty response".to_string()))?;

        // Debug: Log the status line to understand parsing issues
        debug!("📝 Parsing status line: {:?}", status_line);
        debug!("📝 Status line bytes: {:?}", status_line.as_bytes());

        let status = Self::parse_status_line(status_line)?;

        // Headers
        let (headers, body_start) = Self::parse_headers(&mut lines);

        // Body
        let body_lines: Vec<&str> = response.lines().skip(body_start).collect();
        let body_str = body_lines.join("\n");

        let body: serde_json::Value = if body_str.is_empty() {
            serde_json::json!("")
        } else {
            serde_json::from_str(&body_str).unwrap_or_else(|_| serde_json::json!(body_str))
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Parse HTTP status line
    ///
    /// Extracts status code from "HTTP/1.1 200 OK" format
    fn parse_status_line(status_line: &str) -> Result<u16> {
        status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| {
                debug!("📝 Extracted status code string: {:?}", s);
                s.parse::<u16>().ok()
            })
            .ok_or_else(|| Error::InvalidResponse(format!("Invalid status line: {status_line:?}")))
    }

    /// Parse HTTP headers
    ///
    /// Returns headers map and the line index where body starts
    fn parse_headers(lines: &mut dyn Iterator<Item = &str>) -> (HashMap<String, String>, usize) {
        let mut headers = HashMap::new();
        let mut body_start = 1; // Start after status line

        for (idx, line) in lines.enumerate() {
            if line.is_empty() {
                body_start = idx + 2; // +2 for status line and empty line
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        (headers, body_start)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_parse_simple_response() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}";
        let parsed = ResponseParser::parse(response).unwrap();

        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.headers.get("content-type"), Some(&"application/json".to_string()));
        assert_eq!(parsed.body, serde_json::json!({"status": "ok"}));
    }

    #[test]
    fn test_parse_status_line() {
        assert_eq!(ResponseParser::parse_status_line("HTTP/1.1 200 OK").unwrap(), 200);
        assert_eq!(ResponseParser::parse_status_line("HTTP/1.1 404 Not Found").unwrap(), 404);
        assert_eq!(
            ResponseParser::parse_status_line("HTTP/1.1 500 Internal Server Error").unwrap(),
            500
        );
    }

    #[test]
    fn test_parse_empty_response() {
        let response = b"";
        let result = ResponseParser::parse(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_no_body() {
        let response = b"HTTP/1.1 204 No Content\r\n\r\n";
        let parsed = ResponseParser::parse(response).unwrap();

        assert_eq!(parsed.status, 204);
        assert_eq!(parsed.body, serde_json::json!(""));
    }

    #[test]
    fn test_parse_status_line_invalid() {
        let err = ResponseParser::parse_status_line("HTTP/1.1 OK").unwrap_err();
        assert!(matches!(err, Error::InvalidResponse(_)));

        let err = ResponseParser::parse_status_line("").unwrap_err();
        assert!(matches!(err, Error::InvalidResponse(_)));
    }

    #[test]
    fn test_parse_multiline_headers_and_body_json_fallback() {
        let response = b"HTTP/1.1 200 OK\r\nX-Test: a\r\nY-Test: b\r\n\r\nnot json {{{";
        let parsed = ResponseParser::parse(response).unwrap();
        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.headers.get("x-test"), Some(&"a".to_string()));
        assert_eq!(parsed.body, serde_json::json!("not json {{{"));
    }

    #[test]
    fn test_parse_headers_skips_malformed_lines_until_blank() {
        let response = b"HTTP/1.1 200 OK\r\nbadheader\r\nGood: yes\r\n\r\n";
        let parsed = ResponseParser::parse(response).unwrap();
        assert_eq!(parsed.headers.get("good"), Some(&"yes".to_string()));
    }

    #[test]
    fn test_parse_plain_text_body_fallback() {
        let response = b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nmissing";
        let parsed = ResponseParser::parse(response).unwrap();
        assert_eq!(parsed.status, 404);
        assert_eq!(parsed.body, serde_json::json!("missing"));
    }
}
