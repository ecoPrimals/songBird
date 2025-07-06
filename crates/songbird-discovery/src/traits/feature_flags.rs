//! Feature Flag Traits
//!
//! Universal feature toggles and configuration management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_errors::Result;
use std::collections::HashMap;

/// Universal feature flag provider trait
#[async_trait]
pub trait FeatureFlagProvider: Send + Sync {
    /// Initialize the feature flag provider
    async fn initialize(&mut self, config: &FeatureFlagConfig) -> Result<()>;

    /// Check if a feature is enabled
    async fn is_enabled(
        &self,
        feature_name: &str,
        context: Option<&EvaluationContext>,
    ) -> Result<bool>;

    /// Get feature flag value
    async fn get_flag_value(
        &self,
        feature_name: &str,
        context: Option<&EvaluationContext>,
    ) -> Result<Option<serde_json::Value>>;

    /// Set feature flag value (if provider supports updates)
    async fn set_flag_value(&self, feature_name: &str, value: serde_json::Value) -> Result<()>;

    /// Get all feature flags
    async fn get_all_flags(&self) -> Result<HashMap<String, FeatureFlag>>;

    /// Register a new feature flag
    async fn register_flag(&self, flag: &FeatureFlag) -> Result<()>;

    /// Remove a feature flag
    async fn remove_flag(&self, feature_name: &str) -> Result<()>;

    /// Get flag evaluation history
    async fn get_evaluation_history(&self, feature_name: &str) -> Result<Vec<FlagEvaluation>>;

    /// Get provider information
    fn provider_info(&self) -> FeatureFlagProviderInfo;
}

/// Feature flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Unique feature name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Feature category/group
    pub category: String,
    /// Default value when no rules match
    pub default_value: serde_json::Value,
    /// Flag type
    pub flag_type: FlagType,
    /// Evaluation rules
    pub rules: Vec<EvaluationRule>,
    /// Flag metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Whether the flag is enabled
    pub enabled: bool,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Feature flag types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagType {
    /// Simple boolean flag
    Boolean,
    /// String value flag
    String,
    /// Numeric value flag
    Number,
    /// JSON object flag
    Json,
    /// Multi-variant flag
    Variant { variants: Vec<String> },
    /// Percentage rollout flag
    Percentage,
}

/// Evaluation rule for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Rule conditions
    pub conditions: Vec<RuleCondition>,
    /// Value to return if conditions match
    pub value: serde_json::Value,
    /// Rule priority (lower = higher priority)
    pub priority: u32,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Percentage of traffic this rule applies to (0-100)
    pub traffic_percentage: Option<f64>,
}

/// Rule condition for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Context attribute to evaluate
    pub attribute: String,
    /// Evaluation operator
    pub operator: RuleOperator,
    /// Value to compare against
    pub value: serde_json::Value,
    /// Whether to negate the condition
    pub negate: bool,
}

/// Rule operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleOperator {
    /// Exact equality
    Equals,
    /// Not equal
    NotEquals,
    /// String contains
    Contains,
    /// String starts with
    StartsWith,
    /// String ends with
    EndsWith,
    /// Numeric greater than
    GreaterThan,
    /// Numeric less than
    LessThan,
    /// Numeric greater than or equal
    GreaterThanOrEqual,
    /// Numeric less than or equal
    LessThanOrEqual,
    /// Value in list
    In,
    /// Value not in list
    NotIn,
    /// Regular expression match
    Regex,
    /// Semantic version match
    VersionMatch,
    /// Date/time comparison
    DateBefore,
    DateAfter,
    /// Custom function
    Custom {
        function_name: String,
    },
}

/// Evaluation context for feature flag decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// User/entity identifier
    pub user_id: Option<String>,
    /// Service identifier
    pub service_id: Option<String>,
    /// Request identifier
    pub request_id: Option<String>,
    /// Environment (dev, staging, prod, etc.)
    pub environment: Option<String>,
    /// Application version
    pub version: Option<String>,
    /// Geographic location
    pub geo_location: Option<GeoLocation>,
    /// Device/client information
    pub device_info: Option<DeviceInfo>,
    /// Custom attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Device/client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub platform: Option<String>,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
}

/// Feature flag evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEvaluation {
    /// Feature flag name
    pub feature_name: String,
    /// Evaluated value
    pub value: serde_json::Value,
    /// Rule that matched (if any)
    pub matched_rule: Option<String>,
    /// Evaluation context used
    pub context: EvaluationContext,
    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
    /// Evaluation duration in milliseconds
    pub duration_ms: u64,
    /// Whether default value was used
    pub used_default: bool,
    /// Any errors during evaluation
    pub errors: Vec<String>,
}

/// Feature flag provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagProviderInfo {
    pub name: String,
    pub version: String,
    pub supports_updates: bool,
    pub supports_history: bool,
    pub supports_targeting: bool,
    pub supports_percentage_rollout: bool,
    pub backend_type: String,
}

/// Feature flag statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagStats {
    /// Total number of evaluations
    pub total_evaluations: u64,
    /// Number of true evaluations
    pub true_evaluations: u64,
    /// Number of false evaluations
    pub false_evaluations: u64,
    /// Number of default value usages
    pub default_usages: u64,
    /// Average evaluation time
    pub avg_evaluation_time_ms: f64,
    /// Unique users/contexts evaluated
    pub unique_contexts: u64,
    /// Last evaluation timestamp
    pub last_evaluation: Option<DateTime<Utc>>,
    /// Distribution by rule
    pub rule_distribution: HashMap<String, u64>,
}

/// Feature flag manager trait
#[async_trait]
pub trait FeatureFlagManager: Send + Sync {
    /// Initialize the manager
    async fn initialize(&mut self, config: &FeatureFlagConfig) -> Result<()>;

    /// Register a feature flag provider
    async fn register_provider(
        &mut self,
        name: &str,
        user: Box<dyn FeatureFlagProvider>,
    ) -> Result<()>;

    /// Evaluate a feature flag
    async fn evaluate_flag(
        &self,
        feature_name: &str,
        context: Option<&EvaluationContext>,
    ) -> Result<FlagEvaluation>;

    /// Evaluate multiple flags at once
    async fn evaluate_flags(
        &self,
        feature_names: &[&str],
        context: Option<&EvaluationContext>,
    ) -> Result<HashMap<String, FlagEvaluation>>;

    /// Get all flags and their current states
    async fn get_all_flags_state(
        &self,
        context: Option<&EvaluationContext>,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Get flag statistics
    async fn get_flag_stats(&self, feature_name: &str) -> Result<FlagStats>;

    /// Refresh flags from provider
    async fn refresh_flags(&self) -> Result<()>;

    /// Get manager status
    async fn get_status(&self) -> Result<ManagerStatus>;
}

/// Feature flag manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerStatus {
    pub providers_count: u32,
    pub flags_count: u32,
    pub evaluations_per_second: f64,
    pub cache_hit_rate: f64,
    pub last_refresh: Option<DateTime<Utc>>,
    pub healthy: bool,
}

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagConfig {
    /// Default provider configuration
    pub default_user: ProviderConfig,
    /// Additional providers
    pub providers: HashMap<String, ProviderConfig>,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Evaluation configuration
    pub evaluation: EvaluationConfig,
    /// Monitoring configuration
    pub monitoring: FlagMonitoringConfig,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub refresh_interval: Option<u64>,
    pub timeout_ms: u64,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Cache configuration for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_entries: u32,
    pub cache_evaluations: bool,
    pub cache_flags: bool,
}

/// Evaluation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub default_timeout_ms: u64,
    pub enable_analytics: bool,
    pub enable_debugging: bool,
    pub max_rule_depth: u32,
    pub enable_context_enrichment: bool,
}

/// Monitoring configuration for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagMonitoringConfig {
    pub enabled: bool,
    pub metrics_interval: u64,
    pub alert_on_errors: bool,
    pub track_performance: bool,
    pub export_evaluations: bool,
}

impl Default for FeatureFlagConfig {
    fn default() -> Self {
        Self {
            default_user: ProviderConfig {
                provider_type: "memory".to_string(),
                endpoint: None,
                api_key: None,
                refresh_interval: Some(300),
                timeout_ms: 5000,
                settings: HashMap::new(),
            },
            providers: HashMap::new(),
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 300,
                max_entries: 10000,
                cache_evaluations: true,
                cache_flags: true,
            },
            evaluation: EvaluationConfig {
                default_timeout_ms: 1000,
                enable_analytics: true,
                enable_debugging: false,
                max_rule_depth: 10,
                enable_context_enrichment: true,
            },
            monitoring: FlagMonitoringConfig {
                enabled: true,
                metrics_interval: 60,
                alert_on_errors: true,
                track_performance: true,
                export_evaluations: false,
            },
        }
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            user_id: None,
            service_id: None,
            request_id: None,
            environment: None,
            version: None,
            geo_location: None,
            device_info: None,
            attributes: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}
