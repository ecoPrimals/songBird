//! Configuration Validation Traits
//!
//! Universal validation patterns for runtime configuration verification

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::Result;

/// Universal configuration validator trait
#[async_trait]
pub trait ConfigValidator: Send + Sync {
    /// Validate a configuration value
    async fn validate(&self, value: &serde_json::Value, context: &ValidationContext) -> Result<ValidationResult>;
    
    /// Validate a complete configuration object
    async fn validate_config(&self, config: &serde_json::Value, schema: &ValidationSchema) -> Result<ConfigValidationResult>;
    
    /// Get supported validation types
    fn supported_types(&self) -> Vec<ValidationType>;
    
    /// Get validator information
    fn validator_info(&self) -> ValidatorInfo;
}

/// Validation context for runtime validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    /// Configuration section being validated
    pub section: String,
    
    /// Field path (dot-separated)
    pub field_path: String,
    
    /// Environment context
    pub environment: Option<String>,
    
    /// Service context
    pub service_id: Option<String>,
    
    /// Custom validation context
    pub custom_context: HashMap<String, serde_json::Value>,
    
    /// Validation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Validation result for a single value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    
    /// Validation errors
    pub errors: Vec<ValidationError>,
    
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    
    /// Validation metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Validation duration
    pub duration_ms: u64,
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub context: HashMap<String, serde_json::Value>,
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
    pub suggestion: Option<String>,
    
    /// Warning severity
    pub severity: WarningSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    High,
    Medium,
    Low,
    Info,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    /// Overall validation status
    pub valid: bool,
    
    /// Total number of errors
    pub error_count: u32,
    
    /// Total number of warnings
    pub warning_count: u32,
    
    /// Field-level validation results
    pub field_results: HashMap<String, ValidationResult>,
    
    /// Schema validation errors
    pub schema_errors: Vec<ValidationError>,
    
    /// Cross-field validation errors
    pub cross_field_errors: Vec<ValidationError>,
    
    /// Validation summary
    pub summary: ValidationSummary,
    
    /// Total validation duration
    pub total_duration_ms: u64,
}

/// Validation summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total fields validated
    pub fields_validated: u32,
    
    /// Fields with errors
    pub fields_with_errors: u32,
    
    /// Fields with warnings
    pub fields_with_warnings: u32,
    
    /// Most common error types
    pub common_errors: HashMap<String, u32>,
    
    /// Validation coverage percentage
    pub coverage_percentage: f64,
}

/// Validation schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSchema {
    /// Schema identifier
    pub id: String,
    
    /// Schema version
    pub version: String,
    
    /// Schema description
    pub description: String,
    
    /// Field definitions
    pub fields: HashMap<String, FieldSchema>,
    
    /// Cross-field validation rules
    pub cross_field_rules: Vec<CrossFieldRule>,
    
    /// Schema metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field type
    pub field_type: FieldType,
    
    /// Whether field is required
    pub required: bool,
    
    /// Default value
    pub default: Option<serde_json::Value>,
    
    /// Field description
    pub description: String,
    
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    
    /// Field constraints
    pub constraints: Vec<FieldConstraint>,
    
    /// Field metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Field type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Integer,
    Boolean,
    Array { item_type: Box<FieldType> },
    Object { schema: ValidationSchema },
    Enum { values: Vec<serde_json::Value> },
    Union { types: Vec<FieldType> },
    Any,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule type
    pub rule_type: RuleType,
    
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Error message template
    pub error_message: Option<String>,
    
    /// Rule severity
    pub severity: ErrorSeverity,
    
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    // String rules
    MinLength,
    MaxLength,
    Pattern,
    Format,
    
    // Numeric rules
    Minimum,
    Maximum,
    MultipleOf,
    
    // Array rules
    MinItems,
    MaxItems,
    UniqueItems,
    
    // Object rules
    MinProperties,
    MaxProperties,
    AdditionalProperties,
    
    // Cross-field rules
    DependsOn,
    MutuallyExclusive,
    ConditionalRequired,
    
    // Custom rules
    Custom { name: String },
}

/// Field constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    
    /// Constraint value
    pub value: serde_json::Value,
    
    /// Constraint description
    pub description: String,
    
    /// Whether constraint is enforced
    pub enforced: bool,
}

/// Field constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Range { min: f64, max: f64 },
    Length { min: usize, max: usize },
    Options { values: Vec<serde_json::Value> },
    Format { pattern: String },
    Dependency { field: String, values: Vec<serde_json::Value> },
    Custom { name: String, parameters: HashMap<String, serde_json::Value> },
}

/// Cross-field validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldRule {
    /// Rule identifier
    pub id: String,
    
    /// Rule description
    pub description: String,
    
    /// Fields involved in the rule
    pub fields: Vec<String>,
    
    /// Rule condition
    pub condition: CrossFieldCondition,
    
    /// Error message
    pub error_message: String,
    
    /// Rule severity
    pub severity: ErrorSeverity,
}

/// Cross-field condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossFieldCondition {
    AllPresent,
    AtLeastOne,
    MutuallyExclusive,
    ConditionalRequired { if_field: String, if_value: serde_json::Value },
    Sum { operator: ComparisonOperator, value: f64 },
    Custom { expression: String },
}

/// Comparison operators for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Validation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Schema,
    Business,
    Security,
    Performance,
    Compatibility,
    Custom { name: String },
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_formats: Vec<String>,
    pub performance_impact: PerformanceImpact,
}

/// Performance impact of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImpact {
    Low,
    Medium,
    High,
}

/// Configuration validation manager trait
#[async_trait]
pub trait ValidationManager: Send + Sync {
    /// Register a validator
    async fn register_validator(&mut self, name: &str, validator: Box<dyn ConfigValidator>) -> Result<()>;
    
    /// Validate configuration with schema
    async fn validate_with_schema(&self, config: &serde_json::Value, schema: &ValidationSchema) -> Result<ConfigValidationResult>;
    
    /// Validate configuration against multiple schemas
    async fn validate_multi_schema(&self, config: &serde_json::Value, schemas: &[&ValidationSchema]) -> Result<Vec<ConfigValidationResult>>;
    
    /// Get validation schema for a configuration type
    async fn get_schema(&self, config_type: &str) -> Result<Option<ValidationSchema>>;
    
    /// Register a validation schema
    async fn register_schema(&self, schema: ValidationSchema) -> Result<()>;
    
    /// Get validation statistics
    async fn get_validation_stats(&self) -> Result<ValidationStats>;
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub average_validation_time_ms: f64,
    pub schemas_registered: u32,
    pub validators_registered: u32,
    pub common_errors: HashMap<String, u32>,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether validation is enabled
    pub enabled: bool,
    
    /// Validation mode
    pub mode: ValidationMode,
    
    /// Schema configuration
    pub schema: SchemaConfig,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
}

/// Validation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationMode {
    /// Strict validation - all errors fail
    Strict,
    
    /// Lenient validation - warnings only
    Lenient,
    
    /// Fail-fast validation - stop on first error
    FailFast,
    
    /// Best-effort validation - continue on errors
    BestEffort,
}

/// Schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub auto_generate: bool,
    pub cache_schemas: bool,
    pub schema_registry_url: Option<String>,
    pub default_schema_version: String,
}

/// Performance configuration for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_validation_time_ms: u64,
    pub parallel_validation: bool,
    pub cache_results: bool,
    pub max_cache_size: u32,
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub fail_on_warnings: bool,
    pub max_errors_reported: u32,
    pub group_similar_errors: bool,
    pub include_context: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ValidationMode::Strict,
            schema: SchemaConfig {
                auto_generate: false,
                cache_schemas: true,
                schema_registry_url: None,
                default_schema_version: "1.0".to_string(),
            },
            performance: PerformanceConfig {
                max_validation_time_ms: 5000,
                parallel_validation: true,
                cache_results: true,
                max_cache_size: 1000,
            },
            error_handling: ErrorHandlingConfig {
                fail_on_warnings: false,
                max_errors_reported: 100,
                group_similar_errors: true,
                include_context: true,
            },
        }
    }
} 