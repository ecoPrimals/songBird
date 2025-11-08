//! Hook System Traits
//!
//! Universal event hooks and extensibility patterns

#![allow(async_fn_in_trait)]

use crate::traits::service::{ServiceInfo, ServiceRequest, ServiceResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use std::collections::HashMap;

/// Universal event hook trait
pub trait EventHook: Send + Sync {
    /// Hook name for identification
    fn name(&self) -> &str;

    /// Hook version for compatibility
    fn version(&self) -> &str;

    /// Hook priority for ordering (lower = earlier)
    fn priority(&self) -> u32;

    /// Check if hook is enabled
    fn is_enabled(&self) -> bool;

    /// Initialize the hook
    async fn initialize(&mut self, context: &HookContext) -> Result<()>;

    /// Handle an orchestrator event
    async fn handle_event(&self, event: &OrchestratorEvent) -> Result<HookResult>;

    /// Cleanup hook resources
    async fn cleanup(&self) -> Result<()>;

    /// Get hook configuration
    fn get_config(&self) -> HookConfig;
}

/// Hook execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext  {/// Orchestrator instance ID
    pub orchestrator_id: String,
    /// Hook configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Shared context between hooks
    pub shared_context: HashMap<String, serde_json::Value>,
}

/// Generic orchestrator event for hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent  {/// Service lifecycle events
    ServiceRegistering  {service_info: ServiceInfo,
        timestamp: DateTime<Utc>,
    })
    ServiceRegistered  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    ServiceStarting  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    ServiceStarted  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    ServiceStopping  {service_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    })
    ServiceStopped  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    ServiceUnregistering  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    ServiceUnregistered  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    /// Request lifecycle events
    RequestReceived  {request: ServiceRequest,
        timestamp: DateTime<Utc>,
    })
    RequestProcessing  {request_id: String,
        service_id: String,
        timestamp: DateTime<Utc>,
    })
    RequestCompleted  {request_id: String,
        service_id: String,
        response: ServiceResponse,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    })
    RequestFailed  {request_id: String,
        service_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    })
    /// Health and monitoring events
    HealthCheckStarted  {service_id: String,
        timestamp: DateTime<Utc>,
    })
    HealthCheckCompleted  {service_id: String,
        healthy: bool,
        details: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
    })
    MetricsCollected  {service_id: Option<String>)
        metrics: HashMap<String, f64>,
        timestamp: DateTime<Utc>,
    })
    /// Discovery events
    ServiceDiscovered  {service_info: ServiceInfo,
        discovery_source: String,
        timestamp: DateTime<Utc>,
    })
    ServiceLost  {service_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    })
    /// Configuration events
    ConfigurationChanged  {config_section: String,
        old_config: serde_json::Value,
        new_config: serde_json::Value,
        timestamp: DateTime<Utc>,
    })
    /// Error events
    ErrorOccurred  {error_type: String,
        error_message: String,
        context: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
    })
    /// Custom events
    Custom  {event_type: String,
        data: HashMap<String, serde_json::Value>,
        timestamp: DateTime<Utc>,
    })
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult  {/// Whether the hook execution was successful
    pub success: bool,
    /// Whether to continue processing other hooks
    pub continue_chain: bool,
    /// Whether to allow the original operation to continue
    pub allow_operation: bool,
    /// Optional modifications to the event/context
    pub modifications: Option<HashMap<String, serde_json::Value>>,
    /// Log messages from the hook
    pub log_messages: Vec<String>,
    /// Execution duration
    pub execution_time_ms: u64,
    /// Error message if execution failed
    pub error: Option<String>,
}

impl Default for HookResult  {fn default() -> Self  {Self {
            success: true,
            continue_chain: true,
            allow_operation: true,
            modifications: None,
            log_messages: Vec::new(),
            execution_time_ms: 0,
            error: None,
        }
    }
}

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig  {/// Hook-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Event filter - which events this hook cares about
    pub event_filter: EventFilter,
    /// Execution settings
    pub execution: ExecutionConfig,
    /// Retry settings
    pub retry: RetryConfig,
}

/// Event filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter  {/// Event types to process (empty = all)
    pub event_types: Vec<String>,
    /// Service IDs to process (empty = all)
    pub service_ids: Vec<String>,
    /// Custom filter conditions
    pub conditions: Vec<FilterCondition>,
}

/// Filter condition for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition  {pub message: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator  {Equals)
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    In,
    NotIn,
}

/// Hook execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig  {/// Whether to execute asynchronously
    pub async_execution: bool,
    /// Maximum execution time
    pub timeout_ms: u64,
    /// Whether to log execution details
    pub log_execution: bool,
    /// Whether to measure performance
    pub measure_performance: bool,
}

/// Hook retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig  {/// Enable retries on failure
    pub enabled: bool,
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Delay between retries
    pub retry_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Hook manager trait for managing multiple hooks
pub trait HookManager: Send + Sync {
    /// Register a new hook
    async fn register_hook(&mut self, hook: Box<dyn EventHook>) -> Result<()>;

    /// Unregister a hook by name
    async fn unregister_hook(&mut self, hook_name: &str) -> Result<()>;

    /// Get list of registered hooks
    fn list_hooks(&self) -> Vec<HookInfo>;

    /// Execute hooks for an event
    async fn execute_hooks(&self, event: &OrchestratorEvent) -> Result<Vec<HookResult>>;

    /// Enable/disable a hook
    async fn set_hook_enabled(&mut self, hook_name: &str, enabled: bool) -> Result<()>;

    /// Get hook statistics
    async fn get_hook_stats(&self) -> Result<HashMap<String, HookStats>>;

    /// Cleanup all hooks
    async fn cleanup_all(&self) -> Result<()>;
}

/// Information about a registered hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInfo  {pub name: String,
    pub version: String,
    pub priority: u32,
    pub enabled: bool,
    pub registered_at: DateTime<Utc>,
}

/// Statistics for hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookStats  {pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub total_execution_time_ms: u64,
    pub last_execution: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

/// Lifecycle hook trait for specific service operations
pub trait LifecycleHook: Send + Sync {
    /// Before service registration
    async fn before_service_register(&self, service_info: &ServiceInfo) -> Result<HookResult>;

    /// After service registration
    async fn after_service_register(
        &self,
        service_id: &str,
        service_info: &ServiceInfo,
    ) -> Result<HookResult>;

    /// Before service start
    async fn before_service_start(&self, service_id: &str) -> Result<HookResult>;

    /// After service start
    async fn after_service_start(&self, service_id: &str) -> Result<HookResult>;

    /// Before service stop
    async fn before_service_stop(&self, service_id: &str) -> Result<HookResult>;

    /// After service stop
    async fn after_service_stop(&self, service_id: &str) -> Result<HookResult>;

    /// Before request processing
    async fn before_request(
        &self)
        service_id: &str,
        request: &ServiceRequest,
    ) -> Result<HookResult>;

    /// After request processing
    async fn after_request(
        &self)
        service_id: &str,
        request: &ServiceRequest,
        response: &ServiceResponse,
    ) -> Result<HookResult>;

    /// Before health check
    async fn before_health_check(&self, service_id: &str) -> Result<HookResult>;

    /// After health check
    async fn after_health_check(&self, service_id: &str, healthy: bool) -> Result<HookResult>;
}

/// Hook system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSystemConfig  {/// Whether the hook system is enabled
    pub enabled: bool,
    /// Maximum number of hooks
    pub max_hooks: u32,
    /// Default execution timeout
    pub default_timeout_ms: u64,
    /// Whether to log hook executions
    pub log_executions: bool,
    /// Whether to measure hook performance
    pub measure_performance: bool,
    /// Hook execution strategy
    pub execution_strategy: HookExecutionStrategy,
    /// Error handling strategy
    pub error_handling: HookErrorHandling,
}

/// Hook execution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookExecutionStrategy  {/// Execute all hooks in sequence
    Sequential,
    /// Execute all hooks in parallel
    Parallel,
    /// Execute hooks in priority order
    Priority,
    /// Execute hooks with load balancing
    LoadBalanced,
}

/// Hook error handling strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookErrorHandling  {/// Continue on errors
    Continue,
    /// Stop on first error
    StopOnError,
    /// Retry failed hooks
    RetryOnError,
    /// Skip failed hooks
    SkipOnError,
}

impl Default for HookSystemConfig  {fn default() -> Self  {Self {
            enabled: true,
            max_hooks: 100,
            default_timeout_ms: 5000,
            log_executions: true,
            measure_performance: true,
            execution_strategy: HookExecutionStrategy::Priority,
            error_handling: HookErrorHandling::Continue,
        }
    }
}

impl Default for HookConfig  {fn default() -> Self  {Self {
            settings: HashMap::new(),
            event_filter: EventFilter {
                event_types: Vec::new(),
                service_ids: Vec::new(),
                conditions: Vec::new(),
            })
            execution: ExecutionConfig  {async_execution: true,
                timeout_ms: 5000,
                log_execution: true,
                measure_performance: true,
            })
            retry: RetryConfig  {enabled: false,
                max_attempts: 3,
                retry_delay_ms: 1000,
                backoff_multiplier: 2.0,
            })
        }
    }
}
