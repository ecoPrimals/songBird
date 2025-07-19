use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// String interning for model IDs and frequently used strings
#[derive(Debug, Default)]
pub struct ModelStringPool {
    model_ids: HashMap<String, Arc<str>>,
    common_strings: HashMap<String, Arc<str>>,
}

impl ModelStringPool {
    pub fn new() -> Self {
        Self {
            model_ids: HashMap::new(),
            common_strings: HashMap::new(),
        }
    }

    pub fn intern_model_id(&mut self, model_id: &str) -> Arc<str> {
        if let Some(interned) = self.model_ids.get(model_id) {
            Arc::clone(interned)
        } else {
            let interned = Arc::from(model_id);
            self.model_ids
                .insert(model_id.to_string(), Arc::clone(&interned));
            interned
        }
    }

    pub fn intern_string(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.common_strings.get(s) {
            Arc::clone(interned)
        } else {
            let interned = Arc::from(s);
            self.common_strings
                .insert(s.to_string(), Arc::clone(&interned));
            interned
        }
    }

    pub fn lookup_model_id(&self, model_id: &str) -> Option<Arc<str>> {
        self.model_ids.get(model_id).map(Arc::clone)
    }
}

/// Zero-copy AI workload type with interned strings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AiWorkloadType {
    ModelInference,
    Training,
    DataPreprocessing,
    ModelServing,
    BatchProcessing,
    StreamingProcessing,
    AgentCommunication,
}

impl AiWorkloadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiWorkloadType::ModelInference => "model_inference",
            AiWorkloadType::Training => "training",
            AiWorkloadType::DataPreprocessing => "data_preprocessing",
            AiWorkloadType::ModelServing => "model_serving",
            AiWorkloadType::BatchProcessing => "batch_processing",
            AiWorkloadType::StreamingProcessing => "streaming_processing",
            AiWorkloadType::AgentCommunication => "agent_communication",
        }
    }
}

/// Access pattern analysis for predictive caching with reduced allocations
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub last_access: Instant,
    pub access_frequency: f64,
    pub access_intervals: Vec<Duration>,
    pub workload_context: Option<AiWorkloadType>,
    pub prediction_accuracy: f32,
}

impl AccessPattern {
    pub fn new() -> Self {
        Self {
            last_access: Instant::now(),
            access_frequency: 0.0,
            access_intervals: Vec::with_capacity(10), // Pre-allocate
            workload_context: None,
            prediction_accuracy: 0.0,
        }
    }

    pub fn update_access(&mut self, workload_type: Option<AiWorkloadType>) {
        let now = Instant::now();
        let interval = now.duration_since(self.last_access);

        // Keep only recent intervals for efficient analysis
        if self.access_intervals.len() >= 10 {
            self.access_intervals.remove(0);
        }
        self.access_intervals.push(interval);

        self.last_access = now;
        self.workload_context = workload_type;

        // Update frequency (simple moving average)
        if !self.access_intervals.is_empty() {
            let total_time: Duration = self.access_intervals.iter().sum();
            self.access_frequency = self.access_intervals.len() as f64 / total_time.as_secs_f64();
        }
    }
}

/// AI model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub model_type: String,
    pub status: ModelDeploymentStatus,
    pub resource_requirements: ResourceRequirements,
    pub performance_metrics: ModelPerformanceMetrics,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Resource requirements for AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_mb: u64,
    pub cpu_cores: f32,
    pub gpu_memory_mb: Option<u64>,
    pub storage_mb: u64,
    pub network_bandwidth_mbps: f32,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub inference_latency_ms: f32,
    pub throughput_requests_per_sec: f32,
    pub accuracy: Option<f32>,
    pub memory_utilization: f32,
    pub cpu_utilization: f32,
}

/// Model deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelDeploymentStatus {
    Deployed,
    Deploying,
    Failed,
    Stopped,
    Updating,
    Scaling,
}

/// AI inference request
#[derive(Debug, Serialize, Deserialize)]
pub struct AiInferenceRequest {
    pub model_id: String,
    pub input_data: serde_json::Value,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub timeout_ms: Option<u64>,
    pub priority: Option<u8>,
    pub session_id: Option<String>,
}

/// AI inference response
#[derive(Debug, Serialize)]
pub struct AiInferenceResponse {
    pub request_id: String,
    pub model_id: String,
    pub output: serde_json::Value,
    pub processing_time_ms: u64,
    pub confidence: Option<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AI batch request
#[derive(Debug, Deserialize)]
pub struct AiBatchRequest {
    pub model_id: String,
    pub batch_id: Option<String>,
    pub requests: Vec<AiInferenceRequest>,
    pub options: BatchOptions,
}

/// Batch processing options
#[derive(Debug, Deserialize)]
pub struct BatchOptions {
    pub max_batch_size: Option<usize>,
    pub timeout_seconds: Option<u64>,
    pub priority: Option<u8>,
    pub parallel_processing: Option<bool>,
}

/// AI metrics query
#[derive(Debug, Deserialize, Serialize)]
pub struct AiMetricsQuery {
    pub model_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub metric_types: Vec<String>,
    pub aggregation: Option<String>,
}

/// Estimate JSON serialization size to avoid allocations
pub fn estimate_json_size(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Null => 4, // "null"
        serde_json::Value::Bool(b) => {
            if *b {
                4
            } else {
                5
            }
        } // "true" or "false"
        serde_json::Value::Number(n) => n.to_string().len(),
        serde_json::Value::String(s) => s.len() + 2, // Account for quotes
        serde_json::Value::Array(arr) => {
            2 + arr.iter().map(estimate_json_size).sum::<usize>() + arr.len().saturating_sub(1)
        }
        serde_json::Value::Object(obj) => {
            2 + obj
                .iter()
                .map(|(k, v)| k.len() + 3 + estimate_json_size(v))
                .sum::<usize>()
                + obj.len().saturating_sub(1)
        }
    }
}
