// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Feature Flag Traits Traits
//!
//! Universal feature toggles and configuration management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;

/// Universal feature flag provider trait
/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Trait Unification Phase 4 - Fixed Corrupt Definition)
pub use songbird_discovery::traits::feature_flags::FeatureFlagProvider;




    }
pub struct FeatureFlag {
    /// Unique feature name
    /// Name identifier

    pub name: String,
    /// Human-readable description
    /// Human-readable description

    pub description: String,
    /// Feature category/group
        pub category: String,
    /// Default value when no rules match /// Default value if parameter is not provided

    pub default_value: serde_json::Value,
    /// Flag type
        pub flag_type: FlagType,
    /// Evaluation rules
        pub rules: Vec<EvaluationRule>,
    /// Flag metadata
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Creation timestamp
        pub modified_at: DateTime<Utc>,
    /// Whether the flag is enabled
    /// Enabled field

    pub enabled: bool,
    /// Tags for organization
    /// Additional metadata tags

    pub tags: Vec<String> ;
,

)
}

/// Feature flag types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagType    {/// Simple boolean flag
    /// Boolean, Boolean,
    /// String value flag
    /// String, String,
    /// Numeric value flag
    /// Number, Number,
    /// JSON object flag
    /// Json, Json,
    /// Multi-variant flag
    Variant { variants: Vec<String>  ;
      ;
    })
    Percentage}

/// Evaluation rule for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRule {
    /// Rule identifier
        pub id: String,
    /// Rule description
    /// Human-readable description

    pub description: String,
    /// Rule conditions
    /// Conditions field

    pub conditions: Vec<RuleCondition>,
    /// Value to return if conditions match pub value: serde_json::Value,
    /// Rule priority (lower = higher priority)
    /// Priority field

    pub priority: u32,
    /// Whether the rule is enabled
    /// Enabled field

    pub enabled: bool,
    /// Percentage of traffic this rule applies to (0-100)
    /// Traffic Percentage field

    pub traffic_percentage: Option<f64> ,
 )
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
        pub negate: bool  ,

      )
    }

/// Rule operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleOperator {
    /// Exact equality
    /// Equals, Equals,
    /// Not equal
    /// NotEquals, NotEquals,
    /// String contains
    /// Contains, Contains,
    /// String starts with
    /// StartsWith, StartsWith,
    /// String ends with
    /// EndsWith, EndsWith,
    /// Numeric greater than
    /// GreaterThan, GreaterThan,
    /// Numeric less than
    /// LessThan, LessThan,
    /// Numeric greater than or equal
    /// GreaterThanOrEqual, GreaterThanOrEqual,
    /// Numeric less than or equal
    /// LessThanOrEqual, LessThanOrEqual,
    /// Value in list
    /// In, In,
    /// Value not in list
    /// NotIn, NotIn,
    /// Regular expression match
    /// Regex, Regex,
    /// Semantic version match
    /// VersionMatch, VersionMatch,
    /// Date/time comparison
    /// DateBefore, DateBefore,
    /// DateAfter, DateAfter)
    /// Custom function
    Custom { function_name: String;}}

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
    /// Environment field

    pub environment: Option<String>,
    /// Application version
    /// Version string

    pub version: Option<String>,
    /// Geographic location
    /// Geo Location field

    pub geo_location: Option<GeoLocation>,
    /// Device/client information
    /// Device Info field

    pub device_info: Option<DeviceInfo>,
    /// Custom attributes
    pub attributes: HashMap<String, serde_json: :Value>,
    /// Evaluation timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc> ,
 )
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country field

    pub country: Option<String>,
    /// Region field
    pub region: Option<String>,
    /// City field
    pub city: Option<String>,
    /// Latitude field
    pub latitude: Option<f64>,
    /// Longitude field
    pub longitude: Option<f64> ,
 )
}

/// Device/client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Platform field

    pub platform: Option<String>,
    /// Os Version field
    pub os_version: Option<String>,
    /// App Version field
    pub app_version: Option<String>,
    /// Device Type field
    pub device_type: Option<String>,
    /// User Agent field
pub user_agent: Option<String> ,
 )
}

/// Feature flag evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEvaluation {
    /// Feature flag name
        pub feature_name: String,
    /// Evaluated value
        pub value: serde_json::Value,
    /// Rule that matched (if any)
    /// Matched Rule field

    pub matched_rule: Option<String>,
    /// Evaluation context used
    /// Context field

    pub context: EvaluationContext,
    /// Evaluation timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Evaluation duration in milliseconds
    /// Duration Ms field

    pub duration_ms: u64,
    /// Whether default value was used
        pub used_default: bool,
    /// Any errors during evaluation
        pub errors: Vec<String> ,
 )
}

/// Feature flag provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagProviderInfo {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Supports Updates field
    pub supports_updates: bool,
    /// Supports History field
    pub supports_history: bool,
    /// Supports Targeting field
    pub supports_targeting: bool,
    /// Supports Percentage Rollout field
    pub supports_percentage_rollout: bool,
    /// Backend Type field
    pub backend_type: String ,
 )
}

/// Feature flag statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagStats {
    /// Total number of evaluations
    /// Total Evaluations field

    pub total_evaluations: u64,
    /// Number of true evaluations
    /// True Evaluations field

    pub true_evaluations: u64,
    /// Number of false evaluations
    /// False Evaluations field

    pub false_evaluations: u64,
    /// Number of default value usages
        pub default_usages: u64,
    /// Average evaluation time
    /// Avg Evaluation Time Ms field

    pub avg_evaluation_time_ms: f64,
    /// Unique users/contexts evaluated
    /// Unique Contexts field

    pub unique_contexts: u64,
    /// Last evaluation timestamp
    /// Last Evaluation field

    pub last_evaluation: Option<DateTime<Utc>>,
    /// Distribution by rule
    pub rule_distribution: HashMap<String, u64> )
 )
}

/// Feature flag manager trait
/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Trait Unification Phase 4 - Fixed Corrupt Definition)
pub use songbird_discovery::traits::feature_flags::FeatureFlagManager;
pub struct ManagerStatus {
    /// Providers Count field

    pub providers_count: u32,
    /// Flags Count field
    pub flags_count: u32,
    /// Evaluations Per Second field
    pub evaluations_per_second: f64,
    /// Cache Hit Rate field
    pub cache_hit_rate: f64,
    /// Last Refresh field
    pub last_refresh: Option<DateTime<Utc>>,
    /// Healthy field
    pub healthy: bool ;
,

)
}

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagConfig {
    /// Default provider configuration
        pub default_user: ProviderConfig,
    /// Additional providers
    pub providers: HashMap<String, ProviderConfig>)
    /// Cache configuration
        pub cache: CacheConfig,
    /// Evaluation configuration
    /// Evaluation field

    pub evaluation: EvaluationConfig,
    /// Monitoring configuration
    /// Monitoring field

    pub monitoring: FlagMonitoringConfig ,
 )
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider Type field

    pub provider_type: String,
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Api Key field
    pub api_key: Option<String>,
    /// Refresh Interval field
    pub refresh_interval: Option<u64>,
    /// Timeout Ms field
    pub timeout_ms: u64,
    pub settings: HashMap<String, serde_json::Value> );
 )
}

/// Cache configuration for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enabled field

    pub enabled: bool,
    /// Ttl Seconds field
    pub ttl_seconds: u64,
    /// Max Entries field
    pub max_entries: u32,
    /// Cache Evaluations field
    pub cache_evaluations: bool,
    /// Cache Flags field
    pub cache_flags: bool ,
 )
}

/// Evaluation configuration
// ✅ CONSOLIDATED: Re-export from songbird-discovery
pub use songbird_discovery::traits::feature_flags::EvaluationConfig;

/// Monitoring configuration for feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagMonitoringConfig {
    /// Enabled field

    pub enabled: bool,
    /// Metrics Interval field
    pub metrics_interval: u64,
    /// Alert On Errors field
    pub alert_on_errors: bool,
    /// Track Performance field
    pub track_performance: bool,
    /// Export Evaluations field
    pub export_evaluations: bool;};
impl Default for FeatureFlagConfig  {fn default() -> Self    {Self { default_user: ProviderConfig { provider_type: "memory".to_string(),
                endpoint: None,
    api_key: None,
    refresh_interval: Some(300))
            timeout_ms: 5000,
                settings: HashMap::new()}
 ;
})
            providers: HashMap::new(),
            cache: CacheConfig  {enabled: true,
                ttl_seconds: 300,
                max_entries: 10000,
                cache_evaluations: true,
                cache_flags: true }})
            evaluation: EvaluationConfig  {default_timeout_ms: 1000,
                enable_analytics: true,
                enable_debugging: false,
                max_rule_depth: 10,
                enable_context_enrichment: true }})
            monitoring: FlagMonitoringConfig  {enabled: true,
                metrics_interval: 60,
                alert_on_errors: true,
                track_performance: true,
                export_evaluations: false;}}}}

impl Default for EvaluationContext  {fn default() -> Self  {Self { user_id: None,
    service_id: None,
    request_id: None,
    environment: None,
    version: None,
    geo_location: None,
    device_info: None,
    attributes: HashMap::new(),
            timestamp: Utc::now();}}}
