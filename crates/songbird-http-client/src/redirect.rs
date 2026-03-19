//! HTTP redirect handling
//!
//! Handles detection and resolution of HTTP redirects (301, 302, 303, 307, 308).

use crate::error::{Error, Result};
use crate::http_config::RedirectMode;
use crate::types::HttpResponse;
use hyper::Uri;
use tracing::debug;

/// HTTP redirect handler
///
/// Manages redirect detection, URL resolution, and loop prevention.
pub struct RedirectHandler {
    max_redirects: usize,
}

impl RedirectHandler {
    /// Create a new redirect handler
    ///
    /// # Arguments
    ///
    /// * `max_redirects` - Maximum number of redirects to follow
    pub const fn new(max_redirects: usize) -> Self {
        Self {
            max_redirects,
        }
    }

    /// Check if a status code indicates a redirect
    ///
    /// Recognizes: 301, 302, 303, 307, 308
    pub const fn is_redirect_status(status: u16) -> bool {
        matches!(status, 301 | 302 | 303 | 307 | 308)
    }

    /// Check if we should follow this redirect
    ///
    /// # Arguments
    ///
    /// * `response` - HTTP response to check
    /// * `redirect_count` - Current number of redirects followed
    /// * `redirect_mode` - Redirect following mode
    ///
    /// # Returns
    ///
    /// `true` if redirect should be followed, `false` otherwise
    pub fn should_follow(
        &self,
        response: &HttpResponse,
        redirect_count: usize,
        redirect_mode: RedirectMode,
    ) -> bool {
        // Check redirect limit
        if redirect_count >= self.max_redirects {
            debug!("❌ Max redirects ({}) reached", self.max_redirects);
            return false;
        }

        // Check redirect mode
        if matches!(redirect_mode, RedirectMode::None) {
            return false;
        }

        // Check if status indicates redirect
        if !Self::is_redirect_status(response.status) {
            return false;
        }

        // Check if Location header exists
        response.headers.contains_key("location")
    }

    /// Extract host from location header
    ///
    /// Handles both absolute URLs and relative paths.
    ///
    /// # Arguments
    ///
    /// * `location` - Location header value
    /// * `base_url` - Base URL for relative path resolution
    ///
    /// # Returns
    ///
    /// Hostname if it can be extracted
    #[allow(dead_code)]
    pub fn extract_host(location: &str, base_url: &str) -> Option<String> {
        // Try to parse location as absolute URL
        if let Ok(uri) = Uri::try_from(location) {
            if let Some(host) = uri.host() {
                return Some(host.to_string());
            }
        }

        // If relative URL, use base URL's host
        Uri::try_from(base_url).ok().and_then(|u| u.host().map(std::string::ToString::to_string))
    }

    /// Resolve a redirect URL (handles relative and absolute URLs)
    ///
    /// # Arguments
    ///
    /// * `base_url` - Original request URL
    /// * `location` - Location header value (may be relative or absolute)
    ///
    /// # Returns
    ///
    /// Resolved absolute URL
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Base URL cannot be parsed
    /// - Base URL is missing required components
    pub fn resolve_url(base_url: &str, location: &str) -> Result<String> {
        // If location is absolute, use it directly
        if location.starts_with("http://") || location.starts_with("https://") {
            return Ok(location.to_string());
        }

        // Parse base URL to get scheme and host
        let base: Uri =
            base_url.parse().map_err(|e| Error::InvalidUrl(format!("Invalid base URL: {e}")))?;

        let scheme = base.scheme_str().unwrap_or("https");
        let host =
            base.host().ok_or_else(|| Error::InvalidUrl("Missing host in base URL".to_string()))?;
        let port = base.port_u16();

        // Build new URL
        let new_url = if location.starts_with('/') {
            // Absolute path relative to host
            port.map_or_else(
                || format!("{scheme}://{host}{location}"),
                |p| format!("{scheme}://{host}:{p}{location}"),
            )
        } else {
            // Relative path - append to base path
            let base_path = base.path();
            let parent = base_path.rsplit_once('/').map_or("", |(p, _)| p);
            port.map_or_else(
                || format!("{scheme}://{host}{parent}/{location}"),
                |p| format!("{scheme}://{host}:{p}{parent}/{location}"),
            )
        };

        Ok(new_url)
    }

    /// Check if redirect should be followed based on origin matching
    ///
    /// Used for `RedirectMode::SameOrigin`
    ///
    /// # Arguments
    ///
    /// * `original_url` - Original request URL
    /// * `redirect_url` - Resolved redirect URL
    ///
    /// # Returns
    ///
    /// `true` if both URLs have the same origin (scheme + host + port)
    pub fn is_same_origin(&self, original_url: &str, redirect_url: &str) -> Result<bool> {
        let original: Uri = original_url
            .parse()
            .map_err(|e| Error::InvalidUrl(format!("Invalid original URL: {e}")))?;
        let redirect: Uri = redirect_url
            .parse()
            .map_err(|e| Error::InvalidUrl(format!("Invalid redirect URL: {e}")))?;

        // Compare scheme, host, and port
        Ok(original.scheme_str() == redirect.scheme_str()
            && original.host() == redirect.host()
            && original.port_u16() == redirect.port_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_is_redirect_status() {
        assert!(RedirectHandler::is_redirect_status(301));
        assert!(RedirectHandler::is_redirect_status(302));
        assert!(RedirectHandler::is_redirect_status(303));
        assert!(RedirectHandler::is_redirect_status(307));
        assert!(RedirectHandler::is_redirect_status(308));
        assert!(!RedirectHandler::is_redirect_status(200));
        assert!(!RedirectHandler::is_redirect_status(404));
    }

    #[test]
    fn test_resolve_absolute_url() {
        let handler = RedirectHandler::new(10);
        let result =
            RedirectHandler::resolve_url("https://example.com/path", "https://other.com/new")
                .unwrap();
        assert_eq!(result, "https://other.com/new");
    }

    #[test]
    fn test_resolve_absolute_path() {
        let handler = RedirectHandler::new(10);
        let result =
            RedirectHandler::resolve_url("https://example.com/old/path", "/new/path").unwrap();
        assert_eq!(result, "https://example.com/new/path");
    }

    #[test]
    fn test_resolve_relative_path() {
        let handler = RedirectHandler::new(10);
        let result = RedirectHandler::resolve_url("https://example.com/dir/page", "other").unwrap();
        assert_eq!(result, "https://example.com/dir/other");
    }

    #[test]
    fn test_extract_host() {
        let handler = RedirectHandler::new(10);

        // Absolute URL
        assert_eq!(
            RedirectHandler::extract_host("https://example.com/path", "https://base.com"),
            Some("example.com".to_string())
        );

        // Relative path (uses base)
        assert_eq!(
            RedirectHandler::extract_host("/path", "https://base.com"),
            Some("base.com".to_string())
        );
    }

    #[test]
    fn test_should_follow() {
        let handler = RedirectHandler::new(5);

        let mut headers = HashMap::new();
        headers.insert("location".to_string(), "/new".to_string());

        let response = HttpResponse {
            status: 302,
            headers: headers.clone(),
            body: serde_json::json!(""),
        };

        assert!(handler.should_follow(&response, 0, RedirectMode::Follow));
        assert!(!handler.should_follow(&response, 0, RedirectMode::None));
        assert!(!handler.should_follow(&response, 5, RedirectMode::Follow));

        let non_redirect = HttpResponse {
            status: 200,
            headers,
            body: serde_json::json!(""),
        };
        assert!(!handler.should_follow(&non_redirect, 0, RedirectMode::Follow));
    }

    #[test]
    fn test_is_same_origin() {
        let handler = RedirectHandler::new(10);

        // Same origin
        assert!(handler
            .is_same_origin("https://example.com/path1", "https://example.com/path2")
            .unwrap());

        // Different host
        assert!(!handler
            .is_same_origin("https://example.com/path", "https://other.com/path")
            .unwrap());

        // Different scheme
        assert!(!handler
            .is_same_origin("http://example.com/path", "https://example.com/path")
            .unwrap());

        // Different port
        assert!(!handler
            .is_same_origin("https://example.com:8080/path", "https://example.com/path")
            .unwrap());
    }
}
