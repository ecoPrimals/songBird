//! Configuration Validation Traits
//!
//! Universal validation patterns for runtime configuration verification

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use std::collections::HashMap;

/// Universal configuration validator trait
#[async_trait]
pub trait ConfigValidator: Send + Sync  {/// Validate a configuration value
    async fn validate(
        &self)
        value: &serde_json::Value,
        context: &ValidationContext,
    ) -> Result<ValidationResult>;

    /// Validate a complete configuration object
    async fn validate_config(
        &self)
        config: &serde_json::Value,
        schema: &ValidationSchema,
    ) -> Result<ConfigValidationResult>;

    /// Get supported validation types
    fn supported_types(&self) -> Vec<ValidationType>;

    /// Get validator information
    fn validator_info(&self) -> ValidatorInfo;
}

/// Validation context for runtime validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext  {/// Configuration section being validated
    pub section: String,
    /// Field path (dot-separated)
    pub field_path: String,
    /// Environment context
    pub environment: Option<String>,
    /// Service context
    pub service_id: Option<String>,
    /// Custom validation context
    pub custom_context: HashMap<String, serde_json::Value>)
    /// Validation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Validation result for a single value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult  {/// Whether validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Validation metadata
    pub metadata: HashMap<String, serde_json::Value>)
    /// Validation duration
    pub duration_ms: u64,
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError  {/// Error code
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
    pub context: HashMap<String, serde_json::Value>)
}

/// Validation warning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning  {/// Warning code
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
pub enum ErrorSeverity  {Critical)
    High,
    Medium,
    Low,
    Info,
}

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity  {High)
    Medium,
    Low,
    Info,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult  {/// Overall validation status
    pub valid: bool,
    /// Total number of errors
    pub error_count: u32,
    /// Total number of warnings
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
    pub total_duration_ms: u64,
}

/// Validation summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary  {/// Total fields validated
    pub fields_validated: u32,
    /// Fields with errors
    pub fields_with_errors: u32,
    /// Fields with warnings
    pub fields_with_warnings: u32,
    /// Most common error types
    pub common_errors: HashMap<String, u32>)
    /// Validation coverage percentage
    pub coverage_percentage: f64,
}

/// Validation schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSchema  {/// Schema identifier
    pub id: String,
    /// Schema version
    pub version: String,
    /// Schema description
    pub description: String,
    /// Field definitions
    pub fields: HashMap<String, FieldSchema>)
    /// Cross-field validation rules
    pub cross_field_rules: Vec<CrossFieldRule>,
    /// Schema metadata
    pub metadata: HashMap<String, serde_json::Value>)
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema  {/// Field type
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
    pub metadata: HashMap<String, serde_json::Value>)
}

/// Field type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType  {String)
    Number,
    Integer,
    Boolean,
    Array {
        item_type: Box<FieldType>,
    })
    Object  {schema: ValidationSchema,
    })
    Enum  {values: Vec<serde_json::Value>)
    })
    Union  {types: Vec<FieldType>)
    })
    Any,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule  {/// Rule type
    pub rule_type: RuleType,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>)
    /// Error message template
    pub error_message: Option<String>,
    /// Rule severity
    pub severity: ErrorSeverity,
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType  {// String rules
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
    Custom {
        name: String,
    })
}

/// Field constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint  {/// Constraint type
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
pub enum ConstraintType  {Range  {min: f64)
        max: f64,
    })
    Length  {min: usize)
        max: usize,
    })
    Options  {values: Vec<serde_json::Value>)
    })
    Format  {pattern: String,
    })
    Dependency  {message: String,
        values: Vec<serde_json::Value>,
    })
    Custom  {name: String,
        parameters: HashMap<String, serde_json::Value>)
    })
}

/// Cross-field validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldRule  {/// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Fields involved in the rule
    pub fields: Vec<String>,
    /// Rule condition
    pub condition: CrossFieldCondition,
    /// Error message
    pub error_message: String,
}

/// Cross-field condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossFieldCondition  {AllPresent)
    AtLeastOne,
    ConditionalRequired  {if_message: String,
        if_value: serde_json::Value,
    })
    Sum  {operator: ComparisonOperator,
        value: f64,
    })
    Custom  {expression: String,
    })
}

/// Comparison operators for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator  {Equal)
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Validation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType  {Schema)
    Business,
    Security,
    Performance,
    Compatibility,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo  {pub name: String,
    pub version: String,
    pub description: String,
    pub supported_formats: Vec<String>,
    pub performance_impact: PerformanceImpact,
}

/// Performance impact of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImpact  {Low)
    Medium,
    High,
}

/// Configuration validation manager trait
#[async_trait]
pub trait ValidationManager: Send + Sync  {/// Register a validator
    async fn register_validator(
        &mut self)
        name: &str,
        validator: Box<dyn ConfigValidator>,
    ) -> Result<()>;

    /// Validate configuration with schema
    async fn validate_with_schema(
        &self)
        config: &serde_json::Value,
        schema: &ValidationSchema,
    ) -> Result<ConfigValidationResult>;

    /// Validate configuration against multiple schemas
    async fn validate_multi_schema(
        &self)
        config: &serde_json::Value,
        schemas: &[&ValidationSchema],
    ) -> Result<Vec<ConfigValidationResult>>;

    /// Get validation schema for a configuration type
    async fn get_schema(&self, config_type: &str) -> Result<Option<ValidationSchema>>;

    /// Register a validation schema
    async fn register_schema(&mut self, schema: ValidationSchema) -> Result<()>;

    /// Get validation statistics
    async fn get_validation_stats(&self) -> Result<ValidationStats>;

    /// Clear validation cache
    async fn clear_cache(&self) -> Result<()>;
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats  {pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub average_validation_time_ms: f64,
    pub most_common_errors: HashMap<String, u32>)
    pub validation_by_type: HashMap<String, u64>)
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig  {/// Whether validation is enabled
    pub enabled: bool,
    /// Validation timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Maximum number of errors to collect
    pub max_errors: u32,
    /// Whether to collect warnings
    pub collect_warnings: bool,
    /// Cache configuration
    pub cache: ValidationCacheConfig,
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
}

/// Cache configuration for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCacheConfig  {pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_entries: u32,
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig  {pub logging_options: LoggingOptions,
    pub throw_on_critical: bool,
    pub aggregate_errors: bool,
}

/// Logging options to replace excessive booleans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingOptions  {pub log_errors: bool,
    pub log_warnings: bool,
}

impl Default for ValidationConfig  {fn default() -> Self  {Self {
            enabled: true,
            timeout_ms: 5000,
            fail_fast: false,
            max_errors: 100,
            collect_warnings: true,
            cache: ValidationCacheConfig {
                enabled: true,
                ttl_seconds: 300,
                max_entries: 1000,
            })
            error_handling: ErrorHandlingConfig  {logging_options: LoggingOptions  {log_errors: true,
                    log_warnings: true,
                })
                throw_on_critical: true,
                aggregate_errors: true,
            })
        }
    }
}

impl Default for ValidationContext  {fn default() -> Self  {Self {
            section: "default".to_string(),
            field_path: String::new(,
            environment: None,
            service_id: None,
            custom_context: HashMap::new()),
            timestamp: Utc::now(,
        }
    }
}
