// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Drawbridge authentication and external proxy allowlist.
//!
//! Contains the security policy types for the drawbridge HTTP listener:
//! - [`AuthGate`]: Bearer token auth with trusted-peer and public-path bypass
//! - [`TrustedNetwork`]: CIDR-based peer trust matching
//! - [`ExternalProxyAllowlist`]: Domain-validated forward proxy for composition

use std::net::IpAddr;

/// Route mapping: path prefix → capability name.
#[derive(Debug, Clone)]
pub struct DrawbridgeRoute {
    pub path_prefix: String,
    pub capability: String,
    /// If true, this route bypasses auth enforcement regardless of global auth config.
    pub public: bool,
}

/// CIDR-style network range for trusted peer matching.
#[derive(Debug, Clone)]
pub struct TrustedNetwork {
    addr: IpAddr,
    prefix_len: u8,
}

impl TrustedNetwork {
    /// Parse a CIDR notation string (e.g., "192.168.0.0/16", "127.0.0.1/8").
    #[must_use]
    pub fn parse(cidr: &str) -> Option<Self> {
        let (addr_str, prefix_str) = cidr.split_once('/')?;
        let addr: IpAddr = addr_str.trim().parse().ok()?;
        let prefix_len: u8 = prefix_str.trim().parse().ok()?;
        Some(Self {
            addr,
            prefix_len,
        })
    }

    /// Check if an IP address falls within this network range.
    #[must_use]
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(target)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let mask = u32::MAX.checked_shl(32 - u32::from(self.prefix_len)).unwrap_or(0);
                (u32::from(net) & mask) == (u32::from(target) & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(target)) => {
                if self.prefix_len == 0 {
                    return true;
                }
                let mask = u128::MAX.checked_shl(128 - u32::from(self.prefix_len)).unwrap_or(0);
                (u128::from(net) & mask) == (u128::from(target) & mask)
            }
            _ => false,
        }
    }
}

/// Authentication gate for the drawbridge.
///
/// Enforces bearer token auth with configurable bypass for trusted peers and public paths.
#[derive(Debug, Clone)]
pub struct AuthGate {
    pub(crate) tokens: Vec<String>,
    pub(crate) public_paths: Vec<String>,
    pub(crate) trusted_peers: Vec<TrustedNetwork>,
}

impl AuthGate {
    /// Load auth configuration from environment variables.
    ///
    /// - `SONGBIRD_DRAWBRIDGE_AUTH_TOKENS`: comma-separated valid bearer tokens
    /// - `SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS`: comma-separated path prefixes that skip auth
    /// - `SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS`: comma-separated CIDR ranges that bypass auth
    #[must_use]
    pub fn from_env() -> Self {
        let tokens: Vec<String> = songbird_process_env::var("SONGBIRD_DRAWBRIDGE_AUTH_TOKENS")
            .unwrap_or_default()
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        let public_paths: Vec<String> =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_PUBLIC_PATHS")
                .unwrap_or_default()
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();

        let trusted_peers: Vec<TrustedNetwork> =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_TRUSTED_PEERS")
                .unwrap_or_default()
                .split(',')
                .filter_map(|c| TrustedNetwork::parse(c.trim()))
                .collect();

        Self {
            tokens,
            public_paths,
            trusted_peers,
        }
    }

    /// Returns true if auth enforcement is active (tokens configured).
    #[must_use]
    pub fn is_enforcing(&self) -> bool {
        !self.tokens.is_empty()
    }

    /// Check whether a request is authorized.
    ///
    /// A request passes if ANY of:
    /// 1. No tokens configured (open mode — backward compat)
    /// 2. Path matches a public prefix
    /// 3. Peer IP is in a trusted network
    /// 4. Authorization header contains a valid bearer token
    /// 5. `_sb_token` query parameter contains a valid token
    #[must_use]
    pub fn is_authorized(&self, peer: IpAddr, path: &str, auth_header: Option<&str>) -> bool {
        if self.tokens.is_empty() {
            return true;
        }

        if self.public_paths.iter().any(|prefix| path.starts_with(prefix.as_str())) {
            return true;
        }

        if self.trusted_peers.iter().any(|net| net.contains(peer)) {
            return true;
        }

        if let Some(header) = auth_header {
            let token = header.strip_prefix("Bearer ").unwrap_or(header);
            if self.tokens.iter().any(|t| t == token) {
                return true;
            }
        }

        // Check query parameter fallback (for browser-based access)
        if let Some(query_start) = path.find('?') {
            let query = &path[query_start + 1..];
            for param in query.split('&') {
                if let Some(value) = param.strip_prefix("_sb_token=")
                    && self.tokens.iter().any(|t| t == value)
                {
                    return true;
                }
            }
        }

        false
    }
}

/// Domain-validated external proxy allowlist.
///
/// Maps service names to permitted external base URLs. Only services in this list
/// can be reached through the `_external_proxy` capability. This replaces Express
/// `/api/proxy` for the footPrint composition.
#[derive(Debug, Clone)]
pub struct ExternalProxyAllowlist {
    pub(crate) services: std::collections::HashMap<String, ExternalService>,
}

/// A permitted external service endpoint.
#[derive(Debug, Clone)]
pub struct ExternalService {
    pub base_url: String,
    pub name: String,
}

/// The reserved capability name for external proxy routes.
pub const EXTERNAL_PROXY_CAPABILITY: &str = "_external_proxy";

impl ExternalProxyAllowlist {
    /// Load from `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST` env var.
    ///
    /// Format: `service=base_url,service=base_url,...`
    /// Example: `osm=https://tile.openstreetmap.org,fema=https://hazards.fema.gov`
    #[must_use]
    pub fn from_env() -> Self {
        let mut services = std::collections::HashMap::new();

        let raw =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST").unwrap_or_default();

        for entry in raw.split(',') {
            let entry = entry.trim();
            if let Some((name, url)) = entry.split_once('=') {
                let name = name.trim();
                let url = url.trim();
                if !name.is_empty() && !url.is_empty() {
                    services.insert(
                        name.to_string(),
                        ExternalService {
                            base_url: url.trim_end_matches('/').to_string(),
                            name: name.to_string(),
                        },
                    );
                }
            }
        }

        Self {
            services,
        }
    }

    /// Check if the allowlist has any services configured.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.services.is_empty()
    }

    /// Resolve a service name to its external endpoint.
    #[must_use]
    pub fn resolve(&self, service_name: &str) -> Option<&ExternalService> {
        self.services.get(service_name)
    }

    /// List all allowed service names.
    #[must_use]
    pub fn allowed_services(&self) -> Vec<&str> {
        self.services.keys().map(String::as_str).collect()
    }

    /// Parse a request path into (`service_name`, `remaining_path`).
    ///
    /// Input: `/osm/16/32000/21000.png` → `Some(("osm", "/16/32000/21000.png"))`
    /// Input: `/unknown/path` with "unknown" not in allowlist → `None`
    #[must_use]
    pub fn parse_and_validate<'a>(
        &'a self,
        path_after_prefix: &'a str,
    ) -> Option<(&'a ExternalService, &'a str)> {
        let trimmed = path_after_prefix.trim_start_matches('/');
        let (service_name, remainder) = match trimmed.find('/') {
            Some(i) => (&trimmed[..i], &trimmed[i..]),
            None => (trimmed, "/"),
        };

        self.services.get(service_name).map(move |svc| (svc, remainder))
    }

    /// Construct the full external URL for a validated request.
    #[must_use]
    pub fn build_url(service: &ExternalService, path: &str) -> String {
        let base = &service.base_url;
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            format!("{base}/")
        } else {
            format!("{base}/{path}")
        }
    }

    /// Validate a full URL against the allowlist by checking its host against
    /// known service base URLs. Returns the URL unchanged if the domain matches
    /// any allowlisted service.
    ///
    /// This provides backward compatibility with `?url=<encoded_url>` query-string
    /// proxying (Express-style) while enforcing the same domain allowlist.
    #[must_use]
    pub fn validate_url(&self, url: &str) -> Option<&ExternalService> {
        let host = extract_host_from_url(url)?;
        self.services.values().find(|svc| {
            extract_host_from_url(&svc.base_url).is_some_and(|svc_host| svc_host == host)
        })
    }
}

/// Extract the host portion from a URL (without port).
#[must_use]
pub fn extract_host_from_url(url: &str) -> Option<&str> {
    let after_scheme = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let host_port = after_scheme.split('/').next()?;
    Some(host_port.split(':').next().unwrap_or(host_port))
}

/// Minimal percent-decoding for URL query parameters.
pub fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().and_then(hex_val);
            let lo = chars.next().and_then(hex_val);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push(char::from(h << 4 | l));
            } else {
                out.push('%');
            }
        } else if b == b'+' {
            out.push(' ');
        } else {
            out.push(char::from(b));
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
