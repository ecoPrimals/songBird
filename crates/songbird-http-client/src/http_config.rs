// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP Client Configuration - Adaptive & Routing-Based
//!
//! Provides granular control over HTTP behavior including:
//! - Default User-Agent and headers
//! - Domain-based header routing rules
//! - Redirect following configuration
//! - Bot protection bypass patterns
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     HttpClientConfig                                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
//! │  │  Default Headers │  │  Domain Rules    │  │  Redirect Config │  │
//! │  │  - User-Agent    │  │  - Pattern match │  │  - Max redirects │  │
//! │  │  - Accept        │  │  - Custom headers│  │  - Follow mode   │  │
//! │  │  - Accept-Lang   │  │  - Bot bypass    │  │  - Same-origin   │  │
//! │  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::time::Duration;

/// Version string for User-Agent
pub const SONGBIRD_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default User-Agent header
#[must_use]
pub fn default_user_agent() -> String {
    format!(
        "Songbird/{SONGBIRD_VERSION} (ecoPrimals Tower Atomic; Pure Rust TLS 1.3; +https://github.com/ecoPrimals/songbird)"
    )
}

/// Redirect following mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedirectMode {
    /// Don't follow redirects (return 3xx as-is)
    None,
    /// Follow redirects (default, max 10)
    #[default]
    Follow,
    /// Follow only same-origin redirects
    SameOrigin,
}

/// Domain pattern for routing rules
#[derive(Debug, Clone)]
pub enum DomainPattern {
    /// Exact match: "github.com"
    Exact(String),
    /// Suffix match: ".github.com" matches "api.github.com", "raw.github.com"
    Suffix(String),
    /// Contains match: "api" matches "api.github.com", "myapi.com"
    Contains(String),
    /// All domains (fallback rule)
    Any,
}

impl DomainPattern {
    /// Check if this pattern matches the given domain
    #[must_use]
    pub fn matches(&self, domain: &str) -> bool {
        match self {
            Self::Exact(pattern) => domain.eq_ignore_ascii_case(pattern),
            Self::Suffix(suffix) => domain.to_lowercase().ends_with(&suffix.to_lowercase()),
            Self::Contains(needle) => domain.to_lowercase().contains(&needle.to_lowercase()),
            Self::Any => true,
        }
    }
}

/// Header routing rule for specific domains
#[derive(Debug, Clone)]
pub struct HeaderRule {
    /// Domain pattern to match
    pub pattern: DomainPattern,
    /// Headers to add for matching domains
    pub headers: HashMap<String, String>,
    /// Priority (higher = checked first)
    pub priority: u8,
    /// Description for debugging
    pub description: String,
}

impl HeaderRule {
    /// Create a new header rule
    pub fn new(pattern: DomainPattern, description: impl Into<String>) -> Self {
        Self {
            pattern,
            headers: HashMap::new(),
            priority: 50,
            description: description.into(),
        }
    }

    /// Add a header to this rule
    #[must_use]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set priority
    #[must_use]
    pub const fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// HTTP client configuration with adaptive behavior
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Default User-Agent header
    pub user_agent: String,
    /// Default headers applied to all requests
    pub default_headers: HashMap<String, String>,
    /// Domain-specific header rules (checked in priority order)
    pub header_rules: Vec<HeaderRule>,
    /// Redirect following mode
    pub redirect_mode: RedirectMode,
    /// Maximum number of redirects to follow
    pub max_redirects: u8,
    /// Request timeout
    pub timeout: Duration,
    /// Enable adaptive learning (adjust headers based on response)
    pub adaptive_mode: bool,
    /// Known bot-protected domains (will add extra headers)
    bot_protected_domains: Vec<String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self::standard()
    }
}

impl HttpClientConfig {
    /// Create minimal config (no default headers)
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            user_agent: default_user_agent(),
            default_headers: HashMap::new(),
            header_rules: Vec::new(),
            redirect_mode: RedirectMode::None,
            max_redirects: 0,
            timeout: Duration::from_secs(30),
            adaptive_mode: false,
            bot_protected_domains: Vec::new(),
        }
    }

    /// Create standard config with sensible defaults
    #[must_use]
    pub fn standard() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("Accept".to_string(), "*/*".to_string());
        default_headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
        default_headers.insert("Connection".to_string(), "keep-alive".to_string());

        Self {
            user_agent: default_user_agent(),
            default_headers,
            header_rules: Self::standard_rules(),
            redirect_mode: RedirectMode::Follow,
            max_redirects: 10,
            timeout: Duration::from_secs(30),
            adaptive_mode: true,
            bot_protected_domains: Self::known_bot_protected(),
        }
    }

    /// Create browser-like config (mimics modern browser)
    #[must_use]
    pub fn browser_like() -> Self {
        let mut config = Self::standard();
        config.user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();
        config.default_headers.insert(
            "Accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".to_string(),
        );
        config
            .default_headers
            .insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        config.default_headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
        config.default_headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
        config.default_headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
        config.default_headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
        config
    }

    /// Create API-focused config (for REST APIs)
    #[must_use]
    pub fn api() -> Self {
        let mut config = Self::standard();
        config.default_headers.insert("Accept".to_string(), "application/json".to_string());
        config.default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        config.redirect_mode = RedirectMode::Follow;
        config
    }

    /// Standard header rules for known sites
    fn standard_rules() -> Vec<HeaderRule> {
        vec![
            // GitHub API requires User-Agent
            HeaderRule::new(
                DomainPattern::Suffix(".github.com".to_string()),
                "GitHub API bot protection",
            )
            .with_header("Accept", "application/vnd.github+json")
            .with_priority(90),
            // crates.io requires User-Agent
            HeaderRule::new(
                DomainPattern::Exact("crates.io".to_string()),
                "crates.io bot protection",
            )
            .with_header("Accept", "application/json")
            .with_priority(90),
            // npm requires User-Agent
            HeaderRule::new(DomainPattern::Suffix(".npmjs.org".to_string()), "npm registry")
                .with_priority(90),
            HeaderRule::new(DomainPattern::Exact("registry.npmjs.org".to_string()), "npm registry")
                .with_priority(90),
            // Wikipedia blocks bots without User-Agent
            HeaderRule::new(
                DomainPattern::Suffix(".wikipedia.org".to_string()),
                "Wikipedia bot protection",
            )
            .with_header("Accept", "text/html")
            .with_priority(80),
            // Reddit bot protection
            HeaderRule::new(
                DomainPattern::Suffix(".reddit.com".to_string()),
                "Reddit bot protection",
            )
            .with_priority(80),
            // Stack Overflow
            HeaderRule::new(
                DomainPattern::Suffix(".stackoverflow.com".to_string()),
                "Stack Overflow",
            )
            .with_priority(80),
            HeaderRule::new(
                DomainPattern::Exact("stackoverflow.com".to_string()),
                "Stack Overflow",
            )
            .with_priority(80),
            // AI/ML APIs
            HeaderRule::new(DomainPattern::Suffix(".openai.com".to_string()), "OpenAI API")
                .with_header("Accept", "application/json")
                .with_priority(70),
            HeaderRule::new(DomainPattern::Suffix(".anthropic.com".to_string()), "Anthropic API")
                .with_header("Accept", "application/json")
                .with_priority(70),
            HeaderRule::new(
                DomainPattern::Suffix(".huggingface.co".to_string()),
                "HuggingFace API",
            )
            .with_header("Accept", "application/json")
            .with_priority(70),
        ]
    }

    /// Known bot-protected domains
    fn known_bot_protected() -> Vec<String> {
        vec![
            "github.com".to_string(),
            "api.github.com".to_string(),
            "crates.io".to_string(),
            "registry.npmjs.org".to_string(),
            "wikipedia.org".to_string(),
            "reddit.com".to_string(),
            "stackoverflow.com".to_string(),
        ]
    }

    /// Add a custom header rule
    pub fn add_rule(&mut self, rule: HeaderRule) {
        self.header_rules.push(rule);
        // Sort by priority (highest first)
        self.header_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get headers for a specific domain
    ///
    /// Returns merged headers from:
    /// 1. Default headers
    /// 2. User-Agent header
    /// 3. Matching domain rules (in priority order)
    /// 4. Caller-provided headers (override everything)
    pub fn headers_for_domain(
        &self,
        domain: &str,
        caller_headers: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // 1. Start with default headers
        headers.extend(self.default_headers.clone());

        // 2. Add User-Agent if not already set
        if !caller_headers.contains_key("User-Agent") && !caller_headers.contains_key("user-agent")
        {
            headers.insert("User-Agent".to_string(), self.user_agent.clone());
        }

        // 3. Apply matching domain rules (highest priority first)
        for rule in &self.header_rules {
            if rule.pattern.matches(domain) {
                tracing::debug!("🎯 Applying rule '{}' for domain '{}'", rule.description, domain);
                headers.extend(rule.headers.clone());
            }
        }

        // 4. Apply caller headers (override everything)
        headers.extend(caller_headers.clone());

        headers
    }

    /// Check if a domain is known to have bot protection
    #[must_use]
    pub fn is_bot_protected(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        self.bot_protected_domains.iter().any(|d| domain_lower.contains(d))
    }

    /// Set custom User-Agent
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Add a default header
    #[must_use]
    pub fn with_default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Set redirect mode
    #[must_use]
    pub const fn with_redirect_mode(mut self, mode: RedirectMode) -> Self {
        self.redirect_mode = mode;
        self
    }

    /// Set max redirects
    #[must_use]
    pub const fn with_max_redirects(mut self, max: u8) -> Self {
        self.max_redirects = max;
        self
    }

    /// Set timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable/disable adaptive mode
    #[must_use]
    pub const fn with_adaptive_mode(mut self, enabled: bool) -> Self {
        self.adaptive_mode = enabled;
        self
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_default_user_agent() {
        let ua = default_user_agent();
        assert!(ua.contains("Songbird/"));
        assert!(ua.contains("ecoPrimals"));
        assert!(ua.contains("Tower Atomic"));
    }

    #[test]
    fn test_domain_pattern_exact() {
        let pattern = DomainPattern::Exact("github.com".to_string());
        assert!(pattern.matches("github.com"));
        assert!(pattern.matches("GITHUB.COM"));
        assert!(!pattern.matches("api.github.com"));
        assert!(!pattern.matches("github.com.evil.com"));
    }

    #[test]
    fn test_domain_pattern_suffix() {
        let pattern = DomainPattern::Suffix(".github.com".to_string());
        assert!(pattern.matches("api.github.com"));
        assert!(pattern.matches("raw.github.com"));
        assert!(!pattern.matches("github.com")); // No leading dot
        assert!(!pattern.matches("fakegithub.com"));
    }

    #[test]
    fn test_domain_pattern_contains() {
        let pattern = DomainPattern::Contains("api".to_string());
        assert!(pattern.matches("api.github.com"));
        assert!(pattern.matches("myapi.com"));
        assert!(pattern.matches("the-api-server.net"));
        assert!(!pattern.matches("github.com"));
    }

    #[test]
    fn test_domain_pattern_any() {
        let pattern = DomainPattern::Any;
        assert!(pattern.matches("anything.com"));
        assert!(pattern.matches(""));
        assert!(pattern.matches("a.b.c.d.e.f"));
    }

    #[test]
    fn test_headers_for_domain_github() {
        let config = HttpClientConfig::standard();
        let caller_headers = HashMap::new();

        let headers = config.headers_for_domain("api.github.com", &caller_headers);

        assert!(headers.contains_key("User-Agent"));
        assert!(headers.get("User-Agent").unwrap().contains("Songbird"));
        assert!(headers.contains_key("Accept"));
    }

    #[test]
    fn test_caller_headers_override() {
        let config = HttpClientConfig::standard();
        let mut caller_headers = HashMap::new();
        caller_headers.insert("User-Agent".to_string(), "CustomAgent/1.0".to_string());

        let headers = config.headers_for_domain("example.com", &caller_headers);

        assert_eq!(headers.get("User-Agent").unwrap(), "CustomAgent/1.0");
    }

    #[test]
    fn test_is_bot_protected() {
        let config = HttpClientConfig::standard();

        assert!(config.is_bot_protected("github.com"));
        assert!(config.is_bot_protected("api.github.com"));
        assert!(config.is_bot_protected("crates.io"));
        assert!(!config.is_bot_protected("example.com"));
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = HttpClientConfig::minimal()
            .with_user_agent("TestAgent/1.0")
            .with_default_header("X-Custom", "value")
            .with_redirect_mode(RedirectMode::SameOrigin)
            .with_max_redirects(5)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.user_agent, "TestAgent/1.0");
        assert_eq!(config.default_headers.get("X-Custom").unwrap(), "value");
        assert_eq!(config.redirect_mode, RedirectMode::SameOrigin);
        assert_eq!(config.max_redirects, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_browser_like_config() {
        let config = HttpClientConfig::browser_like();

        assert!(config.user_agent.contains("Mozilla"));
        assert!(config.user_agent.contains("Chrome"));
        assert!(config.default_headers.contains_key("Sec-Fetch-Dest"));
    }

    #[test]
    fn test_api_config() {
        let config = HttpClientConfig::api();

        assert_eq!(config.default_headers.get("Accept").unwrap(), "application/json");
        assert_eq!(config.default_headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_add_rule_sorts_by_priority_descending() {
        let mut config = HttpClientConfig::minimal();
        let low = HeaderRule::new(DomainPattern::Exact("a.com".to_string()), "low")
            .with_header("X-P", "1")
            .with_priority(10);
        let high = HeaderRule::new(DomainPattern::Exact("a.com".to_string()), "high")
            .with_header("X-P", "2")
            .with_priority(90);
        config.add_rule(low);
        config.add_rule(high);

        assert_eq!(config.header_rules[0].priority, 90);
        assert_eq!(config.header_rules[1].priority, 10);
    }

    #[test]
    fn test_headers_for_domain_lowercase_user_agent_override() {
        let config = HttpClientConfig::standard();
        let mut caller = HashMap::new();
        caller.insert("user-agent".to_string(), "LowercaseAgent/1".to_string());
        let headers = config.headers_for_domain("example.com", &caller);
        assert_eq!(headers.get("user-agent"), Some(&"LowercaseAgent/1".to_string()));
    }

    #[test]
    fn redirect_mode_default_is_follow() {
        assert_eq!(RedirectMode::default(), RedirectMode::Follow);
    }
}
