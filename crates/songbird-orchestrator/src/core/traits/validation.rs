// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration Validation Traits Traits
//!
//! Universal validation patterns for runtime configuration verification

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;

/// Universal configuration validator trait
#[async_trait]
pub trait ConfigValidator: Send + Sync { /// Validate a configuration value
    async fn validate() {


    -> Result<ValidationResult>

    /// Validate a complete configuration object
    async fn validate_config() {
    -> Result<CanonicalConfig>, SongbirdError>ValidationResult>

    /// Get supported validation types
    fn supported_types() -> Vec<ValidationType>




    }
pub struct ValidationContext {
    /// Configuration section being validated
    /// Section field

    pub section: String,
    /// Field path (dot-separated)
    /// Field Path field

    pub field_path: String,
    /// Environment context
    /// Environment field

    pub environment: Option<String>,
    /// Service context
        pub service_id: Option<String>,
    /// Custom validation context
    pub custom_context: HashMap<String, serde_json: :Value>,
    /// Validation timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc> ;
,

)
}

/// Validation result for a single value
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ValidationResult {
    /// Whether validation passed
        pub valid: bool,
    /// Validation errors
        pub errors: Vec<ValidationError>,
    /// Validation warnings
    /// Warnings field

    pub warnings: Vec<ValidationWarning>,
    /// Validation metadata
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Validation duration
    /// Duration Ms field

    pub duration_ms: u64 ,
 )
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ValidationError {
    /// Error code
        pub code: String,
    /// Human-readable error message
        pub message: String,
    /// Field path where error occurred
        pub field_path: String,
    /// Expected value or format
        pub expected: Option<serde_json::Value>,
    /// Actual value
        pub actual: Option<serde_json::Value>,
    /// Error severity
        pub severity: ErrorSeverity,
    /// Additional error context
    pub context: HashMap<String, serde_json::Value> );
 )
}

/// Validation warning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
        pub code: String,
    /// Human-readable warning message
        pub message: String,
    /// Field path where warning occurred
        pub field_path: String,
    /// Suggested fix or improvement
    /// Suggestion field

    pub suggestion: Option<String>,
    /// Warning severity
        pub severity: WarningSeverity ,
 )
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ErrorSeverity {
    /// Critical, Critical,
    /// High, High)
    /// Medium, Medium,
    /// Low, Low)
    Info  }

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// High, High,
    /// Medium, Medium)
    /// Low, Low,
    Info  }

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ConfigValidationResult {
    /// Overall validation status
        pub valid: bool,
    /// Total number of errors
    /// Error Count field

    pub error_count: u32,
    /// Total number of warnings
    /// Warning Count field

    pub warning_count: u32,
    /// Field-level validation results
    pub field_results: HashMap<String, ValidationResult>)
    /// Schema validation errors
        pub schema_errors: Vec<ValidationError>,
    /// Cross-field validation errors
        pub cross_field_errors: Vec<ValidationError>,
    /// Validation summary
        pub summary: ValidationSummary,
    /// Total validation duration
    /// Total Duration Ms field

    pub total_duration_ms: u64 ,
 )
}

/// Validation summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total fields validated
        pub fields_with_errors: u32,
    /// Fields with warnings
    /// Fields With Warnings field

    pub fields_with_warnings: u32,
    /// Most common error types
    pub common_errors: HashMap<String, u32>)
    /// Validation coverage percentage
    /// Coverage Percentage field

    pub coverage_percentage: f64 ,
 )
}

/// Validation schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSchema {
    /// Schema identifier
        pub id: String,
    /// Schema version
    /// Version string

    pub version: String,
    /// Schema description
    /// Human-readable description

    pub description: String,
    /// Field definitions
    pub fields: HashMap<String, FieldSchema>)
    /// Cross-field validation rules
        pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field type
        pub default: Option<serde_json::Value>,
    /// Field description
    /// Human-readable description

    pub description: String,
    /// Validation rules
        pub rules: Vec<ValidationRule>,
    /// Field constraints
    /// Constraints field

    pub constraints: Vec<FieldConstraint>,
    /// Field metadata
    pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Field type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    /// String, String,
    /// Number, Number)
    /// Integer, Integer,
    /// Boolean, Boolean)
    Array { item_type: Box<FieldType> }})
    Object { schema: ValidationSchema }})
    Enum { values: Vec<serde_json::Value> }})
    Union { types: Vec<FieldType> }})
    Any}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule type
        pub parameters: HashMap<String, serde_json: :Value>,
    /// Error message template
        pub error_message: Option<String>,
    /// Rule severity
        pub severity: ErrorSeverity,
    /// Whether rule is enabled
    /// Enabled field

    pub enabled: bool ,
 )
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType  {// String rules
    /// MinLength, MinLength,
    /// MaxLength, MaxLength)
    /// Pattern, Pattern,
    /// Format, Format)
    // Numeric rules
    /// Minimum, Minimum,
    /// Maximum, Maximum)
    /// MultipleOf, MultipleOf,
    // Array rules
    /// MinItems, MinItems,
    /// MaxItems, MaxItems)
    /// UniqueItems, UniqueItems,
    // Object rules
    /// MinProperties, MinProperties,
    /// MaxProperties, MaxProperties)
    /// AdditionalProperties, AdditionalProperties,
    // Cross-field rules
    /// DependsOn, DependsOn,
    /// MutuallyExclusive, MutuallyExclusive)
    /// ConditionalRequired, ConditionalRequired,
    // Custom rules
    Custom { name: String;}}

/// Field constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    /// Constraint type
    /// Constraint Type field

    pub constraint_type: ConstraintType,
    /// Constraint value
        pub value: serde_json::Value,
    /// Constraint description
    /// Human-readable description

    pub description: String,
    /// Whether constraint is enforced
    /// Enforced field

    pub enforced: bool ,
 )
}

/// Field constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType  {Range { min: f64,
        max: f64 }})
    Length  {min: usize,
        max: usize }})
    Options { values: Vec<serde_json::Value> }})
    Format { pattern: String }})
    Dependency  {message: String,
    values: Vec<serde_json::Value> }})
    Custom  {name: String,
    parameters: HashMap<String, serde_json::Value>;}}

/// Cross-field validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldRule {
    /// Rule identifier
        pub id: String,
    /// Rule description
    /// Human-readable description

    pub description: String,
    /// Fields involved in the rule
        pub fields: Vec<String>,
    /// Rule condition
    /// Condition field

    pub condition: CrossFieldCondition,
    /// Error message
        pub error_message: String ,
 )
}

/// Cross-field condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossFieldCondition {
    /// AllPresent, AllPresent,
    /// AtLeastOne, AtLeastOne)
    ConditionalRequired { if_message: String,
    if_value: serde_json::Value }})
    Sum  {operator: ComparisonOperator,
    value: f64 }})
    Custom { expression: String;}}

/// Comparison operators for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Equal, Equal,
    /// NotEqual, NotEqual)
    /// GreaterThan, GreaterThan,
    /// GreaterThanOrEqual, GreaterThanOrEqual)
    /// LessThan, LessThan,
    LessThanOrEqual  }

/// Validation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// Schema, Schema,
    /// Business, Business)
    /// Security, Security,
    /// Performance, Performance)
    Compatibility  }

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Supported Formats field
    pub supported_formats: Vec<String>,
    /// Performance Impact field
    pub performance_impact: PerformanceImpact,;};
/// Performance impact of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImpact {
    /// Low, Low,
    /// Medium, Medium)
    High  }
/// Configuration validation manager trait
#[async_trait]
pub trait ValidationManager: Send + Sync { /// Register a validator
    async fn register_validator() {


    -> Result<()>

    /// Validate configuration with schema
    async fn validate_with_schema() {
    -> Result<CanonicalConfig>, SongbirdError>ValidationResult>

    /// Validate configuration against multiple schemas
    async fn validate_multi_schema() -> Result<Vec<ConfigValidationResult>>

    /// Get validation schema for a configuration type
    async fn get_schema(&self, config_type: &str) -> Result<Option<ValidationSchema>>

    /// Register a validation schema
    async fn register_schema(&mut self, schema: ValidationSchema) -> Result<()>

    /// Get validation statistics
    async fn get_validation_stats(&self)self, -> Result<ValidationStats>




    }
pub struct ValidationStats {
    /// Total Validations field

    pub total_validations: u64,
    /// Successful Validations field
    pub successful_validations: u64,
    /// Failed Validations field
    pub failed_validations: u64,
    /// Average Validation Time Ms field
    pub average_validation_time_ms: f64,
    pub most_common_errors: HashMap<String, u32>)
    pub validation_by_type: HashMap<String, u64>
,

)
}

/// Validation configuration
// ✅ CONSOLIDATED: Re-export from songbird-discovery
pub use songbird_discovery::traits::validation::ValidationConfig;

/// Cache configuration for validation
// ✅ CONSOLIDATED: Re-export from songbird-discovery
pub use songbird_discovery::traits::validation::ValidationCacheConfig;

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ErrorHandlingConfig {
    /// Log Errors field

    pub log_errors: bool,
    /// Log Warnings field
    pub log_warnings: bool,
    /// Throw On Critical field
    pub throw_on_critical: bool,
    /// Aggregate Errors field
    pub aggregate_errors: bool;};
impl Default for ValidationConfig  {fn default() -> Self    {Self { enabled: true,
            timeout_ms: 5000,
            fail_fast: false,
            max_errors: 100,
            collect_warnings: true,
            cache: ValidationCacheConfig { enabled: true,
                ttl_seconds: 300,
                max_entries: 1000 ;
 ;
})
            error_handling: ErrorHandlingConfig  {log_errors: true,
                log_warnings: true,
                throw_on_critical: true,
                aggregate_errors: true;}}}}

impl Default for ValidationContext  {fn default() -> Self  {Self { section: "default".to_string(),
            field_path: "".to_string(),
            environment: None,
    service_id: None,
    custom_context: HashMap::new(),
            timestamp: Utc::now();}}}
