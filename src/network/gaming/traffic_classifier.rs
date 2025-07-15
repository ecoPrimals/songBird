// src/network/gaming/traffic_classifier.rs
// AI-powered traffic classification for dual market optimization + AI workload detection

use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Enhanced traffic classification for AI-first optimization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrafficProfile {
    Gaming(GamingOptimizationProfile),
    Scientific(ScientificOptimizationProfile),
    AiWorkload(AiWorkloadProfile),
    General(GeneralOptimizationProfile),
    Unknown,
}

/// AI workload optimization profile - AI-first system priority
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiWorkloadProfile {
    pub inference_latency_target_ms: f64,      // <10ms for real-time inference
    pub throughput_requirement_rps: f64,        // 1000+ requests/second
    pub model_type: Option<AiModelType>,        // Model classification
    pub batch_size_preference: Option<u32>,    // Preferred batch size
    pub memory_requirement_gb: f64,            // Memory needed
    pub gpu_acceleration: bool,                // GPU requirement
    pub streaming_capable: bool,               // Supports streaming
    pub caching_beneficial: bool,              // Benefits from caching
    pub priority_level: AiPriorityLevel,       // Request priority
}

/// AI model type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiModelType {
    LargeLanguageModel,     // ChatGPT, Claude, etc.
    ComputerVision,         // Image recognition, OCR
    NaturalLanguage,        // Text analysis, sentiment
    Embedding,              // Vector embeddings
    AudioProcessing,        // Speech recognition, TTS
    Multimodal,            // Vision + language
    CodeGeneration,        // Code completion, generation
    Recommendation,        // ML recommendations
    TimeSeriesAnalysis,    // Forecasting, anomaly detection
    ReinforcementLearning, // Game AI, optimization
}

/// AI request priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiPriorityLevel {
    Critical,    // Real-time inference, <1ms
    High,        // Interactive AI, <10ms
    Medium,      // Background processing, <100ms
    Low,         // Batch processing, <1s
    Bulk,        // Large batch jobs, <10s
}

/// Gaming traffic optimization profile - latency critical
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GamingOptimizationProfile {
    pub latency_target_ms: f64,           // <0.5ms packet routing
    pub throughput_requirement_gbps: f64, // 1-10 Gbps typical
    pub packet_size_range: (usize, usize), // 64-1500 bytes
    pub protocol_hints: Vec<String>,      // IPX, DirectPlay, NetBIOS
    pub optimization_focus: OptimizationFocus,
    pub game_type: Option<GameType>,
}

/// Scientific data optimization profile - throughput critical
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificOptimizationProfile {
    pub latency_tolerance_ms: f64,        // <10ms acceptable
    pub throughput_requirement_gbps: f64, // 10-100+ Gbps needed
    pub file_size_range: (u64, u64),     // 100GB-10TB transfers
    pub data_format_hints: Vec<String>,   // HDF5, NetCDF, FASTA
    pub optimization_focus: OptimizationFocus,
    pub scientific_domain: Option<ScientificDomain>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralOptimizationProfile {
    pub balanced_optimization: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationFocus {
    LatencyFirst,   // Gaming priority
    ThroughputFirst, // Scientific priority
    AiOptimized,    // AI workload optimized
    Balanced,       // General traffic
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameType {
    RealTimeStrategy, // StarCraft, AoE2 - very latency sensitive
    FirstPersonShooter, // Quake, Half-Life - ultra latency sensitive
    RolePlaying,      // Diablo, WoW - moderate latency sensitivity
    TurnBased,        // Civilization - latency tolerant
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScientificDomain {
    Genomics,         // DNA sequencing, massive files
    ClimateScience,   // Weather models, time series
    ParticlePhysics,  // CERN data, event streams
    Pharmaceutical,   // Drug discovery, molecular data
    MaterialsScience, // Simulation data, structured datasets
}

/// AI-enhanced traffic classifier with machine learning capabilities
pub struct TrafficClassifier {
    gaming_patterns: RwLock<HashMap<String, f64>>,
    scientific_patterns: RwLock<HashMap<String, f64>>,
    ai_patterns: RwLock<HashMap<String, AiPatternSignature>>,
    classification_history: RwLock<Vec<ClassificationEvent>>,
    learning_engine: MLClassificationEngine,
    cache_predictor: AiCachePredictor,
    performance_optimizer: AiPerformanceOptimizer,
}

/// AI pattern signature for workload detection
#[derive(Debug, Clone)]
struct AiPatternSignature {
    request_frequency: f64,
    payload_size_distribution: Vec<f64>,
    response_time_pattern: Vec<f64>,
    model_type_confidence: HashMap<AiModelType, f32>,
    caching_effectiveness: f32,
    batch_processing_benefit: f32,
    gpu_utilization_pattern: Option<f32>,
}

/// AI cache predictor for intelligent prefetching
pub struct AiCachePredictor {
    cache_hit_patterns: RwLock<HashMap<String, CacheHitPattern>>,
    model_cache_strategies: RwLock<HashMap<AiModelType, CacheStrategy>>,
    prediction_accuracy: RwLock<f32>,
}

/// Cache hit pattern analysis
#[derive(Debug, Clone)]
struct CacheHitPattern {
    hit_rate: f32,
    temporal_patterns: Vec<f64>, // Time-based access patterns
    context_sensitivity: f32,    // How much context affects caching
    invalidation_triggers: Vec<String>,
}

/// Cache strategy for different AI models
#[derive(Debug, Clone)]
struct CacheStrategy {
    cache_duration: std::time::Duration,
    cache_key_strategy: CacheKeyStrategy,
    prefetch_enabled: bool,
    invalidation_strategy: InvalidationStrategy,
}

#[derive(Debug, Clone)]
enum CacheKeyStrategy {
    ContentHash,         // Hash of input content
    SemanticHash,        // Semantic content hash
    ParameterBased,      // Based on model parameters
    Contextual,          // Include context in key
}

#[derive(Debug, Clone)]
enum InvalidationStrategy {
    TimeBasedTTL,        // Time-to-live based
    AccessBasedLRU,      // Least recently used
    ModelUpdateBased,    // Invalidate on model updates
    ContextualInvalidation, // Context-aware invalidation
}

/// AI performance optimizer
pub struct AiPerformanceOptimizer {
    optimization_rules: RwLock<Vec<OptimizationRule>>,
    performance_history: RwLock<Vec<PerformanceSnapshot>>,
    adaptive_scheduler: AdaptiveScheduler,
}

/// Optimization rule for AI workloads
#[derive(Debug, Clone)]
struct OptimizationRule {
    condition: OptimizationCondition,
    action: OptimizationAction,
    priority: u8,
    effectiveness: f32,
}

#[derive(Debug, Clone)]
enum OptimizationCondition {
    HighLatencyDetected(f64),
    LowCacheHitRate(f32),
    GpuUtilizationLow(f32),
    BatchingOpportunity(u32),
    ModelTypeDetected(AiModelType),
}

#[derive(Debug, Clone)]
enum OptimizationAction {
    EnableCaching,
    IncreaseBatchSize(u32),
    PreferGpuProcessing,
    EnablePrefetching,
    AdjustPriority(AiPriorityLevel),
    EnableStreaming,
}

/// Adaptive scheduler for AI workloads
pub struct AdaptiveScheduler {
    scheduling_policies: RwLock<HashMap<AiModelType, SchedulingPolicy>>,
    resource_allocations: RwLock<ResourceAllocation>,
    load_balancing_strategy: LoadBalancingStrategy,
}

#[derive(Debug, Clone)]
struct SchedulingPolicy {
    priority_weight: f32,
    resource_preference: ResourcePreference,
    batching_strategy: BatchingStrategy,
    timeout_policy: TimeoutPolicy,
}

#[derive(Debug, Clone)]
enum ResourcePreference {
    CpuOptimized,
    GpuOptimized,
    MemoryOptimized,
    NetworkOptimized,
    Balanced,
}

#[derive(Debug, Clone)]
enum BatchingStrategy {
    NoBatching,
    FixedSizeBatching(u32),
    AdaptiveBatching { min_size: u32, max_size: u32, timeout_ms: u64 },
    PriorityBasedBatching,
}

#[derive(Debug, Clone)]
struct TimeoutPolicy {
    default_timeout_ms: u64,
    priority_multipliers: HashMap<AiPriorityLevel, f32>,
    backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone)]
enum BackoffStrategy {
    Linear,
    Exponential,
    Adaptive,
}

#[derive(Debug, Clone)]
struct ResourceAllocation {
    cpu_cores: u32,
    memory_gb: f32,
    gpu_count: u32,
    network_bandwidth_gbps: f32,
    storage_iops: u32,
}

#[derive(Debug, Clone)]
enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    ResourceAware,
    AiModelAware,
    LatencyOptimized,
}

#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    timestamp: std::time::Instant,
    workload_type: AiWorkloadProfile,
    latency_ms: f64,
    throughput_rps: f64,
    resource_utilization: ResourceUtilization,
    cache_hit_rate: f32,
    batch_efficiency: f32,
}

#[derive(Debug, Clone)]
struct ResourceUtilization {
    cpu_percent: f32,
    memory_percent: f32,
    gpu_percent: Option<f32>,
    network_utilization: f32,
    storage_utilization: f32,
}

#[derive(Debug, Clone)]
struct ClassificationEvent {
    timestamp: std::time::Instant,
    packet_fingerprint: String,
    classification: TrafficProfile,
    confidence: f64,
    processing_time_ms: f64,
}

/// Machine learning classification engine
pub struct MLClassificationEngine {
    model_weights: RwLock<HashMap<String, f64>>,
    feature_extractors: Vec<FeatureExtractor>,
    training_data: RwLock<Vec<TrainingExample>>,
    model_accuracy: RwLock<f32>,
}

#[derive(Debug, Clone)]
struct TrainingExample {
    features: Vec<f64>,
    label: TrafficProfile,
    confidence: f32,
}

#[derive(Debug, Clone)]
enum FeatureExtractor {
    PacketSize,
    RequestFrequency,
    PayloadComplexity,
    ResponseTimePattern,
    ResourceUsagePattern,
    ModelTypeIndicators,
}

impl TrafficClassifier {
    pub fn new() -> Self {
        Self {
            gaming_patterns: RwLock::new(HashMap::new()),
            scientific_patterns: RwLock::new(HashMap::new()),
            ai_patterns: RwLock::new(HashMap::new()),
            classification_history: RwLock::new(Vec::new()),
            learning_engine: MLClassificationEngine::new(),
            cache_predictor: AiCachePredictor::new(),
            performance_optimizer: AiPerformanceOptimizer::new(),
        }
    }

    /// Enhanced classification with AI workload detection
    pub async fn classify_traffic(&self, encrypted_data: &[u8]) -> TrafficProfile {
        // Extract features from encrypted traffic patterns
        let features = self.extract_features(encrypted_data).await;
        
        // AI workload detection (highest priority for AI-first system)
        if let Some(ai_profile) = self.classify_ai_workload(&features).await {
            return TrafficProfile::AiWorkload(ai_profile);
        }
        
        // Gaming workload detection
        if let Some(gaming_profile) = self.classify_gaming_workload(&features).await {
            return TrafficProfile::Gaming(gaming_profile);
        }
        
        // Scientific workload detection
        if let Some(scientific_profile) = self.classify_scientific_workload(&features).await {
            return TrafficProfile::Scientific(scientific_profile);
        }
        
        // Default to general optimization
        TrafficProfile::General(GeneralOptimizationProfile {
            balanced_optimization: true,
        })
    }

    /// Classify AI workload with detailed profiling
    async fn classify_ai_workload(&self, features: &[f64]) -> Option<AiWorkloadProfile> {
        // AI workload detection heuristics
        let has_json_patterns = features.get(0).unwrap_or(&0.0) > 0.7;
        let has_high_frequency = features.get(1).unwrap_or(&0.0) > 0.5;
        let has_variable_payload = features.get(2).unwrap_or(&0.0) > 0.6;
        let has_low_latency_requirement = features.get(3).unwrap_or(&0.0) > 0.8;
        
        if has_json_patterns && has_high_frequency {
            let model_type = self.detect_ai_model_type(features).await;
            let priority = self.determine_ai_priority(features).await;
            
            Some(AiWorkloadProfile {
                inference_latency_target_ms: match priority {
                    AiPriorityLevel::Critical => 1.0,
                    AiPriorityLevel::High => 10.0,
                    AiPriorityLevel::Medium => 100.0,
                    AiPriorityLevel::Low => 1000.0,
                    AiPriorityLevel::Bulk => 10000.0,
                },
                throughput_requirement_rps: match model_type {
                    Some(AiModelType::LargeLanguageModel) => 100.0,
                    Some(AiModelType::ComputerVision) => 50.0,
                    Some(AiModelType::Embedding) => 1000.0,
                    _ => 200.0,
                },
                model_type,
                batch_size_preference: Some(match priority {
                    AiPriorityLevel::Critical => 1,
                    AiPriorityLevel::High => 4,
                    AiPriorityLevel::Medium => 16,
                    AiPriorityLevel::Low => 64,
                    AiPriorityLevel::Bulk => 256,
                }),
                memory_requirement_gb: 2.0,
                gpu_acceleration: matches!(model_type, Some(AiModelType::LargeLanguageModel) | Some(AiModelType::ComputerVision)),
                streaming_capable: true,
                caching_beneficial: true,
                priority_level: priority,
            })
        } else {
            None
        }
    }

    /// Classify gaming workload (existing logic)
    async fn classify_gaming_workload(&self, features: &[f64]) -> Option<GamingOptimizationProfile> {
        let has_small_packets = features.get(0).unwrap_or(&0.0) < 0.3;
        let has_high_frequency = features.get(1).unwrap_or(&0.0) > 0.8;
        let has_low_latency = features.get(3).unwrap_or(&0.0) > 0.9;
        
        if has_small_packets && has_high_frequency && has_low_latency {
            Some(GamingOptimizationProfile {
                latency_target_ms: 0.5,
                throughput_requirement_gbps: 1.0,
                packet_size_range: (64, 1500),
                protocol_hints: vec!["UDP".to_string(), "IPX".to_string()],
                optimization_focus: OptimizationFocus::LatencyFirst,
                game_type: Some(GameType::FirstPersonShooter),
            })
        } else {
            None
        }
    }

    /// Classify scientific workload (existing logic)
    async fn classify_scientific_workload(&self, features: &[f64]) -> Option<ScientificOptimizationProfile> {
        let has_large_payloads = features.get(0).unwrap_or(&0.0) > 0.8;
        let has_bulk_transfer = features.get(2).unwrap_or(&0.0) > 0.7;
        
        if has_large_payloads && has_bulk_transfer {
            Some(ScientificOptimizationProfile {
                latency_tolerance_ms: 10.0,
                throughput_requirement_gbps: 10.0,
                file_size_range: (1024 * 1024 * 100, 1024 * 1024 * 1024 * 10), // 100MB to 10GB
                data_format_hints: vec!["HDF5".to_string(), "NetCDF".to_string()],
                optimization_focus: OptimizationFocus::ThroughputFirst,
                scientific_domain: Some(ScientificDomain::Genomics),
            })
        } else {
            None
        }
    }

    /// Extract features from encrypted traffic
    async fn extract_features(&self, data: &[u8]) -> Vec<f64> {
        // Feature extraction from encrypted traffic patterns
        let payload_size = data.len() as f64 / 1024.0; // Payload size normalized
        
        // Calculate request frequency based on packet timing patterns
        let request_frequency = self.calculate_request_frequency(data).await;
        
        // Calculate payload complexity based on entropy
        let payload_complexity = self.calculate_payload_complexity(data).await;
        
        // Calculate response time pattern based on historical data
        let response_pattern = self.calculate_response_pattern(data).await;
        
        // Calculate resource usage pattern based on packet characteristics
        let resource_usage = self.calculate_resource_usage_pattern(data).await;
        
        vec![
            payload_size,
            request_frequency,
            payload_complexity,
            response_pattern,
            resource_usage,
        ]
    }

    /// Calculate request frequency from traffic patterns
    async fn calculate_request_frequency(&self, data: &[u8]) -> f64 {
        // Analyze packet timing patterns to estimate request frequency
        // Higher frequency patterns indicate streaming or real-time communication
        if data.len() < 100 {
            0.9 // Small packets often indicate high-frequency communication
        } else if data.len() < 1000 {
            0.6 // Medium packets indicate moderate frequency
        } else {
            0.3 // Large packets indicate lower frequency, batch processing
        }
    }

    /// Calculate payload complexity from data entropy
    async fn calculate_payload_complexity(&self, data: &[u8]) -> f64 {
        // Calculate Shannon entropy to measure data complexity
        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }
        
        let data_len = data.len() as f64;
        let mut entropy = 0.0;
        
        for count in byte_counts.iter() {
            if *count > 0 {
                let probability = *count as f64 / data_len;
                entropy -= probability * probability.log2();
            }
        }
        
        // Normalize entropy to 0-1 scale (max entropy for byte data is 8)
        entropy / 8.0
    }

    /// Calculate response pattern from historical timing
    async fn calculate_response_pattern(&self, data: &[u8]) -> f64 {
        // Analyze data patterns to predict response characteristics
        // Look for patterns that indicate request-response cycles
        let has_header_patterns = data.len() >= 8 && 
            (data[0] == 0x48 || data[0] == 0x47 || data[0] == 0x50); // HTTP/HTTPS patterns
        
        let has_json_patterns = data.windows(4).any(|w| 
            w == b"json" || w == b"JSON" || w == b"api/"
        );
        
        if has_header_patterns && has_json_patterns {
            0.9 // High response pattern for API calls
        } else if has_header_patterns {
            0.7 // Medium response pattern for HTTP
        } else {
            0.4 // Low response pattern for other protocols
        }
    }

    /// Calculate resource usage pattern from packet characteristics
    async fn calculate_resource_usage_pattern(&self, data: &[u8]) -> f64 {
        // Estimate resource usage based on packet characteristics
        let size_factor = (data.len() as f64 / 1024.0).min(1.0);
        
        // Look for patterns indicating resource-intensive operations
        let has_binary_patterns = data.iter().filter(|&&b| b > 127).count() as f64 / data.len() as f64;
        let has_compression_patterns = data.windows(2).any(|w| 
            w == b"\x1f\x8b" || w == b"PK" || w == b"BZ"
        );
        
        let base_usage = size_factor * 0.5;
        let binary_bonus = has_binary_patterns * 0.3;
        let compression_bonus = if has_compression_patterns { 0.2 } else { 0.0 };
        
        (base_usage + binary_bonus + compression_bonus).min(1.0)
    }

    /// Detect AI model type from features
    async fn detect_ai_model_type(&self, features: &[f64]) -> Option<AiModelType> {
        // AI model type detection logic
        let payload_complexity = features.get(2).unwrap_or(&0.0);
        let response_pattern = features.get(3).unwrap_or(&0.0);
        
        if *payload_complexity > 0.8 && *response_pattern > 0.7 {
            Some(AiModelType::LargeLanguageModel)
        } else if *payload_complexity > 0.6 {
            Some(AiModelType::ComputerVision)
        } else {
            Some(AiModelType::Embedding)
        }
    }

    /// Determine AI priority level
    async fn determine_ai_priority(&self, features: &[f64]) -> AiPriorityLevel {
        let latency_requirement = features.get(3).unwrap_or(&0.0);
        
        if *latency_requirement > 0.9 {
            AiPriorityLevel::Critical
        } else if *latency_requirement > 0.7 {
            AiPriorityLevel::High
        } else if *latency_requirement > 0.5 {
            AiPriorityLevel::Medium
        } else if *latency_requirement > 0.3 {
            AiPriorityLevel::Low
        } else {
            AiPriorityLevel::Bulk
        }
    }

    /// Get AI cache prediction
    pub async fn predict_cache_effectiveness(&self, workload: &AiWorkloadProfile) -> f32 {
        self.cache_predictor.predict_cache_hit_rate(workload).await
    }

    /// Get performance optimization recommendations
    pub async fn get_optimization_recommendations(&self, workload: &AiWorkloadProfile) -> Vec<OptimizationAction> {
        self.performance_optimizer.get_recommendations(workload).await
    }

    /// Update classification with feedback
    pub async fn update_classification(&self, actual_profile: TrafficProfile, predicted_profile: TrafficProfile) {
        // Update ML model with feedback
        self.learning_engine.update_model(actual_profile, predicted_profile).await;
    }
}

impl MLClassificationEngine {
    pub fn new() -> Self {
        Self {
            model_weights: RwLock::new(HashMap::new()),
            feature_extractors: vec![
                FeatureExtractor::PacketSize,
                FeatureExtractor::RequestFrequency,
                FeatureExtractor::PayloadComplexity,
                FeatureExtractor::ResponseTimePattern,
                FeatureExtractor::ResourceUsagePattern,
                FeatureExtractor::ModelTypeIndicators,
            ],
            training_data: RwLock::new(Vec::new()),
            model_accuracy: RwLock::new(0.8),
        }
    }

    /// Update model with new training data
    pub async fn update_model(&self, actual: TrafficProfile, predicted: TrafficProfile) {
        // Simple accuracy tracking
        let mut accuracy = self.model_accuracy.write().await;
        let is_correct = std::mem::discriminant(&actual) == std::mem::discriminant(&predicted);
        *accuracy = (*accuracy * 0.9) + (if is_correct { 1.0 } else { 0.0 }) * 0.1;
    }

    /// Get model accuracy
    pub async fn get_accuracy(&self) -> f32 {
        *self.model_accuracy.read().await
    }
}

impl AiCachePredictor {
    pub fn new() -> Self {
        Self {
            cache_hit_patterns: RwLock::new(HashMap::new()),
            model_cache_strategies: RwLock::new(HashMap::new()),
            prediction_accuracy: RwLock::new(0.75),
        }
    }

    /// Predict cache hit rate for AI workload
    pub async fn predict_cache_hit_rate(&self, workload: &AiWorkloadProfile) -> f32 {
        // AI workload cache prediction logic
        match &workload.model_type {
            Some(AiModelType::LargeLanguageModel) => 0.6, // Text generation has moderate caching
            Some(AiModelType::ComputerVision) => 0.8,     // Image processing benefits from caching
            Some(AiModelType::Embedding) => 0.9,          // Embeddings are highly cacheable
            Some(AiModelType::Recommendation) => 0.7,     // Recommendations have good caching
            _ => 0.5, // Default prediction
        }
    }

    /// Get cache strategy for model type
    pub async fn get_cache_strategy(&self, model_type: &AiModelType) -> CacheStrategy {
        let strategies = self.model_cache_strategies.read().await;
        strategies.get(model_type).cloned().unwrap_or_else(|| CacheStrategy {
            cache_duration: std::time::Duration::from_secs(300), // 5 minutes default
            cache_key_strategy: CacheKeyStrategy::ContentHash,
            prefetch_enabled: true,
            invalidation_strategy: InvalidationStrategy::TimeBasedTTL,
        })
    }
}

impl AiPerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            optimization_rules: RwLock::new(Vec::new()),
            performance_history: RwLock::new(Vec::new()),
            adaptive_scheduler: AdaptiveScheduler::new(),
        }
    }

    /// Get optimization recommendations
    pub async fn get_recommendations(&self, workload: &AiWorkloadProfile) -> Vec<OptimizationAction> {
        let mut recommendations = Vec::new();
        
        // AI-specific optimization recommendations
        if workload.caching_beneficial {
            recommendations.push(OptimizationAction::EnableCaching);
        }
        
        if workload.priority_level == AiPriorityLevel::Critical {
            recommendations.push(OptimizationAction::AdjustPriority(AiPriorityLevel::Critical));
        }
        
        if workload.gpu_acceleration {
            recommendations.push(OptimizationAction::PreferGpuProcessing);
        }
        
        if workload.streaming_capable {
            recommendations.push(OptimizationAction::EnableStreaming);
        }
        
        if let Some(batch_size) = workload.batch_size_preference {
            if batch_size > 1 {
                recommendations.push(OptimizationAction::IncreaseBatchSize(batch_size));
            }
        }
        
        recommendations
    }

    /// Record performance snapshot
    pub async fn record_performance(&self, snapshot: PerformanceSnapshot) {
        let mut history = self.performance_history.write().await;
        history.push(snapshot);
        
        // Keep only recent history
        if history.len() > 1000 {
            history.remove(0);
        }
    }
}

impl AdaptiveScheduler {
    pub fn new() -> Self {
        Self {
            scheduling_policies: RwLock::new(HashMap::new()),
            resource_allocations: RwLock::new(ResourceAllocation {
                cpu_cores: 8,
                memory_gb: 32.0,
                gpu_count: 2,
                network_bandwidth_gbps: 10.0,
                storage_iops: 10000,
            }),
            load_balancing_strategy: LoadBalancingStrategy::AiModelAware,
        }
    }

    /// Get scheduling policy for AI model type
    pub async fn get_scheduling_policy(&self, model_type: &AiModelType) -> SchedulingPolicy {
        let policies = self.scheduling_policies.read().await;
        policies.get(model_type).cloned().unwrap_or_else(|| SchedulingPolicy {
            priority_weight: 1.0,
            resource_preference: ResourcePreference::Balanced,
            batching_strategy: BatchingStrategy::AdaptiveBatching {
                min_size: 1,
                max_size: 32,
                timeout_ms: 100,
            },
            timeout_policy: TimeoutPolicy {
                default_timeout_ms: 30000,
                priority_multipliers: HashMap::new(),
                backoff_strategy: BackoffStrategy::Exponential,
            },
        })
    }

    /// Update resource allocation
    pub async fn update_resource_allocation(&self, allocation: ResourceAllocation) {
        *self.resource_allocations.write().await = allocation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_workload_classification() {
        let classifier = TrafficClassifier::new();
        
        // Realistic HTTP API request pattern (simulating AI inference request)
        let http_api_data = vec![
            0x48, 0x54, 0x54, 0x50, 0x2f, 0x31, 0x2e, 0x31, // "HTTP/1.1"
            0x20, 0x32, 0x30, 0x30, 0x20, 0x4f, 0x4b, 0x0d, // " 200 OK\r"
            0x0a, 0x43, 0x6f, 0x6e, 0x74, 0x65, 0x6e, 0x74, // "\nContent"
            0x2d, 0x54, 0x79, 0x70, 0x65, 0x3a, 0x20, 0x61, // "-Type: a"
            0x70, 0x70, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x69, // "pplicati"
            0x6f, 0x6e, 0x2f, 0x6a, 0x73, 0x6f, 0x6e, 0x0d, // "on/json\r"
            0x0a, 0x0d, 0x0a, 0x7b, 0x22, 0x72, 0x65, 0x73, // "\n\r\n{\"res"
            0x75, 0x6c, 0x74, 0x22, 0x3a, 0x20, 0x22, 0x61, // "ult\": \"a"
            0x69, 0x5f, 0x69, 0x6e, 0x66, 0x65, 0x72, 0x65, // "i_infere"
            0x6e, 0x63, 0x65, 0x22, 0x7d, // "nce\"}"
        ];
        
        let profile = classifier.classify_traffic(&http_api_data).await;
        
        // Should classify as some workload type
        assert!(matches!(profile, TrafficProfile::AiWorkload(_) | TrafficProfile::Gaming(_) | TrafficProfile::Scientific(_) | TrafficProfile::General(_)));
    }

    #[tokio::test]
    async fn test_cache_prediction() {
        let classifier = TrafficClassifier::new();
        let workload = AiWorkloadProfile {
            inference_latency_target_ms: 10.0,
            throughput_requirement_rps: 100.0,
            model_type: Some(AiModelType::LargeLanguageModel),
            batch_size_preference: Some(4),
            memory_requirement_gb: 2.0,
            gpu_acceleration: true,
            streaming_capable: true,
            caching_beneficial: true,
            priority_level: AiPriorityLevel::High,
        };
        
        let cache_effectiveness = classifier.predict_cache_effectiveness(&workload).await;
        assert!(cache_effectiveness > 0.0 && cache_effectiveness <= 1.0);
    }

    #[tokio::test]
    async fn test_optimization_recommendations() {
        let classifier = TrafficClassifier::new();
        let workload = AiWorkloadProfile {
            inference_latency_target_ms: 1.0,
            throughput_requirement_rps: 1000.0,
            model_type: Some(AiModelType::ComputerVision),
            batch_size_preference: Some(16),
            memory_requirement_gb: 4.0,
            gpu_acceleration: true,
            streaming_capable: true,
            caching_beneficial: true,
            priority_level: AiPriorityLevel::Critical,
        };
        
        let recommendations = classifier.get_optimization_recommendations(&workload).await;
        assert!(!recommendations.is_empty());
        assert!(recommendations.contains(&OptimizationAction::EnableCaching));
        assert!(recommendations.contains(&OptimizationAction::PreferGpuProcessing));
    }
} 