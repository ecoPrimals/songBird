// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal HTTP Proxy - Works with ANY provider without hardcoding
//!
//! This module implements a universal proxy that can handle requests to any
//! external API provider without vendor-specific logic.
//!
//! # Philosophy
//! - **Zero Vendor Hardcoding**: No "`OpenAI`", "`HuggingFace`", "Anthropic" logic
//! - **Universal Design**: One implementation works for all providers
//! - **Configuration-Driven**: Provider behavior defined by configuration
//! - **Transform-Based**: Use transformations instead of custom code
//!
//! # Evolution Strategy
//! Instead of building separate handlers for each vendor (`OpenAI`, `HuggingFace`, etc.),
//! we build ONE universal proxy that uses:
//! 1. **Request Transformations**: Map generic requests to provider-specific formats
//! 2. **Response Transformations**: Map provider responses back to generic format
//! 3. **Configuration**: All provider-specific details in config, not code
//!
//! # Example
//! ```text
//! // NO hardcoded vendor logic like this:
//! // if provider == "openai" { ... }
//! // if provider == "huggingface" { ... }
//!
//! // Instead, universal proxy uses transforms:
//! // Transform request -> Send to provider -> Transform response
//! ```

use super::cache::ResponseCache;
use super::capability_router::Route;
use super::credentials::CredentialManager;
use super::rate_limiter::RateLimiter;
use anyhow::{Result, anyhow};
use serde_json::Value;
use songbird_http_client::SongbirdHttpClient;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn}; // ✅ Pure Rust HTTP (Tower Atomic)

/// Universal HTTP Proxy - Works with any provider
pub struct UniversalProxy {
    http_client: SongbirdHttpClient,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ResponseCache>,
    credentials: Arc<CredentialManager>,
}

impl UniversalProxy {
    /// Create a new universal proxy
    #[must_use]
    pub fn new(
        http_client: SongbirdHttpClient,
        rate_limiter: Arc<RateLimiter>,
        cache: Arc<ResponseCache>,
        credentials: Arc<CredentialManager>,
    ) -> Self {
        info!("Creating Universal HTTP Proxy (zero vendor hardcoding)");
        Self {
            http_client,
            rate_limiter,
            cache,
            credentials,
        }
    }

    /// Proxy a request to an external provider
    ///
    /// This is the ONLY method needed - no vendor-specific code!
    /// All provider differences are handled via configuration and transforms.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn proxy_request(
        &self,
        route: &Route,
        method: &str,
        payload: Option<&Value>,
    ) -> Result<Value> {
        let provider_id = &route.provider.id;

        debug!(
            "Universal proxy: {} request to '{}' for capability '{}'",
            method, route.provider.name, route.capability.id
        );

        // Check rate limits
        if let Err(e) = self.rate_limiter.check(provider_id).await {
            warn!("Rate limit exceeded for provider '{}': {}", provider_id, e);
            return Err(anyhow!("Rate limit exceeded"));
        }

        // Check cache
        let cache_key = self.generate_cache_key(route, method, payload);
        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!("Cache hit for provider '{}'", provider_id);
            return Ok(cached);
        }

        // Transform request (provider-specific format)
        let transformed_payload = self.transform_request(route, payload)?;

        // Make external request
        let response_data =
            self.make_external_request(route, method, transformed_payload.as_ref()).await?;

        // Transform response (back to generic format)
        let generic_response = self.transform_response(route, &response_data)?;

        // Cache the response
        // FUTURE (Phase 2): Implement response caching with TTL from provider config
        // Current: Direct proxy mode is safer (no stale data)

        Ok(generic_response)
    }

    /// Make the actual HTTP request to the external provider
    async fn make_external_request(
        &self,
        route: &Route,
        method: &str,
        payload: Option<&Value>,
    ) -> Result<Value> {
        // Get backend configuration
        let backend = route.provider.backend.as_ref().ok_or_else(|| {
            anyhow!("Provider '{}' has no backend configuration", route.provider.name)
        })?;

        trace!("Making {} request to: {}", method, backend.base_url);

        // Get API key if configured
        let api_key = if backend.api_key_env.is_some() {
            self.credentials.get_api_key(&route.provider.id)
        } else {
            None
        };

        // Build headers
        let mut headers = std::collections::HashMap::new();

        // Add API key (provider-agnostic - works with any auth scheme)
        if let Some(key) = api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {key}"));
        }

        // Add custom headers from configuration
        for (name, value) in &backend.headers {
            headers.insert(name.clone(), value.clone());
        }

        // Add content-type if not specified
        if !backend.headers.contains_key("content-type") && payload.is_some() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        // Send request using Pure Rust HTTP client
        let response = self
            .http_client
            .request(&method.to_uppercase(), &backend.base_url, headers, payload.cloned())
            .await?;

        // Extract status and body from Pure Rust HTTP response
        let status = response.status;
        let json = response.body;

        // Check for errors
        if !(200..300).contains(&status) {
            error!("External API returned error: {} - {:?}", status, json);
            return Err(anyhow!("External API error: {status} - {json:?}"));
        }

        trace!("External request successful: status {}", status);
        Ok(json)
    }

    /// Transform a generic request to provider-specific format
    ///
    /// This uses the transformation rules from the provider configuration
    /// instead of hardcoded vendor logic.
    fn transform_request(&self, route: &Route, payload: Option<&Value>) -> Result<Option<Value>> {
        let Some(payload) = payload else {
            return Ok(None);
        };

        // Get transformation config
        let backend = route.provider.backend.as_ref();
        let transform_config = backend.and_then(|b| b.request_transform.as_ref());

        let Some(config) = transform_config else {
            // No transformation needed - use payload as-is
            trace!("No request transformation configured, using payload as-is");
            return Ok(Some(payload.clone()));
        };

        debug!("Applying request transformation with {} mappings", config.field_mappings.len());

        // Apply field mappings
        let mut transformed = payload.clone();
        for (from_field, to_field) in &config.field_mappings {
            if let Some(value) = payload.get(from_field) {
                transformed[to_field] = value.clone();
                trace!("Mapped field: {} → {}", from_field, to_field);
            }
        }

        // Apply template if configured
        if let Some(template) = &config.template {
            // FUTURE (Phase 2): Template-based transformation (e.g., Handlebars, Tera)
            // Current: Field mapping (config.field_mappings) is sufficient for most use cases
            // Future use case: Complex JSON transformations for LLM prompt templates
            warn!("Template transformation not yet implemented: {}", template);
        }

        Ok(Some(transformed))
    }

    /// Transform provider-specific response to generic format
    ///
    /// This uses the transformation rules from the provider configuration.
    fn transform_response(&self, route: &Route, response: &Value) -> Result<Value> {
        // Get transformation config
        let backend = route.provider.backend.as_ref();
        let transform_config = backend.and_then(|b| b.response_transform.as_ref());

        let Some(config) = transform_config else {
            // No transformation needed - use response as-is
            trace!("No response transformation configured, using response as-is");
            return Ok(response.clone());
        };

        debug!("Applying response transformation with {} mappings", config.field_mappings.len());

        // Apply field mappings
        let mut transformed = response.clone();
        for (from_field, to_field) in &config.field_mappings {
            if let Some(value) = response.get(from_field) {
                transformed[to_field] = value.clone();
                trace!("Mapped field: {} → {}", from_field, to_field);
            }
        }

        // Apply template if configured
        if let Some(template) = &config.template {
            // FUTURE (Phase 2): Template-based transformation (e.g., Handlebars, Tera)
            // Current: Field mapping (config.field_mappings) is sufficient for most use cases
            // Future use case: Complex JSON transformations for LLM response formatting
            warn!("Template transformation not yet implemented: {}", template);
        }

        Ok(transformed)
    }

    /// Generate a cache key for a request
    fn generate_cache_key(&self, route: &Route, method: &str, payload: Option<&Value>) -> String {
        let payload_str =
            payload.map(|p| serde_json::to_string(p).unwrap_or_default()).unwrap_or_default();

        format!("{}:{}:{}:{}", route.capability.id, route.provider.id, method, payload_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_gateway::capability_router::{
        BackendConfig, Capability, ProviderConfig, TransformConfig,
    };
    use std::collections::HashMap;

    fn create_test_route() -> Route {
        let capability = Capability {
            id: "ai:text-generation".to_string(),
            description: "Test capability".to_string(),
            category: "ai".to_string(),
            capability_type: "text-generation".to_string(),
            sub_type: None,
            metadata: HashMap::new(),
        };

        let provider = ProviderConfig {
            id: "test_provider".to_string(),
            name: "Test Provider".to_string(),
            capabilities: vec![capability.clone()],
            socket_path: None,
            backend: Some(BackendConfig {
                base_url: "https://api.test.com/v1/completions".to_string(),
                api_key_env: Some("TEST_API_KEY".to_string()),
                request_transform: Some(TransformConfig {
                    field_mappings: {
                        let mut map = HashMap::new();
                        map.insert("prompt".to_string(), "text".to_string());
                        map
                    },
                    template: None,
                }),
                response_transform: Some(TransformConfig {
                    field_mappings: {
                        let mut map = HashMap::new();
                        map.insert("result".to_string(), "response".to_string());
                        map
                    },
                    template: None,
                }),
                headers: HashMap::new(),
            }),
            metadata: HashMap::new(),
        };

        Route {
            provider,
            capability,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let http_client = SongbirdHttpClient::new("/tmp/beardog-test.sock");
        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));
        let cache = Arc::new(ResponseCache::new(100 * 1024 * 1024));
        let credentials = Arc::new(CredentialManager::new());

        let proxy = UniversalProxy::new(http_client, rate_limiter, cache, credentials);
        let route = create_test_route();

        let payload = serde_json::json!({"prompt": "test"});
        let key = proxy.generate_cache_key(&route, "POST", Some(&payload));

        assert!(key.contains("ai:text-generation"));
        assert!(key.contains("test_provider"));
        assert!(key.contains("POST"));
    }

    #[test]
    fn test_request_transformation() {
        let http_client = SongbirdHttpClient::new("/tmp/beardog-test.sock");
        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));
        let cache = Arc::new(ResponseCache::new(100 * 1024 * 1024));
        let credentials = Arc::new(CredentialManager::new());

        let proxy = UniversalProxy::new(http_client, rate_limiter, cache, credentials);
        let route = create_test_route();

        let payload = serde_json::json!({"prompt": "Hello, world!"});
        let transformed = proxy.transform_request(&route, Some(&payload)).unwrap();

        assert!(transformed.is_some());
        let transformed = transformed.unwrap();
        assert_eq!(transformed["text"], "Hello, world!");
        assert_eq!(transformed["prompt"], "Hello, world!");
    }

    #[test]
    fn test_response_transformation() {
        let http_client = SongbirdHttpClient::new("/tmp/beardog-test.sock");
        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(60)));
        let cache = Arc::new(ResponseCache::new(100 * 1024 * 1024));
        let credentials = Arc::new(CredentialManager::new());

        let proxy = UniversalProxy::new(http_client, rate_limiter, cache, credentials);
        let route = create_test_route();

        let response = serde_json::json!({"result": "Generated text"});
        let transformed = proxy.transform_response(&route, &response).unwrap();

        assert_eq!(transformed["response"], "Generated text");
        assert_eq!(transformed["result"], "Generated text");
    }
}
